// Test lane: default
//! `yzx config` family implemented in Rust for `yzx_control`.

use crate::active_config_surface::ActiveConfigPaths;
use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::config_apply::{
    ConfigEditApplyRequest, ConfigEditApplyStatus, PaneOrchestratorRuntimeRefreshRequest,
    apply_mode_for_setting, apply_status_after_config_edit,
};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::config_ui::{ConfigUiRequest, run_config_ui};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, runtime_dir_from_env,
    runtime_materialization_plan_request_from_env, state_dir_from_env,
};
use crate::settings_jsonc_patch::{
    SettingsJsoncPatchMutation, set_settings_jsonc_value_text, unset_settings_jsonc_value_text,
};
use crate::settings_surface::{is_settings_config_path, parse_jsonc_value};
use serde_json::{Value as JsonValue, json};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use yazelix_ghostty_cursors::{CursorRegistry, render_cursor_settings_jsonc};

const HOME_MANAGER_FILES_MARKER: &str = "-home-manager-files/";

#[derive(Debug, Clone)]
struct ConfigEditTarget {
    path: PathBuf,
    path_in_file: String,
    kind: ConfigEditTargetKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigEditTargetKind {
    Main,
    Cursors,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ConfigArgs {
    action: ConfigAction,
    print_path: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum ConfigAction {
    #[default]
    Show,
    Ui,
    Set {
        path: String,
        value: String,
    },
    Unset {
        path: String,
    },
}

pub fn run_yzx_config(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_config_args(args)?;
    if parsed.help {
        print_config_help();
        return Ok(0);
    }

    match parsed.action {
        ConfigAction::Show => run_config_show(parsed.print_path),
        ConfigAction::Ui => run_config_ui_from_env(),
        ConfigAction::Set { path, value } => run_config_set(&path, &value),
        ConfigAction::Unset { path } => run_config_unset(&path),
    }
}

fn parse_config_args(args: &[String]) -> Result<ConfigArgs, CoreError> {
    if args.is_empty() {
        return Ok(ConfigArgs::default());
    }

    match args[0].as_str() {
        "set" => {
            if args.len() != 3 {
                return Err(CoreError::usage(
                    "Usage: yzx config set <settings.path> <json-value>",
                ));
            }
            return Ok(ConfigArgs {
                action: ConfigAction::Set {
                    path: args[1].clone(),
                    value: args[2].clone(),
                },
                print_path: false,
                help: false,
            });
        }
        "unset" => {
            if args.len() != 2 {
                return Err(CoreError::usage("Usage: yzx config unset <settings.path>"));
            }
            return Ok(ConfigArgs {
                action: ConfigAction::Unset {
                    path: args[1].clone(),
                },
                print_path: false,
                help: false,
            });
        }
        _ => {}
    }

    let mut parsed = ConfigArgs::default();
    for arg in args {
        match arg.as_str() {
            "ui" if parsed.action == ConfigAction::Show && !parsed.print_path => {
                parsed.action = ConfigAction::Ui;
            }
            "--path" if parsed.action == ConfigAction::Show => parsed.print_path = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx config: {other}. Try `yzx config --help`."
                )));
            }
        }
    }

    Ok(parsed)
}

fn print_config_help() {
    println!("Show the active Yazelix configuration");
    println!();
    println!("Usage:");
    println!("  yzx config [--path]");
    println!("  yzx config ui");
    println!("  yzx config set <settings.path> <json-value>");
    println!("  yzx config unset <settings.path>");
    println!();
    println!("Flags:");
    println!("      --path       Print the resolved config path");
    println!();
    println!("Subcommands:");
    println!("  ui              Open the config browser");
    println!("  set             Set a supported config value using a JSON literal");
    println!("  unset           Remove an explicit config value");
}

fn io_err(path: &Path, source: io::Error) -> CoreError {
    CoreError::io(
        "config_io",
        format!(
            "Could not access the Yazelix config path {}.",
            path.display()
        ),
        "Fix permissions or restore the missing path, then retry.",
        path.display().to_string(),
        source,
    )
}

fn render_config_text(path: &Path) -> Result<String, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| io_err(path, source))?;
    if is_settings_config_path(path) {
        parse_jsonc_value(path, &raw)?;
        return Ok(raw);
    }

    toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_config_surface",
            format!(
                "Could not parse the active Yazelix config at {}.",
                path.display()
            ),
            "Fix the config syntax or run `yzx reset config` to restore the managed template.",
            path.display().to_string(),
            source,
        )
    })?;
    Ok(raw)
}

