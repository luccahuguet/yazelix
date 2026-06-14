// Test lane: default
//! `yzx import` family implemented in Rust for `yzx_control`.

use crate::backup_timestamp::compact_utc_backup_timestamp;
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, home_dir_from_env, state_dir_from_env};
use crate::user_config_paths;
use crate::zellij_materialization::zellij_config_contains_keybinds_block;
use serde_json::json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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
    kind: ImportEntryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportEntryKind {
    File,
    Directory,
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
    user_config_paths::zellij_config(config_dir)
}

fn get_native_yazi_config_dir(home: &Path) -> PathBuf {
    get_xdg_config_home(home).join("yazi")
}

fn get_managed_yazi_config_dir(config_dir: &Path) -> PathBuf {
    user_config_paths::yazi_config_dir(config_dir)
}

fn get_native_helix_config_path(home: &Path) -> PathBuf {
    get_xdg_config_home(home).join("helix").join("config.toml")
}

fn get_managed_helix_config_path(config_dir: &Path) -> PathBuf {
    user_config_paths::helix_config(config_dir)
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
            kind: ImportEntryKind::File,
        }]),
        "yazi" => {
            let source_dir = get_native_yazi_config_dir(home);
            let dest_dir = get_managed_yazi_config_dir(config_dir);
            Ok(vec![
                ImportEntry {
                    name: "yazi.toml",
                    source: source_dir.join("yazi.toml"),
                    destination: dest_dir.join("yazi.toml"),
                    kind: ImportEntryKind::File,
                },
                ImportEntry {
                    name: "keymap.toml",
                    source: source_dir.join("keymap.toml"),
                    destination: dest_dir.join("keymap.toml"),
                    kind: ImportEntryKind::File,
                },
                ImportEntry {
                    name: "init.lua",
                    source: source_dir.join("init.lua"),
                    destination: dest_dir.join("init.lua"),
                    kind: ImportEntryKind::File,
                },
                ImportEntry {
                    name: "package.toml",
                    source: source_dir.join("package.toml"),
                    destination: dest_dir.join("package.toml"),
                    kind: ImportEntryKind::File,
                },
                ImportEntry {
                    name: "plugins/",
                    source: source_dir.join("plugins"),
                    destination: dest_dir.join("plugins"),
                    kind: ImportEntryKind::Directory,
                },
                ImportEntry {
                    name: "flavors/",
                    source: source_dir.join("flavors"),
                    destination: dest_dir.join("flavors"),
                    kind: ImportEntryKind::Directory,
                },
            ])
        }
        "helix" => Ok(vec![ImportEntry {
            name: "config.toml",
            source: get_native_helix_config_path(home),
            destination: get_managed_helix_config_path(config_dir),
            kind: ImportEntryKind::File,
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

fn source_matches_kind(entry: &ImportEntry) -> Result<bool, CoreError> {
    match fs::metadata(&entry.source) {
        Ok(metadata) => Ok(match entry.kind {
            ImportEntryKind::File => metadata.is_file(),
            ImportEntryKind::Directory => metadata.is_dir(),
        }),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(io_err(&entry.source, source, "import_source_stat")),
    }
}

fn copy_import_entry(entry: &ImportEntry) -> Result<(), CoreError> {
    match entry.kind {
        ImportEntryKind::File => {
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
        ImportEntryKind::Directory => {
            copy_directory_recursive(&entry.source, &entry.destination)?;
        }
    }
    Ok(())
}

fn validate_import_entry_source(target: &str, entry: &ImportEntry) -> Result<(), CoreError> {
    if target != "zellij" {
        return Ok(());
    }

    let content = fs::read_to_string(&entry.source).map_err(|source| {
        CoreError::io(
            "import_read_zellij_source",
            format!("Could not read {}.", entry.source.display()),
            "Fix permissions or restore the native Zellij config, then retry.",
            entry.source.display().to_string(),
            source,
        )
    })?;

    if zellij_config_contains_keybinds_block(&content) {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "import_zellij_keybinds_unsupported",
            format!(
                "Cannot import native Zellij keybinds into Yazelix-managed config: {}",
                entry.source.display()
            ),
            "Remove the keybinds block before importing, or keep full native Zellij keybinding ownership in plain zellij. Use zellij.keybindings and zellij.native_keybindings in settings.jsonc for Yazelix sessions.",
            json!({ "source": entry.source.to_string_lossy() }),
        ));
    }

    Ok(())
}

fn copy_directory_recursive(source: &Path, destination: &Path) -> Result<(), CoreError> {
    fs::create_dir_all(destination).map_err(|source_err| {
        CoreError::io(
            "import_copy_directory_create",
            format!("Could not create {}.", destination.display()),
            "Fix permissions or choose another managed Yazelix config directory, then retry.",
            destination.display().to_string(),
            source_err,
        )
    })?;

    for entry in fs::read_dir(source).map_err(|source_err| {
        CoreError::io(
            "import_copy_directory_read",
            format!("Could not read {}.", source.display()),
            "Fix permissions or restore the missing source, then retry.",
            source.display().to_string(),
            source_err,
        )
    })? {
        let entry = entry.map_err(|source_err| {
            CoreError::io(
                "import_copy_directory_entry",
                format!("Could not read an entry under {}.", source.display()),
                "Fix permissions or restore the missing source, then retry.",
                source.display().to_string(),
                source_err,
            )
        })?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .map_err(|source_err| io_err(&source_path, source_err, "import_copy_stat"))?;
        if metadata.is_dir() {
            copy_directory_recursive(&source_path, &destination_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &destination_path).map_err(|source_err| {
                CoreError::io(
                    "import_copy_file",
                    format!(
                        "Could not copy {} to {}.",
                        source_path.display(),
                        destination_path.display()
                    ),
                    "Fix permissions or restore the missing source, then retry.",
                    destination_path.display().to_string(),
                    source_err,
                )
            })?;
        } else {
            return Err(CoreError::classified(
                ErrorClass::Usage,
                "unsupported_import_source_entry",
                format!(
                    "Could not import unsupported source entry: {}",
                    source_path.display()
                ),
                "Remove symlinks or special files from the native Yazi plugin directory, then retry.",
                json!({ "path": source_path.to_string_lossy() }),
            ));
        }
    }

    Ok(())
}

fn invalidate_generated_runtime_state() -> Result<bool, CoreError> {
    let state_path = state_dir_from_env()?.join("state").join("rebuild_hash");
    match fs::remove_file(&state_path) {
        Ok(()) => Ok(true),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(source) => Err(io_err(
            &state_path,
            source,
            "import_invalidate_runtime_state",
        )),
    }
}

fn import_target(
    target: &str,
    force: bool,
    home: &Path,
    config_dir: &Path,
) -> Result<i32, CoreError> {
    let entries = get_import_entries(target, home, config_dir)?;

    let mut existing_sources = Vec::new();
    let mut missing_sources = Vec::new();
    for entry in &entries {
        if source_matches_kind(entry)? {
            existing_sources.push(entry);
        } else {
            missing_sources.push(entry);
        }
    }

    if existing_sources.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "no_import_sources",
            format!("No native {target} sources found to import."),
            &format!("Create the native {target} config or plugin sources first, then retry."),
            json!({ "target": target }),
        ));
    }

    for entry in &existing_sources {
        validate_import_entry_source(target, entry)?;
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
                    "Managed destinations already exist for `yzx import {target}`:\n{conflict_lines}"
                ),
                &format!(
                    "Use `yzx import {target} --force` to overwrite them after writing backups."
                ),
                json!({ "target": target }),
            ));
        }
    }

    let timestamp = compact_utc_backup_timestamp();
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

        copy_import_entry(entry)?;
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

    if invalidate_generated_runtime_state()? {
        println!("   Marked generated runtime state for refresh on next launch.");
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
    println!("  zellij    Import native Zellij config without keybinds blocks");
    println!("  yazi      Import native Yazi config files and plugin directories");
    println!("  helix     Import native Helix config");
    println!();
    println!("Flags:");
    println!("  --force   Overwrite managed destinations after writing backups");
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: import command parsing keeps the --force switch and rejects unknown flags.
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

    // Defends: import entry resolution keeps the three supported targets and the native Yazi home shape.
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
        assert_eq!(yazi.len(), 6);
        assert_eq!(
            yazi[0].destination,
            Path::new("/home/test/.config/yazelix/yazi/yazi.toml")
        );
        assert_eq!(
            yazi[1].destination,
            Path::new("/home/test/.config/yazelix/yazi/keymap.toml")
        );
        assert_eq!(
            yazi[2].destination,
            Path::new("/home/test/.config/yazelix/yazi/init.lua")
        );
        assert_eq!(
            yazi[3].destination,
            Path::new("/home/test/.config/yazelix/yazi/package.toml")
        );
        assert_eq!(yazi[4].name, "plugins/");
        assert_eq!(yazi[4].kind, ImportEntryKind::Directory);
        assert!(yazi[4].source.to_string_lossy().contains("yazi/plugins"));
        assert_eq!(
            yazi[4].destination,
            Path::new("/home/test/.config/yazelix/yazi/plugins")
        );
        assert_eq!(yazi[5].name, "flavors/");
        assert_eq!(
            yazi[5].destination,
            Path::new("/home/test/.config/yazelix/yazi/flavors")
        );

        let helix = get_import_entries("helix", home, config).unwrap();
        assert_eq!(helix.len(), 1);

        assert!(get_import_entries("unknown", home, config).is_err());
    }

    // Defends: import cannot create a managed zellij.kdl that later bypasses generated Yazelix keybindings.
    #[test]
    fn rejects_importing_zellij_keybind_blocks() {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("config.kdl");
        fs::write(
            &source,
            "keybinds { normal { bind \"Alt t\" { ToggleFloatingPanes; } } }\n",
        )
        .unwrap();
        let entry = ImportEntry {
            name: "config.kdl",
            source,
            destination: tmp.path().join("zellij.kdl"),
            kind: ImportEntryKind::File,
        };

        let err = validate_import_entry_source("zellij", &entry).unwrap_err();

        match err {
            CoreError::Classified {
                code, remediation, ..
            } => {
                assert_eq!(code, "import_zellij_keybinds_unsupported");
                assert!(remediation.contains("settings.jsonc"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
