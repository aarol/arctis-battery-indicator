extern crate winres;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon_with_id("src/bat/battery4.ico", "1");
    res.compile().unwrap();
}
