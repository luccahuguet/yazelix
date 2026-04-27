use crate::repo_validation::ValidationReport;
use serde::Deserialize;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::{Table as TomlTable, Value as TomlValue};
use yazelix_core::config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateRequest, compute_config_state,
    record_config_state,
};
use yazelix_core::control_plane::read_yazelix_version_from_runtime;

const MAIN_TEMPLATE_RELATIVE_PATH: &str = "yazelix_default.toml";
const MODULE_RELATIVE_PATH: &str = "home_manager/module.nix";
const HOME_MANAGER_MODULE_DECLARATION_PATH: &str = "yazelix/home_manager/module.nix";
const MAIN_CONTRACT_RELATIVE_PATH: &str = "config_metadata/main_config_contract.toml";
const NUSHELL_BUDGET_RELATIVE_PATH: &str = "config_metadata/nushell_budget.toml";
const TAPLO_RELATIVE_PATH: &str = ".taplo.toml";
const README_LATEST_SERIES_BEGIN: &str = "<!-- BEGIN GENERATED README LATEST SERIES -->";
const README_LATEST_SERIES_END: &str = "<!-- END GENERATED README LATEST SERIES -->";
const GUARDED_FILES: &[&str] = &[
    "nushell/scripts/utils/constants.nu",
    "yazelix_default.toml",
    "home_manager/module.nix",
    "docs/upgrade_notes.toml",
    "CHANGELOG.md",
];
const ACK_REQUIRED_FILES: &[&str] = &["yazelix_default.toml", "home_manager/module.nix"];
const IMPACT_VALUES: &[&str] = &[
    "no_user_action",
    "migration_available",
    "manual_action_required",
];
const ALLOWED_NUSHELL_BUDGET_STATUSES: &[&str] = &["allowlisted_floor", "transitional_exception"];

