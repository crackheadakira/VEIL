use common::Albums;
use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window, div,
};

use crate::app::state::AppState;
use crate::ui::components::album_card::AlbumCard;
use crate::ui::theme::Theme;
use crate::ui::theme::text_elements::h6;

#[derive(Clone, IntoElement)]
pub struct AllAlbumsView {
    albums: Vec<Albums>,
}

impl AllAlbumsView {
    pub fn new(cx: &mut App) -> Self {
        let state = &cx.global::<AppState>().0;
        let albums = state
            .db
            .album_pagination(20, 0)
            .expect("failed to fetch all albums");

        Self { albums }
    }
}

impl RenderOnce for AllAlbumsView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        div()
            .bg(theme.background.primary.default)
            .flex()
            .flex_col()
            .items_center()
            .size_full()
            .gap_4()
            .child(
                h6(format!("{} albums", self.albums.len()))
                    .w_full()
                    .text_color(theme.text.primary.default),
            )
            .child(
                div()
                    .id("all_albums_view")
                    .overflow_y_scroll()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .justify_center()
                    .gap_4()
                    .children(self.albums.into_iter().map(|album| AlbumCard { album })),
            )
    }
}
