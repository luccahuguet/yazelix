// Test lane: default
//! Maintainer checks for the user-facing docs entrypoint.

use crate::repo_validation::ValidationReport;
use serde_json::Value as JsonValue;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use yazelix_core::settings_surface::{parse_jsonc_value, read_settings_jsonc_value};

const DOCS_INDEX: &str = "docs/README.md";
const ROOT_README: &str = "README.md";
const MAIN_CONFIG_CONTRACT: &str = "config_metadata/main_config_contract.toml";

const MAIN_DOC_ROUTES: &[(&str, &str)] = &[
    ("Installation", "docs/installation.md"),
    ("yzx CLI", "docs/yzx_cli.md"),
    ("Keybindings", "docs/keybindings.md"),
    ("Customization", "docs/customization.md"),
    ("Troubleshooting", "docs/troubleshooting.md"),
    ("Terminal emulators", "docs/terminal_emulators.md"),
    ("Editor configuration", "docs/editor_configuration.md"),
    ("Yazi configuration", "docs/yazi-configuration.md"),
    ("Zellij configuration", "docs/zellij-configuration.md"),
    ("Layouts", "docs/layouts.md"),
    ("Architecture map", "docs/architecture_map.md"),
    (
        "Documentation architecture",
        "docs/documentation_architecture.md",
    ),
    (
        "Contract inventory",
        "docs/contracts/contracts_inventory.md",
    ),
];

const CURRENT_USER_DOCS: &[&str] = &[
    "docs/README.md",
    "docs/installation.md",
    "docs/customization.md",
    "docs/yzx_cli.md",
    "docs/keybindings.md",
    "docs/troubleshooting.md",
];

const FORBIDDEN_CURRENT_DOC_MARKERS: &[(&str, &str)] = &[
    (
        "yzx warp",
        "`yzx warp` has been removed from the current command surface",
    ),
    (
        "yazelix_packs.toml",
        "Pack sidecars are historical, not current user docs",
    ),
];

pub fn validate_docs_experience(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let docs_index = read_repo_text(repo_root, DOCS_INDEX)?;
    let root_readme = read_repo_text(repo_root, ROOT_README)?;

    if !root_readme.contains("[Yazelix Docs](./docs/README.md)") {
        report
            .errors
            .push("README.md must link to docs/README.md as the docs front door".to_string());
    }

    for (label, path) in MAIN_DOC_ROUTES {
        if !repo_root.join(path).is_file() {
            report
                .errors
                .push(format!("docs front door route target is missing: {path}"));
            continue;
        }

        let relative_link = path
            .strip_prefix("docs/")
            .map(|inner| format!("./{inner}"))
            .unwrap_or_else(|| format!("../{path}"));
        if !docs_index.contains(&relative_link) {
            report.errors.push(format!(
                "docs/README.md must link route `{label}` to `{relative_link}`"
            ));
        }
    }

    for doc in CURRENT_USER_DOCS {
        let content = match read_repo_text(repo_root, doc) {
            Ok(content) => content,
            Err(error) => {
                report.errors.push(error);
                continue;
            }
        };
        for (marker, reason) in FORBIDDEN_CURRENT_DOC_MARKERS {
            if content.contains(marker) {
                report
                    .errors
                    .push(format!("{doc} contains stale marker `{marker}`: {reason}"));
            }
        }
    }
    validate_internal_markdown_links(repo_root, &mut report.errors)?;
    validate_source_backed_keybinding_docs(repo_root, &mut report.errors)?;
    validate_source_backed_jsonc_examples(repo_root, &mut report.errors)?;

    Ok(report)
}

fn read_repo_text(repo_root: &Path, relative_path: &str) -> Result<String, String> {
    let path = repo_root.join(relative_path);
    fs::read_to_string(&path).map_err(|error| format!("Failed to read {}: {error}", path.display()))
}

fn validate_internal_markdown_links(
    repo_root: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    for path in collect_markdown_docs(repo_root)? {
        let relative = relative_path(repo_root, &path);
        let raw = fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
        for target in markdown_link_targets(&raw) {
            if should_skip_markdown_link(&target) {
                continue;
            }
            let target_path = target.split('#').next().unwrap_or("").trim();
            if target_path.is_empty() {
                continue;
            }
            let raw_resolved = path.parent().unwrap_or(repo_root).join(target_path);
            let resolved = NormalizeLexically::normalize_lexically(raw_resolved.as_path());
            if !resolved.exists() {
                errors.push(format!(
                    "{} links to missing internal target `{}`",
                    relative, target
                ));
            }
        }
    }
    Ok(())
}

fn collect_markdown_docs(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut docs = vec![repo_root.join(ROOT_README)];
    collect_markdown_docs_in(&repo_root.join("docs"), &mut docs)?;
    docs.sort();
    Ok(docs)
}

fn collect_markdown_docs_in(dir: &Path, docs: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {error}", dir.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            collect_markdown_docs_in(&path, docs)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            docs.push(path);
        }
    }
    Ok(())
}

