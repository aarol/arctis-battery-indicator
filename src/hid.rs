use std::{ffi::CStr, rc::Rc};

use anyhow::Context;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use log::{debug, error, info, trace, warn};
use rust_i18n::t;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargingState {
    Disconnected = 0,
    Charging = 1,
    Discharging = 3,
}

#[derive(Debug)]
pub struct Headphone {
    device: HidDevice,
    path: Rc<CStr>,
    model: HeadphoneModel,
    name: String,
    /// percentage in range [0,4]
    pub battery_state: u8,
    /// - 0: not connected
    /// - 1: charging
    /// - 3: discharging
    pub charging_state: Option<ChargingState>,
}

impl Headphone {
    /// Maps to an int between 0 and 100
    pub fn battery_percentage(&self) -> u8 {
        let (min, max) = self.model.battery_range;
        let clamped = self.battery_state.clamp(min, max);
        let normalized = (clamped - min) as f32 / ((max - min) as f32);
        (normalized * 100.0).round() as u8
    }

    pub fn status_text(&self) -> Option<String> {
        self.charging_state.map(|state| match state {
            ChargingState::Charging => t!("device_charging").into(),
            ChargingState::Discharging => "".into(),
            ChargingState::Disconnected => t!("device_disconnected").into(),
        })
    }

    /// if return is Ok(true), state has changed
    pub fn update(&mut self, hidapi: &HidApi) -> anyhow::Result<bool> {
        // Reconnect for every update
        // This may seem very wasteful, but for some reason
        // not reconnecting delays the information about changing state
        // for a significant time.
        self.device = hidapi.open_path(&self.path)?;

        self.device
            .write(&self.model.write_bytes)
            .with_context(|| format!("writing {:?} to device", self.model.write_bytes))?;

        let mut buf = [0u8; 128]; // buf is larger than any model's read_buf_size

        let n = self
            .device
            // timeout because we don't want to block indefinitely here
            .read_timeout(&mut buf[0..self.model.read_buf_size], 100)
            .with_context(|| "reading from device")?;

        trace!("read {n}: {:?}", &buf[0..self.model.read_buf_size]);

        // explicitly run each check
        if n == 0 {
            debug!("No data read from device; ignoring");
            return Ok(false);
        }
        if buf[0] == 0 {
            debug!("Read invalid bytes from device: {:?}; ignoring", &buf[0..5]);
            return Ok(false);
        }
        // skip this check? i think its unnecessary
        // if !self.model.write_bytes.contains(&buf[0]) {
        //     debug!("Read write bytes from device: {:?}; ignoring {:?}", &buf[0..5], &buf[0]);
        //     return Ok(false);
        // }

        // save old state
        let Headphone {
            battery_state: old_battery,
            charging_state: old_charging,
            ..
        } = *self;

        let battery_state = buf[self.model.battery_percent_idx];

        // check if battery state is within correct range
        let (battery_min, battery_max) = self.model.battery_range;
        if battery_state >= battery_min && battery_state <= battery_max {
            self.battery_state = battery_state;
        } else {
            // otherwise the data might be garbage
            debug!(
                "Returned battery state is invalid: {:x}; ignoring",
                battery_state
            );
        }

        if let Some(charging_state_idx) = self.model.charging_status_idx {
            // get the charging state, if theres no connected state, this'll be used.
            let mut device_state = buf[charging_state_idx];

            // if device config has a separate index for connected state
            if let Some(connected_state_idx) = self.model.connected_status_idx {
                let connected_state = buf[connected_state_idx];

                // for arctis 9, this is 1 when on and 3 when off/disconnected, but maybe its different for different devices.
                // assume 1 means on
                if connected_state != 1 {
                    // device isn't on, so set the state to disconnected
                    device_state = 0; // disconnected
                } else {
                    // device is on, set state
                    
                    // all devices with separate values probably use 0 for not charging and 1 for charging, like arctis 9
                    // maybe this has to be changed in future for other devices
                    if buf[charging_state_idx] == 0 {
                        device_state = 3; // not charging
                    } else if buf[charging_state_idx] == 1 {
                        device_state = 1; // charging
                    } else {
                        // invalid state
                        debug!("Returned charge state is invalid: {:x}; ignoring", device_state);
                        return Ok(false);
                    }
                }
            }

            self.charging_state = match device_state {
                0 => Some(ChargingState::Disconnected),
                1 => Some(ChargingState::Charging),
                3 => Some(ChargingState::Discharging),
                _ => {
                    debug!("Returned charge state is invalid: {}; ignoring", device_state);
                    None
                }
            }
        }

        Ok(self.battery_state != old_battery || self.charging_state != old_charging)
    }
}

