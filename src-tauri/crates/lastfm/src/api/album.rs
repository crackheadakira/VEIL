use crate::{
    LastFM, LastFMError, LastFMParams,
    models::{APIMethod, Image},
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

pub struct Album<'a> {
    last_fm: &'a LastFM,
}

impl<'a> Album<'a> {
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    /// Returns a new user info token handler
    pub fn info(&self, album: &'a str, artist: &'a str) -> GetAlbumInfo<'_> {
        GetAlbumInfo::new(self.last_fm, album, artist)
    }
}

pub struct GetAlbumInfo<'a> {
    last_fm: &'a LastFM,
    album: &'a str,
    artist: &'a str,
    method: APIMethod,
}

impl<'a> GetAlbumInfo<'a> {
    fn new(last_fm: &'a LastFM, album: &'a str, artist: &'a str) -> Self {
        Self {
            last_fm,
            artist,
            album,
            method: APIMethod::AlbumGetInfo,
        }
    }

    fn params(&'_ self) -> Result<LastFMParams<'_>, LastFMError> {
        let mut params = HashMap::new();

        params.insert("album", Cow::from(self.album));
        params.insert("artist", Cow::from(self.artist));

        Ok(params)
    }

    pub async fn send(self) -> Result<AlbumSearch, LastFMError> {
        let mut params = self.params()?;
        let response: AlbumSearchResponse = self
            .last_fm
            .send_request(Method::GET, &self.method, &mut params)
            .await?;
        Ok(response.album)
    }
}

#[derive(Serialize, Deserialize)]
struct AlbumSearchResponse {
    pub album: AlbumSearch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumSearch {
    pub artist: String,

    #[serde(default)]
    pub mbid: Option<String>,

    #[serde(alias = "playcount", default)]
    pub play_count: Option<String>,

    #[serde(default)]
    pub image: Vec<Image>,

    pub url: String,

    pub name: String,

    #[serde(default)]
    pub listeners: Option<String>,
}
