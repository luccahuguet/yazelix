//! Public Rust-owned `yzx` root router.

use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
use yazelix_core::bridge::{CoreError, ErrorClass};
use yazelix_core::control_plane::{read_yazelix_version_from_runtime, runtime_dir_from_env};
use yazelix_core::{render_yzx_help, yzx_command_metadata};

const CORE_DOCTOR_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "core", "yzx_doctor.nu"];
const CORE_SESSION_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "core", "yzx_session.nu"];
const CORE_SUPPORT_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "core", "yzx_support.nu"];
const CORE_WORKSPACE_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "core", "yzx_workspace.nu"];
const YZX_CONFIG_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "config.nu"];
const YZX_DESKTOP_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "desktop.nu"];
const YZX_DEV_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "dev.nu"];
const YZX_EDIT_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "edit.nu"];
const YZX_ENTER_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "enter.nu"];
const YZX_HOME_MANAGER_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "home_manager.nu"];
const YZX_IMPORT_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "import.nu"];
const YZX_KEYS_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "keys.nu"];
const YZX_LAUNCH_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "launch.nu"];
const YZX_MENU_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "menu.nu"];
const YZX_POPUP_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "popup.nu"];
const YZX_SCREEN_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "screen.nu"];
const YZX_TUTOR_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "tutor.nu"];
const YZX_WHATS_NEW_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "whats_new.nu"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RootRoute<'a> {
    Help,
    Version,
    RustControl,
    InternalNu(&'a str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NuRoutePlan<'a> {
    module_relative_path: &'static [&'static str],
    command_name: &'static str,
    tail: &'a [String],
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
            println!(
                "Yazelix ({})",
                read_yazelix_version_from_runtime(&runtime_dir)?
            );
            Ok(0)
        }
        RootRoute::RustControl => run_rust_control(&runtime_dir, &argv),
        RootRoute::InternalNu(route) => run_internal_nu_route(&runtime_dir, route, &argv[1..]),
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

    if matches!(first, "env" | "run" | "status" | "update") {
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

fn plan_nu_route<'a>(
    module_relative_path: &'static [&'static str],
    command_name: &'static str,
    tail: &'a [String],
) -> NuRoutePlan<'a> {
    NuRoutePlan {
        module_relative_path,
        command_name,
        tail,
    }
}

fn first_arg(argv: &[String]) -> Option<&str> {
    argv.first().map(String::as_str)
}

fn unknown_subcommand_error(route: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "unknown_subcommand",
        format!("Unknown yzx {route} subcommand"),
        format!("Run `yzx {route} --help` or `yzx --help` to see supported commands."),
        json!({ "route": route }),
    )
}

fn required_subcommand_error(route: &str, expected: &[&str]) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "missing_subcommand",
        format!("yzx {route} requires one of: {}", expected.join(", ")),
        format!("Run `yzx {route} --help` or `yzx --help` to see supported subcommands."),
        json!({ "route": route, "expected": expected }),
    )
}

