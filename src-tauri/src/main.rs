// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    http::{HeaderValue, Method},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use tauri_plugin_fs::FsExt;
use tauri_specta::*;

mod commands;
mod db;
mod interface;
mod models;

use commands::metadata::*;
use commands::music_folder::*;
use commands::sqlite::*;

use std::fs::create_dir;
use std::path::Path;

#[tokio::main]
async fn main() {
    let invoke_handler = {
        let builder = ts::builder().commands(collect_commands![
            read_metadata,
            select_music_folder,
            get_sqlite,
            get_album_with_tracks,
            get_artist_with_albums,
            get_all_albums,
            track_by_id,
            async_metadata,
        ]);

        #[cfg(debug_assertions)] // <- Only export on non-release builds
        let builder = builder.path("../src/bindings.ts");

        builder.build().unwrap()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(invoke_handler)
        .setup(|app| {
            #[cfg(target_os = "linux")]
            tokio::spawn(async move {
                let serve_dir = ServeDir::new("/");

                let axum_app = Router::new().nest_service("/", serve_dir).layer(
                    CorsLayer::new()
                        .allow_origin("*".parse::<HeaderValue>().unwrap())
                        .allow_methods([Method::GET]),
                );

                axum::Server::bind(&"127.0.0.1:16780".parse().unwrap())
                    .serve(axum_app.into_make_service())
                    .await
                    .unwrap();
            });

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
            scope.allow_directory(data, true);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
