extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    let _obs = pkg_config::probe_library("libobs")
        .expect("Error finding libobs with pkg-config");

    let bindings = bindgen::builder()
        .header("wrapper.h")
        .blacklist_type("_bindgen_ty_2")
        .blacklist_type("_bindgen_ty_3")
        .blacklist_type("_bindgen_ty_4")
        .rustified_enum("*")
        .blacklist_item("true_")
        .blacklist_item("false_")
        .blacklist_item(".+_defined")
        .blacklist_item("__have_.+")
        .blacklist_item("__glibc_c[0-9]+_.+")
        .blacklist_item("math_errhandling")
        .generate()
        .expect("Error generating libobs bindings");

    let out_path: PathBuf = env::var("OUT_DIR").unwrap().into();

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Error writing bindings to file");
}
