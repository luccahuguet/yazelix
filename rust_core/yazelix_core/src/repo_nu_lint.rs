use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_repo_nu_lint(repo_root: &Path, format: &str, paths: &[String]) -> Result<(), String> {
    let config_path = repo_root.join(".nu-lint.toml");
    if !config_path.is_file() {
        return Err(format!(
            ".nu-lint.toml not found at {}",
            config_path.display()
        ));
    }

    if !command_exists("nu-lint") {
        return Err(
            "nu-lint not found in PATH.\nInstall nu-lint in your maintainer environment, then rerun this command."
                .to_string(),
        );
    }

    let targets = if paths.is_empty() {
        vec![repo_root.join("nushell").display().to_string()]
    } else {
        paths.to_vec()
    };

    let status = Command::new("nu-lint")
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg(format)
        .args(&targets)
        .current_dir(repo_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("Failed to launch nu-lint: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "nu-lint failed with exit code {}",
            status.code().unwrap_or(1)
        ))
    }
}

fn command_exists(name: &str) -> bool {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