fn markdown_link_targets(raw: &str) -> Vec<String> {
    let mut in_fence = false;
    let mut links = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let mut rest = line;
        while let Some(label_end) = rest.find("](") {
            let after_label = &rest[label_end + 2..];
            let Some(target_end) = after_label.find(')') else {
                break;
            };
            let raw_target = after_label[..target_end]
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches('<')
                .trim_matches('>')
                .trim()
                .to_string();
            if !raw_target.is_empty() {
                links.push(raw_target);
            }
            rest = &after_label[target_end + 1..];
        }
    }
    links
}

fn should_skip_markdown_link(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    target.starts_with('#')
        || lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
}

trait NormalizeLexically {
    fn normalize_lexically(&self) -> PathBuf;
}

impl NormalizeLexically for Path {
    fn normalize_lexically(&self) -> PathBuf {
        let mut out = PathBuf::new();
        for component in self.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    out.pop();
                }
                other => out.push(other.as_os_str()),
            }
        }
        out
    }
}

fn relative_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn validate_source_backed_keybinding_docs(
    repo_root: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let defaults = read_settings_jsonc_value(&repo_root.join("settings_default.jsonc"))
        .map_err(|error| error.message())?;
    let docs = read_repo_text(repo_root, "docs/keybindings.md")?;
    let expected = defaults
        .pointer("/zellij/keybindings/toggle_editor_right_sidebar_focus/0")
        .and_then(JsonValue::as_str)
        .ok_or_else(|| {
            "settings_default.jsonc is missing zellij.keybindings.toggle_editor_right_sidebar_focus[0]"
                .to_string()
        })?;
    let documented = markdown_table_key_for_action(&docs, "toggle_editor_right_sidebar_focus");
    if documented.as_deref() != Some(expected) {
        errors.push(format!(
            "docs/keybindings.md must document toggle_editor_right_sidebar_focus default key `{expected}`, got `{}`",
            documented.unwrap_or_else(|| "<missing>".to_string())
        ));
    }
    Ok(())
}

fn markdown_table_key_for_action(raw: &str, action: &str) -> Option<String> {
    raw.lines().find_map(|line| {
        if !line.contains(&format!("`{action}`")) {
            return None;
        }
        let cells = line
            .split('|')
            .map(str::trim)
            .filter(|cell| !cell.is_empty())
            .collect::<Vec<_>>();
        if cells.len() < 2 {
            return None;
        }
        Some(cells[1].trim_matches('`').to_string())
    })
}

fn validate_source_backed_jsonc_examples(
    repo_root: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let supported_fields = read_supported_config_fields(repo_root)?;
    let docs_path = repo_root.join("docs").join("zellij-configuration.md");
    let raw = fs::read_to_string(&docs_path)
        .map_err(|error| format!("Failed to read {}: {error}", docs_path.display()))?;
    let mut checked_count = 0usize;
    let mut saw_usage_periods_example = false;
    for snippet in fenced_code_blocks(&raw, &["json", "jsonc"]) {
        if !snippet.trim_start().starts_with('{') {
            continue;
        }
        checked_count += 1;
        if snippet.contains("codex_usage_periods") {
            saw_usage_periods_example = true;
        }
        let parsed = match parse_jsonc_value(&docs_path, &snippet) {
            Ok(parsed) => parsed,
            Err(error) => {
                errors.push(format!(
                    "docs/zellij-configuration.md has an invalid JSONC settings example: {}",
                    error.message()
                ));
                continue;
            }
        };
        for leaf in collect_json_leaf_paths(&parsed) {
            if !config_leaf_is_supported(&leaf, &supported_fields) {
                errors.push(format!(
                    "docs/zellij-configuration.md settings example uses unsupported key `{leaf}`"
                ));
            }
        }
    }
    if checked_count == 0 {
        errors.push(
            "docs/zellij-configuration.md must contain at least one JSONC settings example"
                .to_string(),
        );
    }
    if !saw_usage_periods_example {
        errors.push(
            "docs/zellij-configuration.md must keep a source-backed codex_usage_periods JSONC example"
                .to_string(),
        );
    }
    Ok(())
}

fn read_supported_config_fields(repo_root: &Path) -> Result<BTreeSet<String>, String> {
    let path = repo_root.join(MAIN_CONFIG_CONTRACT);
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    let parsed = toml::from_str::<toml::Value>(&raw)
        .map_err(|error| format!("Failed to parse {}: {error}", path.display()))?;
    let fields = parsed
        .get("fields")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("{} is missing [fields]", path.display()))?;
    Ok(fields.keys().cloned().collect())
}

fn fenced_code_blocks(raw: &str, accepted_langs: &[&str]) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut active_lang: Option<String> = None;
    let mut current = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("```") {
            if let Some(lang) = active_lang.take() {
                if accepted_langs.contains(&lang.as_str()) {
                    blocks.push(current.join("\n"));
                }
                current.clear();
            } else {
                let lang = rest.split_whitespace().next().unwrap_or("").to_string();
                active_lang = Some(lang);
            }
            continue;
        }
        if active_lang.is_some() {
            current.push(line.to_string());
        }
    }
    blocks
}

fn collect_json_leaf_paths(value: &JsonValue) -> Vec<String> {
    let mut out = Vec::new();
    collect_json_leaf_paths_inner(value, "", &mut out);
    out
}

