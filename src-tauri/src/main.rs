// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::anyhow;
use config::{SodapopConfig, SodapopConfigEvent};
use discord::PayloadData;
use discord_rich_presence::DiscordIpc;
use serde::Serialize;
use specta::Type;
use specta_typescript::BigIntExportBehavior;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use std::{fs::File, fs::create_dir, path::PathBuf};
use tauri::{Emitter, Manager, RunEvent, State};
use tauri_specta::{Builder, Event, collect_commands, collect_events};
use tracing::{error, info};

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
    tracing_subscriber::fmt::init();

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
            "../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);

            #[cfg(not(target_os = "windows"))]
            let hwnd = None;

            #[cfg(target_os = "windows")]
            let hwnd = {
                let main_window = app.get_webview_window("sodapop-reimagined").unwrap();
                let window_handle = main_window.window_handle().unwrap();

                match window_handle.as_raw() {
                    RawWindowHandle::Win32(handle) => {
                        Some(handle.hwnd.get() as *mut std::ffi::c_void)
                    }
                    _ => panic!("Failed to get a Win32 HWND! Are you running on Windows?"),
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
                let sodapop_config = match SodapopConfig::new() {
                    Ok(cfg) => {
                        info!("Loaded Sodapop config");
                        cfg
                    }
                    Err(e) => {
                        error!("Error loading Sodapop config: {e}");
                        return Err(anyhow!("Config init failed: {e}").into());
                    }
                };

                let mut lastfm = match lastfm::LastFM::builder()
                    .api_key("abc01a1c2188ad44508b12229563de11")
                    .api_secret("e2cbf26c15d7cabc5e72d34bc6d7829c")
                    .build()
                {
                    Ok(lfm) => {
                        info!("Started LastFM API");
                        lfm
                    }
                    Err(e) => {
                        error!("Error starting LastFM API: {e}");
                        return Err(anyhow!("LastFM start failed: {e}").into());
                    }
                };

                let mut discord = match discord::DiscordState::new("1339694314074275882") {
                    Ok(d) => {
                        info!("Started Discord RPC connection");
                        d
                    }
                    Err(e) => {
                        error!("Error starting Discord RPC connection: {e}");
                        return Err(anyhow!("Discord RPC start failed: {e}").into());
                    }
                };

                lastfm.enable(sodapop_config.last_fm_enabled);
                discord.enable(sodapop_config.discord_enabled);

                if let Some(sk) = sodapop_config.last_fm_key.clone() {
                    lastfm.set_session_key(sk);
                }

                app.manage(SodapopState {
                    player: Mutex::new(media_controls::Player::new(platform_config)),
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
                discord.rpc.connect()?;
                discord.enabled = true;
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

            std::thread::spawn(move || {
                let duration = std::time::Duration::from_millis(50);
                loop {
                    std::thread::sleep(duration);
                    // get the player state
                    let state = app_handle.state::<SodapopState>();
                    let player = state.player.lock().unwrap();

                    if let media_controls::PlayerState::Playing = player.state {
                        let progress = player.get_progress();
                        app_handle.emit("player-progress", progress).unwrap();

                        if progress >= (player.duration - 0.05) as f64 {
                            app_handle.emit("track-end", 0.0).unwrap();
                        };
                    }
                }
            });

            let app_handle = app.handle().clone();
            SodapopConfigEvent::listen(app, move |event| {
                let state = app_handle.state::<SodapopState>();
                if let Some(d) = event.payload.discord_enabled {
                    let mut discord = state.discord.lock().unwrap();
                    let player = state.player.lock().unwrap();

                    if d {
                        discord.rpc.connect().unwrap();
                        discord.enabled = true;
                        let curr_payload = PayloadData {
                            progress: player.progress,
                            ..discord.payload.clone()
                        };
                        discord.make_activity(curr_payload).unwrap();
                    } else {
                        discord.rpc.close().unwrap();
                        discord.enabled = false;
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

                let mut config = state.config.write().unwrap();
                config.update_config(event.payload).unwrap();
            });

            let covers = path.join("covers");
            if !covers.exists() {
                create_dir(&covers).expect("Error creating covers directory");
                let pc = include_bytes!("../../public/placeholder.png");
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
                let config = state.config.read().unwrap();
                let mut discord = state.discord.lock().unwrap();

                if config.discord_enabled {
                    discord.rpc.close().unwrap();
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

fn lock_or_log<T>(
    guard: Result<T, std::sync::PoisonError<T>>,
    lock_name: &str,
) -> Result<T, anyhow::Error> {
    match guard {
        Ok(g) => Ok(g),
        Err(poisoned) => {
            error!("{} lock poisoned: {:?}", lock_name, poisoned);
            anyhow::bail!("{} lock poisoned: {:?}", lock_name, poisoned);
        }
    }
}
