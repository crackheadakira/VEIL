use crate::{
    models::{APIMethod, Image},
    LastFM, LastFMError, LastFMParams,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct User<'a> {
    last_fm: &'a LastFM,
}

impl<'a> User<'a> {
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    /// Returns a new user info token handler
    pub fn info(&self, username: Option<String>) -> GetUserInfo<'_> {
        GetUserInfo::new(self.last_fm, username)
    }
}

pub struct GetUserInfo<'a> {
    last_fm: &'a LastFM,
    user: Option<String>,
    method: APIMethod,
}

impl<'a> GetUserInfo<'a> {
    fn new(last_fm: &'a LastFM, user: Option<String>) -> Self {
        Self {
            last_fm,
            user,
            method: APIMethod::UserGetInfo,
        }
    }

    fn params(&self) -> Result<LastFMParams, LastFMError> {
        let mut params = HashMap::new();

        if let Some(u) = self.user.clone() {
            params.insert(String::from("user"), u);
        } else {
            let session_key = self.last_fm.session_key.clone();
            params.insert(
                String::from("sk"),
                session_key.ok_or(LastFMError::MissingAuthentication)?,
            );
        };

        params.insert(String::from("api_key"), self.last_fm.api_key.clone());

        Ok(params)
    }

    pub async fn send(self) -> Result<UserInfo, LastFMError> {
        let mut params = self.params()?;
        let response: UserInfoResponse = self
            .last_fm
            .send_request(Method::GET, self.method, &mut params)
            .await?;
        Ok(response.user)
    }
}

#[derive(Serialize, Deserialize)]
struct UserInfoResponse {
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    name: String,
    age: String,
    subscriber: String,
    #[serde(alias = "realname")]
    real_name: String,
    bootstrap: String,
    #[serde(alias = "playcount")]
    play_count: String,
    artist_count: String,
    playlists: String,
    track_count: String,
    album_count: String,
    image: Vec<Image>,
    country: String,
    gender: String,
    url: String,
}
