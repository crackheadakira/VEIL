use std::{fs, sync::Arc};

use gpui::App;
use logging::lock_or_log;

use crate::{
    VeilState,
    app::state::{AppState, attach_media_controls_to_player, initialize_state},
    events::EventSystemHandler,
    queue::{QueueEvent, QueueOrigin},
    systems::{
        player::{PlayerEvent, next_track_status, try_preloading_next_sound_handle},
        ui::UIUpdateEvent,
        utils::data_path,
    },
};

pub fn handle_state_setup(cx: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let veil_state = initialize_state(cx)?;
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
        let pc = include_bytes!("../../../../../assets/placeholder.png");

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

        let saved_progress = config.playback.progress;
        queue.set_current_index(config.playback.queue_idx);

        if let Some(track_id) = queue.current() {
            drop(queue);
            drop(config);

            if let Ok(track) = state.db.by_id::<common::Tracks>(&track_id) {
                state.player_bus.emit(PlayerEvent::Initialize {
                    track,
                    progress: saved_progress,
                });
            }
        }
    }

    initiate_track_ended_thread(state.clone());
    initiate_player_progress_thread(state.clone());

    Ok(())
}

fn initiate_track_ended_thread(state: Arc<VeilState>) {
    tokio::spawn(async move {
        let check_interval = std::time::Duration::from_millis(25);

        loop {
            tokio::time::sleep(check_interval).await;

            let ended = {
                let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();
                player.has_ended()
            };

            if !ended {
                continue;
            }

            {
                let mut player = lock_or_log(state.player.write(), "Player Write Lock").unwrap();
                try_preloading_next_sound_handle(&state, &mut player);

                if let Some(track) = next_track_status(&state, &player) {
                    state.player_bus.emit(PlayerEvent::NewTrack { track });
                }
            }

            let queue_has_ended = {
                let queue = lock_or_log(state.queue.lock(), "Queue Mutex").unwrap();
                queue.reached_end
            };

            if queue_has_ended {
                state.player_bus.emit(PlayerEvent::Stop);
                state.resume_notify.notified().await;
            }
        }
    });
}

fn initiate_player_progress_thread(state: Arc<VeilState>) {
    tokio::spawn(async move {
        let check_interval = std::time::Duration::from_millis(400);

        loop {
            tokio::time::sleep(check_interval).await;

            let progress = {
                let player = lock_or_log(state.player.read(), "Player Read Lock").unwrap();

                if matches!(player.state, media_controls::PlayerState::Playing) {
                    Some(player.get_progress())
                } else {
                    None
                }
            };

            if let Some(progress) = progress {
                state
                    .ui_bus
                    .emit(UIUpdateEvent::ProgressUpdate { progress })
            }
        }
    });
}
