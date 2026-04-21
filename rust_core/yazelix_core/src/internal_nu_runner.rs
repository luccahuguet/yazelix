//! Shared internal Nushell module invocation for Rust-owned public owners.

use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

fn resolve_override_path(env_key: &str) -> Option<PathBuf> {
    let raw = std::env::var(env_key).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
}

fn require_executable(
    path: PathBuf,
    missing_label: &str,
    remediation: &str,
) -> Result<PathBuf, CoreError> {
    if path.is_file() {
        return Ok(path);
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_runtime_helper",
        format!("{missing_label}: {}", path.display()),
        remediation,
        json!({ "path": path }),
    ))
}

fn resolve_nu_bin() -> Result<PathBuf, CoreError> {
    let Some(path) = resolve_override_path("YAZELIX_NU_BIN") else {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_nu_bin",
            "YAZELIX_NU_BIN is not set.",
            "Run `yzx` through the packaged POSIX launcher so the runtime bootstraps correctly.",
            json!({}),
        ));
    };

    require_executable(
        path,
        "Missing Nushell binary for Yazelix",
        "Fix YAZELIX_NU_BIN or reinstall Yazelix so the runtime includes a working Nushell binary.",
    )
}

fn resolve_internal_nu_module_path(
    runtime_dir: &Path,
    module_relative_path: &[&str],
) -> Result<PathBuf, CoreError> {
    let route_root = resolve_override_path("YAZELIX_YZX_NU_ROUTE_ROOT")
        .unwrap_or_else(|| runtime_dir.to_path_buf());
    let module_path = module_relative_path
        .iter()
        .fold(route_root, |acc, part| acc.join(part));

    if module_path.is_file() {
        return Ok(module_path);
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_internal_nu_module",
        format!(
            "Missing Yazelix internal Nu module: {}",
            module_path.display()
        ),
        "Fix YAZELIX_YZX_NU_ROUTE_ROOT or reinstall Yazelix so the required Nushell module exists under nushell/scripts/.",
        json!({ "path": module_path }),
    ))
}

fn is_safe_unquoted_nu_token(arg: &str) -> bool {
    !arg.is_empty()
        && arg.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(
                    ch,
                    '-' | '_' | '.' | '/' | ':' | '=' | '+' | ',' | '@' | '%'
                )
        })
}

fn render_nu_string_literal(raw: &str) -> String {
    let mut out = String::from("\"");
    for ch in raw.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn render_nu_arg(arg: &str) -> String {
    if arg == "--" || is_safe_unquoted_nu_token(arg) {
        arg.to_string()
    } else {
        render_nu_string_literal(arg)
    }
}

pub fn render_nu_command_script(module_path: &Path, command_name: &str, tail: &[String]) -> String {
    let mut script = format!(
        "use {} [{}]; {}",
        render_nu_string_literal(&module_path.to_string_lossy()),
        render_nu_string_literal(command_name),
        command_name
    );
    for arg in tail {
        script.push(' ');
        script.push_str(&render_nu_arg(arg));
    }
    script
}

pub fn run_internal_nu_module_command(
    runtime_dir: &Path,
    module_relative_path: &[&str],
    command_name: &str,
    tail: &[String],
    extra_env: &[(&str, &str)],
) -> Result<i32, CoreError> {
    let nu_bin = resolve_nu_bin()?;
    let module_path = resolve_internal_nu_module_path(runtime_dir, module_relative_path)?;
    let script = render_nu_command_script(&module_path, command_name, tail);
    let mut command = Command::new(nu_bin);
    command.arg("-c").arg(script);
    for (key, value) in extra_env {
        command.env(key, value);
    }

    let status = command.status().map_err(|source| {
        CoreError::io(
            "internal_yzx_route",
            "Failed to launch internal_yzx_route.",
            "Reinstall Yazelix or restore the missing runtime helper, then retry.",
            ".",
            source,
        )
    })?;

    Ok(status.code().unwrap_or(1))
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: shared internal Nu route rendering preserves spaces, quotes, and direct module targeting for Rust-owned command handoff.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_direct_nu_module_invocation_script() {
        let script = render_nu_command_script(
            Path::new("/tmp/runtime/nushell/scripts/core/yzx_session.nu"),
            "yzx restart",
            &[
                "--".into(),
                "--help".into(),
                "/tmp/has space".into(),
                "quote\"value".into(),
            ],
        );
        assert_eq!(
            script,
            "use \"/tmp/runtime/nushell/scripts/core/yzx_session.nu\" [\"yzx restart\"]; yzx restart -- --help \"/tmp/has space\" \"quote\\\"value\""
        );
    }
}
