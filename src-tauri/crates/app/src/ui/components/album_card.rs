use common::Albums;
use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, RenderOnce, Styled, StyledImage, Window,
    div, img, list, rems,
};

use crate::ui::{
    image_cache::AlbumCoverCache,
    theme::{
        Theme,
        text_elements::{p, small},
    },
};

#[derive(Clone, IntoElement)]
pub struct AlbumCard {
    pub album: Albums,
}

impl RenderOnce for AlbumCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let image = {
            let cache = cx.global_mut::<AlbumCoverCache>();
            cache.get_or_load(&self.album.cover_path)
        };

        let theme = cx.global::<Theme>();
        let group_name = format!("big-card-{}", self.album.id);

        div()
            .group(&group_name)
            .cursor_pointer()
            .flex()
            .flex_col()
            .h(rems(17.5))
            .w_48()
            .gap_4()
            .child(
                img(image)
                    .size_full()
                    .object_fit(gpui::ObjectFit::Cover)
                    .rounded_md()
                    .group_hover(&group_name, |this| this.opacity(0.9)),
            )
            .child(
                div()
                    .child(
                        p(self.album.name)
                            .id(format!("{group_name}:album_name"))
                            .truncate()
                            .text_color(theme.text.primary.default)
                            .group_hover(&group_name, |this| {
                                this.text_color(theme.text.primary.hovered)
                            }),
                    )
                    .child(
                        p(self.album.artist_name)
                            .id(format!("{group_name}:artist_name"))
                            .text_color(theme.text.secondary.default)
                            .truncate()
                            .group_hover(&group_name, |this| {
                                this.text_color(theme.text.secondary.hovered)
                            }),
                    )
                    .child(
                        small(self.album.album_type.to_string())
                            .id(format!("{group_name}:album_type"))
                            .text_color(theme.text.tertiary.default)
                            .group_hover(&group_name, |this| {
                                this.text_color(theme.text.tertiary.hovered)
                            }),
                    ),
            )
    }
}
