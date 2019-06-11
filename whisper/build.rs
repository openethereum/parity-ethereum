extern crate mml;

fn main() {
    let dest: String = concat!("target/doc/", env!("CARGO_PKG_NAME")).to_string();

    let _ = mml::src2both("src", dest.replace("-", "_").as_str());
}