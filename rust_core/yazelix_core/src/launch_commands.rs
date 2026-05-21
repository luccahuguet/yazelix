// Test lane: default
//! Rust-owned `yzx enter`, `yzx launch`, `yzx desktop`, and `yzx restart` owners.

use crate::bridge::CoreError;
use crate::control_plane::home_dir_from_env;
use std::path::PathBuf;

mod config_override;
mod desktop;
mod enter;
mod launch;
mod process;
mod restart;
mod terminal;

use desktop::*;

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
    use super::desktop::*;
    use super::process::*;
    use super::restart::*;
    use super::terminal::*;
    use super::*;
    use crate::control_plane::load_normalized_config_for_control;
    use crate::settings_surface::read_settings_jsonc_value;
    use serde_json::Map as JsonMap;
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
            include_str!("../../../yazelix_ghostty_cursors_default.toml"),
        )
        .expect("cursor defaults");
        fs::write(
            runtime.join(crate::active_config_surface::TOML_TOOLING_CONFIG_FILENAME),
            include_str!("../../../tombi.toml"),
        )
        .expect("tombi config");
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
                "editor.sidebar_width_percent".to_string(),
                SessionConfigOverrideField {
                    kind: SessionConfigOverrideKind::Int,
                },
            ),
            (
                "terminal.terminals".to_string(),
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
            "terminal": { "terminals": ["ghostty"] },
            "zellij": { "keybindings": { "bottom_popup": ["Alt p"] } }
        });

        for raw in [
            "editor.command=nvim",
            "core.skip_welcome_screen=true",
            "core.welcome_duration_seconds=3.5",
            "editor.sidebar_width_percent=24",
            "terminal.terminals=[\"wezterm\", \"kitty\"]",
            "zellij.keybindings={\"bottom_popup\":[\"Alt Shift J\"],\"config\":[]}",
        ] {
            let patch = parse_session_config_patch(raw, &fields).unwrap();
            apply_session_config_patch(&mut root, &patch).unwrap();
        }

        assert_eq!(root["editor"]["command"], "nvim");
        assert_eq!(root["core"]["skip_welcome_screen"], true);
        assert_eq!(root["core"]["welcome_duration_seconds"], 3.5);
        assert_eq!(root["editor"]["sidebar_width_percent"], 24);
        assert_eq!(
            root["terminal"]["terminals"],
            serde_json::json!(["wezterm", "kitty"])
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
                "terminal.terminals=[\"wezterm\"]".to_string(),
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
        assert_eq!(
            session_value["terminal"]["terminals"],
            serde_json::json!(["wezterm"])
        );

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

    // Defends: the Rust launch owner still filters duplicate or unsupported configured terminals before fallback logic runs.
    #[test]
    fn normalized_configured_terminals_filters_and_dedupes() {
        let mut config = JsonMap::new();
        config.insert(
            "terminals".into(),
            serde_json::json!(["ghostty", "", "warp", "ghostty", "kitty"]),
        );

        assert_eq!(
            normalized_configured_terminals(&config),
            vec!["ghostty".to_string(), "kitty".to_string()]
        );
    }

    // Defends: missing terminal config keeps Ghostty as the default while preserving WezTerm as the first fallback.
    #[test]
    fn normalized_configured_terminals_defaults_to_ghostty_then_wezterm() {
        let config = JsonMap::new();

        assert_eq!(
            normalized_configured_terminals(&config),
            vec!["ghostty".to_string(), "wezterm".to_string()]
        );
    }

    // Defends: Ghostty user-mode config discovery follows upstream file-name and macOS path candidates instead of hard-coding the old config name.
    #[test]
    fn ghostty_user_config_candidates_follow_upstream_paths() {
        let home = Path::new("/Users/demo");
        let xdg = Path::new("/Users/demo/.config");

        assert_eq!(
            ghostty_user_config_candidates(home, xdg, "linux"),
            vec![
                PathBuf::from("/Users/demo/.config/ghostty/config.ghostty"),
                PathBuf::from("/Users/demo/.config/ghostty/config"),
            ]
        );
        assert_eq!(
            ghostty_user_config_candidates(home, xdg, "macos"),
            vec![
                PathBuf::from("/Users/demo/.config/ghostty/config.ghostty"),
                PathBuf::from("/Users/demo/.config/ghostty/config"),
                PathBuf::from(
                    "/Users/demo/Library/Application Support/com.mitchellh.ghostty/config.ghostty",
                ),
                PathBuf::from(
                    "/Users/demo/Library/Application Support/com.mitchellh.ghostty/config",
                ),
            ]
        );
    }

    // Regression: terminal.config_mode=user accepts Ghostty's current config.ghostty name and reports every checked candidate when none exists.
    #[test]
    fn ghostty_user_config_selection_accepts_config_ghostty_and_lists_misses() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg = home.join(".config");
        let ghostty_dir = xdg.join("ghostty");
        fs::create_dir_all(&ghostty_dir).unwrap();
        let current = ghostty_dir.join("config.ghostty");
        fs::write(&current, "font-family = PragmataPro Mono Liga\n").unwrap();
        let candidates =
            user_terminal_config_candidates_for_platform(&home, "ghostty", &xdg, "macos").unwrap();

        assert_eq!(
            select_existing_user_terminal_config_path("ghostty", &candidates).unwrap(),
            current
        );

        fs::remove_file(&current).unwrap();
        let missing =
            select_existing_user_terminal_config_path("ghostty", &candidates).unwrap_err();
        assert!(missing.contains("config.ghostty"));
        assert!(missing.contains("Application Support/com.mitchellh.ghostty/config"));
    }

    // Invariant: restart must relaunch through the stable owner without leaking old window runtime/session helper env.
    #[test]
    fn restart_launch_clears_stale_runtime_session_and_helper_env() {
        for key in [
            "YAZELIX_BOOTSTRAP_RUNTIME_DIR",
            "YAZELIX_CURSOR_COLOR",
            "YAZELIX_CURSOR_DIVIDER",
            "YAZELIX_CURSOR_FAMILY",
            "YAZELIX_CURSOR_NAME",
            "YAZELIX_CURSOR_PRIMARY_COLOR",
            "YAZELIX_CURSOR_SECONDARY_COLOR",
            "YAZELIX_RUNTIME_DIR",
            "YAZELIX_SESSION_CONFIG_PATH",
            "YAZELIX_SESSION_FACTS_PATH",
            "YAZELIX_STARTUP_PROFILE_SKIP_WELCOME",
            "YAZELIX_STATUS_BAR_CACHE_PATH",
            "YAZELIX_YZX_BIN",
            "YAZELIX_YZX_CONTROL_BIN",
            "YAZELIX_YZX_CORE_BIN",
        ] {
            assert!(
                RESTART_LAUNCH_CLEARED_ENV_KEYS.contains(&key),
                "restart launch must clear stale {key}"
            );
        }
    }

    // Defends: desktop entry rendering keeps a quoted launcher path so spaces do not corrupt the Exec owner surface.
    #[test]
    fn render_desktop_entry_quotes_exec_path() {
        let entry = render_desktop_entry(Path::new("/tmp/with space/yzx"));
        assert!(entry.contains("Exec=\"/tmp/with space/yzx\" desktop launch"));
        assert!(entry.contains("Terminal=false"));
    }

    // Regression: desktop launch schedules the real terminal only after the desktop-launch parent exits.
    #[test]
    fn desktop_deferred_launch_helper_schedules_after_starter_parent_exits() {
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

        for _ in 0..20 {
            if marker.is_file() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("deferred desktop helper did not launch scheduled command");
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
