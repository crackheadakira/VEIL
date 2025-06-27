use std::{collections::HashMap, time::UNIX_EPOCH};

use reqwest::Method;

use crate::{
    models::{APIMethod, TrackData},
    LastFM, LastFMError, LastFMParams,
};

pub struct Track<'a> {
    last_fm: &'a LastFM,
}

impl<'a> Track<'a> {
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    pub fn update_now_playing(&self, track: TrackData) -> UpdateNowPlaying {
        UpdateNowPlaying::new(self.last_fm, track)
    }

    pub fn scrobble(&self, tracks: Vec<TrackData>) -> TrackScrobble {
        TrackScrobble::new(self.last_fm, tracks)
    }
}

pub struct UpdateNowPlaying<'a> {
    last_fm: &'a LastFM,
    track: TrackData,
    method: APIMethod,
}

impl<'a> UpdateNowPlaying<'a> {
    fn new(last_fm: &'a LastFM, track: TrackData) -> Self {
        Self {
            last_fm,
            track,
            method: APIMethod::TrackUpdateNowPlaying,
        }
    }

    fn params(&self) -> Result<LastFMParams, LastFMError> {
        let mut params = HashMap::new();

        params.insert(String::from("artist"), self.track.artist.clone());
        params.insert(String::from("track"), self.track.name.clone());

        Ok(params)
    }

    pub async fn send(self) -> Result<(), LastFMError> {
        let mut params = self.params()?;
        let result = self
            .last_fm
            .send_request::<()>(Method::POST, self.method, &mut params)
            .await;

        match result {
            Ok(_) => Ok(()),

            // ignore this specific error, as we're passing in `()` as type
            // so it'll always return this
            Err(LastFMError::JsonError(ref e))
                if e.to_string() == "invalid type: map, expected unit" =>
            {
                Ok(())
            }

            Err(LastFMError::JsonError(e)) => {
                Err(LastFMError::JsonError(e)) // Propagate the error
            }

            Err(e) => Err(e), // Propagate all other non-JsonError errors
        }
    }
}

pub struct TrackScrobble<'a> {
    last_fm: &'a LastFM,
    tracks: Vec<TrackData>,
    method: APIMethod,
}

impl<'a> TrackScrobble<'a> {
    fn new(last_fm: &'a LastFM, tracks: Vec<TrackData>) -> Self {
        Self {
            last_fm,
            tracks,
            method: APIMethod::TrackScrobble,
        }
    }

    fn params(&self) -> Result<LastFMParams, LastFMError> {
        let mut params = HashMap::new();

        if self.tracks.len() > 50 {
            return Err(LastFMError::BatchScrobble);
        }

        let current_timestamp = UNIX_EPOCH.elapsed().expect("Time went backwards");

        if self.tracks.len() > 1 {
            for (index, track) in self.tracks.iter().enumerate() {
                let artist_key = format!("artist[{index}]");
                let track_key = format!("track[{index}]");
                let timestamp_key = format!("timestamp[{index}]");

                if let Some(album) = track.album.clone() {
                    let album_key: String = format!("album[{index}]");
                    params.insert(album_key, album);
                };

                params.insert(artist_key, track.artist.clone());
                params.insert(track_key, track.name.clone());
                params.insert(
                    timestamp_key,
                    track
                        .timestamp
                        .unwrap_or(current_timestamp.as_secs() as i64)
                        .to_string(),
                );
            }
        } else if let Some(track) = self.tracks.first() {
            if let Some(album) = track.album.clone() {
                params.insert("album".to_string(), album);
            };
            params.insert("artist".to_string(), track.artist.clone());
            params.insert("track".to_string(), track.name.clone());
            params.insert(
                "timestamp".to_string(),
                track
                    .timestamp
                    .unwrap_or(current_timestamp.as_secs() as i64)
                    .to_string(),
            );
        }

        Ok(params)
    }

    pub async fn send(self) -> Result<(), LastFMError> {
        let mut params = self.params()?;
        let result = self
            .last_fm
            .send_request::<()>(Method::POST, self.method, &mut params)
            .await;

        match result {
            Ok(_) => Ok(()),

            // ignore this specific error, as we're passing in `()` as type
            // so it'll always return this
            Err(LastFMError::JsonError(ref e))
                if e.to_string() == "invalid type: map, expected unit" =>
            {
                Ok(())
            }

            Err(LastFMError::JsonError(e)) => {
                Err(LastFMError::JsonError(e)) // Propagate the error
            }

            Err(e) => Err(e), // Propagate all other non-JsonError errors
        }
    }
}
