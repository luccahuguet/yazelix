//! Public Rust-owned `yzx` root router.

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
use yazelix_core::bridge::{CoreError, ErrorClass};
use yazelix_core::control_plane::{read_yazelix_version_from_runtime, runtime_dir_from_env};
use yazelix_core::{
    YzxPublicRootRoute, classify_yzx_root_route, render_yzx_help, yzx_command_metadata,
};

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("{}", err.message());
            let remediation = err.remediation();
            if !remediation.is_empty() {
                eprintln!("{remediation}");
            }
            std::process::exit(err.class().exit_code());
        }
    }
}

fn run() -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let argv: Vec<String> = std::env::args().skip(1).collect();

    match classify_yzx_root_route(&argv)? {
        YzxPublicRootRoute::Help => {
            print!("{}", render_yzx_help(&yzx_command_metadata()));
            Ok(0)
        }
        YzxPublicRootRoute::Version => {
            println!(
                "Yazelix ({})",
                read_yazelix_version_from_runtime(&runtime_dir)?
            );
            Ok(0)
        }
        YzxPublicRootRoute::RustControl => run_rust_control(&runtime_dir, &argv),
        YzxPublicRootRoute::InternalNu(plan) => run_internal_nu_route(&runtime_dir, &plan),
    }
}

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

fn resolve_rust_control_path(runtime_dir: &Path) -> Result<PathBuf, CoreError> {
    if let Some(path) = resolve_override_path("YAZELIX_YZX_CONTROL_BIN") {
        return require_executable(
            path,
            "Missing Yazelix control helper",
            "Build `yzx_control`, fix YAZELIX_YZX_CONTROL_BIN, or reinstall Yazelix.",
        );
    }

    for candidate in [
        runtime_dir.join("libexec").join("yzx_control"),
        runtime_dir
            .join("rust_core")
            .join("target")
            .join("release")
            .join("yzx_control"),
        runtime_dir
            .join("rust_core")
            .join("target")
            .join("debug")
            .join("yzx_control"),
    ] {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_control_helper",
        format!(
            "Missing Yazelix control helper under {}.",
            runtime_dir.display()
        ),
        "Build `yzx_control`, set YAZELIX_YZX_CONTROL_BIN, or reinstall Yazelix so the runtime includes libexec/yzx_control.",
        json!({ "runtime_dir": runtime_dir }),
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

fn render_nu_command_script(module_path: &Path, command_name: &str, tail: &[String]) -> String {
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

fn run_rust_control(runtime_dir: &Path, argv: &[String]) -> Result<i32, CoreError> {
    let control_bin = resolve_rust_control_path(runtime_dir)?;
    run_child(Command::new(control_bin).args(argv), "yzx_control")
}

fn run_internal_nu_route(
    runtime_dir: &Path,
    plan: &yazelix_core::YzxInternalNuRoutePlan<'_>,
) -> Result<i32, CoreError> {
    let nu_bin = resolve_nu_bin()?;
    let module_path = resolve_internal_nu_module_path(runtime_dir, plan.module_relative_path)?;
    let script = render_nu_command_script(&module_path, plan.command_name, plan.tail);
    run_child(
        Command::new(nu_bin).arg("-c").arg(script),
        "internal_yzx_route",
    )
}

fn run_child(command: &mut Command, owner: &str) -> Result<i32, CoreError> {
    let status = command.status().map_err(|source| {
        CoreError::io(
            owner,
            format!("Failed to launch {owner}."),
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

    // Defends: the Rust root renders direct Nu module invocations without reviving the old dispatcher script hop.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_direct_nu_module_invocation_script() {
        let script = render_nu_command_script(
            Path::new("/tmp/runtime/nushell/scripts/core/yzx_workspace.nu"),
            "yzx reveal",
            &[
                "--".into(),
                "--help".into(),
                "/tmp/has space".into(),
                "quote\"value".into(),
            ],
        );
        assert_eq!(
            script,
            "use \"/tmp/runtime/nushell/scripts/core/yzx_workspace.nu\" [\"yzx reveal\"]; yzx reveal -- --help \"/tmp/has space\" \"quote\\\"value\""
        );
    }
}
