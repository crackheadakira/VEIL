use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Copy, Clone, Debug, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum QueueOrigin {
    Playlist { id: u32 },
    Album { id: u32 },
}

#[derive(Copy, Clone, Serialize, Deserialize, Type, Default)]
pub enum RepeatMode {
    /// Do not repeat anything when the end of the queue is hit.
    #[default]
    None,

    /// Repeat the queue when the end of the queue is hit.
    Queue,

    /// Repeat the track when the end of the track is hit.
    Track,
}

#[derive(Copy, Clone)]
enum Direction {
    Next,
    Previous,
}

#[derive(Copy, Clone)]
enum Mode {
    Peek,
    Consume,
}

pub struct QueueSystem {
    personal_queue: VecDeque<u32>,
    global_queue: Vec<u32>,
    pub origin: Option<QueueOrigin>,
    pub shuffle: bool,
    pub repeat_mode: RepeatMode,

    /// Index into the global queue
    current_index: usize,

    /// Internal state for PRNG
    rng_state: u32,
}

// TODO: Currently with preloading a track, how do we handle the
// scenario that a track is already preloaded, but a user adds
// a song to the queue last millisecond?

// 1. Do we just add it to the queue as the next song (easiest) --> makes most sense
//      we will already have consumed the personal queue track at this point.
//
// 2. Somehow quickly preload the next one (difficult)
//      Would have to unconsume the personal queue track by simply setting an index
//      rather than actually consuming.

// TODO: Support sorting based on different criteria --> would depend on queries and
// how to fetch it from database.

// TODO: How do we handle recreating the queue on boot-up, i.e. how do we know if the queue
// is from playlist or album & how do we remember the current track --> store in config?

impl QueueSystem {
    pub fn new(rng_state: u32, origin: Option<QueueOrigin>, repeat_mode: RepeatMode) -> Self {
        Self {
            personal_queue: VecDeque::with_capacity(50),
            global_queue: Vec::with_capacity(50),
            shuffle: false,
            repeat_mode,
            current_index: 0,
            origin,
            rng_state,
        }
    }

    /// Set queue origin
    pub fn set_origin(&mut self, origin: QueueOrigin) {
        logging::debug!("Setting queue origin to {origin:?}");
        self.origin = Some(origin);
    }

    /// Add a track to the personal queue
    pub fn enqueue_personal(&mut self, track_id: u32) {
        logging::debug!("Enqueued track {track_id} to personal queue");
        self.personal_queue.push_back(track_id);
    }

    /// Add a track to the global queue
    pub fn enqueue_global(&mut self, track_id: u32) {
        logging::debug!("Enqueued track {track_id} to global queue");
        self.global_queue.push(track_id);
    }

    /// Mass-replace whole global queue
    pub fn set_global(&mut self, new_global: Vec<u32>) {
        logging::debug!("Set global queue to a vec of length {}", new_global.len());
        self.global_queue = new_global;
    }

    /// Internal method to get the previous index.
    ///
    /// Handles the repeat methods as well
    fn get_previous_index(&self) -> usize {
        (self.current_index + 1) % self.global_queue.len()
    }

    /// Internal method to get the next index.
    ///
    /// Handles the repeat methods as well.
    fn get_next_index(&self) -> usize {
        (self.current_index + 1) % self.global_queue.len()
    }

    /// Internal method to get a track from the queue.
    fn get_track(&mut self, dir: Direction, mode: Mode) -> Option<u32> {
        // As personal queue is meant to be consumed, it does not have a way
        // of checking the previous direction.
        match mode {
            Mode::Consume => {
                if let Some(track) = self.personal_queue.pop_front() {
                    logging::debug!("Consuming next track from personal queue");
                    return Some(track);
                }
            }
            Mode::Peek => {
                if let Some(track) = self.personal_queue.front() {
                    logging::debug!("Peeking at next track from personal queue");
                    return Some(*track);
                }
            }
        };

        // If there is no queue simply return.
        if self.global_queue.is_empty() {
            return None;
        };

        let track = if self.shuffle {
            // Currently as we don't shuffle the global queue this regardless
            // of direction will return a random track.
            let idx = self.rand() % self.global_queue.len();
            logging::debug!(
                "{} next track from shuffled global queue at index {idx}",
                match mode {
                    Mode::Peek => "Peeking",
                    Mode::Consume => "Consuming",
                }
            );

            if let Mode::Consume = mode {
                self.set_current_index(idx);
            };

            self.global_queue[idx]
        } else {
            let idx = match dir {
                Direction::Next => self.get_next_index(),
                Direction::Previous => self.get_previous_index(),
            };

            logging::debug!(
                "{} track from non-shuffled global queue at index {idx}",
                match mode {
                    Mode::Peek => "Peeking",
                    Mode::Consume => "Consuming",
                }
            );

            let track = self.global_queue[idx];

            if let Mode::Consume = mode {
                self.set_current_index(idx);
            };

            track
        };

        Some(track)
    }

    /// Get the next track
    pub fn next(&mut self) -> Option<u32> {
        self.get_track(Direction::Next, Mode::Consume)
    }

    /// Peek at the next track
    pub fn peek_next(&mut self) -> Option<u32> {
        self.get_track(Direction::Next, Mode::Peek)
    }

    /// Get the previous track
    pub fn previous(&mut self) -> Option<u32> {
        self.get_track(Direction::Previous, Mode::Consume)
    }

    /// Peek at the previous track
    pub fn peek_previous(&mut self) -> Option<u32> {
        self.get_track(Direction::Previous, Mode::Peek)
    }

    pub fn current_index(&self) -> usize {
        self.current_index
    }

    pub fn set_current_index(&mut self, new_index: usize) {
        logging::debug!("Setting index from {} to {new_index}", self.current_index);
        self.current_index = new_index;
    }

    /// Check if the queues are empty
    pub fn is_empty(&self) -> bool {
        self.personal_queue.is_empty() && self.global_queue.is_empty()
    }

    pub fn shuffle_global(&mut self) {
        logging::debug!("Shuffling global queue");
        let len = self.global_queue.len();
        for i in (1..len).rev() {
            let j = self.rand() % (i + 1);
            self.global_queue.swap(i, j);
        }
    }

    /// Simple pseudo-random function
    fn rand(&mut self) -> usize {
        self.rng_state = self
            .rng_state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        (self.rng_state % 0xFFFFFFFF) as usize
    }
}
