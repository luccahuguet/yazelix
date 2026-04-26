// Test lane: maintainer

use crate::repo_validation::ValidationReport;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const RUST_BUDGET_RELATIVE_PATH: &str = "config_metadata/rust_ownership_budget.toml";
const ALLOWED_RUST_BUDGET_STATUSES: &[&str] =
    &["canonical", "canonical_maintainer", "extension_surface"];

#[derive(Debug, Deserialize)]
struct RustBudgetManifest {
    contract: RustBudgetContract,
    families: Vec<RustBudgetFamily>,
}

#[derive(Debug, Deserialize)]
struct RustBudgetContract {
    measured_total_loc: usize,
    measured_total_file_count: usize,
    current_total_loc_max: usize,
    current_total_file_count_max: usize,
    hard_target_loc: usize,
}

#[derive(Debug, Deserialize)]
struct RustBudgetFamily {
    id: String,
    status: String,
    owner_bead: String,
    measured_loc: usize,
    measured_files: usize,
    max_loc: usize,
    target_loc: usize,
    max_files: usize,
    allowed_paths: Vec<String>,
}

pub fn validate_rust_ownership_budget(repo_root: &Path) -> Result<ValidationReport, String> {
    let manifest = load_rust_budget_manifest(repo_root)?;
    let actual_paths = load_rust_source_paths(repo_root)?;
    let mut report = ValidationReport::default();
    let mut family_index_by_id = HashMap::new();
    let mut family_index_by_path = HashMap::new();
    let mut family_loc_totals = vec![0usize; manifest.families.len()];
    let mut family_file_totals = vec![0usize; manifest.families.len()];

    for (index, family) in manifest.families.iter().enumerate() {
        if !ALLOWED_RUST_BUDGET_STATUSES.contains(&family.status.as_str()) {
            report.errors.push(format!(
                "Rust budget family `{}` declares unsupported status `{}` in {}",
                family.id, family.status, RUST_BUDGET_RELATIVE_PATH
            ));
        }

        if family.owner_bead.trim().is_empty() {
            report.errors.push(format!(
                "Rust budget family `{}` is missing an owner_bead in {}",
                family.id, RUST_BUDGET_RELATIVE_PATH
            ));
        }

        if family.measured_loc > family.max_loc {
            report.errors.push(format!(
                "Rust budget family `{}` has measured_loc {} above max_loc {} in {}",
                family.id, family.measured_loc, family.max_loc, RUST_BUDGET_RELATIVE_PATH
            ));
        }

        if family.target_loc > family.max_loc {
            report.errors.push(format!(
                "Rust budget family `{}` has target_loc {} above max_loc {} in {}",
                family.id, family.target_loc, family.max_loc, RUST_BUDGET_RELATIVE_PATH
            ));
        }

        if family.measured_files > family.max_files {
            report.errors.push(format!(
                "Rust budget family `{}` has measured_files {} above max_files {} in {}",
                family.id, family.measured_files, family.max_files, RUST_BUDGET_RELATIVE_PATH
            ));
        }

        if family.allowed_paths.is_empty() {
            report.errors.push(format!(
                "Rust budget family `{}` has no allowed_paths in {}",
                family.id, RUST_BUDGET_RELATIVE_PATH
            ));
        }

        if let Some(existing_index) = family_index_by_id.insert(family.id.clone(), index) {
            let existing_id = &manifest.families[existing_index].id;
            report.errors.push(format!(
                "Rust budget family id `{}` is duplicated in {}",
                existing_id, RUST_BUDGET_RELATIVE_PATH
            ));
        }

        for relative_path in &family.allowed_paths {
            if !is_valid_rust_budget_path(relative_path) {
                report.errors.push(format!(
                    "Rust budget family `{}` lists a noncanonical Rust path `{}` in {}",
                    family.id, relative_path, RUST_BUDGET_RELATIVE_PATH
                ));
                continue;
            }

            if let Some(previous_family_index) =
                family_index_by_path.insert(relative_path.clone(), index)
            {
                let previous_family = &manifest.families[previous_family_index].id;
                report.errors.push(format!(
                    "Rust budget path `{}` is assigned to both `{}` and `{}` in {}",
                    relative_path, previous_family, family.id, RUST_BUDGET_RELATIVE_PATH
                ));
            }

            if !repo_root.join(relative_path).is_file() {
                report.errors.push(format!(
                    "Rust budget path `{}` is listed under `{}` but does not exist in the repo",
                    relative_path, family.id
                ));
            }
        }
    }

    let mut total_loc = 0usize;
    let total_files = actual_paths.len();

    for relative_path in actual_paths {
        let Some(&family_index) = family_index_by_path.get(&relative_path) else {
            report.errors.push(format!(
                "Unexpected Rust file outside the canonical ownership budget: {}",
                relative_path
            ));
            continue;
        };

        let line_count = count_lines(&repo_root.join(&relative_path))?;
        family_loc_totals[family_index] += line_count;
        family_file_totals[family_index] += 1;
        total_loc += line_count;
    }

    let expected_measured_loc: usize = manifest
        .families
        .iter()
        .map(|family| family.measured_loc)
        .sum();
    let expected_measured_files: usize = manifest
        .families
        .iter()
        .map(|family| family.measured_files)
        .sum();
    let expected_total_loc: usize = manifest.families.iter().map(|family| family.max_loc).sum();
    let expected_total_files: usize = manifest
        .families
        .iter()
        .map(|family| family.max_files)
        .sum();

    if expected_measured_loc != manifest.contract.measured_total_loc {
        report.errors.push(format!(
            "Rust budget manifest measured LOC mismatch in {}: contract={}, family_sum={}",
            RUST_BUDGET_RELATIVE_PATH, manifest.contract.measured_total_loc, expected_measured_loc
        ));
    }

    if expected_measured_files != manifest.contract.measured_total_file_count {
        report.errors.push(format!(
            "Rust budget manifest measured file-count mismatch in {}: contract={}, family_sum={}",
            RUST_BUDGET_RELATIVE_PATH,
            manifest.contract.measured_total_file_count,
            expected_measured_files
        ));
    }

    if expected_total_loc != manifest.contract.current_total_loc_max {
        report.errors.push(format!(
            "Rust budget manifest total LOC mismatch in {}: contract={}, family_sum={}",
            RUST_BUDGET_RELATIVE_PATH, manifest.contract.current_total_loc_max, expected_total_loc
        ));
    }

    if expected_total_files != manifest.contract.current_total_file_count_max {
        report.errors.push(format!(
            "Rust budget manifest total file-count mismatch in {}: contract={}, family_sum={}",
            RUST_BUDGET_RELATIVE_PATH,
            manifest.contract.current_total_file_count_max,
            expected_total_files
        ));
    }

    if total_loc > manifest.contract.current_total_loc_max {
        report.errors.push(format!(
            "Rust ownership budget grew above the tracked ceiling: measured {} LOC > allowed {} LOC",
            total_loc, manifest.contract.current_total_loc_max
        ));
    }

    if total_files > manifest.contract.current_total_file_count_max {
        report.errors.push(format!(
            "Rust file count grew above the tracked ceiling: measured {} files > allowed {} files",
            total_files, manifest.contract.current_total_file_count_max
        ));
    }

    for (index, family) in manifest.families.iter().enumerate() {
        let measured_loc = family_loc_totals[index];
        let measured_files = family_file_totals[index];

        if measured_loc > family.max_loc {
            report.errors.push(format!(
                "Rust budget family `{}` grew above its LOC ceiling: measured {} LOC > allowed {} LOC",
                family.id, measured_loc, family.max_loc
            ));
        }

        if measured_files > family.max_files {
            report.errors.push(format!(
                "Rust budget family `{}` grew above its file-count ceiling: measured {} files > allowed {} files",
                family.id, measured_files, family.max_files
            ));
        }
    }

    if total_loc > manifest.contract.hard_target_loc {
        report.warnings.push(format!(
            "Current tracked Rust surface is {} LOC, still above the long-term hard target of {} LOC. Reduce or justify ownership before raising ceilings.",
            total_loc, manifest.contract.hard_target_loc
        ));
    }

    Ok(report)
}

