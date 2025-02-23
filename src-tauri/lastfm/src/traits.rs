use crate::{api, LastFM};

pub trait Auth {
    fn auth(&self) -> api::auth::Auth;
}

pub trait User {
    fn user(&self) -> api::user::User;
}

pub trait LastFMData {
    fn api_key(&self) -> String;
    fn api_secret(&self) -> String;
}

pub trait LastFMAuthentication {
    fn add_session_key(&mut self, session_key: String) -> ();
    fn session_key(&self) -> Option<String>;
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

impl LastFMAuthentication for LastFM {
    fn add_session_key(&mut self, session_key: String) -> () {
        self.session_key = Some(session_key);
    }

    fn session_key(&self) -> Option<String> {
        self.session_key.clone()
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
