use crate::{api, LastFM};

pub trait Auth {
    fn auth(&self) -> api::auth::Auth;
}

pub trait User {
    fn user(&self) -> api::user::User;
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