fn load_rust_budget_manifest(repo_root: &Path) -> Result<RustBudgetManifest, String> {
    let manifest_path = repo_root.join(RUST_BUDGET_RELATIVE_PATH);
    let content = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Failed to read {}: {}", manifest_path.display(), error))?;
    toml::from_str(&content)
        .map_err(|error| format!("Invalid TOML in {}: {}", manifest_path.display(), error))
}

fn load_rust_source_paths(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for root_name in ["rust_core", "rust_plugins"] {
        collect_rust_source_paths(repo_root, &repo_root.join(root_name), &mut paths)?;
    }
    paths.sort();
    Ok(paths)
}

fn collect_rust_source_paths(
    repo_root: &Path,
    directory: &Path,
    paths: &mut Vec<String>,
) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(directory)
        .map_err(|error| format!("Failed to read {}: {}", directory.display(), error))?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to read entry under {}: {}",
                directory.display(),
                error
            )
        })?;
        let path = entry.path();
        let file_name = entry.file_name();
        if file_name.to_string_lossy() == "target" {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| {
            format!("Failed to read file type for {}: {}", path.display(), error)
        })?;
        if file_type.is_dir() {
            collect_rust_source_paths(repo_root, &path, paths)?;
        } else if file_type.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        {
            paths.push(relative_path_string(repo_root, &path)?);
        }
    }
    Ok(())
}

