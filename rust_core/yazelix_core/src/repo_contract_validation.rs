use crate::config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateRequest, compute_config_state,
    record_config_state,
};
use crate::control_plane::read_yazelix_version_from_runtime;
use crate::repo_validation::ValidationReport;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::{Table as TomlTable, Value as TomlValue};

const MAIN_TEMPLATE_RELATIVE_PATH: &str = "yazelix_default.toml";
const MODULE_RELATIVE_PATH: &str = "home_manager/module.nix";
const MAIN_CONTRACT_RELATIVE_PATH: &str = "config_metadata/main_config_contract.toml";
const TAPLO_RELATIVE_PATH: &str = ".taplo.toml";
const GUARDED_FILES: &[&str] = &[
    "nushell/scripts/utils/constants.nu",
    "yazelix_default.toml",
    "home_manager/module.nix",
    "nushell/scripts/utils/config_schema.nu",
    "docs/upgrade_notes.toml",
    "CHANGELOG.md",
];
const ACK_REQUIRED_FILES: &[&str] = &[
    "yazelix_default.toml",
    "home_manager/module.nix",
    "nushell/scripts/utils/config_schema.nu",
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

pub fn validate_config_surface_contract(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    report
        .errors
        .extend(validate_main_contract_parity(repo_root)?);
    report
        .errors
        .extend(validate_home_manager_desktop_entry_contract(repo_root)?);
    report
        .errors
        .extend(validate_generated_state_contract(repo_root)?);
    Ok(report)
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
