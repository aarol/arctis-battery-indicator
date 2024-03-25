extern crate winres;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("src/bat/battery50.ico", "1");

    // register dark mode icons (10 - 50)
    (10..=50)
        .step_by(10)
        // also register light mode icons (11 - 51)
        .chain((11..=51).step_by(10))
        .for_each(|d| {
            res.set_icon_with_id(&format!("src/bat/battery{d}.ico"), &d.to_string());
        });

    res.compile().unwrap();
}
