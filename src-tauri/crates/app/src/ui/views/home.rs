use gpui::{
    AppContext, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::ui::{Button, Slider, Switch, Theme, views::modal_layer::GlobalModalLayer};

#[derive(Clone)]
pub struct Home {
    focus_handle: FocusHandle,
}

impl Home {
    pub fn new(focus_handle: FocusHandle) -> Self {
        Self { focus_handle }
    }
}

struct TestModal;

impl Render for TestModal {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        div().child("Hello!").text_color(theme.text.primary.default)
    }
}

impl Render for Home {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .track_focus(&self.focus_handle)
            .gap_4()
            .flex_wrap()
            .child(Switch::new("switch-1"))
            .child(
                Button::new("button-1", "Click me!").on_click(cx.listener(|this, _, _, cx| {
                    let modal = cx.new(|_| TestModal {});
                    let modal_layer = cx.global::<GlobalModalLayer>().0.clone();

                    modal_layer.update(cx, |layer, cx| {
                        layer.show_modal(modal.into(), cx);
                    });
                })),
            )
            .child(Slider::new("slider-1", self.focus_handle.clone(), 40.0))
    }
}
