// Test lane: default
//! Rust-owned `yzx enter`, `yzx launch`, `yzx desktop`, and `yzx restart` owners.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{home_dir_from_env, state_dir_from_env};
use crate::sidebar_bootstrap::{
    SIDEBAR_BOOTSTRAP_CWD_ENV, is_sidebar_bootstrap_file, sidebar_bootstrap_owner_dir,
};
use crate::terminal_materialization::MARS_EMOJI_ENV_KEYS;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod config_override;
mod desktop;
mod enter;
mod launch;
mod process;
mod restart;
mod terminal;

use desktop::*;

pub(super) const RUNTIME_RELAUNCH_CLEARED_ENV_KEYS: &[&str] = &[
    "IN_YAZELIX_SHELL",
    "RIO_CONFIG_HOME",
    "YAZELIX_BOOTSTRAP_RUNTIME_DIR",
    "YAZELIX_CURSOR_COLOR",
    "YAZELIX_CURSOR_DIVIDER",
    "YAZELIX_CURSOR_FAMILY",
    "YAZELIX_CURSOR_NAME",
    "YAZELIX_CURSOR_PRIMARY_COLOR",
    "YAZELIX_CURSOR_SECONDARY_COLOR",
    "YAZELIX_DIR",
    "YAZELIX_INVOKED_YZX_PATH",
    "YAZELIX_NU_BIN",
    "YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH",
    "YAZELIX_RUNTIME_DIR",
    "YAZELIX_SESSION_CONFIG_PATH",
    "YAZELIX_SESSION_FACTS_PATH",
    "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT",
    "YAZELIX_STARTUP_PROFILE_SKIP_WELCOME",
    "YAZELIX_STATUS_BAR_CACHE_PATH",
    "MARS",
    "MARS_APP_ID",
    "MARS_CONFIG",
    "MARS_CONFIG_HOME",
    "MARS_APPEARANCE",
    "MARS_CHILD_ENV_SANITIZE",
    "MARS_EFFECTS",
    "MARS_EMOJI_FONT",
    "MARS_GRAPHICS_WRAPPER",
    "MARS_PROFILE",
    "MARS_RENDER_STRATEGY",
    MARS_EMOJI_ENV_KEYS[0],
    MARS_EMOJI_ENV_KEYS[1],
    "YAZELIX_YZX_BIN",
    "YAZELIX_YZX_CONTROL_BIN",
    "YAZELIX_YZX_CORE_BIN",
    "YAZI_ID",
    "ZELLIJ",
    "ZELLIJ_DEFAULT_LAYOUT",
    "ZELLIJ_PANE_ID",
    "ZELLIJ_SESSION_NAME",
    "ZELLIJ_TAB_NAME",
    "ZELLIJ_TAB_POSITION",
];

