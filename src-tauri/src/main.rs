// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use commands::{discord::DiscordState, player::progress_as_position};
use config::{SodapopConfig, SodapopConfigEvent};
use discord_rich_presence::DiscordIpc;
use serde::Serialize;
use souvlaki::{MediaControlEvent, MediaControls, MediaPlayback};
use specta::Type;

use std::sync::Mutex;
use std::{fs::create_dir, path::PathBuf};

#[cfg(debug_assertions)]
use specta_typescript::Typescript;

use tauri::{Emitter, Manager, RunEvent};
use tauri_specta::{collect_commands, collect_events, Builder, Event};

mod commands;
mod config;
mod error;
mod player;

pub struct SodapopState {
    pub player: player::Player,
    pub db: db::Database,
    pub controls: MediaControls,
    pub discord: DiscordState,
    pub config: SodapopConfig,
}

#[derive(Type, Serialize, Clone)]
pub enum MediaPayload {
    Play(bool),
    Pause(bool),
    Next(bool),
    Previous(bool),
    /// Volume as f64 (0.0 - 1.0)
    Volume(f64),
    /// Duration as f64 (e.g., in seconds)
    Seek(f64),
    /// Position in seconds
    Position(f64),
}

fn main() {
    let builder = Builder::<tauri::Wry>::new()
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
            commands::config::get_config,
        ])
        .events(collect_events![
            commands::music_folder::MusicDataEvent,
            SodapopConfigEvent
        ])
        .typ::<MediaPayload>()
        .typ::<SodapopConfig>();

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::default()
                .formatter(specta_typescript::formatter::prettier)
                .header("// @ts-nocheck"),
            "../src/bindings.ts",
        )
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            builder.mount_events(app);

            #[cfg(not(target_os = "windows"))]
            let hwnd = None;

            #[cfg(target_os = "windows")]
            let hwnd = {
                let main_window = app.get_webview_window("main").unwrap();
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

            let config = souvlaki::PlatformConfig {
                dbus_name: "com.sodapop.reimagined.dbus",
                display_name: "Sodapop Reimagined",
                hwnd,
            };

            app.manage(Mutex::new(SodapopState {
                player: player::Player::new(),
                db: db::Database::new(path.clone()),
                controls: MediaControls::new(config)?,
                config: SodapopConfig::new().expect("error making config"),
                discord: DiscordState::new("1339694314074275882")?,
            }));

            let state = app.state::<Mutex<SodapopState>>();
            let mut state_guard = state.lock().unwrap();

            if state_guard.config.discord_enabled {
                state_guard.discord.rpc.connect()?;
                state_guard.discord.enabled = true;
            }

            let handle = app.handle().clone();
            state_guard
                .controls
                .attach(move |event| match event {
                    MediaControlEvent::Play => {
                        handle
                            .emit("media-control", MediaPayload::Play(false))
                            .unwrap();
                    }
                    MediaControlEvent::Pause => {
                        handle
                            .emit("media-control", MediaPayload::Pause(false))
                            .unwrap();
                    }
                    MediaControlEvent::Next => {
                        handle
                            .emit("media-control", MediaPayload::Next(false))
                            .unwrap();
                    }
                    MediaControlEvent::Previous => {
                        handle
                            .emit("media-control", MediaPayload::Previous(false))
                            .unwrap();
                    }
                    MediaControlEvent::SetVolume(value) => {
                        handle
                            .emit("media-control", MediaPayload::Volume(value))
                            .unwrap();
                    }
                    MediaControlEvent::SeekBy(direction, duration) => {
                        let duration = match direction {
                            souvlaki::SeekDirection::Forward => duration.as_secs_f64(),
                            souvlaki::SeekDirection::Backward => -(duration.as_secs_f64()),
                        };
                        handle
                            .emit("media-control", MediaPayload::Seek(duration))
                            .unwrap();
                    }
                    MediaControlEvent::SetPosition(position) => {
                        handle
                            .emit(
                                "media-control",
                                MediaPayload::Position(position.0.as_secs_f64()),
                            )
                            .unwrap();
                    }
                    _ => (),
                })
                .unwrap();

            let app_handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_millis(50));
                // get the player state
                let state = app_handle.state::<Mutex<SodapopState>>();
                let mut state_guard = state.lock().unwrap();

                let progress = state_guard.player.progress;

                if let player::PlayerState::Playing = state_guard.player.state {
                    state_guard.player.update();
                    app_handle.emit("player-progress", progress).unwrap();

                    state_guard
                        .controls
                        .set_playback(MediaPlayback::Playing {
                            progress: progress_as_position(progress),
                        })
                        .unwrap();

                    if progress >= (state_guard.player.duration - 0.05) as f64 {
                        app_handle.emit("track-end", 0.0).unwrap();
                    };
                }
            });

            let app_handle = app.handle().clone();
            SodapopConfigEvent::listen(app, move |event| {
                let state = app_handle.state::<Mutex<SodapopState>>();
                let mut state_guard = state.lock().unwrap();
                if let Some(d) = event.payload.discord_enabled {
                    if d {
                        state_guard.discord.rpc.connect().unwrap();
                        state_guard.discord.enabled = true;
                        let curr_payload = state_guard.discord.payload.clone();
                        state_guard.discord.make_activity(curr_payload).unwrap();
                    } else {
                        state_guard.discord.rpc.close().unwrap();
                        state_guard.discord.enabled = false;
                    }
                }
                state_guard.config.update_config(event.payload).unwrap();
            });

            let covers = path.join("covers");
            if !covers.exists() {
                create_dir(covers).expect("Error creating covers directory")
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {
            if let RunEvent::ExitRequested { .. } = _event {
                let state = _app.state::<Mutex<SodapopState>>();
                let mut state_guard = state.lock().unwrap();

                if state_guard.config.discord_enabled {
                    state_guard.discord.rpc.close().unwrap();
                };

                state_guard.db.shutdown().unwrap();
            }
        });
}

pub fn data_path() -> PathBuf {
    let home_dir = dirs::data_local_dir().unwrap();
    home_dir.join("Sodapop-Reimagined")
}
