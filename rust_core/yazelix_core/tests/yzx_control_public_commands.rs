// Test lane: default

use std::fs;

mod support;

use support::commands::{apply_managed_config_env, yzx_control_command};
use support::envelopes::stdout_text;
use support::fixtures::managed_config_fixture;

// Defends: `yzx reset config` preserves adjacent managed overrides and user-owned files instead of deleting them silently.
#[test]
fn yzx_control_reset_config_warns_about_preserved_adjacent_files() {
    let fixture = managed_config_fixture("");
    let settings_path = fixture.config_dir.join("config.toml");
    let legacy_cursor_path = fixture.config_dir.join("cursors.toml");
    let helix_override_path = fixture.config_dir.join("helix/config.toml");
    let legacy_helix_path = fixture.config_dir.join("helix.toml");
    let notes_path = fixture.config_dir.join("notes.txt");
    let settings_backup_path = fixture
        .config_dir
        .join("settings.jsonc.backup-20260505_000000");

    fs::write(&settings_path, "[editor]\ncommand = \"nvim\"\n").unwrap();
    fs::write(&legacy_cursor_path, "legacy cursor data").unwrap();
    fs::create_dir_all(helix_override_path.parent().unwrap()).unwrap();
    fs::write(&helix_override_path, "rainbow-brackets = true\n").unwrap();
    fs::write(&legacy_helix_path, "legacy helix data\n").unwrap();
    fs::write(&notes_path, "do not delete\n").unwrap();
    fs::write(&settings_backup_path, "old backup\n").unwrap();

    let mut command = yzx_control_command();
    apply_managed_config_env(&mut command, &fixture)
        .arg("reset")
        .arg("config")
        .arg("--yes")
        .arg("--no-backup");
    let stdout = stdout_text(command.output().unwrap());
    let reset = fs::read_to_string(&settings_path).unwrap();

    assert!(stdout.contains("only replaces config.toml"));
    assert!(stdout.contains("Managed override files were left untouched: helix"));
    assert!(
        stdout
            .contains("legacy Yazelix config inputs were left untouched: cursors.toml, helix.toml")
    );
    assert!(
        stdout.contains(
            "unknown adjacent entries in ~/.config/yazelix were left untouched: notes.txt"
        )
    );
    assert!(!stdout.contains("settings.jsonc.backup-20260505_000000"));
    assert!(reset.contains("[editor]"));
    assert!(!reset.contains("[cursors]"));
    assert!(helix_override_path.exists());
    assert!(legacy_cursor_path.exists());
    assert!(notes_path.exists());
    assert!(settings_backup_path.exists());
}
