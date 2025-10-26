// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use config::{SodapopConfig, SodapopConfigEvent};
use logging::{error, lock_or_log, try_with_log};
use serde::Serialize;
use specta::Type;
use specta_typescript::BigIntExportBehavior;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use std::{fs::File, fs::create_dir, path::PathBuf};
use tauri::{Emitter, Manager, RunEvent, State};
use tauri_specta::{Builder, Event, collect_commands, collect_events};

#[cfg(target_os = "windows")]
use raw_window_handle::HasWindowHandle;

#[cfg(debug_assertions)]
use specta_typescript::Typescript;

mod commands;
mod config;
mod discord;
mod error;
pub struct SodapopState {
    pub player: Mutex<media_controls::Player>,
    pub db: Arc<db::Database>,
    pub discord: Mutex<discord::DiscordState>,
    pub config: Arc<RwLock<SodapopConfig>>,
    pub lastfm: Arc<tokio::sync::Mutex<lastfm::LastFM>>,
}

pub type TauriState<'a> = State<'a, SodapopState>;

#[derive(Type, Serialize, Clone)]
pub enum MediaPayload {
    Play(bool),
    Pause(bool),
    Next(bool),
    Previous(bool),
    /// Volume as f64 (0.0 - 1.0)
    Volume(f64),
    /// Duration as f64
    Seek(f64),
    /// Position in seconds
    Position(f64),
}

