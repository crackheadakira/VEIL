use crate::{
    app::{builder::handle_state_setup, state::AppState},
    ui::{
        components::{button::Button, sidebar::Sidebar, switch::Switch},
        theme::Theme,
        views::all_albums::AllAlbumsView,
    },
};
use gpui::{
    App, AppContext, Application, Bounds, Context, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Point, Render, SharedString, StatefulInteractiveElement, Styled,
    TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
    actions, div, px, size,
};
use logging::lock_or_log;

pub use state::VeilState;

mod builder;
pub mod state;

actions!(app, [Quit]);

pub fn run() {
    Application::new().run(|cx: &mut App| {
        cx.on_action(|_: &Quit, cx| cx.quit());

        handle_state_setup(cx).expect("Failed setting up the state");
        let theme = Theme::default();
        cx.set_global::<Theme>(theme);

        cx.on_app_quit(|cx: &mut App| {
            let state = cx.global::<AppState>().0.clone();

            async move {
                let config = lock_or_log(state.config.read(), "Config Lock").unwrap();
                let mut discord = lock_or_log(state.discord.lock(), "Discord Mutex").unwrap();

                if config.integrations.discord_enabled {
                    discord.close();
                };

                state.db.shutdown().unwrap();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(1280.0), px(720.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: WindowBackgroundAppearance::Opaque,
                window_min_size: Some(size(px(1280.0), px(720.0))),
                titlebar: Some(TitlebarOptions {
                    title: Some(SharedString::from("VEIL")),
                    appears_transparent: true,
                    traffic_light_position: Some(Point {
                        x: px(12.0),
                        y: px(11.0),
                    }),
                }),
                app_id: Some("org.crackheadakira.veil".to_owned()),
                kind: WindowKind::Normal,
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("VEIL");

                cx.new(AppWindow::new)
            },
        )
        .unwrap();
    });
}

struct AppWindow {
    focus_handle: FocusHandle,
    all_albums_view: AllAlbumsView,
}

impl AppWindow {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            all_albums_view: AllAlbumsView::new(cx),
        }
    }
}

impl Focusable for AppWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AppWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        div()
            .size_full()
            .bg(theme.background.primary.default)
            .flex()
            .flex_col()
            .child(
                div().flex().size_full().child(Sidebar::new()).child(
                    div()
                        .flex_grow()
                        .p_8()
                        /*.child(Switch::new("switch-1"))
                        .child(Button::new("button-1", "Click me!"))*/
                        .child(self.all_albums_view.clone()),
                ),
            )
    }
}
