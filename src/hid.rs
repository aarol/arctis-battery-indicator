use hidapi::HidDevice;
use log::{debug, error, info, warn};

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

    pub fn charging_status(&self) -> Option<&str> {
        self.charging_state.map(|state| match state {
            1 => "Charging",
            3 => {
                if self.battery_state < 2 {
                    "Charge soon!"
                } else {
                    "Connected"
                }
            }
            _ => "Disconnected",
        })
    }

    /// if return is Ok(true), state has changed
    pub fn update(&mut self) -> hidapi::HidResult<bool> {
        self.device.write(&self.model.write_bytes)?;

        let mut buf = [0u8; 4];

        // timeout because some devices will block here indefinitely
        self.device.read_timeout(&mut buf, 100)?;

        // save old state
        let Headphone {
            battery_state: old_battery,
            charging_state: old_charging,
            ..
        } = *self;

        // Why do the values overflow on first connection?
        // Overflow values seen: [46, 48]
        if buf[self.model.battery_percent_idx] <= 4 {
            self.battery_state = buf[self.model.battery_percent_idx];
        } else {
            debug!(
                "Returned battery state overflows; ignoring: {}",
                buf[self.model.battery_percent_idx]
            );
        }

        if let Some(idx) = self.model.charging_status_idx {
            if buf[idx] < 4 {
                self.charging_state = Some(buf[idx]);
            } else {
                debug!("Returned charge state overflows; ignoring: {}", buf[idx])
            }
        }

        Ok(self.battery_state != old_battery || self.charging_state != old_charging)
    }
}

impl std::fmt::Display for Headphone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}: {}%",
            self.name,
            self.battery_percentage()
        ))?;

        if let Some(status) = self.charging_status() {
            f.write_fmt(format_args!(" - {status}"))?;
        }

        Ok(())
    }
}

/// Returns the first matching device
pub fn find_headphone() -> Option<Headphone> {
    info!("Searching for connected headphones...");

    let mut api = hidapi::HidApi::new_without_enumerate().expect("Failed to initialize hidapi");

    // SteelSeries HID vendor ID
    // https://devicehunt.com/search/type/usb/vendor/1038/device/any
    api.add_devices(0x1038, 0).expect("Failed to scan devices");

    for device in api.device_list() {
        for model in KNOWN_HEADPHONES {
            if device.product_id() == model.product_id
                && device.interface_number() == model.interface_num
            {
                let device = match device.open_device(&api) {
                    Ok(d) => d,
                    Err(err) => {
                        error!("Failed to open device: {err:?}");
                        continue;
                    }
                };

                let device_name = device.get_product_string().unwrap_or(None);

                info!("Found headphone: {}", model.name);

                return Some(Headphone {
                    device,
                    model: *model,
                    name: device_name.unwrap_or(model.name.to_owned()),
                    battery_state: 0,
                    charging_state: None,
                });
            }
        }
    }

    warn!("Found no connected headphones!");

    None
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
}

// found in https://github.com/richrace/arctis-usb-finder/blob/745a4f68b8394487ae549ef0eebf637ef6e26dd3/src/models/known_headphone.ts
const KNOWN_HEADPHONES: &[HeadphoneModel] = &[
    HeadphoneModel {
        name: "Arctis Pro Wireless",
        product_id: 0x1290,
        write_bytes: [0x40, 0xaa],
        interface_num: 0,
        battery_percent_idx: 0,
        charging_status_idx: None,
    },
    HeadphoneModel {
        name: "Arctis 7 2017",
        product_id: 0x1260,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
    },
    HeadphoneModel {
        name: "Arctis 7 2019",
        product_id: 0x12ad,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
    },
    HeadphoneModel {
        name: "Arctis Pro 2019",
        product_id: 0x1252,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
    },
    HeadphoneModel {
        name: "Arctis Pro GameDac",
        product_id: 0x1280,
        write_bytes: [0x06, 0x18],
        interface_num: 5,
        battery_percent_idx: 2,
        charging_status_idx: None,
    },
    HeadphoneModel {
        name: "Arctis 9",
        product_id: 0x12c2,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
    },
    HeadphoneModel {
        name: "Arctis 1 Wireless",
        product_id: 0x12b3,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
    },
    HeadphoneModel {
        name: "Arctis 1 Xbox",
        product_id: 0x12b6,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
    },
    HeadphoneModel {
        name: "Arctis 7X",
        product_id: 0x12d7,
        write_bytes: [0x06, 0x12],
        interface_num: 3,
        battery_percent_idx: 3,
        charging_status_idx: Some(4),
    },
    // 7 Plus
    HeadphoneModel {
        name: "Arctis 7 Plus",
        product_id: 0x220e,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
    },
    HeadphoneModel {
        name: "Arctis 7P Plus",
        product_id: 0x2212,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
    },
    HeadphoneModel {
        name: "Arctis 7X Plus",
        product_id: 0x2216,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
    },
    HeadphoneModel {
        name: "Arctis 7 Destiny Plus",
        product_id: 0x2236,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
    },
    // Nova
    HeadphoneModel {
        name: "Arctis Nova 7",
        product_id: 0x2202,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
    },
    HeadphoneModel {
        name: "Arctis Nova 7X",
        product_id: 0x2206,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
    },
    HeadphoneModel {
        name: "Arctis Nova 7P",
        product_id: 0x220a,
        write_bytes: [0x00, 0xb0],
        interface_num: 3,
        battery_percent_idx: 2,
        charging_status_idx: Some(3),
    },
];

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::KNOWN_HEADPHONES;

    #[test]
    fn unique_product_ids() {
        let mut seen: HashMap<u16, bool> = HashMap::new();
        KNOWN_HEADPHONES.iter().for_each(|h| {
            assert!(
                !seen.contains_key(&h.product_id),
                "duplicate entries for {:#x}",
                &h.product_id
            );
            seen.insert(h.product_id, true);
        });
    }
}
