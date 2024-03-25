#![allow(unused)]

mod hid;

use std::{
    fmt::{self},
    io::{self, Read},
    os::raw,
    thread,
    time::{Duration, Instant},
};

use hid::Headphone;
use hidapi::HidApi;
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder, TrayIconEvent,
};
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoopBuilder},
};

struct IconState {
    headphone: Option<Headphone>,
    icons: Vec<tray_icon::Icon>,
}

fn main() {
    let controller = hid::Controller::new();

    let mut icon_state = IconState {
        headphone: controller.get_device(),
        icons: load_icons(),
    };

    let event_loop = EventLoopBuilder::new().build().unwrap();

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let menu_close = MenuItem::new("Close", true, None);

    let menu = Menu::new();

    menu.append(&menu_close);

    let mut tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(tray_icon::Icon::from_resource(1, None).unwrap())
        .build()
        .unwrap();

    update(&tray, &mut icon_state);

    let mut last_update = Instant::now();

    event_loop.run(move |_event, event_loop| {
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs(10),
        ));

        if last_update.elapsed() > Duration::from_secs(1) {
            update(&tray, &mut icon_state);
            last_update = Instant::now();
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == menu_close.id() {
                event_loop.exit()
            }
        }
    });
}

fn update(tray: &tray_icon::TrayIcon, state: &mut IconState) -> tray_icon::Result<()> {
    if let Some(ref mut headphone) = state.headphone {
        let changed = headphone.update().unwrap();

        if changed {
            tray.set_tooltip(Some(dbg!(headphone.to_string())))?;

            let icon_idx: usize = if is_dark_mode() {
                headphone.battery_state as usize
            } else {
                headphone.battery_state as usize + 5
            };
            tray.set_icon(Some(state.icons[icon_idx].clone()))?;
        }
    } else {
        tray.set_tooltip(Some("Unconnected"))?;
    }

    Ok(())
}

// Icons are in resources at IDs [10, 20, 30, 40, 50]
fn load_icons() -> Vec<tray_icon::Icon> {
    (10..=50)
        .step_by(10)
        .chain((11..=51).step_by(10))
        .map(|d| {
            tray_icon::Icon::from_resource(d, None).expect(&format!("Failed to find icon #{d}"))
        })
        .collect()
}

const SUBKEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
const VALUE: &str = "AppsUseLightTheme";

pub fn is_dark_mode() -> bool {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    if let Ok(subkey) = hkcu.open_subkey(SUBKEY) {
        if let Ok(dword) = subkey.get_value::<u32, _>(VALUE) {
            return dword == 0;
        }
    }

    false
}
