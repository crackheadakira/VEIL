use crate::{api, LastFM};

pub trait Auth {
    fn auth(&self) -> api::auth::Auth;
}

impl Auth for LastFM {
    fn auth(&self) -> api::auth::Auth {
        api::auth::Auth::new(self)
    }
}