fn print_text_with_trailing_newline(text: &str) {
    print!("{text}");
    if !text.ends_with('\n') {
        println!();
    }
}

fn run_config_show(print_path: bool) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let paths = resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;

    if print_path {
        println!("{}", paths.config_file.display());
        return Ok(0);
    }

    let rendered = render_config_text(&paths.config_file)?;
    print_text_with_trailing_newline(&rendered);
    Ok(0)
}

fn run_config_ui_from_env() -> Result<i32, CoreError> {
    run_config_ui(ConfigUiRequest {
        runtime_dir: runtime_dir_from_env()?,
        config_dir: config_dir_from_env()?,
        config_override: config_override_from_env(),
    })
}

fn run_config_set(setting_path: &str, raw_value: &str) -> Result<i32, CoreError> {
    let value = serde_json::from_str::<JsonValue>(raw_value).map_err(|source| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_config_json_value",
            format!("Could not parse config value for {setting_path} as JSON."),
            "Pass a JSON literal, for example true, 20, \"bash\", or [\"ghostty\"].",
            json!({
                "path": setting_path,
                "value": raw_value,
                "error": source.to_string(),
            }),
        )
    })?;
    let paths = resolve_editable_settings_path()?;
    let target = edit_target(&paths, setting_path);
    ensure_edit_target_writable(&target)?;
    let raw = read_config_for_edit_or_default(&paths, &target)?;
    let outcome = set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, &value)?;
    if outcome.changed() {
        validate_patched_edit_target(&paths, &target, &outcome.text)?;
    }
    write_config_edit(&target.path, &outcome.text, outcome.mutation)?;
    let apply_status =
        apply_after_config_edit(setting_path, outcome.mutation, &paths.contract_path)?;
    print_edit_outcome(setting_path, outcome.mutation, apply_status.as_ref());
    Ok(0)
}

fn run_config_unset(setting_path: &str) -> Result<i32, CoreError> {
    let paths = resolve_editable_settings_path()?;
    let target = edit_target(&paths, setting_path);
    ensure_edit_target_writable(&target)?;
    let raw = read_config_for_edit_or_default(&paths, &target)?;
    let outcome = unset_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file)?;
    if outcome.changed() {
        validate_patched_edit_target(&paths, &target, &outcome.text)?;
    }
    write_config_edit(&target.path, &outcome.text, outcome.mutation)?;
    let apply_status =
        apply_after_config_edit(setting_path, outcome.mutation, &paths.contract_path)?;
    print_edit_outcome(setting_path, outcome.mutation, apply_status.as_ref());
    Ok(0)
}

fn resolve_editable_settings_path() -> Result<ActiveConfigPaths, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let paths = resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;
    if !is_settings_config_path(&paths.config_file) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "unsupported_config_edit_surface",
            format!(
                "Yazelix can only edit settings.jsonc, but the active config is {}.",
                paths.config_file.display()
            ),
            "Move this setting to the canonical settings.jsonc surface, or clear YAZELIX_CONFIG_OVERRIDE.",
            json!({ "path": paths.config_file.display().to_string() }),
        ));
    }
    Ok(paths)
}

fn edit_target(paths: &ActiveConfigPaths, setting_path: &str) -> ConfigEditTarget {
    if let Some(cursor_path) = setting_path.strip_prefix("cursors.") {
        ConfigEditTarget {
            path: paths.user_cursor_config.clone(),
            path_in_file: cursor_path.to_string(),
            kind: ConfigEditTargetKind::Cursors,
        }
    } else {
        ConfigEditTarget {
            path: paths.config_file.clone(),
            path_in_file: setting_path.to_string(),
            kind: ConfigEditTargetKind::Main,
        }
    }
}

