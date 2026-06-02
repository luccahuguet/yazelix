//! Public Rust-owned `yzx` root router.

use serde_json::json;
use std::path::Path;
use std::process::Command;
use yazelix_core::bridge::{CoreError, ErrorClass};
use yazelix_core::control_plane::{
    read_runtime_identity_from_runtime, read_yazelix_version_from_runtime, runtime_dir_from_env,
};
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
        YzxPublicRootRoute::VersionFull => print_full_version(&runtime_dir),
        YzxPublicRootRoute::RustControl => run_rust_control(&runtime_dir, &argv),
    }
}

fn print_full_version(runtime_dir: &Path) -> Result<i32, CoreError> {
    let payload = json!({
        "schema_version": 1,
        "version": read_yazelix_version_from_runtime(runtime_dir)?,
        "runtime_dir": runtime_dir.to_string_lossy(),
        "runtime_identity": read_runtime_identity_from_runtime(runtime_dir)?,
    });
    serde_json::to_writer_pretty(std::io::stdout(), &payload).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "version_full_render_failed",
            "Failed to render Yazelix full version output.",
            "Retry the command and report this as a Yazelix internal error if it persists.",
            json!({ "error": source.to_string() }),
        )
    })?;
    println!();
    Ok(0)
}

fn require_executable(
    path: std::path::PathBuf,
    missing_label: &str,
    remediation: &str,
) -> Result<std::path::PathBuf, CoreError> {
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

fn resolve_rust_control_path(runtime_dir: &Path) -> Result<std::path::PathBuf, CoreError> {
    if let Some(path) = std::env::var("YAZELIX_YZX_CONTROL_BIN")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|trimmed| !trimmed.is_empty())
        .map(std::path::PathBuf::from)
    {
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

fn run_rust_control(runtime_dir: &Path, argv: &[String]) -> Result<i32, CoreError> {
    let control_bin = resolve_rust_control_path(runtime_dir)?;
    run_child(Command::new(control_bin).args(argv), "yzx_control")
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
