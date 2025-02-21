use discord_rich_presence::{
    activity::{self, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

pub struct DiscordState {
    pub rpc: DiscordIpcClient,
    pub payload: PayloadData,
    pub enabled: bool,
}

#[derive(Clone, PartialEq)]
pub struct PayloadData {
    pub state: String,
    pub details: String,
    pub small_image: String,
    pub small_text: String,
    pub show_timestamps: bool,
    pub progress: f64,
    pub duration: f32,
}

impl DiscordState {
    pub fn new(client_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rpc = DiscordIpcClient::new(client_id)?;
        Ok(Self {
            enabled: false,
            rpc,
            payload: PayloadData {
                state: String::from("Browsing"),
                details: String::from("Sodapop Reimagined"),
                small_image: String::from("paused"),
                small_text: String::from("Paused"),
                show_timestamps: false,
                progress: 0.0,
                duration: -1.0,
            },
        })
    }

    pub fn make_activity(
        &mut self,
        new_payload: PayloadData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let data = new_payload;

        if data == self.payload {
            return Ok(()); // No need to update the activity
        }

        self.payload = data;

        if !self.enabled {
            return Ok(());
        }

        let mut activity = activity::Activity::new()
            .state(&self.payload.state)
            .details(&self.payload.details)
            .activity_type(activity::ActivityType::Listening)
            .assets(
                Assets::new()
                    .large_image("sodapop")
                    .large_text("Sodapop Reimagined")
                    .small_image(&self.payload.small_image)
                    .small_text(&self.payload.small_text),
            );

        if self.payload.show_timestamps && self.payload.duration != -1.0 {
            let (timestamp, start) = make_timestamp();
            activity = activity.timestamps(
                timestamp
                    .start(start - self.payload.progress as i64 * 1000)
                    .end(
                        start
                            + (self.payload.duration as f64 - self.payload.progress) as i64 * 1000,
                    ),
            )
        };

        self.rpc.set_activity(activity)?;
        Ok(())
    }
}

fn make_timestamp() -> (Timestamps, i64) {
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64;
    (Timestamps::new(), start_time)
}