#[derive(Debug, Clone, Default)]
pub struct UpgradeContractOptions {
    pub ci: bool,
    pub diff_base: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ColdProfileInstallOptions {
    pub phase: String,
    pub temp_home: Option<PathBuf>,
}

impl Default for ColdProfileInstallOptions {
    fn default() -> Self {
        Self {
            phase: "all".to_string(),
            temp_home: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NushellBudgetManifest {
    contract: NushellBudgetContract,
    families: Vec<NushellBudgetFamily>,
}

#[derive(Debug, Deserialize)]
struct NushellBudgetContract {
    current_total_loc_max: usize,
    current_total_file_count_max: usize,
    hard_target_loc: usize,
}

#[derive(Debug, Deserialize)]
struct NushellBudgetFamily {
    id: String,
    status: String,
    owner_bead: String,
    max_loc: usize,
    target_loc: usize,
    max_files: usize,
    allowed_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadmeSyncResult {
    pub readme_path: PathBuf,
    pub title_changed: bool,
    pub series_changed: bool,
}

pub fn validate_config_surface_contract(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    report
        .errors
        .extend(validate_main_contract_parity(repo_root)?);
    report
        .errors
        .extend(validate_home_manager_option_declaration_contract(
            repo_root,
        )?);
    report
        .errors
        .extend(validate_home_manager_desktop_entry_contract(repo_root)?);
    report
        .errors
        .extend(validate_home_manager_activation_hook_static_contract(
            repo_root,
        )?);
    report
        .errors
        .extend(validate_generated_state_contract(repo_root)?);
    Ok(report)
}

pub fn validate_home_manager_option_declaration_contract(
    repo_root: &Path,
) -> Result<Vec<String>, String> {
    let declarations = load_home_manager_option_declarations(repo_root)?;
    let mut errors = Vec::new();
    for (option_name, option_declarations) in declarations {
        for declaration in option_declarations {
            if declaration != HOME_MANAGER_MODULE_DECLARATION_PATH {
                errors.push(format!(
                    "Home Manager option `{}` declaration path must be `{}`, got `{}`",
                    option_name, HOME_MANAGER_MODULE_DECLARATION_PATH, declaration
                ));
            }
        }
    }
    Ok(errors)
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
        read_yazelix_version_from_runtime(repo_root).map_err(|error| error.message())?;

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

pub fn validate_readme_version(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let version = read_yazelix_version_from_runtime(repo_root).map_err(|error| error.message())?;
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

    let release_heading = actual_block
        .lines()
        .nth(1)
        .unwrap_or_default()
        .trim()
        .to_string();
    let expected_release_heading = "## Latest Tagged Releases".to_string();
    if release_heading != expected_release_heading {
        report.errors.push(format!(
            "README latest tagged release drift detected. Expected '{}' but found '{}'.",
            expected_release_heading, release_heading
        ));
    }

    let expected_first_release_heading = format!("### {}", major_series_key(&version)?);
    let first_release_heading = actual_block
        .lines()
        .skip(3)
        .find(|line| line.trim_start().starts_with("### "))
        .unwrap_or_default()
        .trim()
        .to_string();
    if first_release_heading != expected_first_release_heading {
        report.errors.push(format!(
            "README first latest major release drift detected. Expected '{}' but found '{}'.",
            expected_first_release_heading, first_release_heading
        ));
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
        None => read_yazelix_version_from_runtime(repo_root).map_err(|error| error.message())?,
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

pub fn validate_nushell_budget(repo_root: &Path) -> Result<ValidationReport, String> {
    let manifest = load_nushell_budget_manifest(repo_root)?;
    let actual_paths = load_nushell_script_paths(repo_root)?;
    let mut report = ValidationReport::default();
    let mut family_index_by_id = HashMap::new();
    let mut family_index_by_path = HashMap::new();
    let mut family_loc_totals = vec![0usize; manifest.families.len()];
    let mut family_file_totals = vec![0usize; manifest.families.len()];

    for (index, family) in manifest.families.iter().enumerate() {
        if !ALLOWED_NUSHELL_BUDGET_STATUSES.contains(&family.status.as_str()) {
            report.errors.push(format!(
                "Nushell budget family `{}` declares unsupported status `{}` in {}",
                family.id, family.status, NUSHELL_BUDGET_RELATIVE_PATH
            ));
        }

        if family.owner_bead.trim().is_empty() {
            report.errors.push(format!(
                "Nushell budget family `{}` is missing an owner_bead in {}",
                family.id, NUSHELL_BUDGET_RELATIVE_PATH
            ));
        }

        if family.target_loc > family.max_loc {
            report.errors.push(format!(
                "Nushell budget family `{}` has target_loc {} above max_loc {} in {}",
                family.id, family.target_loc, family.max_loc, NUSHELL_BUDGET_RELATIVE_PATH
            ));
        }

        if family.allowed_paths.is_empty() {
            report.errors.push(format!(
                "Nushell budget family `{}` has no allowed_paths in {}",
                family.id, NUSHELL_BUDGET_RELATIVE_PATH
            ));
        }

        if let Some(existing_index) = family_index_by_id.insert(family.id.clone(), index) {
            let existing_id = &manifest.families[existing_index].id;
            report.errors.push(format!(
                "Nushell budget family id `{}` is duplicated in {}",
                existing_id, NUSHELL_BUDGET_RELATIVE_PATH
            ));
        }

        for relative_path in &family.allowed_paths {
            if !relative_path.starts_with("nushell/scripts/") || !relative_path.ends_with(".nu") {
                report.errors.push(format!(
                    "Nushell budget family `{}` lists a non-Nushell path `{}` in {}",
                    family.id, relative_path, NUSHELL_BUDGET_RELATIVE_PATH
                ));
                continue;
            }

            if let Some(previous_family_index) =
                family_index_by_path.insert(relative_path.clone(), index)
            {
                let previous_family = &manifest.families[previous_family_index].id;
                report.errors.push(format!(
                    "Nushell budget path `{}` is assigned to both `{}` and `{}` in {}",
                    relative_path, previous_family, family.id, NUSHELL_BUDGET_RELATIVE_PATH
                ));
            }

            if !repo_root.join(relative_path).is_file() {
                report.errors.push(format!(
                    "Nushell budget path `{}` is listed under `{}` but does not exist in the repo",
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
                "Unexpected Nushell file outside the canonical budget allowlist: {}",
                relative_path
            ));
            continue;
        };

        let line_count = count_tracked_lines(&repo_root.join(&relative_path))?;
        family_loc_totals[family_index] += line_count;
        family_file_totals[family_index] += 1;
        total_loc += line_count;
    }

    let expected_total_loc: usize = manifest.families.iter().map(|family| family.max_loc).sum();
    let expected_total_files: usize = manifest
        .families
        .iter()
        .map(|family| family.max_files)
        .sum();

    if expected_total_loc != manifest.contract.current_total_loc_max {
        report.errors.push(format!(
            "Nushell budget manifest total LOC mismatch in {}: contract={}, family_sum={}",
            NUSHELL_BUDGET_RELATIVE_PATH,
            manifest.contract.current_total_loc_max,
            expected_total_loc
        ));
    }

    if expected_total_files != manifest.contract.current_total_file_count_max {
        report.errors.push(format!(
            "Nushell budget manifest total file-count mismatch in {}: contract={}, family_sum={}",
            NUSHELL_BUDGET_RELATIVE_PATH,
            manifest.contract.current_total_file_count_max,
            expected_total_files
        ));
    }

    if total_loc > manifest.contract.current_total_loc_max {
        report.errors.push(format!(
            "Nushell budget grew above the tracked ceiling: measured {} LOC > allowed {} LOC",
            total_loc, manifest.contract.current_total_loc_max
        ));
    }

    if total_files > manifest.contract.current_total_file_count_max {
        report.errors.push(format!(
            "Nushell file count grew above the tracked ceiling: measured {} files > allowed {} files",
            total_files, manifest.contract.current_total_file_count_max
        ));
    }

    for (index, family) in manifest.families.iter().enumerate() {
        let measured_loc = family_loc_totals[index];
        let measured_files = family_file_totals[index];

        if measured_loc > family.max_loc {
            report.errors.push(format!(
                "Nushell budget family `{}` grew above its LOC ceiling: measured {} LOC > allowed {} LOC",
                family.id, measured_loc, family.max_loc
            ));
        }

        if measured_files > family.max_files {
            report.errors.push(format!(
                "Nushell budget family `{}` grew above its file-count ceiling: measured {} files > allowed {} files",
                family.id, measured_files, family.max_files
            ));
        }
    }

    if total_loc > manifest.contract.hard_target_loc {
        report.warnings.push(format!(
            "Current tracked Nushell surface is {} LOC, still above the long-term hard target of {} LOC. Keep deleting until the tracked ceilings fall again.",
            total_loc, manifest.contract.hard_target_loc
        ));
    }

    Ok(report)
}

pub fn validate_installed_runtime_contract(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    report
        .errors
        .extend(validate_installed_runtime_contract_inner(repo_root)?);
    Ok(report)
}

pub fn validate_flake_interface(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let ok = run_nix_eval(repo_root, &build_flake_interface_expr(repo_root))?
        .as_bool()
        .ok_or("Top-level flake interface validation did not return a boolean")?;
    if !ok {
        report.errors.push(
            "Top-level flake interface is missing required package/app/Home Manager outputs, still exposes legacy install outputs, or still points packages.default at the lower-level runtime."
                .to_string(),
        );
    }

    let platform_rows = run_nix_eval(repo_root, &build_flake_package_platform_expr(repo_root))?;
    let rows = platform_rows
        .as_array()
        .ok_or("First-party flake package platform validation did not return a JSON array")?;
    let unavailable = rows
        .iter()
        .filter(|row| {
            !row.get("available")
                .and_then(JsonValue::as_bool)
                .unwrap_or(false)
        })
        .map(|row| {
            let system = row
                .get("system")
                .and_then(JsonValue::as_str)
                .unwrap_or("<unknown>");
            let platforms = row.get("platforms").unwrap_or(&JsonValue::Null);
            format!("{system} (meta.platforms={})", format_json_value(platforms))
        })
        .collect::<Vec<_>>();
    if !unavailable.is_empty() {
        report.errors.push(format!(
            "First-party flake package reports as unavailable on exported systems: {}. Each system exported in flake.nix must be included in the package meta.platforms.",
            unavailable.join(", ")
        ));
    }

    Ok(report)
}

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

pub fn validate_nixpkgs_package(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let package_root = build_flake_output_path(
        repo_root,
        "yazelix",
        "building .#yazelix during nixpkgs package validation",
    )?;
    verify_yazelix_package(&package_root, &mut report.errors)?;
    Ok(report)
}

pub fn validate_nixpkgs_submission(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let package_root = build_nix_file_output_path(
        repo_root,
        Path::new("packaging").join("nixpkgs").join("default.nix"),
        "building packaging/nixpkgs/default.nix during nixpkgs submission validation",
    )?;
    verify_yazelix_package(&package_root, &mut report.errors)?;
    Ok(report)
}

pub fn validate_flake_profile_install(
    repo_root: &Path,
    options: &ColdProfileInstallOptions,
) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    match options.phase.as_str() {
        "all" => {
            let temp_home = match &options.temp_home {
                Some(path) => {
                    prepare_temp_home(path)?;
                    path.clone()
                }
                None => create_unique_temp_dir("yazelix_profile_install")?,
            };
            run_profile_install(repo_root, &temp_home, &mut report.errors)?;
            if report.errors.is_empty() {
                verify_profile_installed_runtime(repo_root, &temp_home, &mut report.errors)?;
            }
            let _ = fs::remove_dir_all(&temp_home);
        }
        "install" => {
            let Some(temp_home) = &options.temp_home else {
                return Err("The `install` phase requires an explicit temp_home path".to_string());
            };
            prepare_temp_home(temp_home)?;
            run_profile_install(repo_root, temp_home, &mut report.errors)?;
        }
        "verify" => {
            let Some(temp_home) = &options.temp_home else {
                return Err("The `verify` phase requires an explicit temp_home path".to_string());
            };
            require_path_exists_abs(
                temp_home,
                "cold profile-install temp home",
                &mut report.errors,
            );
            if report.errors.is_empty() {
                verify_profile_installed_runtime(repo_root, temp_home, &mut report.errors)?;
            }
        }
        other => {
            return Err(format!(
                "Unsupported cold profile-install phase `{}`. Expected one of: all, install, verify",
                other
            ));
        }
    }
    Ok(report)
}

fn validate_installed_runtime_contract_inner(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let cli_wrapper = "shells/posix/yzx_cli.sh";
    let detached_launch_probe = "shells/posix/detached_launch_probe.sh";
    let runtime_env = "shells/posix/runtime_env.sh";
    let environment_setup = "nushell/scripts/setup/environment.nu";
    let runtime_tree = "packaging/mk_runtime_tree.nix";
    let flake_path = "flake.nix";

    require_path_exists(repo_root, flake_path, "flake definition", &mut errors);
    require_path_missing(
        repo_root,
        "shells/posix/install_yazelix.sh.in",
        "legacy flake installer template",
        &mut errors,
    );
    require_path_exists(
        repo_root,
        cli_wrapper,
        "stable POSIX CLI wrapper",
        &mut errors,
    );
    require_path_exists(
        repo_root,
        detached_launch_probe,
        "detached launch probe helper",
        &mut errors,
    );
    require_path_exists(repo_root, runtime_env, "runtime env helper", &mut errors);
    require_path_exists(
        repo_root,
        environment_setup,
        "environment setup script",
        &mut errors,
    );
    require_path_exists(repo_root, runtime_tree, "runtime tree builder", &mut errors);

    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"yzx_root_bin="${YAZELIX_YZX_BIN:-$RUNTIME_DIR/libexec/yzx}""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"exec "$yzx_root_bin" "$@""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        runtime_env,
        "export YAZELIX_DIR=",
        "runtime env helper",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        environment_setup,
        "get_installed_yazelix_runtime_reference_dir",
        "environment setup script",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        environment_setup,
        "ensure_user_cli_wrapper",
        "environment setup script",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        "import ./runtime_deps.nix",
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        r#"ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml""#,
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        "for bin_dir in ${escapedRuntimeBinDirs}; do",
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        r#"cat > "$out/bin/yzx" <<EOF"#,
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        runtime_tree,
        "yazelix_packs_default.toml",
        "runtime tree builder",
        &mut errors,
    )?;

    if !errors.is_empty() {
        return Ok(errors);
    }

    let flake_show = run_repo_command(repo_root, "nix", &["flake", "show", "--json"])?;
    if !flake_show.status.success() {
        errors.push(format!(
            "Failed to evaluate flake outputs during installed-runtime contract validation\n{}",
            command_stderr(&flake_show)
        ));
        return Ok(errors);
    }
    let flake: JsonValue = serde_json::from_slice(&flake_show.stdout)
        .map_err(|error| format!("Failed to parse `nix flake show --json`: {}", error))?;
    let package_keys = json_object_keys(
        flake
            .pointer("/packages/x86_64-linux")
            .ok_or("Missing packages.x86_64-linux in flake output")?,
    );
    for expected in ["default", "runtime", "yazelix"] {
        require_list_contains(
            &package_keys,
            expected,
            "x86_64-linux package outputs",
            &mut errors,
        );
    }
    for forbidden in ["install", "locked_devenv"] {
        require_list_not_contains(
            &package_keys,
            forbidden,
            "x86_64-linux package outputs",
            &mut errors,
        );
    }
    let app_keys = json_object_keys(
        flake
            .pointer("/apps/x86_64-linux")
            .ok_or("Missing apps.x86_64-linux in flake output")?,
    );
    for expected in ["default", "yazelix"] {
        require_list_contains(&app_keys, expected, "x86_64-linux app outputs", &mut errors);
    }
    require_list_not_contains(
        &app_keys,
        "install",
        "x86_64-linux app outputs",
        &mut errors,
    );

    if !errors.is_empty() {
        return Ok(errors);
    }

    let runtime_out = build_flake_output_path(
        repo_root,
        "runtime",
        "building runtime package for installed-runtime validation",
    )?;
    validate_rust_routed_nu_modules(&runtime_out, "built runtime package", &mut errors);
    require_path_exists_abs(
        &runtime_out.join(detached_launch_probe),
        "built runtime detached launch probe helper",
        &mut errors,
    );

    let yazelix_out = build_flake_output_path(
        repo_root,
        "yazelix",
        "building yazelix package for installed-runtime validation",
    )?;
    validate_rust_routed_nu_modules(&yazelix_out, "built yazelix package", &mut errors);
    require_path_exists_abs(
        &yazelix_out.join(detached_launch_probe),
        "built yazelix detached launch probe helper",
        &mut errors,
    );
    errors.extend(validate_home_manager_activation_contract(repo_root)?);

    let built_yzx = yazelix_out.join("bin").join("yzx");
    require_path_exists_abs(&built_yzx, "built yazelix CLI wrapper", &mut errors);
    if !errors.is_empty() {
        return Ok(errors);
    }

    let built_yzx_string = built_yzx.display().to_string();
    let smoke_result = Command::new("env")
        .args([
            "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1",
            built_yzx_string.as_str(),
            "why",
        ])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to smoke-run built yazelix public CLI: {}", error))?;
    if !smoke_result.status.success() {
        errors.push(format!(
            "Built yazelix package failed the public CLI smoke check\n{}",
            command_output_summary(&smoke_result)
        ));
        return Ok(errors);
    }
    require_file_contains_abs(
        &built_yzx,
        "shells/posix/yzx_cli.sh",
        "built yazelix CLI wrapper",
        &mut errors,
    )?;
    let smoke_stdout = String::from_utf8_lossy(&smoke_result.stdout);
    if !smoke_stdout.contains("Yazelix is a reproducible terminal IDE") {
        errors.push(
            "Built yazelix public CLI smoke check returned unexpected output for `yzx why`"
                .to_string(),
        );
    }

    Ok(errors)
}

fn validate_home_manager_activation_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    errors.extend(validate_home_manager_activation_mode(repo_root, false)?);
    errors.extend(validate_home_manager_activation_mode(repo_root, true)?);
    Ok(errors)
}

fn validate_home_manager_activation_mode(
    repo_root: &Path,
    manage_config: bool,
) -> Result<Vec<String>, String> {
    let temp_root = create_unique_temp_dir("yazelix_home_manager_activation")?;
    let cleanup_result = (|| {
        let flake_root = temp_root.join("flake");
        let home_root = temp_root.join("home");
        let system = resolve_nix_current_system(repo_root)?;
        let xdg_config_home = home_root.join(".config");
        let xdg_data_home = home_root.join(".local").join("share");
        fs::create_dir_all(&flake_root)
            .map_err(|error| format!("Failed to create {}: {}", flake_root.display(), error))?;
        fs::create_dir_all(&xdg_config_home).map_err(|error| {
            format!("Failed to create {}: {}", xdg_config_home.display(), error)
        })?;
        fs::create_dir_all(&xdg_data_home)
            .map_err(|error| format!("Failed to create {}: {}", xdg_data_home.display(), error))?;
        fs::write(
            flake_root.join("flake.nix"),
            build_home_manager_activation_validation_flake(
                repo_root,
                &home_root,
                &system,
                manage_config,
            ),
        )
        .map_err(|error| {
            format!(
                "Failed to write Home Manager activation validation flake: {}",
                error
            )
        })?;

        let build_output = Command::new("nix")
            .args([
                "build",
                "--no-link",
                "--print-out-paths",
                ".#homeConfigurations.validation.activationPackage",
            ])
            .current_dir(&flake_root)
            .output()
            .map_err(|error| {
                format!(
                    "Failed to build the temporary Home Manager activation package: {}",
                    error
                )
            })?;
        if !build_output.status.success() {
            return Ok(vec![format!(
                "Temporary Home Manager activation package failed to build\n{}",
                command_output_summary(&build_output)
            )]);
        }

        let stdout = String::from_utf8_lossy(&build_output.stdout);
        let Some(activation_package) = stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(PathBuf::from)
        else {
            return Ok(vec![
                "Temporary Home Manager activation build did not return an activation package path"
                    .to_string(),
            ]);
        };

        let activate_script = activation_package.join("activate");
        let mut errors = Vec::new();
        require_path_exists_abs(
            &activate_script,
            "temporary Home Manager activation script",
            &mut errors,
        );
        if !errors.is_empty() {
            return Ok(errors);
        }

        let activate_output = Command::new(&activate_script)
            .env("HOME", &home_root)
            .env("USER", "validator")
            .env("XDG_CONFIG_HOME", &xdg_config_home)
            .env("XDG_DATA_HOME", &xdg_data_home)
            .env(
                "PATH",
                env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
            )
            .current_dir(&flake_root)
            .output()
            .map_err(|error| {
                format!(
                    "Failed to run the temporary Home Manager activation script: {}",
                    error
                )
            })?;
        if !activate_output.status.success() {
            return Ok(vec![format!(
                "Temporary Home Manager activation script failed\n{}",
                command_output_summary(&activate_output)
            )]);
        }

        require_path_exists_abs(
            &home_root
                .join(".config")
                .join("yazelix")
                .join("user_configs")
                .join("yazelix.toml"),
            if manage_config {
                "Home Manager managed yazelix.toml surface after activation"
            } else {
                "Yazelix bootstrapped mutable yazelix.toml surface after Home Manager activation"
            },
            &mut errors,
        );
        let main_config_path = home_root
            .join(".config")
            .join("yazelix")
            .join("user_configs")
            .join("yazelix.toml");
        if let Ok(metadata) = fs::symlink_metadata(&main_config_path) {
            if manage_config && !metadata.file_type().is_symlink() {
                errors.push(format!(
                    "Home Manager managed yazelix.toml should be a profile symlink: {}",
                    main_config_path.display()
                ));
            }
            if !manage_config && metadata.file_type().is_symlink() {
                errors.push(format!(
                    "Home Manager manage_config=false should leave yazelix.toml mutable, got symlink: {}",
                    main_config_path.display()
                ));
            }
        }
        require_path_exists_abs(
            &home_root
                .join(".local")
                .join("share")
                .join("yazelix")
                .join("configs")
                .join("zellij")
                .join("config.kdl"),
            "generated Zellij config after Home Manager activation",
            &mut errors,
        );
        require_path_exists_abs(
            &home_root
                .join(".local")
                .join("share")
                .join("yazelix")
                .join("configs")
                .join("yazi")
                .join("yazi.toml"),
            "generated Yazi config after Home Manager activation",
            &mut errors,
        );
        // Home Manager activation repairs generated runtime state, while terminal
        // configs remain launch-time materialization outputs.
        Ok(errors)
    })();
    let _ = fs::remove_dir_all(&temp_root);
    cleanup_result
}

fn resolve_nix_current_system(repo_root: &Path) -> Result<String, String> {
    let output = run_repo_command(
        repo_root,
        "nix",
        &[
            "eval",
            "--raw",
            "--impure",
            "--expr",
            "builtins.currentSystem",
        ],
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed to resolve the current Nix system for Home Manager activation validation\n{}",
            command_output_summary(&output)
        ));
    }
    let system = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if system.is_empty() {
        return Err(
            "Home Manager activation validation could not resolve the current Nix system"
                .to_string(),
        );
    }
    Ok(system)
}

fn build_home_manager_activation_validation_flake(
    repo_root: &Path,
    home_root: &Path,
    system: &str,
    manage_config: bool,
) -> String {
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
    let home_root_literal = escape_nix_string(&home_root.display().to_string());
    let system_literal = escape_nix_string(system);
    [
        "{".to_string(),
        "  description = \"Yazelix Home Manager activation validation\";".to_string(),
        String::new(),
        "  inputs = {".to_string(),
        format!("    yazelix.url = \"path:{}\";", repo_root_literal),
        "    nixpkgs.follows = \"yazelix/nixpkgs\";".to_string(),
        "    home-manager.follows = \"yazelix/home-manager\";".to_string(),
        "  };".to_string(),
        String::new(),
        "  outputs = { nixpkgs, home-manager, yazelix, ... }:".to_string(),
        "    let".to_string(),
        format!("      system = \"{}\";", system_literal),
        "      pkgs = import nixpkgs { inherit system; };".to_string(),
        "    in {".to_string(),
        "      homeConfigurations.validation = home-manager.lib.homeManagerConfiguration {"
            .to_string(),
        "        inherit pkgs;".to_string(),
        "        modules = [".to_string(),
        "          yazelix.homeManagerModules.default".to_string(),
        "          ({ ... }: {".to_string(),
        "            home.username = \"validator\";".to_string(),
        format!(
            "            home.homeDirectory = \"{}\";",
            home_root_literal
        ),
        "            home.stateVersion = \"24.11\";".to_string(),
        "            programs.home-manager.enable = true;".to_string(),
        "            programs.yazelix.enable = true;".to_string(),
        if manage_config {
            "            programs.yazelix.manage_config = true;".to_string()
        } else {
            "            # manage_config=false default is intentionally exercised here.".to_string()
        },
        "          })".to_string(),
        "        ];".to_string(),
        "      };".to_string(),
        "    };".to_string(),
        "}".to_string(),
    ]
    .join("\n")
}

fn build_flake_interface_expr(repo_root: &Path) -> String {
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{}\";", repo_root_literal),
        "  system = builtins.currentSystem;".to_string(),
        "in".to_string(),
        "  builtins.hasAttr \"packages\" flake &&".to_string(),
        "  builtins.hasAttr system flake.packages &&".to_string(),
        "  builtins.hasAttr \"default\" flake.packages.${system} &&".to_string(),
        "  builtins.hasAttr \"runtime\" flake.packages.${system} &&".to_string(),
        "  builtins.hasAttr \"yazelix\" flake.packages.${system} &&".to_string(),
        "  !builtins.hasAttr \"install\" flake.packages.${system} &&".to_string(),
        "  flake.packages.${system}.default.outPath == flake.packages.${system}.yazelix.outPath &&"
            .to_string(),
        "  builtins.hasAttr \"apps\" flake &&".to_string(),
        "  builtins.hasAttr system flake.apps &&".to_string(),
        "  builtins.hasAttr \"default\" flake.apps.${system} &&".to_string(),
        "  builtins.hasAttr \"yazelix\" flake.apps.${system} &&".to_string(),
        "  !builtins.hasAttr \"install\" flake.apps.${system} &&".to_string(),
        "  builtins.hasAttr \"homeManagerModules\" flake &&".to_string(),
        "  builtins.hasAttr \"default\" flake.homeManagerModules &&".to_string(),
        "  builtins.hasAttr \"yazelix\" flake.homeManagerModules &&".to_string(),
        "  builtins.isFunction flake.homeManagerModules.default &&".to_string(),
        "  builtins.isFunction flake.homeManagerModules.yazelix".to_string(),
    ]
    .join("\n")
}

fn build_flake_package_platform_expr(repo_root: &Path) -> String {
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{}\";", repo_root_literal),
        "  lib = flake.inputs.nixpkgs.lib;".to_string(),
        "  systems = builtins.attrNames flake.packages;".to_string(),
        "  check = system:".to_string(),
        "    let".to_string(),
        "      pkg = flake.packages.${system}.yazelix;".to_string(),
        "      platformEntry = lib.systems.elaborate { inherit system; };".to_string(),
        "    in {".to_string(),
        "      inherit system;".to_string(),
        "      available = lib.meta.availableOn platformEntry pkg;".to_string(),
        "      platforms = pkg.meta.platforms or [];".to_string(),
        "    };".to_string(),
        "in".to_string(),
        "  builtins.map check systems".to_string(),
    ]
    .join("\n")
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

fn verify_yazelix_package(package_root: &Path, errors: &mut Vec<String>) -> Result<(), String> {
    let temp_home = create_unique_temp_dir("yazelix_nixpkgs_package")?;
    let validation = (|| -> Result<(), String> {
        require_path_exists_abs(
            &package_root.join("bin").join("yzx"),
            "packaged yzx wrapper",
            errors,
        );
        require_path_missing_abs(
            &package_root.join("yazelix_packs_default.toml"),
            "legacy packaged packs config",
            errors,
        );
        if !errors.is_empty() {
            return Ok(());
        }

        let version_result = run_packaged_yzx(package_root, &temp_home, &["--version-short"])?;
        if !version_result.status.success() {
            errors.push(format!(
                "Packaged yzx --version-short failed\n{}",
                command_output_summary(&version_result)
            ));
            return Ok(());
        }
        let version_text = String::from_utf8_lossy(&version_result.stdout)
            .trim()
            .to_string();
        if !version_text.starts_with("Yazelix (v") {
            errors.push(format!(
                "Unexpected packaged yzx version output: {}",
                version_text
            ));
        }

        let doctor_result = run_packaged_yzx(package_root, &temp_home, &["doctor", "--verbose"])?;
        if !doctor_result.status.success() {
            errors.push(format!(
                "Packaged yzx doctor --verbose failed\n{}",
                command_output_summary(&doctor_result)
            ));
            return Ok(());
        }

        let runtime_probe = run_packaged_yzx(
            package_root,
            &temp_home,
            &["run", "nu", "-c", RUNTIME_ENV_PROBE_NU],
        )?;
        if !runtime_probe.status.success() {
            errors.push(format!(
                "Packaged yzx run probe failed\n{}",
                command_output_summary(&runtime_probe)
            ));
            return Ok(());
        }
        let probe: JsonValue = serde_json::from_slice(&runtime_probe.stdout)
            .map_err(|error| format!("Failed to parse packaged runtime probe JSON: {}", error))?;
        let expected_bin = package_root.join("bin").display().to_string();
        let expected_toolbin = package_root.join("toolbin").display().to_string();
        validate_runtime_env_probe(
            &probe,
            package_root,
            &expected_toolbin,
            &expected_bin,
            None,
            "Packaged Yazelix runtime probe",
            errors,
        );

        require_path_exists_abs(
            &package_root.join("toolbin").join("rg"),
            "exported runtime tool `rg`",
            errors,
        );
        require_path_missing_abs(
            &package_root.join("toolbin").join("dirname"),
            "runtime-private helper leaked into exported toolbin",
            errors,
        );
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_home);
    validation
}

fn run_profile_install(
    repo_root: &Path,
    temp_home: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let state_root = temp_home.join(".local").join("share");
    let config_root = temp_home.join(".config");
    let profile_root = temp_home.join(".nix-profile");
    let output = Command::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "profile",
            "add",
            "--profile",
        ])
        .arg(&profile_root)
        .arg(".#yazelix")
        .current_dir(repo_root)
        .env("HOME", temp_home)
        .env("XDG_CONFIG_HOME", &config_root)
        .env("XDG_DATA_HOME", &state_root)
        .env_remove("YAZELIX_CONFIG_DIR")
        .env_remove("YAZELIX_CONFIG_OVERRIDE")
        .env_remove("YAZELIX_LOGS_DIR")
        .env_remove("YAZELIX_RUNTIME_DIR")
        .env_remove("YAZELIX_STATE_DIR")
        .output()
        .map_err(|error| format!("Failed to run cold profile install: {}", error))?;
    if !output.status.success() {
        errors.push(format!(
            "Cold profile-install validation failed while running `nix profile add --profile ... .#yazelix`\n{}",
            command_output_summary(&output)
        ));
    }
    Ok(())
}

