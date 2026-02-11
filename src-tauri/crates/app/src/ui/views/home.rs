use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div};

use crate::ui::components::{button::Button, switch::Switch};

#[derive(IntoElement)]
pub struct Home {}

impl RenderOnce for Home {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .gap_4()
            .flex_wrap()
            .child(Switch::new("switch-1"))
            .child(Button::new("button-1", "Click me!"))
    }
}
