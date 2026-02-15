use std::{cell::Cell, rc::Rc};

use gpui::{
    App, Bounds, ElementId, Entity, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    MouseButton, MouseEvent, ParentElement, Pixels, Point, Refineable, RenderOnce, StyleRefinement,
    Styled, Window, actions, canvas, div, relative, rems,
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
    state: SliderState,
    on_commit: Option<Rc<ChangeCallback>>,
    style: StyleRefinement,
}

#[derive(Clone, Copy)]
pub struct SliderState {
    default_value: f32,
    min: f32,
    max: f32,
    step: f32,
}

impl Default for SliderState {
    fn default() -> Self {
        Self {
            default_value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
        }
    }
}

impl SliderState {
    pub fn snap_to_step(&self, value: f32) -> f32 {
        let clamped = value.clamp(self.min, self.max);
        let steps = ((clamped - self.min) / self.step).round();

        self.min + steps * self.step
    }

    pub fn mouse_to_value(&self, mouse: Point<Pixels>, bounds: Bounds<Pixels>) -> f32 {
        let mouse_x = mouse.x - bounds.origin.x;
        let track_width = bounds.size.width;

        let normalized = (mouse_x / track_width).clamp(0.0, 1.0);
        self.snap_to_step(self.min + normalized * (self.max - self.min))
    }
}

impl Slider {
    pub fn new(
        id: impl Into<ElementId> + Clone,
        focus_handle: FocusHandle,
        default_value: f32,
    ) -> Self {
        Self {
            id: id.clone().into(),
            focus_handle,
            on_commit: None,
            state: SliderState {
                default_value,
                ..SliderState::default()
            },
            style: StyleRefinement::default(),
        }
    }

    pub fn on_commit(mut self, callback: impl Fn(f32, &mut App) + 'static) -> Self {
        self.on_commit = Some(Rc::new(callback));
        self
    }

    pub fn min(mut self, min: f32) -> Self {
        self.state.min = min;
        self
    }

    pub fn max(mut self, max: f32) -> Self {
        self.state.max = max;
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        self.state.step = step;
        self
    }

    fn action<T>(
        &self,
        internal: &Entity<f32>,
        f: impl Fn(f32, SliderState) -> f32 + 'static,
    ) -> impl Fn(&T, &mut Window, &mut App) + 'static {
        let internal = internal.clone();
        let on_commit = self.on_commit.clone();
        let state = self.state;

        move |_, _window, cx| {
            let current = internal.read(cx);
            let new_value = f(*current, state);
            internal.write(cx, new_value);

            if let Some(ref callback) = on_commit {
                callback(new_value, cx);
            }
        }
    }

    fn stop_drag_handler<T>(
        &self,
        internal: &Entity<f32>,
        is_dragging: &Entity<bool>,
        check_drag: bool,
    ) -> impl Fn(&T, &mut Window, &mut App) + 'static
    where
        T: MouseEvent,
    {
        let internal = internal.clone();
        let is_dragging = is_dragging.clone();
        let on_commit = self.on_commit.clone();

        move |_event, _window, cx| {
            let should_stop = !check_drag || *is_dragging.read(cx);

            if should_stop {
                is_dragging.write(cx, false);
                if let Some(ref cb) = on_commit {
                    let final_value = *internal.read(cx);
                    cb(final_value, cx);
                }
            }
        }
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // TODO: rework this so the parent passes in the value instead of using a keyed state.

        let internal = window.use_keyed_state(self.id.clone(), cx, |_, _| self.state.default_value);
        let is_dragging = window.use_keyed_state(format!("{}-dragging", self.id), cx, |_, _| false);

        let theme = cx.global::<Theme>();
        let group_name = format!("{}:group", &self.id);

        let track_bounds = Rc::new(Cell::new(Bounds::default()));

        let normalized_value =
            (internal.read(cx) - self.state.min) / (self.state.max - self.state.min);

        let mut root = div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle)
            .child(canvas(
                {
                    let track_bounds = track_bounds.clone();
                    move |bounds, _window, _cx| {
                        track_bounds.set(bounds);
                    }
                },
                |_bounds, _state, _window, _cx| {},
            ))
            .on_action::<Decrease>(self.action(&internal, |v, s| s.snap_to_step(v - s.step)))
            .on_action::<Increase>(self.action(&internal, |v, s| s.snap_to_step(v + s.step)))
            .on_action::<JumpStart>(self.action(&internal, |_, s| s.min))
            .on_action::<JumpEnd>(self.action(&internal, |_, s| s.max))
            .on_mouse_down(MouseButton::Left, {
                let internal = internal.clone();
                let track_bounds = track_bounds.clone();
                let is_dragging = is_dragging.clone();

                move |event, _window, cx| {
                    is_dragging.write(cx, true);
                    let new_value = self
                        .state
                        .mouse_to_value(event.position, track_bounds.get());
                    internal.write(cx, new_value);
                }
            })
            .on_mouse_move({
                let internal = internal.clone();

                move |event, _window, cx| {
                    if event.dragging() {
                        let new_value = self
                            .state
                            .mouse_to_value(event.position, track_bounds.get());
                        internal.write(cx, new_value);
                    }
                }
            })
            .on_mouse_up(
                MouseButton::Left,
                self.stop_drag_handler(&internal, &is_dragging, false),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                self.stop_drag_handler(&internal, &is_dragging, true),
            )
            .group(&group_name)
            .min_w(rems(9.0))
            .h_2()
            .relative()
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
