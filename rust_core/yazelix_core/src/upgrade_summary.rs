//! Rust-owned upgrade-summary loading, rendering, and state tracking.

use crate::bridge::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct UpgradeNotesRegistry {
    #[serde(default)]
    releases: BTreeMap<String, UpgradeNoteEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpgradeNoteEntry {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub headline: String,
    #[serde(default)]
    pub summary: Vec<String>,
    #[serde(default = "default_upgrade_impact")]
    pub upgrade_impact: String,
    #[serde(default)]
    pub migration_ids: Vec<String>,
    #[serde(default)]
    pub manual_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpgradeSummaryReport {
    pub found: bool,
    pub version: String,
    pub notes_path: String,
    pub changelog_path: String,
    pub state_path: String,
    pub last_seen_version: Option<String>,
    pub matching_migrations: Vec<String>,
    pub matching_migration_ids: Vec<String>,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpgradeSummaryDisplayResult {
    #[serde(flatten)]
    pub report: UpgradeSummaryReport,
    pub shown: bool,
    pub reason: String,
}

fn default_upgrade_impact() -> String {
    "no_user_action".to_string()
}

fn upgrade_notes_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("docs").join("upgrade_notes.toml")
}

fn changelog_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("CHANGELOG.md")
}

fn summary_state_path(state_dir: &Path) -> PathBuf {
    state_dir
        .join("state")
        .join("upgrade_summary")
        .join("last_seen_version.txt")
}

fn load_upgrade_notes_registry(
    runtime_dir: &Path,
) -> Result<Option<UpgradeNotesRegistry>, CoreError> {
    let notes_path = upgrade_notes_path(runtime_dir);
    if !notes_path.is_file() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&notes_path).map_err(|source| {
        CoreError::io(
            "upgrade_notes_read",
            "Failed to read docs/upgrade_notes.toml.",
            "Restore docs/upgrade_notes.toml in the active Yazelix runtime, then retry.",
            notes_path.display().to_string(),
            source,
        )
    })?;
    let registry: UpgradeNotesRegistry = toml::from_str(&raw).map_err(|source| {
        CoreError::toml(
            "upgrade_notes_parse",
            "Failed to parse docs/upgrade_notes.toml.",
            "Fix docs/upgrade_notes.toml in the active Yazelix runtime, then retry.",
            notes_path.display().to_string(),
            source,
        )
    })?;
    Ok(Some(registry))
}

fn normalize_string_list(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn read_last_seen_upgrade_version(state_dir: &Path) -> Result<Option<String>, CoreError> {
    let state_path = summary_state_path(state_dir);
    if !state_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&state_path).map_err(|source| {
        CoreError::io(
            "upgrade_summary_state_read",
            "Failed to read the Yazelix upgrade-summary state file.",
            "Check permissions under the Yazelix state directory, then retry.",
            state_path.display().to_string(),
            source,
        )
    })?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}

fn write_last_seen_upgrade_version(state_dir: &Path, version: &str) -> Result<PathBuf, CoreError> {
    let state_path = summary_state_path(state_dir);
    let parent = state_path.parent().expect("upgrade summary state parent");
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "upgrade_summary_state_dir",
            "Failed to create the Yazelix upgrade-summary state directory.",
            "Check permissions under the Yazelix state directory, then retry.",
            parent.display().to_string(),
            source,
        )
    })?;

    let temporary_path = state_path.with_extension("txt.tmp");
    fs::write(&temporary_path, format!("{version}\n")).map_err(|source| {
        CoreError::io(
            "upgrade_summary_state_write",
            "Failed to write the Yazelix upgrade-summary state file.",
            "Check permissions under the Yazelix state directory, then retry.",
            temporary_path.display().to_string(),
            source,
        )
    })?;
    fs::rename(&temporary_path, &state_path).map_err(|source| {
        CoreError::io(
            "upgrade_summary_state_commit",
            "Failed to commit the Yazelix upgrade-summary state file.",
            "Check permissions under the Yazelix state directory, then retry.",
            state_path.display().to_string(),
            source,
        )
    })?;
    Ok(state_path)
}

fn render_upgrade_summary(entry: &UpgradeNoteEntry, changelog_path: &Path) -> String {
    let release_date = entry.date.trim();
    let headline = entry.headline.trim();
    let summary_items = normalize_string_list(&entry.summary);
    let migration_ids = normalize_string_list(&entry.migration_ids);
    let manual_actions = normalize_string_list(&entry.manual_actions);

    let mut lines = vec![
        String::new(),
        format!("=== What's New In Yazelix {} ===", entry.version),
        format!("Released: {release_date}"),
    ];

    if !headline.is_empty() {
        lines.push(headline.to_string());
    }

    if !summary_items.is_empty() {
        lines.push(String::new());
        lines.push("Highlights:".to_string());
        for item in summary_items {
            lines.push(format!("- {item}"));
        }
    }

    match entry.upgrade_impact.trim() {
        "migration_available" => {
            lines.push(String::new());
            lines.push(
                "Upgrade impact: this historical release included config-shape changes."
                    .to_string(),
            );
            lines.push(
                "Yazelix v15 no longer ships an automatic config migration engine.".to_string(),
            );
            lines.push(
                "If you are jumping from this release era, compare your config manually with the current template or run `yzx reset config` to start fresh."
                    .to_string(),
            );
        }
        "manual_action_required" => {
            lines.push(String::new());
            lines.push("Upgrade impact: manual follow-up is required.".to_string());
            for action in manual_actions {
                lines.push(format!("- {action}"));
            }
        }
        _ => {
            lines.push(String::new());
            lines.push("Upgrade impact: no user action required.".to_string());
            if !migration_ids.is_empty() {
                lines.push(format!(
                    "Recorded migration ids: {}",
                    migration_ids.join(", ")
                ));
            }
        }
    }

    lines.push(String::new());
    lines.push("Reopen later: `yzx whats_new`".to_string());
    lines.push(format!("Full notes: {}", changelog_path.display()));
    lines.join("\n")
}

