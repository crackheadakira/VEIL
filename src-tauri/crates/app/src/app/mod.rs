use crate::{
    app::{builder::handle_state_setup, state::AppState},
    ui::{components::toggle::ToggleButton, theme::Theme},
};
use common::Tracks;
use gpui::{
    App, AppContext, Application, Bounds, Context, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Point, Render, ScrollHandle, SharedString, Styled,
    TitlebarOptions, Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
    actions, div, px, size,
};
use logging::{info, lock_or_log};

pub use state::VeilState;

mod builder;
mod state;

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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        div()
            .text_color(theme.text.primary.default)
            .size_full()
            .bg(theme.background.primary.default)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w_full()
                    .p_3()
                    .text_sm()
                    .border_1()
                    .border_color(theme.border.primary.default)
                    .child("Hello from GPUI!")
                    .child(
                        div()
                            .bg(theme.background.secondary.default)
                            .w_full()
                            .p_3()
                            .text_sm()
                            .border_1()
                            .border_color(theme.border.secondary.default)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_, _, _, cx| {
                                    let state = cx.global::<AppState>().0.clone();

                                    info!("Trying to play track!");
                                    let mut player =
                                        lock_or_log(state.player.write(), "Player Write Lock")
                                            .unwrap();

                                    let track = state.db.by_id::<Tracks>(&1).unwrap();

                                    player.play(&track, None).unwrap();
                                }),
                            )
                            .child("Play"),
                    )
                    .child(ToggleButton::new("toggle-1"))
                    .child(ToggleButton::new("toggle-2").label("Discord RPC")),
            )
    }
}
