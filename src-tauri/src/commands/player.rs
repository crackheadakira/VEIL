use crate::{discord, error::FrontendError, player::PlayerState, TauriState};
use db::models::Tracks;
use lastfm::Track;

#[tauri::command]
#[specta::specta]
pub async fn play_track(track_id: u32, state: TauriState<'_>) -> Result<(), FrontendError> {
    let track = state.db.by_id::<Tracks>(&track_id)?;

    // temporary scope to drop player before await
    {
        let mut player = state.player.lock().unwrap();

        if player.track.is_some() {
            player.stop();
        };

        let duration = player.duration;

        if track.duration == 0 {
            state
                .db
                .update_duration(&track_id, &track.album_id, &(duration as u32))?;
        }

        let _ = player.play(&track);

        let progress = player.progress;
        let payload = discord::PayloadData {
            state: track.artist_name.clone() + " - " + &track.album_name,
            details: track.name.clone(),
            small_image: String::from("playing"),
            small_text: String::from("Playing"),
            show_timestamps: true,
            duration,
            progress,
        };

        let mut discord = state.discord.lock().unwrap();
        discord.make_activity(payload)?;
    }

    state
        .lastfm
        .track()
        .update_now_playing(track.artist_name, track.name)
        .send()
        .await?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn pause_track(state: TauriState) {
    let mut player = state.player.lock().unwrap();
    let mut discord = state.discord.lock().unwrap();
    player.pause();

    let mut payload = discord.payload.clone();
    payload = discord::PayloadData {
        small_image: String::from("paused"),
        small_text: String::from("Paused"),
        show_timestamps: false,
        ..payload
    };

    discord.make_activity(payload).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn resume_track(state: TauriState) {
    let mut player = state.player.lock().unwrap();
    let mut discord = state.discord.lock().unwrap();

    player.resume();

    let mut payload = discord.payload.clone();
    payload = discord::PayloadData {
        small_image: String::from("playing"),
        small_text: String::from("Playing"),
        show_timestamps: true,
        ..payload
    };

    discord.make_activity(payload).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn seek_track(position: f64, resume: bool, state: TauriState) {
    let mut player = state.player.lock().unwrap();
    let mut discord = state.discord.lock().unwrap();
    player.seek(position, resume);

    let text_display = if resume {
        String::from("Playing")
    } else {
        String::from("Paused")
    };

    let mut payload = discord.payload.clone();
    payload = discord::PayloadData {
        small_image: text_display.to_lowercase(),
        small_text: text_display,
        show_timestamps: true,
        ..payload
    };

    discord.make_activity(payload).unwrap();
}

#[tauri::command]
#[specta::specta]
pub fn set_volume(volume: f32, state: TauriState) {
    let mut player = state.player.lock().unwrap();

    player.set_volume(volume);
}

#[tauri::command]
#[specta::specta]
pub fn get_player_state(state: TauriState) -> PlayerState {
    let player = state.player.lock().unwrap();
    player.state
}

#[tauri::command]
#[specta::specta]
pub fn get_player_progress(state: TauriState) -> f64 {
    let player = state.player.lock().unwrap();
    player.progress
}

#[tauri::command]
#[specta::specta]
pub fn player_has_ended(state: TauriState) -> bool {
    let player = state.player.lock().unwrap();
    player.has_ended()
}

#[tauri::command]
#[specta::specta]
pub fn player_has_track(state: TauriState) -> bool {
    let player = state.player.lock().unwrap();
    player.track.is_some()
}

#[tauri::command]
#[specta::specta]
pub fn get_player_duration(state: TauriState) -> f32 {
    let player = state.player.lock().unwrap();
    player.duration
}

#[tauri::command]
#[specta::specta]
pub fn stop_player(state: TauriState) {
    let mut player = state.player.lock().unwrap();
    player.stop();
}

#[tauri::command]
#[specta::specta]
pub fn initialize_player(
    track_id: u32,
    progress: f64,
    state: TauriState,
) -> Result<(), FrontendError> {
    let mut player = state.player.lock().unwrap();
    let track = state.db.by_id::<Tracks>(&track_id)?;
    player.initialize_player(track, progress)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_player_progress(progress: f64, state: TauriState) {
    let mut player = state.player.lock().unwrap();
    player.set_progress(progress);
}