fn main() -> anyhow::Result<()> {
    logging::init();

    // TODO: maybe easier way to pass in commands than typing all by hand?
    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::music_folder::select_music_folder,
            commands::sqlite::get_album_with_tracks,
            commands::sqlite::get_artist_with_albums,
            commands::sqlite::get_all_albums,
            commands::sqlite::track_by_id,
            commands::sqlite::new_playlist,
            commands::sqlite::get_all_playlists,
            commands::sqlite::add_to_playlist,
            commands::sqlite::get_playlist_tracks,
            commands::sqlite::remove_from_playlist,
            commands::sqlite::search_db,
            commands::sqlite::get_albums_offset,
            commands::sqlite::get_total_albums,
            commands::sqlite::get_batch_track,
            commands::player::play_track,
            commands::player::pause_track,
            commands::player::resume_track,
            commands::player::seek_track,
            commands::player::set_volume,
            commands::player::get_player_state,
            commands::player::player_has_track,
            commands::player::get_player_progress,
            commands::player::get_player_duration,
            commands::player::stop_player,
            commands::player::initialize_player,
            commands::player::set_player_progress,
            commands::player::player_has_ended,
            commands::lastfm::get_token,
            commands::lastfm::get_session,
            commands::read_custom_style,
            commands::read_config,
            commands::plugins::open_url,
        ])
        .events(collect_events![SodapopConfigEvent])
        .typ::<MediaPayload>()
        .typ::<SodapopConfig>();

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    specta_builder
        .export(
            Typescript::default()
                .formatter(specta_typescript::formatter::prettier)
                .bigint(BigIntExportBehavior::Number)
                .header("// @ts-nocheck"),
            "../../../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);

            #[cfg(not(target_os = "windows"))]
            let hwnd = None;

            #[cfg(target_os = "windows")]
            let hwnd = {
                if let Some(main_window) = app.get_webview_window("sodapop-reimagined")
                    && let Ok(window_handle) = main_window.window_handle()
                    && let raw_window_handle::RawWindowHandle::Win32(handle) =
                        window_handle.as_raw()
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

            {
                let sodapop_config = try_with_log!("Sodapop Config", SodapopConfig::new)?;

                let mut lastfm = try_with_log!("LastFM API", || lastfm::LastFM::builder()
                    .api_key("abc01a1c2188ad44508b12229563de11")
                    .api_secret("e2cbf26c15d7cabc5e72d34bc6d7829c")
                    .build())?;

                let mut discord = try_with_log!("Discord RPC", || discord::DiscordState::new(
                    "1339694314074275882"
                ))?;

                lastfm.enable(sodapop_config.last_fm_enabled);
                discord.enable(sodapop_config.discord_enabled);

                if let Some(sk) = sodapop_config.last_fm_key.clone() {
                    lastfm.set_session_key(sk);
                }

                let player = try_with_log!("Music Player", || media_controls::Player::new(
                    platform_config
                ))?;

                app.manage(SodapopState {
                    player: Mutex::new(player),
                    db: Arc::new(db::Database::new(path.clone())),
                    lastfm: Arc::new(tokio::sync::Mutex::new(lastfm)),
                    config: Arc::new(RwLock::new(sodapop_config)),
                    discord: Mutex::new(discord),
                });
            }

            let state = app.state::<SodapopState>();

            let config = lock_or_log(state.config.read(), "Config RwLock")?;
            if config.discord_enabled {
                let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
                discord.connect();
            }

            let handle = app.handle().clone();
            let mut player = lock_or_log(state.player.lock(), "Player Mutex")?;
            player.controls.attach(move |event| {
                use media_controls::MediaControlEvent;

                let result = match event {
                    MediaControlEvent::Play => {
                        handle.emit("media-control", MediaPayload::Play(false))
                    }
                    MediaControlEvent::Pause => {
                        handle.emit("media-control", MediaPayload::Pause(false))
                    }
                    MediaControlEvent::Next => {
                        handle.emit("media-control", MediaPayload::Next(false))
                    }
                    MediaControlEvent::Previous => {
                        handle.emit("media-control", MediaPayload::Previous(false))
                    }
                    MediaControlEvent::SetVolume(value) => {
                        handle.emit("media-control", MediaPayload::Volume(value))
                    }
                    MediaControlEvent::SeekBy(direction, duration) => {
                        let sign = match direction {
                            media_controls::SeekDirection::Forward => 1.0,
                            media_controls::SeekDirection::Backward => -1.0,
                        };
                        let secs = duration.as_secs_f64() * sign;
                        handle.emit("media-control", MediaPayload::Seek(secs))
                    }
                    MediaControlEvent::SetPosition(position) => handle.emit(
                        "media-control",
                        MediaPayload::Position(position.0.as_secs_f64()),
                    ),
                    _ => Ok(()),
                };

                match result {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Media control event got an error during emit: {e}");
                    }
                }
            })?;

            let app_handle = app.handle().clone();

            // TODO: Is there a better way to handle this other than constant polling & the progress rounding issue?
            std::thread::spawn(move || {
                let duration = std::time::Duration::from_millis(50);
                let state = app_handle.state::<SodapopState>();

                loop {
                    std::thread::sleep(duration);
                    let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();

                    if let media_controls::PlayerState::Playing = player.state {
                        let progress = player.get_progress();
                        app_handle.emit("player-progress", progress).unwrap();

                        if let Some(player_state) = player.get_player_state() {
                            if player_state == media_controls::PlaybackState::Stopped
                                || player_state == media_controls::PlaybackState::Stopping
                            {
                                app_handle.emit("track-end", 0.0).unwrap();
                            }
                        }
                    }
                }
            });

            let app_handle: tauri::AppHandle = app.handle().clone();
            SodapopConfigEvent::listen(app, move |event| {
                let state = app_handle.state::<SodapopState>();
                if let Some(discord_enabled) = event.payload.discord_enabled {
                    let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex").unwrap();
                    let player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();

                    if discord_enabled {
                        if discord.connect() {
                            discord.update_activity_progress(player.progress);
                        }
                    } else {
                        discord.close();
                    }
                }

                let app_handle = app_handle.clone();
                tokio::spawn(async move {
                    if let Some(l) = event.payload.last_fm_enabled {
                        let state = app_handle.state::<SodapopState>();
                        let mut lastfm = state.lastfm.lock().await;
                        lastfm.enable(l);
                    };
                });

                let mut config = lock_or_log(state.config.write(), "Config Write").unwrap();
                config.update_config(event.payload).unwrap();
            });

            let covers = path.join("covers");
            if !covers.exists() {
                create_dir(&covers).expect("Error creating covers directory");
                let pc = include_bytes!("../../../../public/placeholder.png");
                let mut file = File::create(covers.join("placeholder.png"))?;
                file.write_all(pc)?;
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {
            if let RunEvent::ExitRequested { .. } = _event {
                let state = _app.state::<SodapopState>();
                let config = lock_or_log(state.config.read(), "Config Lock").unwrap();
                let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex").unwrap();

                if config.discord_enabled {
                    discord.close();
                };

                state.db.shutdown().unwrap();
            }
        });

    Ok(())
}

pub fn data_path() -> PathBuf {
    let home_dir = dirs::data_local_dir().unwrap();
    home_dir.join("com.sodapop.reimagined")
}
