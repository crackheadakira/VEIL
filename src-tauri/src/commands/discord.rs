use discord_rich_presence::{
    activity::{self, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};

pub struct DiscordState {
    pub rpc: DiscordIpcClient,
    pub payload: PayloadData,
}

impl DiscordState {
    pub fn new(client_id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let rpc = DiscordIpcClient::new(client_id)?;
        Ok(Self {
            rpc,
            payload: PayloadData {
                state: String::from("Browsing"),
                details: String::from("Sodapop Reimagined"),
                small_image: String::from("paused"),
                small_text: String::from("Paused"),
                show_timestamps: false,
                progress: 0.0,
                duration: 0.0,
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

        let mut activity = activity::Activity::new()
            .state(&data.state)
            .details(&data.details)
            .activity_type(activity::ActivityType::Listening)
            .assets(
                Assets::new()
                    .large_image("sodapop")
                    .large_text("Sodapop Reimagined")
                    .small_image(&data.small_image)
                    .small_text(&data.small_text),
            );

        if data.show_timestamps && data.duration != 0.0 {
            let (timestamp, start) = make_timestamp();
            activity = activity.timestamps(
                timestamp
                    .start(start - data.progress as i64 * 1000)
                    .end(start + (data.duration as f64 - data.progress) as i64 * 1000),
            )
        };

        self.rpc.set_activity(activity)?;

        self.payload = data;
        Ok(())
    }
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

fn make_timestamp() -> (Timestamps, i64) {
    let start_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64;
    (Timestamps::new(), start_time)
}
