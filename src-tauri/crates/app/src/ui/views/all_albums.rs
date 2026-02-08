use std::sync::Arc;

use common::Albums;
use gpui::{
    App, ImageFormat, ImageSource, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, StyledImage, Window, div, img, rems,
};

use crate::app::state::AppState;
use crate::ui::theme::Theme;
use crate::ui::theme::text_elements::{h6, p};

#[derive(Clone, IntoElement)]
pub struct AllAlbumsView {
    albums: Vec<Albums>,
}

impl AllAlbumsView {
    pub fn new(cx: &mut App) -> Self {
        let state = &cx.global::<AppState>().0;
        let albums = state
            .db
            .album_pagination(50, 0)
            .expect("failed to fetch all albums");

        Self { albums }
    }
}

impl RenderOnce for AllAlbumsView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let cards: Vec<_> = self
            .albums
            .into_iter()
            .enumerate()
            .map(|(idx, album)| {
                let group_name = format!("big-card-{idx}");

                // TODO: add an image cache
                let file_bytes = std::fs::read(&album.cover_path).unwrap();
                let image = gpui::Image::from_bytes(ImageFormat::Jpeg, file_bytes);

                div()
                    .group(&group_name)
                    .flex()
                    .flex_col()
                    .h(rems(17.5))
                    .w_48()
                    .gap_4()
                    .child(
                        img(ImageSource::Image(Arc::new(image)))
                            .size_full()
                            .object_fit(gpui::ObjectFit::Cover)
                            .rounded_md()
                            .group_hover(&group_name, |this| this.opacity(0.7)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                h6(album.name)
                                    .text_color(theme.text.primary.default)
                                    .truncate()
                                    .group_hover(&group_name, |this| {
                                        this.text_color(theme.text.primary.hovered)
                                    }),
                            )
                            .child(
                                p(album.artist_name)
                                    .text_color(theme.text.secondary.default)
                                    .truncate()
                                    .group_hover(&group_name, |this| {
                                        this.text_color(theme.text.secondary.hovered)
                                    }),
                            )
                            .child(
                                p(album.album_type.to_string())
                                    .text_color(theme.text.tertiary.default)
                                    .group_hover(&group_name, |this| {
                                        this.text_color(theme.text.tertiary.hovered)
                                    }),
                            ),
                    )
            })
            .collect();

        div()
            .bg(theme.background.primary.default)
            .text_color(theme.text.primary.default)
            .flex()
            .flex_col()
            .items_center()
            .size_full()
            .gap_4()
            .child(h6(format!("{} albums", cards.len())).w_full())
            .child(
                div()
                    .id("all_albums_view")
                    .overflow_scroll()
                    .h_full()
                    .grid()
                    .grid_cols(5)
                    .gap_4()
                    .children(cards),
            )
    }
}
