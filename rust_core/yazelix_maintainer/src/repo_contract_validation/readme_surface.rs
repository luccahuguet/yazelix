use super::{as_string_list, read_toml_file};
use crate::repo_validation::ValidationReport;
use std::fs;
use std::path::{Path, PathBuf};
use toml::{Table as TomlTable, Value as TomlValue};
use yazelix_core::control_plane::read_release_metadata_version;

const README_LATEST_SERIES_BEGIN: &str = "<!-- BEGIN GENERATED README LATEST SERIES -->";
const README_LATEST_SERIES_END: &str = "<!-- END GENERATED README LATEST SERIES -->";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadmeSyncResult {
    pub readme_path: PathBuf,
    pub title_changed: bool,
    pub series_changed: bool,
}

pub fn validate_readme_version(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let version = read_release_metadata_version(repo_root).map_err(|error| error.message())?;
    let readme_path = repo_root.join("README.md");
    let readme = fs::read_to_string(&readme_path)
        .map_err(|error| format!("Failed to read {}: {}", readme_path.display(), error))?;
    let first_line = readme.lines().next().unwrap_or_default().trim();
    let expected_title = format!("# Yazelix {version}");
    if first_line != expected_title {
        report.errors.push(format!(
            "README title/version drift detected. Expected '{}' but found '{}'.",
            expected_title, first_line
        ));
    }

    let expected_block = render_readme_latest_series_section(repo_root, &version)?;
    let actual_block = extract_readme_latest_series_section(&readme)?;
    if actual_block != expected_block {
        report.errors.push(
            "README generated latest-series block drift detected. Regenerate the managed block from docs/upgrade_notes.toml."
                .to_string(),
        );
    }

    Ok(report)
}

pub fn sync_readme_surface(
    repo_root: &Path,
    readme_path: Option<&Path>,
    version: Option<&str>,
) -> Result<ReadmeSyncResult, String> {
    let resolved_version = match version.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => value.to_string(),
        None => read_release_metadata_version(repo_root).map_err(|error| error.message())?,
    };
    let target_readme_path = readme_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| repo_root.join("README.md"));
    let contents = fs::read_to_string(&target_readme_path).map_err(|error| {
        format!(
            "Failed to read README surface {}: {}",
            target_readme_path.display(),
            error
        )
    })?;
    let normalized = contents.replace("\r\n", "\n");
    let expected_title = format!("# Yazelix {resolved_version}");
    let (title_updated, title_changed) = replace_readme_title(&normalized, &expected_title);
    let rendered = render_readme_latest_series_section(repo_root, &resolved_version)?;
    let current_block = extract_readme_latest_series_section(&title_updated)?;
    let series_changed = current_block != rendered;
    let updated = if series_changed {
        title_updated.replacen(&current_block, &rendered, 1)
    } else {
        title_updated
    };

    if title_changed || series_changed {
        fs::write(&target_readme_path, updated).map_err(|error| {
            format!(
                "Failed to write README surface {}: {}",
                target_readme_path.display(),
                error
            )
        })?;
    }

    Ok(ReadmeSyncResult {
        readme_path: target_readme_path,
        title_changed,
        series_changed,
    })
}

fn replace_readme_title(contents: &str, expected_title: &str) -> (String, bool) {
    let mut lines = contents.lines().map(str::to_string).collect::<Vec<_>>();
    let had_trailing_newline = contents.ends_with('\n');
    if lines.is_empty() {
        return (format!("{expected_title}\n"), true);
    }
    if lines[0] == expected_title {
        return (contents.to_string(), false);
    }
    if lines[0].starts_with("# Yazelix v") {
        lines[0] = expected_title.to_string();
        let mut updated = lines.join("\n");
        if had_trailing_newline {
            updated.push('\n');
        }
        return (updated, true);
    }
    (contents.to_string(), false)
}

fn extract_readme_latest_series_section(contents: &str) -> Result<String, String> {
    let normalized = contents.replace("\r\n", "\n");
    let Some(start_index) = normalized.find(README_LATEST_SERIES_BEGIN) else {
        return Err("README is missing the generated latest-series start marker".to_string());
    };
    let after_start = start_index + README_LATEST_SERIES_BEGIN.len();
    let Some(relative_end_index) = normalized[after_start..].find(README_LATEST_SERIES_END) else {
        return Err("README is missing the generated latest-series end marker".to_string());
    };
    let end_index = after_start + relative_end_index + README_LATEST_SERIES_END.len();
    Ok(normalized[start_index..end_index].to_string())
}

