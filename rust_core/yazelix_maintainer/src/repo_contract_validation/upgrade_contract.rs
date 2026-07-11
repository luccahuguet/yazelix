use super::{as_string_list, read_toml_file, sorted_keys};
use crate::repo_validation::ValidationReport;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use toml::{Table as TomlTable, Value as TomlValue};
use yazelix_core::control_plane::read_release_metadata_version;

const GUARDED_FILES: &[&str] = &[
    "release_metadata.toml",
    "config_default.toml",
    "home_manager/module.nix",
    "docs/upgrade_notes.toml",
    "CHANGELOG.md",
];
const ACK_REQUIRED_FILES: &[&str] = &["config_default.toml", "home_manager/module.nix"];
const HISTORICAL_ACKNOWLEDGEMENT_FILES: &[&str] = &[
    "settings_default.jsonc",
    "yazelix_default.toml",
    "yazelix_packs_default.toml",
    "nushell/scripts/utils/constants.nu",
    "nushell/scripts/utils/config_schema.nu",
    "nushell/scripts/utils/config_migrations.nu",
];
const IMPACT_VALUES: &[&str] = &[
    "no_user_action",
    "migration_available",
    "manual_action_required",
];

#[derive(Debug, Clone, Default)]
pub struct UpgradeContractOptions {
    pub ci: bool,
    pub diff_base: Option<String>,
}

pub fn validate_upgrade_contract(
    repo_root: &Path,
    options: &UpgradeContractOptions,
) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let changelog_path = repo_root.join("CHANGELOG.md");
    let notes_path = repo_root.join("docs").join("upgrade_notes.toml");

    if !changelog_path.is_file() {
        report.errors.push("CHANGELOG.md is missing".to_string());
    }
    if !notes_path.is_file() {
        report
            .errors
            .push("docs/upgrade_notes.toml is missing".to_string());
    }
    if !report.errors.is_empty() {
        return Ok(report);
    }

    let changelog = fs::read_to_string(&changelog_path)
        .map_err(|error| format!("Failed to read {}: {}", changelog_path.display(), error))?;
    let notes = read_toml_file(&notes_path)?;
    let entries = notes
        .get("releases")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            format!(
                "Failed to read {}: missing top-level [releases] table",
                notes_path.display()
            )
        })?;
    let current_version =
        read_release_metadata_version(repo_root).map_err(|error| error.message())?;

    let current_entry = entries.get(current_version.as_str());
    let unreleased_entry = entries.get("unreleased");

    if current_entry.is_none() {
        report.errors.push(format!(
            "docs/upgrade_notes.toml is missing the current release entry `{}`",
            current_version
        ));
    }
    if unreleased_entry.is_none() {
        report
            .errors
            .push("docs/upgrade_notes.toml is missing the `unreleased` entry".to_string());
    }

    if let Some(entry) = current_entry.and_then(TomlValue::as_table) {
        report
            .errors
            .extend(validate_upgrade_entry(&current_version, entry));
        report.errors.extend(validate_changelog_entry(
            &current_version,
            entry,
            &changelog,
        ));
    }
    if let Some(entry) = unreleased_entry.and_then(TomlValue::as_table) {
        report
            .errors
            .extend(validate_upgrade_entry("unreleased", entry));
        report
            .errors
            .extend(validate_changelog_entry("unreleased", entry, &changelog));
    }

    if options.ci {
        let diff_base = get_diff_base(options.diff_base.as_deref());
        let (warnings, errors) =
            validate_upgrade_ci_rules(repo_root, entries, &current_version, &diff_base)?;
        report.warnings.extend(warnings);
        report.errors.extend(errors);
    }

    Ok(report)
}

