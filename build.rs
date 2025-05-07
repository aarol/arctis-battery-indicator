extern crate winres;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("src/bat/main.ico");

    // register dark mode icons (10,20,...,50)
    (10..=50)
    .step_by(10)
        // also register light mode icons (11,21,...,51)
        .chain((11..=51).step_by(10))
        .for_each(|d| {
            res.set_icon_with_id(&format!("src/bat/battery{d}.ico"), &format!("{d}"));
        });

    res.compile().unwrap();
}
