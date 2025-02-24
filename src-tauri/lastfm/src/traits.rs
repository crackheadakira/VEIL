use crate::{api, LastFM};

pub trait Auth {
    fn auth(&self) -> api::auth::Auth;
}

pub trait User {
    fn user(&self) -> api::user::User;
}

pub trait LastFMData {
    /// Get `api_key` from [`LastFM`]
    fn api_key(&self) -> String;
    /// Get `api_secret` from [`LastFM`]
    fn api_secret(&self) -> String;
}

impl Auth for LastFM {
    fn auth(&self) -> api::auth::Auth {
        api::auth::Auth::new(self)
    }
}

impl User for LastFM {
    fn user(&self) -> api::user::User {
        api::user::User::new(self)
    }
}

impl LastFMData for LastFM {
    fn api_key(&self) -> String {
        String::from(&self.api_key)
    }

    fn api_secret(&self) -> String {
        String::from(&self.api_secret)
    }
}
