use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{self, Assets, Timestamps},
};

pub struct DiscordState {
    payload: PayloadData,
    rpc: DiscordIpcClient,
    enabled: bool,
    payload_changed: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PayloadData {
    pub state: String,
    pub details: String,
    pub small_image: String,
    pub small_text: String,
    pub album_cover: Option<String>,
    pub show_timestamps: bool,
    pub progress: f64,
    pub duration: f64,
}

impl DiscordState {
    pub fn new(client_id: &str) -> Self {
        let rpc = DiscordIpcClient::new(client_id);

        Self {
            enabled: false,
            rpc,
            payload_changed: false,
            payload: PayloadData {
                state: String::from("Browsing"),
                details: String::from("Sodapop Reimagined"),
                small_image: String::from("paused"),
                small_text: String::from("Paused"),
                album_cover: None,
                show_timestamps: false,
                progress: 0.0,
                duration: -1.0,
            },
        }
    }

    pub fn enable(&mut self, enable: bool) {
        self.enabled = enable;
    }

    pub fn connect(&mut self) -> bool {
        let res = logging::try_with_log!("Connect Discord RPC", || self.rpc.connect());
        if res.is_ok() {
            self.enabled = true;
            true
        } else {
            self.enabled = false;
            false
        }
    }

    pub fn close(&mut self) -> bool {
        let res = logging::try_with_log!("Close Discord RPC", || self.rpc.close());
        match res {
            Ok(_) => {
                self.enabled = false;
                true
            }
            Err(_) => false,
        }
    }

    pub fn update_activity_progress(&mut self, progress: f64) -> bool {
        if !self.enabled {
            return false;
        }

        if progress != self.payload.progress {
            self.payload.progress = progress;
            self.payload_changed = true;
        }

        if self.payload_changed {
            let res = self.make_activity_from_payload();
            match res {
                Ok(_) => true,
                Err(e) => {
                    logging::error!("Failed to make Discord activity: {e}");
                    false
                }
            }
        } else {
            true
        }
    }

    pub fn update_activity(
        &mut self,
        small_image: &str,
        small_text: &str,
        show_timestamps: bool,
        progress: Option<f64>,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        if small_image != self.payload.small_image {
            self.payload.small_image = small_image.to_owned();
            self.payload_changed = true;
        }

        if small_text != self.payload.small_text {
            self.payload.small_text = small_text.to_owned();
            self.payload_changed = true;
        }

        if show_timestamps != self.payload.show_timestamps {
            self.payload.show_timestamps = show_timestamps;
            self.payload_changed = true;
        }

        if let Some(p) = progress
            && p != self.payload.progress
        {
            self.payload.progress = p;
            self.payload_changed = true;
        }

        if self.payload_changed {
            let res = self.make_activity_from_payload();
            match res {
                Ok(_) => true,
                Err(e) => {
                    logging::error!("Failed to make Discord activity: {e}");
                    false
                }
            }
        } else {
            true
        }
    }

    fn make_activity_from_payload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let assets = if let Some(cover) = &self.payload.album_cover {
            Assets::new()
                .large_image(cover)
                .large_text("Sodapop Reimagined")
                .small_image(&self.payload.small_image)
                .small_text(&self.payload.small_text)
        } else {
            Assets::new()
                .large_image("sodapop")
                .large_text("Sodapop Reimagined")
                .small_image(&self.payload.small_image)
                .small_text(&self.payload.small_text)
        };

        let mut activity = activity::Activity::new()
            .state(&self.payload.state)
            .details(&self.payload.details)
            .activity_type(activity::ActivityType::Listening)
            .assets(assets);

        if self.payload.show_timestamps && self.payload.duration != -1.0 {
            let (timestamp, start) = Self::make_timestamp();
            activity = activity.timestamps(
                timestamp
                    .start(start - self.payload.progress as i64 * 1000)
                    .end(start + (self.payload.duration - self.payload.progress) as i64 * 1000),
            );
        };

        self.rpc.set_activity(activity)?;
        self.payload_changed = false;
        Ok(())
    }

    pub fn make_activity(
        &mut self,
        new_payload: &PayloadData,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut update_activity = false;

        // Avoid updating the activity
        if *new_payload == self.payload {
            return Ok(update_activity);
        }

        self.payload = new_payload.clone();

        if self.enabled {
            self.make_activity_from_payload()?;
            update_activity = true;
        }

        Ok(update_activity)
    }

    fn make_timestamp() -> (Timestamps, i64) {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as i64;
        (Timestamps::new(), start_time)
    }
}
