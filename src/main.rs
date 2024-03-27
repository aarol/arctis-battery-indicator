#![windows_subsystem = "windows"]

use std::fs::File;

use arctis_battery_indicator::run;
use log::error;
use simplelog::{ConfigBuilder, WriteLogger};

fn main() {
    // Cannot really log anything if initializing logging fails
    let _ = init_file_logger();

    if let Err(e) = run() {
        error!("Application stopped unexpectedly: {e:?}");
    }
}

pub fn init_file_logger() -> anyhow::Result<()> {
    match dirs::config_local_dir() {
        None => {
            anyhow::bail!("Failed to locate APPDATA_LOCAL dir")
        }
        Some(appdata_local) => {
            let log_dir = appdata_local.join("ArctisBatteryIndicator");
            std::fs::DirBuilder::new()
                .recursive(true)
                .create(&log_dir)?;

            let log_file = File::options()
                .append(true)
                .create(true)
                .open(log_dir.join("arctisBatteryIndicator.log"))?;

            WriteLogger::init(
                log::LevelFilter::Info,
                ConfigBuilder::new().set_time_format_rfc3339().build(),
                log_file,
            )?;

            Ok(())
        }
    }
}
