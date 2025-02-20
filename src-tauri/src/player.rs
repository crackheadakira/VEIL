use kira::{
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle},
        FromFileError, PlaybackState,
    },
    AudioManager, AudioManagerSettings, DefaultBackend, Tween,
};
use serde::Serialize;

use db::models::Tracks;

#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error(transparent)]
    KiraError(#[from] kira::PlaySoundError<FromFileError>),
    #[error(transparent)]
    FromFileError(#[from] FromFileError),
}

#[derive(Clone, Copy, Serialize, specta::Type, Default, Debug)]
pub enum PlayerState {
    Playing,
    #[default]
    Paused,
}

pub struct Player {
    manager: AudioManager<DefaultBackend>,
    sound_handle: Option<StreamingSoundHandle<FromFileError>>,
    tween: Tween,
    pub track: Option<u32>,
    pub progress: f64,
    pub duration: f32,
    pub volume: f32,
    pub state: PlayerState,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

        Player {
            manager,
            sound_handle: None,
            tween: Tween::default(),
            track: None,
            progress: 0.0,
            duration: 0.0,
            volume: -6.0, // externally 0.0 - 1.0
            state: PlayerState::Paused,
        }
    }

    /// Takes a value from 0.0 to 1.0 and passes to player. Range gets converted to -60.0 to 1.0
    pub fn set_volume(&mut self, volume: f32) {
        let converted_volume = -60.0 + volume * (61.0);

        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.set_volume(converted_volume, self.tween);
        }

        self.volume = converted_volume;
    }

    pub fn play(&mut self, track: &Tracks) -> Result<(), PlayerError> {
        self.track = Some(track.id);

        if self.track.is_some() {
            let sound_data =
                StreamingSoundData::from_file(&track.path)?.start_position(self.progress);
            self.duration = sound_data.duration().as_secs_f32();
            self.sound_handle = Some(self.manager.play(sound_data)?);
            self.sound_handle
                .as_mut()
                .unwrap()
                .set_volume(self.volume, Tween::default());
            self.state = PlayerState::Playing;
        }

        Ok(())
    }

    /// Initialize the player with a track and a progress
    pub fn initialize_player(&mut self, track: Tracks, progress: f64) -> Result<(), PlayerError> {
        let sound_data: StreamingSoundData<FromFileError> =
            StreamingSoundData::from_file(track.path)?;
        self.duration = sound_data.duration().as_secs_f32();
        self.progress = progress;

        Ok(())
    }

    /// Check if the track has ended
    pub fn has_ended(&self) -> bool {
        if let Some(ref sound_handle) = self.sound_handle {
            sound_handle.state() == PlaybackState::Stopped
        } else {
            false
        }
    }

    /// Pause track if has sound_handle
    pub fn pause(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.pause(self.tween);
            self.state = PlayerState::Paused;
            self.progress = sound_handle.position();
        }
    }

    /// Resume track if has sound_handle
    pub fn resume(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.resume(self.tween);
            self.state = PlayerState::Playing;
        }
    }

    /// Seek to a specific position in the track and resume playing if the player is paused and resume is true
    pub fn seek(&mut self, position: f64, resume: bool) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            match self.state {
                PlayerState::Playing => {
                    sound_handle.seek_to(position);
                    self.progress = position;
                }
                _ => {
                    sound_handle.seek_to(position);
                    self.progress = position;
                    if resume {
                        self.resume()
                    };
                }
            }
        }
    }

    /// Stop track if has sound_handle
    pub fn stop(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.stop(self.tween);
        }
        self.state = PlayerState::Paused;
        self.progress = 0.0;
        self.track = None;
    }

    /// Set the progress of the player
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress;
    }

    /// Update the progress of the player if the player is playing
    pub fn update(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            if let PlayerState::Playing = self.state {
                self.progress = sound_handle.position();
            }
        }
    }
}
