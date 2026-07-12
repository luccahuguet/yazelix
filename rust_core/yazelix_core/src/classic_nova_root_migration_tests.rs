// Test lane: default

use super::*;
use std::fs;
use tempfile::tempdir;

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn request(config_dir: &Path) -> ClassicNovaMigrationRequest {
    ClassicNovaMigrationRequest {
        config_dir: config_dir.to_path_buf(),
        classic_default_config: repo_path("config_default.toml"),
        classic_contract: repo_path("config_metadata/main_config_contract.toml"),
    }
}

fn artifact(config_dir: &Path, prefix: &str, suffix: &str) -> PathBuf {
    fs::read_dir(config_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            name.starts_with(prefix) && name.ends_with(suffix)
        })
        .unwrap()
}

fn current_legacy_jsonc(body: &str) -> String {
    format!(
        "{{\n{body},\n  \"ratconfig\": {{ \"contract\": {{ \"schema_version\": 1, \"contract_id\": \"yazelix.settings\", \"version\": 16, \"applied_change_ids\": {} }} }}\n}}\n",
        serde_json::to_string(crate::settings_contract::SETTINGS_CONTRACT_APPLIED_CHANGE_IDS)
            .unwrap()
    )
}

// Defends: absent and already-Nova roots are no-write states, including the only shared semantic field.
#[test]
fn leaves_absent_nova_and_shared_only_roots_unchanged() {
    let config_dir = tempdir().unwrap();
    let absent = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap();
    assert_eq!(absent.status, ClassicNovaMigrationStatus::Absent);

    for raw in [
        "# sparse Nova\n[welcome]\nenabled = false\n",
        "# shared path\n[editor]\ncommand = \"nvim\"\n",
        "# intentionally empty sparse root\n",
        r#"[open]
log_level = "debug"
[shell]
program = "fish"
[editor]
command = "nvim"
[agent]
command = "auto"
args = []
[welcome]
enabled = true
style = "boids"
duration_seconds = 5
[popup]
side_margin = 2
vertical_margin = 1
[keybindings]
config = "Alt Shift K"
agent = "Alt Shift L"
git = "Alt Shift J"
menu = "Alt Shift M"
[bar]
widgets = ["session", "editor", "shell", "term", "codex_usage", "cpu", "ram"]
[popups.btm]
command = "btm"
args = ["--basic"]
title = "btm_popup"
keybinding = "Alt Shift B"
keep_alive = true
"#,
    ] {
        let config = config_dir.path().join("config.toml");
        fs::write(&config, raw).unwrap();
        let result = migrate_with(
            &request(config_dir.path()),
            "20260712_000000",
            &RealTransactionIo,
        )
        .unwrap();
        assert_eq!(result.status, ClassicNovaMigrationStatus::NovaUnchanged);
        assert_eq!(fs::read_to_string(&config).unwrap(), raw);
        fs::remove_file(config).unwrap();
    }
    assert_eq!(fs::read_dir(config_dir.path()).unwrap().count(), 0);
}

// Regression: right-agent semantics migrate only through the translator, while source bytes and width loss evidence survive beside the backup.
#[test]
fn migrates_classic_toml_backup_first_with_persistent_report() {
    let config_dir = tempdir().unwrap();
    let config = config_dir.path().join("config.toml");
    let original = "# Classic root\n[workspace.right_sidebar]\ncommand = \"codex\"\nargs = [\"resume\"]\nwidth_percent = 37\n\n[core]\nskip_welcome_screen = true\n";
    fs::write(&config, original).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    }

    let result = migrate_with(
        &request(config_dir.path()),
        "20260712_010203",
        &RealTransactionIo,
    )
    .unwrap();

    assert_eq!(result.status, ClassicNovaMigrationStatus::Migrated);
    let backup = result.backup_path.unwrap();
    assert_eq!(fs::read_to_string(&backup).unwrap(), original);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(&config).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let migrated: Table = toml::from_str(&fs::read_to_string(&config).unwrap()).unwrap();
    assert_eq!(
        value_at(&migrated, "agent.command").and_then(Value::as_str),
        Some("codex")
    );
    assert_eq!(
        value_at(&migrated, "welcome.enabled").and_then(Value::as_bool),
        Some(false)
    );
    assert!(value_at(&migrated, "workspace.right_sidebar.width_percent").is_none());

    let report: JsonValue =
        serde_json::from_str(&fs::read_to_string(result.report_path.unwrap()).unwrap()).unwrap();
    assert_eq!(report["source"], "config_toml");
    assert!(report["entries"].as_array().unwrap().iter().any(|entry| {
        entry["source_path"] == "workspace.right_sidebar.width_percent"
            && entry["disposition"] == "removed"
    }));
}

