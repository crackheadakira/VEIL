use crate::{models::APIMethod, LastFM, LastFMError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Auth<'a> {
    last_fm: &'a LastFM,
}

impl<'a> Auth<'a> {
    /// Creates a new Auth object
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    /// Returns a new Auth token object
    pub fn token(&mut self) -> AuthGetToken<'_> {
        AuthGetToken::new(self.last_fm)
    }

    /// Returns a new Auth session object
    pub fn session(&mut self, token: String) -> AuthGetSession<'_> {
        AuthGetSession::new(self.last_fm, token)
    }
}

pub struct AuthGetSession<'a> {
    last_fm: &'a LastFM,
    token: String,
    method: APIMethod,
}

impl<'a> AuthGetSession<'a> {
    pub fn new(last_fm: &'a LastFM, token: String) -> Self {
        Self {
            last_fm,
            token,
            method: APIMethod::AuthGetSession,
        }
    }

    fn params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        params.insert("api_key".to_string(), self.last_fm.api_key.clone());
        params.insert("token".to_string(), self.token.clone());

        params
    }

    pub fn send(self) -> Result<AuthGetSessionResponse, LastFMError> {
        let mut session_params = self.params();
        let response = self
            .last_fm
            .send_request(true, self.method, &mut session_params)?;

        Ok(response)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthGetSessionResponse {
    pub session: Session,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub name: String,
    pub key: String,
}

pub struct AuthGetToken<'a> {
    last_fm: &'a LastFM,
    method: APIMethod,
}

impl<'a> AuthGetToken<'a> {
    fn new(last_fm: &'a LastFM) -> Self {
        Self {
            last_fm,
            method: APIMethod::AuthGetToken,
        }
    }

    fn params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        params.insert(String::from("api_key"), self.last_fm.api_key.clone());

        params
    }

    pub fn send(self) -> Result<AuthGetTokenResponse, LastFMError> {
        let mut token_params = self.params();
        let response = self
            .last_fm
            .send_request(true, self.method, &mut token_params)?;

        Ok(response)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthGetTokenResponse {
    pub token: String,
}