fn verify_profile_installed_runtime(
    repo_root: &Path,
    temp_home: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let profile_root = temp_home.join(".nix-profile");
    let yzx_path = profile_root.join("bin").join("yzx");
    let local_wrapper = temp_home.join(".local").join("bin").join("yzx");
    let legacy_runtime_link = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("runtime")
        .join("current");
    let desktop_entry = temp_home
        .join(".local")
        .join("share")
        .join("applications")
        .join("com.yazelix.Yazelix.desktop");
    let user_config = temp_home
        .join(".config")
        .join("yazelix")
        .join("user_configs")
        .join("yazelix.toml");
    let pack_config = temp_home
        .join(".config")
        .join("yazelix")
        .join("user_configs")
        .join("yazelix_packs.toml");
    let nushell_config = temp_home.join(".config").join("nushell").join("config.nu");

    require_path_exists_abs(&yzx_path, "profile-installed yzx wrapper", errors);
    require_path_missing_abs(&local_wrapper, "legacy user-local yzx wrapper", errors);
    require_path_missing_abs(
        &legacy_runtime_link,
        "legacy installed runtime symlink",
        errors,
    );
    require_path_missing_abs(
        &desktop_entry,
        "default user-local desktop entry before explicit desktop install",
        errors,
    );
    require_path_missing_abs(
        &user_config,
        "managed user config before first runtime entry",
        errors,
    );
    require_path_missing_abs(&pack_config, "legacy managed pack config", errors);
    require_path_missing_abs(
        &nushell_config,
        "host Nushell hook config before first runtime entry",
        errors,
    );
    if !errors.is_empty() {
        return Ok(());
    }

    let wrapper_target = fs::canonicalize(&yzx_path).map_err(|error| {
        format!(
            "Failed to resolve installed yzx wrapper target {}: {}",
            yzx_path.display(),
            error
        )
    })?;
    let runtime_root = wrapper_target
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| {
            format!(
                "Installed yzx wrapper target has no runtime root: {}",
                wrapper_target.display()
            )
        })?
        .to_path_buf();
    let runtime_bin = runtime_root.join("bin");
    let runtime_toolbin = runtime_root.join("toolbin");
    let runtime_libexec = runtime_root.join("libexec");
    let runtime_variant = fs::read_to_string(runtime_root.join("runtime_variant"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let runtime_terminal = match runtime_variant.as_str() {
        "wezterm" => "wezterm",
        _ => "ghostty",
    };
    let runtime_yzx_cli = runtime_root.join("shells").join("posix").join("yzx_cli.sh");
    let runtime_yzx_core = runtime_libexec.join("yzx_core");
    let runtime_ghostty_wrapper = runtime_root
        .join("shells")
        .join("posix")
        .join("yazelix_ghostty.sh");
    let runtime_yazelix_default = runtime_root.join("yazelix_default.toml");
    let runtime_ghostty_shader_root = runtime_root
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    let generated_ghostty_root = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty");
    let generated_ghostty_config = generated_ghostty_root.join("config");
    let generated_ghostty_effect_dir = generated_ghostty_root
        .join("shaders")
        .join("generated_effects");
    let generated_wezterm_config = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("wezterm")
        .join(".wezterm.lua");

    for (path, label) in [
        (runtime_toolbin.clone(), "runtime toolbin"),
        (runtime_libexec.join("nu"), "runtime-local Nushell binary"),
        (
            runtime_libexec.join("yzx"),
            "runtime-local Rust yzx root helper",
        ),
        (runtime_yzx_core.clone(), "runtime-local yzx_core helper"),
        (
            runtime_libexec.join("yzx_control"),
            "runtime-local yzx_control helper",
        ),
        (runtime_yzx_cli.clone(), "runtime-local POSIX yzx launcher"),
        (
            runtime_ghostty_wrapper.clone(),
            "runtime-local Ghostty env wrapper",
        ),
        (
            runtime_yazelix_default.clone(),
            "runtime-local default config",
        ),
        (
            runtime_ghostty_shader_root.join("build_shaders.nu"),
            "runtime-local Ghostty shader builder",
        ),
        (
            runtime_ghostty_shader_root
                .join("variants")
                .join("reef.glsl"),
            "runtime-local Ghostty trail shader variant",
        ),
        (
            runtime_ghostty_shader_root
                .join("upstream_effects")
                .join("ripple_rectangle_cursor.glsl"),
            "runtime-local Ghostty cursor effect template",
        ),
    ] {
        require_path_exists_abs(&path, label, errors);
    }

    for expected_tool in [
        "zellij",
        runtime_terminal,
        "yazi",
        "hx",
        "nvim",
        "fish",
        "zsh",
        "bash",
        "nix",
        "jq",
        "fd",
        "rg",
    ] {
        require_path_exists_abs(
            &runtime_libexec.join(expected_tool),
            &format!("runtime tool `{expected_tool}`"),
            errors,
        );
    }
    for expected_exported_tool in [
        "nu", "zellij", "yazi", "hx", "nvim", "bash", "jq", "fd", "rg",
    ] {
        require_path_exists_abs(
            &runtime_toolbin.join(expected_exported_tool),
            &format!("exported runtime tool `{expected_exported_tool}`"),
            errors,
        );
    }
    require_path_missing_abs(
        &runtime_toolbin.join("dirname"),
        "runtime-private helper leaked into exported toolbin",
        errors,
    );
    if cfg!(target_os = "linux") {
        require_path_exists_abs(
            &runtime_libexec.join("nixGLMesa"),
            "runtime tool `nixGLMesa`",
            errors,
        );
        require_path_exists_abs(
            &runtime_libexec.join("pgrep"),
            "runtime tool `pgrep`",
            errors,
        );
    }

    let expected_wrapper_target = runtime_root.join("bin").join("yzx");
    if wrapper_target != expected_wrapper_target {
        errors.push(format!(
            "Installed yzx wrapper should point at the packaged runtime. Expected {}, got {}",
            expected_wrapper_target.display(),
            wrapper_target.display()
        ));
    }
    if !errors.is_empty() {
        return Ok(());
    }

    let version_result = run_installed_yzx(repo_root, temp_home, &["--version-short"])?;
    if !version_result.status.success() {
        errors.push(format!(
            "Installed yzx --version-short failed during cold profile-install validation\n{}",
            command_output_summary(&version_result)
        ));
        return Ok(());
    }
    let version_text = String::from_utf8_lossy(&version_result.stdout)
        .trim()
        .to_string();
    if !version_text.starts_with("Yazelix (v") {
        errors.push(format!(
            "Unexpected installed yzx version output: {}",
            version_text
        ));
    }

    let posix_launcher_result =
        run_runtime_posix_launcher_minimal_env(repo_root, temp_home, &runtime_yzx_cli)?;
    if !posix_launcher_result.status.success() {
        errors.push(format!(
            "Runtime-local POSIX yzx launcher failed under minimal PATH during cold profile-install validation\n{}",
            command_output_summary(&posix_launcher_result)
        ));
        return Ok(());
    }
    let posix_version_text = String::from_utf8_lossy(&posix_launcher_result.stdout)
        .trim()
        .to_string();
    if !posix_version_text.starts_with("Yazelix (v") {
        errors.push(format!(
            "Unexpected runtime-local POSIX yzx output: {}",
            posix_version_text
        ));
    }

    let runtime_probe = run_installed_yzx(
        repo_root,
        temp_home,
        &["run", "nu", "-c", INSTALLED_ENV_PROBE_NU],
    )?;
    if !runtime_probe.status.success() {
        errors.push(format!(
            "Installed yzx run probe failed during cold profile-install validation\n{}",
            command_output_summary(&runtime_probe)
        ));
        return Ok(());
    }
    let probe: JsonValue = serde_json::from_slice(&runtime_probe.stdout).map_err(|error| {
        format!(
            "Failed to parse installed runtime probe JSON during cold profile-install validation: {}",
            error
        )
    })?;
    validate_runtime_env_probe(
        &probe,
        &runtime_root,
        &runtime_toolbin.display().to_string(),
        &runtime_bin.display().to_string(),
        Some(&runtime_bin.join("yzx").display().to_string()),
        "Installed runtime probe",
        errors,
    );
    if !probe
        .get("editor")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .contains("yazelix_hx.sh")
    {
        errors.push(format!(
            "Installed runtime probe did not set EDITOR to the managed Helix wrapper: {}",
            format_json_value(&probe)
        ));
    }
    if !errors.is_empty() {
        return Ok(());
    }

    let yzx_core_arg = runtime_yzx_core.display().to_string();
    let selected_terminal_json = format!("[\"{runtime_terminal}\"]");
    let materialization_result = run_installed_yzx(
        repo_root,
        temp_home,
        &[
            "run",
            &yzx_core_arg,
            "launch-materialization.prepare",
            "--from-env",
            "--selected-terminals-json",
            &selected_terminal_json,
        ],
    )?;
    if !materialization_result.status.success() {
        errors.push(format!(
            "Installed runtime failed to materialize the selected terminal config during cold profile-install validation\n{}",
            command_output_summary(&materialization_result)
        ));
        return Ok(());
    }

    if runtime_terminal == "ghostty" {
        require_ghostty_shader_references_exist(&generated_ghostty_config, errors)?;
        require_non_empty_dir_abs(
            &generated_ghostty_effect_dir,
            "generated Ghostty cursor effect shaders directory",
            errors,
        )?;
    } else {
        require_path_exists_abs(
            &generated_wezterm_config,
            "generated WezTerm config for selected runtime variant",
            errors,
        );
    }
    Ok(())
}

fn require_path_exists(
    repo_root: &Path,
    relative_path: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    if !repo_root.join(relative_path).exists() {
        errors.push(format!("Missing {}: {}", label, relative_path));
    }
}

fn require_path_missing(
    repo_root: &Path,
    relative_path: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    if repo_root.join(relative_path).exists() {
        errors.push(format!("Unexpected {}: {}", label, relative_path));
    }
}

fn require_path_missing_abs(path: &Path, label: &str, errors: &mut Vec<String>) {
    if path.exists() {
        errors.push(format!("Unexpected {}: {}", label, path.display()));
    }
}

fn require_path_exists_abs(path: &Path, label: &str, errors: &mut Vec<String>) {
    if !path.exists() {
        errors.push(format!("Missing {}: {}", label, path.display()));
    }
}

fn require_non_empty_dir_abs(
    path: &Path,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    require_path_exists_abs(path, label, errors);
    if !path.exists() {
        return Ok(());
    }
    let has_file = fs::read_dir(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?
        .filter_map(Result::ok)
        .any(|entry| entry.path().is_file());
    if !has_file {
        errors.push(format!("{} is empty: {}", label, path.display()));
    }
    Ok(())
}

fn require_file_contains(
    repo_root: &Path,
    relative_path: &str,
    needle: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    require_file_contains_abs(&repo_root.join(relative_path), needle, label, errors)
}

fn require_file_contains_abs(
    path: &Path,
    needle: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    if !content.contains(needle) {
        errors.push(format!(
            "{} does not contain expected text `{}`: {}",
            label,
            needle,
            path.display()
        ));
    }
    Ok(())
}

fn require_file_not_contains(
    repo_root: &Path,
    relative_path: &str,
    needle: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let path = repo_root.join(relative_path);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    if content.contains(needle) {
        errors.push(format!(
            "{} still contains forbidden text `{}`: {}",
            label, needle, relative_path
        ));
    }
    Ok(())
}

fn require_list_contains(items: &[String], expected: &str, label: &str, errors: &mut Vec<String>) {
    if !items.iter().any(|item| item == expected) {
        errors.push(format!(
            "{} is missing expected entry `{}`. Found: {}",
            label,
            expected,
            items.join(", ")
        ));
    }
}

fn require_list_not_contains(
    items: &[String],
    forbidden: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    if items.iter().any(|item| item == forbidden) {
        errors.push(format!(
            "{} unexpectedly contains forbidden entry `{}`. Found: {}",
            label,
            forbidden,
            items.join(", ")
        ));
    }
}

const RUNTIME_ENV_PROBE_NU: &str = r#"let runtime_dir = ($env.YAZELIX_RUNTIME_DIR | default ""); let path_entries = ($env.PATH | default []); let runtime_libexec = (if ($runtime_dir | is-empty) { "" } else { $runtime_dir | path join "libexec" }); print ({shell: ($env.IN_YAZELIX_SHELL | default ""), runtime: $runtime_dir, path0: ($path_entries | get -o 0 | default ""), path1: ($path_entries | get -o 1 | default ""), libexec_on_path: (if ($runtime_libexec | is-empty) { false } else { $path_entries | any {|entry| $entry == $runtime_libexec } }), yzx: ((which yzx | get -o 0.path | default ""))} | to json -r)"#;
const INSTALLED_ENV_PROBE_NU: &str = r#"let runtime_dir = ($env.YAZELIX_RUNTIME_DIR | default ""); let path_entries = ($env.PATH | default []); let runtime_libexec = (if ($runtime_dir | is-empty) { "" } else { $runtime_dir | path join "libexec" }); print ({shell: ($env.IN_YAZELIX_SHELL | default ""), runtime: $runtime_dir, path0: ($path_entries | get -o 0 | default ""), path1: ($path_entries | get -o 1 | default ""), libexec_on_path: (if ($runtime_libexec | is-empty) { false } else { $path_entries | any {|entry| $entry == $runtime_libexec } }), yzx: ((which yzx | get -o 0.path | default "")), editor: ($env.EDITOR | default "")} | to json -r)"#;

fn run_packaged_yzx(
    package_root: &Path,
    temp_home: &Path,
    args: &[&str],
) -> Result<Output, String> {
    Command::new(package_root.join("bin").join("yzx"))
        .args(args)
        .env("HOME", temp_home)
        .env("XDG_CONFIG_HOME", temp_home.join(".config"))
        .env("XDG_DATA_HOME", temp_home.join(".local").join("share"))
        .env("SHELL", "/usr/bin/true")
        .env_remove("YAZELIX_DIR")
        .env_remove("YAZELIX_CONFIG_DIR")
        .env_remove("YAZELIX_CONFIG_OVERRIDE")
        .env_remove("YAZELIX_LOGS_DIR")
        .env_remove("YAZELIX_RUNTIME_DIR")
        .env_remove("YAZELIX_STATE_DIR")
        .output()
        .map_err(|error| format!("Failed to run packaged yzx: {}", error))
}

fn run_installed_yzx(repo_root: &Path, temp_home: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new(temp_home.join(".nix-profile").join("bin").join("yzx"))
        .args(args)
        .current_dir(repo_root)
        .env("HOME", temp_home)
        .env("XDG_CONFIG_HOME", temp_home.join(".config"))
        .env("XDG_DATA_HOME", temp_home.join(".local").join("share"))
        .env_remove("YAZELIX_CONFIG_DIR")
        .env_remove("YAZELIX_CONFIG_OVERRIDE")
        .env_remove("YAZELIX_LOGS_DIR")
        .env_remove("YAZELIX_RUNTIME_DIR")
        .env_remove("YAZELIX_STATE_DIR")
        .output()
        .map_err(|error| format!("Failed to run profile-installed yzx: {}", error))
}

fn run_runtime_posix_launcher_minimal_env(
    repo_root: &Path,
    temp_home: &Path,
    runtime_yzx_cli: &Path,
) -> Result<Output, String> {
    Command::new("env")
        .arg("-i")
        .arg(format!("HOME={}", temp_home.display()))
        .arg("PATH=/usr/bin:/bin")
        .arg(format!(
            "XDG_CONFIG_HOME={}",
            temp_home.join(".config").display()
        ))
        .arg(format!(
            "XDG_DATA_HOME={}",
            temp_home.join(".local").join("share").display()
        ))
        .arg("sh")
        .arg(runtime_yzx_cli)
        .arg("--version-short")
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to run runtime-local POSIX yzx launcher: {}", error))
}

fn validate_runtime_env_probe(
    probe: &JsonValue,
    runtime_root: &Path,
    expected_path0: &str,
    expected_path1: &str,
    expected_yzx: Option<&str>,
    label: &str,
    errors: &mut Vec<String>,
) {
    let shell = probe
        .get("shell")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let runtime = probe
        .get("runtime")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let path0 = probe
        .get("path0")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let path1 = probe
        .get("path1")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let libexec_on_path = probe
        .get("libexec_on_path")
        .and_then(JsonValue::as_bool)
        .unwrap_or(true);
    let yzx = probe
        .get("yzx")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let expected_yzx = expected_yzx.unwrap_or_else(|| {
        if path1.is_empty() {
            ""
        } else {
            path1
                .strip_suffix('/')
                .unwrap_or(path1)
                .trim_end_matches('/')
        }
    });
    let expected_yzx = if expected_yzx == path1 {
        format!("{expected_yzx}/yzx")
    } else {
        expected_yzx.to_string()
    };

    if shell != "true"
        || runtime != runtime_root.display().to_string()
        || path0 != expected_path0
        || path1 != expected_path1
        || libexec_on_path
        || yzx != expected_yzx
    {
        errors.push(format!(
            "{} saw the wrong Yazelix env: {}",
            label,
            format_json_value(probe)
        ));
    }
}

fn run_repo_command(repo_root: &Path, program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| {
            format!(
                "Failed to run `{}` for installed-runtime validation: {}",
                format_command(program, args),
                error
            )
        })
}

fn build_flake_output_path(repo_root: &Path, attr: &str, label: &str) -> Result<PathBuf, String> {
    let output = run_repo_command(
        repo_root,
        "nix",
        &[
            "build",
            "--no-link",
            "--print-out-paths",
            &format!(".#{attr}"),
        ],
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed while {}\n{}",
            label,
            command_output_summary(&output)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(path) = stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
    else {
        return Err(format!("{} did not return an output path", label));
    };
    if !path.exists() {
        return Err(format!(
            "{} returned missing output path {}",
            label,
            path.display()
        ));
    }
    Ok(path)
}

fn build_nix_file_output_path(
    repo_root: &Path,
    relative_file: PathBuf,
    label: &str,
) -> Result<PathBuf, String> {
    let output = Command::new("nix")
        .args([
            "build",
            "--no-link",
            "--print-out-paths",
            "--extra-experimental-features",
            "nix-command flakes",
            "--file",
        ])
        .arg(repo_root.join(&relative_file))
        .current_dir(repo_root)
        .output()
        .map_err(|error| {
            format!(
                "Failed to run nix build for {}: {}",
                relative_file.display(),
                error
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "Failed while {}\n{}",
            label,
            command_output_summary(&output)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(path) = stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
    else {
        return Err(format!("{} did not return an output path", label));
    };
    if !path.exists() {
        return Err(format!(
            "{} returned missing output path {}",
            label,
            path.display()
        ));
    }
    Ok(path)
}

fn validate_rust_routed_nu_modules(runtime_root: &Path, label: &str, errors: &mut Vec<String>) {
    let scripts_dir = runtime_root.join("nushell").join("scripts");
    for relative_path in [["yzx", "dev.nu"], ["yzx", "menu.nu"]] {
        let path = scripts_dir.join(relative_path.iter().collect::<PathBuf>());
        if !path.exists() {
            errors.push(format!(
                "Missing {} Rust-routed Nu module: {}",
                label,
                path.display()
            ));
        }
    }
}

fn json_object_keys(value: &JsonValue) -> Vec<String> {
    let mut keys = value
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    keys.sort();
    keys
}

fn command_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn command_output_summary(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = command_stderr(output);
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => "No subprocess output captured".to_string(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr),
    }
}

fn format_command(program: &str, args: &[&str]) -> String {
    std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
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

fn validate_main_contract_parity(repo_root: &Path) -> Result<Vec<String>, String> {
    let contract = read_toml_file(&repo_root.join(MAIN_CONTRACT_RELATIVE_PATH))?;
    let template = read_toml_file(&repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH))?;
    let fields = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| "main_config_contract.toml is missing its [fields] table".to_string())?;
    let declared_fields = sorted_keys(fields);
    let hm_option_names = declared_fields
        .iter()
        .filter_map(|field_path| {
            fields
                .get(field_path)
                .and_then(TomlValue::as_table)
                .and_then(|field| field.get("home_manager_option"))
                .and_then(TomlValue::as_str)
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    let hm_defaults = load_home_manager_defaults(repo_root, &hm_option_names)?;
    let mut errors = Vec::new();

    let declared_field_count = contract
        .get("contract")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("field_count"))
        .and_then(TomlValue::as_integer)
        .unwrap_or_default() as usize;
    if declared_field_count != declared_fields.len() {
        errors.push(format!(
            "main_config_contract.toml field_count mismatch: declared={}, actual={}",
            declared_field_count,
            declared_fields.len()
        ));
    }

    for field_path in declared_fields {
        let Some(field) = fields.get(&field_path).and_then(TomlValue::as_table) else {
            continue;
        };
        let hm_option = field
            .get("home_manager_option")
            .and_then(TomlValue::as_str)
            .unwrap_or_default();
        if !hm_defaults.contains_key(hm_option) {
            errors.push(format!(
                "Home Manager option `{}` is missing for main-contract field `{}`",
                hm_option, field_path
            ));
            continue;
        }

        let expected_hm_default = if field
            .get("home_manager_default_is_null")
            .and_then(TomlValue::as_bool)
            .unwrap_or(false)
        {
            JsonValue::Null
        } else {
            toml_to_json(
                field
                    .get("default")
                    .unwrap_or(&TomlValue::String(String::new())),
            )
        };
        let actual_hm_default = hm_defaults
            .get(hm_option)
            .cloned()
            .unwrap_or(JsonValue::Null);
        if !json_values_equal(&actual_hm_default, &expected_hm_default) {
            errors.push(format!(
                "Home Manager default mismatch for `{}` via `{}`: expected {}, got {}",
                field_path,
                hm_option,
                format_json_value(&expected_hm_default),
                format_json_value(&actual_hm_default)
            ));
        }

        let emit_in_template = field
            .get("emit_in_default_template")
            .and_then(TomlValue::as_bool)
            .unwrap_or(true);
        let template_value = get_nested_toml_value(&template, &split_field_path(&field_path));

        if !emit_in_template {
            if let Some(value) = template_value {
                errors.push(format!(
                    "Default template should omit `{}`, but it is present with value {}",
                    field_path,
                    format_toml_value(value)
                ));
            }
            continue;
        }

        let Some(template_value) = template_value else {
            errors.push(format!(
                "Default template is missing required field `{}`",
                field_path
            ));
            continue;
        };
        let expected_template_value = field
            .get("default")
            .ok_or_else(|| format!("Config contract field `{field_path}` is missing `default`"))?;
        if !toml_values_equal(template_value, expected_template_value) {
            errors.push(format!(
                "Default template mismatch for `{}`: expected {}, got {}",
                field_path,
                format_toml_value(expected_template_value),
                format_toml_value(template_value)
            ));
        }
    }

    Ok(errors)
}

fn load_nushell_budget_manifest(repo_root: &Path) -> Result<NushellBudgetManifest, String> {
    let manifest_path = repo_root.join(NUSHELL_BUDGET_RELATIVE_PATH);
    let content = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("Failed to read {}: {}", manifest_path.display(), error))?;
    toml::from_str(&content)
        .map_err(|error| format!("Invalid TOML in {}: {}", manifest_path.display(), error))
}

fn validate_home_manager_activation_hook_static_contract(
    repo_root: &Path,
) -> Result<Vec<String>, String> {
    let module_path = repo_root.join(MODULE_RELATIVE_PATH);
    let content = fs::read_to_string(&module_path)
        .map_err(|error| format!("Failed to read {}: {}", module_path.display(), error))?;
    let mut errors = Vec::new();

    for forbidden in ["yazi_config_merger.nu", "zellij_config_merger.nu"] {
        if content.contains(forbidden) {
            errors.push(format!(
                "{} still references deleted Home Manager activation helper `{}`",
                module_path.display(),
                forbidden
            ));
        }
    }
    if !content.contains("runtime-materialization.repair --from-env --force --summary") {
        errors.push(format!(
            "{} no longer routes Home Manager generated runtime repair through `runtime-materialization.repair --from-env --force --summary`",
            module_path.display()
        ));
    }

    Ok(errors)
}

fn load_nushell_script_paths(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    collect_nushell_script_paths(&repo_root.join("nushell").join("scripts"), &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_nushell_script_paths(dir: &Path, paths: &mut Vec<String>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {}", dir.display(), error))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            collect_nushell_script_paths(&path, paths)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("nu") {
            continue;
        }

        let relative = path
            .components()
            .skip_while(|component| component.as_os_str() != "nushell")
            .map(|component| component.as_os_str().to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join("/");
        paths.push(relative);
    }

    Ok(())
}

fn count_tracked_lines(path: &Path) -> Result<usize, String> {
    Ok(fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count())
}

fn validate_home_manager_desktop_entry_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let entry = load_home_manager_desktop_entry_contract(repo_root)?;
    let actual_exec = entry
        .get("exec")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let mut errors = Vec::new();

    if !entry
        .get("terminal")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        errors.push(
            "Home Manager desktop entry must set terminal = true so desktop-launch pre-terminal failures are visible"
                .to_string(),
        );
    }

    if actual_exec != "/tmp/profile/bin/yzx desktop launch" {
        errors.push(format!(
            "Home Manager desktop entry Exec mismatch: expected /tmp/profile/bin/yzx desktop launch, got {}",
            format_json_value(&JsonValue::String(actual_exec.to_string()))
        ));
    }

    Ok(errors)
}

fn validate_generated_state_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let fixture = setup_config_state_fixture(repo_root)?;
    let mut errors = Vec::new();

    let validation = (|| -> Result<(), String> {
        let baseline = compute_fixture_state(&fixture, &fixture.runtime_root)?;
        record_fixture_state(&fixture, &baseline)?;

        mutate_fixture_config(
            &fixture.main_config_path,
            "core.skip_welcome_screen",
            TomlValue::Boolean(true),
        )?;
        let after_runtime_only = compute_fixture_state(&fixture, &fixture.runtime_root)?;
        if baseline.config_hash != after_runtime_only.config_hash {
            errors.push(
                "Non-rebuild runtime config change unexpectedly altered config_hash".to_string(),
            );
        }
        if baseline.combined_hash != after_runtime_only.combined_hash {
            errors.push(
                "Non-rebuild runtime config change unexpectedly altered combined_hash".to_string(),
            );
        }
        if after_runtime_only.needs_refresh {
            errors.push(
                "Non-rebuild runtime config change unexpectedly marked generated state as stale"
                    .to_string(),
            );
        }

        mutate_fixture_config(
            &fixture.main_config_path,
            "editor.command",
            TomlValue::String("nvim".to_string()),
        )?;
        let after_rebuild_config = compute_fixture_state(&fixture, &fixture.runtime_root)?;
        if after_runtime_only.config_hash == after_rebuild_config.config_hash {
            errors.push("Rebuild-relevant config change did not alter config_hash".to_string());
        }
        if after_runtime_only.combined_hash == after_rebuild_config.combined_hash {
            errors.push("Rebuild-relevant config change did not alter combined_hash".to_string());
        }
        if !after_rebuild_config.needs_refresh {
            errors.push(
                "Rebuild-relevant config change did not mark generated state as stale".to_string(),
            );
        }

        record_fixture_state(&fixture, &after_rebuild_config)?;
        let after_runtime_root_change = compute_fixture_state(&fixture, &fixture.runtime_root_alt)?;
        if after_rebuild_config.config_hash != after_runtime_root_change.config_hash {
            errors.push(
                "Changing only the runtime root unexpectedly altered config_hash".to_string(),
            );
        }
        if after_rebuild_config.runtime_hash == after_runtime_root_change.runtime_hash {
            errors.push("Changing the runtime root did not alter runtime_hash".to_string());
        }
        if after_rebuild_config.combined_hash == after_runtime_root_change.combined_hash {
            errors.push("Changing the runtime root did not alter combined_hash".to_string());
        }
        if !after_runtime_root_change.needs_refresh {
            errors.push(
                "Changing the runtime root did not mark generated state as stale".to_string(),
            );
        }

        Ok(())
    })();

    if let Err(error) = validation {
        errors.push(format!(
            "Generated-state contract validation failed unexpectedly: {}",
            error
        ));
    }

    let _ = fs::remove_dir_all(&fixture.fixture_root);
    Ok(errors)
}

fn load_home_manager_defaults(
    repo_root: &Path,
    option_names: &[String],
) -> Result<JsonMap<String, JsonValue>, String> {
    let expr = build_home_manager_defaults_expr(repo_root, option_names);
    let result = run_nix_eval(repo_root, &expr)?;
    result
        .as_object()
        .cloned()
        .ok_or_else(|| "Home Manager defaults evaluation did not return a JSON object".to_string())
}

fn build_home_manager_defaults_expr(repo_root: &Path, option_names: &[String]) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut names = option_names.to_vec();
    names.sort();
    names.dedup();
    let bindings = names
        .into_iter()
        .map(|name| {
            format!(
                "  {} = module.options.programs.yazelix.{}.default;",
                name, name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    [
        "let".to_string(),
        "  pkgs = import <nixpkgs> {};".to_string(),
        "  lib = pkgs.lib;".to_string(),
        format!(
            "  module = import (builtins.toPath \"{}\") {{ inherit lib pkgs; config = {{ programs.yazelix = {{}}; xdg.configHome = \"/tmp\"; }}; }};",
            module_path
        ),
        "in {".to_string(),
        bindings,
        "}".to_string(),
    ]
    .join("\n")
}

fn load_home_manager_option_declarations(
    repo_root: &Path,
) -> Result<HashMap<String, Vec<String>>, String> {
    let expr = build_home_manager_option_declarations_expr(repo_root);
    let result = run_nix_eval(repo_root, &expr)?;
    serde_json::from_value(result).map_err(|error| {
        format!("Home Manager option declaration evaluation returned invalid JSON: {error}")
    })
}

fn build_home_manager_option_declarations_expr(repo_root: &Path) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    [
        "let".to_string(),
        "  pkgs = import <nixpkgs> {};".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; nixgl = null; };".to_string(),
        "    modules = [".to_string(),
        format!("      (builtins.toPath \"{}\")", module_path),
        "      ({ lib, ... }: {".to_string(),
        "        options.xdg.configHome = lib.mkOption { type = lib.types.str; default = \"/tmp/config\"; };".to_string(),
        "        options.xdg.dataHome = lib.mkOption { type = lib.types.str; default = \"/tmp/data\"; };".to_string(),
        "        options.xdg.dataFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.xdg.configFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.xdg.desktopEntries = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.home.packages = lib.mkOption { type = lib.types.listOf lib.types.package; default = []; };".to_string(),
        "        options.home.activation = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.home.profileDirectory = lib.mkOption { type = lib.types.str; default = \"/tmp/profile\"; };".to_string(),
        "      })".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        "in builtins.mapAttrs (_: option: map builtins.toString option.declarations) eval.options.programs.yazelix".to_string(),
    ]
    .join("\n")
}

fn load_home_manager_desktop_entry_contract(
    repo_root: &Path,
) -> Result<JsonMap<String, JsonValue>, String> {
    let expr = build_home_manager_desktop_entry_expr(repo_root);
    let result = run_nix_eval(repo_root, &expr)?;
    result.as_object().cloned().ok_or_else(|| {
        "Home Manager desktop-entry evaluation did not return a JSON object".to_string()
    })
}

fn build_home_manager_desktop_entry_expr(repo_root: &Path) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    [
        "let".to_string(),
        "  pkgs = import <nixpkgs> {};".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; nixgl = null; };".to_string(),
        "    modules = [".to_string(),
        format!("      (builtins.toPath \"{}\")", module_path),
        "      ({ lib, ... }: {".to_string(),
        "        options.xdg.configHome = lib.mkOption { type = lib.types.str; default = \"/tmp/config\"; };".to_string(),
        "        options.xdg.dataHome = lib.mkOption { type = lib.types.str; default = \"/tmp/data\"; };".to_string(),
        "        options.xdg.dataFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.xdg.configFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.xdg.desktopEntries = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.home.packages = lib.mkOption { type = lib.types.listOf lib.types.package; default = []; };".to_string(),
        "        options.home.activation = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.home.profileDirectory = lib.mkOption { type = lib.types.str; default = \"/tmp/profile\"; };".to_string(),
        "        config.programs.yazelix.enable = true;".to_string(),
        "      })".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        "in {".to_string(),
        "  exec = eval.config.xdg.desktopEntries.yazelix.exec or \"\";".to_string(),
        "  terminal = eval.config.xdg.desktopEntries.yazelix.terminal or false;".to_string(),
        "}".to_string(),
    ]
    .join("\n")
}

