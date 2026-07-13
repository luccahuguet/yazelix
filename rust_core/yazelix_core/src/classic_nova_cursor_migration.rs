//! Temporary Classic-owned cursor transaction that leaves `cursors.toml` Nova-native.
// Test lane: default

use crate::atomic_fs::{
    write_text_atomic_create_new_with_permissions, write_text_atomic_with_permissions,
};
use crate::backup_timestamp::compact_utc_backup_timestamp;
use crate::bridge::{CoreError, ErrorClass};
use crate::native_config_status::path_owned_by_home_manager;
use ratconfig::patch::PatchMutation;
use ratconfig::toml_adapter::unset_toml_value_text;
use serde_json::json;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const RETIRED_KITTY_CURSOR_PATH: &str = "settings.kitty_enable_cursor";

pub(crate) fn migrate_classic_cursor_to_nova(path: &Path) -> Result<Option<PathBuf>, CoreError> {
    migrate_with_timestamp(path, &compact_utc_backup_timestamp())
}

fn migrate_with_timestamp(path: &Path, timestamp: &str) -> Result<Option<PathBuf>, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        cursor_io_error(
            "read_classic_cursor_migration_source",
            path,
            "Could not read the Classic cursor config",
            source,
        )
    })?;
    let patched = unset_toml_value_text(&raw, RETIRED_KITTY_CURSOR_PATH).map_err(|error| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_classic_cursor_migration_source",
            format!("Could not inspect {}: {error:?}.", path.display()),
            "Fix the TOML syntax and keep settings as a normal table, then retry.",
            json!({ "path": path, "field": RETIRED_KITTY_CURSOR_PATH }),
        )
    })?;
    if patched.mutation == PatchMutation::Unchanged {
        return Ok(None);
    }

    let metadata = fs::symlink_metadata(path).map_err(|source| {
        cursor_io_error(
            "inspect_classic_cursor_migration_source",
            path,
            "Could not inspect the Classic cursor config",
            source,
        )
    })?;
    if path_owned_by_home_manager(path)
        || metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.permissions().readonly()
    {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "declarative_classic_cursor_migration",
            format!(
                "{} still declares the retired {RETIRED_KITTY_CURSOR_PATH} field but is declarative, symlinked, read-only, or not a regular file.",
                path.display()
            ),
            "Remove settings.kitty_enable_cursor from the file's owner, then run home-manager switch or relaunch Yazelix.",
            json!({ "path": path, "field": RETIRED_KITTY_CURSOR_PATH }),
        ));
    }

    yazelix_cursors::CursorRegistry::parse_str(path, &patched.text)?;
    let permissions = metadata.permissions();
    let backup = cursor_backup_path(path, timestamp);
    write_text_atomic_create_new_with_permissions(&backup, &raw, &permissions)?;
    let current = fs::read_to_string(path).map_err(|source| {
        cursor_io_error(
            "reread_classic_cursor_migration_source",
            path,
            "Could not verify the Classic cursor config before replacing it",
            source,
        )
    })?;
    if current != raw {
        return Err(CoreError::classified(
            ErrorClass::Io,
            "classic_cursor_migration_source_changed",
            format!(
                "{} changed while its migration was being prepared.",
                path.display()
            ),
            "Preserve the timestamped backup, review the current file, then retry from one stable cursor config.",
            json!({ "path": path, "backup": backup }),
        ));
    }
    write_text_atomic_with_permissions(path, &patched.text, &permissions)?;
    Ok(Some(backup))
}

fn cursor_backup_path(path: &Path, timestamp: &str) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("cursors.toml");
    path.with_file_name(format!("{name}.backup-{timestamp}"))
}

fn cursor_io_error(code: &'static str, path: &Path, message: &str, source: io::Error) -> CoreError {
    CoreError::io(
        code,
        message,
        "Fix the cursor file ownership or permissions, then retry.",
        path.display().to_string(),
        source,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[cfg(unix)]
    use std::os::unix::fs::{PermissionsExt, symlink};

    const CLASSIC_CURSOR_CONFIG: &str = r##"# keep this root comment
schema_version = 1
enabled_cursors = ["custom_split"]

[settings]
trail = "custom_split"
trail_effect = "tail"
mode_effect = "none"
glow = "medium"
duration = 1.0
# retired Classic-only toggle
kitty_enable_cursor = true

# keep this custom cursor comment
[[cursor]]
name = "custom_split"
family = "split"
divider = "horizontal"
transition = "soft"
colors = ["#112233", "#445566"]
cursor_color = "#778899"
"##;

    // Regression: the final Classic bridge preserves user TOML while retiring the one field rejected by Nova and Mars.
    #[test]
    fn retires_kitty_field_backup_first_and_idempotently() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("cursors.toml");
        fs::write(&path, CLASSIC_CURSOR_CONFIG).unwrap();
        #[cfg(unix)]
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();

        let backup = migrate_with_timestamp(&path, "test").unwrap().unwrap();
        let migrated = fs::read_to_string(&path).unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), CLASSIC_CURSOR_CONFIG);
        assert!(!migrated.contains("kitty_enable_cursor"));
        assert!(migrated.contains("# keep this root comment"));
        assert!(migrated.contains("# keep this custom cursor comment"));
        assert!(migrated.contains("name = \"custom_split\""));
        yazelix_cursors::CursorRegistry::parse_str(&path, &migrated).unwrap();
        #[cfg(unix)]
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o640
        );

        assert_eq!(migrate_with_timestamp(&path, "second").unwrap(), None);
        assert_eq!(fs::read_to_string(&path).unwrap(), migrated);
        assert!(!cursor_backup_path(&path, "second").exists());
    }

    // Defends: Classic never takes ownership of declarative or read-only cursor files during the compatibility window.
    #[cfg(unix)]
    #[test]
    fn refuses_symlinked_and_read_only_cursor_sources() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.toml");
        let linked = dir.path().join("linked.toml");
        fs::write(&target, CLASSIC_CURSOR_CONFIG).unwrap();
        symlink(&target, &linked).unwrap();
        let error = migrate_with_timestamp(&linked, "linked").unwrap_err();
        assert_eq!(error.code(), "declarative_classic_cursor_migration");
        assert_eq!(fs::read_to_string(&target).unwrap(), CLASSIC_CURSOR_CONFIG);

        let read_only = dir.path().join("read_only.toml");
        fs::write(&read_only, CLASSIC_CURSOR_CONFIG).unwrap();
        fs::set_permissions(&read_only, fs::Permissions::from_mode(0o444)).unwrap();
        let error = migrate_with_timestamp(&read_only, "read_only").unwrap_err();
        assert_eq!(error.code(), "declarative_classic_cursor_migration");
        assert_eq!(
            fs::read_to_string(&read_only).unwrap(),
            CLASSIC_CURSOR_CONFIG
        );
    }
}
