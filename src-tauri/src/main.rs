// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_specta::collect_commands;

use specta_typescript::Typescript;
use tauri_plugin_fs::FsExt;
use tauri_specta::Builder;

mod commands;
mod db;
mod interface;
mod models;
mod player;

use commands::metadata::*;
use commands::music_folder::*;
use commands::player::*;
use commands::sqlite::*;

use std::fs::create_dir;
use std::path::Path;
use std::sync::Mutex;

#[derive(Default)]
pub struct SodapopState {
    pub player: player::Player,
}

#[tokio::main]
async fn main() {
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        read_metadata,
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

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .manage(Mutex::new(SodapopState::default()))
        .setup(|app| {
            let data = db::data_path();
            if !Path::new(&data).exists() {
                create_dir(&data).expect("Error creating data directory");
            }

            let covers = data.clone() + "/covers";
            if !Path::new(&covers).exists() {
                create_dir(covers).expect("Error creating covers directory")
            }

            db::init();

            let scope = app.fs_scope();
            if let Err(e) = scope.allow_directory(data, true) {
                eprintln!("Error allowing directory: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
