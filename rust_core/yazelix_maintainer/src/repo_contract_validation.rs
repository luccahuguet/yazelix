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
    require_file_not_contains, require_list_contains, require_list_not_contains,
    require_non_empty_dir_abs, require_path_exists, require_path_exists_abs, require_path_missing,
    require_path_missing_abs, run_nix_eval, run_repo_command, set_nested_toml_value, sorted_keys,
    split_field_path, toml_to_json, toml_values_equal, validate_rust_routed_nu_modules,
};
pub use installed_runtime::validate_installed_runtime_contract;
pub use nix_interface::{validate_flake_interface, validate_nix_customization_api};
pub use nix_package::{
    ColdProfileInstallOptions, validate_flake_profile_install, validate_nixpkgs_package,
    validate_nixpkgs_submission,
};
pub use readme_surface::{ReadmeSyncResult, sync_readme_surface, validate_readme_version};
pub use upgrade_contract::{UpgradeContractOptions, validate_upgrade_contract};

const MAIN_TEMPLATE_RELATIVE_PATH: &str = "yazelix_default.toml";
const MODULE_RELATIVE_PATH: &str = "home_manager/module.nix";
const HOME_MANAGER_MODULE_DECLARATION_PATH: &str = "yazelix/home_manager/module.nix";
const MAIN_CONTRACT_RELATIVE_PATH: &str = "config_metadata/main_config_contract.toml";
const TOML_TOOLING_CONFIG_RELATIVE_PATH: &str = "tombi.toml";

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

    for path in files {
        let relative = relative_display(repo_root, &path);
        let output = Command::new("nu")
            .args(["--no-config-file", "--ide-check", "100"])
            .arg(&path)
            .current_dir(repo_root)
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

fn collect_nushell_syntax_files(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for relative_dir in [
        "nushell/scripts/core",
        "nushell/scripts/integrations",
        "nushell/scripts/setup",
        "nushell/scripts/utils",
        "nushell/scripts/dev",
        "nushell/scripts/dev/sweep",
        "nushell/config",
    ] {
        let dir = repo_root.join(relative_dir);
        if !dir.exists() {
            continue;
        }
        collect_nushell_files_in_dir(&dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect_nushell_files_in_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {}", dir.display(), error))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("nu") {
            files.push(path);
        }
    }
    Ok(())
}