pub fn current_release_headline(runtime_dir: &Path, version: &str) -> Result<String, CoreError> {
    let Some(entry) = get_upgrade_note_entry(runtime_dir, version)? else {
        return Ok(String::new());
    };
    Ok(entry.headline.trim().trim_end_matches('.').to_string())
}

pub fn get_upgrade_note_entry(
    runtime_dir: &Path,
    version: &str,
) -> Result<Option<UpgradeNoteEntry>, CoreError> {
    let Some(registry) = load_upgrade_notes_registry(runtime_dir)? else {
        return Ok(None);
    };
    Ok(registry.releases.get(version).cloned().map(|mut entry| {
        entry.version = version.to_string();
        entry
    }))
}

pub fn build_upgrade_summary_report(
    runtime_dir: &Path,
    state_dir: &Path,
    version: &str,
) -> Result<UpgradeSummaryReport, CoreError> {
    let notes_path = upgrade_notes_path(runtime_dir);
    let changelog_path = changelog_path(runtime_dir);
    let state_path = summary_state_path(state_dir);
    let last_seen_version = read_last_seen_upgrade_version(state_dir)?;
    let Some(entry) = get_upgrade_note_entry(runtime_dir, version)? else {
        return Ok(UpgradeSummaryReport {
            found: false,
            version: version.to_string(),
            notes_path: notes_path.display().to_string(),
            changelog_path: changelog_path.display().to_string(),
            state_path: state_path.display().to_string(),
            last_seen_version,
            matching_migrations: Vec::new(),
            matching_migration_ids: Vec::new(),
            output: String::new(),
        });
    };

    let output = render_upgrade_summary(&entry, &changelog_path);
    Ok(UpgradeSummaryReport {
        found: true,
        version: version.to_string(),
        notes_path: notes_path.display().to_string(),
        changelog_path: changelog_path.display().to_string(),
        state_path: state_path.display().to_string(),
        last_seen_version,
        matching_migrations: Vec::new(),
        matching_migration_ids: Vec::new(),
        output,
    })
}

pub fn maybe_show_first_run_upgrade_summary(
    runtime_dir: &Path,
    state_dir: &Path,
    version: &str,
) -> Result<UpgradeSummaryDisplayResult, CoreError> {
    let report = build_upgrade_summary_report(runtime_dir, state_dir, version)?;
    if !report.found {
        return Ok(UpgradeSummaryDisplayResult {
            report,
            shown: false,
            reason: "missing_release_entry".to_string(),
        });
    }

    if report.last_seen_version.as_deref() == Some(version) {
        return Ok(UpgradeSummaryDisplayResult {
            report,
            shown: false,
            reason: "already_seen".to_string(),
        });
    }

    let state_path = write_last_seen_upgrade_version(state_dir, version)?;
    let mut shown_report = report.clone();
    shown_report.state_path = state_path.display().to_string();
    shown_report.last_seen_version = Some(version.to_string());
    Ok(UpgradeSummaryDisplayResult {
        report: shown_report,
        shown: true,
        reason: "displayed".to_string(),
    })
}

