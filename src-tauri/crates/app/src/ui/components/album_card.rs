use std::sync::Arc;

use common::Albums;
use gpui::{
    App, ImageFormat, ImageSource, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    Styled, StyledImage, Window, div, img, rems,
};

use crate::ui::theme::{
    Theme,
    text_elements::{h6, p},
};

#[derive(Clone, IntoElement)]
pub struct AlbumCard {
    pub album: Albums,
}

impl RenderOnce for AlbumCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let group_name = format!("big-card-{}", self.album.id);

        // TODO: add an image cache
        let file_bytes = std::fs::read(&self.album.cover_path).unwrap();
        let image = gpui::Image::from_bytes(ImageFormat::Jpeg, file_bytes);

        div()
            .group(&group_name)
            .cursor_pointer()
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
                    .group_hover(&group_name, |this| this.opacity(0.9)),
            )
            // https://github.com/zed-industries/zed/issues/43214
            // group hover for some reason still doesn't apply
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        h6(self.album.name)
                            .text_color(theme.text.primary.default)
                            .truncate()
                            .group_hover(&group_name, |this| {
                                this.text_color(theme.text.primary.hovered)
                            }),
                    )
                    .child(
                        p(self.album.artist_name)
                            .text_color(theme.text.secondary.default)
                            .truncate()
                            .group_hover(&group_name, |this| {
                                this.text_color(theme.text.secondary.hovered)
                            }),
                    )
                    .child(
                        p(self.album.album_type.to_string())
                            .text_color(theme.text.tertiary.default)
                            .group_hover(&group_name, |this| {
                                this.text_color(theme.text.tertiary.hovered)
                            }),
                    ),
            )
    }
}
