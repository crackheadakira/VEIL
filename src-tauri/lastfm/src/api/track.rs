use std::collections::HashMap;

use reqwest::Method;
use serde::Deserialize;

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

    pub async fn send(self) -> Result<TrackInfo, LastFMError> {
        let mut params = self.params()?;
        let result = self
            .last_fm
            .send_request::<TrackInfo>(Method::POST, self.method, &mut params)
            .await?;

        Ok(result)
    }
}

#[derive(Deserialize, Debug)]
pub struct TrackInfo {
    #[serde(rename = "nowplaying")]
    pub now_playing: NowPlaying,
}

#[derive(Deserialize, Debug)]
pub struct NowPlaying {
    pub album: Album,
    #[serde(rename = "albumArtist")]
    pub album_artist: AlbumArtist,
    pub artist: Artist,
    #[serde(rename = "ignoredMessage")]
    pub ignored_message: IgnoredMessage,
    pub track: TrackStruct,
}

#[derive(Deserialize, Debug)]
pub struct Album {
    #[serde(rename = "#text")]
    pub text: String,
    pub corrected: String,
}

#[derive(Deserialize, Debug)]
pub struct AlbumArtist {
    #[serde(rename = "#text")]
    pub text: String,
    pub corrected: String,
}

#[derive(Deserialize, Debug)]
pub struct Artist {
    #[serde(rename = "#text")]
    pub text: String,
    pub corrected: String,
}

#[derive(Deserialize, Debug)]
pub struct IgnoredMessage {
    #[serde(rename = "#text")]
    pub text: String,
    pub code: String,
}

#[derive(Deserialize, Debug)]
pub struct TrackStruct {
    #[serde(rename = "#text")]
    pub text: String,
    pub corrected: String,
}