fn validate_upgrade_entry(key: &str, entry: &TomlTable) -> Vec<String> {
    let required_fields = [
        "version",
        "date",
        "headline",
        "summary",
        "upgrade_impact",
        "acknowledged_guarded_changes",
        "migration_ids",
        "manual_actions",
    ];
    let mut errors = Vec::new();

    for field in required_fields {
        if !entry.contains_key(field) {
            errors.push(format!(
                "upgrade_notes.toml: entry `{}` is missing required field `{}`",
                key, field
            ));
        }
    }
    if !errors.is_empty() {
        return errors;
    }

    let version = entry
        .get("version")
        .and_then(TomlValue::as_str)
        .unwrap_or_default()
        .trim();
    let date = entry
        .get("date")
        .and_then(TomlValue::as_str)
        .unwrap_or_default()
        .trim();
    let headline = entry
        .get("headline")
        .and_then(TomlValue::as_str)
        .unwrap_or_default()
        .trim();
    let summary = as_string_list(entry.get("summary"));
    let impact = entry
        .get("upgrade_impact")
        .and_then(TomlValue::as_str)
        .unwrap_or_default()
        .trim();
    let acknowledged = as_string_list(entry.get("acknowledged_guarded_changes"));
    let migration_ids = as_string_list(entry.get("migration_ids"));
    let manual_actions = as_string_list(entry.get("manual_actions"));

    if version != key {
        errors.push(format!(
            "upgrade_notes.toml: entry `{}` must declare version = `{}`",
            key, key
        ));
    }

    if key == "unreleased" {
        if !date.is_empty() {
            errors.push(
                "upgrade_notes.toml: `unreleased` must keep date empty until a real release exists"
                    .to_string(),
            );
        }
    } else if date.is_empty() {
        errors.push(format!(
            "upgrade_notes.toml: release entry `{}` must declare a real release date",
            key
        ));
    }

    if headline.is_empty() {
        errors.push(format!(
            "upgrade_notes.toml: entry `{}` must have a non-empty headline",
            key
        ));
    }
    if summary.is_empty() {
        errors.push(format!(
            "upgrade_notes.toml: entry `{}` must have a non-empty summary list",
            key
        ));
    }
    if !IMPACT_VALUES.contains(&impact) {
        errors.push(format!(
            "upgrade_notes.toml: entry `{}` has invalid upgrade_impact `{}`",
            key, impact
        ));
    }

    match impact {
        "no_user_action" => {
            if !migration_ids.is_empty() {
                errors.push(format!(
                    "upgrade_notes.toml: entry `{}` must keep migration_ids empty when upgrade_impact = no_user_action",
                    key
                ));
            }
            if !manual_actions.is_empty() {
                errors.push(format!(
                    "upgrade_notes.toml: entry `{}` must keep manual_actions empty when upgrade_impact = no_user_action",
                    key
                ));
            }
        }
        "migration_available" => {
            if key == "unreleased" {
                errors.push(
                    "upgrade_notes.toml: `unreleased` must not use migration_available because v15 no longer ships a live config migration engine"
                        .to_string(),
                );
            }
        }
        "manual_action_required" => {
            if manual_actions.is_empty() {
                errors.push(format!(
                    "upgrade_notes.toml: entry `{}` must list manual_actions when upgrade_impact = manual_action_required",
                    key
                ));
            }
        }
        _ => {}
    }

    for path in acknowledged {
        if !GUARDED_FILES.contains(&path.as_str())
            && !ACK_REQUIRED_FILES.contains(&path.as_str())
            && !HISTORICAL_ACKNOWLEDGEMENT_FILES.contains(&path.as_str())
        {
            errors.push(format!(
                "upgrade_notes.toml: entry `{}` acknowledges non-guarded path `{}`",
                key, path
            ));
        }
    }

    errors
}

fn validate_changelog_entry(key: &str, entry: &TomlTable, changelog: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let heading = if key == "unreleased" {
        "## Unreleased".to_string()
    } else {
        let date = entry
            .get("date")
            .and_then(TomlValue::as_str)
            .unwrap_or_default();
        format!("## {} - {}", key, date)
    };
    if !changelog.contains(&heading) {
        errors.push(format!("CHANGELOG.md: missing heading `{}`", heading));
    }
    let headline = entry
        .get("headline")
        .and_then(TomlValue::as_str)
        .unwrap_or_default();
    if !headline.is_empty() && !changelog.contains(headline) {
        errors.push(format!(
            "CHANGELOG.md: missing headline for `{}`: {}",
            key, headline
        ));
    }
    errors
}

