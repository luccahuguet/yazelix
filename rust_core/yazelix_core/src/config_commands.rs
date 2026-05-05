// Test lane: default
//! `yzx config` family implemented in Rust for `yzx_control`.

use crate::active_config_surface::ActiveConfigPaths;
use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::config_ui::{ConfigUiRequest, run_config_ui};
use crate::control_plane::{config_dir_from_env, config_override_from_env, runtime_dir_from_env};
use crate::settings_jsonc_patch::{
    SettingsJsoncPatchMutation, set_settings_jsonc_value_text, unset_settings_jsonc_value_text,
};
use crate::settings_surface::{is_settings_config_path, parse_jsonc_value};
use serde_json::{Value as JsonValue, json};
use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const HOME_MANAGER_FILES_MARKER: &str = "-home-manager-files/";

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
    println!("  set             Set a settings.jsonc value using a JSON literal");
    println!("  unset           Remove an explicit settings.jsonc value");
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
    let raw = read_config_for_edit(&paths.config_file)?;
    let outcome = set_settings_jsonc_value_text(&paths.config_file, &raw, setting_path, &value)?;
    if outcome.changed() {
        validate_patched_settings(&paths, &outcome.text)?;
    }
    write_config_edit(&paths.config_file, &outcome.text, outcome.mutation)?;
    print_edit_outcome(setting_path, outcome.mutation);
    Ok(0)
}

fn run_config_unset(setting_path: &str) -> Result<i32, CoreError> {
    let paths = resolve_editable_settings_path()?;
    let raw = read_config_for_edit(&paths.config_file)?;
    let outcome = unset_settings_jsonc_value_text(&paths.config_file, &raw, setting_path)?;
    if outcome.changed() {
        validate_patched_settings(&paths, &outcome.text)?;
    }
    write_config_edit(&paths.config_file, &outcome.text, outcome.mutation)?;
    print_edit_outcome(setting_path, outcome.mutation);
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
    if is_home_manager_owned_path(&paths.config_file) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "home_manager_owned_config",
            "The active Yazelix settings file is owned by Home Manager.",
            "Edit your Home Manager module options instead, then run home-manager switch.",
            json!({ "path": paths.config_file.display().to_string() }),
        ));
    }
    if config_path_is_read_only(&paths.config_file) {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "read_only_settings_config",
            format!(
                "The active Yazelix settings file is read-only: {}.",
                paths.config_file.display()
            ),
            "Fix file permissions or edit the owning configuration source.",
            json!({ "path": paths.config_file.display().to_string() }),
        ));
    }
    Ok(paths)
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

fn print_edit_outcome(setting_path: &str, mutation: SettingsJsoncPatchMutation) {
    match mutation {
        SettingsJsoncPatchMutation::Inserted => println!("Inserted {setting_path}."),
        SettingsJsoncPatchMutation::Replaced => println!("Updated {setting_path}."),
        SettingsJsoncPatchMutation::Removed => println!("Removed {setting_path}."),
        SettingsJsoncPatchMutation::Unchanged => println!("{setting_path} was already unset."),
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
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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
