use std::time::{Duration, SystemTime, UNIX_EPOCH};

use common::Tracks;
pub use kira::sound::PlaybackState;
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, StartTime, Tween,
    clock::{ClockHandle, ClockSpeed},
    sound::{
        FromFileError,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
};
pub use souvlaki::{MediaControlEvent, PlatformConfig, SeekDirection};
use souvlaki::{MediaControls, MediaMetadata, MediaPlayback};

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

pub(crate) type Result<T, U = Error> = std::result::Result<T, U>;

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

pub struct PlayerTrack {
    /// Handle for the tracks [`StreamingSoundData`]
    sound_handle: StreamingSoundHandle<FromFileError>,

    /// Database ID of the track
    pub id: u32,

    /// How far along the track the player has progressed
    pub progress: f64,

    /// Duration of the track
    pub duration: f64,

    /// At what time did the user start listening to the track
    pub timestamp: u64,

    /// If the track has already been scrobbled
    pub scrobbled: bool,

    /// How many seconds does the user need to listen to to be able to scrobble the track
    scrobble_condition: Option<f64>,
}

impl PlayerTrack {
    pub fn new(
        track_id: u32,
        duration: f64,
        sound_handle: StreamingSoundHandle<FromFileError>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Self {
            id: track_id,
            progress: sound_handle.position(),
            sound_handle,
            duration,
            timestamp,
            scrobbled: false,
            scrobble_condition: Self::get_scrobble_condition(duration),
        }
    }

    // https://www.last.fm/api/scrobbling#when-is-a-scrobble-a-scrobble
    /// If duration is greater than 4 mins, set condition to 4 mins,
    /// otherwise half of the duration as long as it's above 30 seconds.
    fn get_scrobble_condition(duration: f64) -> Option<f64> {
        if duration > 240.0 {
            Some(240.0)
        } else if duration > 30.0 {
            Some(duration / 2.0)
        } else {
            None
        }
    }
}

pub struct Player {
    manager: AudioManager<DefaultBackend>,

    /// To use for playback & volume
    tween: Tween,

    /// Clock to keep track of user's progress, can't use [`Player::progress`] as the user can manipulate that
    clock: ClockHandle,

    /// Volume that ranges from -60 to 1.0
    pub volume: f32,

    /// Player State
    pub state: PlayerState,

    /// Souvlaki Controls
    pub controls: MediaControls,

    pub track: Option<PlayerTrack>,
    preloaded_track: Option<PlayerTrack>,
}

impl Player {
    pub fn new(c: souvlaki::PlatformConfig) -> Result<Self> {
        let mut manager = AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        // 1 tick per second
        let clock = manager.add_clock(ClockSpeed::TicksPerMinute(60.0))?;

        Ok(Player {
            manager,
            clock,
            tween: Tween {
                start_time: StartTime::Immediate,
                duration: Duration::from_millis(0),
                easing: kira::Easing::Linear,
            },
            track: None,
            preloaded_track: None,
            volume: -6.0, // externally 0.0 - 1.0
            state: PlayerState::Paused,
            controls: MediaControls::new(c)?,
        })
    }

    /// Takes a value from 0.0 to 1.0 and passes to player. Range gets converted to -60.0 to 1.0
    pub fn set_volume(&mut self, volume: f32) -> Result<()> {
        // https://www.desmos.com/calculator/cj1nmmamzb
        let converted_volume = -60.0 + 61.0 * volume.powf(0.44);

        if let Some(ref mut player_track) = self.track {
            player_track
                .sound_handle
                .set_volume(converted_volume, self.tween);
            self.controls.set_volume(volume as f64)?;
        }

        self.volume = converted_volume;

        Ok(())
    }

    pub fn play(&mut self, track: &Tracks, start_position: Option<f64>) -> Result<(f64, f64)> {
        logging::debug!("Trying to play track {}", track.name);

        let (progress, track_duration) = {
            if let Some(mut player_track) =
                self.preloaded_track.take().or_else(|| self.track.take())
            {
                logging::debug!("Found existing player track for track {}", track.name);
                player_track
                    .sound_handle
                    .set_volume(self.volume, self.tween);

                player_track.sound_handle.resume(self.tween);

                let duration = player_track.duration;
                let progress = player_track.progress;

                self.track = Some(player_track);

                (progress, duration)
            } else {
                logging::debug!("Creating new player track for {}", track.name);
                let mut player_track = self.make_player_track(track, start_position)?;
                player_track.sound_handle.resume(self.tween);

                let duration = player_track.duration;
                let progress = player_track.progress;

                self.track = Some(player_track);

                (progress, duration)
            }
        };

        self.state = PlayerState::Playing;
        self.clock.start();

        self.controls.set_metadata(MediaMetadata {
            title: Some(&track.name),
            album: Some(&track.album_name),
            artist: Some(&track.artist_name),
            cover_url: Some(&track.cover_path),
            duration: Some(Duration::from_secs_f64(track_duration)),
        })?;
        self.set_media_controls_playing(progress)?;

        logging::debug!("Succesfully playing track {}", track.name);

        Ok((progress, track_duration))
    }

    /// Initialize the player with a track and a progress
    pub fn initialize_player(&mut self, track: Tracks, progress: f64) -> Result<()> {
        logging::debug!(
            "Initializing player with track {} and progress {progress}",
            track.name
        );

        let player_track = self.make_player_track(&track, Some(progress))?;
        self.track = Some(player_track);

        Ok(())
    }

