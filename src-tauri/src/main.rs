// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{http, routing::get, serve, Router};
use tauri_specta::collect_commands;
use tokio::net::TcpListener;
use tower_http::cors;
use tower_http::services::ServeDir;

use specta_typescript::Typescript;
use tauri_plugin_fs::FsExt;
use tauri_specta::Builder;

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
    let builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        read_metadata,
        select_music_folder,
        get_sqlite,
        get_album_with_tracks,
        get_artist_with_albums,
        get_all_albums,
        track_by_id,
        async_metadata,
    ]);
    /*let builder = ts::builder().commands(collect_commands![
        read_metadata,
        select_music_folder,
        get_sqlite,
        get_album_with_tracks,
        get_artist_with_albums,
        get_all_albums,
        track_by_id,
        async_metadata,
    ]);*/

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export TypeScript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(|app| {
            #[cfg(target_os = "linux")]
            tokio::spawn(async move {
                let serve_dir = ServeDir::new("/");

                let app = Router::new()
                    .route("/health", get(|| async { "Server is running" }))
                    .layer(
                        cors::CorsLayer::new()
                            .allow_origin(cors::Any)
                            .allow_methods([http::Method::GET]),
                    )
                    .fallback_service(serve_dir);

                let listener = TcpListener::bind("127.0.0.1:16780")
                    .await
                    .expect("Error binding to port");

                serve(listener, app)
                    .await
                    .expect("Error initializing server");
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
            if let Err(e) = scope.allow_directory(data, true) {
                eprintln!("Error allowing directory: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