fn get_diff_base(requested: Option<&str>) -> String {
    if let Some(value) = requested.map(str::trim).filter(|value| !value.is_empty()) {
        return value.to_string();
    }
    if let Ok(base_ref) = env::var("GITHUB_BASE_REF") {
        let trimmed = base_ref.trim();
        if !trimmed.is_empty() {
            return format!("origin/{}", trimmed);
        }
    }
    "HEAD~1".to_string()
}

fn validate_upgrade_ci_rules(
    repo_root: &Path,
    entries: &TomlTable,
    current_version: &str,
    diff_base: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    let changed_files = get_changed_files(repo_root, diff_base)?;
    let current_entry = entries.get(current_version);
    let unreleased_entry = entries.get("unreleased");
    let previous_version = get_previous_version(repo_root, diff_base)?;
    let version_bumped = previous_version
        .as_deref()
        .map(|previous| previous != current_version)
        .unwrap_or(false);
    let docs_changed = changed_files
        .iter()
        .any(|path| path == "docs/upgrade_notes.toml")
        && changed_files.iter().any(|path| path == "CHANGELOG.md");
    let one_doc_changed = (changed_files
        .iter()
        .any(|path| path == "docs/upgrade_notes.toml")
        || changed_files.iter().any(|path| path == "CHANGELOG.md"))
        && !docs_changed;
    let changed_ack_required = changed_files
        .iter()
        .filter(|path| ACK_REQUIRED_FILES.contains(&path.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let ack_only_notes_change = changed_files
        .iter()
        .any(|path| path == "docs/upgrade_notes.toml")
        && !changed_files.iter().any(|path| path == "CHANGELOG.md")
        && notes_changed_only_acknowledgements(repo_root, entries, diff_base)?;
    let series_only_notes_change = changed_files
        .iter()
        .any(|path| path == "docs/upgrade_notes.toml")
        && !changed_files.iter().any(|path| path == "CHANGELOG.md")
        && notes_changed_only_series(repo_root, diff_base)?;
    let target_key = if version_bumped {
        current_version.to_string()
    } else {
        "unreleased".to_string()
    };
    let target_entry = if target_key == "unreleased" {
        unreleased_entry
    } else {
        current_entry
    };
    let acknowledged = target_entry
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("acknowledged_guarded_changes"))
        .map(|value| as_string_list(Some(value)))
        .unwrap_or_default();
    let mut errors = Vec::new();

    if one_doc_changed && !ack_only_notes_change && !series_only_notes_change {
        errors
            .push("CI: CHANGELOG.md and docs/upgrade_notes.toml must change together".to_string());
    }
    if version_bumped && !docs_changed {
        errors.push(format!(
            "CI: version bump from {} to {} must update both CHANGELOG.md and docs/upgrade_notes.toml",
            previous_version.unwrap_or_default(),
            current_version
        ));
    }
    for path in changed_ack_required {
        if !acknowledged.contains(&path) {
            errors.push(format!(
                "CI: entry `{}` must acknowledge guarded change `{}`",
                target_key, path
            ));
        }
    }
    if !version_bumped
        && changed_files
            .iter()
            .any(|path| path == "release_metadata.toml")
        && !docs_changed
    {
        errors.push(
            "CI: changes to release_metadata.toml must update both CHANGELOG.md and docs/upgrade_notes.toml"
                .to_string(),
        );
    }

    let warnings = if errors.is_empty() {
        Vec::new()
    } else {
        vec![
            format!("Upgrade contract diff base: {}", diff_base),
            format!("Changed files: {}", changed_files.join(", ")),
            format!("Target upgrade-notes entry: {}", target_key),
            format!("Acknowledged guarded changes: {}", acknowledged.join(", ")),
        ]
    };

    Ok((warnings, errors))
}

fn notes_changed_only_acknowledgements(
    repo_root: &Path,
    entries: &TomlTable,
    diff_base: &str,
) -> Result<bool, String> {
    let Some(previous_notes) = load_notes_from_ref(repo_root, diff_base)? else {
        return Ok(false);
    };
    let Some(previous_entries) = previous_notes.get("releases").and_then(TomlValue::as_table)
    else {
        return Ok(false);
    };

    let current_keys = sorted_keys(entries);
    let previous_keys = sorted_keys(previous_entries);
    if current_keys != previous_keys {
        return Ok(false);
    }

    let changed_keys = current_keys
        .into_iter()
        .filter(|key| entries.get(key) != previous_entries.get(key))
        .collect::<Vec<_>>();
    if changed_keys.is_empty() {
        return Ok(false);
    }

    for key in changed_keys {
        let Some(current_entry) = entries.get(&key).and_then(TomlValue::as_table) else {
            return Ok(false);
        };
        let Some(previous_entry) = previous_entries.get(&key).and_then(TomlValue::as_table) else {
            return Ok(false);
        };
        if drop_acknowledged_guarded_changes(current_entry)
            != drop_acknowledged_guarded_changes(previous_entry)
        {
            return Ok(false);
        }
    }

    Ok(true)
}

fn notes_changed_only_series(repo_root: &Path, diff_base: &str) -> Result<bool, String> {
    let Some(previous_notes) = load_notes_from_ref(repo_root, diff_base)? else {
        return Ok(false);
    };
    let current_notes = read_toml_file(&repo_root.join("docs").join("upgrade_notes.toml"))?;
    let current_without_series = drop_optional_series(&current_notes);
    let previous_without_series = drop_optional_series(&previous_notes);
    if current_without_series != previous_without_series {
        return Ok(false);
    }

    Ok(current_notes.get("series") != previous_notes.get("series"))
}

fn load_notes_from_ref(repo_root: &Path, git_ref: &str) -> Result<Option<TomlTable>, String> {
    if !ref_exists(repo_root, git_ref)? {
        return Ok(None);
    }
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root.display().to_string(),
            "show",
            &format!("{git_ref}:docs/upgrade_notes.toml"),
        ])
        .output()
        .map_err(|error| format!("Failed to run `git show` for {}: {}", git_ref, error))?;
    if !output.status.success() {
        return Ok(None);
    }
    parse_toml_from_bytes(&output.stdout, "previous docs/upgrade_notes.toml").map(Some)
}

