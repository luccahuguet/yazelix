// Test lane: default
//! `yzx reset` family implemented in Rust for `yzx_control`.

use crate::active_config_surface::primary_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, runtime_dir_from_env};
use crate::settings_surface::{render_default_settings_jsonc, replace_cursor_settings_in_jsonc};
use serde_json::json;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResetArgs {
    yes: bool,
    no_backup: bool,
    help: bool,
}

#[derive(Debug, Clone)]
struct ResetSurface {
    command: &'static str,
    display_name: &'static str,
    target_file_name: &'static str,
    default_path: PathBuf,
    target_path: PathBuf,
    missing_default_code: &'static str,
    missing_default_remediation: &'static str,
}

pub fn run_yzx_reset(args: &[String]) -> Result<i32, CoreError> {
    match args.first().map(String::as_str) {
        None | Some("-h" | "--help" | "help") => {
            print_reset_help();
            Ok(0)
        }
        Some("config") => run_reset_config(&args[1..]),
        Some("cursor") => run_reset_cursor(&args[1..]),
        Some(other) => Err(CoreError::usage(format!(
            "Unknown reset target for yzx reset: {other}. Try `yzx reset --help`."
        ))),
    }
}

fn run_reset_config(args: &[String]) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = primary_config_paths(&runtime_dir, &config_dir);
    let content = render_default_settings_jsonc(
        &paths.default_config_path,
        &paths.default_cursor_config_path,
    )?;
    reset_surface_with_content(
        args,
        ResetSurface {
            command: "yzx reset config",
            display_name: "main Yazelix config",
            target_file_name: "settings.jsonc",
            default_path: paths.default_config_path,
            target_path: paths.user_config,
            missing_default_code: "missing_default_config",
            missing_default_remediation: "Reinstall Yazelix or restore yazelix_default.toml in the runtime, then retry.",
        },
        content,
    )
}

fn run_reset_cursor(args: &[String]) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = primary_config_paths(&runtime_dir, &config_dir);
    let content = if paths.user_config.exists() {
        replace_cursor_settings_in_jsonc(&paths.user_config, &paths.default_cursor_config_path)?
    } else {
        render_default_settings_jsonc(
            &paths.default_config_path,
            &paths.default_cursor_config_path,
        )?
    };
    reset_surface_with_content(
        args,
        ResetSurface {
            command: "yzx reset cursor",
            display_name: "Ghostty cursor registry",
            target_file_name: "settings.jsonc cursor section",
            default_path: paths.default_cursor_config_path,
            target_path: paths.user_config,
            missing_default_code: "missing_default_cursor_config",
            missing_default_remediation: "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml, then retry.",
        },
        content,
    )
}

fn parse_reset_args(args: &[String], command: &str) -> Result<ResetArgs, CoreError> {
    let mut parsed = ResetArgs::default();

    for arg in args {
        match arg.as_str() {
            "--yes" => parsed.yes = true,
            "--no-backup" => parsed.no_backup = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for {command}: {other}. Try `{command} --help`."
                )));
            }
        }
    }

    Ok(parsed)
}

fn print_reset_help() {
    println!("Reset Yazelix-managed config surfaces");
    println!();
    println!("Usage:");
    println!("  yzx reset config [--yes] [--no-backup]");
    println!("  yzx reset cursor [--yes] [--no-backup]");
    println!();
    println!("Targets:");
    println!("  config  Replace settings.jsonc with fresh shipped settings");
    println!("  cursor  Replace the settings.jsonc cursors section with shipped defaults");
}

fn print_reset_surface_help(surface: &ResetSurface) {
    println!(
        "Replace the {} with a fresh shipped template",
        surface.display_name
    );
    println!();
    println!("Usage:");
    println!("  {} [--yes] [--no-backup]", surface.command);
    println!();
    println!("Flags:");
    println!("      --yes        Skip confirmation prompt");
    println!("      --no-backup  Replace the file without writing a timestamped backup first");
}