fn ensure_edit_target_writable(target: &ConfigEditTarget) -> Result<(), CoreError> {
    if !is_settings_config_path(&target.path) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "unsupported_config_edit_surface",
            format!(
                "Yazelix can only edit settings.jsonc, but the active config is {}.",
                target.path.display()
            ),
            "Move this setting to the canonical settings.jsonc surface, or clear YAZELIX_CONFIG_OVERRIDE.",
            json!({ "path": target.path.display().to_string() }),
        ));
    }
    if is_home_manager_owned_path(&target.path) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "home_manager_owned_config",
            "The active Yazelix settings file is owned by Home Manager.",
            "Edit your Home Manager module options instead, then run home-manager switch.",
            json!({ "path": target.path.display().to_string() }),
        ));
    }
    if config_path_is_read_only(&target.path) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "read_only_settings_config",
            format!(
                "The active Yazelix settings file is read-only: {}.",
                target.path.display()
            ),
            "Fix file permissions or edit the owning configuration source.",
            json!({ "path": target.path.display().to_string() }),
        ));
    }
    Ok(())
}

fn read_config_for_edit(path: &Path) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_settings_jsonc_for_edit",
            "Could not read Yazelix settings.jsonc for editing",
            "Fix permissions or restore the settings file, then retry.",
            path.display().to_string(),
            source,
        )
    })
}

fn read_config_for_edit_or_default(
    paths: &ActiveConfigPaths,
    target: &ConfigEditTarget,
) -> Result<String, CoreError> {
    if target.path.exists() {
        return read_config_for_edit(&target.path);
    }
    match target.kind {
        ConfigEditTargetKind::Main => read_config_for_edit(&target.path),
        ConfigEditTargetKind::Cursors => {
            let raw = fs::read_to_string(&paths.default_cursor_config_path).map_err(|source| {
                CoreError::io(
                    "read_default_cursor_config_for_edit",
                    "Could not read the default Yazelix cursor settings",
                    "Reinstall Yazelix so the runtime includes yazelix_ghostty_cursors_default.toml.",
                    paths.default_cursor_config_path.display().to_string(),
                    source,
                )
            })?;
            let registry = CursorRegistry::parse_str(&paths.default_cursor_config_path, &raw)?;
            Ok(render_cursor_settings_jsonc(&registry))
        }
    }
}

fn validate_patched_edit_target(
    paths: &ActiveConfigPaths,
    target: &ConfigEditTarget,
    raw: &str,
) -> Result<(), CoreError> {
    match target.kind {
        ConfigEditTargetKind::Main => validate_patched_settings(paths, raw),
        ConfigEditTargetKind::Cursors => {
            let value = parse_jsonc_value(&target.path, raw)?;
            CursorRegistry::parse_json_value(&target.path, value)?;
            Ok(())
        }
    }
}

fn validate_patched_settings(paths: &ActiveConfigPaths, raw: &str) -> Result<(), CoreError> {
    let temp_dir = std::env::temp_dir().join(format!(
        "yazelix_settings_check_{}_{}",
        std::process::id(),
        monotonic_suffix()
    ));
    fs::create_dir_all(&temp_dir).map_err(|source| {
        CoreError::io(
            "create_settings_validation_temp_dir",
            "Could not create a temporary directory to validate settings.jsonc",
            "Check the system temporary directory permissions, then retry.",
            temp_dir.display().to_string(),
            source,
        )
    })?;
    let temp_config = temp_dir.join("settings.jsonc");
    let result = (|| {
        fs::write(&temp_config, raw).map_err(|source| {
            CoreError::io(
                "write_settings_validation_temp_config",
                "Could not write a temporary settings.jsonc validation file",
                "Check the system temporary directory permissions, then retry.",
                temp_config.display().to_string(),
                source,
            )
        })?;
        normalize_config(&NormalizeConfigRequest {
            config_path: temp_config,
            default_config_path: paths.default_config_path.clone(),
            contract_path: paths.contract_path.clone(),
            include_missing: true,
        })?;
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_dir);
    result
}

fn monotonic_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn write_config_edit(
    path: &Path,
    raw: &str,
    mutation: SettingsJsoncPatchMutation,
) -> Result<(), CoreError> {
    if mutation == SettingsJsoncPatchMutation::Unchanged {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "create_settings_jsonc_parent",
                "Could not create the Yazelix config directory",
                "Fix permissions for the config directory, then retry.",
                parent.display().to_string(),
                source,
            )
        })?;
    }
    fs::write(path, raw).map_err(|source| {
        CoreError::io(
            "write_settings_jsonc_edit",
            "Could not write Yazelix settings.jsonc",
            "Fix permissions for the settings file, then retry.",
            path.display().to_string(),
            source,
        )
    })
}

