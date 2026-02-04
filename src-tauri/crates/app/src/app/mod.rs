use crate::app::{builder::handle_state_setup, state::AppState};
use gpui::{
    App, AppContext, Application, Bounds, Context, FocusHandle, Focusable, IntoElement,
    ParentElement, Point, Render, SharedString, Styled, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, actions, div, px, rgb,
    size, white,
};
use logging::lock_or_log;

pub use state::VeilState;

mod builder;
mod state;

actions!(app, [Quit]);

pub fn run() {
    Application::new().run(|cx: &mut App| {
        cx.on_action(|_: &Quit, cx| cx.quit());

        handle_state_setup(cx).expect("Failed setting up the state");

        cx.on_app_quit(|cx: &mut App| {
            let state = cx.global::<AppState>().0;

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

        let window = cx
            .open_window(
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
                    app_id: Some("org.crackheadakira.veil".to_string()),
                    kind: WindowKind::Normal,
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("VEIL");

                    cx.new(|cx| AppWindow::new(cx))
                },
            )
            .unwrap();
    });
}

struct AppWindow {
    focus_handle: FocusHandle,
}

impl AppWindow {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self { focus_handle }
    }
}

impl Focusable for AppWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for AppWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(white()).child(
            div()
                .w_full()
                .p(px(12.0))
                .line_height(px(14.0))
                .text_sm()
                .border_1()
                .border_color(rgb(0xcccccc))
                .child("Hello, world!"),
        )
    }
}
