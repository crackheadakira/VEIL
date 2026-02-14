use std::rc::Rc;

use gpui::{
    App, Div, InteractiveElement, IntoElement, KeyBinding, MouseButton, ParentElement, RenderOnce,
    Stateful, StyleRefinement, Styled, Window, actions, anchored, deferred, div, point,
    prelude::FluentBuilder, px, rgba,
};

#[derive(IntoElement)]
pub struct Modal {
    div: Stateful<Div>,
    on_close: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

actions!(modal, [Close]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", Close, None)]);
}

impl Modal {
    pub fn new() -> Self {
        Self {
            div: div().id("modal-content"),
            on_close: None,
        }
    }

    pub fn on_close(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Rc::new(f));
        self
    }
}

impl ParentElement for Modal {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.div.extend(elements);
    }

    fn child(mut self, child: impl IntoElement) -> Self
    where
        Self: Sized,
    {
        self.div = self.div.child(child);
        self
    }
}

impl Styled for Modal {
    fn style(&mut self) -> &mut StyleRefinement {
        self.div.style()
    }
}

impl RenderOnce for Modal {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let size = window.viewport_size();

        anchored().position(point(px(0.0), px(0.0))).child(deferred(
            div()
                .id("modal-backdrop")
                .w(size.width)
                .h(size.height)
                .bg(rgba(0x00000080))
                .p_8()
                .flex()
                .occlude()
                .justify_center()
                .items_center()
                .when_some(self.on_close, |this, on_close| {
                    let on_close_clone = on_close.clone();

                    this.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                        on_close_clone(window, cx)
                    })
                    .on_action(move |_: &Close, window, cx| {
                        println!("Close action fired!");
                        on_close(window, cx)
                    })
                })
                .child(self.div.occlude()),
        ))
    }
}
