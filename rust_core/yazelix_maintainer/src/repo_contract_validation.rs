use crate::repo_validation::ValidationReport;
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

mod config_surface;
mod helpers;
mod installed_runtime;
mod nix_interface;
mod nix_package;
mod readme_surface;
mod upgrade_contract;

pub use config_surface::{
    validate_config_surface_contract, validate_home_manager_option_declaration_contract,
};
use helpers::{
    as_string_list, build_flake_output_path, build_nix_file_output_path, command_output_summary,
    create_unique_temp_dir, escape_nix_string, format_json_value, format_toml_value,
    get_nested_toml_value, json_values_equal, prepare_temp_home, read_toml_file, relative_display,
    require_list_contains, require_list_not_contains, require_path_exists_abs,
    require_path_missing_abs, run_nix_eval, run_repo_command, set_nested_toml_value, sorted_keys,
    split_field_path, toml_to_json, toml_values_equal,
};
pub use installed_runtime::validate_installed_runtime_contract;
pub use nix_interface::{validate_flake_interface, validate_nix_customization_api};
pub use nix_package::{
    ColdProfileInstallOptions, validate_flake_profile_install, validate_nixpkgs_package,
    validate_nixpkgs_submission, validate_runtime_package_smoke,
};
pub use readme_surface::{ReadmeSyncResult, sync_readme_surface, validate_readme_version};
pub use upgrade_contract::{UpgradeContractOptions, validate_upgrade_contract};

const MAIN_TEMPLATE_RELATIVE_PATH: &str = "config_default.toml";
const MODULE_RELATIVE_PATH: &str = "home_manager/module.nix";
const HOME_MANAGER_MODULE_DECLARATION_PATH: &str = "yazelix/home_manager/module.nix";
const MAIN_CONTRACT_RELATIVE_PATH: &str = "config_metadata/main_config_contract.toml";

pub fn validate_nushell_syntax(
    repo_root: &Path,
    verbose: bool,
) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let files = collect_nushell_syntax_files(repo_root)?;
    if files.is_empty() {
        report
            .errors
            .push("No Nushell scripts found to validate".to_string());
        return Ok(report);
    }

    let syntax_home = create_unique_temp_dir("yazelix_nushell_syntax_home")?;
    prepare_nushell_syntax_home(&syntax_home)?;

    for path in files {
        let relative = relative_display(repo_root, &path);
        let output = Command::new("nu")
            .args(["--no-config-file", "--ide-check", "100"])
            .arg(&path)
            .current_dir(repo_root)
            .env("HOME", &syntax_home)
            .env("IN_YAZELIX_SHELL", "1")
            .output()
            .map_err(|error| format!("Failed to run `nu --ide-check`: {}", error))?;
        if !output.status.success() {
            report.errors.push(format!(
                "Nushell syntax check failed to inspect {}\n{}",
                relative,
                command_output_summary(&output)
            ));
            continue;
        }

        let diagnostics = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<JsonValue>(line).ok())
            .filter(|item| {
                item.get("type").and_then(JsonValue::as_str) == Some("diagnostic")
                    && item.get("severity").and_then(JsonValue::as_str) == Some("Error")
            })
            .map(|item| {
                item.get("message")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("unknown Nushell parser diagnostic")
                    .to_string()
            })
            .collect::<Vec<_>>();
        if diagnostics.is_empty() {
            if verbose {
                report
                    .warnings
                    .push(format!("nu --ide-check passed: {relative}"));
            }
        } else {
            report.errors.push(format!(
                "Nushell syntax error in {}\n{}",
                relative,
                diagnostics.join("\n")
            ));
        }
    }

    Ok(report)
}

fn prepare_nushell_syntax_home(home: &Path) -> Result<(), String> {
    let initializer_dir = home.join(".local/share/yazelix/initializers/nushell");
    fs::create_dir_all(&initializer_dir).map_err(|error| {
        format!(
            "Failed to create Nushell syntax fixture dir {}: {}",
            initializer_dir.display(),
            error
        )
    })?;
    for file_name in ["yazelix_init.nu", "yazelix_extern.nu"] {
        let path = initializer_dir.join(file_name);
        fs::write(&path, "").map_err(|error| {
            format!(
                "Failed to write Nushell syntax fixture {}: {}",
                path.display(),
                error
            )
        })?;
    }
    Ok(())
}

fn collect_nushell_syntax_files(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let nushell_dir = repo_root.join("nushell");
    if nushell_dir.exists() {
        collect_nushell_files_in_dir(&nushell_dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect_nushell_files_in_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {}", dir.display(), error))?
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("Failed to inspect {}: {}", path.display(), error))?;
        if file_type.is_dir() {
            collect_nushell_files_in_dir(&path, files)?;
            continue;
        }
        if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("nu") {
            files.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Test lane: default

    use super::*;

    // Regression: CI syntax validation must not depend on generated files in a
    // maintainer's real home directory.
    #[test]
    fn nushell_syntax_validation_stubs_generated_initializers() {
        if Command::new("nu").arg("--version").output().is_err() {
            return;
        }

        let repo = tempfile::tempdir().expect("temp repo");
        let config_dir = repo.path().join("nushell/config");
        fs::create_dir_all(&config_dir).expect("config dir");
        fs::write(
            config_dir.join("config.nu"),
            r#"
if (($env.IN_YAZELIX_SHELL? | is-empty) and ($env.YAZELIX_RUNTIME_DIR? | is-empty)) {
    return
}

source ~/.local/share/yazelix/initializers/nushell/yazelix_init.nu
source ~/.local/share/yazelix/initializers/nushell/yazelix_extern.nu

let parsed_after_generated_sources = true
"#,
        )
        .expect("config fixture");

        let report = validate_nushell_syntax(repo.path(), false).expect("syntax validation");
        assert!(
            report.errors.is_empty(),
            "unexpected syntax errors: {:#?}",
            report.errors
        );
    }

    // Defends: syntax validation follows the current Nushell tree instead of a
    // stale allowlist of deleted helper directories.
    #[test]
    fn nushell_syntax_file_collection_is_recursive() {
        let repo = tempfile::tempdir().expect("temp repo");
        let paths = vec![
            repo.path().join("nushell/config/config.nu"),
            repo.path().join("nushell/extra/nested/custom_check.nu"),
        ];
        for path in &paths {
            fs::create_dir_all(path.parent().expect("fixture parent")).expect("fixture dir");
            fs::write(path, "").expect("fixture file");
        }

        let files = collect_nushell_syntax_files(repo.path()).expect("syntax files");

        assert_eq!(files, paths);
    }
}
