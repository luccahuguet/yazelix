use crate::atomic_fs::is_executable_file;
use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

const CODEX_AGENT_COMMAND: &str = "codex";
const PLACEHOLDER_SHELL_CANDIDATES: &[&str] = &["nu", "bash", "sh"];
const MISSING_CODEX_PLACEHOLDER: &str = "\
Yazelix right sidebar

Codex is not installed or is not on PATH.
This pane is a normal shell; run any command here, or configure the managed right sidebar.

Agent examples:
  yzx config set agent.command opencode
  yzx config set agent.command claude

The right sidebar command does not have to be an AI agent:
  yzx config set agent.command nu
  yzx config ui

Starting a shell...
";

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
            "Run `yzx agent` without arguments for the managed right agent pane.",
            json!({ "argument": args[0] }),
        ));
    }

    let path = std::env::var_os("PATH").unwrap_or_default();
    if resolve_command_on_path(CODEX_AGENT_COMMAND, &path).is_none() {
        print_missing_codex_placeholder()?;
        return run_placeholder_shell(&path);
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
    println!("Open the managed right agent pane");
    println!();
    println!("Usage:");
    println!("  yzx agent");
    println!();
    println!("When host-installed Codex is available, this command launches `codex`.");
    println!("When Codex is missing, it opens a normal shell with setup guidance.");
}

fn print_missing_codex_placeholder() -> Result<(), CoreError> {
    let mut stdout = io::stdout();
    stdout
        .write_all(MISSING_CODEX_PLACEHOLDER.as_bytes())
        .map_err(render_placeholder_error)?;
    stdout.flush().map_err(render_placeholder_error)
}

fn render_placeholder_error(source: io::Error) -> CoreError {
    CoreError::io(
        "agent_placeholder_render_failed",
        "Failed to render the Yazelix agent placeholder.",
        "Retry the command; if stdout is closed, reopen the right sidebar pane.",
        "stdout",
        source,
    )
}

fn run_placeholder_shell(path: &OsStr) -> Result<i32, CoreError> {
    let Some(shell) = resolve_placeholder_shell(path) else {
        return Err(missing_placeholder_shell_error(path));
    };
    let status = Command::new(&shell).status().map_err(|source| {
        CoreError::io(
            "agent_placeholder_shell",
            "Failed to launch the right sidebar placeholder shell.",
            "Install Codex, configure agent.command to another executable, or make a shell available on PATH.",
            shell.display().to_string(),
            source,
        )
    })?;
    Ok(status.code().unwrap_or(1))
}

fn missing_placeholder_shell_error(path: &OsStr) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_agent_placeholder_shell",
        "Codex is not on PATH, and Yazelix could not find a shell for the right sidebar placeholder.",
        "Install Codex, configure agent.command to another executable, or make `nu`, `bash`, or `sh` available on PATH.",
        json!({
            "missing_command": CODEX_AGENT_COMMAND,
            "path": path.to_string_lossy(),
        }),
    )
}

fn resolve_placeholder_shell(path: &OsStr) -> Option<PathBuf> {
    resolve_placeholder_shell_for(path, std::env::var_os("SHELL").as_deref())
}

fn resolve_placeholder_shell_for(path: &OsStr, env_shell: Option<&OsStr>) -> Option<PathBuf> {
    if let Some(shell) = env_shell
        .map(PathBuf::from)
        .filter(|candidate| is_executable_file(candidate))
    {
        return Some(shell);
    }

    PLACEHOLDER_SHELL_CANDIDATES
        .iter()
        .find_map(|candidate| resolve_command_on_path(candidate, path))
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

// Test lane: default
#[cfg(test)]
mod tests {
    use super::{resolve_command_on_path, resolve_placeholder_shell_for};
    use std::ffi::OsStr;
    use std::fs;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    // Defends: yzx agent only launches a real executable from PATH, so missing host Codex does not accidentally exec arbitrary names.
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

    // Defends: the missing-Codex placeholder becomes an interactive pane by selecting an explicit shell.
    #[test]
    fn placeholder_shell_prefers_executable_user_shell() {
        let temp = tempfile::tempdir().unwrap();
        let shell = temp.path().join("shell");
        fs::write(&shell, "#!/bin/sh\nexit 0\n").unwrap();

        #[cfg(unix)]
        {
            fs::set_permissions(&shell, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert_eq!(
            resolve_placeholder_shell_for(OsStr::new(""), Some(shell.as_os_str())),
            Some(shell)
        );
    }
}
