mod app;
// mod commands;
mod config;
mod discord;
mod error;
mod events;
mod queue;
mod systems;
mod ui;

pub use app::VeilState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    logging::init();

    app::run();
}
