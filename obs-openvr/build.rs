extern crate cc;

use std::{
    env,
    str::FromStr,
};

const LIBRARY_NAME: &'static str = "libobs-openvr-utils.a";

trait OptionExt<T: Sized> {
    fn or_default(self) -> T;
}

impl<T> OptionExt<T> for Option<T> where
    T: Default,
{
    fn or_default(self) -> T {
        self.unwrap_or_else(Default::default)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildProfile {
    Release,
    Debug,
}

impl BuildProfile {
    pub fn current() -> Result<Self, String> {
        env::var("PROFILE")
            .ok()
            .map(|s| s.parse())
            .unwrap_or(Ok(Default::default()))
    }

    pub fn current_or_default() -> Self {
        Self::current().ok().or_default()
    }

    #[inline(always)]
    pub fn is_debug(self) -> bool {
        self == BuildProfile::Debug
    }

    pub fn as_str(self) -> &'static str {
        use BuildProfile::*;
        match self {
            Release => "release",
            Debug => "debug",
        }
    }
}

impl Default for BuildProfile {
    fn default() -> Self {
        BuildProfile::Debug
    }
}

impl FromStr for BuildProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use BuildProfile::*;
        match s {
            "release" => Ok(Release),
            "debug" => Ok(Debug),
            s => Err(format!("invalid build profile: {}", s)),
        }
    }
}

fn main() {
    let profile = BuildProfile::current_or_default();
    let mut vr_utils_build = cc::Build::new();
    if profile.is_debug() {
        vr_utils_build
            .define("DEBUG", Some("1"));
    }
    vr_utils_build
        .include("src")
        .file("src/utils.c")
        .compile(LIBRARY_NAME);
}
