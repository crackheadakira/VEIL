use anyhow::Result;
use kira::{
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle},
        FromFileError,
    },
    AudioManager, AudioManagerSettings, DefaultBackend, Tween,
};

use crate::interface::track::get_track_by_id;

pub enum PlayerState {
    Playing,
    Paused,
    Stopped,
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

impl Player {
    pub fn new() -> Result<Self> {
        let manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;
        Ok(Player {
            manager,
            sound_handle: None,
            tween: Tween::default(),
            track: None,
            progress: 0.0,
            duration: 0.0,
            volume: -6.0,
            state: PlayerState::Stopped,
        })
    }

    pub fn set_volume(&mut self, volume: f32) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.set_volume(volume, self.tween);
        }
        self.volume = volume;
    }

    pub fn play(&mut self, track_id: u32) -> Result<()> {
        self.track = Some(track_id);

        if let Some(ref mut track_id) = self.track {
            let track = get_track_by_id(&track_id);
            let sound_data: StreamingSoundData<FromFileError> =
                StreamingSoundData::from_file(track.path)?;
            self.duration = sound_data.duration().as_secs_f32();
            self.sound_handle = Some(self.manager.play(sound_data)?);
            self.sound_handle
                .as_mut()
                .unwrap()
                .set_volume(self.volume, Tween::default());
        }

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.pause(self.tween);
            self.state = PlayerState::Paused;
            self.progress = sound_handle.position();
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.resume(self.tween);
            self.state = PlayerState::Playing;
        }
    }

    pub fn seek(&mut self, position: f64) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            match self.state {
                PlayerState::Playing => sound_handle.seek_to(position),
                _ => {
                    sound_handle.seek_to(position);
                    self.resume();
                }
            }
        }
    }

    pub fn seek_by(&mut self, amount: f64) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            match self.state {
                PlayerState::Playing => sound_handle.seek_by(amount),
                _ => {
                    sound_handle.seek_by(amount);
                    self.resume()
                }
            }
        }
    }
}
