// Test lane: default
//! Maintainer checks for the user-facing docs entrypoint and command reference.

use crate::repo_validation::ValidationReport;
use std::fs;
use std::path::Path;

const DOCS_INDEX: &str = "docs/README.md";
const ROOT_README: &str = "README.md";

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

const COMMAND_REFERENCE_MARKERS: &[&str] = &[
    "yzx launch",
    "yzx enter",
    "yzx warp",
    "yzx tutor begin",
    "yzx edit cursors",
    "yzx update upstream",
    "yzx update home_manager",
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
    ("yzx cwd", "Use `yzx warp` in current user docs"),
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

    let command_reference = read_repo_text(repo_root, "docs/yzx_cli.md")?;
    for marker in COMMAND_REFERENCE_MARKERS {
        if !command_reference.contains(marker) {
            report.errors.push(format!(
                "docs/yzx_cli.md is missing current command marker `{marker}`"
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

    Ok(report)
}

fn read_repo_text(repo_root: &Path, relative_path: &str) -> Result<String, String> {
    let path = repo_root.join(relative_path);
    fs::read_to_string(&path).map_err(|error| format!("Failed to read {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
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
            "docs/yzx_cli.md",
            "yzx launch\nyzx enter\nyzx warp\nyzx tutor begin\nyzx edit cursors\nyzx update upstream\nyzx update home_manager\n",
        );
        (temp, repo)
    }

    // Defends: the docs validator accepts a complete front-door route map and current command reference markers.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn docs_experience_validator_accepts_complete_route_map() {
        let (_temp, repo) = write_minimal_docs_fixture();

        let report = validate_docs_experience(&repo).unwrap();

        assert!(report.errors.is_empty(), "{:?}", report.errors);
    }

    // Defends: docs front-door routes are end-to-end checked against real files instead of drifting into dead links.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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

    // Regression: current user docs must not route users toward the deleted workspace retarget command name.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn docs_experience_validator_rejects_stale_current_user_command_marker() {
        let (_temp, repo) = write_minimal_docs_fixture();
        write(&repo, "docs/customization.md", "Use yzx cwd here\n");

        let report = validate_docs_experience(&repo).unwrap();

        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("Use `yzx warp`"))
        );
    }
}