pub fn show_current_upgrade_summary(
    runtime_dir: &Path,
    state_dir: &Path,
    version: &str,
    mark_seen: bool,
) -> Result<UpgradeSummaryDisplayResult, CoreError> {
    let report = build_upgrade_summary_report(runtime_dir, state_dir, version)?;
    if !report.found {
        return Err(CoreError::classified(
            crate::bridge::ErrorClass::Runtime,
            "missing_upgrade_notes",
            format!(
                "No upgrade notes found for {version}. Expected an entry in {}.",
                report.notes_path
            ),
            "Add the current version to docs/upgrade_notes.toml or reinstall Yazelix with a runtime that includes the matching release notes.",
            serde_json::json!({
                "version": version,
                "notes_path": report.notes_path,
            }),
        ));
    }

    if mark_seen {
        let state_path = write_last_seen_upgrade_version(state_dir, version)?;
        let mut shown_report = report.clone();
        shown_report.state_path = state_path.display().to_string();
        shown_report.last_seen_version = Some(version.to_string());
        Ok(UpgradeSummaryDisplayResult {
            report: shown_report,
            shown: true,
            reason: "displayed".to_string(),
        })
    } else {
        Ok(UpgradeSummaryDisplayResult {
            report,
            shown: true,
            reason: "displayed".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Test lane: default
    // Defends: current release headlines still trim trailing punctuation without inventing copy when the current version is missing.
    // Strength: defect=2 behavior=2 resilience=1 cost=2 uniqueness=1 total=8/10
    #[test]
    fn current_release_headline_trims_trailing_periods() {
        let tmp = tempdir().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        fs::create_dir_all(runtime_dir.join("docs")).unwrap();
        fs::write(runtime_dir.join("CHANGELOG.md"), "# Changelog\n").unwrap();
        fs::write(
            runtime_dir.join("docs").join("upgrade_notes.toml"),
            r#"
        [releases."v15.4"]
headline = "Faster startup."
"#,
        )
        .unwrap();

        assert_eq!(
            current_release_headline(&runtime_dir, "v15.4").unwrap(),
            "Faster startup"
        );
        assert_eq!(current_release_headline(&runtime_dir, "v15.5").unwrap(), "");
    }

    // Defends: the first-run upgrade summary still renders once, records the seen version, and keeps the current-version report available for manual reopen.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn upgrade_summary_first_run_and_manual_reopen_share_the_same_report() {
        let tmp = tempdir().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let state_dir = tmp.path().join("state");
        fs::create_dir_all(runtime_dir.join("docs")).unwrap();
        fs::create_dir_all(&state_dir).unwrap();
        fs::write(runtime_dir.join("CHANGELOG.md"), "# Changelog\n").unwrap();
        fs::write(
            runtime_dir.join("docs").join("upgrade_notes.toml"),
            r#"
        [releases."v15.4"]
headline = "Config migration follow-up after the v15.4 upgrade"
summary = [
  "This fixture simulates a historical release that mentioned config-shape changes.",
  "It should render historical guidance without probing or rewriting the current config."
]
upgrade_impact = "migration_available"
migration_ids = ["remove_zellij_widget_tray_layout", "remove_shell_enable_atuin"]
"#,
        )
        .unwrap();

        let first =
            maybe_show_first_run_upgrade_summary(&runtime_dir, &state_dir, "v15.4").unwrap();
        assert!(first.shown);
        assert_eq!(first.reason, "displayed");
        assert!(first.report.output.contains("What's New In Yazelix v15.4"));
        assert!(
            first
                .report
                .output
                .contains("historical release included config-shape changes")
        );

        let second =
            maybe_show_first_run_upgrade_summary(&runtime_dir, &state_dir, "v15.4").unwrap();
        assert!(!second.shown);
        assert_eq!(second.reason, "already_seen");

        let manual = show_current_upgrade_summary(&runtime_dir, &state_dir, "v15.4", true).unwrap();
        assert!(manual.report.output.contains("yzx reset config"));
        assert_eq!(manual.report.last_seen_version.as_deref(), Some("v15.4"));
    }

    // Defends: the retained historical v12/v13 floor still resolves exact-version upgrade notes through the Rust-owned summary loader.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn historical_upgrade_notes_floor_stays_loadable() {
        let runtime_dir = tempdir().unwrap();
        let state_dir = tempdir().unwrap();
        fs::create_dir_all(runtime_dir.path().join("docs")).unwrap();
        fs::write(
            runtime_dir.path().join("docs/upgrade_notes.toml"),
            r#"
[releases.v12]
version = "v12"
date = "2025-01-01"
headline = "v12 baseline"
summary = ["Historical floor fixture."]
upgrade_impact = "no_user_action"
migration_ids = []
manual_actions = []

[releases."v12.10"]
version = "v12.10"
date = "2025-02-10"
headline = "v12.10 baseline"
summary = ["Historical floor fixture."]
upgrade_impact = "no_user_action"
migration_ids = []
manual_actions = []

[releases."v13.2"]
version = "v13.2"
date = "2025-03-02"
headline = "v13.2 baseline"
summary = ["Historical floor fixture."]
upgrade_impact = "no_user_action"
migration_ids = []
manual_actions = []

[releases."v13.3"]
version = "v13.3"
date = "2025-03-03"
headline = "v13.3 baseline"
summary = ["Historical floor fixture."]
upgrade_impact = "no_user_action"
migration_ids = []
manual_actions = []

[releases."v13.7"]
version = "v13.7"
date = "2025-03-07"
headline = "v13.7 baseline"
summary = ["Historical floor fixture."]
upgrade_impact = "no_user_action"
migration_ids = []
manual_actions = []
"#,
        )
        .unwrap();
        fs::write(runtime_dir.path().join("CHANGELOG.md"), "# Changelog\n").unwrap();

        for version in ["v12", "v12.10", "v13.2", "v13.3", "v13.7"] {
            let report =
                build_upgrade_summary_report(runtime_dir.path(), state_dir.path(), version)
                    .unwrap();
            assert!(
                report.found,
                "missing historical upgrade-note entry for {version}"
            );
        }
    }
}
