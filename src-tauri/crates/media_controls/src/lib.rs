use std::time::{SystemTime, UNIX_EPOCH};

use common::Tracks;
pub use kira::sound::PlaybackState;
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, Tween,
    clock::{ClockHandle, ClockSpeed},
    sound::{
        FromFileError,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
};
pub use souvlaki::{MediaControlEvent, PlatformConfig, SeekDirection};
use souvlaki::{MediaControls, MediaMetadata, MediaPlayback};

#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error(transparent)]
    Kira(#[from] kira::PlaySoundError<FromFileError>),
    #[error(transparent)]
    FromFile(#[from] FromFileError),
    #[error(transparent)]
    Souvlaki(#[from] souvlaki::Error),
    #[error(transparent)]
    Cpal(#[from] kira::backend::cpal::Error),
    #[error(transparent)]
    ResourceLimitReached(#[from] kira::ResourceLimitReached),
}

#[cfg(feature = "serialization")]
use serde::Serialize;
#[cfg(feature = "serialization")]
use specta::Type;

#[derive(Clone, Copy, Default, Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Type))]
pub enum PlayerState {
    Playing,
    #[default]
    Paused,
}

pub struct Player {
    /// Handler for  [`StreamingSoundData`]
    pub sound_handle: Option<StreamingSoundHandle<FromFileError>>,
    pub manager: AudioManager<DefaultBackend>,
    /// To use for playback & volume
    tween: Tween,
    /// Clock to keep track of user's progress, can't use [`Player::progress`] as the user can manipulate that
    clock: ClockHandle,
    /// How many seconds does the user need to listen to to scrobble
    scrobble_condition: f64,
    /// If it has already scrobbled this track
    pub scrobbled: bool,
    /// ID of the track
    pub track: Option<u32>,
    /// Progress of the player
    pub progress: f64,
    /// Duration of the track
    pub duration: f32,
    /// Volume that ranges from -60 to 1.0
    pub volume: f32,
    /// Player State
    pub state: PlayerState,
    /// Souvlaki Controls
    pub controls: MediaControls,
    // When the user started listening to track
    pub timestamp: i64,
}

impl Player {
    pub fn new(c: souvlaki::PlatformConfig) -> Result<Self, PlayerError> {
        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        // 1 tick per second
        let clock = manager.add_clock(ClockSpeed::TicksPerMinute(60.0))?;

        Ok(Player {
            sound_handle: None,
            manager,
            clock,
            scrobble_condition: -1.0,
            scrobbled: false,
            tween: Tween::default(),
            track: None,
            timestamp: 0,
            progress: 0.0,
            duration: 0.0,
            volume: -6.0, // externally 0.0 - 1.0
            state: PlayerState::Paused,
            controls: MediaControls::new(c)?,
        })
    }

    /// Takes a value from 0.0 to 1.0 and passes to player. Range gets converted to -60.0 to 1.0
    pub fn set_volume(&mut self, volume: f32) -> Result<(), PlayerError> {
        // https://www.desmos.com/calculator/cj1nmmamzb
        let converted_volume = -60.0 + 61.0 * volume.powf(0.44);

        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.set_volume(converted_volume, self.tween);
            self.controls.set_volume(volume as f64)?;
        }

        self.volume = converted_volume;

        Ok(())
    }

    pub fn play(&mut self, track: &Tracks) -> Result<(), PlayerError> {
        self.track = Some(track.id);

        logging::debug!("Trying to play track {}", track.name);
        let sound_data = self.load_sound(track)?.start_position(self.progress);

        let mut sh = self.manager.play(sound_data)?;
        sh.set_volume(self.volume, self.tween);

        self.sound_handle = Some(sh);
        self.state = PlayerState::Playing;
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        self.scrobbled = false;
        self.clock.start();
        self.set_playback(true)?;
        logging::debug!("Succesfully playing track {}", track.name);

        Ok(())
    }

    /// Initialize the player with a track and a progress
    pub fn initialize_player(&mut self, track: Tracks, progress: f64) -> Result<(), PlayerError> {
        logging::debug!(
            "Initializing player with track {} and progress {progress}",
            track.name
        );
        self.load_sound(&track)?;
        self.progress = progress;

        self.set_playback(false)?;

        Ok(())
    }

    fn load_sound(
        &mut self,
        track: &Tracks,
    ) -> Result<StreamingSoundData<FromFileError>, PlayerError> {
        let sound_data = StreamingSoundData::from_file(&track.path)?;
        self.duration = sound_data.duration().as_secs_f32();

        self.set_scrobble_condition();

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
    pub fn pause(&mut self) -> Result<(), PlayerError> {
        if let Some(ref mut sound_handle) = self.sound_handle {
            logging::debug!("Pausing track");
            sound_handle.pause(self.tween);
            self.state = PlayerState::Paused;
            self.progress = sound_handle.position();

            self.clock.pause();
            self.set_playback(false)?;
        }

        Ok(())
    }

    /// Resume track if has sound_handle
    pub fn resume(&mut self) -> Result<(), PlayerError> {
        if let Some(ref mut sound_handle) = self.sound_handle {
            logging::debug!("Resuming track");
            sound_handle.resume(self.tween);
            self.state = PlayerState::Playing;

            self.clock.start();
            self.set_playback(true)?;
        }

        Ok(())
    }

    /// Seek to a specific position in the track and resume playing if the player is paused and resume is true
    pub fn seek(&mut self, position: f64, resume: bool) -> Result<(), PlayerError> {
        if let Some(ref mut sound_handle) = self.sound_handle {
            logging::debug!("Seeking track to {position}");
            match self.state {
                PlayerState::Playing => {
                    sound_handle.seek_to(position);
                    self.progress = position;
                    self.set_playback(true)?;
                }
                _ => {
                    sound_handle.seek_to(position);
                    self.progress = position;

                    if resume {
                        self.resume()?
                    } else {
                        self.set_playback(false)?
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop track if has sound_handle
    pub fn stop(&mut self) -> Result<(), PlayerError> {
        logging::debug!("Stopping track");
        if let Some(ref mut sound_handle) = self.sound_handle {
            sound_handle.stop(self.tween);
        }
        self.state = PlayerState::Paused;
        self.progress = 0.0;
        self.track = None;

        self.clock.stop();
        self.controls.set_playback(MediaPlayback::Stopped)?;

        Ok(())
    }

    /// Set the progress of the player
    pub fn set_progress(&mut self, progress: f64) {
        logging::debug!("Setting track progress");
        self.progress = progress;
    }

    /// Gets player progress from `sound_handle`
    pub fn get_progress(&self) -> f64 {
        if let Some(ref sound_handle) = self.sound_handle {
            sound_handle.position()
        } else {
            -1.0
        }
    }

    /// Whether or not the clock progress has passsed `scrobble_condition`
    fn scrobble(&self) -> bool {
        let clock_time = self.clock.time();
        let ticks = clock_time.ticks;
        ticks as f64 >= self.scrobble_condition
    }

    /// Whether or not the track should be scrobbled depending on three factors.
    ///
    /// 1. The clock progress has passed `scrobble_condition`.
    /// 2. The track hasn't already been scrobbled.
    /// 3. `track` contains Some(id)
    pub fn should_scrobble(&mut self) -> Option<(u32, i64)> {
        let should_scrobble = self.track.is_some() && self.scrobble() && !self.scrobbled;

        if let Some(track_id) = self.track
            && should_scrobble
        {
            self.scrobbled = true;
            Some((track_id, self.timestamp))
        } else {
            None
        }
    }

    /// Gets players state from sound handle if exists.
    pub fn get_player_state(&self) -> Option<PlaybackState> {
        if let Some(handle) = &self.sound_handle {
            Some(handle.state())
        } else {
            None
        }
    }

    fn set_scrobble_condition(&mut self) {
        // if duration is greater than 4 mins, set to 4 min,
        // otherwise half of the duration.
        if self.duration > 240.0 {
            self.scrobble_condition = 240.0;
        } else if self.duration > 30.0 {
            self.scrobble_condition = (self.duration as f64) / 2.0;
        };
    }

    fn set_playback(&mut self, play: bool) -> Result<(), souvlaki::Error> {
        let progress: Option<souvlaki::MediaPosition> = progress_as_position(self.progress);
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
