//! Rust-owned maintainer version bump policy and release-note rotation.

use crate::repo_contract_validation::sync_readme_surface;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionBumpResult {
    pub repo_root: PathBuf,
    pub previous_version: String,
    pub target_version: String,
    pub release_date: String,
    pub commit_message: String,
    pub commit_sha: String,
    pub tag: String,
}

pub fn perform_version_bump(repo_root: &Path, target_version: &str) -> Result<VersionBumpResult, String> {
    let resolved_repo_root = repo_root
        .canonicalize()
        .map_err(|error| format!("Failed to resolve repo root {}: {}", repo_root.display(), error))?;
    let resolved_target_version = validate_target_version(target_version)?;
    let previous_version = current_version(&resolved_repo_root)?;

    if previous_version == resolved_target_version {
        return Err(format!(
            "YAZELIX_VERSION already matches {}",
            resolved_target_version
        ));
    }

    ensure_clean_git_worktree(&resolved_repo_root)?;
    ensure_target_tag_absent(&resolved_repo_root, &resolved_target_version)?;

    let release_date = chrono_like_today()?;
    rotate_upgrade_notes(
        &resolved_repo_root,
        &previous_version,
        &resolved_target_version,
        &release_date,
    )?;
    rotate_changelog(
        &resolved_repo_root,
        &previous_version,
        &resolved_target_version,
        &release_date,
    )?;
    update_version_constant(&resolved_repo_root, &resolved_target_version)?;
    sync_readme_surface(&resolved_repo_root, None, Some(&resolved_target_version))?;

    run_git(
        &resolved_repo_root,
        &[
            "add",
            "nushell/scripts/utils/constants.nu",
            "docs/upgrade_notes.toml",
            "CHANGELOG.md",
            "README.md",
        ],
    )?;
    let commit_message = format!("Bump version to {}", resolved_target_version);
    run_git(&resolved_repo_root, &["commit", "--quiet", "-m", &commit_message])?;
    run_git(
        &resolved_repo_root,
        &["tag", "-a", &resolved_target_version, "-m", &format!("Release {}", resolved_target_version)],
    )?;

    let final_version = current_version(&resolved_repo_root)?;
    if final_version != resolved_target_version {
        return Err(format!(
            "Version mismatch after bump: constants declare {}, expected {}",
            final_version, resolved_target_version
        ));
    }

    let commit_sha = run_git(&resolved_repo_root, &["rev-parse", "HEAD"])?;
    let created_tag = run_git(&resolved_repo_root, &["tag", "--list", &resolved_target_version])?;
    if created_tag != resolved_target_version {
        return Err(format!(
            "Failed to verify created git tag {}",
            resolved_target_version
        ));
    }

    Ok(VersionBumpResult {
        repo_root: resolved_repo_root,
        previous_version,
        target_version: resolved_target_version.clone(),
        release_date,
        commit_message,
        commit_sha,
        tag: resolved_target_version,
    })
}

