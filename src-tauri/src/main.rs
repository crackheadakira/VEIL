// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use souvlaki::MediaControls;

mod commands;
mod db;
mod models;
mod player;
mod setup;

use commands::metadata::*;

pub struct SodapopState {
    pub player: player::Player,
    pub db: db::Database,
    pub controls: MediaControls,
}

#[tokio::main]
async fn main() {
    let builder = setup::get_builder();

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(builder.invoke_handler())
        .setup(setup::setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
