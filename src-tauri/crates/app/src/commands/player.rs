use crate::{
    TauriState, VeilState,
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
        let state = handle.state::<VeilState>();
        let progress_interval = std::time::Duration::from_millis(400);

        loop {
            tokio::time::sleep(progress_interval).await;
            send_player_progress_via_channel(&state, &on_event);
        }
    });

    Ok(())
}

pub fn initiate_track_ended_thread(handle: &AppHandle) {
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let state = handle.state::<VeilState>();
        let check_interval = std::time::Duration::from_millis(25);

        loop {
            tokio::time::sleep(check_interval).await;

            let ended = {
                let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
                player.has_ended()
            };

            if !ended {
                continue;
            }

            {
                let mut player = lock_or_log(state.player.write(), "Player Write Lock").unwrap();
                try_preloading_next_sound_handle(&state, &mut player);

                if let Some(track) = next_track_status(&state, &player) {
                    let _ = PlayerEvent::emit(&PlayerEvent::NewTrack { track }, &handle);
                }
            }

            let queue_has_ended = {
                let queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
                queue.reached_end
            };

            if queue_has_ended {
                let _ = PlayerEvent::emit(&PlayerEvent::Stop, &handle);
                state.resume_notify.notified().await;
            }
        }
    });
}
