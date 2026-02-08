use gpui::{App, Global};
use logging::{lock_or_log, try_with_log};
use std::{
    env,
    fs::create_dir,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::Notify;

use crate::{
    config::{VeilConfig, VeilConfigEvent},
    discord::DiscordState,
    error::FrontendError,
    events::EventBus,
    queue::{QueueEvent, QueueSystem},
    systems::{player::PlayerEvent, ui::UIUpdateEvent, utils::data_path},
};

pub struct VeilState {
    pub player: Arc<RwLock<media_controls::DefaultPlayer>>,
    pub queue: Arc<Mutex<QueueSystem>>,
    pub db: Arc<db::Database>,
    pub discord: Mutex<DiscordState>,
    pub config: Arc<RwLock<VeilConfig>>,
    pub lastfm: Arc<tokio::sync::Mutex<lastfm::LastFM>>,
    pub resume_notify: Arc<Notify>,

    pub player_bus: EventBus<PlayerEvent>,
    pub ui_bus: EventBus<UIUpdateEvent>,
    pub config_bus: EventBus<VeilConfigEvent>,
    pub queue_bus: EventBus<QueueEvent>,
}

pub struct AppState(pub Arc<VeilState>);

impl Global for AppState {}

pub fn initialize_state(_cx: &mut App) -> Result<VeilState, FrontendError> {
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        // TODO: untested, will probably fail
        if let Some(handle) = _cx.windows().get(0) {
            handle.get_raw_handle()
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

        player_bus: EventBus::new(128),
        ui_bus: EventBus::new(16),
        config_bus: EventBus::new(16),
        queue_bus: EventBus::new(16),
    })
}

// TODO: MIGRATE TO EVENT MANAGER FOR GPUI
pub fn attach_media_controls_to_player(state: Arc<VeilState>) -> Result<(), anyhow::Error> {
    let mut player = lock_or_log(state.player.write(), "Player Write Lock")?;

    let state = state.clone();
    player.controls.attach(move |event| {
        use media_controls::MediaControlEvent;

        match event {
            MediaControlEvent::Play => state.player_bus.emit(PlayerEvent::Resume),
            MediaControlEvent::Pause => state.player_bus.emit(PlayerEvent::Pause),
            MediaControlEvent::Next => state.player_bus.emit(PlayerEvent::NextTrackInQueue),
            MediaControlEvent::Previous => state.player_bus.emit(PlayerEvent::PreviousTrackInQueue),
            MediaControlEvent::SetVolume(volume) => state.player_bus.emit(PlayerEvent::SetVolume {
                volume: volume as f32,
            }),
            MediaControlEvent::SeekBy(direction, duration) => {
                let sign = match direction {
                    media_controls::SeekDirection::Forward => 1.0,
                    media_controls::SeekDirection::Backward => -1.0,
                };
                let secs = duration.as_secs_f64() * sign;
                state.player_bus.emit(PlayerEvent::Seek {
                    position: secs,
                    resume: true,
                });
            }
            MediaControlEvent::SetPosition(position) => state.player_bus.emit(PlayerEvent::Seek {
                position: position.0.as_secs_f64(),
                resume: false,
            }),
            _ => (),
        };
    })?;

    Ok(())
}
