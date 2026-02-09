use gpui::{
    App, InteractiveElement, IntoElement, RenderOnce, StatefulInteractiveElement, Styled, Window,
    div,
};

use crate::ui::theme::Theme;

#[derive(IntoElement)]
pub struct Sidebar {}

impl Sidebar {
    pub fn new() -> Self {
        Self {}
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div()
            .id("sidebar")
            .overflow_y_scroll()
            .flex_shrink_0()
            .h_full()
            .w_72()
            .bg(theme.background.primary.default)
            .border_r_1()
            .border_color(theme.border.secondary.default)
            .flex()
            .flex_col()
    }
}
