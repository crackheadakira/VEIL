use std::collections::VecDeque;

use logging::lock_or_log;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager};

use crate::{
    SodapopState, config::SodapopConfigEvent, error::FrontendError, events::EventSystemHandler,
};

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

    /// Gets set to true when the personal queue is consumed
    /// then immediately to false when going to global queue.
    personal_consumed: bool,

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
            personal_consumed: false,
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
        (self.current_index - 1) % self.global_queue.len()
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

                    if self.personal_queue.is_empty() {
                        self.personal_consumed = true;
                    }

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

        if self.personal_consumed {
            let idx = self.current_index;
            let track = self.global_queue[idx];

            if let Mode::Consume = mode {
                self.personal_consumed = false;
            }

            return Some(track);
        }

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

    /// Get the track at the current index
    pub fn current(&self) -> Option<u32> {
        if !self.personal_queue.is_empty() {
            self.personal_queue.front().copied()
        } else {
            if self.global_queue.is_empty() {
                return None;
            }

            Some(self.global_queue[self.current_index])
        }
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
        self.current_index = new_index % self.global_queue.len();
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

#[derive(Serialize, Deserialize, Type, tauri_specta::Event, Clone)]
#[serde(tag = "type", content = "data")]
pub enum QueueEvent {
    /// Add to personal queue via context menu
    EnqueuePersonal { track_id: u32 },

    SetGlobalQueue {
        tracks: Vec<u32>,
        queue_idx: usize,
        origin: QueueOrigin,
    },
}

impl EventSystemHandler for QueueEvent {
    async fn handle(
        event: tauri_specta::TypedEvent<QueueEvent>,
        handle: &AppHandle,
    ) -> Result<(), FrontendError> {
        match event.payload {
            QueueEvent::EnqueuePersonal { track_id } => {
                Self::enqueue_personal_track(handle, track_id)?
            }
            QueueEvent::SetGlobalQueue {
                tracks,
                queue_idx,
                origin,
            } => Self::set_global_queue(handle, tracks, queue_idx, origin)?,
        }

        Ok(())
    }
}

impl QueueEvent {
    fn enqueue_personal_track(handle: &AppHandle, track_id: u32) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex")?;

        queue.enqueue_personal(track_id);
        Ok(())
    }

    fn set_global_queue(
        handle: &AppHandle,
        tracks: Vec<u32>,
        queue_idx: usize,
        origin: QueueOrigin,
    ) -> Result<(), FrontendError> {
        let state = handle.state::<SodapopState>();
        let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex")?;
        let mut config = lock_or_log(state.config.write(), "Config Write Lock")?;

        if let Some(original_origin) = queue.origin
            && original_origin != origin
        {
            queue.set_global(tracks);
            queue.set_origin(origin);

            config.update_config(SodapopConfigEvent {
                queue_origin: Some(origin),
                ..SodapopConfigEvent::default()
            })?;
        }

        if queue.current_index() != queue_idx {
            queue.set_current_index(queue_idx);
            config.update_config(SodapopConfigEvent {
                queue_idx: Some(queue_idx),
                ..SodapopConfigEvent::default()
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_queue_origin() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);
        assert_eq!(queue.origin, None);

        queue.set_origin(QueueOrigin::Album { id: 5 });
        assert_eq!(queue.origin, Some(QueueOrigin::Album { id: 5 }))
    }

    #[test]
    fn consuming_personal_queue() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);
        queue.enqueue_personal(50); // pushes to back [50]
        queue.enqueue_personal(20); // pushes to back [50, 20]
        queue.set_global(vec![0, 1, 2]);

        assert_eq!(queue.current_index, 0);

        //          ↓ idx after
        // [50] <- [50, 20], [0, 1, 2]
        //          ↑ idx before
        assert_eq!(queue.peek_next(), Some(50));
        assert_eq!(queue.current_index, 0);

        //                ↓ idx after (consumed)
        // [50] <- [(50), 20], [0, 1, 2]
        //          ↑ idx before
        assert_eq!(queue.next(), Some(50));
        assert_eq!(queue.current_index, 0);

        //                ↓ peeking after
        // [20] <- [(50), 20], [0, 1, 2]
        //                ↑ idx before
        assert_eq!(queue.peek_next(), Some(20));
        assert_eq!(queue.current_index, 0);

        //          ↓ idx after(consumed)
        // [20] <- [ ]->[0, 1, 2]
        //          ↑ idx before
        assert_eq!(queue.next(), Some(20));
        assert_eq!(queue.current_index, 0);

        //              ↓ peeking at
        // [0] <- [ ]->[0, 1, 2]
        //         ↑ idx
        assert_eq!(queue.peek_next(), Some(0));
        assert_eq!(queue.current_index, 0);

        //              ↓ idx after
        // [0] <- [ ]->[0, 1, 2]
        //          ↑ idx before
        assert_eq!(queue.next(), Some(0));
        assert_eq!(queue.current_index, 0);

        //                 ↓ peeking at
        // [1] <- [ ]->[0, 1, 2]
        //              ↑ idx
        assert_eq!(queue.peek_next(), Some(1));
        assert_eq!(queue.current_index, 0);

        //                 ↓ idx after
        // [1] <- [ ]->[0, 1, 2]
        //              ↑ idx before
        assert_eq!(queue.next(), Some(1));
        assert_eq!(queue.current_index, 1);
    }

    #[test]
    fn traversing_global_queue() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);
        queue.set_global(vec![0, 1, 2]);

        assert_eq!(queue.current_index, 0);

        //            ↓ peeking at
        // [1] <- [0, 1, 2]
        //         ↑ idx
        assert_eq!(queue.peek_next(), Some(1));
        assert_eq!(queue.current_index, 0);

        //            ↓ idx after
        // [1] <- [0, 1, 2]
        //         ↑ idx before
        assert_eq!(queue.next(), Some(1));
        assert_eq!(queue.current_index, 1);

        //               ↓ peeking at
        // [2] <- [0, 1, 2]
        //            ↑ idx
        assert_eq!(queue.peek_next(), Some(2));
        assert_eq!(queue.current_index, 1);

        //               ↓ idx after
        // [2] <- [0, 1, 2]
        //            ↑ idx before
        assert_eq!(queue.next(), Some(2));
        assert_eq!(queue.current_index, 2);
    }

    #[test]
    fn wrapping_idx_set() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);
        queue.set_global(vec![0, 1, 2]);
        assert_eq!(queue.current_index, 0);

        queue.set_current_index(3);
        assert_eq!(queue.current(), Some(0));
        assert_eq!(queue.current_index, 0);

        queue.set_current_index(4);
        assert_eq!(queue.current(), Some(1));
        assert_eq!(queue.current_index, 1);
    }

    #[test]
    fn empty_queue() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);

        assert_eq!(queue.previous(), None);
        assert_eq!(queue.current(), None);
        assert_eq!(queue.next(), None);
    }

    #[test]
    fn shuffle_global_changes_order() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);
        queue.set_global(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let original = queue.global_queue.clone();
        queue.shuffle_global();

        assert_eq!(queue.global_queue.len(), original.len());
        assert_ne!(
            queue.global_queue, original,
            "Expected shuffled order to differ"
        );
    }

    #[test]
    fn shuffled_next_updates_index_randomly() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);
        queue.set_global(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        queue.shuffle = true;

        queue.next();

        assert!(queue.current_index < queue.global_queue.len());
    }

    #[test]
    fn repeat_none_behaves_normally() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::None);
        queue.set_global(vec![0, 1, 2]);
        assert_eq!(queue.next(), Some(1));
        assert_eq!(queue.next(), Some(2));
        assert_eq!(queue.next(), Some(0));
    }

    #[test]
    fn repeat_track_keeps_current() {
        let mut queue = QueueSystem::new(0x12345678, None, RepeatMode::Track);
        queue.set_global(vec![0, 1, 2]);
        let first = queue.current();
        let next = queue.next();
        assert_ne!(first, next, "RepeatMode::Track not implemented yet");
    }
}