fn parse_toml_from_bytes(bytes: &[u8], label: &str) -> Result<TomlTable, String> {
    let raw = String::from_utf8(bytes.to_vec())
        .map_err(|error| format!("Failed to decode {} as UTF-8: {}", label, error))?;
    toml::from_str::<TomlTable>(&raw)
        .map_err(|error| format!("Failed to parse {} as TOML: {}", label, error))
}

fn ref_exists(repo_root: &Path, git_ref: &str) -> Result<bool, String> {
    let status = Command::new("git")
        .args([
            "-C",
            &repo_root.display().to_string(),
            "rev-parse",
            "--verify",
            git_ref,
        ])
        .output()
        .map_err(|error| format!("Failed to run `git rev-parse` for {}: {}", git_ref, error))?;
    Ok(status.status.success())
}

fn get_changed_files(repo_root: &Path, base: &str) -> Result<Vec<String>, String> {
    if !ref_exists(repo_root, base)? {
        return Ok(Vec::new());
    }
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root.display().to_string(),
            "diff",
            "--name-only",
            &format!("{base}..HEAD"),
        ])
        .output()
        .map_err(|error| format!("Failed to run `git diff` for {}: {}", base, error))?;
    if !output.status.success() {
        return Err(format!(
            "Failed to run `git diff --name-only {}..HEAD`\n{}",
            base,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn get_previous_version(repo_root: &Path, base: &str) -> Result<Option<String>, String> {
    if !ref_exists(repo_root, base)? {
        return Ok(None);
    }
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root.display().to_string(),
            "show",
            &format!("{base}:release_metadata.toml"),
        ])
        .output()
        .map_err(|error| format!("Failed to run `git show` for {}: {}", base, error))?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(extract_version_from_release_metadata(
        &String::from_utf8_lossy(&output.stdout),
    ))
}

fn extract_version_from_release_metadata(content: &str) -> Option<String> {
    toml::from_str::<TomlTable>(content)
        .ok()
        .and_then(|metadata| {
            metadata
                .get("version")
                .and_then(TomlValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })
}

fn drop_acknowledged_guarded_changes(entry: &TomlTable) -> TomlTable {
    let mut cloned = entry.clone();
    cloned.remove("acknowledged_guarded_changes");
    cloned
}

fn drop_optional_series(notes: &TomlTable) -> TomlTable {
    let mut cloned = notes.clone();
    cloned.remove("series");
    cloned
}
