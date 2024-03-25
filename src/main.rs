#![allow(unused)]

mod hid;

use core::{slice::SlicePattern, time};
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
    TrayIconBuilder, TrayIconEvent,
};
use winit::event_loop::{ControlFlow, EventLoopBuilder};

fn main() {
    let controller = hid::Controller::new();

    let icon_state = IconState {
        headphone: controller.get_device(),
        last_state: 0,
    };

    let event_loop = EventLoopBuilder::new().build().unwrap();

    let menu_channel = MenuEvent::receiver();
    let tray_channel = TrayIconEvent::receiver();
    let menu_close = MenuItem::new("Close", true, None);
    let menu = Menu::new();

    menu.append(&menu_close);

    let mut tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .build()
        .unwrap();

    update_icon(&tray, headphone.as_ref());

    let mut last_update = Instant::now();

    event_loop.run(move |_event, event_loop| {
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_secs(10),
        ));

        if last_update.elapsed() > Duration::from_secs(1) {
            update_icon(&tray, headphone.as_ref());
            last_update = Instant::now();
        }

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == menu_close.id() {
                event_loop.exit()
            }
        }
    });
}

fn update_icon(tray: &tray_icon::TrayIcon, state: &mut IconState) -> tray_icon::Result<()> {
    if let Some(headphone) = &headphone {
        tray.set_tooltip(Some(headphone.to_string()))?;
    } else {
        tray.set_tooltip(Some("Unconnected"))?;
    }

    Ok(())
}

const RAW_ICONS: &[&'static [u8]] = &[
    include_bytes!("bat/battery0.png"),
    include_bytes!("bat/battery1.png"),
    include_bytes!("bat/battery2.png"),
    include_bytes!("bat/battery3.png"),
    include_bytes!("bat/battery4.png"),
];

fn load_icons() -> Result<&'static [&'static tray_icon::Icon], tray_icon::BadIcon> {
    RAW_ICONS.iter().map(|raw| {
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory_with_format(raw, image::ImageFormat::Png)
                .unwrap()
                .to_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        &tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
    }).collect();

}

struct IconState {
    headphone: Option<Headphone>,
    last_state: u8,
}