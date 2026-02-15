use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::ui::{Button, Modal, Slider, Switch, Theme};

#[derive(Clone)]
pub struct Home {
    focus_handle: FocusHandle,
    show_modal: bool,
}

impl Home {
    pub fn new(focus_handle: FocusHandle) -> Self {
        Self {
            focus_handle,
            show_modal: false,
        }
    }
}

impl Render for Home {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let weak = cx.weak_entity();

        let mut home = div()
            .flex()
            .track_focus(&self.focus_handle)
            .gap_4()
            .flex_wrap()
            .child(Switch::new("switch-1"))
            .child(
                Button::new("button-1", "Click me!").on_click(cx.listener(|this, _, _, _| {
                    this.show_modal = true;
                })),
            )
            .child(Slider::new(
                "slider-1",
                self.focus_handle.clone(),
                0.0,
                100.0,
                1.0,
                40.0,
            ));

        if self.show_modal {
            home = home.child(
                Modal::new()
                    .text_color(theme.text.primary.default)
                    .child("Hello")
                    .on_close(move |_, cx| {
                        weak.update(cx, |this, _| {
                            this.show_modal = false;
                        })
                        .expect("failed to close modal on home");
                    }),
            );
        };

        home
    }
}
