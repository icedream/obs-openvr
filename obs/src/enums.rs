use std::{
    str::FromStr,
};

pub trait ObsEnum: FromStr {
    fn as_str(&self) -> &'static str;
}
