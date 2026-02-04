use common::Tracks;
use serde::{Deserialize, Serialize};

use crate::queue::RepeatMode;

#[derive(Serialize, Deserialize, Clone)]
pub enum PlayButtonState {
    Playing,
    Paused,
}

#[derive(Serialize, Deserialize, Clone)]
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