pub(super) fn current_runtime_yzx_launcher(runtime_dir: &Path) -> PathBuf {
    let bin_yzx = runtime_dir.join("bin").join("yzx");
    if bin_yzx.exists() {
        return bin_yzx;
    }
    runtime_dir.join("shells").join("posix").join("yzx_cli.sh")
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DesktopArgs {
    subcommand: Option<String>,
    action: Option<String>,
    print_path: bool,
    help: bool,
}

pub fn run_yzx_enter(args: &[String]) -> Result<i32, CoreError> {
    enter::run_enter(args)
}

pub fn run_yzx_restart(args: &[String]) -> Result<i32, CoreError> {
    restart::run_restart(args)
}

pub fn run_yzx_launch(args: &[String]) -> Result<i32, CoreError> {
    launch::run_launch(args)
}

pub fn run_yzx_desktop(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_desktop_args(args)?;
    if parsed.help || parsed.subcommand.is_none() {
        print_desktop_help();
        return Ok(0);
    }

    match parsed.subcommand.as_deref() {
        Some("install") => run_desktop_install(parsed.print_path),
        Some("uninstall") => run_desktop_uninstall(parsed.print_path),
        Some("launch") => run_desktop_launch(),
        Some("macos_preview") => match parsed.action.as_deref() {
            Some("install") => run_macos_preview_install(parsed.print_path),
            Some("uninstall") => run_macos_preview_uninstall(parsed.print_path),
            Some(other) => Err(CoreError::usage(format!(
                "Unknown yzx desktop macos_preview action: {other}. Try `yzx desktop --help`."
            ))),
            None => Err(CoreError::usage(
                "yzx desktop macos_preview requires an action: install or uninstall.",
            )),
        },
        Some(other) => Err(CoreError::usage(format!(
            "Unknown yzx desktop subcommand: {other}. Try `yzx desktop --help`."
        ))),
        None => unreachable!(),
    }
}

fn create_sidebar_bootstrap_file(owner: &str, target_dir: &Path) -> Result<PathBuf, CoreError> {
    create_sidebar_bootstrap_file_in_state(&state_dir_from_env()?, owner, target_dir)
}

fn create_sidebar_bootstrap_file_in_state(
    state_dir: &Path,
    owner: &str,
    target_dir: &Path,
) -> Result<PathBuf, CoreError> {
    let bootstrap_dir = sidebar_bootstrap_owner_dir(state_dir, owner);
    fs::create_dir_all(&bootstrap_dir).map_err(|source| {
        CoreError::io(
            "sidebar_bootstrap_state_dir",
            format!(
                "Could not create sidebar bootstrap state directory {}.",
                bootstrap_dir.display()
            ),
            "Fix the directory permissions, then retry.",
            bootstrap_dir.display().to_string(),
            source,
        )
    })?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            CoreError::classified(
                ErrorClass::Internal,
                "system_clock_error",
                format!("System clock error while preparing sidebar bootstrap file: {error}"),
                "Fix the system clock, then retry.",
                serde_json::json!({}),
            )
        })?
        .as_millis();
    let path = bootstrap_dir.join(format!("cwd_{}_{}.tmp", std::process::id(), timestamp));
    fs::write(&path, target_dir.to_string_lossy().into_owned()).map_err(|source| {
        CoreError::io(
            "sidebar_bootstrap_write",
            format!("Could not write sidebar bootstrap file {}.", path.display()),
            "Fix the directory permissions, then retry.",
            path.display().to_string(),
            source,
        )
    })?;
    Ok(path)
}

fn sidebar_bootstrap_extra_env(
    owner: &str,
    target_dir: &Path,
) -> Result<Vec<(String, Option<String>)>, CoreError> {
    let inherited = std::env::var(SIDEBAR_BOOTSTRAP_CWD_ENV).ok();
    sidebar_bootstrap_extra_env_for_state(
        &state_dir_from_env()?,
        owner,
        target_dir,
        inherited.as_deref(),
    )
}

fn sidebar_bootstrap_extra_env_for_state(
    state_dir: &Path,
    owner: &str,
    target_dir: &Path,
    inherited: Option<&str>,
) -> Result<Vec<(String, Option<String>)>, CoreError> {
    if inherited_sidebar_bootstrap_file(state_dir, inherited).is_some() {
        return Ok(Vec::new());
    }

    let path = create_sidebar_bootstrap_file_in_state(state_dir, owner, target_dir)?;
    Ok(vec![(
        SIDEBAR_BOOTSTRAP_CWD_ENV.to_string(),
        Some(path.to_string_lossy().into_owned()),
    )])
}

fn inherited_sidebar_bootstrap_file(state_dir: &Path, raw: Option<&str>) -> Option<PathBuf> {
    let raw = raw.map(str::trim).filter(|value| !value.is_empty())?;
    let path = PathBuf::from(raw);
    is_sidebar_bootstrap_file(state_dir, &path).then_some(path)
}