// Defends: retired JSONC goes directly to sparse Nova TOML, reports native preference review, and never touches a sidecar.
#[test]
fn migrates_lone_jsonc_without_intermediate_classic_or_sidecar_writes() {
    let config_dir = tempdir().unwrap();
    let legacy = config_dir.path().join("settings.jsonc");
    let original = current_legacy_jsonc(
        "  \"editor\": { \"command\": \"nvim\" },\n  \"zellij\": { \"disable_tips\": false, \"pane_frames\": false, \"rounded_corners\": false, \"default_mode\": \"locked\", \"support_kitty_keyboard_protocol\": true }",
    );
    fs::write(&legacy, &original).unwrap();
    fs::create_dir_all(config_dir.path().join("zellij")).unwrap();
    let sidecar = config_dir.path().join("zellij/config.kdl");
    fs::write(&sidecar, "sentinel\n").unwrap();

    let result = migrate_with(
        &request(config_dir.path()),
        "20260712_020304",
        &RealTransactionIo,
    )
    .unwrap();

    assert!(!legacy.exists());
    assert_eq!(fs::read_to_string(sidecar).unwrap(), "sentinel\n");
    let backup = result.backup_path.unwrap();
    assert_eq!(fs::read_to_string(backup).unwrap(), original);
    let config = fs::read_to_string(config_dir.path().join("config.toml")).unwrap();
    assert!(config.contains("command = \"nvim\""));
    assert!(!config.contains("disable_tips"));
    let report: JsonValue =
        serde_json::from_str(&fs::read_to_string(result.report_path.unwrap()).unwrap()).unwrap();
    assert_eq!(report["source"], "settings_jsonc");
    assert!(report["entries"].as_array().unwrap().iter().any(|entry| {
        entry["source_path"] == "zellij.default_mode" && entry["disposition"] == "manual"
    }));
}

// Defends: competing owners and mixed schemas fail before any backup, report, or replacement is written.
#[test]
fn rejects_coexistence_and_mixed_schema_without_writes() {
    let config_dir = tempdir().unwrap();
    let config = config_dir.path().join("config.toml");
    let legacy = config_dir.path().join("settings.jsonc");
    fs::write(&config, "[core]\ndebug_mode = false\n").unwrap();
    fs::write(&legacy, "{}\n").unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "classic_nova_root_coexistence");
    assert_eq!(fs::read_dir(config_dir.path()).unwrap().count(), 2);

    fs::remove_file(legacy).unwrap();
    let mixed = "[core]\ndebug_mode = false\n[welcome]\nenabled = true\n";
    fs::write(&config, mixed).unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "mixed_classic_nova_root");
    assert_eq!(fs::read_to_string(config).unwrap(), mixed);
    assert!(
        fs::read_dir(config_dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .all(|entry| !entry.file_name().to_string_lossy().contains("backup"))
    );
}

// Defends: malformed syntax, invalid Nova values, and pre-existing transaction artifacts all stop before source mutation.
#[test]
fn rejects_malformed_or_colliding_transaction_inputs() {
    let config_dir = tempdir().unwrap();
    let config = config_dir.path().join("config.toml");
    fs::write(&config, "[core\n").unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "invalid_classic_nova_root_toml");
    assert_eq!(fs::read_to_string(&config).unwrap(), "[core\n");

    fs::write(&config, "[welcome]\nduration_seconds = 0\n").unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "invalid_nova_root");

    let classic = "[core]\nskip_welcome_screen = true\n";
    fs::write(&config, classic).unwrap();
    let backup = config_dir.path().join("config.toml.backup-20260712_000000");
    fs::write(&backup, "existing backup\n").unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "classic_nova_migration_artifact_collision");
    assert_eq!(fs::read_to_string(&config).unwrap(), classic);
    assert_eq!(fs::read_to_string(backup).unwrap(), "existing backup\n");
}

