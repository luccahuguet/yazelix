// Test lane: default
//! `yzx config` family implemented in Rust for `yzx_control`.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::CoreError;
use crate::config_ui::{ConfigUiRequest, run_config_ui};
use crate::control_plane::{config_dir_from_env, config_override_from_env, runtime_dir_from_env};
use crate::settings_surface::{is_settings_config_path, parse_jsonc_value};
use std::fs;
use std::io;
use std::path::Path;

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
    }
}

fn parse_config_args(args: &[String]) -> Result<ConfigArgs, CoreError> {
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
    println!();
    println!("Flags:");
    println!("      --path       Print the resolved config path");
    println!();
    println!("Subcommands:");
    println!("  ui              Open the read-only config browser");
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
        assert!(parse_config_args(&["--force".into()]).is_err());
        assert!(parse_config_args(&["ui".into(), "--path".into()]).is_err());
    }
}
