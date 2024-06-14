extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let _obs = pkg_config::probe_library("libobs")
        .expect("Error finding libobs with pkg-config");

    let bindings = bindgen::builder()
        .header("wrapper.h")
        .blocklist_type("_bindgen_ty_2")
        .blocklist_type("_bindgen_ty_3")
        .blocklist_type("_bindgen_ty_4")
        .rustified_enum("*")
        .blocklist_item("true_")
        .blocklist_item("false_")
        .blocklist_item(".+_defined")
        .blocklist_item("__have_.+")
        .blocklist_item("__glibc_c[0-9]+_.+")
        .blocklist_item("math_errhandling")
        .generate()
        .expect("Error generating libobs bindings");

    let out_path: PathBuf = env::var("OUT_DIR").unwrap().into();

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Error writing bindings to file");
}
