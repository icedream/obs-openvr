extern crate cc;
extern crate build_profile;
extern crate pkg_config;
extern crate thiserror;

use build_profile::{
    BuildProfile,
    has_feature,
};

use std::{
    error::Error,
    env,
    iter,
    path::{
        Path,
        PathBuf,
    },
    ffi::OsStr,
    str::FromStr,
};

#[derive(Debug, thiserror::Error)]
#[error("Unknown variant for {container_name}: {variant_name}")]
pub struct UnknownVariantError {
    container_name: &'static str,
    variant_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetFamily {
    Unix,
    Windows,
    Wasm,
}

impl TargetFamily {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let s = env::var("CARGO_CFG_TARGET_FAMILY")?;
        s.parse()
            .map_err(|e| Box::new(e) as _)
    }
}

impl FromStr for TargetFamily {
    type Err = UnknownVariantError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "unix" => Ok(TargetFamily::Unix),
            "windows" => Ok(TargetFamily::Windows),
            "wasm" => Ok(TargetFamily::Wasm),
            _ => Err(UnknownVariantError {
                container_name: "TargetFamily",
                variant_name: s.to_owned(),
            }),
        }
    }
}

const LIBRARY_NAME: &'static str = "libobs-openvr-utils.a";

fn add_link_library(name: &str, kind: Option<&str>) {
    if let Some(kind) = kind {
        println!("cargo:rustc-link-lib={}={}", kind, name);
    } else {
        println!("cargo:rustc-link-lib={}", name);
    }
}

// fn add_sources<'a, It, HIt>(build: &mut cc::Build, sources: It, headers: HIt) where
//     It: Iterator,
//     HIt: Iterator,
// {
//     sources.for_each(|source| {
//         build.file(source);
//         println!("cargo:rerun-if-changed={}", source);
//     });
//     headers.for_each(|header| {
//         println!("cargo:rerun-if-changed={}", header);
//     });
// }

fn glad_sources<P: AsRef<Path>>(tf: TargetFamily, source_dir: P) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let source_dir = source_dir.as_ref();
    let mut sources = vec!["src/glad.c"];
    let mut headers = vec!["include/glad/glad.h"];
    match tf {
        TargetFamily::Unix => {
            sources.push("src/glad_egl.c");
            headers.extend(["include/EGL/eglplatform.h", "include/glad/glad_egl.h"]);
        },
        _ => {
            unimplemented!()
        },
    }
    let prepend_path = |relative: Vec<&'static str>| -> Vec<PathBuf> {
        relative.into_iter()
            .map(|f| source_dir.join(f))
            .collect()
    };
    (prepend_path(sources), prepend_path(headers))
}

fn main() {
    let profile = BuildProfile::current_or_default();
    let target_family = TargetFamily::from_env()
        .expect("failed to get target family");
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
            .flag("-fPIC")
            .flag("-lEGL")
            .include("src")
            .file("src/mirror-utils.c");
        let glad_source_dir = env::var("OBS_SOURCE_DIR")
            .map(PathBuf::from)
            .map(|mut p| {
                p.extend(["deps", "glad"]);
                p
            })
            .expect("couldn't find OBS_SOURCE_DIR");
        let (g_sources, g_headers) = glad_sources(target_family, &glad_source_dir);
        mirror_utils_build
            .include(glad_source_dir.join("include"))
            .files(g_sources.into_iter().chain(g_headers.into_iter()))
            .compile("libobs-openvr-mirror-utils.a");

        // add_link_library("obsglad", Some("dylib"));
    }
}
