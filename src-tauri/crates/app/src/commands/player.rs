use std::sync::Arc;

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
