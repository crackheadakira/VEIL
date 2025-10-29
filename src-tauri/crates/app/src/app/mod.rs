use crate::app::builder::{handle_tauri_setup, make_specta_type_builder, mount_tauri_builder};
use logging::lock_or_log;
use tauri::{Manager, RunEvent};

pub use state::{SodapopState, TauriState};

mod builder;
mod state;

pub fn run() -> Result<(), anyhow::Error> {
    let specta_builder = make_specta_type_builder();
    mount_tauri_builder(&specta_builder)
        .setup(|app| handle_tauri_setup(app, specta_builder))
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::ExitRequested { .. } = event {
                let state = app.state::<SodapopState>();
                let config = lock_or_log(state.config.read(), "Config Lock").unwrap();
                let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex").unwrap();

                if config.discord_enabled {
                    discord.close();
                };

                state.db.shutdown().unwrap();
            }
        });

    Ok(())
}