impl std::fmt::Display for Headphone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name}: {battery}% {remaining}",
            name = self.name,
            battery = self.battery_percentage(),
            remaining = t!("battery_remaining")
        )?;

        if let Some(status) = self.status_text() {
            write!(f, " {status}",)?;
        }

        Ok(())
    }
}

/// Returns the first matching device
pub fn find_headphone(
    models: &Vec<HeadphoneModel>,
    api: &mut HidApi,
) -> anyhow::Result<Option<Headphone>> {
    info!("Searching for connected headphones...");

    // Filter by SteelSeries HID vendor ID, no filter (0) for product ID
    //
    // https://devicehunt.com/search/type/usb/vendor/1038/device/any
    api.add_devices(0x1038, 0)
        .context("Failed to scan devices")?;

    for device in api.device_list() {
        let product_id = device.product_id();
        let interface_number = device.interface_number();
        let usage_id = device.usage();
        let usage_page = device.usage_page();

        // first, try using usage_id and usage_page
        for model in models.iter() {
            if let Some((model_usage_page, model_usage_id)) = model.usage_page_and_id {
                if product_id == model.product_id
                    && usage_id == model_usage_id
                    && usage_page == model_usage_page
                {
                    debug!("Connecting to device with usage id {model_usage_id:x}, page {model_usage_page:x}");
                    match connect_device(&api, &model, device) {
                        Some(headphone) => return Ok(Some(headphone)),
                        None => continue,
                    }
                }
            }
        }

        // then, try to connect with p_id and interface number
        for model in models.iter() {
            let same_interface_num = match model.interface_num {
                Some(n) => n == interface_number,
                None => true,
            };

            if model.product_id == product_id && same_interface_num {
                debug!(
                    "Connecting to device at inteface {}", 
                    model.interface_num.unwrap_or(0)
                );
                match connect_device(&api, model, device) {
                    Some(headphone) => return Ok(Some(headphone)),
                    None => continue,
                }
            }
        }
    }

    warn!("Found no connected headphones!");

    Ok(None)
}

fn connect_device(api: &HidApi, model: &HeadphoneModel, device: &DeviceInfo) -> Option<Headphone> {
    let path = device.path();
    debug!("Connecting to device at {}", path.to_string_lossy());

    let device = match device.open_device(api) {
        Ok(d) => d,
        Err(err) => {
            error!("Failed to open device: {err:?}");
            return None;
        }
    };

    let device_name = model.name.to_owned();

    info!("Found headphone: {device_name}");
    debug!("Model: {model:?}");

    Some(Headphone {
        device,
        model: *model,
        path: Rc::from(path),
        name: device_name,
        battery_state: 0,
        charging_state: None,
    })
}

#[derive(Copy, Clone)]
pub struct HeadphoneModel {
    pub name: &'static str,
    pub product_id: u16,
    /// Magic bytes that make the device output battery information
    pub write_bytes: [u8; 2],
    pub interface_num: Option<i32>,
    pub battery_percent_idx: usize,
    pub charging_status_idx: Option<usize>,
    pub connected_status_idx: Option<usize>,
    /// Usage page first, then id
    pub usage_page_and_id: Option<(u16, u16)>,
    /// Size of buffer to send when reading battery status
    pub read_buf_size: usize,
    pub battery_range: (u8, u8),
}

impl std::fmt::Debug for HeadphoneModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeadphoneModel")
            .field("product_id", &format!("0x{:x}", self.product_id))
            .finish_non_exhaustive()
    }
}
