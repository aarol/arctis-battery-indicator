mod hid;
mod icon;
mod headphone_models;

use std::time::{Duration, Instant};

use anyhow::Context;
use hid::Headphone;
use icon::IconLoader;
use log::{error, info, trace};
use rust_i18n::t;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIcon, TrayIconBuilder,
};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Theme,
};

rust_i18n::i18n!("locales", fallback = "en-US");

struct AppState {
    headphone: Option<Headphone>,
    icon_loader: icon::IconLoader,

    tray_icon: TrayIcon,
    menu_logs: MenuItem,
    menu_github: MenuItem,
    menu_close: MenuItem,

    last_update: Instant,
    is_debug: bool,
}

pub fn run(is_debug: bool) -> anyhow::Result<()> {
    info!("Starting application");

    let locale = &sys_locale::get_locale().unwrap_or("en-US".to_owned());
    trace!("Using locale {locale}");
    dbg!(rust_i18n::available_locales!());
    rust_i18n::set_locale(locale);

    let event_loop = EventLoop::new().context("Error initializing event loop")?;

    let mut app = AppState::init(is_debug)?;

    Ok(event_loop.run_app(&mut app)?)
}

impl AppState {
    pub fn init(is_debug: bool) -> anyhow::Result<Self> {
        let headphone = hid::find_headphone().unwrap_or_else(|err| {
            error!("{err:?}");
            None
        });

        let mut icon_loader = IconLoader::new();

        const VERSION: &'static str = env!("CARGO_PKG_VERSION");
        let version_str = t!("version");

        let menu_version = MenuItem::new(format!("{version_str} v{VERSION}"), false, None);

        let menu_logs = MenuItem::new(t!("view_logs"), true, None);
        let menu_github = MenuItem::new(t!("view_updates"), true, None);
        let menu_close = MenuItem::new(t!("quit_program"), true, None);

        let menu = Menu::new();

        menu.append_items(&[&menu_version, &menu_logs, &menu_github, &menu_close])
            .context("Failed to add context menu item")?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icon_loader.load(icon::IconImage::Battery0, true))
            .build()
            .context("Failed to create tray icon")?;

        Ok(Self {
            headphone,
            icon_loader,
            tray_icon,
            menu_close,
            menu_github,
            menu_logs,
            last_update: Instant::now(),
            is_debug,
        })
    }

    fn update(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        if self.headphone.is_none() {
            self.headphone = hid::find_headphone().unwrap_or_else(|err| {
                error!("{err:?}");
                None
            });
        }

        match self.headphone {
            Some(ref mut headphone) => match headphone.update() {
                Err(err) => {
                    // an error will only occur when reading/writing to the device fails
                    // in that situation, the best course of action is to try to reconnect
                    error!("Failed to access device: {err:?}; trying to reconnect...");
                    self.headphone = None
                }
                Ok(changed) => {
                    if changed {
                        let mut tooltip_text = headphone.to_string();

                        if self.is_debug {
                            tooltip_text += " (Debug)"
                        }

                        info!("State has changed. New state: {headphone:?}");
                        self.tray_icon.set_tooltip(Some(tooltip_text))?;

                        self.tray_icon.set_icon(Some(
                            self.icon_loader
                                .load(headphone.status_icon(), Self::is_dark_mode(event_loop)),
                        ))?;
                    }
                }
            },
            None => {
                self.tray_icon.set_tooltip(Some(t!("no_adapter_found")))?;
            }
        }

        Ok(())
    }

    fn is_dark_mode(event_loop: &ActiveEventLoop) -> bool {
        event_loop.system_theme() != Some(Theme::Light)
    }
}

impl ApplicationHandler<()> for AppState {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        // Kick off polling every 1 second
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs(1),
        ));
    }
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            // Overwrite the current polling time
            //
            // If not overwritten, it starts polling multiple times a second
            // since the timer is already elapsed.
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_secs(1),
            ));
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // This will be called at least every second
        if self.last_update.elapsed() > Duration::from_millis(500) {
            if let Err(e) = self.update(event_loop) {
                error!("Failed to update status: {e:?}");
            };
            self.last_update = Instant::now();
        }
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id {
                id if id == self.menu_close.id() => event_loop.exit(),

                id if id == self.menu_github.id() => {
                    let url = "https://github.com/aarol/arctis-battery-indicator/releases";

                    if let Err(e) = std::process::Command::new("explorer").arg(url).spawn() {
                        error!("Failed to open {url}: {e:?}");
                    }
                }
                id if id == self.menu_logs.id() => {
                    if let Some(local_appdata) = dirs::data_local_dir() {
                        let path = local_appdata.join("ArctisBatteryIndicator");

                        if let Err(e) = std::process::Command::new("explorer").arg(&path).spawn() {
                            error!("Failed to open path {path:?}: {e:?}");
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: WindowEvent,
    ) {
        // Since we don't have a window attached, this will never be called
    }
}
