use logging::{lock_or_log, try_with_log};
use std::{
    env,
    fs::create_dir,
    sync::{Arc, Mutex, RwLock},
};
use tauri::{AppHandle, State};
use tauri_specta::Event;
use tokio::sync::Notify;

use crate::{
    config::VeilConfig,
    discord::DiscordState,
    error::FrontendError,
    queue::QueueSystem,
    systems::{player::PlayerEvent, utils::data_path},
};

pub struct VeilState {
    pub player: Arc<RwLock<media_controls::DefaultPlayer>>,
    pub queue: Arc<Mutex<QueueSystem>>,
    pub db: Arc<db::Database>,
    pub discord: Mutex<DiscordState>,
    pub config: Arc<RwLock<VeilConfig>>,
    pub lastfm: Arc<tokio::sync::Mutex<lastfm::LastFM>>,
    pub resume_notify: Arc<Notify>,
}

pub type TauriState<'a> = State<'a, VeilState>;

pub fn initialize_state(app: &mut tauri::App) -> Result<VeilState, FrontendError> {
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        use raw_window_handle::HasWindowHandle;
        use tauri::Manager;
        if let Some(main_window) = app.get_webview_window("veil")
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
        create_dir(&path).expect("Error creating main data directory");
    }

    let platform_config = media_controls::PlatformConfig {
        dbus_name: "com.veil.dbus",
        display_name: "VEIL",
        hwnd,
    };

    let veil_config = try_with_log!("VEIL Config", VeilConfig::new)?;

    #[cfg(debug_assertions)]
    let api_key = env::var("LASTFM_API_KEY").expect("Missing LASTFM_API_KEY environment variable");

    #[cfg(not(debug_assertions))]
    let api_key = env!("LASTFM_API_KEY");

    #[cfg(debug_assertions)]
    let api_secret =
        env::var("LASTFM_API_SECRET").expect("Missing LASTFM_API_SECRET environment variable");

    #[cfg(not(debug_assertions))]
    let api_secret = env!("LASTFM_API_SECRET");

    let mut lastfm = try_with_log!("LastFM API", || lastfm::LastFM::builder()
        .api_key(&api_key)
        .api_secret(&api_secret)
        .build())?;

    #[cfg(debug_assertions)]
    let discord_client_id =
        env::var("DISCORD_CLIENT_ID").expect("Missing DISCORD_CLIENT_ID environment variable");

    #[cfg(not(debug_assertions))]
    let discord_client_id = env!("DISCORD_CLIENT_ID");

    let mut discord = DiscordState::new(&discord_client_id);

    lastfm.enable(veil_config.integrations.last_fm_enabled);
    discord.enable(veil_config.integrations.discord_enabled);

    if let Some(session_key) = veil_config.integrations.last_fm_session_key.clone() {
        lastfm.set_session_key(session_key);
    }

    let player = try_with_log!("Music Player", || {
        media_controls::Player::new(platform_config)
    })?;

    Ok(VeilState {
        player: Arc::new(RwLock::new(player)),
        queue: Arc::new(Mutex::new(QueueSystem::new(
            veil_config.playback.queue_origin,
            veil_config.playback.repeat_mode,
        ))),
        db: Arc::new(db::Database::new(path.clone())),
        lastfm: Arc::new(tokio::sync::Mutex::new(lastfm)),
        config: Arc::new(RwLock::new(veil_config)),
        discord: Mutex::new(discord),
        resume_notify: Arc::new(Notify::new()),
    })
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
            MediaControlEvent::Next => PlayerEvent::emit(&PlayerEvent::NextTrackInQueue, &handle),
            MediaControlEvent::Previous => {
                PlayerEvent::emit(&PlayerEvent::PreviousTrackInQueue, &handle)
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
