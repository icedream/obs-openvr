extern crate cc;
extern crate pkg_config;

use std::{
    collections::LinkedList,
    env,
    iter,
    path::{
        PathBuf,
    },
};

struct DependencyConfig {
    cflags: LinkedList<String>,
}

impl DependencyConfig {
    pub fn new() -> Result<Self, pkg_config::Error> {
        let mut cflags: LinkedList<String> = LinkedList::new();
        let openvr_cflags = pkg_config::get_variable("openvr", "Cflags")?;
        openvr_cflags.split(" ")
            .filter(|flag| flag.len() > 0)
            .map(String::from)
            .for_each(|flag| cflags.extend(iter::once(flag)));
        Ok(DependencyConfig {
            cflags: cflags,
        })
    }

    pub fn cflags(&self) -> String {
        self.cflags.iter().fold(String::from(""), |all_flags, s| format!("{} {}", all_flags, s))
    }
}

const OPENVR_UTILS_LIBRARY_NAME: &'static str = "libopenvr-utils.a";

fn main() {
    let dep_config = DependencyConfig::new()
        .expect("Failed to find dependency information");
    let mut vr_utils_build = cc::Build::new();
    vr_utils_build
        .cpp(true)
        .include("src");
    dep_config.cflags.iter().fold(&mut vr_utils_build, |build, flag| build.flag(flag));
    vr_utils_build
        .flag("-std=c++2a")
        .file("src/openvr-utils.cpp")
        .compile(OPENVR_UTILS_LIBRARY_NAME);
    // let library_path = {
    //     let mut p = PathBuf::from(env::var("OUT_DIR").unwrap());
    //     p.push("libopenvr-utils.a");
    //     p
    // };
    // println!("cargo:rustc-link-search={}={}", "native", env::var("OUT_DIR").unwrap());
    // println!("cargo:rustc-link-lib=static={}", OPENVR_UTILS_LIBRARY_NAME);
}