fn run_nix_eval(repo_root: &Path, expr: &str) -> Result<JsonValue, String> {
    let output = Command::new("nix")
        .args(["eval", "--impure", "--json", "--expr", expr])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to run `nix eval`: {}", error))?;
    if !output.status.success() {
        return Err(format!(
            "Failed to evaluate Nix expression for validator.\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice::<JsonValue>(&output.stdout)
        .map_err(|error| format!("Failed to parse `nix eval` JSON output: {}", error))
}

#[derive(Debug)]
struct ConfigStateFixture {
    fixture_root: PathBuf,
    runtime_root: PathBuf,
    runtime_root_alt: PathBuf,
    main_config_path: PathBuf,
    managed_config_path: PathBuf,
    state_path: PathBuf,
}

fn setup_config_state_fixture(repo_root: &Path) -> Result<ConfigStateFixture, String> {
    let fixture_root = create_unique_temp_dir("yazelix_config_contract")?;
    let runtime_root = fixture_root.join("runtime");
    let runtime_root_alt = fixture_root.join("runtime_alt");
    let config_root = fixture_root.join("config");
    let user_config_dir = config_root.join("user_configs");
    let home_root = fixture_root.join("home");
    fs::create_dir_all(&runtime_root)
        .map_err(|error| format!("Failed to create {}: {}", runtime_root.display(), error))?;
    fs::create_dir_all(&runtime_root_alt)
        .map_err(|error| format!("Failed to create {}: {}", runtime_root_alt.display(), error))?;
    fs::create_dir_all(&user_config_dir)
        .map_err(|error| format!("Failed to create {}: {}", user_config_dir.display(), error))?;
    fs::create_dir_all(&home_root)
        .map_err(|error| format!("Failed to create {}: {}", home_root.display(), error))?;

    for relative_path in [
        TAPLO_RELATIVE_PATH,
        MAIN_TEMPLATE_RELATIVE_PATH,
        MAIN_CONTRACT_RELATIVE_PATH,
    ] {
        copy_fixture_file(repo_root, &runtime_root, relative_path)?;
        copy_fixture_file(repo_root, &runtime_root_alt, relative_path)?;
    }

    let main_config_path = user_config_dir.join("yazelix.toml");
    fs::copy(
        repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH),
        &main_config_path,
    )
    .map_err(|error| {
        format!(
            "Failed to copy {} into fixture config: {}",
            repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH).display(),
            error
        )
    })?;

    Ok(ConfigStateFixture {
        fixture_root: fixture_root.clone(),
        runtime_root,
        runtime_root_alt,
        main_config_path: main_config_path.clone(),
        managed_config_path: main_config_path,
        state_path: home_root
            .join(".local")
            .join("share")
            .join("yazelix")
            .join("state")
            .join("rebuild_hash"),
    })
}

fn create_unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
    let base = env::temp_dir();
    for attempt in 0..100u32 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("System clock error: {}", error))?
            .as_nanos();
        let candidate = base.join(format!(
            "{}_{}_{}_{}",
            prefix,
            process::id(),
            nanos,
            attempt
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Failed to create temporary directory {}: {}",
                    candidate.display(),
                    error
                ));
            }
        }
    }
    Err(format!(
        "Failed to create unique temporary directory for {}",
        prefix
    ))
}

