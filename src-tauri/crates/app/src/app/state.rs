use logging::{lock_or_log, try_with_log};
use serde::Serialize;
use specta::Type;
use std::{
    env,
    fs::create_dir,
    sync::{Arc, Mutex, RwLock},
};
use tauri::{AppHandle, Emitter, State};
use tauri_specta::Event;

use crate::{
    config::SodapopConfig,
    discord::DiscordState,
    error::FrontendError,
    queue::QueueSystem,
    systems::{player::PlayerEvent, utils::data_path},
};

pub struct SodapopState {
    pub player: Arc<RwLock<media_controls::Player>>,
    pub queue: Arc<Mutex<QueueSystem>>,
    pub db: Arc<db::Database>,
    pub discord: Mutex<DiscordState>,
    pub config: Arc<RwLock<SodapopConfig>>,
    pub lastfm: Arc<tokio::sync::Mutex<lastfm::LastFM>>,
}

pub type TauriState<'a> = State<'a, SodapopState>;

pub fn initialize_state() -> Result<SodapopState, FrontendError> {
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        if let Some(main_window) = app.get_webview_window("sodapop-reimagined")
            && let Ok(window_handle) = main_window.window_handle()
            && let raw_window_handle::RawWindowHandle::Win32(handle) = window_handle.as_raw()
        {
            Some(handle.hwnd.get() as *mut std::ffi::c_void)
        } else {
            None
        }
    };

    let path = data_path();

    if !path.exists() {
        create_dir(&path).expect("Error creating main data directory")
    }

    let platform_config = media_controls::PlatformConfig {
        dbus_name: "com.sodapop.reimagined.dbus",
        display_name: "Sodapop Reimagined",
        hwnd,
    };

    let sodapop_config = try_with_log!("Sodapop Config", SodapopConfig::new)?;

    let api_key = env::var("LASTFM_API_KEY").expect("Missing LASTFM_API_KEY environment variable");
    let api_secret =
        env::var("LASTFM_API_SECRET").expect("Missing LASTFM_API_SECRET environment variable");

    let mut lastfm = try_with_log!("LastFM API", || lastfm::LastFM::builder()
        .api_key(&api_key)
        .api_secret(&api_secret)
        .build())?;

    let discord_client_id =
        env::var("DISCORD_CLIENT_ID").expect("Missing DISCORD_CLIENT_ID environment variable");

    let mut discord = try_with_log!("Discord RPC", || DiscordState::new(&discord_client_id))?;

    lastfm.enable(sodapop_config.last_fm_enabled);
    discord.enable(sodapop_config.discord_enabled);

    if let Some(sk) = sodapop_config.last_fm_key.clone() {
        lastfm.set_session_key(sk);
    }

    let player = try_with_log!("Music Player", || media_controls::Player::new(
        platform_config
    ))?;

    Ok(SodapopState {
        player: Arc::new(RwLock::new(player)),
        queue: Arc::new(Mutex::new(QueueSystem::new(
            0x12345678,
            sodapop_config.queue_origin,
            sodapop_config.repeat_mode,
        ))),
        db: Arc::new(db::Database::new(path.clone())),
        lastfm: Arc::new(tokio::sync::Mutex::new(lastfm)),
        config: Arc::new(RwLock::new(sodapop_config)),
        discord: Mutex::new(discord),
    })
}

#[derive(Type, Serialize, Clone)]
pub enum MediaPayload {
    Next(bool),
    Previous(bool),
}

pub fn attach_media_controls_to_player(
    handle: &AppHandle,
    state: &TauriState,
) -> Result<(), anyhow::Error> {
    let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;

    let handle = handle.clone();
    player.controls.attach(move |event| {
        use media_controls::MediaControlEvent;

        let result = match event {
            MediaControlEvent::Play => PlayerEvent::emit(&PlayerEvent::Resume, &handle),
            MediaControlEvent::Pause => PlayerEvent::emit(&PlayerEvent::Pause, &handle),
            MediaControlEvent::Next => handle.emit("media-control", MediaPayload::Next(false)),
            MediaControlEvent::Previous => {
                handle.emit("media-control", MediaPayload::Previous(false))
            }
            MediaControlEvent::SetVolume(volume) => PlayerEvent::emit(
                &PlayerEvent::SetVolume {
                    volume: volume as f32,
                },
                &handle,
            ),
            MediaControlEvent::SeekBy(direction, duration) => {
                let sign = match direction {
                    media_controls::SeekDirection::Forward => 1.0,
                    media_controls::SeekDirection::Backward => -1.0,
                };
                let secs = duration.as_secs_f64() * sign;
                PlayerEvent::emit(
                    &PlayerEvent::Seek {
                        position: secs,
                        resume: true,
                    },
                    &handle,
                )
            }
            MediaControlEvent::SetPosition(position) => PlayerEvent::emit(
                &PlayerEvent::Seek {
                    position: position.0.as_secs_f64(),
                    resume: false,
                },
                &handle,
            ),
            _ => Ok(()),
        };

        match result {
            Ok(_) => (),
            Err(e) => {
                logging::error!("Media control event got an error during emit: {e}");
            }
        }
    })?;

    Ok(())
}