// Regression: malformed legacy-native JSONC values, unknown schema roots, and embedded cursors fail without migration artifacts.
#[test]
fn rejects_malformed_jsonc_and_ambiguous_root_ownership() {
    let config_dir = tempdir().unwrap();
    let legacy = config_dir.path().join("settings.jsonc");
    let malformed = current_legacy_jsonc(
        "  \"zellij\": { \"pane_frames\": \"sometimes\", \"support_kitty_keyboard_protocol\": true }",
    );
    fs::write(&legacy, &malformed).unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "invalid_legacy_native_zellij_value");
    assert_eq!(fs::read_to_string(&legacy).unwrap(), malformed);
    assert_eq!(fs::read_dir(config_dir.path()).unwrap().count(), 1);

    fs::remove_file(&legacy).unwrap();
    let config = config_dir.path().join("config.toml");
    fs::write(&config, "[mystery]\nenabled = true\n").unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "ambiguous_root_schema");

    fs::write(&config, "[cursors]\nenabled_cursors = [\"reef\"]\n").unwrap();
    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "embedded_cursor_settings_unsupported");
    assert_eq!(fs::read_dir(config_dir.path()).unwrap().count(), 1);
}

// Defends: mapping collisions are not silently converted into lossy reports.
#[test]
fn rejects_mapping_collisions_before_backup() {
    let config_dir = tempdir().unwrap();
    let config = config_dir.path().join("config.toml");
    fs::write(
        &config,
        "[[zellij.custom_popups]]\nid = \"one\"\ncommand = [\"btm\"]\nkeybindings = [\"Alt Shift B\"]\n\n[[zellij.custom_popups]]\nid = \"two\"\ncommand = [\"lazygit\"]\nkeybindings = [\"Alt Shift B\"]\n",
    )
    .unwrap();

    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "classic_nova_mapping_collision");
    assert_eq!(fs::read_dir(config_dir.path()).unwrap().count(), 1);
}

