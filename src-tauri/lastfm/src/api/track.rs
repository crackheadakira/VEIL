use std::collections::HashMap;

use reqwest::Method;

use crate::{models::APIMethod, LastFM, LastFMError, LastFMParams};

pub struct Track<'a> {
    last_fm: &'a LastFM,
}

impl<'a> Track<'a> {
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    pub fn update_now_playing(&self, artist: String, track: String) -> UpdateNowPlaying {
        UpdateNowPlaying::new(self.last_fm, artist, track)
    }
}

pub struct UpdateNowPlaying<'a> {
    last_fm: &'a LastFM,
    artist: String,
    track: String,
    method: APIMethod,
}

impl<'a> UpdateNowPlaying<'a> {
    fn new(last_fm: &'a LastFM, artist: String, track: String) -> Self {
        Self {
            last_fm,
            artist,
            track,
            method: APIMethod::TrackUpdateNowPlaying,
        }
    }

    fn params(&self) -> Result<LastFMParams, LastFMError> {
        let mut params = HashMap::new();

        params.insert(String::from("artist"), self.artist.clone());
        params.insert(String::from("track"), self.track.clone());

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
