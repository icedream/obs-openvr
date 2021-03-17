extern crate thiserror;

mod option_ext;

use std::{
    env,
    str::FromStr,
};
use thiserror::Error;
use option_ext::OptionExt;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildProfile {
    Release,
    Debug,
}

impl BuildProfile {
    pub fn current() -> Result<Self, BuildProfileParseError> {
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

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum BuildProfileParseError {
    #[error("invalid build profile `{0}`")]
    InvalidBuildProfile(String),
}

impl FromStr for BuildProfile {
    type Err = BuildProfileParseError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        use BuildProfile::*;
        use BuildProfileParseError::*;
        match s {
            "release" => Ok(Release),
            "debug" => Ok(Debug),
            s => Err(InvalidBuildProfile(String::from(s))),
        }
    }
}

pub fn has_feature<S: AsRef<str>>(feature: S) -> bool {
    let feature = feature.as_ref().to_uppercase().replace("-", "_");
    env::var_os(format!("CARGO_FEATURE_{}", &feature)).is_some()
}
