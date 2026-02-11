use std::rc::Rc;

use gpui::{
    App, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::{app::Route, ui::theme::Theme};

type NavigateHandler = Rc<dyn Fn(&Route, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Sidebar {
    on_navigate: Option<NavigateHandler>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self { on_navigate: None }
    }

    pub fn on_navigate(
        mut self,
        handler: impl Fn(&Route, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_navigate = Some(Rc::new(handler));
        self
    }

    fn route_button(
        label: impl Into<SharedString> + IntoElement + Clone,
        route: Route,
        navigate: Option<NavigateHandler>,
    ) -> impl IntoElement {
        div()
            .id(format!("sidebar:{}", label.clone().into()))
            .on_click({
                let navigate = navigate.clone();
                move |_, window, app| {
                    if let Some(handler) = &navigate {
                        handler(&route, window, app);
                    }
                }
            })
            .cursor_pointer()
            .child(label)
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let navigate = self.on_navigate.clone();

        div()
            .id("sidebar")
            .overflow_y_scroll()
            .flex_shrink_0()
            .h_full()
            .w_72()
            .bg(theme.background.primary.default)
            .border_r_1()
            .border_color(theme.border.secondary.default)
            .text_color(theme.text.primary.default)
            .flex()
            .flex_col()
            .gap_2()
            .child(Self::route_button("Home", Route::Home, navigate.clone()))
            .child(Self::route_button(
                "All Albums",
                Route::AllAlbums,
                navigate.clone(),
            ))
    }
}
