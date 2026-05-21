use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

const CODEX_AGENT_COMMAND: &str = "codex";

pub fn run_yzx_agent(args: &[String]) -> Result<i32, CoreError> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h" | "help") {
        print_agent_help();
        return Ok(0);
    }

    if !args.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "unexpected_agent_argument",
            format!("Unexpected argument for yzx agent: {}", args[0]),
            "Run `yzx agent` without arguments for the hardcoded Codex validation pane.",
            json!({ "argument": args[0] }),
        ));
    }

    let path = std::env::var_os("PATH").unwrap_or_default();
    if resolve_command_on_path(CODEX_AGENT_COMMAND, &path).is_none() {
        return Err(missing_codex_error(&path));
    }

    let status = Command::new(CODEX_AGENT_COMMAND).status().map_err(|source| {
        CoreError::io(
            "codex_agent",
            "Failed to launch the Codex agent command.",
            "Install Codex on the host, make sure `codex` is executable on PATH, then restart Yazelix.",
            CODEX_AGENT_COMMAND,
            source,
        )
    })?;

    Ok(status.code().unwrap_or(1))
}

fn print_agent_help() {
    println!("Open the hardcoded Codex agent command");
    println!();
    println!("Usage:");
    println!("  yzx agent");
    println!();
    println!("Codex is host-installed by default; Yazelix does not bundle it.");
}

fn missing_codex_error(path: &OsStr) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_codex_agent",
        "Codex is the default Yazelix agent, but `codex` is not on PATH.",
        "Install Codex on the host, or add it through Home Manager, then restart Yazelix. This validation slice is hardcoded to Codex and does not fall back to another agent.",
        json!({
            "command": CODEX_AGENT_COMMAND,
            "path": path.to_string_lossy(),
        }),
    )
}

fn resolve_command_on_path(command: &str, path: &OsStr) -> Option<PathBuf> {
    if command.contains('/') {
        let candidate = PathBuf::from(command);
        return is_executable_file(&candidate).then_some(candidate);
    }

    std::env::split_paths(path)
        .map(|entry| entry.join(command))
        .find(|candidate| is_executable_file(candidate))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && path
            .metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::resolve_command_on_path;
    use std::fs;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    // Defends: yzx agent only launches a real executable from PATH, so a missing host Codex gets the Yazelix error instead of a shell fallback.
    #[test]
    fn resolves_only_executable_commands_on_path() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let codex = bin.join("codex");
        fs::write(&codex, "#!/bin/sh\nexit 0\n").unwrap();

        #[cfg(unix)]
        {
            fs::set_permissions(&codex, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert_eq!(
            resolve_command_on_path("codex", bin.as_os_str()),
            Some(codex)
        );
        assert_eq!(resolve_command_on_path("opencode", bin.as_os_str()), None);
    }
}
