use crate::Result;
use kira::{
    AudioManager, DefaultBackend, Tween,
    clock::{ClockHandle, ClockTime},
    sound::{
        FromFileError, PlaybackState,
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
};

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

pub trait Clock {
    /// Starts the clock.
    ///
    /// See [`ClockHandle::start`] for detailed behavior.
    fn start(&mut self);

    /// Pauses the clock.
    ///
    /// See [`ClockHandle::pause`] for detailed behavior.
    fn pause(&mut self);

    /// Stops the clock.
    ///
    /// See [`ClockHandle::stop`] for detailed behavior.
    fn stop(&mut self);

    /// Returns the current time from the clock.
    ///
    /// See [`ClockHandle::time`] for detailed behavior.
    fn time(&self) -> ClockTime;
}

impl Clock for ClockHandle {
    fn start(&mut self) {
        self.start();
    }

    fn pause(&mut self) {
        self.pause();
    }

    fn stop(&mut self) {
        self.stop();
    }

    fn time(&self) -> ClockTime {
        self.time()
    }
}
