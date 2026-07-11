// Test lane: default
//! `yzx reset` family implemented in Rust for `yzx_control`.

use crate::active_config_surface::primary_config_paths;
use crate::backup_timestamp::compact_utc_backup_timestamp;
use crate::bridge::CoreError;
use crate::control_plane::{config_dir_from_env, runtime_dir_from_env};
use crate::settings_surface::render_default_config;
use crate::user_config_paths::{
    CURRENT_MANAGED_CONFIG_FILE_NAMES, LEGACY_CONFIG_ENTRY_NAMES, SETTINGS_CONFIG,
};
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

const RESET_CONFIG_COMMAND: &str = "yzx reset config";
const RESET_CONFIG_DISPLAY_NAME: &str = "main Yazelix config";
const RESET_CONFIG_FILE_NAME: &str = SETTINGS_CONFIG;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResetArgs {
    yes: bool,
    no_backup: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ResetConfigAdjacencyReport {
    managed_overrides: Vec<String>,
    legacy_inputs: Vec<String>,
    unknown_entries: Vec<String>,
}

pub fn run_yzx_reset(args: &[String]) -> Result<i32, CoreError> {
    match args.first().map(String::as_str) {
        None | Some("-h" | "--help" | "help") => {
            print_reset_help();
            Ok(0)
        }
        Some("config") => run_reset_config(&args[1..]),
        Some(other) => Err(CoreError::usage(format!(
            "Unknown reset target for yzx reset: {other}. Try `yzx reset --help`."
        ))),
    }
}

fn run_reset_config(args: &[String]) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let paths = primary_config_paths(&runtime_dir, &config_dir);
    let adjacency_report = reset_config_adjacency_report(&config_dir)?;
    let content = render_default_config(&paths.default_config_path)?;
    reset_config_with_content(args, paths.user_config, content, adjacency_report)
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
    println!();
    println!("Targets:");
    println!("  config  Replace ~/.config/yazelix/config.toml with fresh shipped settings");
    println!();
    println!("Note:");
    println!("  reset config preserves managed override sidecars and unknown adjacent files");
}

fn print_reset_config_help() {
    println!("Replace the {RESET_CONFIG_DISPLAY_NAME} with a fresh shipped template");
    println!();
    println!("Usage:");
    println!("  {RESET_CONFIG_COMMAND} [--yes] [--no-backup]");
    println!();
    println!("Flags:");
    println!("      --yes        Skip confirmation prompt");
    println!("      --no-backup  Replace the file without writing a timestamped backup first");
}

