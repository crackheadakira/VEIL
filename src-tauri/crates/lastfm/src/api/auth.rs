use crate::{LastFM, LastFMParams, Result, models::APIMethod};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};

pub struct Auth<'a> {
    last_fm: &'a LastFM,
}

impl<'a> Auth<'a> {
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    /// Returns a new auth token handler
    ///
    /// # Example
    /// Assumes you have already built [`LastFM`] and are
    /// holding it under the variable `last_fm`
    /// ```ignore
    /// use lastfm::Auth;
    ///
    /// // Request a token from Last.FM that can be later
    /// // passed onto the session.
    /// let res = last_fm.auth().token().send()?;
    /// ```
    pub fn token(&self) -> AuthGetToken<'_> {
        AuthGetToken::new(self.last_fm)
    }

    /// Returns a new auth session handler
    ///
    /// # Example
    /// Assumes you have already built [`LastFM`] and are
    /// holding it under the variable `last_fm`
    /// ```ignore
    /// use lastfm::Auth;
    ///
    /// // User has already authorized the token
    /// let res = last_fm.auth().session(token).send()?;
    ///
    /// ```
    pub fn session(&self, token: &'a str) -> AuthGetSession<'_> {
        AuthGetSession::new(self.last_fm, token)
    }
}

pub struct AuthGetSession<'a> {
    last_fm: &'a LastFM,
    token: &'a str,
    method: APIMethod,
}

impl<'a> AuthGetSession<'a> {
    fn new(last_fm: &'a LastFM, token: &'a str) -> Self {
        Self {
            last_fm,
            token,
            method: APIMethod::AuthGetSession,
        }
    }

    fn params(&self) -> LastFMParams<'a> {
        let mut params = HashMap::new();

        params.insert("api_key", Cow::Borrowed(self.last_fm.api_key.as_str()));
        params.insert("token", self.token.into());

        params
    }

    pub async fn send(self) -> Result<AuthGetSessionResponse> {
        let mut session_params = self.params();

        let response = self
            .last_fm
            .send_request(Method::GET, &self.method, &mut session_params)
            .await?;

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

    fn params(&self) -> LastFMParams<'a> {
        let mut params = HashMap::new();

        params.insert("api_key", self.last_fm.api_key.clone().into());

        params
    }

    pub async fn send(self) -> Result<AuthGetTokenResponse> {
        let mut token_params = self.params();
        let response = self
            .last_fm
            .send_request(Method::GET, &self.method, &mut token_params)
            .await?;

        Ok(response)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthGetTokenResponse {
    pub token: String,
}
