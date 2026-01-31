use common::Tracks;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::queue::RepeatMode;

#[derive(Serialize, Deserialize, Type, Clone)]
pub enum PlayButtonState {
    Playing,
    Paused,
}

#[derive(Serialize, Deserialize, Type, Event, Clone)]
#[serde(tag = "type", content = "data")]
pub enum UIUpdateEvent {
    /// Updates the state of the shuffle button
    ShuffleButton {
        enabled: bool,
    },

    LoopButton {
        mode: RepeatMode,
    },

    PlayButton {
        state: PlayButtonState,
    },

    TrackChange {
        track: Tracks,
    },
}
