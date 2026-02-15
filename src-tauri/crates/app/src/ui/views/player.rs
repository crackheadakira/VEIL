use common::Tracks;
use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window,
    div, img, rems,
};
use media_controls::PlayerState;

use crate::{
    AppState,
    events::{PlayerEvent, UIUpdateEvent},
    ui::{AlbumCoverCacheProvider, AppStateContext, Slider, StyleFromColorSet, p, small},
};

pub struct PlayerView {
    focus_handle: FocusHandle,
    slider_value: f64,
    is_seeking: bool,
}

impl PlayerView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let progress = {
            let state = cx.app_state();
            let config = state.config.try_read().expect("error reading config lock");

            config.playback.progress
        };

        let view = Self {
            focus_handle: cx.focus_handle(),
            slider_value: progress,
            is_seeking: false,
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
                        view.slider_value = position;
                        view.is_seeking = false;

                        cx.notify();
                    }
                });
            }
        })
        .detach();

        cx.spawn(async move |view, cx| {
            let ui_bus = cx.read_global::<AppState, _>(|state, _| state.0.ui_bus.clone());
            let mut receiver = ui_bus.subscribe();

            while let Ok(event) = receiver.recv().await {
                let _ = view.update(cx, |view, cx| {
                    if let UIUpdateEvent::ProgressUpdate { progress } = event
                        && !view.is_seeking
                    {
                        view.slider_value = progress;

                        cx.notify();
                    }
                });
            }
        })
        .detach();
    }

    fn get_player_info(cx: &Context<Self>) -> Option<Tracks> {
        let state = cx.app_state();

        let track_id = {
            let queue = state.queue.try_lock().ok()?;
            queue.current()?
        };

        // TODO: this whole thing should be improved somehow, code feels messy
        // especially regarding sliders.

        let track = state.db.by_id::<Tracks>(&track_id).ok()?;

        Some(track)
    }
}

impl Render for PlayerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.app_theme();

        let Some(track) = Self::get_player_info(cx) else {
            return div();
        };

        div()
            .image_cache(AlbumCoverCacheProvider::new("cache:player", 3))
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
                    .child(img(track.cover_path).size_20().rounded_md())
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .justify_center()
                            .gap_1()
                            .child(
                                p(track.name)
                                    .id("player:track_name")
                                    .text_from(&theme.text.primary)
                                    .cursor_pointer()
                                    .truncate(),
                            )
                            .child(
                                small(track.artist_name)
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
                                small(format_time(self.slider_value))
                                    .text_color(theme.text.tertiary.default),
                            )
                            .child(
                                Slider::new("player:progress_slider", self.focus_handle.clone())
                                    .value(self.slider_value)
                                    .max(track.duration as f64)
                                    .w_full()
                                    .on_commit({
                                        let entity = cx.entity();
                                        move |progress, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.is_seeking = false;

                                                let state = cx.app_state();
                                                let player_bus = &state.player_bus;

                                                let resume = state
                                                    .player
                                                    .read()
                                                    .expect("error reading player lock")
                                                    .state
                                                    == PlayerState::Playing;

                                                player_bus.emit(PlayerEvent::Seek {
                                                    position: progress,
                                                    resume,
                                                });
                                            });
                                        }
                                    })
                                    .on_input({
                                        let entity = cx.entity();

                                        move |slider_value, cx| {
                                            entity.update(cx, |this, cx| {
                                                this.slider_value = slider_value;
                                                this.is_seeking = true;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                small(format_time(track.duration as f64))
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
