use common::Tracks;
use logging::lock_or_log;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::{Event, TypedEvent};

use crate::{TauriState, config::ThemeMode, error::FrontendError};

#[derive(Serialize, Deserialize, Type, Event, Clone)]
pub struct SodapopConfigEvent {
    pub theme: Option<ThemeMode>,
    pub discord_enabled: Option<bool>,
    pub last_fm_enabled: Option<bool>,
    pub music_dir: Option<String>,
    pub last_fm_key: Option<String>,
}

#[derive(Serialize, Deserialize, Type, Event)]
pub struct NewTrackEvent {
    pub track: Tracks,
}

impl NewTrackEvent {
    // TODO: Discord RPC & Last.FM
    pub fn set_new_track(
        event: TypedEvent<NewTrackEvent>,
        state: TauriState,
    ) -> Result<(), FrontendError> {
        let mut player = lock_or_log(state.player.lock(), "Player Mutex").unwrap();
        let track = event.payload.track;

        // Reset the player's internal progress to 0
        player.set_progress(0.0);

        if player.track.is_some() {
            player.stop()?;
        };

        player.play(&track)?;

        Ok(())
    }
}
