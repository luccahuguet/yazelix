// Test lane: default
//! `yzx import` family implemented in Rust for `yzx_control`.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, home_dir_from_env};
use serde_json::json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ImportArgs {
    target: Option<String>,
    force: bool,
    help: bool,
}

struct ImportEntry {
    name: &'static str,
    source: PathBuf,
    destination: PathBuf,
}

fn get_xdg_config_home(home: &Path) -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"))
}

fn get_native_zellij_config_path(home: &Path) -> PathBuf {
    get_xdg_config_home(home).join("zellij").join("config.kdl")
}

fn get_managed_zellij_config_path(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("zellij")
        .join("config.kdl")
}

fn get_native_yazi_config_dir(home: &Path) -> PathBuf {
    get_xdg_config_home(home).join("yazi")
}

fn get_managed_yazi_config_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("user_configs").join("yazi")
}

fn get_native_helix_config_path(home: &Path) -> PathBuf {
    get_xdg_config_home(home).join("helix").join("config.toml")
}

fn get_managed_helix_config_path(config_dir: &Path) -> PathBuf {
    config_dir
        .join("user_configs")
        .join("helix")
        .join("config.toml")
}

fn get_import_entries(
    target: &str,
    home: &Path,
    config_dir: &Path,
) -> Result<Vec<ImportEntry>, CoreError> {
    match target {
        "zellij" => Ok(vec![ImportEntry {
            name: "config.kdl",
            source: get_native_zellij_config_path(home),
            destination: get_managed_zellij_config_path(config_dir),
        }]),
        "yazi" => {
            let source_dir = get_native_yazi_config_dir(home);
            let dest_dir = get_managed_yazi_config_dir(config_dir);
            Ok(vec![
                ImportEntry {
                    name: "yazi.toml",
                    source: source_dir.join("yazi.toml"),
                    destination: dest_dir.join("yazi.toml"),
                },
                ImportEntry {
                    name: "keymap.toml",
                    source: source_dir.join("keymap.toml"),
                    destination: dest_dir.join("keymap.toml"),
                },
                ImportEntry {
                    name: "init.lua",
                    source: source_dir.join("init.lua"),
                    destination: dest_dir.join("init.lua"),
                },
            ])
        }
        "helix" => Ok(vec![ImportEntry {
            name: "config.toml",
            source: get_native_helix_config_path(home),
            destination: get_managed_helix_config_path(config_dir),
        }]),
        other => Err(CoreError::usage(format!(
            "Unknown import target: {other}. Try `yzx import --help`."
        ))),
    }
}

fn io_err(path: &Path, source: io::Error, code: &str) -> CoreError {
    CoreError::io(
        code,
        format!("Could not access {}.", path.display()),
        "Fix permissions or restore the missing path, then retry.",
        path.display().to_string(),
        source,
    )
}

fn backup_timestamp() -> String {
    let epoch_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
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

fn import_target(
    target: &str,
    force: bool,
    home: &Path,
    config_dir: &Path,
) -> Result<i32, CoreError> {
    let entries = get_import_entries(target, home, config_dir)?;

    let existing_sources: Vec<_> = entries.iter().filter(|e| e.source.exists()).collect();
    let missing_sources: Vec<_> = entries.iter().filter(|e| !e.source.exists()).collect();

    if existing_sources.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "no_import_sources",
            format!("No native {target} config files found to import."),
            &format!("Create the native {target} config files first, then retry."),
            json!({ "target": target }),
        ));
    }

    if !force {
        let conflicts: Vec<_> = existing_sources
            .iter()
            .filter(|e| e.destination.exists())
            .collect();
        if !conflicts.is_empty() {
            let conflict_lines = conflicts
                .iter()
                .map(|e| format!("  - {}", e.destination.display()))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(CoreError::classified(
                ErrorClass::Usage,
                "import_conflicts",
                format!(
                    "Managed destination files already exist for `yzx import {target}`:\n{conflict_lines}"
                ),
                &format!(
                    "Use `yzx import {target} --force` to overwrite them after writing backups."
                ),
                json!({ "target": target }),
            ));
        }
    }

    let timestamp = backup_timestamp();
    let mut backup_records = Vec::new();

    for entry in &existing_sources {
        if let Some(parent) = entry.destination.parent() {
            fs::create_dir_all(parent).map_err(|source| io_err(parent, source, "import_mkdir"))?;
        }

        if force && entry.destination.exists() {
            let backup_path = entry.destination.with_file_name(format!(
                "{}.backup-{}",
                entry
                    .destination
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(entry.name),
                timestamp
            ));
            fs::rename(&entry.destination, &backup_path)
                .map_err(|source| io_err(&entry.destination, source, "import_backup"))?;
            backup_records.push((entry.name, backup_path));
        }

        fs::copy(&entry.source, &entry.destination).map_err(|source| {
            CoreError::io(
                "import_copy",
                format!(
                    "Could not copy {} to {}.",
                    entry.source.display(),
                    entry.destination.display()
                ),
                "Fix permissions or restore the missing source, then retry.",
                entry.destination.display().to_string(),
                source,
            )
        })?;
    }

    println!(
        "✅ Imported native {target} config into: {}",
        match target {
            "zellij" => get_managed_zellij_config_path(config_dir)
                .parent()
                .unwrap_or(config_dir)
                .display()
                .to_string(),
            "yazi" => get_managed_yazi_config_dir(config_dir)
                .display()
                .to_string(),
            "helix" => get_managed_helix_config_path(config_dir)
                .parent()
                .unwrap_or(config_dir)
                .display()
                .to_string(),
            _ => config_dir.display().to_string(),
        }
    );

    if target == "yazi" {
        let imported_names = existing_sources
            .iter()
            .map(|e| e.name)
            .collect::<Vec<_>>()
            .join(", ");
        println!("   Imported files: {imported_names}");
    } else {
        println!("   Source: {}", existing_sources[0].source.display());
    }

    if !backup_records.is_empty() {
        println!("   Backup files:");
        for (name, path) in backup_records {
            println!("   - {name}: {}", path.display());
        }
    }

    if !missing_sources.is_empty() {
        let skipped = missing_sources
            .iter()
            .map(|e| e.name)
            .collect::<Vec<_>>()
            .join(", ");
        println!("   Skipped missing native files: {skipped}");
    }

    Ok(0)
}

