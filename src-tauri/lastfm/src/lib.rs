mod api;
mod models;
mod traits;
pub use crate::traits::*;

use models::{APIError, APIMethod};
use reqwest::Method;
use serde::Deserialize;
use std::collections::HashMap;

type LastFMParams = HashMap<String, String>;

#[derive(Debug, thiserror::Error)]
pub enum LastFMError {
    #[error("missing API key")]
    MissingAPIKey,
    #[error("missing API secret")]
    MissingAPISecret,
    #[error("missing session key")]
    MissingAuthentication,
    #[error("invalid HTTP method")]
    InvalidHTTPMethod,
    #[error(transparent)]
    APIError(#[from] APIError),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}

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
    client: reqwest::blocking::Client,
}

impl LastFM {
    pub fn builder() -> LastFMBuilder {
        LastFMBuilder {
            api_key: None,
            api_secret: None,
        }
    }

    fn http_request(
        &self,
        method: Method,
        params: &mut LastFMParams,
    ) -> Result<serde_json::Value, LastFMError> {
        let url = &self.base_url;

        let response = match method {
            Method::GET => self.client.get(url).query(&params).send()?,
            Method::POST => self.client.post(url).query(&params).send()?,
            _ => return Err(LastFMError::InvalidHTTPMethod),
        };

        let json = response.json()?;
        Ok(json)
    }

    fn send_request<T: for<'a> Deserialize<'a>>(
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

        let res = self.http_request(method, params)?;

        if res.get("error").is_some() {
            let error: APIError = serde_json::from_value(res)?;
            return Err(LastFMError::APIError(error));
        }

        let response: T = serde_json::from_value(res)?;
        Ok(response)
    }

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
}

pub struct LastFMBuilder {
    api_key: Option<String>,
    api_secret: Option<String>,
}

impl LastFMBuilder {
    pub fn api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub fn api_secret(mut self, api_secret: &str) -> Self {
        self.api_secret = Some(api_secret.to_string());
        self
    }

    pub fn build(self) -> Result<LastFM, LastFMError> {
        Ok(LastFM {
            api_key: self.api_key.ok_or(LastFMError::MissingAPIKey)?,
            api_secret: self.api_secret.ok_or(LastFMError::MissingAPISecret)?,
            session_key: None,
            base_url: "http://ws.audioscrobbler.com/2.0/".to_string(),
            client: reqwest::blocking::Client::new(),
        })
    }
}
