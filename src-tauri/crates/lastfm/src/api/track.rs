use std::{borrow::Cow, collections::HashMap, sync::LazyLock, time::UNIX_EPOCH};

use reqwest::Method;

use crate::{
    Error, LastFM, LastFMParams, Result,
    models::{APIMethod, TrackData},
};

pub struct Track<'a> {
    last_fm: &'a LastFM,
}

impl<'a> Track<'a> {
    pub fn new(last_fm: &'a LastFM) -> Self {
        Self { last_fm }
    }

    pub fn update_now_playing(&'_ self, track: &'a TrackData) -> UpdateNowPlaying<'_> {
        UpdateNowPlaying::new(self.last_fm, track)
    }

    pub fn scrobble_one(&'_ self, track: &'a TrackData) -> TrackScrobble<'_> {
        TrackScrobble::new(self.last_fm, ScrobbleBatch::One(track))
    }

    pub fn scrobble_many(&'_ self, tracks: &'a [TrackData]) -> TrackScrobble<'_> {
        TrackScrobble::new(self.last_fm, ScrobbleBatch::Many(tracks))
    }
}

pub struct UpdateNowPlaying<'a> {
    last_fm: &'a LastFM,
    track: &'a TrackData,
    method: APIMethod,
}

impl<'a> UpdateNowPlaying<'a> {
    fn new(last_fm: &'a LastFM, track: &'a TrackData) -> Self {
        Self {
            last_fm,
            track,
            method: APIMethod::TrackUpdateNowPlaying,
        }
    }

    fn params(&'a self) -> LastFMParams<'a> {
        let mut params = HashMap::new();

        params.insert("artist", Cow::Borrowed(self.track.artist.as_str()));
        params.insert("track", Cow::Borrowed(self.track.name.as_str()));

        params
    }

    pub async fn send(self) -> Result<()> {
        let mut params = self.params();

        let result = self
            .last_fm
            .send_request::<()>(Method::POST, &self.method, &mut params)
            .await;

        match result {
            Ok(_) => Ok(()),

            // ignore this specific error, as we're passing in `()` as type
            // so it'll always return this
            Err(Error::JsonError(ref e)) if e.to_string() == "invalid type: map, expected unit" => {
                Ok(())
            }

            Err(Error::JsonError(e)) => {
                Err(Error::JsonError(e)) // Propagate the error
            }

            Err(e) => Err(e), // Propagate all other non-JsonError errors
        }
    }
}

enum ScrobbleBatch<'a> {
    One(&'a TrackData),
    Many(&'a [TrackData]),
}

pub struct TrackScrobble<'a> {
    last_fm: &'a LastFM,
    tracks: ScrobbleBatch<'a>,
    method: APIMethod,
}

static ARTIST_KEYS: LazyLock<[&'static str; 50]> = LazyLock::new(|| {
    let mut keys = Vec::with_capacity(50);
    for i in 0..50 {
        let s = format!("artist[{i}]");
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        keys.push(leaked);
    }
    keys.try_into().unwrap()
});

static ALBUM_KEYS: LazyLock<[&'static str; 50]> = LazyLock::new(|| {
    let mut keys = Vec::with_capacity(50);
    for i in 0..50 {
        let s = format!("artist[{i}]");
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        keys.push(leaked);
    }
    keys.try_into().unwrap()
});

static TRACK_KEYS: LazyLock<[&'static str; 50]> = LazyLock::new(|| {
    let mut keys = Vec::with_capacity(50);
    for i in 0..50 {
        let s = format!("track[{i}]");
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        keys.push(leaked);
    }
    keys.try_into().unwrap()
});

static TIMESTAMP_KEYS: LazyLock<[&'static str; 50]> = LazyLock::new(|| {
    let mut keys = Vec::with_capacity(50);
    for i in 0..50 {
        let s = format!("timestamp[{i}]");
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        keys.push(leaked);
    }
    keys.try_into().unwrap()
});

impl<'a> TrackScrobble<'a> {
    fn new(last_fm: &'a LastFM, tracks: ScrobbleBatch<'a>) -> Self {
        Self {
            last_fm,
            tracks,
            method: APIMethod::TrackScrobble,
        }
    }

    fn params(&'_ self) -> Result<LastFMParams<'_>> {
        let mut params = HashMap::new();

        let current_timestamp = UNIX_EPOCH.elapsed().expect("Time went backwards");

        match &self.tracks {
            ScrobbleBatch::One(track) => {
                if let Some(album) = track.album.clone() {
                    params.insert("album", Cow::from(album));
                };
                params.insert("artist", Cow::from(track.artist.as_str()));
                params.insert("track", Cow::from(track.name.as_str()));
                params.insert(
                    "timestamp",
                    track
                        .timestamp
                        .unwrap_or(current_timestamp.as_secs() as i64)
                        .to_string()
                        .into(),
                );
            }
            ScrobbleBatch::Many(tracks) => {
                if tracks.len() > 50 {
                    return Err(Error::BatchScrobble);
                }

                for (index, track) in tracks.iter().enumerate() {
                    if let Some(album) = &track.album {
                        params.insert(ALBUM_KEYS[index], album.into());
                    };

                    params.insert(ARTIST_KEYS[index], track.artist.as_str().into());
                    params.insert(TRACK_KEYS[index], track.name.as_str().into());
                    params.insert(
                        TIMESTAMP_KEYS[index],
                        track
                            .timestamp
                            .unwrap_or(current_timestamp.as_secs() as i64)
                            .to_string()
                            .into(),
                    );
                }
            }
        };

        Ok(params)
    }

    pub async fn send(self) -> Result<(), Error> {
        let mut params = self.params()?;
        let result = self
            .last_fm
            .send_request::<()>(Method::POST, &self.method, &mut params)
            .await;

        match result {
            Ok(_) => Ok(()),

            // ignore this specific error, as we're passing in `()` as type
            // so it'll always return this
            Err(Error::JsonError(ref e)) if e.to_string() == "invalid type: map, expected unit" => {
                Ok(())
            }

            Err(Error::JsonError(e)) => {
                Err(Error::JsonError(e)) // Propagate the error
            }

            Err(e) => Err(e), // Propagate all other non-JsonError errors
        }
    }
}
