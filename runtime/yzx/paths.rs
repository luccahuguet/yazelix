use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    PATH_PREFIX,
    error::{AppError, startup},
};

pub(crate) fn config_home() -> Result<PathBuf, AppError> {
    if let Some(path) = nonempty_env("YAZELIX_CONFIG_HOME") {
        return Ok(path.into());
    }
    if let Some(path) = nonempty_env("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("yazelix"));
    }
    nonempty_env("HOME")
        .map(|path| PathBuf::from(path).join(".config/yazelix"))
        .ok_or_else(|| {
            startup(
                "HOME is required when YAZELIX_CONFIG_HOME and XDG_CONFIG_HOME are unset.",
                "",
                1,
            )
        })
}

pub(crate) fn home_dir() -> Result<PathBuf, AppError> {
    nonempty_env("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| startup("HOME is required to scope home-marker new tabs.", "", 1))
}

pub(crate) fn state_dir() -> PathBuf {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/yazelix"))
}

pub(crate) fn enter_terminal_label() -> OsString {
    nonempty_env("YAZELIX_SESSION_TERMINAL")
        .or_else(|| nonempty_env("TERM_PROGRAM"))
        .or_else(|| nonempty_env("TERM"))
        .unwrap_or_else(|| OsString::from("unknown"))
}

pub(crate) fn runtime_path() -> OsString {
    match nonempty_env("PATH") {
        Some(path) => {
            let mut merged = OsString::from(PATH_PREFIX);
            merged.push(":");
            merged.push(path);
            merged
        }
        None => PATH_PREFIX.into(),
    }
}

pub(crate) fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}

pub(crate) fn parent(path: &Path) -> &Path {
    path.parent().unwrap_or_else(|| Path::new("."))
}

pub(crate) fn zellij_session_label(inside: &'static str, outside: &'static str) -> &'static str {
    if nonempty_env("ZELLIJ_SESSION_NAME").is_some() {
        inside
    } else {
        outside
    }
}