fn parse_import_args(args: &[String]) -> Result<ImportArgs, CoreError> {
    let mut parsed = ImportArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--force" => parsed.force = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx import: {other}. Try `yzx import --help`."
                )));
            }
            other => {
                if parsed.target.is_some() {
                    return Err(CoreError::usage(
                        "yzx import accepts at most one target.".to_string(),
                    ));
                }
                parsed.target = Some(other.to_string());
            }
        }
    }

    Ok(parsed)
}

pub fn run_yzx_import(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_import_args(args)?;
    if parsed.help {
        print_import_help();
        return Ok(0);
    }

    let target = parsed.target.ok_or_else(|| {
        CoreError::usage("yzx import requires a target. Try `yzx import --help`.".to_string())
    })?;

    let home = home_dir_from_env()?;
    let config_dir = config_dir_from_env()?;

    import_target(&target, parsed.force, &home, &config_dir)
}

fn print_import_help() {
    println!("Import native config files into Yazelix-managed override paths");
    println!();
    println!("Usage:");
    println!("  yzx import <target> [--force]");
    println!();
    println!("Targets:");
    println!("  zellij    Import native Zellij config");
    println!("  yazi      Import native Yazi config files");
    println!("  helix     Import native Helix config");
    println!();
    println!("Flags:");
    println!("  --force   Overwrite managed destinations after writing backups");
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: import command parsing keeps the --force switch and rejects unknown flags.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parses_import_args() {
        let parsed = parse_import_args(&["zellij".into(), "--force".into()]).unwrap();
        assert_eq!(parsed.target, Some("zellij".to_string()));
        assert!(parsed.force);

        let parsed = parse_import_args(&["yazi".into()]).unwrap();
        assert_eq!(parsed.target, Some("yazi".to_string()));
        assert!(!parsed.force);

        assert!(parse_import_args(&["--unknown".into()]).is_err());
    }

    // Defends: import entry resolution keeps the three supported targets.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn resolves_import_entries_for_supported_targets() {
        let home = Path::new("/home/test");
        let config = Path::new("/home/test/.config/yazelix");

        let zellij = get_import_entries("zellij", home, config).unwrap();
        assert_eq!(zellij.len(), 1);
        assert!(
            zellij[0]
                .source
                .to_string_lossy()
                .contains("zellij/config.kdl")
        );

        let yazi = get_import_entries("yazi", home, config).unwrap();
        assert_eq!(yazi.len(), 3);

        let helix = get_import_entries("helix", home, config).unwrap();
        assert_eq!(helix.len(), 1);

        assert!(get_import_entries("unknown", home, config).is_err());
    }

    // Defends: backup timestamp format stays human-readable after the Rust owner cut.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
    #[test]
    fn formats_import_backup_timestamp() {
        assert_eq!(format_backup_timestamp(0), "19700101_000000");
        assert_eq!(format_backup_timestamp(1_713_398_400), "20240418_000000");
    }

    fn format_backup_timestamp(epoch_secs: i64) -> String {
        let days = epoch_secs.div_euclid(86_400);
        let seconds_of_day = epoch_secs.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        let hour = seconds_of_day / 3_600;
        let minute = (seconds_of_day % 3_600) / 60;
        let second = seconds_of_day % 60;
        format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
    }
}
