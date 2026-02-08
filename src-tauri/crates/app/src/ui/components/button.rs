use gpui::{
    App, Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Stateful, StyleRefinement, Styled, Window, div,
};

use crate::ui::theme::{StyleFromColorSet, Theme, text_elements::p};

#[derive(IntoElement)]
pub struct Button {
    div: Stateful<Div>,
    label: SharedString,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            div: div().id(id),
            label: label.into(),
        }
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.div.interactivity()
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.div.style()
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        self.div
            .flex()
            .items_center()
            .justify_center()
            .min_w_24()
            .h_12()
            .rounded_md()
            .p_3()
            .border_1()
            .text_color(theme.text.secondary.default)
            .border_from(&theme.border.secondary)
            .bg(theme.background.primary.default)
            .child(p(self.label))
            .cursor_pointer()
    }
}
