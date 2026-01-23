use std::fs;

use logging::lock_or_log;
use tauri::{Manager, Wry};
use tauri_specta::{Builder, Event, collect_commands, collect_events};

use crate::{
    app::{
        SodapopState,
        state::{attach_media_controls_to_player, initialize_state},
    },
    commands,
    config::{SodapopConfig, SodapopConfigEvent},
    error::FrontendError,
    events::EventSystemHandler,
    queue::{QueueEvent, QueueOrigin},
    systems::{player::PlayerEvent, ui::UIUpdateEvent, utils::data_path},
};

pub fn make_specta_type_builder() -> Builder {
    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::music_folder::select_music_folder,
            commands::db::get_album_with_tracks,
            commands::db::get_artist_with_albums,
            commands::db::get_all_albums,
            commands::db::track_by_id,
            commands::db::new_playlist,
            commands::db::get_all_playlists,
            commands::db::add_to_playlist,
            commands::db::get_playlist_tracks,
            commands::db::remove_from_playlist,
            commands::db::search_db,
            commands::db::get_albums_offset,
            commands::db::get_total_albums,
            commands::db::get_batch_track,
            commands::player::get_player_state,
            commands::player::player_has_track,
            commands::player::get_player_progress,
            commands::player::get_player_duration,
            commands::player::player_has_ended,
            commands::player::player_progress_channel,
            commands::lastfm::get_token,
            commands::lastfm::get_session,
            commands::read_custom_style,
            commands::read_config,
            commands::plugins::open_url,
            commands::db::get_playlist_tracks_offset,
            commands::db::get_total_tracks_in_playlist,
            commands::db::get_playlist_details,
            commands::db::update_playlist,
        ])
        .events(collect_events![
            SodapopConfigEvent,
            PlayerEvent,
            FrontendError,
            QueueEvent,
            UIUpdateEvent,
        ])
        .typ::<SodapopConfig>();

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    {
        use specta_typescript::{BigIntExportBehavior, Typescript};
        use std::path::PathBuf;

        let bindings_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../..") // go up from crates/app
            .join("src")
            .join("bindings.ts");

        specta_builder
            .export(
                Typescript::default()
                    .formatter(specta_typescript::formatter::prettier)
                    .bigint(BigIntExportBehavior::Number)
                    .header("// @ts-nocheck"),
                bindings_path,
            )
            .expect("Failed to export TypeScript bindings");
    };

    specta_builder
}

pub fn mount_tauri_builder(specta_builder: &Builder) -> tauri::Builder<Wry> {
    tauri::Builder::default().invoke_handler(specta_builder.invoke_handler())
}

pub fn handle_tauri_setup(
    app: &mut tauri::App,
    specta_builder: Builder,
) -> Result<(), Box<dyn std::error::Error>> {
    specta_builder.mount_events(app);

    let sodapop_state = initialize_state()?;
    app.manage(sodapop_state);

    let state = app.state::<SodapopState>();

    {
        let config = lock_or_log(state.config.read(), "Config RwLock")?;
        if config.integrations.discord_enabled {
            let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex")?;
            discord.connect();
        }
    }

    let app_handle = app.handle();
    attach_media_controls_to_player(app_handle, &state)?;

    let app_handle = app.handle().clone();
    SodapopConfigEvent::listen(app, move |event| {
        let state = app_handle.state::<SodapopState>();
        if let Some(discord_enabled) = event.payload.discord_enabled {
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

        let app_handle = app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(l) = event.payload.last_fm_enabled {
                let state = app_handle.state::<SodapopState>();
                let mut lastfm = state.lastfm.lock().await;
                lastfm.enable(l);
            };
        });

        let mut config = lock_or_log(state.config.write(), "Config Write").unwrap();
        config.update_config_and_write(event.payload).unwrap();
    });

    let app_handle = app.handle();
    PlayerEvent::attach_listener(app_handle);
    QueueEvent::attach_listener(app_handle);

    let path = data_path();
    let covers = path.join("covers");
    if !covers.exists() {
        fs::create_dir(&covers).expect("Error creating covers directory");
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

    Ok(())
}
