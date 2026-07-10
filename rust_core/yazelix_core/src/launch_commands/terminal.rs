use crate::bridge::{CoreError, ErrorClass};
use crate::runtime_contract::TerminalCandidate;
use crate::terminal_variant::terminal_window_title;
use crate::user_config_paths;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(super) fn resolve_mars_config_path(
    config_dir: &Path,
    runtime_dir: &Path,
) -> Result<PathBuf, String> {
    let user = user_config_paths::mars_config(config_dir);
    match fs::symlink_metadata(&user) {
        Ok(_) if user.is_file() => return validate_mars_config(user),
        Ok(_) => {
            return Err(format!(
                "Yazelix Mars config is not a readable file: {}",
                user.display()
            ));
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!(
                "Could not inspect Yazelix Mars config {}: {error}",
                user.display()
            ));
        }
    }

    let packaged = user_config_paths::packaged_mars_config(runtime_dir);
    if packaged.is_file() {
        validate_mars_config(packaged)
    } else {
        Err(format!(
            "Packaged Mars config is missing: {}",
            packaged.display()
        ))
    }
}

fn validate_mars_config(path: PathBuf) -> Result<PathBuf, String> {
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read Mars config {}: {error}", path.display()))?;
    toml::from_str::<toml::Table>(&raw)
        .map_err(|error| format!("Mars config {} is invalid TOML: {error}", path.display()))?;
    Ok(path)
}

pub(super) fn current_platform_name() -> String {
    std::env::var("YAZELIX_TEST_OS")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::env::consts::OS.to_string())
}

pub(super) fn build_launch_command_argv(
    runtime_dir: &Path,
    terminal: &TerminalCandidate,
    working_dir: &Path,
    session_name: Option<&str>,
) -> Result<Vec<String>, CoreError> {
    let startup_script = runtime_dir
        .join("shells")
        .join("posix")
        .join("start_yazelix.sh");
    if !startup_script.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_startup_script",
            format!(
                "Missing Yazelix startup script at {}.",
                startup_script.display()
            ),
            "Restore shells/posix/start_yazelix.sh or reinstall Yazelix.",
            serde_json::json!({}),
        ));
    }

    Ok(vec![
        terminal.command.clone(),
        "--title-placeholder".to_string(),
        terminal_window_title(&terminal.terminal, session_name),
        "--working-dir".to_string(),
        working_dir.to_string_lossy().into_owned(),
        "-e".to_string(),
        startup_script.to_string_lossy().into_owned(),
    ])
}

#[cfg(test)]
mod tests {
    // Test lane: default

    use super::*;
    use tempfile::tempdir;

    // Defends: a canonical complete Mars config wins over the packaged complete config without merging either file.
    #[test]
    fn canonical_mars_config_wins_without_materialization() {
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join("config/yazelix");
        let runtime_dir = temp.path().join("runtime");
        let user = user_config_paths::mars_config(&config_dir);
        let packaged = user_config_paths::packaged_mars_config(&runtime_dir);
        fs::create_dir_all(user.parent().unwrap()).unwrap();
        fs::create_dir_all(packaged.parent().unwrap()).unwrap();
        fs::write(&user, "[window]\nopacity = 0.5\n").unwrap();
        fs::write(&packaged, "[window]\nopacity = 1.0\n").unwrap();

        assert_eq!(
            resolve_mars_config_path(&config_dir, &runtime_dir).unwrap(),
            user
        );
    }

    // Defends: absence of a user file selects the packaged complete Mars config directly.
    #[test]
    fn packaged_mars_config_is_the_absent_user_default() {
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join("config/yazelix");
        let runtime_dir = temp.path().join("runtime");
        let packaged = user_config_paths::packaged_mars_config(&runtime_dir);
        fs::create_dir_all(packaged.parent().unwrap()).unwrap();
        fs::write(&packaged, "[window]\nopacity = 1.0\n").unwrap();

        assert_eq!(
            resolve_mars_config_path(&config_dir, &runtime_dir).unwrap(),
            packaged
        );
    }

    // Regression: invalid complete config fails before Mars can replace it with internal defaults.
    #[test]
    fn invalid_user_mars_config_fails_fast() {
        let temp = tempdir().unwrap();
        let config_dir = temp.path().join("config/yazelix");
        let runtime_dir = temp.path().join("runtime");
        let user = user_config_paths::mars_config(&config_dir);
        fs::create_dir_all(user.parent().unwrap()).unwrap();
        fs::write(&user, "[window\n").unwrap();

        let error = resolve_mars_config_path(&config_dir, &runtime_dir).unwrap_err();

        assert!(error.contains("invalid TOML"));
        assert!(error.contains("config/yazelix/mars/config.toml"));
    }
}
