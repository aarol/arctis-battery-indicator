use anyhow::Context;
use serde::Deserialize;

use crate::hid::HeadphoneModel;

#[derive(Deserialize, Clone)]
pub struct ConfigFile {
    pub name: String,
    pub product_id: u16,
    /// Magic bytes that make the device output battery information
    pub write_bytes: [u8; 2],
    pub interface_num: Option<i32>,
    pub battery_percent_idx: usize,
    pub charging_status_idx: Option<usize>,
    pub connected_status_idx: Option<usize>,
    /// Usage page first, then id
    pub usage_page_id: Option<(u16, u16)>,
    /// Size of buffer to send when reading battery status
    pub read_buf_size: usize,
    pub battery_range: (u8, u8),
}

impl From<ConfigFile> for HeadphoneModel {
    fn from(value: ConfigFile) -> Self {
        Self {
            // HeadphoneModel expects a static string
            // leak this string to make it a 'static str
            // OK since this will happen only once
            name: Box::leak(value.name.into_boxed_str()),
            battery_percent_idx: value.battery_percent_idx,
            product_id: value.product_id,
            write_bytes: value.write_bytes,
            interface_num: value.interface_num,
            charging_status_idx: value.charging_status_idx,
            connected_status_idx: value.connected_status_idx,
            usage_page_and_id: value.usage_page_id,
            read_buf_size: value.read_buf_size,
            battery_range: value.battery_range,
        }
    }
}

pub fn config_file_exists() -> bool {
    if let Ok(dir) = std::env::current_dir().context("getting current working directory") {
        let path = dir.join("config.toml");
        return path.exists();
    } else {
        false
    }
}

pub fn load_config() -> anyhow::Result<ConfigFile> {
    let dir = std::env::current_dir().context("getting current working directory")?;
    let path = dir.join("config.toml");

    let string = std::fs::read_to_string(&path)
        .with_context(|| format!("trying to read config file at {path:?}"))?;

    toml::from_str(string.as_str())
        .with_context(|| format!("parsing configuration file at {path:?}"))
}
