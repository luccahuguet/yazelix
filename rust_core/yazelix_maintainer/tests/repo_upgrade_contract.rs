// Test lane: default

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;
use yazelix_maintainer::repo_contract_validation::{
    UpgradeContractOptions, validate_upgrade_contract,
};

fn write_fixture_repo() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempdir().unwrap();
    let fixture_root = tmp.path().join("repo");
    fs::create_dir_all(fixture_root.join("docs")).unwrap();
    fs::create_dir_all(fixture_root.join("nushell/scripts/utils")).unwrap();
    fs::write(
        fixture_root.join("CHANGELOG.md"),
        r#"## Unreleased

Post-v15.4 work in progress

Reserved for post-release changes after v15.4 lands.

## v15.4 - 2026-04-21

v15.4 Rust-owns public yzx families and deletes bridge seams
"#,
    )
    .unwrap();
    fs::write(
        fixture_root.join("docs/upgrade_notes.toml"),
        r#"[releases.unreleased]
version = "unreleased"
date = ""
headline = "Post-v15.4 work in progress"
summary = ["Reserved for post-release changes after v15.4 lands."]
upgrade_impact = "no_user_action"
acknowledged_guarded_changes = []
migration_ids = []
manual_actions = []

[releases."v15.4"]
version = "v15.4"
date = "2026-04-21"
headline = "v15.4 Rust-owns public yzx families and deletes bridge seams"
summary = ["Moved more public yzx command families onto Rust-owned execution paths."]
upgrade_impact = "no_user_action"
acknowledged_guarded_changes = []
migration_ids = []
manual_actions = []
"#,
    )
    .unwrap();
    fs::write(
        fixture_root.join("nushell/scripts/utils/constants.nu"),
        "export const YAZELIX_VERSION = \"v15.4\"\n",
    )
    .unwrap();
    run_git(&fixture_root, &["init", "--quiet"]);
    run_git(
        &fixture_root,
        &["config", "user.email", "codex@example.com"],
    );
    run_git(&fixture_root, &["config", "user.name", "Codex"]);
    run_git(&fixture_root, &["add", "-A"]);
    run_git(
        &fixture_root,
        &["commit", "--quiet", "-m", "Fixture baseline"],
    );
    (tmp, fixture_root)
}

fn run_git(repo_root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git command failed: {:?}", args);
}

// Defends: the Rust-owned upgrade-contract validator still rejects unreleased `migration_available` entries after the Nu E2E runner is deleted.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn validate_upgrade_contract_rejects_unreleased_migration_available() {
    let (_tmp, fixture_root) = write_fixture_repo();
    let notes_path = fixture_root.join("docs/upgrade_notes.toml");
    let raw = fs::read_to_string(&notes_path).unwrap();
    let updated = raw.replace(
        "upgrade_impact = \"no_user_action\"",
        "upgrade_impact = \"migration_available\"",
    );
    fs::write(&notes_path, updated).unwrap();

    let report =
        validate_upgrade_contract(&fixture_root, &UpgradeContractOptions::default()).unwrap();
    assert!(report.errors.iter().any(|error| error.contains(
        "must not use migration_available because v15 no longer ships a live config migration engine"
    )));
}

// Defends: the Rust-owned CI upgrade-contract validator still rejects user-facing upgrade-note edits that skip the matching changelog update.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn validate_upgrade_contract_ci_rejects_summary_without_changelog() {
    let (_tmp, fixture_root) = write_fixture_repo();
    let notes_path = fixture_root.join("docs/upgrade_notes.toml");
    let raw = fs::read_to_string(&notes_path).unwrap();
    let updated = raw.replace(
        "Reserved for post-release changes after",
        "A real unreleased note without the matching changelog update",
    );
    fs::write(&notes_path, updated).unwrap();
    run_git(&fixture_root, &["add", "-A"]);
    run_git(
        &fixture_root,
        &[
            "commit",
            "--quiet",
            "-m",
            "Change unreleased summary without changelog",
        ],
    );

    let report = validate_upgrade_contract(
        &fixture_root,
        &UpgradeContractOptions {
            ci: true,
            diff_base: Some("HEAD~1".to_string()),
        },
    )
    .unwrap();
    assert!(report.errors.iter().any(|error| {
        error.contains("CHANGELOG.md and docs/upgrade_notes.toml must change together")
    }));
}

// Defends: the Rust-owned CI upgrade-contract validator still accepts acknowledged guarded-note updates without forcing a changelog edit.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn validate_upgrade_contract_ci_accepts_ack_only_note_updates() {
    let (_tmp, fixture_root) = write_fixture_repo();
    let notes_path = fixture_root.join("docs/upgrade_notes.toml");
    let raw = fs::read_to_string(&notes_path).unwrap();
    let updated = raw.replace(
        "acknowledged_guarded_changes = []",
        "acknowledged_guarded_changes = [\"home_manager/module.nix\"]",
    );
    fs::write(&notes_path, updated).unwrap();
    run_git(&fixture_root, &["add", "-A"]);
    run_git(
        &fixture_root,
        &[
            "commit",
            "--quiet",
            "-m",
            "Ack guarded change in upgrade notes only",
        ],
    );

    let report = validate_upgrade_contract(
        &fixture_root,
        &UpgradeContractOptions {
            ci: true,
            diff_base: Some("HEAD~1".to_string()),
        },
    )
    .unwrap();
    assert!(report.errors.is_empty(), "{:?}", report.errors);
}