fn reset_surface_with_content(
    args: &[String],
    surface: ResetSurface,
    content: String,
) -> Result<i32, CoreError> {
    let parsed = parse_reset_args(args, surface.command)?;
    if parsed.help {
        print_reset_surface_help(&surface);
        return Ok(0);
    }

    let target_exists = surface.target_path.exists();
    let removed_without_backup = parsed.no_backup && target_exists;

    if !surface.default_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            surface.missing_default_code,
            format!(
                "Default {} not found: {}",
                surface.display_name,
                surface.default_path.display()
            ),
            surface.missing_default_remediation,
            json!({ "path": surface.default_path.display().to_string() }),
        ));
    }

    if !parsed.yes {
        println!(
            "⚠️  This replaces {} with a fresh shipped template.",
            surface.target_file_name
        );
        if target_exists && !parsed.no_backup {
            println!(
                "   Your current {} will be backed up first.",
                surface.target_file_name
            );
        }
        if target_exists && parsed.no_backup {
            println!(
                "   Your current {} will be removed without a backup.",
                surface.target_file_name
            );
        }
        print!("Continue? [y/N]: ");
        let confirm = read_confirmation();
        if confirm != "y" && confirm != "yes" {
            println!("Aborted.");
            return Ok(0);
        }
    }

    let backup_path = if target_exists && !parsed.no_backup {
        let path = backup_path(&surface.target_path, surface.target_file_name);
        fs::rename(&surface.target_path, &path).map_err(|source| {
            CoreError::io(
                "reset_backup",
                format!(
                    "Could not back up the current {} at {}.",
                    surface.display_name,
                    surface.target_path.display()
                ),
                format!(
                    "Fix permissions or move the file manually, then retry `{}`.",
                    surface.command
                ),
                surface.target_path.display().to_string(),
                source,
            )
        })?;
        Some(path)
    } else if target_exists && parsed.no_backup {
        fs::remove_file(&surface.target_path).map_err(|source| {
            CoreError::io(
                "reset_remove_existing",
                format!(
                    "Could not remove the current {} at {}.",
                    surface.display_name,
                    surface.target_path.display()
                ),
                format!(
                    "Fix permissions or remove the file manually, then retry `{} --no-backup`.",
                    surface.command
                ),
                surface.target_path.display().to_string(),
                source,
            )
        })?;
        None
    } else {
        None
    };

    write_reset_surface(&surface.target_path, &content)?;

    if let Some(path) = backup_path {
        println!("✅ Backed up previous file to: {}", path.display());
    }
    println!(
        "✅ Replaced {} with a fresh template: {}",
        surface.target_file_name,
        surface.target_path.display()
    );
    if removed_without_backup {
        println!("⚠️  Previous file was removed without backup.");
    }

    Ok(0)
}

fn read_confirmation() -> String {
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
    line.trim().to_lowercase()
}

fn write_reset_surface(target_path: &Path, content: &str) -> Result<(), CoreError> {
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(|source| io_err(parent, source))?;
    }
    fs::write(target_path, content).map_err(|source| {
        CoreError::io(
            "reset_write_default",
            format!(
                "Could not write the default Yazelix template to {}.",
                target_path.display()
            ),
            "Fix permissions or restore the missing runtime template, then retry.",
            target_path.display().to_string(),
            source,
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::Permissions::from_mode(0o644);
        let _ = fs::set_permissions(target_path, mode);
    }
    Ok(())
}

fn io_err(path: &Path, source: io::Error) -> CoreError {
    CoreError::io(
        "reset_io",
        format!(
            "Could not access the Yazelix reset path {}.",
            path.display()
        ),
        "Fix permissions or restore the missing path, then retry.",
        path.display().to_string(),
        source,
    )
}

fn backup_path(target_path: &Path, fallback_name: &str) -> PathBuf {
    target_path.with_file_name(format!(
        "{}.backup-{}",
        target_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(fallback_name),
        backup_timestamp()
    ))
}

fn backup_timestamp() -> String {
    let epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_backup_timestamp_utc(epoch_secs)
}

fn format_backup_timestamp_utc(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let seconds_of_day = epoch_secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };

    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: `yzx reset config` and `yzx reset cursor` keep the real reset flags while rejecting stale force-style reset shapes.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parses_reset_surface_flags() {
        assert_eq!(
            parse_reset_args(&["--yes".into(), "--no-backup".into()], "yzx reset config").unwrap(),
            ResetArgs {
                yes: true,
                no_backup: true,
                help: false,
            }
        );
        assert_eq!(
            parse_reset_args(&["help".into()], "yzx reset cursor").unwrap(),
            ResetArgs {
                yes: false,
                no_backup: false,
                help: true,
            }
        );
        assert!(parse_reset_args(&["--force".into()], "yzx reset config").is_err());
    }

    // Regression: reset backup names must stay human-readable instead of regressing to opaque epoch-only suffixes.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn formats_compact_utc_backup_timestamps() {
        assert_eq!(format_backup_timestamp_utc(0), "19700101_000000");
        assert_eq!(
            format_backup_timestamp_utc(1_713_398_400),
            "20240418_000000"
        );
    }
}