fn parse_desktop_args(args: &[String]) -> Result<DesktopArgs, CoreError> {
    let mut parsed = DesktopArgs::default();
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" | "help" => parsed.help = true,
            "--print-path" | "-p" => parsed.print_path = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx desktop: {other}. Try `yzx desktop --help`."
                )));
            }
            other => {
                if parsed.subcommand.as_deref() == Some("macos_preview") && parsed.action.is_none()
                {
                    parsed.action = Some(other.to_string());
                } else if parsed.subcommand.is_some() {
                    return Err(CoreError::usage(
                        "yzx desktop requires one subcommand: install, launch, uninstall, or macos_preview install|uninstall.",
                    ));
                } else if ["install", "launch", "uninstall"].contains(&other) {
                    parsed.subcommand = Some(other.to_string());
                } else if other == "macos_preview" {
                    parsed.subcommand = Some(other.to_string());
                } else {
                    parsed.subcommand = Some(other.to_string());
                }
            }
        }
    }
    Ok(parsed)
}

fn print_desktop_help() {
    println!("Desktop integration commands");
    println!();
    println!("Usage:");
    println!("  yzx desktop install [--print-path]");
    println!("  yzx desktop launch");
    println!("  yzx desktop uninstall [--print-path]");
    println!("  yzx desktop macos_preview install [--print-path]");
    println!("  yzx desktop macos_preview uninstall [--print-path]");
    println!("  macos_preview is unsigned, unnotarized, and community-tested");
}

