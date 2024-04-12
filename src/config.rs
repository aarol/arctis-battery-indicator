use std::io;

use anyhow::Context;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct CustomHeadphoneModel {
    pub name: String,
    pub product_id: u16,
    /// Magic bytes that make the device output battery information
    pub write_bytes: [u8; 2],
    pub interface_num: i32,
    pub battery_percent_idx: usize,
    pub charging_status_idx: Option<usize>,
}

#[derive(Clone, Deserialize)]
struct Config {
    headphones: Vec<CustomHeadphoneModel>,
}

pub fn load_custom_headphones(path: &str) -> anyhow::Result<Vec<CustomHeadphoneModel>> {
  let file = std::fs::File::open(path).with_context(|| format!("Failed to open {path}"))?;
  let contents = io::read_to_string(file)?;

  let config: Config = toml::from_str(contents.as_str()).context("Failed to parse config file")?;

  Ok(config.headphones)
}
