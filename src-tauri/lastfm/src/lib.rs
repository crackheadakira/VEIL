//! Easy-to-use Last.FM API wrapper

mod api;
mod models;
mod traits;
pub use crate::traits::*;

use models::{APIError, APIMethod};
use reqwest::Method;
use serde::Deserialize;
use std::collections::HashMap;

/// A helper type for all parameters to have same type without any accidental change
type LastFMParams = HashMap<String, String>;

#[derive(Debug, thiserror::Error)]
pub enum LastFMError {
    /// [`LastFMBuilder`] is missing an API key
    #[error("missing API key")]
    MissingAPIKey,
    /// [`LastFMBuilder`] is missing an API secret
    #[error("missing API secret")]
    MissingAPISecret,
    /// [`LastFM`] is missing the user's session key
    #[error("missing session key")]
    MissingAuthentication,
    /// Passed an invalid HTTP method to [`LastFM::send_request`]
    #[error("invalid HTTP method")]
    InvalidHTTPMethod,
    #[error(transparent)]
    APIError(#[from] APIError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

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
}

impl LastFM {
    pub fn builder() -> LastFMBuilder {
        LastFMBuilder {
            api_key: None,
            api_secret: None,
            session_key: None,
        }
    }

    /// Sends the HTTP Request to Last.FM
    async fn http_request(
        &self,
        method: Method,
        params: &mut LastFMParams,
    ) -> Result<serde_json::Value, LastFMError> {
        let url = &self.base_url;

        let response = match method {
            Method::GET => self.client.get(url).query(&params).send().await?,
            Method::POST => self.client.post(url).query(&params).send().await?,
            _ => return Err(LastFMError::InvalidHTTPMethod),
        };

        let json = response.json().await?;
        Ok(json)
    }

    /// Wraps around [`LastFM::http_request`] and inserts all the required
    /// parameters into `params`
    async fn send_request<T: for<'a> Deserialize<'a>>(
        &self,
        method: Method,
        api_method: APIMethod,
        params: &mut LastFMParams,
    ) -> Result<T, LastFMError> {
        params.insert(String::from("method"), api_method.as_query());
        params.insert(String::from("api_key"), self.api_key.clone());

        if api_method.need_sig() {
            let signature = self.sign_api(params);
            params.insert(String::from("api_sig"), signature);
        }

        params.insert(String::from("format"), String::from("json"));

        let res = self.http_request(method, params).await?;

        if res.get("error").is_some() {
            let error: APIError = serde_json::from_value(res)?;
            return Err(LastFMError::APIError(error));
        }

        let response: T = serde_json::from_value(res)?;
        Ok(response)
    }

    /// Create the MD5 hash of all `params` (except format).
    ///
    /// This is needed by Last.FM to sign your API requests
    fn sign_api(&self, params: &mut LastFMParams) -> String {
        let mut sorted_keys = params.keys().cloned().collect::<Vec<String>>();
        sorted_keys.sort();

        let mut params_string = String::new();

        for key in sorted_keys {
            if let Some(v) = params.get(&key) {
                params_string.push_str(&key);
                params_string.push_str(v);
            };
        }

        params_string.push_str(&self.api_secret);

        let digest = md5::compute(params_string);

        // converts it to a hexadecimal string
        format!("{:x}", digest)
    }

    pub fn session_key(&self) -> Option<String> {
        self.session_key.clone()
    }
}

pub struct LastFMBuilder {
    api_key: Option<String>,
    api_secret: Option<String>,
    session_key: Option<String>,
}

impl LastFMBuilder {
    /// Add `api_key` to builder
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    /// Add `api_secret` to builder
    pub fn api_secret(mut self, api_secret: &str) -> Self {
        self.api_secret = Some(api_secret.to_string());
        self
    }

    /// Add `session_key` to builder
    pub fn session_key(&mut self, session_key: String) -> () {
        self.session_key = Some(session_key);
    }

    /// Consume `LastFMBuilder` and returns a wrapper to send API calls with.
    pub fn build(self) -> Result<LastFM, LastFMError> {
        let secret = self.api_secret.clone();
        Ok(LastFM {
            api_key: self.api_key.ok_or(LastFMError::MissingAPIKey)?,
            api_secret: self.api_secret.ok_or(LastFMError::MissingAPISecret)?,
            session_key: secret,
            base_url: "http://ws.audioscrobbler.com/2.0/".to_string(),
            client: reqwest::Client::new(),
        })
    }
}
