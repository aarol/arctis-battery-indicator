use crate::hid::HeadphoneModel;

// found in https://github.com/richrace/arctis-usb-finder/blob/745a4f68b8394487ae549ef0eebf637ef6e26dd3/src/models/known_headphone.ts
// & https://github.com/Sapd/HeadsetControl/blob/master/src/devices
pub const KNOWN_HEADPHONES: &[HeadphoneModel] = &[
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