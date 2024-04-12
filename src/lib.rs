mod hid;

use std::time::{Duration, Instant};

use anyhow::Context;
use hid::Headphone;
use log::{error, info};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    TrayIconBuilder,
};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

struct AppState {
    headphone: Option<Headphone>,
    icons: Vec<tray_icon::Icon>,
}

pub fn run() -> anyhow::Result<()> {
    info!("Starting application");

    let headphone = hid::find_headphone().unwrap_or_else(|err| {
        error!("{err:?}");
        None
    });

    let mut app_state = AppState {
        headphone,
        icons: load_icons(),
    };

    let event_loop = EventLoopBuilder::new()
        .build()
        .context("Failed to create event loop")?;

    let menu_channel = MenuEvent::receiver();
    let menu_logs = MenuItem::new("View logs", true, None);
    let menu_github = MenuItem::new("View on Github", true, None);
    let menu_close = MenuItem::new("Close", true, None);

    let menu = Menu::new();

    menu.append(&menu_logs)
        .context("Failed to add context menu item")?;
    menu.append(&menu_github)
        .context("Failed to add context menu item")?;
    menu.append(&menu_close)
        .context("Failed to add context menu item")?;

    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(app_state.icons[icon_index(0)].clone())
        .build()
        .context("Failed to create tray icon")?;

    if let Err(e) = update(&tray, &mut app_state) {
        error!("Failed to update status: {e:?}");
    };

    let mut last_update = Instant::now();

    event_loop
        .run(move |_event, event_loop| {
            event_loop.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_secs(1),
            ));

            if last_update.elapsed() > Duration::from_millis(500) {
                if let Err(e) = update(&tray, &mut app_state) {
                    error!("Failed to update status: {e:?}");
                };
                last_update = Instant::now();
            }

            if let Ok(event) = menu_channel.try_recv() {
                if event.id == menu_close.id() {
                    // close button
                    event_loop.exit()
                } else if event.id == menu_github.id() {
                    // github button
                    let url = "https://github.com/aarol/arctis-battery-indicator";

                    if let Err(e) = std::process::Command::new("explorer").arg(url).spawn() {
                        error!("Failed to open {url}: {e:?}");
                    }
                } else if event.id == menu_logs.id() {
                    // logs button
                    if let Some(local_appdata) = dirs::data_local_dir() {
                        let path = local_appdata.join("ArctisBatteryIndicator");

                        if let Err(e) = std::process::Command::new("explorer").arg(&path).spawn() {
                            error!("Failed to path {path:?}: {e:?}");
                        }
                    }
                }
            }
            // system dark/light mode changes would be listened for here
            // unfortunately, window events don't seem to be emitted
            // when the event loop doesn't have a window attached.
        })
        .context("Event loop exited unexpectedly")?;

    Ok(())
}

fn update(tray: &tray_icon::TrayIcon, state: &mut AppState) -> anyhow::Result<()> {
    if state.headphone.is_none() {

        state.headphone = hid::find_headphone().unwrap_or_else(|err| {
            error!("{err:?}");
            None
        });
    }

    match state.headphone {
        Some(ref mut headphone) => match headphone.update() {
            Err(err) => {
                // an error will only occur when reading/writing to the device fails
                // in that situation, the best course of action is to try to reconnect
                error!("Failed to access device: {err:?}; trying to reconnect...");
                state.headphone = None
            }
            Ok(changed) => {
                if changed {
                    let tooltip_text = headphone.to_string();
                    info!("State has changed. New state: {tooltip_text}");
                    tray.set_tooltip(Some(tooltip_text))?;

                    tray.set_icon(Some(
                        state.icons[icon_index(headphone.battery_state)].clone(),
                    ))?;
                }
            }
        },
        None => {
            tray.set_tooltip(Some("No headphone adapter found"))?;
        }
    }

    Ok(())
}

// Icons are in resources at IDs [10, 20, 30, 40, 50]
// additionally, light mode icons are at [11, 21, 31, 41, 51]
fn load_icons() -> Vec<tray_icon::Icon> {
    (10..=50)
        .step_by(10)
        .chain((11..=51).step_by(10))
        .map(|d| {
            tray_icon::Icon::from_resource(d, None)
                .unwrap_or_else(|err| panic!("Failed to find icon #{d}: {err}"))
        })
        .collect()
}

fn icon_index(state: u8) -> usize {
    if is_dark_mode() {
        state as usize
    } else {
        state as usize + 5
    }
}

const SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
const VALUE: &str = "AppsUseLightTheme";

fn is_dark_mode() -> bool {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    if let Ok(subkey) = hkcu.open_subkey(SUBKEY) {
        if let Ok(dword) = subkey.get_value::<u32, _>(VALUE) {
            return dword == 0;
        }
    }

    false
}