fn collect_json_leaf_paths_inner(value: &JsonValue, prefix: &str, out: &mut Vec<String>) {
    match value {
        JsonValue::Object(map) => {
            for (key, child) in map {
                let next = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                collect_json_leaf_paths_inner(child, &next, out);
            }
        }
        _ => out.push(prefix.to_string()),
    }
}

fn config_leaf_is_supported(leaf: &str, supported_fields: &BTreeSet<String>) -> bool {
    supported_fields
        .iter()
        .any(|field| leaf == field || leaf.starts_with(&format!("{field}.")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(repo: &Path, relative_path: &str, content: &str) {
        let path = repo.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn docs_index() -> String {
        MAIN_DOC_ROUTES
            .iter()
            .map(|(_, path)| {
                let link = path.strip_prefix("docs/").unwrap();
                format!("- [route](./{link})\n")
            })
            .collect()
    }

    fn write_minimal_docs_fixture() -> (tempfile::TempDir, PathBuf) {
        let temp = tempdir().unwrap();
        let repo = temp.path().to_path_buf();
        write(
            &repo,
            ROOT_README,
            "# Yazelix\n\nSee [Yazelix Docs](./docs/README.md)\n",
        );
        write(&repo, DOCS_INDEX, &docs_index());
        for (_, path) in MAIN_DOC_ROUTES {
            write(&repo, path, "# Doc\n");
        }
        write(
            &repo,
            MAIN_CONFIG_CONTRACT,
            r#"[fields]
"zellij.codex_usage_periods" = { kind = "list" }
"zellij.keybindings" = { kind = "table" }
"#,
        );
        write(
            &repo,
            "settings_default.jsonc",
            r#"{
  "zellij": {
    "keybindings": {
      "toggle_editor_right_sidebar_focus": ["Ctrl Shift Y"]
    }
  }
}
"#,
        );
        write(
            &repo,
            "docs/keybindings.md",
            "| Action id | Default key |\n| --- | --- |\n| `toggle_editor_right_sidebar_focus` | `Ctrl Shift Y` |\n",
        );
        write(
            &repo,
            "docs/zellij-configuration.md",
            r#"# Zellij

```jsonc
{
  "zellij": {
    "codex_usage_periods": ["5h", "week"],
    "keybindings": {
      "toggle_editor_right_sidebar_focus": ["Ctrl Shift Y"]
    }
  }
}
```
"#,
        );
        (temp, repo)
    }

    // Defends: the docs validator accepts a complete front-door route map.
    #[test]
    fn docs_experience_validator_accepts_complete_route_map() {
        let (_temp, repo) = write_minimal_docs_fixture();

        let report = validate_docs_experience(&repo).unwrap();

        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    // Defends: docs front-door routes are end-to-end checked against real files instead of drifting into dead links.
    #[test]
    fn docs_experience_validator_rejects_missing_route_target() {
        let (_temp, repo) = write_minimal_docs_fixture();
        fs::remove_file(repo.join("docs/troubleshooting.md")).unwrap();

        let report = validate_docs_experience(&repo).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("docs/troubleshooting.md"))
        );
    }

    // Regression: current user docs must not route users toward the deleted workspace navigation command.
    #[test]
    fn docs_experience_validator_rejects_stale_current_user_command_marker() {
        let (_temp, repo) = write_minimal_docs_fixture();
        write(&repo, "docs/customization.md", "Use yzx warp here\n");

        let report = validate_docs_experience(&repo).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("`yzx warp` has been removed"))
        );
    }

    // Defends: internal Markdown links are checked across README and docs without pinning ordinary prose.
    #[test]
    fn docs_experience_validator_rejects_broken_internal_markdown_link() {
        let (_temp, repo) = write_minimal_docs_fixture();
        write(
            &repo,
            "docs/troubleshooting.md",
            "See [missing](./does-not-exist.md)\n",
        );

        let report = validate_docs_experience(&repo).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("does-not-exist.md"))
        );
    }

    // Regression: source-backed keybinding rows must track settings_default.jsonc instead of drifting silently.
    #[test]
    fn docs_experience_validator_rejects_stale_keybinding_row() {
        let (_temp, repo) = write_minimal_docs_fixture();
        write(
            &repo,
            "docs/keybindings.md",
            "| Action id | Default key |\n| --- | --- |\n| `toggle_editor_right_sidebar_focus` | `Ctrl y` |\n",
        );

        let report = validate_docs_experience(&repo).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("Ctrl Shift Y"))
        );
    }

    // Defends: JSONC settings examples in current docs must parse and stay on supported config keys.
    #[test]
    fn docs_experience_validator_rejects_unsupported_jsonc_example_key() {
        let (_temp, repo) = write_minimal_docs_fixture();
        write(
            &repo,
            "docs/zellij-configuration.md",
            r#"```jsonc
{
  "zellij": {
    "codex_usage_periods": ["5h", "week"],
    "unknown_setting": true
  }
}
```
"#,
        );

        let report = validate_docs_experience(&repo).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("zellij.unknown_setting"))
        );
    }
}
