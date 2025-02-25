use std::collections::HashMap;

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

        let session_key = self.last_fm.session_key.clone();
        params.insert(
            String::from("sk"),
            session_key.ok_or(LastFMError::MissingAuthentication)?,
        );

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
                eprintln!("Unexpected LastFM JSON error encountered: {:?}", e);
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

        for (index, track) in self.tracks.iter().enumerate() {
            let artist_key = format!("artist[{}]", index);
            let track_key = format!("track[{}]", index);

            params.insert(artist_key, track.artist.clone());
            params.insert(track_key, track.name.clone());
        }

        let session_key = self.last_fm.session_key.clone();
        params.insert(
            String::from("sk"),
            session_key.ok_or(LastFMError::MissingAuthentication)?,
        );

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
                eprintln!("Unexpected LastFM JSON error encountered: {:?}", e);
                Err(LastFMError::JsonError(e)) // Propagate the error
            }

            Err(e) => Err(e), // Propagate all other non-JsonError errors
        }
    }
}
