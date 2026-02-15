use std::{cell::Cell, rc::Rc};

use gpui::{
    App, Bounds, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    MouseButton, ParentElement, Pixels, Refineable, RenderOnce, StyleRefinement, Styled, Window,
    actions, canvas, div, relative, rems,
};

use crate::ui::Theme;

actions!(slider, [Decrease, Increase, JumpStart, JumpEnd]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("left", Decrease, None),
        KeyBinding::new("right", Increase, None),
        KeyBinding::new("home", JumpStart, None),
        KeyBinding::new("end", JumpEnd, None),
    ]);
}

type ChangeCallback = dyn Fn(f32, &mut App);

#[derive(Clone, IntoElement)]
pub struct Slider {
    id: ElementId,
    focus_handle: FocusHandle,

    min: f32,
    max: f32,
    step: f32,
    initial_value: f32,

    style: StyleRefinement,
    on_change: Option<Rc<ChangeCallback>>,
}

impl Slider {
    pub fn new(
        id: impl Into<ElementId> + Clone,
        focus_handle: FocusHandle,
        min: f32,
        max: f32,
        step: f32,
        initial_value: f32,
    ) -> Self {
        Self {
            id: id.clone().into(),
            focus_handle,
            min,
            max,
            step,
            initial_value,
            style: StyleRefinement::default(),
            on_change: None,
        }
    }

    pub fn on_change(mut self, callback: impl Fn(f32, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(callback));
        self
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let keyed_value = window.use_keyed_state(self.id.clone(), cx, |_, _| self.initial_value);
        let is_dragging = window.use_keyed_state(format!("{}-dragging", self.id), cx, |_, _| false);

        let theme = cx.global::<Theme>();
        let group_name = format!("{}:group", &self.id);

        let track_bounds = Rc::new(Cell::new(Bounds::default()));
        let normalized_value = (keyed_value.read(cx) - self.min) / (self.max - self.min);

        let calculate_and_set = move |value: f32, keyed_value: &Entity<f32>, cx: &mut App| {
            let clamped = value.clamp(self.min, self.max);
            let steps = ((clamped - self.min) / self.step).round();
            let final_value = self.min + steps * self.step;
            keyed_value.write(cx, final_value);

            final_value
        };

        let value_from_mouse = move |x: Pixels, bounds: Bounds<Pixels>| {
            let mouse_x = f32::from(x) - f32::from(bounds.origin.x);
            let track_width = f32::from(bounds.size.width);
            let normalized = (mouse_x / track_width).clamp(0.0, 1.0);
            self.min + normalized * (self.max - self.min)
        };

        // man wtf have i written.

        let mut root = div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .on_action({
                let keyed_value = keyed_value.clone();
                let on_change = self.on_change.clone();
                move |_: &Decrease, _window, cx| {
                    let current = keyed_value.read(cx);
                    let final_value = calculate_and_set(current - self.step, &keyed_value, cx);

                    if let Some(ref callback) = on_change {
                        callback(final_value, cx);
                    }
                }
            })
            .on_action({
                let keyed_value = keyed_value.clone();
                let on_change = self.on_change.clone();
                move |_: &Increase, _window, cx| {
                    let current = keyed_value.read(cx);
                    let final_value = calculate_and_set(current - self.step, &keyed_value, cx);

                    if let Some(ref callback) = on_change {
                        callback(final_value, cx);
                    }
                }
            })
            .on_action({
                let keyed_value = keyed_value.clone();
                let on_change = self.on_change.clone();
                move |_: &JumpStart, _window, cx| {
                    keyed_value.write(cx, self.min);

                    if let Some(ref callback) = on_change {
                        callback(self.min, cx);
                    }
                }
            })
            .on_action({
                let keyed_value = keyed_value.clone();
                let on_change = self.on_change.clone();
                move |_: &JumpEnd, _window, cx| {
                    keyed_value.write(cx, self.max);

                    if let Some(ref callback) = on_change {
                        callback(self.max, cx);
                    }
                }
            })
            .group(&group_name)
            .min_w(rems(9.0))
            .h_2()
            .relative()
            .child(canvas(
                {
                    let track_bounds = track_bounds.clone();
                    move |bounds, _window, _cx| {
                        track_bounds.set(bounds);
                    }
                },
                |_bounds, _state, _window, _cx| {},
            ))
            .on_mouse_down(MouseButton::Left, {
                let keyed_value = keyed_value.clone();
                let track_bounds = track_bounds.clone();
                let is_dragging = is_dragging.clone();
                move |event, _window, cx| {
                    is_dragging.write(cx, true);
                    let new_value = value_from_mouse(event.position.x, track_bounds.get());
                    calculate_and_set(new_value, &keyed_value, cx);
                }
            })
            .on_mouse_move({
                let keyed_value = keyed_value.clone();
                move |event, _window, cx| {
                    if event.dragging() {
                        let new_value = value_from_mouse(event.position.x, track_bounds.get());
                        calculate_and_set(new_value, &keyed_value, cx);
                    }
                }
            })
            .on_mouse_up(MouseButton::Left, {
                let keyed_value = keyed_value.clone();
                let on_change = self.on_change.clone();
                let is_dragging = is_dragging.clone();
                move |_event, _window, cx| {
                    is_dragging.write(cx, false);
                    if let Some(ref callback) = on_change {
                        let final_value = keyed_value.read(cx);
                        callback(*final_value, cx);
                    }
                }
            })
            .on_mouse_up_out(MouseButton::Left, {
                let keyed_value = keyed_value.clone();
                let on_change = self.on_change.clone();
                let is_dragging = is_dragging.clone();
                move |_event, _window, cx| {
                    if *is_dragging.read(cx) {
                        is_dragging.write(cx, false);
                        if let Some(ref callback) = on_change {
                            let final_value = keyed_value.read(cx);
                            callback(*final_value, cx);
                        }
                    }
                }
            })
            .child(
                div()
                    .size_full()
                    .bg(theme.border.secondary.default)
                    .rounded_full()
                    .absolute(),
            )
            .child(
                div()
                    .w(relative(normalized_value))
                    .h_full()
                    .bg(theme.text.secondary.default)
                    .rounded_full()
                    .absolute()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .id(format!("{}:thumb", &self.id))
                            .absolute()
                            .right(-rems(0.5))
                            .size_4()
                            .rounded_full()
                            .bg(theme.text.primary.default)
                            .border_1()
                            .border_color(theme.border.primary.default)
                            .opacity(0.0)
                            .group_hover(&group_name, |this| this.opacity(1.0))
                            .in_focus(|this| this.border_color(theme.text.primary.default)),
                    ),
            );

        root.style().refine(&self.style);

        root
    }
}
