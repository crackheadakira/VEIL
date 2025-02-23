use crate::{
    models::{APIMethod, Image},
    LastFM, LastFMError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct User<'a> {
    last_fm: &'a LastFM,
}

impl<'a> User<'a> {
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    pub fn info(&self, user: Option<String>) -> GetUserInfo<'_> {
        GetUserInfo::new(self.last_fm, user)
    }
}

pub struct GetUserInfo<'a> {
    last_fm: &'a LastFM,
    user: Option<String>,
    method: APIMethod,
}

impl<'a> GetUserInfo<'a> {
    pub fn new(last_fm: &'a LastFM, user: Option<String>) -> Self {
        Self {
            last_fm,
            user,
            method: APIMethod::UserGetInfo,
        }
    }

    fn params(&self) -> Result<HashMap<String, String>, LastFMError> {
        let mut params = HashMap::new();

        if let Some(u) = self.user.clone() {
            params.insert(String::from("user"), u);
        } else {
            let session_key = self.last_fm.session_key.clone();
            if session_key.is_none() {
                return Err(LastFMError::MissingAuthentication);
            };
            params.insert(String::from("sk"), session_key.unwrap());
        };

        params.insert(String::from("api_key"), self.last_fm.api_key.clone());

        Ok(params)
    }

    pub fn send(self) -> Result<UserInfoResponse, LastFMError> {
        let mut params = self.params()?;
        let response = self.last_fm.send_request(true, self.method, &mut params)?;

        Ok(response)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfoResponse {
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
