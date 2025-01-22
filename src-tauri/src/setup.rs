use commands::music_folder::*;
use commands::player::*;
use commands::sqlite::*;

use souvlaki::MediaControlEvent;
use souvlaki::MediaControls;
use specta_typescript::Typescript;
use tauri_plugin_fs::FsExt;
use tauri_specta::collect_commands;
use tauri_specta::Builder;

use std::error::Error;
use std::fs::create_dir;
use std::path::Path;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

use crate::commands;
use crate::db;
use crate::player;
use crate::SodapopState;

pub fn get_builder() -> Builder<tauri::Wry> {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
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
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    builder
}

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        let main_window = app.get_webview_window("main").unwrap();
        let window_handle = main_window.window_handle().unwrap();

        match window_handle.as_raw() {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as *mut std::ffi::c_void),
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
                handle.emit("media-control", "play").unwrap();
            }
            MediaControlEvent::Pause => {
                handle.emit("media-control", "pause").unwrap();
            }
            MediaControlEvent::Next => {
                handle.emit("media-control", "next").unwrap();
            }
            MediaControlEvent::Previous => {
                handle.emit("media-control", "previous").unwrap();
            }
            MediaControlEvent::SetVolume(value) => {
                handle.emit("media-control", (value, "volume")).unwrap();
            }
            MediaControlEvent::SeekBy(direction, duration) => {
                let positive = match direction {
                    souvlaki::SeekDirection::Forward => true,
                    souvlaki::SeekDirection::Backward => false,
                };
                handle
                    .emit("media-control", (positive, duration.as_secs_f64()))
                    .unwrap();
            }
            MediaControlEvent::SetPosition(position) => {
                handle
                    .emit("media-control", (position.0.as_secs_f64(), "position"))
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
}
