use kira::{
    clock::{ClockHandle, ClockSpeed},
    sound::{
        streaming::{StreamingSoundData, StreamingSoundHandle},
        FromFileError, PlaybackState,
    },
    AudioManager, AudioManagerSettings, DefaultBackend, Tween,
};
use serde::Serialize;

use db::models::Tracks;
use souvlaki::{MediaControls, MediaMetadata, MediaPlayback};

#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error(transparent)]
    Kira(#[from] kira::PlaySoundError<FromFileError>),
    #[error(transparent)]
    FromFile(#[from] FromFileError),
    #[error(transparent)]
    Souvlaki(#[from] souvlaki::Error),
}

#[derive(Clone, Copy, Serialize, specta::Type, Default, Debug)]
pub enum PlayerState {
    Playing,
    #[default]
    Paused,
}

pub struct Player {
    sound_handle: Option<StreamingSoundHandle<FromFileError>>,
    manager: AudioManager<DefaultBackend>,
    tween: Tween,
    clock: ClockHandle,
    scrobble_condition: f64,
    pub track: Option<u32>,
    pub progress: f64,
    pub duration: f32,
    pub volume: f32,
    pub state: PlayerState,
    pub controls: MediaControls,
}

impl Player {
    pub fn new(c: souvlaki::PlatformConfig) -> Self {
        let mut manager =
            AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).unwrap();

        // 1 tick per second
        let clock = manager.add_clock(ClockSpeed::TicksPerMinute(60.0)).unwrap();

        Player {
            sound_handle: None,
            manager,
            clock,
            scrobble_condition: -1.0,
            tween: Tween::default(),
            track: None,
            progress: 0.0,
            duration: 0.0,
            volume: -6.0, // externally 0.0 - 1.0
            state: PlayerState::Paused,
            controls: MediaControls::new(c).unwrap(),
        }
    }

    /// Takes a value from 0.0 to 1.0 and passes to player. Range gets converted to -60.0 to 1.0
    pub fn set_volume(&mut self, volume: f32) {
        let converted_volume = -60.0 + volume * (61.0);

        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.set_volume(converted_volume, self.tween);
            self.controls.set_volume(volume as f64).unwrap();
        }

        self.volume = converted_volume;
    }

    pub fn play(&mut self, track: &Tracks) -> Result<(), PlayerError> {
        self.track = Some(track.id);

        if self.track.is_some() {
            let sound_data = self.load_sound(track)?.start_position(self.progress);
            self.duration = sound_data.duration().as_secs_f32();

            let mut sh = self.manager.play(sound_data)?;
            sh.set_volume(self.volume, Tween::default());

            self.sound_handle = Some(sh);
            self.state = PlayerState::Playing;

            // if duration is greater than 4 mins, set to 4 min,
            // otherwise half of the duration.
            if self.duration > 240.0 {
                self.scrobble_condition = 240.0;
            } else if self.duration > 30.0 {
                self.scrobble_condition = (self.duration as f64) / 2.0;
            };

            self.clock.start();
            self.set_playback(true).unwrap();
        }

        Ok(())
    }

    /// Initialize the player with a track and a progress
    pub fn initialize_player(&mut self, track: Tracks, progress: f64) -> Result<(), PlayerError> {
        self.load_sound(&track)?;
        self.progress = progress;

        self.set_playback(false).unwrap();

        Ok(())
    }

    fn load_sound(
        &mut self,
        track: &Tracks,
    ) -> Result<StreamingSoundData<FromFileError>, PlayerError> {
        let sound_data = StreamingSoundData::from_file(&track.path)?;
        self.duration = sound_data.duration().as_secs_f32();

        self.controls.set_metadata(MediaMetadata {
            title: Some(&track.name),
            album: Some(&track.album_name),
            artist: Some(&track.artist_name),
            cover_url: Some(&track.cover_path),
            duration: Some(std::time::Duration::from_secs(self.duration as u64)),
        })?;

        Ok(sound_data)
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

            self.clock.pause();
            self.set_playback(false).unwrap();
        }
    }

    /// Resume track if has sound_handle
    pub fn resume(&mut self) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.resume(self.tween);
            self.state = PlayerState::Playing;

            self.clock.start();
            self.set_playback(true).unwrap();
        }
    }

    /// Seek to a specific position in the track and resume playing if the player is paused and resume is true
    pub fn seek(&mut self, position: f64, resume: bool) {
        if let Some(ref mut sound_handle) = self.sound_handle {
            match self.state {
                PlayerState::Playing => {
                    sound_handle.seek_to(position);
                    self.progress = position;
                    self.set_playback(true).unwrap();
                }
                _ => {
                    sound_handle.seek_to(position);
                    self.progress = position;

                    if resume {
                        self.resume()
                    } else {
                        self.set_playback(false).unwrap()
                    }
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

        self.clock.stop();
        self.controls.set_playback(MediaPlayback::Stopped).unwrap();
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

                self.set_playback(true).unwrap();
            }
        }
    }

    /// Whether or not the clock progress has passsed `scrobble_condition`
    pub fn scrobble(&self) -> bool {
        let clock_time = self.clock.time();
        let ticks = clock_time.ticks;
        ticks as f64 >= self.scrobble_condition
    }

    fn set_playback(&mut self, play: bool) -> Result<(), souvlaki::Error> {
        let progress = progress_as_position(self.progress);
        let playback = match play {
            true => MediaPlayback::Playing { progress },
            false => MediaPlayback::Paused { progress },
        };

        self.controls.set_playback(playback)?;

        Ok(())
    }
}

fn progress_as_position(progress: f64) -> Option<souvlaki::MediaPosition> {
    Some(souvlaki::MediaPosition(std::time::Duration::from_secs_f64(
        progress,
    )))
}
