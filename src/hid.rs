use anyhow::Context;
use hidapi::{DeviceInfo, HidApi, HidDevice};
use log::{debug, error, info, trace, warn};
use rust_i18n::t;

#[derive(Debug)]
pub struct Headphone {
    device: HidDevice,
    model: HeadphoneModel,
    name: String,
    /// percentage in range [0,4]
    pub battery_state: u8,
    /// - 0: not connected
    /// - 1: charging
    /// - 3: discharging
    charging_state: Option<u8>,
}

impl Headphone {
    pub fn battery_percentage(&self) -> i32 {
        ((self.battery_state as f32 / 4.0) * 100.0) as i32
    }

    pub fn charging_status(&self) -> Option<String> {
        self.charging_state.map(|state| match state {
            1 => t!("device_charging").into(),
            3 => "".into(),
            _ => t!("device_disconnected").into(),
        })
    }

    /// if return is Ok(true), state has changed
    pub fn update(&mut self) -> hidapi::HidResult<bool> {
        self.device.write(&self.model.write_bytes)?;
        let mut buf = [0u8; 64];

        // timeout because we don't want to block indefinitely here
        let n = self.device.read_timeout(&mut buf, 100)?;

        trace!("read {n}: {:?}", &buf[0..5]);

        if n == 0 || buf[0] == 0 || !self.model.write_bytes.contains(&buf[0]) {
            trace!("Read invalid bytes from device: {:?}; ignoring", &buf[0..5]);
            return Ok(false);
        }

        // save old state
        let Headphone {
            battery_state: old_battery,
            charging_state: old_charging,
            ..
        } = *self;

        let battery_state = buf[self.model.battery_percent_idx];

        if battery_state <= 4 {
            self.battery_state = battery_state;
        } else {
            debug!(
                "Returned battery state overflows: {}; ignoring",
                battery_state
            );
        }

        if let Some(idx) = self.model.charging_status_idx {
            let charging_state = buf[idx];

            if charging_state < 4 {
                self.charging_state = Some(charging_state);
            } else {
                debug!("Returned charge state overflows: {}; ignoring", buf[idx])
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

        if let Some(status) = self.charging_status() {
            write!(f, " {status}",)?;
        }

        Ok(())
    }
}

/// Returns the first matching device
pub fn find_headphone() -> anyhow::Result<Option<Headphone>> {
    info!("Searching for connected headphones...");

    let mut api = hidapi::HidApi::new_without_enumerate().context("Failed to initialize hidapi")?;

    // Filter by SteelSeries HID vendor ID, no filter (0) for product ID
    //
    // https://devicehunt.com/search/type/usb/vendor/1038/device/any
    api.add_devices(0x1038, 0)
        .context("Failed to scan devices")?;

    for device in api.device_list() {
        // first, try using usage_id and usage_page
        let usage_id = device.usage();
        let usage_page = device.usage_page();

        for model in KNOWN_HEADPHONES {
            if let Some((model_usage_id, model_usage_page)) = model.usage_page {
                if model_usage_id == usage_id && model_usage_page == usage_page {
                    debug!("Connecting to device with usage id {model_usage_id:x}, page {model_usage_page:x}");
                    match connect_device(&api, model, device) {
                        Some(headphone) => return Ok(Some(headphone)),
                        None => continue,
                    }
                }
            }
        }

        // then, try to connect with p_id and interface number
        let product_id = device.product_id();
        let interface_number = device.interface_number();

        for model in KNOWN_HEADPHONES {
            if model.product_id == product_id && model.interface_num == interface_number {
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
    let device = match device.open_device(api) {
        Ok(d) => d,
        Err(err) => {
            error!("Failed to open device: {err:?}");
            return None;
        }
    };
    let device_name = device
        .get_product_string()
        .ok()
        .flatten()
        .unwrap_or_else(|| model.name.to_owned());

    info!("Found headphone: {device_name}");

    Some(Headphone {
        device,
        model: *model,
        name: device_name,
        battery_state: 0,
        charging_state: None,
    })
}

#[derive(Copy, Clone)]
struct HeadphoneModel {
    name: &'static str,
    product_id: u16,
    /// Magic bytes that make the device output battery information
    write_bytes: [u8; 2],
    interface_num: i32,
    battery_percent_idx: usize,
    charging_status_idx: Option<usize>,
    usage_page: Option<(u16, u16)>,
}

impl std::fmt::Debug for HeadphoneModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HeadphoneModel")
            .field("product_id", &format!("0x{:x}", self.product_id))
            .finish_non_exhaustive()
    }
}

// found in https://github.com/richrace/arctis-usb-finder/blob/745a4f68b8394487ae549ef0eebf637ef6e26dd3/src/models/known_headphone.ts
// & https://github.com/Sapd/HeadsetControl/blob/master/src/devices
const KNOWN_HEADPHONES: &[HeadphoneModel] = &[
    HeadphoneModel {
        name: "Arctis Pro Wireless",
        product_id: 0x1290,
        write_bytes: [0x40, 0xaa],
        interface_num: 0,
        battery_percent_idx: 0,
        charging_status_idx: None,
        usage_page: None,
    },
    HeadphoneModel {
        name: "Arctis 7 2017",
        product_id: 0x1260,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
        usage_page: None,
    },
    HeadphoneModel {
        name: "Arctis 7 2019",
        product_id: 0x12ad,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
        usage_page: None,
    },
    HeadphoneModel {
        name: "Arctis Pro 2019",
        product_id: 0x1252,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
        usage_page: None,
    },
    HeadphoneModel {
        name: "Arctis Pro GameDac",
        product_id: 0x1280,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
        usage_page: None,
    },
    HeadphoneModel {
        name: "Arctis 9",
        product_id: 0x12c2,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
        usage_page: None,
    },
    HeadphoneModel {
        name: "Arctis 1 Wireless",
        product_id: 0x12b3,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
        usage_page: Some((0xff43, 0x202)),
    },
    HeadphoneModel {
        name: "Arctis 1 Xbox",
        product_id: 0x12b6,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
        usage_page: Some((0xff43, 0x202)),
    },
    HeadphoneModel {
        name: "Arctis 7X",
        product_id: 0x12d7,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
        usage_page: Some((0xff43, 0x202)),
    },
    HeadphoneModel {
        name: "Arctis 7 Plus",
        product_id: 0x220e,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    HeadphoneModel {
        name: "Arctis 7P Plus",
        product_id: 0x2212,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    HeadphoneModel {
        name: "Arctis 7X Plus",
        product_id: 0x2216,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    HeadphoneModel {
        name: "Arctis 7 Destiny Plus",
        product_id: 0x2236,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    // Nova
    HeadphoneModel {
        name: "Arctis Nova 7",
        product_id: 0x2202,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    HeadphoneModel {
        name: "Arctis Nova 7X",
        product_id: 0x2206,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    HeadphoneModel {
        name: "Arctis Nova 7X v2",
        product_id: 0x2258,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    HeadphoneModel {
        name: "Arctis Nova 7P",
        product_id: 0x220a,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
    HeadphoneModel {
        name: "Arctis Nova 7 Diablo IV",
        product_id: 0x223a,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
        usage_page: Some((0xffc0, 0x1)),
    },
];

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::KNOWN_HEADPHONES;

    #[test]
    fn unique_product_ids() {
        let mut seen_product_ids: HashMap<u16, bool> = HashMap::new();
        let mut seen_names: HashMap<&str, bool> = HashMap::new();
        KNOWN_HEADPHONES.iter().for_each(|h| {
            assert!(
                seen_product_ids.insert(h.product_id, true).is_none(),
                "duplicate entries for {:#x}",
                &h.product_id
            );
            assert!(
                seen_names.insert(h.name, true).is_none(),
                "duplicate entries for {}",
                &h.name
            );
        });
    }
}
