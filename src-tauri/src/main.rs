// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use souvlaki::{MediaControlEvent, MediaControls};
use specta::Type;

use std::fs::create_dir;
use std::path::Path;
use std::sync::Mutex;

use specta_typescript::Typescript;
use tauri::{Emitter, Manager};
use tauri_plugin_fs::FsExt;
use tauri_specta::{collect_commands, Builder};

mod commands;
mod db;
mod models;
mod player;

use commands::metadata::*;
use commands::music_folder::*;
use commands::player::*;
use commands::sqlite::*;

pub struct SodapopState {
    pub player: player::Player,
    pub db: db::Database,
    pub controls: MediaControls,
}

#[derive(Type, Serialize, Clone)]
pub enum MediaPayload {
    Play(bool),
    Pause(bool),
    Next(bool),
    Previous(bool),
    Volume(f64),   // Volume as f64 (0.0 - 1.0)
    Seek(f64),     // Duration as f64 (e.g., in seconds)
    Position(f64), // Position in seconds
}

#[tokio::main]
async fn main() {
    let builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            select_music_folder,
            get_album_with_tracks,
            get_artist_with_albums,
            get_all_albums,
            track_by_id,
            play_track,
            pause_track,
            resume_track,
            seek_track,
            set_volume,
            get_player_state,
            player_has_track,
            get_player_progress,
            get_player_duration,
            stop_player,
            update_progress,
            initialize_player,
            set_player_progress,
            player_has_ended,
            get_features,
        ])
        .typ::<MediaPayload>();

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
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

            let config = souvlaki::PlatformConfig {
                dbus_name: "com.sodapop.reimagined.dbus",
                display_name: "Sodapop Reimagined",
                hwnd,
            };

            app.manage(Mutex::new(SodapopState {
                player: player::Player::new(),
                db: db::Database::new(),
                controls: MediaControls::new(config)?,
            }));

            let state = app.state::<Mutex<SodapopState>>();
            let mut state_guard = state.lock().unwrap();

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

            let data_path = db::data_path();

            let covers = data_path.clone() + "/covers";
            if !Path::new(&covers).exists() {
                create_dir(covers).expect("Error creating covers directory")
            }

            let scope = app.fs_scope();
            if let Err(e) = scope.allow_directory(data_path, true) {
                eprintln!("Error allowing directory: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
