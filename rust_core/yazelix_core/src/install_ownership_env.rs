//! Shared control-plane environment helpers for install-ownership based commands.

use crate::bridge::CoreError;
use crate::control_plane::{
    config_dir_from_env, expand_user_path, home_dir_from_env, runtime_dir_from_env,
};
use crate::install_ownership_report::InstallOwnershipEvaluateRequest;
use std::path::{Path, PathBuf};
use std::process::Command;

fn xdg_config_home(home: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("XDG_CONFIG_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home);
        }
    }
    home.join(".config")
}

fn xdg_data_home(home: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("XDG_DATA_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home);
        }
    }
    home.join(".local").join("share")
}

fn yazelix_state_dir(home: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("YAZELIX_STATE_DIR") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home);
        }
    }
    if let Ok(raw) = std::env::var("XDG_DATA_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home).join("yazelix");
        }
    }
    home.join(".local").join("share").join("yazelix")
}

fn shell_resolved_yzx_path(home: &Path) -> Option<String> {
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg("command -v yzx")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let resolved = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if resolved.is_empty() {
        None
    } else {
        Some(
            expand_user_path(&resolved, home)
                .to_string_lossy()
                .into_owned(),
        )
    }
}

pub fn install_ownership_request_from_env() -> Result<InstallOwnershipEvaluateRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let home_dir = home_dir_from_env()?;
    let config_root = config_dir_from_env()?;
    let main_config_path = config_root.join("user_configs").join("yazelix.toml");

    Ok(InstallOwnershipEvaluateRequest {
        runtime_dir,
        home_dir: home_dir.clone(),
        user: std::env::var("USER").ok().filter(|s| !s.trim().is_empty()),
        xdg_config_home: xdg_config_home(&home_dir),
        xdg_data_home: xdg_data_home(&home_dir),
        yazelix_state_dir: yazelix_state_dir(&home_dir),
        main_config_path,
        invoked_yzx_path: std::env::var("YAZELIX_INVOKED_YZX_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty()),
        redirected_from_stale_yzx_path: std::env::var("YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty()),
        shell_resolved_yzx_path: shell_resolved_yzx_path(&home_dir),
    })
}
