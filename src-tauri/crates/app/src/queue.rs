use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Copy, Clone)]
#[serde(tag = "type", content = "data")]
pub enum QueueOrigin {
    Playlist { id: u32 },
    Album { id: u32 },
}

pub struct QueueSystem {
    personal_queue: VecDeque<u32>,
    global_queue: Vec<u32>,
    pub origin: Option<QueueOrigin>,

    pub shuffle: bool,

    /// Index into the global queue
    pub current_index: usize,

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
    pub fn new(rng_state: u32, origin: Option<QueueOrigin>) -> Self {
        Self {
            personal_queue: VecDeque::with_capacity(50),
            global_queue: Vec::with_capacity(50),
            shuffle: false,
            current_index: 0,
            origin,
            rng_state,
        }
    }

    /// Set queue origin
    pub fn set_origin(&mut self, origin: Option<QueueOrigin>) {
        self.origin = origin;
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

    /// Get the next track
    pub fn next(&mut self) -> Option<u32> {
        if let Some(track) = self.personal_queue.pop_front() {
            logging::debug!("Fetching next track from personal queue");
            Some(track)
        } else if !self.global_queue.is_empty() {
            let track = if self.shuffle {
                logging::debug!("Fetching next track from shuffled global queue");
                let idx = self.rand() % self.global_queue.len();
                self.global_queue[idx]
            } else {
                logging::debug!("Fetching next track from non-shuffled global queue");
                let track = self.global_queue[self.current_index % self.global_queue.len()];
                self.current_index = (self.current_index + 1) % self.global_queue.len();
                track
            };

            Some(track)
        } else {
            None
        }
    }

    /// Peek at the next track, and consumes if it's from personal queue
    pub fn peek_next(&mut self) -> Option<u32> {
        if let Some(track) = self.personal_queue.pop_front() {
            logging::debug!("Peeking at next track from personal queue");
            Some(track)
        } else if !self.global_queue.is_empty() {
            if self.shuffle {
                logging::debug!("Peeking at next track from shuffled global queue");
                let idx = self.rand() % self.global_queue.len();
                Some(self.global_queue[idx])
            } else {
                logging::debug!("Peeking at next track from non-shuffled global queue");
                Some(self.global_queue[self.current_index % self.global_queue.len()])
            }
        } else {
            None
        }
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
