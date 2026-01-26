// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod config;
mod discord;
mod error;
mod events;
mod queue;
mod systems;

pub use app::{SodapopState, TauriState};

fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    logging::init();

    app::run()
}
