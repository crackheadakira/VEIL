use crate::Result;
use kira::{
    AudioManager, DefaultBackend, Tween,
    sound::{
        FromFileError, PlaybackState,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
};
use souvlaki::MediaControls;

pub trait Sound {
    /// Pauses playback.
    ///
    /// See [`StreamingSoundHandle::pause`] for detailed behavior.
    fn pause(&mut self, tween: Tween);

    /// Resumes playback.
    ///
    /// See [`StreamingSoundHandle::resume`] for detailed behavior.
    fn resume(&mut self, tween: Tween);

    /// Sets playback to specified position.
    ///
    /// See [`StreamingSoundHandle::seek_to`] for detailed behavior.
    fn seek_to(&mut self, pos: f64);

    /// Stops playback.
    ///
    /// See [`StreamingSoundHandle::stop`] for detailed behavior.
    fn stop(&mut self, tween: Tween);

    /// Returns current playback position.
    ///
    /// See [`StreamingSoundHandle::position`] for detailed behavior.
    fn position(&self) -> f64;

    /// Returns the current playback state.
    ///
    /// See [`StreamingSoundHandle::state`] for detailed behavior.
    fn state(&self) -> PlaybackState;

    /// Sets the playback volume.
    ///
    /// See [`StreamingSoundHandle::set_volume`] for detailed behavior.
    fn set_volume(&mut self, volume: f32, tween: Tween);
}

pub trait SoundFactory {
    type Sound: Sound;

    fn create(
        &mut self,
        path: &str,
        start_position: f64,
        volume: f32,
        tween: Tween,
    ) -> Result<(Self::Sound, f64)>;
}

impl Sound for StreamingSoundHandle<FromFileError> {
    fn pause(&mut self, tween: Tween) {
        self.pause(tween);
    }

    fn resume(&mut self, tween: Tween) {
        self.resume(tween);
    }

    fn seek_to(&mut self, pos: f64) {
        self.seek_to(pos);
    }

    fn stop(&mut self, tween: Tween) {
        self.stop(tween);
    }

    fn position(&self) -> f64 {
        self.position()
    }

    fn state(&self) -> PlaybackState {
        self.state()
    }

    fn set_volume(&mut self, volume: f32, tween: Tween) {
        self.set_volume(volume, tween);
    }
}

pub trait SouvlakiControls {
    fn set_volume(&mut self, volume: f64) -> Result<(), souvlaki::Error>;
    fn set_metadata(&mut self, metadata: souvlaki::MediaMetadata) -> Result<(), souvlaki::Error>;
    fn set_playback(&mut self, playback: souvlaki::MediaPlayback) -> Result<(), souvlaki::Error>;
}

impl SouvlakiControls for MediaControls {
    fn set_volume(&mut self, volume: f64) -> Result<(), souvlaki::Error> {
        self.set_volume(volume)
    }

    fn set_metadata(&mut self, metadata: souvlaki::MediaMetadata) -> Result<(), souvlaki::Error> {
        self.set_metadata(metadata)
    }

    fn set_playback(&mut self, playback: souvlaki::MediaPlayback) -> Result<(), souvlaki::Error> {
        self.set_playback(playback)
    }
}

pub struct FakeMediaControls;
impl SouvlakiControls for FakeMediaControls {
    fn set_volume(&mut self, _volume: f64) -> Result<(), souvlaki::Error> {
        Ok(())
    }

    fn set_metadata(&mut self, _metadata: souvlaki::MediaMetadata) -> Result<(), souvlaki::Error> {
        Ok(())
    }

    fn set_playback(&mut self, _playback: souvlaki::MediaPlayback) -> Result<(), souvlaki::Error> {
        Ok(())
    }
}

pub struct KiraSoundFactory {
    pub manager: AudioManager<DefaultBackend>,
}

impl SoundFactory for KiraSoundFactory {
    type Sound = StreamingSoundHandle<FromFileError>;

    fn create(
        &mut self,
        path: &str,
        start_position: f64,
        volume: f32,
        tween: Tween,
    ) -> Result<(Self::Sound, f64)> {
        let sound_data = StreamingSoundData::from_file(path)?.start_position(start_position);
        let track_duration = sound_data.duration().as_secs_f64();

        let mut handle = self.manager.play(sound_data)?;
        handle.set_volume(volume, tween);
        handle.pause(tween);

        Ok((handle, track_duration))
    }
}

pub struct FakeSoundHandle {
    pub pos: f64,
    pub state: PlaybackState,
    pub volume: f32,
}

impl Sound for FakeSoundHandle {
    fn pause(&mut self, _tween: Tween) {
        self.state = PlaybackState::Paused;
    }

    fn resume(&mut self, _tween: Tween) {
        self.state = PlaybackState::Playing;
    }

    fn seek_to(&mut self, pos: f64) {
        self.pos = pos;
    }

    fn stop(&mut self, _tween: Tween) {
        self.state = PlaybackState::Stopped;
    }

    fn position(&self) -> f64 {
        self.pos
    }

    fn state(&self) -> PlaybackState {
        self.state
    }

    fn set_volume(&mut self, volume: f32, _tween: Tween) {
        self.volume = volume;
    }
}

#[derive(Default)]
pub struct FakeSoundFactory;
impl crate::seams::SoundFactory for FakeSoundFactory {
    type Sound = FakeSoundHandle;

    fn create(
        &mut self,
        _path: &str,
        start_position: f64,
        volume: f32,
        _tween: Tween,
    ) -> Result<(Self::Sound, f64)> {
        Ok((
            FakeSoundHandle {
                pos: start_position,
                state: PlaybackState::Paused,
                volume,
            },
            120.0,
        ))
    }
}
