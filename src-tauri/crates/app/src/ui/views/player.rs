use common::Tracks;
use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    div, img, rems,
};
use media_controls::PlayerState;

use crate::{
    AppState,
    events::{PlayerEvent, UIUpdateEvent},
    ui::{Slider, StyleFromColorSet, Theme, p, small},
};

pub struct PlayerView {
    focus_handle: FocusHandle,
    cached_progress: Option<f64>,
}

struct PlayerInfo {
    progress: f64,
    track: Tracks,
}

impl PlayerView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let view = Self {
            focus_handle: cx.focus_handle(),
            cached_progress: None,
        };

        Self::subscribe_to_events(cx);

        view
    }

    fn subscribe_to_events(cx: &mut Context<Self>) {
        cx.spawn(async move |view, cx| {
            let player_bus = cx.read_global::<AppState, _>(|state, _| state.0.player_bus.clone());
            let mut receiver = player_bus.subscribe();

            while let Ok(event) = receiver.recv().await {
                let _ = view.update(cx, |view, cx| {
                    if let PlayerEvent::Seek {
                        position,
                        resume: _,
                    } = event
                    {
                        view.cached_progress = Some(position);
                    }

                    cx.notify();
                });
            }
        })
        .detach();

        cx.spawn(async move |view, cx| {
            let ui_bus = cx.read_global::<AppState, _>(|state, _| state.0.ui_bus.clone());
            let mut receiver = ui_bus.subscribe();

            while let Ok(event) = receiver.recv().await {
                let _ = view.update(cx, |view, cx| {
                    if let UIUpdateEvent::ProgressUpdate { progress } = event {
                        view.cached_progress = Some(progress);
                    }

                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn get_player_info(&self, cx: &Context<Self>) -> Option<PlayerInfo> {
        let state = cx.global::<AppState>();

        let track_id = {
            let queue = state.0.queue.try_lock().ok()?;
            queue.current()?
        };

        // TODO: this whole thing should be improved somehow, code feels messy
        // especially regarding sliders.
        let progress = self.cached_progress.unwrap_or_else(|| {
            let config = state
                .0
                .config
                .try_read()
                .expect("error reading config lock");

            config.playback.progress
        });

        let track = state.0.db.by_id::<Tracks>(&track_id).ok()?;

        let info = PlayerInfo { progress, track };

        Some(info)
    }
}

impl Render for PlayerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let Some(info) = self.get_player_info(cx) else {
            return div();
        };

        div()
            .track_focus(&self.focus_handle)
            .h(rems(7.0))
            .w_full()
            .border_t_1()
            .border_color(theme.border.secondary.default)
            .bg(theme.background.secondary.default)
            .grid()
            .grid_cols(4)
            .items_center()
            .p_4()
            .text_color(theme.text.primary.default)
            .child(
                div()
                    .col_span(1)
                    .flex()
                    .gap_5()
                    .child(img(info.track.cover_path).size_20().rounded_md())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .justify_center()
                            .gap_1()
                            .child(
                                p(info.track.name)
                                    .id("player:track_name")
                                    .text_from(&theme.text.primary)
                                    .cursor_pointer()
                                    .truncate(),
                            )
                            .child(
                                small(info.track.artist_name)
                                    .id("player:artist_name")
                                    .text_from(&theme.text.secondary)
                                    .cursor_pointer()
                                    .truncate(),
                            ),
                    ),
            )
            .child(
                div()
                    .col_span(2)
                    .flex()
                    .flex_col()
                    .gap_2()
                    .items_center()
                    .w_full()
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .gap_4()
                            .px_6()
                            .child(
                                small(format_time(info.progress))
                                    .text_color(theme.text.tertiary.default),
                            )
                            .child(
                                Slider::new(
                                    "player:progress_slider",
                                    self.focus_handle.clone(),
                                    info.progress as f32,
                                )
                                .max(info.track.duration as f32)
                                .w_full()
                                .on_commit(move |progress, cx| {
                                    let state = cx.global::<AppState>();
                                    let player_state = state
                                        .0
                                        .player
                                        .read()
                                        .expect("error setting RwLock for player")
                                        .state;
                                    let player_bus = &state.0.player_bus;

                                    let resume = player_state == PlayerState::Playing;

                                    player_bus.emit(PlayerEvent::Seek {
                                        position: progress as f64,
                                        resume,
                                    });
                                }),
                            )
                            .child(
                                small(format_time(info.track.duration as f64))
                                    .text_color(theme.text.tertiary.default),
                            ),
                    ),
            )
            .child(div().col_span(1).child("volume"))
    }
}

fn format_time(seconds: f64) -> String {
    let mins = (seconds / 60.0) as u32;
    let secs = (seconds % 60.0) as u32;
    format!("{:02}:{:02}", mins, secs)
}
