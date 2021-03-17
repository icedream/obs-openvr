extern crate cc;
extern crate build_profile;

use build_profile::{
    BuildProfile,
    has_feature,
};

const LIBRARY_NAME: &'static str = "libobs-openvr-utils.a";

fn main() {
    let profile = BuildProfile::current_or_default();
    if has_feature("mirror-source") {
        let mut vr_utils_build = cc::Build::new();
        if profile.is_debug() {
            vr_utils_build
                .define("DEBUG", Some("1"));
        }
        vr_utils_build
            .flag("-std=c11")
            .flag("-Wno-unused-parameter")
            .include("src")
            .file("src/utils.c")
            .compile(LIBRARY_NAME);
    }
}