    fn make_player_track(
        &mut self,
        track: &Tracks,
        start_position: Option<f64>,
    ) -> Result<PlayerTrack> {
        let progress = start_position.unwrap_or(0.0);
        let sound_data = StreamingSoundData::from_file(&track.path)?.start_position(progress);
        let track_duration = sound_data.duration();

        let mut sound_handle = self.manager.play(sound_data)?;
        sound_handle.set_volume(self.volume, self.tween);
        sound_handle.pause(self.tween);

        Ok(PlayerTrack::new(
            track.id,
            track_duration.as_secs_f64(),
            sound_handle,
        ))
    }

    pub fn maybe_queue_next(&mut self, track: &Tracks) -> Result<()> {
        logging::debug!("Preloading next track for gapless playback.");

        let player_track = self.make_player_track(&track, None)?;
        self.preloaded_track = Some(player_track);

        Ok(())
    }

    /// Check if the track has ended
    pub fn has_ended(&self) -> bool {
        if let Some(ref player_track) = self.track {
            player_track.sound_handle.state() == PlaybackState::Stopped
        } else {
            true
        }
    }

    /// Pause track if has sound_handle
    pub fn pause(&mut self) -> Result<()> {
        if let Some(ref mut player_track) = self.track {
            logging::debug!("Pausing track");

            let progress = player_track.sound_handle.position();
            player_track.sound_handle.pause(self.tween);
            player_track.progress = progress;

            self.state = PlayerState::Paused;

            self.clock.pause();
            self.set_media_controls_paused(progress)?;
        }

        Ok(())
    }

    /// Resume track if has sound_handle
    pub fn resume(&mut self) -> Result<()> {
        if let Some(ref mut player_track) = self.track {
            logging::debug!("Resuming track");

            player_track.sound_handle.resume(self.tween);
            self.state = PlayerState::Playing;

            let progress = player_track.progress;
            self.set_media_controls_playing(progress)?;

            self.clock.start();
        }

        Ok(())
    }

    /// Seek to a specific position in the track and resume playing if the player is paused and resume is true
    pub fn seek(&mut self, position: f64, resume: bool) -> Result<()> {
        if let Some(ref mut player_track) = self.track {
            logging::debug!("Seeking track to {position}");

            match self.state {
                PlayerState::Playing => {
                    player_track.sound_handle.seek_to(position);
                    player_track.progress = position;
                    self.set_media_controls_playing(position)?;
                }
                _ => {
                    player_track.sound_handle.seek_to(position);
                    player_track.progress = position;

                    if resume {
                        self.resume()?
                    } else {
                        self.set_media_controls_paused(position)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop track if has sound_handle
    pub fn stop(&mut self) -> Result<()> {
        logging::debug!("Stopping track");

        if let Some(mut player_track) = self.track.take() {
            player_track.sound_handle.stop(self.tween);
        }

        self.state = PlayerState::Paused;

        self.clock.stop();
        self.controls.set_playback(MediaPlayback::Stopped)?;

        Ok(())
    }

    /// Gets player progress from `sound_handle`
    pub fn get_progress(&self) -> f64 {
        if let Some(ref player_track) = self.track {
            player_track.sound_handle.position()
        } else {
            -1.0
        }
    }

    /// Gets player progress from `track`
    pub fn get_duration(&self) -> f64 {
        if let Some(ref player_track) = self.track {
            player_track.duration
        } else {
            -1.0
        }
    }

    /// Whether or not the clock progress has passsed `scrobble_condition`
    fn scrobble(&self) -> bool {
        self.track
            .as_ref()
            .and_then(|track| {
                track.scrobble_condition.map(|cond| {
                    let ticks = self.clock.time().ticks as f64;
                    ticks >= cond
                })
            })
            .unwrap_or(false)
    }

    /// Whether or not the track should be scrobbled depending on three factors.
    ///
    /// 1. The clock progress has passed `scrobble_condition`.
    /// 2. The track hasn't already been scrobbled.
    /// 3. `track` contains Some(id)
    pub fn should_scrobble(&mut self) -> Option<(u32, i64)> {
        let can_scrobble = self.scrobble();

        if let Some(track) = self.track.as_mut() {
            if !can_scrobble || track.scrobbled {
                return None;
            }

            track.scrobbled = true;
            Some((track.id, track.timestamp as i64))
        } else {
            None
        }
    }

    /// Gets players state from sound handle if exists.
    pub fn get_player_state(&self) -> Option<PlaybackState> {
        self.track
            .as_ref()
            .map(|player_track| player_track.sound_handle.state())
    }

    pub fn has_preloaded_track(&self) -> bool {
        self.preloaded_track.is_some()
    }

    fn set_media_controls_playing(&mut self, progress: f64) -> Result<(), souvlaki::Error> {
        let progress = Self::progress_as_position(progress);
        self.controls
            .set_playback(MediaPlayback::Playing { progress })
    }

    fn set_media_controls_paused(&mut self, progress: f64) -> Result<(), souvlaki::Error> {
        let progress = Self::progress_as_position(progress);
        self.controls
            .set_playback(MediaPlayback::Paused { progress })
    }

    fn progress_as_position(progress: f64) -> Option<souvlaki::MediaPosition> {
        Some(souvlaki::MediaPosition(std::time::Duration::from_secs_f64(
            progress,
        )))
    }
}
