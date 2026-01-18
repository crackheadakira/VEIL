use crate::{
    SodapopState, TauriState,
    error::FrontendError,
    systems::player::{
        PlayerEvent, next_track_status, send_player_progress_via_channel,
        try_preloading_next_sound_handle,
    },
};

use logging::lock_or_log;
use media_controls::PlayerState;
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Manager, ipc::Channel};
use tauri_specta::Event;

#[tauri::command]
#[specta::specta]
pub fn get_player_state(state: TauriState) -> PlayerState {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.state
}

#[tauri::command]
#[specta::specta]
pub fn get_player_progress(state: TauriState) -> f64 {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.get_progress()
}

#[tauri::command]
#[specta::specta]
pub fn player_has_ended(state: TauriState) -> bool {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.has_ended()
}

#[tauri::command]
#[specta::specta]
pub fn player_has_track(state: TauriState) -> bool {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.track.is_some()
}

#[tauri::command]
#[specta::specta]
pub fn get_player_duration(state: TauriState) -> f64 {
    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
    player.get_duration()
}

#[derive(Clone, Serialize, Type)]
#[serde(tag = "event", content = "data")]
// rust-analyzer expected Expr error: https://github.com/specta-rs/specta/issues/387
pub enum PlayerProgressEvent {
    Progress { progress: f64 },
}

#[tauri::command]
#[specta::specta]
pub fn player_progress_channel(
    handle: AppHandle,
    on_event: Channel<PlayerProgressEvent>,
) -> Result<(), FrontendError> {
    tauri::async_runtime::spawn(async move {
        let state = handle.state::<SodapopState>();

        let sleep_duration = std::time::Duration::from_millis(10);
        let track_end_interval = std::time::Duration::from_millis(25);
        let progress_interval = std::time::Duration::from_millis(400);

        let mut last_track_end_check = tokio::time::Instant::now();
        let mut last_progress_sent = tokio::time::Instant::now();

        loop {
            tokio::time::sleep(sleep_duration).await;
            let ends_soon = {
                let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
                (player.get_duration() - player.get_progress()) <= 0.5
            };

            if !ends_soon {
                if last_progress_sent.elapsed() >= progress_interval {
                    send_player_progress_via_channel(&state, &on_event);
                    last_progress_sent = tokio::time::Instant::now();
                }

                continue;
            }

            // Workaround due to RwLock not being thread-safe (using std::sync)
            {
                let mut player = lock_or_log(state.player.write(), "Player Write Lock").unwrap();

                try_preloading_next_sound_handle(&state, &mut player);

                if last_track_end_check.elapsed() >= track_end_interval {
                    if let Some(track) = next_track_status(&state, &player) {
                        match PlayerEvent::emit(&PlayerEvent::NewTrack { track }, &handle) {
                            Ok(_) => {
                                logging::debug!("Emitted new track event to frontend.");
                            }
                            Err(e) => {
                                logging::error!("Error emitting new track from queue system: {e}");
                            }
                        };
                    };

                    last_track_end_check = tokio::time::Instant::now();
                }
            };

            let queue_has_ended = {
                let queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
                queue.reached_end
            };

            if queue_has_ended {
                // Wait until a notificaton to resume this loop
                match PlayerEvent::emit(&PlayerEvent::Stop, &handle) {
                    Ok(_) => {}
                    Err(e) => {
                        logging::error!("Error emitting stop player event: {e}");
                    }
                }

                state.resume_notify.notified().await;
                logging::debug!("New track added, resuming loop.");
            }
        }
    });

    Ok(())
}
