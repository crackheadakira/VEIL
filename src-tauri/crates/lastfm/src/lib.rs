//! Easy-to-use Last.FM API wrapper

mod api;
mod models;
pub use models::TrackData;

use api::*;
use models::{APIError, APIMethod};
use reqwest::Method;
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap};

/// A helper type for all parameters to have same type without any accidental change
type LastFMParams<'a> = HashMap<&'static str, Cow<'a, str>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// [`LastFMBuilder`] is missing an API key
    #[error("Missing API key")]
    MissingAPIKey,
    /// [`LastFMBuilder`] is missing an API secret
    #[error("Missing API secret")]
    MissingAPISecret,
    /// [`LastFM`] is missing the user's session key
    #[error("Missing session key")]
    MissingAuthentication,
    /// Passed an invalid HTTP method to [`LastFM::send_request`]
    #[error("Invalid HTTP method")]
    InvalidHTTPMethod,
    #[error("Sent request when disabled")]
    RequestWhenDisabled,
    #[error("Missing parameter {0}")]
    MissingParameter(String),
    #[error("Batch scrobble too large")]
    BatchScrobble,
    #[error(transparent)]
    APIError(#[from] APIError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

pub(crate) type Result<T, U = Error> = std::result::Result<T, U>;

#[derive(Clone)]
pub struct LastFM {
    /// The Last.FM API Key
    api_key: String,
    /// The Last.FM API Secret
    api_secret: String,
    /// The user's session key,
    session_key: Option<String>,
    /// The URL to send API requests to
    base_url: String,
    /// The HTTP request client
    client: reqwest::Client,
    /// If `LastFM` API should be enabled, if `false` doesn't send any HTTP requests.
    ///
    /// By default is `true`.
    enabled: bool,
}

impl<'a> LastFM {
    #[must_use]
    pub fn builder() -> LastFMBuilder {
        LastFMBuilder {
            api_key: None,
            api_secret: None,
        }
    }

    /// Sends the HTTP Request to Last.FM
    async fn http_request(
        &self,
        method: Method,
        params: &LastFMParams<'a>,
    ) -> Result<serde_json::Value> {
        let url = &self.base_url;

        let response = match method {
            Method::GET => self.client.get(url).form(&params).send().await?,
            Method::POST => {
                self.client
                    .post(url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .form(&params)
                    .send()
                    .await?
            }
            _ => return Err(Error::InvalidHTTPMethod),
        };

        let json = response.json().await?;
        Ok(json)
    }

    /// Wraps around [`LastFM::http_request`] and inserts all the required
    /// parameters into `params`
    async fn send_request<T: for<'b> Deserialize<'b>>(
        &'a self,
        method: Method,
        api_method: &APIMethod,
        params: &mut LastFMParams<'a>,
    ) -> Result<T> {
        if !self.enabled {
            return Err(Error::RequestWhenDisabled);
        }
        params.insert("method", Cow::Borrowed(api_method.as_query()));
        params.insert("api_key", Cow::Borrowed(self.api_key.as_str()));

        if api_method.need_auth() {
            params.insert(
                "sk",
                Cow::from(
                    self.session_key
                        .clone()
                        .ok_or(Error::MissingAuthentication)?,
                ),
            );
        }

        if api_method.need_sig() {
            let signature = self.sign_api(params);
            params.insert("api_sig", Cow::from(signature));
        }

        params.insert("format", Cow::from("json"));

        let res = self.http_request(method, params).await?;

        if res.get("error").is_some() {
            let error: APIError = serde_json::from_value(res)?;
            return Err(Error::APIError(error));
        }

        let response: T = serde_json::from_value(res)?;
        Ok(response)
    }

    /// Create the MD5 hash of all `params` (except format).
    ///
    /// This is needed by Last.FM to sign your API requests
    fn sign_api(&self, params: &mut LastFMParams) -> String {
        let mut sorted_keys = params.keys().copied().collect::<Vec<&str>>();
        sorted_keys.sort_unstable();

        let mut params_string = String::new();

        for key in sorted_keys {
            if let Some(v) = params.get(&key) {
                params_string.push_str(key);
                params_string.push_str(v);
            };
        }

        params_string.push_str(&self.api_secret);

        let digest = md5::compute(params_string);

        // converts it to a hexadecimal string
        format!("{digest:x}",)
    }

    pub fn set_session_key(&mut self, session_key: String) {
        self.session_key = Some(session_key);
    }

    #[must_use]
    pub fn session_key(&self) -> Option<String> {
        self.session_key.clone()
    }

    pub fn enable(&mut self, value: bool) {
        self.enabled = value;
    }

    #[must_use]
    pub fn auth(&'_ self) -> auth::Auth<'_> {
        auth::Auth::new(self)
    }

    #[must_use]
    pub fn user(&'_ self) -> user::User<'_> {
        user::User::new(self)
    }

    #[must_use]
    pub fn track(&'_ self) -> track::Track<'_> {
        track::Track::new(self)
    }

    #[must_use]
    pub fn album(&'_ self) -> album::Album<'_> {
        album::Album::new(self)
    }
}

pub trait LastFMData {
    /// Get `api_key` from [`LastFM`]
    fn api_key(&self) -> String;
    /// Get `api_secret` from [`LastFM`]
    fn api_secret(&self) -> String;
}

impl LastFMData for LastFM {
    fn api_key(&self) -> String {
        String::from(&self.api_key)
    }

    fn api_secret(&self) -> String {
        String::from(&self.api_secret)
    }
}

pub struct LastFMBuilder {
    api_key: Option<String>,
    api_secret: Option<String>,
}

impl LastFMBuilder {
    /// Add `api_key` to builder
    #[must_use]
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_owned());
        self
    }

    /// Add `api_secret` to builder
    #[must_use]
    pub fn api_secret(mut self, api_secret: &str) -> Self {
        self.api_secret = Some(api_secret.to_owned());
        self
    }

    /// Consume `LastFMBuilder` and returns a wrapper to send API calls with.
    ///
    /// # Errors
    /// Returns [`Error::MissingAPIKey`] or [`Error::MissingAPISecret`]
    /// if the respective fields were not set on the builder.
    pub fn build(self) -> Result<LastFM> {
        Ok(LastFM {
            api_key: self.api_key.ok_or(Error::MissingAPIKey)?,
            api_secret: self.api_secret.ok_or(Error::MissingAPISecret)?,
            session_key: None,
            base_url: "http://ws.audioscrobbler.com/2.0/".to_owned(),
            client: reqwest::Client::new(),
            enabled: true,
        })
    }
}