fn render_readme_latest_series_section(repo_root: &Path, version: &str) -> Result<String, String> {
    let entries = resolve_readme_latest_release_entries(repo_root, version)?;
    let mut lines = vec![
        README_LATEST_SERIES_BEGIN.to_string(),
        "## Latest Tagged Releases".to_string(),
        String::new(),
    ];

    for (index, (entry_key, entry)) in entries.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("### {entry_key}"));
        lines.push(String::new());

        let headline = entry
            .get("headline")
            .and_then(TomlValue::as_str)
            .unwrap_or_default()
            .trim();
        if !headline.is_empty() {
            lines.push(headline.to_string());
            lines.push(String::new());
        }

        for item in as_string_list(entry.get("summary")) {
            lines.push(format!("- {}", trim_readme_release_summary_item(&item)));
        }
    }
    lines.extend([
        String::new(),
        "For exact tagged release notes, see [CHANGELOG](./CHANGELOG.md) or run `yzx whats_new` after installing that release".to_string(),
        "For the longer project story, see [Version History](./docs/history.md)".to_string(),
        README_LATEST_SERIES_END.to_string(),
    ]);

    Ok(lines.join("\n"))
}

fn trim_readme_release_summary_item(item: &str) -> &str {
    item.trim_end().strip_suffix('.').unwrap_or(item).trim_end()
}

fn resolve_readme_latest_release_entries(
    repo_root: &Path,
    version: &str,
) -> Result<Vec<(String, TomlTable)>, String> {
    resolve_readme_latest_release_entries_with_limit(repo_root, version, 2)
}

fn resolve_readme_latest_release_entries_with_limit(
    repo_root: &Path,
    version: &str,
    limit: usize,
) -> Result<Vec<(String, TomlTable)>, String> {
    let notes = read_toml_file(&repo_root.join("docs").join("upgrade_notes.toml"))?;
    let releases = notes
        .get("releases")
        .and_then(TomlValue::as_table)
        .ok_or("upgrade notes are missing the `releases` table")?;

    let mut release_entries = releases
        .iter()
        .filter_map(|(key, value)| {
            if key == "unreleased" || !is_major_release_key(key) {
                return None;
            }
            value
                .as_table()
                .map(|entry| (key.to_string(), entry.clone()))
        })
        .collect::<Vec<_>>();
    release_entries.sort_by(|(left, _), (right, _)| compare_release_versions_desc(left, right));

    if release_entries.is_empty() {
        return Err("upgrade notes are missing tagged release entries".to_string());
    }

    let series_key = major_series_key(version)?;
    if release_entries.iter().any(|(key, _)| key == &series_key) {
        return Ok(release_entries.into_iter().take(limit).collect());
    }

    let series = notes
        .get("series")
        .and_then(TomlValue::as_table)
        .ok_or("upgrade notes are missing the `series` table")?;
    let entry = series
        .get(&series_key)
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            format!("upgrade notes are missing the current major series entry `{series_key}`")
        })?;
    Ok(vec![(series_key, entry.clone())])
}

fn is_major_release_key(version: &str) -> bool {
    let Some(rest) = version.trim().strip_prefix('v') else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|ch| ch.is_ascii_digit())
}

fn compare_release_versions_desc(left: &str, right: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let left_parts = parse_release_version_parts(left);
    let right_parts = parse_release_version_parts(right);
    let max_len = left_parts.len().max(right_parts.len());

    for index in 0..max_len {
        let left_part = *left_parts.get(index).unwrap_or(&0);
        let right_part = *right_parts.get(index).unwrap_or(&0);
        match left_part.cmp(&right_part) {
            Ordering::Equal => continue,
            ordering => return ordering.reverse(),
        }
    }

    Ordering::Equal
}

fn parse_release_version_parts(version: &str) -> Vec<u32> {
    version
        .trim()
        .strip_prefix('v')
        .unwrap_or(version.trim())
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

fn major_series_key(version: &str) -> Result<String, String> {
    let trimmed = version.trim();
    let Some(rest) = trimmed.strip_prefix('v') else {
        return Err(format!(
            "failed to derive a major series key from version `{version}`"
        ));
    };
    let digits = rest
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return Err(format!(
            "failed to derive a major series key from version `{version}`"
        ));
    }
    Ok(format!("v{digits}"))
}
