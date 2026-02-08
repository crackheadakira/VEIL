use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::ui::theme::{StyleFromColorSet, Theme, text_elements::small};

#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    label: Option<SharedString>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let enabled = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let switch_handle = enabled.clone();

        let theme = cx.global::<Theme>();
        let border_color = &theme.border.secondary;
        let thumb_color = &theme.text.primary;

        // TODO: Add animations somehow when toggling
        div()
            .flex()
            .gap_4()
            .items_center()
            .child(
                div()
                    .id(self.id)
                    .relative()
                    .w_16()
                    .h_8()
                    .flex()
                    .items_center()
                    .when_else(
                        *enabled.read(cx),
                        |this| this.justify_end().border_color(border_color.active),
                        |this| this.justify_start().border_color(border_color.default),
                    )
                    .border_from(border_color)
                    .rounded_full()
                    .px_2()
                    .border_1()
                    .on_click(move |_, _, cx| {
                        let current = *switch_handle.read(cx);
                        switch_handle.write(cx, !current);
                    })
                    .child(div().size_4().rounded_full().bg(thumb_color.default)),
            )
            .when_some(self.label, |this, label| this.child(small(label)))
    }
}
