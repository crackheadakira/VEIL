use common::Albums;
use gpui::{App, Context, IntoElement, ParentElement, Render, Styled, Window, div, rems};

use crate::app::state::AppState;
use crate::ui::components::album_card::AlbumCard;
use crate::ui::components::uniform_grid::{UniformGridScrollHandle, uniform_grid};
use crate::ui::theme::Theme;
use crate::ui::theme::text_elements::h6;

#[derive(Clone)]
pub struct AllAlbumsView {
    albums: Vec<Albums>,
    scroll_handle: UniformGridScrollHandle,
}

impl AllAlbumsView {
    pub fn new(cx: &mut App) -> Self {
        let state = &cx.global::<AppState>().0;
        let albums = state
            .db
            .all::<Albums>()
            .expect("failed to fetch all albums");

        Self {
            albums,
            scroll_handle: UniformGridScrollHandle::new(),
        }
    }
}

impl Render for AllAlbumsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div()
            .size_full()
            .bg(theme.background.primary.default)
            .flex()
            .flex_col()
            .items_center()
            .gap_4()
            .child(
                h6(format!("{} albums", self.albums.len()))
                    .w_full()
                    .text_color(theme.text.primary.default),
            )
            .child(
                uniform_grid(
                    "all_albums_list",
                    self.albums.len(),
                    cx.processor(|this, range, _window, _cx| {
                        let mut items = Vec::new();
                        for idx in range {
                            let item: &Albums = &this.albums[idx];

                            items.push(AlbumCard {
                                album: item.clone(),
                            })
                        }

                        items
                    }),
                )
                .size_full()
                .gap(rems(1.0))
                .track_scroll(&self.scroll_handle)
                .preload_rows(1),
            )
    }
}
