use std::{collections::VecDeque, time::Duration};

use gpui::{
    AnyView, App, Context, Entity, EntityId, FocusHandle, Focusable, Global, InteractiveElement,
    IntoElement, KeyBinding, MouseButton, ParentElement, Pixels, Point, Render, Styled, Window,
    actions, div, prelude::FluentBuilder, rgba,
};

actions!(modal_layer, [CloseTopmost]);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("escape", CloseTopmost, None)]);
}

pub struct ModalLayer {
    focus_handle: FocusHandle,
    modal: Option<AnyView>,
    toasts: VecDeque<AnyView>,
    context_menu: Option<(AnyView, Point<Pixels>)>,
}

impl ModalLayer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            modal: None,
            toasts: VecDeque::with_capacity(3),
            context_menu: None,
        }
    }

    pub fn close_topmost(&mut self, cx: &mut Context<Self>) -> bool {
        if self.context_menu.is_some() {
            self.close_context_menu(cx);
            true
        } else if self.modal.is_some() {
            self.close_modal(cx);
            true
        } else {
            false
        }
    }

    // MODAL METHODS

    pub fn show_modal(&mut self, view: AnyView, cx: &mut Context<Self>) {
        self.modal = Some(view);
        cx.notify();
    }

    pub fn close_modal(&mut self, cx: &mut Context<Self>) {
        self.modal = None;
        cx.notify();
    }

    pub fn has_modal(&self) -> bool {
        self.modal.is_some()
    }

    // TOAST METHODS

    fn show_toast(&mut self, view: AnyView, cx: &mut Context<Self>) {
        let toast_view = view.clone();
        self.toasts.push_front(view);
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(3)).await;
            this.update(cx, |this, cx| {
                this.toasts
                    .retain(|t| !t.entity_id().eq(&toast_view.entity_id()));
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn dismiss_toast(&mut self, toast_id: &EntityId, cx: &mut Context<Self>) {
        self.toasts.retain(|t| !t.entity_id().eq(toast_id));
        cx.notify();
    }

    pub fn clear_toasts(&mut self, cx: &mut Context<Self>) {
        self.toasts.clear();
        cx.notify();
    }

    // CONTEXT MENU METHODS

    pub fn show_context_menu(
        &mut self,
        view: AnyView,
        position: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        self.context_menu = Some((view, position));
        cx.notify();
    }

    pub fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        self.context_menu = None;
        cx.notify();
    }

    pub fn has_context_menu(&self) -> bool {
        self.context_menu.is_some()
    }
}

impl Focusable for ModalLayer {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ModalLayer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let size = window.viewport_size();

        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .absolute()
            .top_0()
            .left_0()
            .on_action(cx.listener(|this, _: &CloseTopmost, _window, cx| {
                this.close_topmost(cx);
            }))
            .when_some(self.modal.clone(), |this, modal| {
                this.child(
                    div()
                        .id("modal-backdrop")
                        .w(size.width)
                        .h(size.height)
                        .bg(rgba(0x00000080))
                        .flex()
                        .justify_center()
                        .items_center()
                        .occlude()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|this, _, _, cx| {
                                this.close_modal(cx);
                            }),
                        )
                        .child(
                            div()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    cx.stop_propagation();
                                })
                                .child(modal),
                        ),
                )
            })
            .when_some(self.context_menu.clone(), |this, (menu, position)| {
                this.child(
                    div()
                        .absolute()
                        .left(position.x)
                        .top(position.y)
                        .child(menu),
                )
            })
            .child(
                div()
                    .absolute()
                    .bottom_0()
                    .right_0()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .children(self.toasts.iter().map(|toast| toast.clone())),
            )
    }
}

pub struct GlobalModalLayer(pub Entity<ModalLayer>);

impl Global for GlobalModalLayer {}

// might remove these...

/// Trait for modal views that can be displayed in the modal layer
pub trait ModalView: Render {
    /// Called when the modal should close (escape key, backdrop click, etc.)
    fn on_close(&mut self, cx: &mut Context<Self>) {
        let modal_layer = cx.global::<GlobalModalLayer>().0.clone();
        modal_layer.update(cx, |layer, cx| {
            layer.close_modal(cx);
        });
    }

    /// Whether clicking the backdrop should close this modal
    fn dismiss_on_backdrop_click(&self) -> bool {
        true
    }
}

/// Trait for context menu views
pub trait ContextMenuView: Render {
    /// The position where the context menu should appear
    fn position(&self) -> Point<Pixels>;

    /// Called when a menu item is selected or menu is dismissed
    fn on_close(&mut self, cx: &mut Context<Self>) {
        let modal_layer = cx.global::<GlobalModalLayer>().0.clone();
        modal_layer.update(cx, |layer, cx| {
            layer.close_context_menu(cx);
        });
    }

    /// Called when clicking outside the menu
    fn on_click_outside(&mut self, cx: &mut Context<Self>) {
        self.on_close(cx);
    }
}

/// Trait for toast notifications
pub trait ToastView: Render {
    /// Severity level for styling
    fn severity(&self) -> ToastSeverity {
        ToastSeverity::Info
    }

    /// How long before auto-dismissing
    fn duration(&self) -> Duration {
        Duration::from_secs(3)
    }

    /// Whether user can manually dismiss
    fn dismissible(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToastSeverity {
    Info,
    Success,
    Warning,
    Error,
}
