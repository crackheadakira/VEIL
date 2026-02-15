use std::time::Duration;

use gpui::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, ease_out_quint, prelude::FluentBuilder, rems,
};

use crate::ui::{StyleFromColorSet, Theme, small};

use gpui_transitions::WindowUseTransition;

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
        let enabled_transition = window
            .use_keyed_transition(
                format!("{}:transition", self.id),
                cx,
                Duration::from_millis(75),
                |_, _| 0.0_f32,
            )
            .with_easing(ease_out_quint());
        let switch_handle = enabled.clone();

        let enabled_delta = *enabled_transition.evaluate(window, cx);

        let theme = cx.global::<Theme>();
        let border_color = &theme.border.secondary;
        let thumb_color = &theme.text.primary;

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
                    .border_from(border_color)
                    .rounded_full()
                    .px_2()
                    .border_1()
                    .on_click(move |_, _, cx| {
                        let current = *switch_handle.read(cx);
                        switch_handle.write(cx, !current);

                        let new_goal = if current { 0.0 } else { 1.0 };
                        enabled_transition.update(cx, |this, cx| {
                            *this = new_goal;
                            cx.notify();
                        });
                    })
                    .child(
                        div()
                            .size_4()
                            .rounded_full()
                            .bg(thumb_color.default)
                            .absolute()
                            .left(rems(0.5 + enabled_delta * 2.0)),
                    ),
            )
            .when_some(self.label, |this, label| this.child(small(label)))
    }
}
