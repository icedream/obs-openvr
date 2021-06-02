extern crate cc;
extern crate build_profile;
extern crate pkg_config;

use build_profile::{
    BuildProfile,
    has_feature,
};

use std::{
    iter,
};

const LIBRARY_NAME: &'static str = "libobs-openvr-utils.a";

fn add_link_library(name: &str, kind: Option<&str>) {
    if let Some(kind) = kind {
        println!("cargo:rustc-link-lib={}={}", kind, name);
    } else {
        println!("cargo:rustc-link-lib={}", name);
    }
}

fn add_sources<'a, It, HIt>(build: &mut cc::Build, sources: It, headers: HIt) where
    It: Iterator<Item=&'a str>,
    HIt: Iterator<Item=&'a str>,
{
    sources.for_each(|source| {
        build.file(source);
        println!("cargo:rerun-if-changed={}", source);
    });
    headers.for_each(|header| {
        println!("cargo:rerun-if-changed={}", header);
    });
}

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

        let mut mirror_utils_build = cc::Build::new();
        if profile.is_debug() {
            mirror_utils_build.define("DEBUG", Some("1"));
        }
        mirror_utils_build
            .flag("-std=c11")
            .flag("-Wno-unused-parameter")
            .include("src");
        add_sources(&mut mirror_utils_build, iter::once("src/mirror-utils.c"), iter::once("src/mirror-utils.h"));
        mirror_utils_build
            .compile("libobs-openvr-mirror-utils.a");

        add_link_library("obsglad", Some("dylib"));
    }
}
