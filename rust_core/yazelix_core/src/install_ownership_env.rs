//! Shared control-plane environment helpers for install-ownership based commands.

use crate::bridge::CoreError;
use crate::control_plane::{
    config_dir_from_env, expand_user_path, home_dir_from_env, runtime_dir_from_env,
};
use crate::install_ownership_report::InstallOwnershipEvaluateRequest;
use crate::user_config_paths;
use std::path::{Path, PathBuf};
use std::process::Command;

fn env_text(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

fn expanded_path_value(raw: &str, home: &Path) -> PathBuf {
    let expanded = expand_user_path(raw, home);
    if expanded.is_absolute() {
        expanded
    } else {
        std::path::absolute(&expanded).unwrap_or(expanded)
    }
}

fn xdg_config_home(home: &Path) -> PathBuf {
    if let Some(raw) = env_text("XDG_CONFIG_HOME") {
        return expanded_path_value(&raw, home);
    }
    home.join(".config")
}

fn xdg_data_home(home: &Path) -> PathBuf {
    if let Some(raw) = env_text("XDG_DATA_HOME") {
        return expanded_path_value(&raw, home);
    }
    home.join(".local").join("share")
}

fn yazelix_state_dir(home: &Path) -> PathBuf {
    if let Some(raw) = env_text("YAZELIX_STATE_DIR") {
        return expanded_path_value(&raw, home);
    }
    if let Some(raw) = env_text("XDG_DATA_HOME") {
        return expanded_path_value(&raw, home).join("yazelix");
    }
    home.join(".local").join("share").join("yazelix")
}

fn path_to_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().into_owned()
}

fn shell_resolved_yzx_path_for_report(home: &Path) -> Option<String> {
    if let Some(invoked) = env_text("YAZELIX_INVOKED_YZX_PATH") {
        return Some(path_to_string(expanded_path_value(&invoked, home)));
    }

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
        Some(path_to_string(expanded_path_value(&resolved, home)))
    }
}

pub fn install_ownership_request_from_env() -> Result<InstallOwnershipEvaluateRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    install_ownership_request_from_env_with_runtime_dir(runtime_dir)
}

pub fn install_ownership_request_from_env_with_runtime_dir(
    runtime_dir: PathBuf,
) -> Result<InstallOwnershipEvaluateRequest, CoreError> {
    let home_dir = home_dir_from_env()?;
    let config_root = config_dir_from_env()?;
    let main_config_path = user_config_paths::main_config(&config_root);

    Ok(InstallOwnershipEvaluateRequest {
        runtime_dir,
        home_dir: home_dir.clone(),
        user: std::env::var("USER").ok().filter(|s| !s.trim().is_empty()),
        xdg_config_home: xdg_config_home(&home_dir),
        xdg_data_home: xdg_data_home(&home_dir),
        yazelix_state_dir: yazelix_state_dir(&home_dir),
        main_config_path,
        invoked_yzx_path: env_text("YAZELIX_INVOKED_YZX_PATH"),
        redirected_from_stale_yzx_path: env_text("YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH"),
        shell_resolved_yzx_path: shell_resolved_yzx_path_for_report(&home_dir),
    })
}