fn prepare_temp_home(temp_home: &Path) -> Result<(), String> {
    if let Some(parent) = temp_home.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
    }
    if temp_home.exists() {
        fs::remove_dir_all(temp_home)
            .map_err(|error| format!("Failed to remove {}: {}", temp_home.display(), error))?;
    }
    Ok(())
}

fn relative_display(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn resolve_ghostty_shader_reference(ghostty_config_path: &Path, shader_ref: &str) -> PathBuf {
    let raw_ref = shader_ref.trim().trim_matches('"');
    let path = Path::new(raw_ref);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let relative = raw_ref.strip_prefix("./").unwrap_or(raw_ref);
    ghostty_config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(relative)
}

fn require_ghostty_shader_references_exist(
    ghostty_config_path: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    require_path_exists_abs(ghostty_config_path, "generated Ghostty config", errors);
    if !ghostty_config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(ghostty_config_path).map_err(|error| {
        format!(
            "Failed to read generated Ghostty config {}: {}",
            ghostty_config_path.display(),
            error
        )
    })?;
    let shader_refs = content
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("custom-shader = ")
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    if shader_refs.is_empty() {
        errors.push(format!(
            "Generated Ghostty config references no shader assets: {}",
            ghostty_config_path.display()
        ));
    }
    for shader_ref in shader_refs {
        let shader_path = resolve_ghostty_shader_reference(ghostty_config_path, &shader_ref);
        require_path_exists_abs(
            &shader_path,
            &format!("generated Ghostty shader `{shader_ref}`"),
            errors,
        );
    }
    Ok(())
}

fn copy_fixture_file(
    source_root: &Path,
    target_root: &Path,
    relative_path: &str,
) -> Result<(), String> {
    let source = source_root.join(relative_path);
    let target = target_root.join(relative_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
    }
    fs::copy(&source, &target).map_err(|error| {
        format!(
            "Failed to copy {} to {}: {}",
            source.display(),
            target.display(),
            error
        )
    })?;
    Ok(())
}

fn compute_fixture_state(
    fixture: &ConfigStateFixture,
    runtime_root: &Path,
) -> Result<ConfigStateData, String> {
    compute_config_state(&ComputeConfigStateRequest {
        config_path: fixture.main_config_path.clone(),
        default_config_path: runtime_root.join(MAIN_TEMPLATE_RELATIVE_PATH),
        contract_path: runtime_root.join(MAIN_CONTRACT_RELATIVE_PATH),
        runtime_dir: runtime_root.to_path_buf(),
        state_path: fixture.state_path.clone(),
    })
    .map_err(|error| error.message())
}

fn record_fixture_state(
    fixture: &ConfigStateFixture,
    state: &ConfigStateData,
) -> Result<(), String> {
    record_config_state(&RecordConfigStateRequest {
        config_file: state.config_file.clone(),
        managed_config_path: fixture.managed_config_path.clone(),
        state_path: fixture.state_path.clone(),
        config_hash: state.config_hash.clone(),
        runtime_hash: state.runtime_hash.clone(),
    })
    .map_err(|error| error.message())?;
    Ok(())
}

fn mutate_fixture_config(
    config_path: &Path,
    field_path: &str,
    value: TomlValue,
) -> Result<(), String> {
    let mut table = read_toml_file(config_path)?;
    set_nested_toml_value(&mut table, &split_field_path(field_path), value);
    fs::write(
        config_path,
        toml::to_string(&table)
            .map_err(|error| format!("Failed to serialize fixture config: {}", error))?,
    )
    .map_err(|error| format!("Failed to write {}: {}", config_path.display(), error))
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
        if !GUARDED_FILES.contains(&path.as_str()) && !ACK_REQUIRED_FILES.contains(&path.as_str()) {
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
    if !version_bumped
        && !changed_ack_required.is_empty()
        && !docs_changed
        && !ack_only_notes_change
    {
        errors.push(
            "CI: guarded config-contract changes must update both CHANGELOG.md and docs/upgrade_notes.toml in the same diff"
                .to_string(),
        );
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
            .any(|path| path == "nushell/scripts/utils/constants.nu")
        && !docs_changed
    {
        errors.push(
            "CI: changes to nushell/scripts/utils/constants.nu must update both CHANGELOG.md and docs/upgrade_notes.toml"
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
            &format!("{base}:nushell/scripts/utils/constants.nu"),
        ])
        .output()
        .map_err(|error| format!("Failed to run `git show` for {}: {}", base, error))?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(extract_version_from_constants(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn extract_version_from_constants(content: &str) -> Option<String> {
    const PREFIX: &str = "export const YAZELIX_VERSION = \"";
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix(PREFIX)
            .and_then(|rest| rest.strip_suffix('"'))
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

fn read_toml_file(path: &Path) -> Result<TomlTable, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    parse_toml_from_str(&raw, &path.display().to_string())
}

fn parse_toml_from_bytes(bytes: &[u8], label: &str) -> Result<TomlTable, String> {
    let raw = String::from_utf8(bytes.to_vec())
        .map_err(|error| format!("Failed to decode {} as UTF-8: {}", label, error))?;
    parse_toml_from_str(&raw, label)
}

fn parse_toml_from_str(raw: &str, label: &str) -> Result<TomlTable, String> {
    toml::from_str::<TomlTable>(raw)
        .map_err(|error| format!("Failed to parse {} as TOML: {}", label, error))
}

fn split_field_path(path: &str) -> Vec<&str> {
    path.split('.').collect()
}

fn get_nested_toml_value<'a>(table: &'a TomlTable, path: &[&str]) -> Option<&'a TomlValue> {
    if path.is_empty() {
        return None;
    }
    let mut current = table.get(path[0])?;
    for segment in &path[1..] {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

fn set_nested_toml_value(table: &mut TomlTable, path: &[&str], value: TomlValue) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        table.insert(path[0].to_string(), value);
        return;
    }
    let entry = table
        .entry(path[0].to_string())
        .or_insert_with(|| TomlValue::Table(TomlTable::new()));
    if !entry.is_table() {
        *entry = TomlValue::Table(TomlTable::new());
    }
    if let Some(child) = entry.as_table_mut() {
        set_nested_toml_value(child, &path[1..], value);
    }
}

fn toml_to_json(value: &TomlValue) -> JsonValue {
    match value {
        TomlValue::String(value) => JsonValue::String(value.clone()),
        TomlValue::Integer(value) => JsonValue::Number((*value).into()),
        TomlValue::Float(value) => JsonNumber::from_f64(*value)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        TomlValue::Boolean(value) => JsonValue::Bool(*value),
        TomlValue::Datetime(value) => JsonValue::String(value.to_string()),
        TomlValue::Array(values) => JsonValue::Array(values.iter().map(toml_to_json).collect()),
        TomlValue::Table(table) => JsonValue::Object(
            table
                .iter()
                .map(|(key, value)| (key.clone(), toml_to_json(value)))
                .collect(),
        ),
    }
}

fn json_values_equal(left: &JsonValue, right: &JsonValue) -> bool {
    match (left, right) {
        (JsonValue::Null, JsonValue::Null) => true,
        (JsonValue::Bool(left), JsonValue::Bool(right)) => left == right,
        (JsonValue::Number(left), JsonValue::Number(right)) => left
            .as_f64()
            .zip(right.as_f64())
            .map(|(l, r)| l == r)
            .unwrap_or(false),
        (JsonValue::String(left), JsonValue::String(right)) => left == right,
        (JsonValue::Array(left), JsonValue::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| json_values_equal(left, right))
        }
        (JsonValue::Object(left), JsonValue::Object(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, left_value)| {
                    right
                        .get(key)
                        .map(|right_value| json_values_equal(left_value, right_value))
                        .unwrap_or(false)
                })
        }
        _ => false,
    }
}

fn toml_values_equal(left: &TomlValue, right: &TomlValue) -> bool {
    match (left, right) {
        (TomlValue::String(left), TomlValue::String(right)) => left == right,
        (TomlValue::Integer(left), TomlValue::Integer(right)) => left == right,
        (TomlValue::Float(left), TomlValue::Float(right)) => left == right,
        (TomlValue::Integer(left), TomlValue::Float(right))
        | (TomlValue::Float(right), TomlValue::Integer(left)) => (*left as f64) == *right,
        (TomlValue::Boolean(left), TomlValue::Boolean(right)) => left == right,
        (TomlValue::Datetime(left), TomlValue::Datetime(right)) => left == right,
        (TomlValue::Array(left), TomlValue::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| toml_values_equal(left, right))
        }
        (TomlValue::Table(left), TomlValue::Table(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, left_value)| {
                    right
                        .get(key)
                        .map(|right_value| toml_values_equal(left_value, right_value))
                        .unwrap_or(false)
                })
        }
        _ => false,
    }
}

fn format_json_value(value: &JsonValue) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unrenderable json>".to_string())
}

fn format_toml_value(value: &TomlValue) -> String {
    format_json_value(&toml_to_json(value))
}

fn as_string_list(value: Option<&TomlValue>) -> Vec<String> {
    value
        .and_then(TomlValue::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(TomlValue::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn sorted_keys(table: &TomlTable) -> Vec<String> {
    let mut keys = table.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

fn escape_nix_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
