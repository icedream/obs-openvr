extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let _openvr = pkg_config::probe_library("openvr")
        .expect("Error finding libobs with pkg-config");

    let bindings = bindgen::builder()
        .header("wrapper.h")
        .rustified_enum("*")
        .generate()
        .expect("Error generating libobs bindings");

    let out_path: PathBuf = env::var("OUT_DIR").unwrap().into();

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Error writing bindings to file");
}