fn resolve_requested_working_dir(path: Option<&str>, home: bool) -> Result<PathBuf, CoreError> {
    if home {
        return home_dir_from_env();
    }
    if let Some(path) = path.map(str::trim).filter(|path| !path.is_empty()) {
        return Ok(PathBuf::from(path));
    }
    std::env::current_dir().map_err(|source| {
        CoreError::io(
            "cwd",
            "Could not read the current working directory.",
            "cd into a valid directory, then retry.",
            ".",
            source,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::config_override::*;
    use super::process::*;
    use super::restart::*;
    use super::*;
    use crate::control_plane::load_normalized_config_for_control;
    use crate::settings_surface::read_settings_jsonc_value;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;

    fn write_runtime_layout(runtime: &Path) {
        fs::create_dir_all(runtime.join("config_metadata")).expect("metadata dir");
        fs::write(
            runtime
                .join("config_metadata")
                .join("main_config_contract.toml"),
            include_str!("../../../config_metadata/main_config_contract.toml"),
        )
        .expect("main config contract");
        fs::write(
            runtime.join("settings_default.jsonc"),
            include_str!("../../../settings_default.jsonc"),
        )
        .expect("main defaults");
        fs::write(
            runtime.join(crate::ghostty_cursor_registry::DEFAULT_CURSOR_CONFIG_FILENAME),
            include_str!("../../../yazelix_cursors_default.toml"),
        )
        .expect("cursor defaults");
        fs::write(
            runtime.join("runtime_components.json"),
            r#"{
              "cursors": { "enabled": true, "disableable": true, "notes": [] },
              "screen": { "enabled": true, "disableable": true, "notes": [] }
            }"#,
        )
        .expect("runtime component manifest");
    }

    // Defends: public one-shot config file overrides resolve before launch so child terminals do not reinterpret relative paths from a different cwd.
    #[test]
    fn config_override_paths_are_invocation_scoped() {
        let cwd = Path::new("/tmp/project");
        let home = Path::new("/home/demo");

        assert_eq!(
            resolve_config_override_path("alt/settings.jsonc", cwd, home).unwrap(),
            "/tmp/project/alt/settings.jsonc"
        );
        assert_eq!(
            resolve_config_override_path("~/settings.jsonc", cwd, home).unwrap(),
            "/home/demo/settings.jsonc"
        );
        assert_eq!(
            config_override_extra_env(Some("/tmp/custom.jsonc")),
            vec![(
                "YAZELIX_CONFIG_OVERRIDE".to_string(),
                Some("/tmp/custom.jsonc".to_string())
            )]
        );
    }

    // Regression: startup sidebar yazi gets the requested session cwd through an explicit bootstrap file instead of inheriting Zellij's home-scoped pane cwd.
    #[test]
    fn sidebar_bootstrap_env_writes_requested_startup_cwd() {
        let state = TempDir::new().expect("state dir");
        let target = state.path().join("project");
        fs::create_dir_all(&target).expect("target dir");

        let env =
            sidebar_bootstrap_extra_env_for_state(state.path(), "enter", &target, None).unwrap();

        assert_eq!(env.len(), 1);
        assert_eq!(env[0].0, SIDEBAR_BOOTSTRAP_CWD_ENV);
        let bootstrap_file = PathBuf::from(env[0].1.as_ref().unwrap());
        assert!(bootstrap_file.starts_with(state.path().join("sidebar_bootstrap").join("enter")));
        assert_eq!(
            fs::read_to_string(bootstrap_file).unwrap(),
            target.to_string_lossy()
        );
    }

    // Defends: restart can pass its existing one-shot sidebar cwd through launch/enter without enter replacing it with a terminal-emulator fallback cwd.
    #[test]
    fn sidebar_bootstrap_env_preserves_existing_restart_file() {
        let state = TempDir::new().expect("state dir");
        let inherited = state
            .path()
            .join("sidebar_bootstrap")
            .join("restart")
            .join("cwd.tmp");
        fs::create_dir_all(inherited.parent().unwrap()).expect("bootstrap dir");
        fs::write(&inherited, "/restart/cwd").expect("restart bootstrap");

        let env = sidebar_bootstrap_extra_env_for_state(
            state.path(),
            "enter",
            Path::new("/terminal/cwd"),
            Some(&inherited.to_string_lossy()),
        )
        .unwrap();

        assert!(env.is_empty());
        assert_eq!(fs::read_to_string(inherited).unwrap(), "/restart/cwd");
    }

    // Defends: unrelated inherited env paths cannot suppress the launch-owned sidebar cwd file.
    #[test]
    fn sidebar_bootstrap_env_ignores_unowned_inherited_file() {
        let state = TempDir::new().expect("state dir");
        let target = state.path().join("project");
        let inherited = state.path().join("outside.tmp");
        fs::create_dir_all(&target).expect("target dir");
        fs::write(&inherited, "/wrong/cwd").expect("unowned bootstrap");

        let env = sidebar_bootstrap_extra_env_for_state(
            state.path(),
            "enter",
            &target,
            Some(&inherited.to_string_lossy()),
        )
        .unwrap();

        assert_eq!(env.len(), 1);
        let bootstrap_file = PathBuf::from(env[0].1.as_ref().unwrap());
        assert!(bootstrap_file.starts_with(state.path().join("sidebar_bootstrap").join("enter")));
        assert_eq!(fs::read_to_string(inherited).unwrap(), "/wrong/cwd");
    }

    // Defends: restart exposes a one-shot welcome skip flag without making the config skip setting sticky.
    #[test]
    fn parse_restart_args_accepts_skip_aliases() {
        for arg in ["-s", "--skip"] {
            let parsed = parse_restart_args(&[arg.into()]).unwrap();

            assert!(parsed.skip_welcome);
            assert!(!parsed.help);
        }

        let help = parse_restart_args(&["--help".into()]).unwrap();
        assert!(help.help);
    }

    // Defends: restart can replace the inherited config override for one relaunched window without mutating settings.jsonc.
    #[test]
    fn parse_restart_args_accepts_config_override() {
        let expected_config = resolve_config_override_path(
            "minimal.jsonc",
            &std::env::current_dir().unwrap(),
            &home_dir_from_env().unwrap(),
        )
        .unwrap();
        let parsed = parse_restart_args(&[
            "--skip".into(),
            "--config".into(),
            "minimal.jsonc".into(),
            "--with".into(),
            "core.welcome_style=static".into(),
            "--with".into(),
            "zellij.pane_frames=false".into(),
        ])
        .unwrap();

        assert!(parsed.skip_welcome);
        assert_eq!(parsed.config.as_deref(), Some(expected_config.as_str()));
        assert_eq!(
            parsed.with_overrides,
            vec!["core.welcome_style=static", "zellij.pane_frames=false"]
        );
    }

    // Defends: repeatable --with patches stay contract-typed and reject unknown settings before launch materialization.
    #[test]
    fn session_config_patches_are_contract_typed() {
        let fields = HashMap::from([
            (
                "editor.command".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::String,
                },
            ),
            (
                "core.skip_welcome_screen".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::Bool,
                },
            ),
            (
                "core.welcome_duration_seconds".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::Float,
                },
            ),
            (
                "workspace.left_sidebar.width_percent".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::Int,
                },
            ),
            (
                "zellij.widget_tray".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::StringList,
                },
            ),
            (
                "zellij.keybindings".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::StringListMap,
                },
            ),
        ]);
        let mut root = serde_json::json!({
            "core": { "skip_welcome_screen": false },
            "editor": {},
            "workspace": { "left_sidebar": {} },
            "terminal": {},
            "zellij": { "keybindings": { "bottom_popup": ["Alt p"] } }
        });

        for raw in [
            "editor.command=nvim",
            "core.skip_welcome_screen=true",
            "core.welcome_duration_seconds=3.5",
            "workspace.left_sidebar.width_percent=24",
            "zellij.widget_tray=[\"editor\", \"term\"]",
            "zellij.keybindings={\"bottom_popup\":[\"Alt Shift J\"],\"config\":[]}",
        ] {
            let patch = parse_session_config_patch(raw, &fields).unwrap();
            apply_session_config_patch(&mut root, &patch).unwrap();
        }

        assert_eq!(root["editor"]["command"], "nvim");
        assert_eq!(root["core"]["skip_welcome_screen"], true);
        assert_eq!(root["core"]["welcome_duration_seconds"], 3.5);
        assert_eq!(root["workspace"]["left_sidebar"]["width_percent"], 24);
        assert_eq!(
            root["zellij"]["widget_tray"],
            serde_json::json!(["editor", "term"])
        );
        assert_eq!(
            root["zellij"]["keybindings"],
            serde_json::json!({
                "bottom_popup": ["Alt Shift J"],
                "config": [],
            })
        );

        let unknown = parse_session_config_patch("editor.nope=true", &fields).unwrap_err();
        assert!(
            unknown
                .to_string()
                .contains("Unknown Yazelix config setting")
        );
        let invalid_bool =
            parse_session_config_patch("core.skip_welcome_screen=maybe", &fields).unwrap_err();
        assert!(invalid_bool.to_string().contains("Invalid boolean value"));
        let invalid_map = parse_session_config_patch(
            "zellij.keybindings={\"bottom_popup\":\"Alt Shift J\"}",
            &fields,
        )
        .unwrap_err();
        assert!(
            invalid_map
                .to_string()
                .contains("Invalid string-list-map value")
        );
    }

    // Defends: --with writes an ephemeral settings.jsonc snapshot and validates it through the normal config contract without mutating the user's config.
    #[test]
    fn session_config_overrides_materialize_valid_ephemeral_settings() {
        let runtime = TempDir::new().unwrap();
        write_runtime_layout(runtime.path());
        let config = TempDir::new().unwrap();
        let state = TempDir::new().unwrap();

        let session_config = materialize_session_config_override(
            runtime.path(),
            config.path(),
            state.path(),
            None,
            &[
                "editor.command=nvim".to_string(),
                "core.welcome_style=static".to_string(),
                "zellij.pane_frames=false".to_string(),
                "terminal.transparency=high".to_string(),
            ],
        )
        .unwrap();

        let session_path = Path::new(&session_config);
        assert_eq!(
            session_path.file_name().and_then(|name| name.to_str()),
            Some(crate::user_config_paths::SETTINGS_CONFIG)
        );
        assert!(session_path.starts_with(state.path()));

        let session_value = read_settings_jsonc_value(session_path).unwrap();
        assert_eq!(session_value["editor"]["command"], "nvim");
        assert_eq!(session_value["core"]["welcome_style"], "static");
        assert_eq!(session_value["zellij"]["pane_frames"], false);
        assert_eq!(session_value["terminal"]["transparency"], "high");

        let user_value = read_settings_jsonc_value(&config.path().join("settings.jsonc")).unwrap();
        assert_ne!(user_value["editor"]["command"], "nvim");

        let normalized = load_normalized_config_for_control(
            runtime.path(),
            config.path(),
            Some(&session_config),
        )
        .unwrap();
        assert_eq!(normalized.get("editor_command").unwrap(), "nvim");
        assert_eq!(normalized.get("welcome_style").unwrap(), "static");
        assert_eq!(normalized.get("zellij_pane_frames").unwrap(), "false");
    }

    // Defends: Mars runtime metadata is accepted as a shipped packaged terminal.
    #[test]
    fn active_terminal_accepts_mars_runtime_variant() {
        let runtime = TempDir::new().unwrap();
        fs::write(runtime.path().join("runtime_variant"), "mars\n").unwrap();

        assert_eq!(
            crate::terminal_variant::active_terminal_from_runtime_dir(runtime.path()).unwrap(),
            "mars"
        );
    }

    // Defends: desktop launch logs use the terminal executable basename, so mars diagnostics can find them reliably.
    #[test]
    fn launch_probe_log_path_uses_command_basename() {
        let state = TempDir::new().unwrap();

        let log =
            get_launch_probe_log_path(state.path(), "/nix/store/test-yazelix/bin/mars").unwrap();

        assert!(log.starts_with(state.path().join("logs/terminal_launch")));
        assert!(
            log.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .starts_with("mars_")
        );
    }

    // Defends: unsupported package terminal metadata fails clearly instead of falling back to another terminal.
    #[test]
    fn active_terminal_rejects_unknown_runtime_variant_metadata() {
        let runtime = TempDir::new().unwrap();
        fs::write(runtime.path().join("runtime_variant"), "warpterm\n").unwrap();

        let error =
            crate::terminal_variant::active_terminal_from_runtime_dir(runtime.path()).unwrap_err();
        assert_eq!(error.code(), "unsupported_terminal_variant");
        assert!(error.message().contains("warpterm"));
    }

    // Invariant: runtime handoffs must not leak old window runtime/session helper env.
    #[test]
    fn restart_launch_clears_stale_runtime_session_and_helper_env() {
        for key in [
            "RIO_CONFIG_HOME",
            "YAZELIX_BOOTSTRAP_RUNTIME_DIR",
            "YAZELIX_CURSOR_COLOR",
            "YAZELIX_CURSOR_DIVIDER",
            "YAZELIX_CURSOR_FAMILY",
            "YAZELIX_CURSOR_NAME",
            "YAZELIX_CURSOR_PRIMARY_COLOR",
            "YAZELIX_CURSOR_SECONDARY_COLOR",
            "YAZELIX_INVOKED_YZX_PATH",
            "YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH",
            "YAZELIX_RUNTIME_DIR",
            "YAZELIX_SESSION_CONFIG_PATH",
            "YAZELIX_SESSION_FACTS_PATH",
            "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT",
            "YAZELIX_STARTUP_PROFILE_SKIP_WELCOME",
            "YAZELIX_STATUS_BAR_CACHE_PATH",
            "MARS_APP_ID",
            "MARS_CONFIG",
            "MARS_CONFIG_HOME",
            "MARS_APPEARANCE",
            "MARS_CHILD_ENV_SANITIZE",
            "MARS_EFFECTS",
            "MARS_EMOJI_FONT",
            "MARS_GRAPHICS_WRAPPER",
            "MARS_PROFILE",
            "MARS_RENDER_STRATEGY",
            MARS_EMOJI_ENV_KEYS[0],
            MARS_EMOJI_ENV_KEYS[1],
            "YAZELIX_YZX_BIN",
            "YAZELIX_YZX_CONTROL_BIN",
            "YAZELIX_YZX_CORE_BIN",
        ] {
            assert!(
                RUNTIME_RELAUNCH_CLEARED_ENV_KEYS.contains(&key),
                "runtime handoff must clear stale {key}"
            );
        }
    }

    // Regression: restart must prefer the current packaged runtime launcher instead of the stable Home Manager wrapper.
    #[test]
    fn current_runtime_yzx_launcher_prefers_runtime_bin_yzx() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let bin_yzx = runtime_dir.join("bin/yzx");
        fs::create_dir_all(bin_yzx.parent().unwrap()).unwrap();
        fs::write(&bin_yzx, "#!/bin/sh\n").unwrap();

        assert_eq!(current_runtime_yzx_launcher(&runtime_dir), bin_yzx);
    }

    // Defends: source checkouts without a packaged bin wrapper can still use the POSIX runtime CLI helper.
    #[test]
    fn current_runtime_yzx_launcher_falls_back_to_posix_helper() {
        let runtime_dir = Path::new("/repo/runtime");

        assert_eq!(
            current_runtime_yzx_launcher(runtime_dir),
            PathBuf::from("/repo/runtime/shells/posix/yzx_cli.sh")
        );
    }

    // Defends: desktop entry rendering keeps a quoted launcher path and terminal-backed starter window so spaces do not corrupt the Exec owner surface and pre-terminal failures stay visible.
    #[test]
    fn render_desktop_entry_quotes_exec_path() {
        let entry = render_desktop_entry(Path::new("/tmp/with space/yzx"), "mars");
        assert!(entry.contains("Exec=\"/tmp/with space/yzx\" desktop launch"));
        assert!(entry.contains("Name=New Yazelix - Mars"));
        assert!(entry.contains("Terminal=true"));
    }

    // Regression: desktop launch schedules the real terminal only after the desktop-launch parent exits.
    #[test]
    fn desktop_deferred_launch_helper_records_lifetime_status() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap();
        let helper = desktop_deferred_launch_probe_path(repo_root);
        let tmp = TempDir::new().unwrap();
        let launch_log = tmp.path().join("deferred.log");
        let marker = tmp.path().join("marker");

        let output = Command::new(&helper)
            .arg(&launch_log)
            .arg("999999999")
            .arg("--")
            .arg("sh")
            .arg("-c")
            .arg(format!("printf done > {}", marker.display()))
            .output()
            .unwrap();

        assert!(output.status.success());
        assert_eq!(
            String::from_utf8_lossy(&output.stdout).trim(),
            launch_log.display().to_string()
        );

        let mut launched = false;
        for _ in 0..20 {
            if marker.is_file() {
                launched = true;
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        assert!(
            launched,
            "deferred desktop helper did not launch scheduled command"
        );

        let mut raw_log = std::fs::read_to_string(&launch_log).unwrap();
        for _ in 0..40 {
            if raw_log.contains("final_exit_status=") {
                break;
            }
            thread::sleep(Duration::from_millis(50));
            raw_log = std::fs::read_to_string(&launch_log).unwrap();
        }
        assert!(raw_log.contains("desktop deferred launch"));
        assert!(raw_log.contains("argv:"));
        assert!(raw_log.contains("terminal_or_wrapper_pid="));
        assert!(raw_log.contains("early_exit_status=0"));
        assert!(raw_log.contains("final_exit_status=0"));
        assert!(raw_log.contains("final_exit_kind=exit"));
        assert!(raw_log.contains("final_exit_code=0"));
    }

    // Defends: macOS preview desktop parsing keeps the opt-in nested action explicit.
    #[test]
    fn parse_desktop_args_accepts_macos_preview_action() {
        let parsed = parse_desktop_args(&[
            "macos_preview".into(),
            "install".into(),
            "--print-path".into(),
        ])
        .unwrap();

        assert_eq!(parsed.subcommand.as_deref(), Some("macos_preview"));
        assert_eq!(parsed.action.as_deref(), Some("install"));
        assert!(parsed.print_path);
    }

    // Defends: the macOS preview app bundle points at a stable package profile wrapper and reports actionable package-first repair steps.
    #[test]
    fn render_macos_preview_launcher_uses_profile_yzx_and_actionable_failures() {
        let script =
            render_macos_preview_launcher_script(Path::new("/Users/demo/.nix-profile/bin/yzx"));

        assert!(script.contains("YAZELIX_STABLE_YZX='/Users/demo/.nix-profile/bin/yzx'"));
        assert!(script.contains("\"$YAZELIX_STABLE_YZX\" desktop launch"));
        assert!(script.contains("yzx doctor --verbose"));
        assert!(script.contains("yzx desktop macos_preview install"));
        assert!(!script.contains("/pjs/yazelix"));
    }

    // Defends: the macOS preview bundle carries owned app metadata instead of looking like a throwaway script bundle.
    #[test]
    fn render_macos_preview_info_plist_carries_owned_app_metadata() {
        let info = render_macos_preview_info_plist();

        assert!(info.contains("<key>CFBundlePackageType</key>"));
        assert!(info.contains("<string>APPL</string>"));
        assert!(info.contains("<key>CFBundleShortVersionString</key>"));
        assert!(info.contains(&format!(
            "<string>{MACOS_PREVIEW_BUNDLE_SHORT_VERSION}</string>"
        )));
        assert!(info.contains("<key>CFBundleVersion</key>"));
        assert!(info.contains(&format!("<string>{MACOS_PREVIEW_BUNDLE_VERSION}</string>")));
        assert!(info.contains("<key>LSApplicationCategoryType</key>"));
        assert!(info.contains("<string>public.app-category.developer-tools</string>"));
        assert!(info.contains("<key>NSHighResolutionCapable</key>"));
    }

    // Defends: the macOS preview installer creates only a Yazelix-marked app bundle with a profile-owned launcher script.
    #[test]
    fn install_macos_preview_app_writes_managed_bundle() {
        let tmp = TempDir::new().unwrap();
        let app_path = tmp
            .path()
            .join("Applications")
            .join(MACOS_PREVIEW_APP_DIR_NAME);
        let launcher_path = tmp.path().join(".nix-profile").join("bin").join("yzx");

        install_macos_preview_app(&app_path, &launcher_path).unwrap();

        let info = fs::read_to_string(app_path.join("Contents").join("Info.plist")).unwrap();
        let marker = app_path
            .join("Contents")
            .join("Resources")
            .join(MACOS_PREVIEW_MARKER_FILE);
        let script = fs::read_to_string(
            app_path
                .join("Contents")
                .join("MacOS")
                .join(MACOS_PREVIEW_EXECUTABLE_NAME),
        )
        .unwrap();

        assert!(info.contains(MACOS_PREVIEW_BUNDLE_ID));
        assert!(marker.is_file());
        assert!(script.contains(&launcher_path.to_string_lossy().to_string()));
        assert!(macos_preview_bundle_is_managed(&app_path));
    }

    // Regression: uninstall and refresh paths must not take ownership of an unrelated app bundle at the preview path.
    #[test]
    fn macos_preview_bundle_guard_rejects_unmarked_app_path() {
        let tmp = TempDir::new().unwrap();
        let app_path = tmp
            .path()
            .join("Applications")
            .join(MACOS_PREVIEW_APP_DIR_NAME);
        fs::create_dir_all(app_path.join("Contents")).unwrap();
        fs::write(
            app_path.join("Contents").join("Info.plist"),
            render_macos_preview_info_plist(),
        )
        .unwrap();

        let err = ensure_macos_preview_bundle_is_managed(&app_path).unwrap_err();
        assert_eq!(err.code(), "macos_preview_app_conflict");
    }
}
