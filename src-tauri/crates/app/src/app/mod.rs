use std::borrow::Cow;

use crate::{
    app::{builder::handle_state_setup, state::AppState},
    ui::{
        components::{sidebar::Sidebar, slider},
        image_cache::AlbumCoverCacheProvider,
        theme::Theme,
        views::{all_albums::AllAlbumsView, home::Home},
    },
};
use gpui::{
    App, AppContext, Application, AssetSource, Bounds, Context, Entity, FocusHandle, Focusable,
    IntoElement, ParentElement, Point, Render, SharedString, Styled, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, actions, div, px, size,
};
use logging::lock_or_log;

pub use state::VeilState;

mod builder;
pub mod state;
actions!(app, [Quit]);

struct DiskAssets;

impl AssetSource for DiskAssets {
    fn list(&self, _path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Vec::new())
    }

    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(Some(Cow::Owned(bytes))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

pub fn run() {
    Application::new()
        .with_assets(DiskAssets)
        .run(|cx: &mut App| {
            cx.on_action(|_: &Quit, cx| cx.quit());

            handle_state_setup(cx).expect("Failed setting up the state");
            let theme = Theme::default();
            cx.set_global::<Theme>(theme);

            slider::bind_keys(cx);

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

#[derive(Clone, Copy)]
pub enum Route {
    Home,
    AllAlbums,
    Album { id: u32 },
    Playlist { id: u32 },
    Settings,
}

struct AppWindow {
    focus_handle: FocusHandle,
    all_albums_view: Option<Entity<AllAlbumsView>>,
    home: Option<Home>,
    route: Route,
}

impl AppWindow {
    fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        Self {
            focus_handle,
            all_albums_view: None,
            home: None,
            route: Route::Home,
        }
    }

    fn navigate(&mut self, route: Route, cx: &mut Context<Self>) {
        self.route = route;
        cx.notify();
    }

    fn render_route(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        match &self.route {
            Route::Home => {
                let view = self
                    .home
                    .get_or_insert_with(|| Home::new(self.focus_handle.clone()));
                view.clone().into_any_element()
            }
            Route::AllAlbums => {
                let view = self
                    .all_albums_view
                    .get_or_insert_with(|| cx.new(|cx| AllAlbumsView::new(cx)));

                view.clone().into_any_element()
            }
            _ => div().into_any_element(),
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
                div()
                    .flex()
                    .size_full()
                    .child(Sidebar::new().on_navigate(cx.listener(
                        |this: &mut AppWindow, route: &Route, _, cx| {
                            this.navigate(route.clone(), cx);
                        },
                    )))
                    .child(div().flex_grow().p_8().child(self.render_route(cx))),
            )
    }
}
