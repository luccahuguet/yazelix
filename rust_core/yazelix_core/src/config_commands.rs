// Test lane: default
//! `yzx config` family implemented in Rust for `yzx_control`.

use crate::active_config_surface::{primary_config_paths, resolve_active_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, config_override_from_env, runtime_dir_from_env};
use serde_json::json;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ConfigArgs {
    print_path: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResetArgs {
    yes: bool,
    no_backup: bool,
    help: bool,
}

pub fn run_yzx_config(args: &[String]) -> Result<i32, CoreError> {
    if matches!(args.first().map(String::as_str), Some("reset")) {
        return run_config_reset(&args[1..]);
    }

    let parsed = parse_config_args(args)?;
    if parsed.help {
        print_config_help();
        return Ok(0);
    }

    run_config_show(parsed.print_path)
}

fn parse_config_args(args: &[String]) -> Result<ConfigArgs, CoreError> {
    let mut parsed = ConfigArgs::default();

    for arg in args {
        match arg.as_str() {
            "--path" => parsed.print_path = true,
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

fn parse_reset_args(args: &[String]) -> Result<ResetArgs, CoreError> {
    let mut parsed = ResetArgs::default();

    for arg in args {
        match arg.as_str() {
            "--yes" => parsed.yes = true,
            "--no-backup" => parsed.no_backup = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx config reset: {other}. Try `yzx config reset --help`."
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
    println!("  yzx config reset [--yes] [--no-backup]");
    println!();
    println!("Flags:");
    println!("      --path       Print the resolved config path");
    println!();
    println!("Subcommands:");
    println!("  reset            Replace the managed config with a fresh shipped template");
}

fn print_config_reset_help() {
    println!("Replace the managed config with a fresh shipped template");
    println!();
    println!("Usage:");
    println!("  yzx config reset [--yes] [--no-backup]");
    println!();
    println!("Flags:");
    println!("      --yes        Skip confirmation prompt");
    println!("      --no-backup  Replace the config without writing a timestamped backup first");
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
    toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_config_surface",
            format!(
                "Could not parse the active Yazelix config at {}.",
                path.display()
            ),
            "Fix the TOML syntax or run `yzx config reset` to restore the managed template.",
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

fn read_confirmation() -> String {
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
    line.trim().to_lowercase()
}

fn copy_default_config(
    default_config_path: &Path,
    target_config_path: &Path,
) -> Result<(), CoreError> {
    if let Some(parent) = target_config_path.parent() {
        fs::create_dir_all(parent).map_err(|source| io_err(parent, source))?;
    }
    fs::copy(default_config_path, target_config_path).map_err(|source| {
        CoreError::io(
            "config_copy_default",
            format!(
                "Could not copy the default Yazelix config from {} to {}.",
                default_config_path.display(),
                target_config_path.display()
            ),
            "Fix permissions or restore the missing runtime config, then retry.",
            target_config_path.display().to_string(),
            source,
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::Permissions::from_mode(0o644);
        let _ = fs::set_permissions(target_config_path, mode);
    }
    Ok(())
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

fn backup_timestamp() -> String {
    let epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    format_backup_timestamp_utc(epoch_secs)
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

fn run_config_reset(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_reset_args(args)?;
    if parsed.help {
        print_config_reset_help();
        return Ok(0);
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = primary_config_paths(&runtime_dir, &config_dir);
    let user_config_exists = paths.user_config.exists();
    let removed_without_backup = parsed.no_backup && user_config_exists;

    if !paths.default_config_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "missing_default_config",
            format!(
                "Default config not found: {}",
                paths.default_config_path.display()
            ),
            "Reinstall Yazelix or restore yazelix_default.toml in the runtime, then retry.",
            json!({ "path": paths.default_config_path.display().to_string() }),
        ));
    }

    if !parsed.yes {
        println!("⚠️  This replaces yazelix.toml with a fresh shipped template.");
        if user_config_exists && !parsed.no_backup {
            println!("   Your current yazelix.toml will be backed up first.");
        }
        if user_config_exists && parsed.no_backup {
            println!("   Your current yazelix.toml will be removed without a backup.");
        }
        print!("Continue? [y/N]: ");
        let confirm = read_confirmation();
        if confirm != "y" && confirm != "yes" {
            println!("Aborted.");
            return Ok(0);
        }
    }

    let backup_path = if user_config_exists && !parsed.no_backup {
        let path = paths.user_config.with_file_name(format!(
            "{}.backup-{}",
            paths
                .user_config
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("yazelix.toml"),
            backup_timestamp()
        ));
        fs::rename(&paths.user_config, &path).map_err(|source| {
            CoreError::io(
                "config_backup",
                format!(
                    "Could not back up the current Yazelix config at {}.",
                    paths.user_config.display()
                ),
                "Fix permissions or move the config manually, then retry `yzx config reset`.",
                paths.user_config.display().to_string(),
                source,
            )
        })?;
        Some(path)
    } else if user_config_exists && parsed.no_backup {
        fs::remove_file(&paths.user_config).map_err(|source| {
            CoreError::io(
                "config_remove_existing",
                format!(
                    "Could not remove the current Yazelix config at {}.",
                    paths.user_config.display()
                ),
                "Fix permissions or remove the config manually, then retry `yzx config reset --no-backup`.",
                paths.user_config.display().to_string(),
                source,
            )
        })?;
        None
    } else {
        None
    };

    copy_default_config(&paths.default_config_path, &paths.user_config)?;

    if let Some(path) = backup_path {
        println!("✅ Backed up previous config to: {}", path.display());
    }
    println!(
        "✅ Replaced yazelix.toml with a fresh template: {}",
        paths.user_config.display()
    );
    if removed_without_backup {
        println!("⚠️  Previous config surface was removed without backup.");
    }

    Ok(0)
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
                print_path: true,
                help: false,
            }
        );
        assert_eq!(
            parse_config_args(&["--help".into()]).unwrap(),
            ConfigArgs {
                print_path: false,
                help: true,
            }
        );
        assert!(parse_config_args(&["--force".into()]).is_err());
    }

    // Defends: `yzx config reset` keeps the real `--yes` and `--no-backup` public contract rather than the stale metadata-only `--force` shape.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn parses_config_reset_flags() {
        assert_eq!(
            parse_reset_args(&["--yes".into(), "--no-backup".into()]).unwrap(),
            ResetArgs {
                yes: true,
                no_backup: true,
                help: false,
            }
        );
        assert_eq!(
            parse_reset_args(&["help".into()]).unwrap(),
            ResetArgs {
                yes: false,
                no_backup: false,
                help: true,
            }
        );
        assert!(parse_reset_args(&["--force".into()]).is_err());
    }

    // Regression: config reset backup names must stay human-readable after the Rust owner cut instead of regressing to opaque epoch-only suffixes.
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
