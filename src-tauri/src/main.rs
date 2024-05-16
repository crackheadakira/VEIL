// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    http::{HeaderValue, Method},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use tauri_specta::*;

mod commands;
mod db;
mod interface;
mod models;

use commands::metadata::*;
use commands::music_folder::*;
use commands::sqlite::*;

#[tokio::main]
async fn main() {
    let invoke_handler = {
        let builder = tauri_specta::ts::builder().commands(tauri_specta::collect_commands![
            read_metadata,
            select_music_folder,
            get_sqlite,
            get_album_with_tracks,
            get_artist_with_albums
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
        .setup(|_app| {
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

            db::init();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