fn reset_config_with_content(
    args: &[String],
    target_path: PathBuf,
    content: String,
    adjacency_report: ResetConfigAdjacencyReport,
) -> Result<i32, CoreError> {
    let parsed = parse_reset_args(args, RESET_CONFIG_COMMAND)?;
    if parsed.help {
        print_reset_config_help();
        return Ok(0);
    }

    let target_exists = target_path.exists();
    let removed_without_backup = parsed.no_backup && target_exists;

    print_reset_config_adjacency_warnings(&adjacency_report);

    if !parsed.yes {
        println!(
            "⚠️  This replaces {} with a fresh shipped template.",
            RESET_CONFIG_FILE_NAME
        );
        if target_exists && !parsed.no_backup {
            println!(
                "   Your current {} will be backed up first.",
                RESET_CONFIG_FILE_NAME
            );
        }
        if target_exists && parsed.no_backup {
            println!(
                "   Your current {} will be removed without a backup.",
                RESET_CONFIG_FILE_NAME
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
        let path = backup_path(&target_path, RESET_CONFIG_FILE_NAME);
        fs::rename(&target_path, &path).map_err(|source| {
            CoreError::io(
                "reset_backup",
                format!(
                    "Could not back up the current {} at {}.",
                    RESET_CONFIG_DISPLAY_NAME,
                    target_path.display()
                ),
                format!(
                    "Fix permissions or move the file manually, then retry `{}`.",
                    RESET_CONFIG_COMMAND
                ),
                target_path.display().to_string(),
                source,
            )
        })?;
        Some(path)
    } else if target_exists && parsed.no_backup {
        fs::remove_file(&target_path).map_err(|source| {
            CoreError::io(
                "reset_remove_existing",
                format!(
                    "Could not remove the current {} at {}.",
                    RESET_CONFIG_DISPLAY_NAME,
                    target_path.display()
                ),
                format!(
                    "Fix permissions or remove the file manually, then retry `{} --no-backup`.",
                    RESET_CONFIG_COMMAND
                ),
                target_path.display().to_string(),
                source,
            )
        })?;
        None
    } else {
        None
    };

    write_reset_surface(&target_path, &content)?;

    if let Some(path) = backup_path {
        println!("✅ Backed up previous file to: {}", path.display());
    }
    println!(
        "✅ Replaced {} with a fresh template: {}",
        RESET_CONFIG_FILE_NAME,
        target_path.display()
    );
    if removed_without_backup {
        println!("⚠️  Previous file was removed without backup.");
    }

    Ok(0)
}

fn reset_config_adjacency_report(
    config_dir: &Path,
) -> Result<ResetConfigAdjacencyReport, CoreError> {
    let current_managed: BTreeSet<String> = CURRENT_MANAGED_CONFIG_FILE_NAMES
        .iter()
        .filter_map(|entry| {
            Path::new(entry)
                .components()
                .next()
                .map(|component| component.as_os_str().to_string_lossy().to_string())
        })
        .collect();
    let legacy: BTreeSet<&str> = LEGACY_CONFIG_ENTRY_NAMES.iter().copied().collect();
    let mut report = ResetConfigAdjacencyReport::default();

    let entries = match fs::read_dir(config_dir) {
        Ok(entries) => entries,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(report),
        Err(source) => return Err(io_err(config_dir, source)),
    };

    for entry in entries {
        let entry = entry.map_err(|source| io_err(config_dir, source))?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == SETTINGS_CONFIG
            || name.starts_with("config.toml.backup-")
            || name.starts_with("settings.jsonc.backup-")
        {
            continue;
        }
        if current_managed.contains(&name) {
            report.managed_overrides.push(name);
        } else if legacy.contains(name.as_str()) {
            report.legacy_inputs.push(name);
        } else {
            report.unknown_entries.push(name);
        }
    }
    report.managed_overrides.sort();
    report.legacy_inputs.sort();
    report.unknown_entries.sort();

    Ok(report)
}

fn print_reset_config_adjacency_warnings(report: &ResetConfigAdjacencyReport) {
    if !report.managed_overrides.is_empty() {
        println!(
            "Warning: {} only replaces {}. Managed override files were left untouched: {}.",
            RESET_CONFIG_COMMAND,
            RESET_CONFIG_FILE_NAME,
            report.managed_overrides.join(", ")
        );
        println!(
            "         These files can still affect Helix, Yazi, Zellij, terminal, or shell behavior after reset."
        );
    }
    if !report.legacy_inputs.is_empty() {
        println!(
            "Warning: legacy Yazelix config inputs were left untouched: {}.",
            report.legacy_inputs.join(", ")
        );
        println!(
            "         Move them aside; stale old inputs block startup and are not migrated automatically."
        );
    }
    if !report.unknown_entries.is_empty() {
        println!(
            "Warning: unknown adjacent entries in ~/.config/yazelix were left untouched: {}.",
            report.unknown_entries.join(", ")
        );
        println!("         Yazelix will not delete or adopt user-managed files automatically.");
    }
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
        compact_utc_backup_timestamp()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: `yzx reset config` keeps the real reset flags while rejecting stale force-style reset shapes.
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
        assert!(
            parse_reset_args(&["help".into()], "yzx reset config")
                .unwrap()
                .help
        );
        assert!(parse_reset_args(&["--force".into()], "yzx reset config").is_err());
    }
}