fn chrono_like_today() -> Result<String, String> {
    let output = std::process::Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .map_err(|error| format!("Failed to run `date`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Failed to determine release date\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_git(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|error| format!("Failed to run git {}: {}", args.join(" "), error))?;
    if !output.status.success() {
        return Err(format!(
            "Git command failed: git -C {} {}\n{}",
            repo_root.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn ensure_clean_git_worktree(repo_root: &Path) -> Result<(), String> {
    let status = run_git(repo_root, &["status", "--porcelain"])?;
    if status.trim().is_empty() {
        Ok(())
    } else {
        Err("yzx dev bump requires a clean git worktree.".to_string())
    }
}

fn validate_target_version(target_version: &str) -> Result<String, String> {
    let normalized = target_version.trim();
    let valid = normalized
        .strip_prefix('v')
        .map(|tail| {
            !tail.is_empty()
                && tail
                    .split('.')
                    .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
        })
        .unwrap_or(false);
    if valid {
        Ok(normalized.to_string())
    } else {
        Err(format!(
            "Invalid version `{target_version}`. Expected a git tag like v14 or v14.1"
        ))
    }
}

fn current_version(repo_root: &Path) -> Result<String, String> {
    let constants_path = repo_root.join("nushell/scripts/utils/constants.nu");
    let raw = fs::read_to_string(&constants_path)
        .map_err(|error| format!("Failed to read {}: {}", constants_path.display(), error))?;
    let marker = "export const YAZELIX_VERSION = \"";
    let version = raw
        .lines()
        .find_map(|line| line.trim().strip_prefix(marker))
        .and_then(|tail| tail.strip_suffix('"'))
        .unwrap_or_default();
    if version.is_empty() {
        Err(format!(
            "Failed to read YAZELIX_VERSION from {}",
            constants_path.display()
        ))
    } else {
        Ok(version.to_string())
    }
}

fn ensure_target_tag_absent(repo_root: &Path, target_version: &str) -> Result<(), String> {
    let existing = run_git(repo_root, &["tag", "--list", target_version])?;
    if existing.trim().is_empty() {
        Ok(())
    } else {
        Err(format!("Tag already exists: {target_version}"))
    }
}

fn render_default_unreleased_summary(released_version: &str) -> Vec<String> {
    vec![format!(
        "Reserved for post-release changes after {} lands.",
        released_version
    )]
}

fn build_default_unreleased_entry(released_version: &str) -> TomlValue {
    let mut table = toml::map::Map::new();
    table.insert("version".to_string(), TomlValue::String("unreleased".to_string()));
    table.insert("date".to_string(), TomlValue::String(String::new()));
    table.insert(
        "headline".to_string(),
        TomlValue::String(format!("Post-{} work in progress", released_version)),
    );
    table.insert(
        "summary".to_string(),
        TomlValue::Array(
            render_default_unreleased_summary(released_version)
                .into_iter()
                .map(TomlValue::String)
                .collect(),
        ),
    );
    table.insert(
        "upgrade_impact".to_string(),
        TomlValue::String("no_user_action".to_string()),
    );
    table.insert(
        "acknowledged_guarded_changes".to_string(),
        TomlValue::Array(Vec::new()),
    );
    table.insert("migration_ids".to_string(), TomlValue::Array(Vec::new()));
    table.insert("manual_actions".to_string(), TomlValue::Array(Vec::new()));
    TomlValue::Table(table)
}

fn render_default_unreleased_changelog(released_version: &str) -> String {
    [
        "## Unreleased".to_string(),
        String::new(),
        format!("Post-{} work in progress", released_version),
        String::new(),
        "Upgrade impact: no user action required".to_string(),
        String::new(),
        "Highlights:".to_string(),
        format!(
            "- Reserved for post-release changes after {} lands.",
            released_version
        ),
    ]
    .join("\n")
}

fn ensure_releasable_unreleased_entry(entry: &TomlValue, current_version: &str) -> Result<(), String> {
    if entry == &build_default_unreleased_entry(current_version) {
        Err(format!(
            "Refusing to bump version while docs/upgrade_notes.toml still has the untouched unreleased placeholder for {}.",
            current_version
        ))
    } else {
        Ok(())
    }
}

fn ensure_releasable_unreleased_changelog(
    unreleased_section: &str,
    current_version: &str,
) -> Result<(), String> {
    if unreleased_section.trim() == render_default_unreleased_changelog(current_version).trim() {
        Err(format!(
            "Refusing to bump version while CHANGELOG.md still has the untouched unreleased placeholder for {}.",
            current_version
        ))
    } else {
        Ok(())
    }
}

fn update_version_constant(repo_root: &Path, target_version: &str) -> Result<(), String> {
    let constants_path = repo_root.join("nushell/scripts/utils/constants.nu");
    let raw = fs::read_to_string(&constants_path)
        .map_err(|error| format!("Failed to read {}: {}", constants_path.display(), error))?;
    let updated = raw.replace(
        &format!("export const YAZELIX_VERSION = \"{}\"", current_version(repo_root)?),
        &format!("export const YAZELIX_VERSION = \"{}\"", target_version),
    );
    fs::write(&constants_path, updated)
        .map_err(|error| format!("Failed to write {}: {}", constants_path.display(), error))
}

fn rotate_upgrade_notes(
    repo_root: &Path,
    current_version: &str,
    target_version: &str,
    release_date: &str,
) -> Result<(), String> {
    let notes_path = repo_root.join("docs/upgrade_notes.toml");
    let raw = fs::read_to_string(&notes_path)
        .map_err(|error| format!("Failed to read {}: {}", notes_path.display(), error))?;
    let mut notes: TomlValue =
        toml::from_str(&raw).map_err(|error| format!("Failed to parse {}: {}", notes_path.display(), error))?;
    let releases = notes
        .get_mut("releases")
        .and_then(TomlValue::as_table_mut)
        .ok_or_else(|| "docs/upgrade_notes.toml is missing releases.unreleased".to_string())?;
    let unreleased = releases
        .get("unreleased")
        .cloned()
        .ok_or_else(|| "docs/upgrade_notes.toml is missing releases.unreleased".to_string())?;
    if releases.contains_key(target_version) {
        return Err(format!(
            "docs/upgrade_notes.toml already contains release entry `{}`",
            target_version
        ));
    }

    ensure_releasable_unreleased_entry(&unreleased, current_version)?;
    let mut released_entry = unreleased
        .as_table()
        .cloned()
        .ok_or_else(|| "releases.unreleased must be a table".to_string())?;
    released_entry.insert(
        "version".to_string(),
        TomlValue::String(target_version.to_string()),
    );
    released_entry.insert("date".to_string(), TomlValue::String(release_date.to_string()));

    let existing_entries = releases
        .iter()
        .filter(|(key, _)| key.as_str() != "unreleased" && key.as_str() != target_version)
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<_>>();

    releases.clear();
    releases.insert(
        "unreleased".to_string(),
        build_default_unreleased_entry(target_version),
    );
    releases.insert(target_version.to_string(), TomlValue::Table(released_entry));
    for (key, value) in existing_entries {
        releases.insert(key, value);
    }

    fs::write(&notes_path, toml::to_string(&notes).unwrap())
        .map_err(|error| format!("Failed to write {}: {}", notes_path.display(), error))
}

fn rotate_changelog(
    repo_root: &Path,
    current_version: &str,
    target_version: &str,
    release_date: &str,
) -> Result<(), String> {
    let changelog_path = repo_root.join("CHANGELOG.md");
    let raw = fs::read_to_string(&changelog_path)
        .map_err(|error| format!("Failed to read {}: {}", changelog_path.display(), error))?;
    let updated = rotate_changelog_text(&raw, current_version, target_version, release_date)?;
    fs::write(&changelog_path, updated)
        .map_err(|error| format!("Failed to write {}: {}", changelog_path.display(), error))
}

fn rotate_changelog_text(
    raw: &str,
    current_version: &str,
    target_version: &str,
    release_date: &str,
) -> Result<String, String> {
    let lines = raw.lines().collect::<Vec<_>>();
    let unreleased_index = lines
        .iter()
        .position(|line| *line == "## Unreleased")
        .ok_or_else(|| "CHANGELOG.md is missing the `## Unreleased` heading.".to_string())?;
    let next_heading_index = lines
        .iter()
        .enumerate()
        .skip(unreleased_index + 1)
        .find(|(_, line)| line.starts_with("## "))
        .map(|(index, _)| index)
        .unwrap_or(lines.len());
    let unreleased_section = lines[unreleased_index..next_heading_index].join("\n");
    ensure_releasable_unreleased_changelog(&unreleased_section, current_version)?;

    let mut final_lines = Vec::new();
    final_lines.extend(lines[..unreleased_index].iter().map(|line| line.to_string()));
    final_lines.extend(
        render_default_unreleased_changelog(target_version)
            .lines()
            .map(|line| line.to_string()),
    );
    final_lines.push(String::new());
    final_lines.push(format!("## {} - {}", target_version, release_date));
    final_lines.extend(
        lines[unreleased_index + 1..next_heading_index]
            .iter()
            .map(|line| line.to_string()),
    );
    if next_heading_index < lines.len() {
        final_lines.push(String::new());
        final_lines.extend(lines[next_heading_index..].iter().map(|line| line.to_string()));
    }
    Ok(final_lines.join("\n") + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default
    // Defends: version bump validation still accepts the Yazelix tag grammar without reviving loose maintainer-only version aliases.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn validate_target_version_accepts_only_yazelix_tag_shape() {
        assert_eq!(validate_target_version("v14").unwrap(), "v14");
        assert_eq!(validate_target_version("v14.1").unwrap(), "v14.1");
        assert!(validate_target_version("14").is_err());
        assert!(validate_target_version("v14-beta").is_err());
    }

    // Defends: changelog rotation still turns the unreleased section into a released heading and reinstates the post-release placeholder.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rotate_changelog_releases_unreleased_section() {
        let raw = "\
## Unreleased

Ready to ship

Upgrade impact: no user action required

Highlights:
- Important change

## v15.3 - 2026-04-21

Older release
";
        let rotated = rotate_changelog_text(raw, "v15.3", "v15.4", "2026-04-23").unwrap();
        assert!(rotated.contains("## Unreleased"));
        assert!(rotated.contains("Post-v15.4 work in progress"));
        assert!(rotated.contains("## v15.4 - 2026-04-23"));
        assert!(rotated.contains("- Important change"));
    }
}
