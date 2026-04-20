//! Public Rust-owned `yzx` root router.

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use yazelix_core::bridge::{CoreError, ErrorClass};
use yazelix_core::control_plane::runtime_dir_from_env;
use yazelix_core::{render_yzx_help, yzx_command_metadata};

const INTERNAL_DISPATCH_RELATIVE_PATH: &[&str] =
    &["nushell", "scripts", "core", "yzx_internal_dispatch.nu"];
const VERSION_LINE_PREFIX: &str = "export const YAZELIX_VERSION = \"";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootRoute<'a> {
    Help,
    Version,
    RustControl,
    InternalNu(&'a str),
}

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

    match classify_root_route(&argv)? {
        RootRoute::Help => {
            print!("{}", render_yzx_help(&yzx_command_metadata()));
            Ok(0)
        }
        RootRoute::Version => {
            println!("Yazelix ({})", read_yazelix_version(&runtime_dir)?);
            Ok(0)
        }
        RootRoute::RustControl => run_rust_control(&runtime_dir, &argv),
        RootRoute::InternalNu(route) => run_internal_dispatch(&runtime_dir, route, &argv[1..]),
    }
}

fn classify_root_route(argv: &[String]) -> Result<RootRoute<'_>, CoreError> {
    let Some(first) = argv.first().map(|s| s.as_str()) else {
        return Ok(RootRoute::Help);
    };

    if matches!(first, "help" | "-h" | "--help") {
        return Ok(RootRoute::Help);
    }

    if matches!(first, "-V" | "--version" | "-v" | "--version-short") {
        return Ok(RootRoute::Version);
    }

    if matches!(first, "env" | "run" | "update") {
        return Ok(RootRoute::RustControl);
    }

    if matches!(
        first,
        "config"
            | "cwd"
            | "desktop"
            | "dev"
            | "doctor"
            | "edit"
            | "enter"
            | "home_manager"
            | "import"
            | "keys"
            | "launch"
            | "menu"
            | "popup"
            | "restart"
            | "reveal"
            | "screen"
            | "sponsor"
            | "status"
            | "tutor"
            | "whats_new"
            | "why"
    ) {
        return Ok(RootRoute::InternalNu(first));
    }

    Err(CoreError::classified(
        ErrorClass::Usage,
        "unknown_command",
        format!("Unknown yzx command: {first}"),
        "Run `yzx --help` to see available commands.",
        json!({ "command": first }),
    ))
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

fn resolve_internal_dispatch_path(runtime_dir: &Path) -> Result<PathBuf, CoreError> {
    if let Some(path) = resolve_override_path("YAZELIX_YZX_INTERNAL_DISPATCH_SCRIPT") {
        if path.is_file() {
            return Ok(path);
        }
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_internal_dispatch",
            format!("Missing Yazelix internal Nu dispatcher: {}", path.display()),
            "Fix YAZELIX_YZX_INTERNAL_DISPATCH_SCRIPT or reinstall Yazelix.",
            json!({ "path": path }),
        ));
    }

    let script_path = INTERNAL_DISPATCH_RELATIVE_PATH
        .iter()
        .fold(runtime_dir.to_path_buf(), |acc, part| acc.join(part));

    if script_path.is_file() {
        return Ok(script_path);
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_internal_dispatch",
        format!(
            "Missing Yazelix internal Nu dispatcher: {}",
            script_path.display()
        ),
        "Reinstall Yazelix or restore nushell/scripts/core/yzx_internal_dispatch.nu.",
        json!({ "path": script_path }),
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

fn run_rust_control(runtime_dir: &Path, argv: &[String]) -> Result<i32, CoreError> {
    let control_bin = resolve_rust_control_path(runtime_dir)?;
    run_child(Command::new(control_bin).args(argv), "yzx_control")
}

fn run_internal_dispatch(
    runtime_dir: &Path,
    route: &str,
    argv: &[String],
) -> Result<i32, CoreError> {
    let nu_bin = resolve_nu_bin()?;
    let script_path = resolve_internal_dispatch_path(runtime_dir)?;
    run_child(
        Command::new(nu_bin).arg(script_path).arg(route).args(argv),
        "internal_yzx_dispatch",
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

fn read_yazelix_version(runtime_dir: &Path) -> Result<String, CoreError> {
    let constants_path = runtime_dir
        .join("nushell")
        .join("scripts")
        .join("utils")
        .join("constants.nu");
    let contents = fs::read_to_string(&constants_path).map_err(|source| {
        CoreError::io(
            "version",
            format!(
                "Failed to read Yazelix version from {}.",
                constants_path.display()
            ),
            "Restore nushell/scripts/utils/constants.nu or reinstall Yazelix.",
            ".",
            source,
        )
    })?;

    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(VERSION_LINE_PREFIX) {
            if let Some(value) = rest.strip_suffix('"') {
                return Ok(value.to_string());
            }
        }
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_version_constant",
        format!(
            "Could not find YAZELIX_VERSION in {}.",
            constants_path.display()
        ),
        "Restore nushell/scripts/utils/constants.nu so the version constant is present.",
        json!({ "path": constants_path }),
    ))
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the public Rust root keeps the already migrated control-plane family on the Rust-owned path.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn classifies_rust_owned_control_family_at_root() {
        assert_eq!(
            classify_root_route(&["env".into(), "--no-shell".into()]).unwrap(),
            RootRoute::RustControl
        );
        assert_eq!(
            classify_root_route(&["run".into(), "rg".into()]).unwrap(),
            RootRoute::RustControl
        );
        assert_eq!(
            classify_root_route(&["update".into(), "nix".into()]).unwrap(),
            RootRoute::RustControl
        );
    }

    // Defends: the Rust root rejects unknown top-level commands instead of reviving the old generic Nu root fallback.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn rejects_unknown_top_level_command() {
        let err = classify_root_route(&["not-a-command".into()]).unwrap_err();
        assert!(matches!(err.class(), ErrorClass::Usage));
        assert_eq!(err.code(), "unknown_command");
    }
}
