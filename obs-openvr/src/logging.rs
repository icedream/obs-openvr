use std::{
    borrow::Cow,
    env,
    ffi::{
        OsStr,
    },
    fmt::Display,
    sync::Once,
};

fn env_or_default<'v, K: AsRef<OsStr>>(k: K, v: &'v OsStr) -> Cow<'v, OsStr> {
    let k = k.as_ref();
    env::var_os(k)
        .map(Cow::Owned)
        .unwrap_or_else(move || {
            env::set_var(k, v);
            Cow::Borrowed(v)
        })
}

fn non_blank_env<K: AsRef<OsStr>>(k: K) -> Option<String> {
    env::var(k)
        .ok()
        .into_iter()
        .filter(|v| v.len() > 0)
        .next()
}

static INIT_LOGGING: Once = Once::new();
const DEFAULT_LOG_LEVEL: &'static str = "info";
const DEBUG_CRATES: [&'static str; 2] = ["obs", "openvr"];

fn append_log_level<CrateName: ?Sized, Level: ?Sized>(crate_name: &CrateName, level: &Level) where
    CrateName: Display,
    Level: Display,
{
    let new_value = match non_blank_env("RUST_LOG") {
        Some(previous) => format!("{},{}={}", previous, crate_name, level),
        None => format!("{}={}", crate_name, level),
    };
    env::set_var("RUST_LOG", &new_value);
}

fn list_has_crate<V: AsRef<str>, Crate: AsRef<str>>(v: V, crate_name: Crate) -> bool {
    let prefix = format!("{}=", crate_name.as_ref());
    let v = v.as_ref();
    v.split(",")
        .find(|s| s.starts_with(&prefix))
        .is_some()
}

#[inline]
fn has_crate<S: AsRef<str>>(crate_name: S) -> bool {
    env::var("RUST_LOG")
        .ok()
        .map(|v| list_has_crate(v, crate_name))
        .unwrap_or(false)
}

pub fn init() {
    INIT_LOGGING.call_once(|| {
        let level = env_or_default("OBS_OPENVR_LOG", DEFAULT_LOG_LEVEL.as_ref());
        let level = level.to_str().unwrap();
        if !has_crate(env!("CARGO_CRATE_NAME")) {
            append_log_level(env!("CARGO_CRATE_NAME"), level);
            if level == "trace" || level == "debug" {
                DEBUG_CRATES.iter().for_each(|debug_crate| {
                    append_log_level(debug_crate, level);
                });
            }
        }
        env::var_os("RUST_LOG").into_iter().for_each(|v| {
            println!("obs_openvr: RUST_LOG={:?}", &v);
        });
        env_logger::init();
    });
}
