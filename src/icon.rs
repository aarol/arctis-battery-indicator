use std::collections::HashMap;

use image::GenericImageView;

#[derive(Eq, PartialEq, Hash)]
pub enum IconImage {
    Battery0,
    Battery1,
    Battery2,
    Battery3,
    Battery4,
}

impl IconImage {
    pub fn from_state(battery_state: u8, charging_state: Option<u8>) -> Self {
        match charging_state {
            // Not connected
            Some(0) => Self::Battery0,
            // Charging
            // Some(1) => match battery_state {
            //     0 => Self::Charging0,
            //     1 => Self::Charging1,
            //     2 => Self::Charging2,
            //     3 => Self::Charging3,
            //     4 => Self::Charging4,
            //     _ => unreachable!(),
            // },
            // Discharging
            _ => match battery_state {
                0 => Self::Battery0,
                1 => Self::Battery1,
                2 => Self::Battery2,
                3 => Self::Battery3,
                4 => Self::Battery4,
                _ => unreachable!(),
            },
        }
    }

    fn image_bytes<'a>(&self) -> &'a [u8] {
        match self {
            // IconImage::BatteryNotFound => include_bytes!("bat/notfound.png"),
            IconImage::Battery0 => include_bytes!("bat/battery0.png"),
            IconImage::Battery1 => include_bytes!("bat/battery1.png"),
            IconImage::Battery2 => include_bytes!("bat/battery2.png"),
            IconImage::Battery3 => include_bytes!("bat/battery3.png"),
            IconImage::Battery4 => include_bytes!("bat/battery4.png"),
            // IconImage::Charging0 => include_bytes!("bat/charging0.png"),
            // IconImage::Charging1 => include_bytes!("bat/charging1.png"),
            // IconImage::Charging2 => include_bytes!("bat/charging2.png"),
            // IconImage::Charging3 => include_bytes!("bat/charging3.png"),
            // IconImage::Charging4 => include_bytes!("bat/charging4.png"),
        }
    }
}

pub struct IconLoader {
    light_cache: HashMap<IconImage, tray_icon::Icon>,
    dark_cache: HashMap<IconImage, tray_icon::Icon>,
}
impl IconLoader {
    pub fn new() -> Self {
        Self {
            light_cache: HashMap::new(),
            dark_cache: HashMap::new(),
        }
    }
    pub fn load(&mut self, icon: IconImage, dark_mode: bool) -> tray_icon::Icon {
        let cached_icon = if dark_mode {
            self.dark_cache.get(&icon)
        } else {
            self.light_cache.get(&icon)
        };

        match cached_icon {
            Some(icon) => icon.clone(),
            None => {
                let image = image::load_from_memory_with_format(
                    &icon.image_bytes(),
                    image::ImageFormat::Png,
                )
                .expect("should be a valid PNG");

                let image = image.brighten(if dark_mode { 0 } else { -256 });
                let (width, height) = image.dimensions();
                let rgba: Vec<u8> = image.into_rgba8().to_vec();
                let tray_icon =
                    tray_icon::Icon::from_rgba(rgba, width, height).expect("should be valid RGBA");

                if dark_mode {
                    self.dark_cache.insert(icon, tray_icon.clone());
                } else {
                    self.dark_cache.insert(icon, tray_icon.clone());
                };

                tray_icon
            }
        }
    }
}

#[test]
fn load_all_icons() {
    for dark_mode in [true, false] {
        let mut loader = IconLoader::new();

        for icon in [
            // IconImage::BatteryNotFound,
            IconImage::Battery0,
            IconImage::Battery1,
            IconImage::Battery2,
            IconImage::Battery3,
            IconImage::Battery4,
        ] {
            let _icon = loader.load(icon, dark_mode);
        }
    }
}