fn relative_path_string(repo_root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(repo_root)
        .map_err(|error| {
            format!(
                "Failed to make {} relative to {}: {}",
                path.display(),
                repo_root.display(),
                error
            )
        })
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn is_valid_rust_budget_path(relative_path: &str) -> bool {
    (relative_path.starts_with("rust_core/") || relative_path.starts_with("rust_plugins/"))
        && relative_path.ends_with(".rs")
        && !relative_path.split('/').any(|part| part == "target")
}

fn count_lines(path: &Path) -> Result<usize, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    Ok(content.lines().count())
}

#[cfg(test)]
mod tests {
    use super::validate_rust_ownership_budget;
    use std::fs;
    use tempfile::tempdir;

    // Defends: the Rust ownership validator rejects unowned files and no-growth ceiling breaches instead of silently expanding the Rust surface.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[ignore = "Rust ownership budget is manual audit-only, not a default test gate"]
    #[test]
    fn rust_budget_rejects_unowned_files_and_growth() {
        let temp = tempdir().unwrap();
        let repo_root = temp.path();
        fs::create_dir_all(repo_root.join("config_metadata")).unwrap();
        fs::create_dir_all(repo_root.join("rust_core/yazelix_core/src")).unwrap();
        fs::write(
            repo_root.join("rust_core/yazelix_core/src/lib.rs"),
            "a\nb\n",
        )
        .unwrap();
        fs::write(repo_root.join("rust_core/yazelix_core/src/extra.rs"), "x\n").unwrap();
        fs::write(
            repo_root.join("config_metadata/rust_ownership_budget.toml"),
            r#"
[contract]
measured_total_loc = 1
measured_total_file_count = 1
current_total_loc_max = 1
current_total_file_count_max = 1
hard_target_loc = 1

[[families]]
id = "core_runtime"
status = "canonical"
owner_bead = "yazelix-test"
measured_loc = 1
measured_files = 1
max_loc = 1
target_loc = 1
max_files = 1
allowed_paths = [
  "rust_core/yazelix_core/src/lib.rs",
]
"#,
        )
        .unwrap();

        let report = validate_rust_ownership_budget(repo_root).unwrap();
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("Unexpected Rust file outside"))
        );
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("grew above its LOC ceiling"))
        );
    }
}
