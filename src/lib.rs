mod headset_control;
mod lang;

use lang::Key::*;
use std::time::{Duration, Instant};

use anyhow::Context;
use log::{error, info};
use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use winit::{
    application::ApplicationHandler,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::Theme,
};

use crate::headset_control::BatteryState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct AppState {
    tray_icon: TrayIcon,
    devices: Vec<headset_control::Device>,
    selected_device_idx: usize,
    menu: Menu,
    menu_logs: MenuItem,
    menu_github: MenuItem,
    menu_close: MenuItem,

    last_update: Instant,
    should_update_icon: bool,
}

pub fn run() -> anyhow::Result<()> {
    info!("Starting application");
    info!("Version {VERSION}");

    let event_loop = EventLoop::new().context("Error initializing event loop")?;

    let mut app = AppState::init()?;

    Ok(event_loop.run_app(&mut app)?)
}

impl AppState {
    pub fn init() -> anyhow::Result<Self> {
        let menu_version = MenuItem::new(format!("{} v{}", lang::t(version), VERSION), false, None);

        let menu_logs = MenuItem::new(lang::t(view_logs), true, None);
        let menu_github = MenuItem::new(lang::t(view_updates), true, None);
        let menu_close = MenuItem::new(lang::t(quit_program), true, None);
        let menu = Menu::new();

        menu.append(&PredefinedMenuItem::separator())?;

        menu.append_items(&[&menu_version, &menu_logs, &menu_github, &menu_close])
            .context("Failed to add context menu item")?;

        let icon = Self::load_icon(Theme::Dark, 0, BatteryState::BatteryUnavailable)
            .context("loading fallback disconnected icon")?;

        let tray_icon = TrayIconBuilder::new()
            .with_icon(icon)
            .with_menu(Box::new(menu.clone()))
            .build()
            .context("Failed to create tray icon")?;

        Ok(Self {
            tray_icon,
            menu,
            menu_close,
            menu_github,
            selected_device_idx: 0,
            menu_logs,
            devices: vec![],
            last_update: Instant::now(),
            should_update_icon: true,
        })
    }

    fn update(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        headset_control::query_devices(&mut self.devices)?;

        if self.devices.is_empty() {
            self.tray_icon
                .set_tooltip(Some(lang::t(no_adapter_found)))?;
            return Ok(());
        }

        if self.selected_device_idx < self.devices.len() {
            self.selected_device_idx = self.devices.len() - 1;
        }
        let device = &self.devices[self.selected_device_idx];

        #[allow(unused_mut)]
        let mut tooltip_text = device.to_string();

        #[cfg(debug_assertions)]
        {
            tooltip_text += " (Debug)";
        }

        self.tray_icon
            .set_tooltip(Some(&tooltip_text))
            .with_context(|| format!("setting tooltip text: {tooltip_text}"))?;

        let battery_percent = device.battery.level;

        match Self::load_icon(
            event_loop.system_theme().unwrap_or(Theme::Dark),
            battery_percent,
            device.battery.status,
        ) {
            Ok(icon) => self.tray_icon.set_icon(Some(icon))?,
            Err(err) => error!("Failed to load icon: {err:?}"),
        }

        self.should_update_icon = false;

        Ok(())
    }

    fn load_icon(
        theme: winit::window::Theme,
        battery_percent: isize,
        state: BatteryState,
    ) -> anyhow::Result<tray_icon::Icon> {
        // Map battery_percent to icon resource id
        let level = match battery_percent {
            -1 => 1,
            0..=12 => 1,  // 0%
            13..=37 => 2, // 25%
            38..=62 => 3, // 50%
            63..=87 => 4, // 75%
            _ => 5,       // 100%
        };

        // light mode icons are (10,20,...,50)
        // dark mode icons are (15,25,...,55)
        let theme_offset: u16 = if theme == Theme::Light { 5 } else { 0 };
        // Charging icons are at icon id + 1
        let charging_offset = (state == BatteryState::BatteryCharging) as u16;

        let res_id = if state == BatteryState::BatteryUnavailable {
            10 + theme_offset // empty icon
        } else {
            level * 10 + theme_offset + charging_offset
        };

        tray_icon::Icon::from_resource(res_id, None)
            .with_context(|| format!("loading icon from resource {res_id}"))
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
        if self.last_update.elapsed() > Duration::from_millis(1000) {
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
                    if let Ok(dir) = std::env::current_dir() {
                        if let Err(e) = std::process::Command::new("explorer").arg(&dir).spawn() {
                            error!("Failed to open path {dir:?}: {e:?}");
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

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        info!("Exiting application..");
    }
}

#[test]
fn load_all_icons() {
    for i in 0..=100 {
        let _ = AppState::load_icon(Theme::Dark, i, BatteryState::BatteryAvailable);
    }
    for i in 0..=100 {
        let _ = AppState::load_icon(Theme::Light, i, BatteryState::BatteryAvailable);
    }
}
