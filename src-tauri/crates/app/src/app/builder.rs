use std::{fs, sync::Arc};

use gpui::App;
use logging::lock_or_log;

use crate::{
    app::state::{AppState, attach_media_controls_to_player, initialize_state},
    commands::player::initiate_track_ended_thread,
    events::EventSystemHandler,
    queue::{QueueEvent, QueueOrigin},
    systems::{player::PlayerEvent, utils::data_path},
};

pub fn handle_state_setup(cx: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let veil_state = initialize_state()?;
    let app_state = Arc::new(veil_state);
    cx.set_global(AppState(app_state));

    let state = cx.global::<AppState>().0.clone();

    {
        let config = lock_or_log(state.config.read(), "Config RwLock")?;
        if config.integrations.discord_enabled {
            let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
            discord.connect();
        }
    }

    attach_media_controls_to_player(state.clone())?;

    tokio::spawn({
        let state = state.clone();
        let mut rx = state.config_bus.subscribe();

        async move {
            while let Ok(event) = rx.recv().await {
                let state = state.clone();

                if let Some(discord_enabled) = event.discord_enabled {
                    let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex").unwrap();
                    let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();

                    if discord_enabled {
                        if discord.connect() {
                            let progress = player.get_progress();
                            discord.update_activity_progress(progress);
                        }
                    } else {
                        discord.close();
                    }
                }

                if let Some(l) = event.last_fm_enabled {
                    let mut lastfm = state.lastfm.lock().await;
                    lastfm.enable(l);
                };

                let mut config = lock_or_log(state.config.write(), "Config Write").unwrap();
                config.update_config_and_write(event).unwrap();
            }
        }
    });

    PlayerEvent::attach_listener(state.player_bus.clone(), state.clone());
    QueueEvent::attach_listener(state.queue_bus.clone(), state.clone());

    let path = data_path();
    let covers = path.join("covers");
    if !covers.exists() {
        fs::create_dir_all(&covers).expect("Error creating covers directory");
        let pc = include_bytes!("../../../../../public/placeholder.png");

        fs::write(covers.join("placeholder.png"), pc)?;
    }

    // Populate the queue if a queue origin exists
    let mut queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
    if let Some(queue_origin) = queue.origin() {
        let config = lock_or_log(state.config.read(), "Config Read").unwrap();
        match queue_origin {
            QueueOrigin::Album { id } => {
                let result = state.db.album_with_tracks(&id)?;
                let track_ids: Vec<u32> = result.tracks.iter().map(|track| track.id).collect();

                queue.set_global(track_ids);
            }
            QueueOrigin::Playlist { id } => {
                let result = state.db.get_playlist_with_tracks(&id)?;
                let track_ids: Vec<u32> = result.tracks.iter().map(|track| track.id).collect();

                queue.set_global(track_ids);
            }
        }

        queue.set_current_index(config.playback.queue_idx);
    }

    initiate_track_ended_thread(state.clone());

    Ok(())
}
