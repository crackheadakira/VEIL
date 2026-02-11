use gpui::{App, FocusHandle, IntoElement, ParentElement, RenderOnce, Styled, Window, div};

use crate::ui::components::{button::Button, slider::Slider, switch::Switch};

#[derive(Clone, IntoElement)]
pub struct Home {
    focus_handle: FocusHandle,
}

impl Home {
    pub fn new(focus_handle: FocusHandle) -> Self {
        Self { focus_handle }
    }
}

impl RenderOnce for Home {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .gap_4()
            .flex_wrap()
            .child(Switch::new("switch-1"))
            .child(Button::new("button-1", "Click me!"))
            .child(Slider::new("slider-1", self.focus_handle, 0.0, 100.0, 1.0))
    }
}