// Defends: read-only and Home Manager-owned Classic state is never replaced automatically.
#[cfg(unix)]
#[test]
fn rejects_read_only_and_home_manager_owned_sources() {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let read_only_dir = tempdir().unwrap();
    let read_only = read_only_dir.path().join("config.toml");
    fs::write(&read_only, "[core]\ndebug_mode = false\n").unwrap();
    fs::set_permissions(&read_only, fs::Permissions::from_mode(0o444)).unwrap();
    let error = migrate_with(
        &request(read_only_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    fs::set_permissions(&read_only, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(error.code(), "read_only_root_migration");

    let hm_dir = tempdir().unwrap();
    let store = hm_dir.path().join("generation-home-manager-files");
    fs::create_dir(&store).unwrap();
    let owned = store.join("config.toml");
    fs::write(&owned, "[core]\ndebug_mode = false\n").unwrap();
    symlink(&owned, hm_dir.path().join("config.toml")).unwrap();
    let error = migrate_with(
        &request(hm_dir.path()),
        "20260712_000000",
        &RealTransactionIo,
    )
    .unwrap_err();
    assert_eq!(error.code(), "home_manager_owned_root_migration");
    assert!(
        fs::symlink_metadata(hm_dir.path().join("config.toml"))
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

struct FailTargetWrite {
    target: PathBuf,
}

struct FailRemoval {
    source: PathBuf,
    target: Option<PathBuf>,
}

impl TransactionIo for FailRemoval {
    fn copy(&self, source: &Path, target: &Path) -> io::Result<()> {
        fs::copy(source, target).map(|_| ())
    }

    fn write_atomic(
        &self,
        path: &Path,
        contents: &str,
        permissions: Option<&fs::Permissions>,
    ) -> Result<(), CoreError> {
        test_write_atomic(path, contents, permissions)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        if path == self.source || self.target.as_deref() == Some(path) {
            return Err(io::Error::other("injected removal failure"));
        }
        fs::remove_file(path)
    }
}

impl TransactionIo for FailTargetWrite {
    fn copy(&self, source: &Path, target: &Path) -> io::Result<()> {
        fs::copy(source, target).map(|_| ())
    }

    fn write_atomic(
        &self,
        path: &Path,
        contents: &str,
        permissions: Option<&fs::Permissions>,
    ) -> Result<(), CoreError> {
        if path == self.target {
            return Err(CoreError::io(
                "injected_target_write_failure",
                "Injected target write failure",
                "Test-only failure",
                path.display().to_string(),
                io::Error::other("injected"),
            ));
        }
        test_write_atomic(path, contents, permissions)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

fn test_write_atomic(
    path: &Path,
    contents: &str,
    permissions: Option<&fs::Permissions>,
) -> Result<(), CoreError> {
    match permissions {
        Some(permissions) => write_text_atomic_with_permissions(path, contents, permissions),
        None => write_text_atomic(path, contents),
    }
}

// Regression: the byte-preserving backup exists before target I/O and an atomic-write failure leaves the source untouched.
#[test]
fn target_write_failure_preserves_original_after_backup() {
    let config_dir = tempdir().unwrap();
    let config = config_dir.path().join("config.toml");
    let original = "[core]\nskip_welcome_screen = true\n";
    fs::write(&config, original).unwrap();

    let error = migrate_with(
        &request(config_dir.path()),
        "20260712_030405",
        &FailTargetWrite {
            target: config.clone(),
        },
    )
    .unwrap_err();

    assert_eq!(error.code(), "injected_target_write_failure");
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    assert_eq!(
        fs::read_to_string(config_dir.path().join("config.toml.backup-20260712_030405")).unwrap(),
        original
    );
    assert!(
        artifact(
            config_dir.path(),
            "config.toml.backup-",
            ".migration_report.json"
        )
        .exists()
    );
}

// Regression: failed JSONC retirement rolls back the complete target, and a failed rollback is surfaced with both files preserved.
#[test]
fn jsonc_retirement_failure_rolls_back_or_reports_both_failures() {
    for fail_rollback in [false, true] {
        let config_dir = tempdir().unwrap();
        let source = config_dir.path().join("settings.jsonc");
        let target = config_dir.path().join("config.toml");
        let original = current_legacy_jsonc("  \"editor\": { \"command\": \"nvim\" }");
        fs::write(&source, &original).unwrap();

        let error = migrate_with(
            &request(config_dir.path()),
            if fail_rollback {
                "20260712_070809"
            } else {
                "20260712_060708"
            },
            &FailRemoval {
                source: source.clone(),
                target: fail_rollback.then(|| target.clone()),
            },
        )
        .unwrap_err();

        assert_eq!(fs::read_to_string(&source).unwrap(), original);
        if fail_rollback {
            assert_eq!(error.code(), "rollback_classic_nova_target");
            assert!(target.exists());
            assert!(
                error.details()["rollback_error"]
                    .as_str()
                    .unwrap()
                    .contains("injected removal failure")
            );
        } else {
            assert_eq!(error.code(), "retire_classic_settings_jsonc");
            assert!(!target.exists());
        }
    }
}

// Defends: a completed transaction is idempotent and never creates a second backup/report pair.
#[test]
fn completed_migration_is_idempotent() {
    let config_dir = tempdir().unwrap();
    fs::write(
        config_dir.path().join("config.toml"),
        "[shell]\ndefault_shell = \"fish\"\n",
    )
    .unwrap();
    migrate_with(
        &request(config_dir.path()),
        "20260712_040506",
        &RealTransactionIo,
    )
    .unwrap();
    let count = fs::read_dir(config_dir.path()).unwrap().count();

    let second = migrate_with(
        &request(config_dir.path()),
        "20260712_050607",
        &RealTransactionIo,
    )
    .unwrap();
    assert_eq!(second.status, ClassicNovaMigrationStatus::NovaUnchanged);
    assert_eq!(fs::read_dir(config_dir.path()).unwrap().count(), count);
}