fn apply_after_config_edit(
    setting_path: &str,
    mutation: SettingsJsoncPatchMutation,
    contract_path: &Path,
) -> Result<Option<ConfigEditApplyStatus>, CoreError> {
    if mutation == SettingsJsoncPatchMutation::Unchanged {
        return Ok(None);
    }
    let apply_mode = apply_mode_for_setting(contract_path, setting_path)?;
    let runtime_materialization = if apply_mode
        == Some(crate::runtime_apply_mode::RuntimeApplyMode::GeneratedRuntimeRefresh)
    {
        Some(runtime_materialization_plan_request_from_env(
            config_override_from_env().as_deref(),
        )?)
    } else {
        None
    };
    let pane_orchestrator_refresh =
        if apply_mode == Some(crate::runtime_apply_mode::RuntimeApplyMode::LiveWithPaneRefresh) {
            let runtime_dir = runtime_dir_from_env()?;
            let config_dir = config_dir_from_env()?;
            let config_override = config_override_from_env();
            let paths =
                resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;
            let state_dir = state_dir_from_env()?;
            Some(PaneOrchestratorRuntimeRefreshRequest {
                config_path: paths.config_file,
                default_config_path: paths.default_config_path,
                contract_path: paths.contract_path,
                zellij_config_dir: state_dir.join("configs").join("zellij"),
            })
        } else {
            None
        };
    Ok(Some(apply_status_after_config_edit(
        &ConfigEditApplyRequest {
            setting_path: setting_path.to_string(),
            contract_path: contract_path.to_path_buf(),
            runtime_materialization,
            pane_orchestrator_refresh,
        },
    )?))
}

fn print_edit_outcome(
    setting_path: &str,
    mutation: SettingsJsoncPatchMutation,
    apply_status: Option<&ConfigEditApplyStatus>,
) {
    match mutation {
        SettingsJsoncPatchMutation::Inserted => println!("Inserted {setting_path}."),
        SettingsJsoncPatchMutation::Replaced => println!("Updated {setting_path}."),
        SettingsJsoncPatchMutation::Removed => println!("Removed {setting_path}."),
        SettingsJsoncPatchMutation::Unchanged => println!("{setting_path} was already unset."),
    }
    if let Some(status) = apply_status {
        println!("Apply: {}.", status.apply_mode.label());
        if let Some(refresh) = &status.generated_refresh {
            println!("{}", refresh.message);
            println!("{}", refresh.remediation);
        }
        if let Some(refresh) = &status.pane_orchestrator_refresh {
            println!("{}", refresh.message);
            println!("{}", refresh.remediation);
        }
    }
}

fn is_home_manager_owned_path(path: &Path) -> bool {
    fs::read_link(path)
        .ok()
        .map(|target| target.to_string_lossy().contains(HOME_MANAGER_FILES_MARKER))
        .unwrap_or(false)
}

fn config_path_is_read_only(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.permissions().readonly())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the Rust-owned `yzx config` parser keeps the public `--path` switch while rejecting unexpected tokens.
    #[test]
    fn parses_config_root_flags() {
        assert_eq!(
            parse_config_args(&["--path".into()]).unwrap(),
            ConfigArgs {
                action: ConfigAction::Show,
                print_path: true,
                help: false,
            }
        );
        assert_eq!(
            parse_config_args(&["--help".into()]).unwrap(),
            ConfigArgs {
                action: ConfigAction::Show,
                print_path: false,
                help: true,
            }
        );
        assert_eq!(
            parse_config_args(&["ui".into()]).unwrap(),
            ConfigArgs {
                action: ConfigAction::Ui,
                print_path: false,
                help: false,
            }
        );
        assert_eq!(
            parse_config_args(&["set".into(), "core.debug_mode".into(), "true".into()]).unwrap(),
            ConfigArgs {
                action: ConfigAction::Set {
                    path: "core.debug_mode".to_string(),
                    value: "true".to_string(),
                },
                print_path: false,
                help: false,
            }
        );
        assert_eq!(
            parse_config_args(&["unset".into(), "core.debug_mode".into()]).unwrap(),
            ConfigArgs {
                action: ConfigAction::Unset {
                    path: "core.debug_mode".to_string(),
                },
                print_path: false,
                help: false,
            }
        );
        assert!(parse_config_args(&["--force".into()]).is_err());
        assert!(parse_config_args(&["ui".into(), "--path".into()]).is_err());
        assert!(parse_config_args(&["set".into(), "core.debug_mode".into()]).is_err());
    }
}