fn plan_internal_nu_route<'a>(
    route: &str,
    argv: &'a [String],
) -> Result<NuRoutePlan<'a>, CoreError> {
    let plan = match route {
        "config" => match first_arg(argv) {
            Some("reset") => {
                plan_nu_route(YZX_CONFIG_RELATIVE_PATH, "yzx config reset", &argv[1..])
            }
            _ => plan_nu_route(YZX_CONFIG_RELATIVE_PATH, "yzx config", argv),
        },
        "cwd" => plan_nu_route(CORE_WORKSPACE_RELATIVE_PATH, "yzx cwd", argv),
        "desktop" => match first_arg(argv) {
            Some("install") => {
                plan_nu_route(YZX_DESKTOP_RELATIVE_PATH, "yzx desktop install", &argv[1..])
            }
            Some("launch") => {
                plan_nu_route(YZX_DESKTOP_RELATIVE_PATH, "yzx desktop launch", &argv[1..])
            }
            Some("uninstall") => plan_nu_route(
                YZX_DESKTOP_RELATIVE_PATH,
                "yzx desktop uninstall",
                &argv[1..],
            ),
            _ => {
                return Err(required_subcommand_error(
                    "desktop",
                    &["install", "launch", "uninstall"],
                ));
            }
        },
        "dev" => match first_arg(argv) {
            None => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev", argv),
            Some("help") => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev", &[]),
            Some("-h") | Some("--help") => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev", argv),
            Some("build_pane_orchestrator") => plan_nu_route(
                YZX_DEV_RELATIVE_PATH,
                "yzx dev build_pane_orchestrator",
                &argv[1..],
            ),
            Some("bump") => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev bump", &argv[1..]),
            Some("lint_nu") => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev lint_nu", &argv[1..]),
            Some("profile") => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev profile", &argv[1..]),
            Some("sync_issues") => {
                plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev sync_issues", &argv[1..])
            }
            Some("test") => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev test", &argv[1..]),
            Some("update") => plan_nu_route(YZX_DEV_RELATIVE_PATH, "yzx dev update", &argv[1..]),
            _ => return Err(unknown_subcommand_error("dev")),
        },
        "doctor" => plan_nu_route(CORE_DOCTOR_RELATIVE_PATH, "yzx doctor", argv),
        "edit" => match first_arg(argv) {
            Some("config") => plan_nu_route(YZX_EDIT_RELATIVE_PATH, "yzx edit config", &argv[1..]),
            _ => plan_nu_route(YZX_EDIT_RELATIVE_PATH, "yzx edit", argv),
        },
        "enter" => plan_nu_route(YZX_ENTER_RELATIVE_PATH, "yzx enter", argv),
        "home_manager" => match first_arg(argv) {
            Some("prepare") => plan_nu_route(
                YZX_HOME_MANAGER_RELATIVE_PATH,
                "yzx home_manager prepare",
                &argv[1..],
            ),
            _ => plan_nu_route(YZX_HOME_MANAGER_RELATIVE_PATH, "yzx home_manager", argv),
        },
        "import" => match first_arg(argv) {
            None => plan_nu_route(YZX_IMPORT_RELATIVE_PATH, "yzx import", argv),
            Some("help") => plan_nu_route(YZX_IMPORT_RELATIVE_PATH, "yzx import", &[]),
            Some("-h") | Some("--help") => {
                plan_nu_route(YZX_IMPORT_RELATIVE_PATH, "yzx import", argv)
            }
            Some("helix") => {
                plan_nu_route(YZX_IMPORT_RELATIVE_PATH, "yzx import helix", &argv[1..])
            }
            Some("yazi") => plan_nu_route(YZX_IMPORT_RELATIVE_PATH, "yzx import yazi", &argv[1..]),
            Some("zellij") => {
                plan_nu_route(YZX_IMPORT_RELATIVE_PATH, "yzx import zellij", &argv[1..])
            }
            _ => return Err(unknown_subcommand_error("import")),
        },
        "keys" => match first_arg(argv) {
            Some("yzx") => plan_nu_route(YZX_KEYS_RELATIVE_PATH, "yzx keys yzx", &argv[1..]),
            Some("yazi") => plan_nu_route(YZX_KEYS_RELATIVE_PATH, "yzx keys yazi", &argv[1..]),
            Some("hx") => plan_nu_route(YZX_KEYS_RELATIVE_PATH, "yzx keys hx", &argv[1..]),
            Some("helix") => plan_nu_route(YZX_KEYS_RELATIVE_PATH, "yzx keys helix", &argv[1..]),
            Some("nu") => plan_nu_route(YZX_KEYS_RELATIVE_PATH, "yzx keys nu", &argv[1..]),
            Some("nushell") => {
                plan_nu_route(YZX_KEYS_RELATIVE_PATH, "yzx keys nushell", &argv[1..])
            }
            _ => plan_nu_route(YZX_KEYS_RELATIVE_PATH, "yzx keys", argv),
        },
        "launch" => plan_nu_route(YZX_LAUNCH_RELATIVE_PATH, "yzx launch", argv),
        "menu" => plan_nu_route(YZX_MENU_RELATIVE_PATH, "yzx menu", argv),
        "popup" => plan_nu_route(YZX_POPUP_RELATIVE_PATH, "yzx popup", argv),
        "restart" => plan_nu_route(CORE_SESSION_RELATIVE_PATH, "yzx restart", argv),
        "reveal" => plan_nu_route(CORE_WORKSPACE_RELATIVE_PATH, "yzx reveal", argv),
        "screen" => plan_nu_route(YZX_SCREEN_RELATIVE_PATH, "yzx screen", argv),
        "sponsor" => plan_nu_route(CORE_SUPPORT_RELATIVE_PATH, "yzx sponsor", argv),
        "tutor" => match first_arg(argv) {
            Some("helix") => plan_nu_route(YZX_TUTOR_RELATIVE_PATH, "yzx tutor helix", &argv[1..]),
            Some("hx") => plan_nu_route(YZX_TUTOR_RELATIVE_PATH, "yzx tutor hx", &argv[1..]),
            Some("nu") => plan_nu_route(YZX_TUTOR_RELATIVE_PATH, "yzx tutor nu", &argv[1..]),
            Some("nushell") => {
                plan_nu_route(YZX_TUTOR_RELATIVE_PATH, "yzx tutor nushell", &argv[1..])
            }
            _ => plan_nu_route(YZX_TUTOR_RELATIVE_PATH, "yzx tutor", argv),
        },
        "whats_new" => plan_nu_route(YZX_WHATS_NEW_RELATIVE_PATH, "yzx whats_new", argv),
        "why" => plan_nu_route(CORE_SUPPORT_RELATIVE_PATH, "yzx why", argv),
        _ => unreachable!("classify_root_route already filters unsupported internal routes"),
    };

    Ok(plan)
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
    route: &str,
    argv: &[String],
) -> Result<i32, CoreError> {
    let nu_bin = resolve_nu_bin()?;
    let plan = plan_internal_nu_route(route, argv)?;
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

    // Defends: grouped Nu-owned families are planned directly to their concrete module instead of a shared dispatcher bridge.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn plans_grouped_internal_family_to_direct_module() {
        let argv = vec!["launch".into()];
        let plan = plan_internal_nu_route("desktop", &argv).unwrap();
        assert_eq!(plan.module_relative_path, YZX_DESKTOP_RELATIVE_PATH);
        assert_eq!(plan.command_name, "yzx desktop launch");
        assert!(plan.tail.is_empty());
    }

    // Defends: the Rust root renders direct Nu module invocations without reviving the old dispatcher script hop.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_direct_nu_module_invocation_script() {
        let script = render_nu_command_script(
            Path::new("/tmp/runtime/nushell/scripts/core/yzx_workspace.nu"),
            "yzx reveal",
            &["--help".into(), "/tmp/has space".into()],
        );
        assert_eq!(
            script,
            "use \"/tmp/runtime/nushell/scripts/core/yzx_workspace.nu\" [\"yzx reveal\"]; yzx reveal --help \"/tmp/has space\""
        );
    }
}
