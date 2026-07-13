use super::{
    HOME_MANAGER_MODULE_DECLARATION_PATH, MAIN_CONTRACT_RELATIVE_PATH, MAIN_TEMPLATE_RELATIVE_PATH,
    MODULE_RELATIVE_PATH, SETTINGS_SCHEMA_RELATIVE_PATH, create_unique_temp_dir, escape_nix_string,
    format_json_value, format_toml_value, get_nested_toml_value, read_toml_file, run_nix_eval,
    set_nested_toml_value, sorted_keys, split_field_path, toml_to_json, toml_values_equal,
};
use crate::repo_validation::ValidationReport;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::{Table as TomlTable, Value as TomlValue};
use yazelix_core::config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateRequest, compute_config_state,
    record_config_state,
};
use yazelix_core::settings_surface::{
    parse_config_value, read_config_table, read_config_value, render_config_value,
};
use yazelix_core::terminal_variant::{
    terminal_desktop_entry_id, terminal_desktop_entry_name, terminal_display_name,
};
use yazelix_core::{RuntimeApplyMode, ZELLIJ_ACTIONS, runtime_apply_mode_codes};

const HOME_MANAGER_DEFAULT_TERMINAL: &str = "mars";
const MAIN_CONFIG_CONTRACT_ID: &str = "yazelix.config";
const MAIN_CONFIG_CONTRACT_VERSION: u64 = 2;
const MAIN_CONFIG_CONTRACT_CHANGE_IDS: &[&str] = &["classic-root-to-nova-v1"];

pub fn validate_config_surface_contract(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    for errors in [
        validate_main_template_schema_structure(repo_root)?,
        validate_main_contract_parity(repo_root)?,
        validate_ratconfig_contract_guard(repo_root)?,
        validate_nova_keybinding_registry_defaults(repo_root)?,
        validate_home_manager_option_declaration_contract(repo_root)?,
        validate_home_manager_native_file_contract(repo_root)?,
        validate_home_manager_desktop_entry_contract(repo_root)?,
        validate_home_manager_activation_contract(repo_root)?,
        validate_generated_state_contract(repo_root)?,
        validate_startup_snapshot_env_contract(repo_root)?,
    ] {
        report.errors.extend(errors);
    }
    Ok(report)
}

fn validate_main_template_schema_structure(repo_root: &Path) -> Result<Vec<String>, String> {
    let template = read_config_value(&repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH))
        .map_err(|error| error.message())?;
    let schema_path = repo_root.join(SETTINGS_SCHEMA_RELATIVE_PATH);
    let raw = fs::read_to_string(&schema_path)
        .map_err(|error| format!("Failed to read {}: {error}", schema_path.display()))?;
    let schema = serde_json::from_str::<JsonValue>(&raw)
        .map_err(|error| format!("Failed to parse {}: {error}", schema_path.display()))?;
    let mut errors = Vec::new();
    let extension = schema.get("x-yazelix").and_then(JsonValue::as_object);
    for (field, expected) in [
        ("schema_role", "multi_source_config_ui"),
        ("root_file", "~/.config/yazelix/config.toml"),
        ("cursor_file", "~/.config/yazelix/cursors.toml"),
    ] {
        if extension
            .and_then(|value| value.get(field))
            .and_then(JsonValue::as_str)
            != Some(expected)
        {
            errors.push(format!(
                "Settings UI schema x-yazelix.{field} must be `{expected}`"
            ));
        }
    }
    if extension.is_some_and(|value| value.contains_key("canonical_file")) {
        errors.push(
            "The multi-source settings UI schema must not advertise one canonical_file".to_string(),
        );
    }
    if schema
        .pointer("/properties/cursors/x-yazelix/source_file")
        .and_then(JsonValue::as_str)
        != Some("~/.config/yazelix/cursors.toml")
    {
        errors.push(
            "The settings UI cursor subtree must declare cursors.toml as its source_file"
                .to_string(),
        );
    }
    validate_schema_structure(&schema, Some(&template), "$", &mut errors);
    Ok(errors)
}

fn validate_schema_structure(
    schema: &JsonValue,
    value: Option<&JsonValue>,
    path: &str,
    errors: &mut Vec<String>,
) {
    if schema.get("type").and_then(JsonValue::as_str) == Some("object") {
        let properties = schema.get("properties").and_then(JsonValue::as_object);
        for field in schema
            .get("required")
            .and_then(JsonValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(JsonValue::as_str)
        {
            if properties.is_none_or(|properties| !properties.contains_key(field)) {
                errors.push(format!(
                    "Settings schema object `{path}` requires `{field}` outside that object's properties"
                ));
            }
        }
        let additional = schema.get("additionalProperties");
        if !matches!(
            additional,
            Some(JsonValue::Bool(false) | JsonValue::Object(_))
        ) {
            errors.push(format!(
                "Settings schema object `{path}` must close or type additionalProperties"
            ));
        }
        for (name, child) in properties.into_iter().flatten() {
            validate_schema_structure(
                child,
                value
                    .and_then(JsonValue::as_object)
                    .and_then(|value| value.get(name)),
                &format!("{path}.{name}"),
                errors,
            );
        }
        for (name, child) in value
            .and_then(JsonValue::as_object)
            .into_iter()
            .flatten()
            .filter(|(name, _)| properties.is_none_or(|fields| !fields.contains_key(*name)))
        {
            match additional.filter(|schema| schema.is_object()) {
                Some(schema) => validate_schema_structure(
                    schema,
                    Some(child),
                    &format!("{path}.{name}"),
                    errors,
                ),
                None => errors.push(format!(
                    "`{path}.{name}` violates the settings schema additionalProperties contract"
                )),
            }
        }
    }
    if let Some(items) = schema.get("items") {
        for item in value.and_then(JsonValue::as_array).into_iter().flatten() {
            validate_schema_structure(items, Some(item), &format!("{path}[]"), errors)
        }
    }
}

pub fn validate_home_manager_option_declaration_contract(
    repo_root: &Path,
) -> Result<Vec<String>, String> {
    let declarations = load_home_manager_option_declarations(repo_root)?;
    let mut errors = Vec::new();
    let actual = declarations.keys().cloned().collect::<BTreeSet<_>>();
    let expected = [
        "config.cursors",
        "config.helix.config",
        "config.helix.init",
        "config.helix.languages",
        "config.helix.module",
        "config.mars",
        "config.nu.config",
        "config.nu.env",
        "config.settings",
        "config.starship",
        "config.yazi.config",
        "config.yazi.init",
        "config.yazi.keymap",
        "config.yazi.package",
        "config.yazi.theme",
        "config.zellij",
        "enable",
        "package",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    if actual != expected {
        let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
        let extra = actual.difference(&expected).cloned().collect::<Vec<_>>();
        errors.push(format!(
            "Home Manager must expose only the Nova package-plus-sidecars API; missing={missing:?}, extra={extra:?}"
        ));
    }
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

fn validate_main_contract_parity(repo_root: &Path) -> Result<Vec<String>, String> {
    let contract = read_toml_file(&repo_root.join(MAIN_CONTRACT_RELATIVE_PATH))?;
    let template_json = read_config_value(&repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH))
        .map_err(|error| error.message())?;
    let template = read_config_table(
        &repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH),
        "read_main_settings_default",
    )
    .map_err(|error| error.message())?;
    let fields = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| "main_config_contract.toml is missing its [fields] table".to_string())?;
    let declared_fields = sorted_keys(fields);
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
        validate_main_contract_apply_mode(&field_path, field, &mut errors);
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

        if field
            .get("home_manager_default_is_null")
            .and_then(TomlValue::as_bool)
            .unwrap_or(false)
            && field.get("kind").and_then(TomlValue::as_str) == Some("helix_external")
        {
            match get_nested_json_value(&template_json, &split_field_path(&field_path)) {
                Some(JsonValue::Null) => continue,
                Some(value) => {
                    errors.push(format!(
                        "Default template mismatch for `{}`: expected null, got {}",
                        field_path,
                        format_json_value(value)
                    ));
                    continue;
                }
                None => {
                    errors.push(format!(
                        "Default template is missing required field `{}`",
                        field_path
                    ));
                    continue;
                }
            }
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

fn get_nested_json_value<'a>(value: &'a JsonValue, path: &[&str]) -> Option<&'a JsonValue> {
    let mut current = value;
    for segment in path {
        current = current.as_object()?.get(*segment)?;
    }
    Some(current)
}

fn validate_main_contract_apply_mode(
    field_path: &str,
    field: &TomlTable,
    errors: &mut Vec<String>,
) {
    let Some(apply_mode) = field.get("apply_mode").and_then(TomlValue::as_str) else {
        errors.push(format!(
            "main_config_contract.toml field `{field_path}` is missing apply_mode"
        ));
        return;
    };

    if apply_mode.parse::<RuntimeApplyMode>().is_err() {
        errors.push(format!(
            "main_config_contract.toml field `{field_path}` has unsupported apply_mode `{}`; expected one of: {}",
            apply_mode,
            runtime_apply_mode_codes().join(", ")
        ));
    }
}

fn validate_ratconfig_contract_guard(repo_root: &Path) -> Result<Vec<String>, String> {
    let contract = read_toml_file(&repo_root.join(MAIN_CONTRACT_RELATIVE_PATH))?;
    let mut errors = Vec::new();
    if main_contract_ratconfig_id(&contract).as_deref() != Some(MAIN_CONFIG_CONTRACT_ID) {
        errors.push(format!(
            "main_config_contract.toml must declare ratconfig_contract_id = {MAIN_CONFIG_CONTRACT_ID}"
        ));
    }
    match main_contract_ratconfig_version(&contract) {
        Some(version) if version == MAIN_CONFIG_CONTRACT_VERSION => {}
        Some(version) => errors.push(format!(
            "main_config_contract.toml declares ratconfig_contract_version = {version}, expected {MAIN_CONFIG_CONTRACT_VERSION} for {MAIN_CONFIG_CONTRACT_ID}"
        )),
        None => errors.push(
            "main_config_contract.toml [contract] must declare ratconfig_contract_version"
                .to_string(),
        ),
    }
    let expected_change_ids = MAIN_CONFIG_CONTRACT_CHANGE_IDS
        .iter()
        .map(|id| (*id).to_string())
        .collect::<Vec<_>>();
    match main_contract_ratconfig_applied_change_ids(&contract) {
        Some(change_ids) if change_ids == expected_change_ids => {}
        Some(change_ids) => errors.push(format!(
            "main_config_contract.toml declares ratconfig_applied_change_ids = [{}], expected [{}] for {MAIN_CONFIG_CONTRACT_ID}",
            change_ids.join(", "),
            expected_change_ids.join(", ")
        )),
        None => errors.push(
            "main_config_contract.toml [contract] must declare ratconfig_applied_change_ids as a string array"
                .to_string(),
        ),
    }
    errors.extend(validate_home_manager_sparse_config(repo_root)?);
    errors.extend(validate_ratconfig_contract_diff_guard(
        repo_root,
        &contract,
        MAIN_CONFIG_CONTRACT_VERSION,
    )?);
    Ok(errors)
}

fn validate_ratconfig_contract_diff_guard(
    repo_root: &Path,
    current_contract: &TomlTable,
    current_version: u64,
) -> Result<Vec<String>, String> {
    let diff_base = config_surface_diff_base();
    validate_ratconfig_contract_diff_guard_from_base(
        repo_root,
        current_contract,
        current_version,
        &diff_base,
    )
}

fn validate_ratconfig_contract_diff_guard_from_base(
    repo_root: &Path,
    current_contract: &TomlTable,
    current_version: u64,
    diff_base: &str,
) -> Result<Vec<String>, String> {
    if !git_ref_exists(repo_root, &diff_base)? {
        return Ok(Vec::new());
    }
    let Some(previous_raw) =
        load_file_from_git_ref(repo_root, &diff_base, MAIN_CONTRACT_RELATIVE_PATH)?
    else {
        return Ok(Vec::new());
    };
    let previous_contract = toml::from_str::<TomlTable>(&previous_raw).map_err(|error| {
        format!("Failed to parse {diff_base}:{MAIN_CONTRACT_RELATIVE_PATH} as TOML: {error}")
    })?;
    if main_contract_ratconfig_id(&previous_contract).as_deref()
        != main_contract_ratconfig_id(current_contract).as_deref()
    {
        return Ok(Vec::new());
    }
    let changes = semantic_main_config_contract_changes(&previous_contract, current_contract);
    if changes.is_empty() {
        return Ok(Vec::new());
    }

    let previous_version = main_contract_ratconfig_version(&previous_contract).or_else(|| {
        previous_settings_contract_version(repo_root, &diff_base)
            .ok()
            .flatten()
    });
    let Some(previous_version) = previous_version else {
        return Ok(vec![format!(
            "main_config_contract.toml changed user-facing settings semantics, but the validator could not determine the previous ratconfig contract version from {diff_base}"
        )]);
    };
    if current_version <= previous_version {
        return Ok(vec![format!(
            "main_config_contract.toml changed user-facing settings semantics without a ratconfig migration or manual blocker: {}. Bump the {MAIN_CONFIG_CONTRACT_ID} contract version and add the matching Ratconfig change, or encode the change as a manual blocker when automation is unsafe.",
            changes.join("; ")
        )]);
    }

    Ok(Vec::new())
}

fn semantic_main_config_contract_changes(
    previous_contract: &TomlTable,
    current_contract: &TomlTable,
) -> Vec<String> {
    let previous_fields = main_contract_fields(previous_contract);
    let current_fields = main_contract_fields(current_contract);
    let mut changes = Vec::new();

    for field_path in previous_fields.keys() {
        if !current_fields.contains_key(field_path) {
            changes.push(format!("removed field `{field_path}`"));
        }
    }
    for field_path in current_fields.keys() {
        if !previous_fields.contains_key(field_path) {
            changes.push(format!("added field `{field_path}`"));
        }
    }
    for (field_path, previous_field) in &previous_fields {
        let Some(current_field) = current_fields.get(field_path) else {
            continue;
        };
        for key in [
            "kind",
            "default",
            "parser_key",
            "parser_behavior",
            "validation",
            "nullable",
            "home_manager_default_is_null",
            "emit_in_default_template",
            "min",
            "max",
        ] {
            if previous_field.get(key) != current_field.get(key) {
                changes.push(format!("changed `{field_path}.{key}`"));
            }
        }
        for key in ["allowed_values", "allowed_symbols"] {
            changes.extend(removed_string_list_values(
                previous_field,
                current_field,
                field_path,
                key,
            ));
        }
    }

    changes
}

fn main_contract_fields(contract: &TomlTable) -> BTreeMap<String, TomlTable> {
    contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(path, value)| {
                    value.as_table().map(|field| (path.clone(), field.clone()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn removed_string_list_values(
    previous_field: &TomlTable,
    current_field: &TomlTable,
    field_path: &str,
    key: &str,
) -> Vec<String> {
    let previous = string_values(previous_field.get(key));
    let current = string_values(current_field.get(key));
    previous
        .difference(&current)
        .map(|value| format!("removed `{field_path}.{key}` value `{value}`"))
        .collect()
}

fn string_values(value: Option<&TomlValue>) -> BTreeSet<String> {
    value
        .and_then(TomlValue::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(TomlValue::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn main_contract_ratconfig_version(contract: &TomlTable) -> Option<u64> {
    contract
        .get("contract")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("ratconfig_contract_version"))
        .and_then(TomlValue::as_integer)
        .and_then(|value| u64::try_from(value).ok())
}

fn main_contract_ratconfig_id(contract: &TomlTable) -> Option<String> {
    contract
        .get("contract")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("ratconfig_contract_id"))
        .and_then(TomlValue::as_str)
        .map(ToOwned::to_owned)
}

fn main_contract_ratconfig_applied_change_ids(contract: &TomlTable) -> Option<Vec<String>> {
    contract
        .get("contract")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("ratconfig_applied_change_ids"))
        .and_then(toml_string_array)
}

fn toml_string_array(value: &TomlValue) -> Option<Vec<String>> {
    value.as_array().and_then(|items| {
        items
            .iter()
            .map(|item| item.as_str().map(ToOwned::to_owned))
            .collect()
    })
}

fn validate_home_manager_sparse_config(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    if load_home_manager_managed_config_toml(repo_root, None)?.is_some() {
        errors.push(
            "Home Manager config.settings = null must leave yazelix/config.toml absent".to_string(),
        );
    }
    let empty = load_home_manager_managed_config_toml(repo_root, Some("{}"))?.ok_or_else(|| {
        "Home Manager explicit empty config.settings did not create config.toml".to_string()
    })?;
    errors.extend(validate_home_manager_sparse_config_content(
        Path::new("config.toml"),
        &empty,
    )?);
    let explicit = load_home_manager_managed_config_toml(
        repo_root,
        Some("{ popups.btm = { command = \"btm\"; keybinding = \"Alt Shift B\"; }; }"),
    )?
    .ok_or_else(|| {
        "Home Manager explicit config.settings did not create config.toml".to_string()
    })?;
    errors.extend(validate_home_manager_explicit_config_content(
        Path::new("config.toml"),
        &explicit,
    )?);
    Ok(errors)
}

fn validate_home_manager_sparse_config_content(
    label: &Path,
    raw: &str,
) -> Result<Vec<String>, String> {
    let value = parse_config_value(label, raw).map_err(|error| error.message().to_string())?;
    match value.as_object() {
        Some(root) if root.is_empty() => Ok(Vec::new()),
        Some(root) => Ok(vec![format!(
            "Home Manager config.settings = {{}} must render an empty sparse config.toml, found top-level keys: {}",
            root.keys().cloned().collect::<Vec<_>>().join(", ")
        )]),
        None => Ok(vec![
            "Home Manager-generated config.toml must be a TOML table".to_string(),
        ]),
    }
}

fn validate_home_manager_explicit_config_content(
    label: &Path,
    raw: &str,
) -> Result<Vec<String>, String> {
    let actual = parse_config_value(label, raw).map_err(|error| error.message().to_string())?;
    let expected = serde_json::json!({
        "popups": { "btm": { "command": "btm", "keybinding": "Alt Shift B" } }
    });
    if actual == expected {
        Ok(Vec::new())
    } else {
        Ok(vec![format!(
            "Home Manager must not materialize omitted optional popup fields, got {}",
            format_json_value(&actual)
        )])
    }
}

fn validate_home_manager_native_file_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let result = run_nix_eval(
        repo_root,
        &build_home_manager_native_file_contract_expr(repo_root),
    )?;
    let object = result.as_object().ok_or_else(|| {
        "Home Manager native-file validation did not return a JSON object".to_string()
    })?;
    let actual_paths = object
        .get("paths")
        .and_then(JsonValue::as_array)
        .into_iter()
        .flatten()
        .filter_map(JsonValue::as_str)
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let expected_paths = [
        "yazelix/cursors.toml",
        "yazelix/helix/config.toml",
        "yazelix/helix/helix.scm",
        "yazelix/helix/init.scm",
        "yazelix/helix/languages.toml",
        "yazelix/mars/config.toml",
        "yazelix/nu/config.nu",
        "yazelix/nu/env.nu",
        "yazelix/starship.toml",
        "yazelix/yazi/init.lua",
        "yazelix/yazi/keymap.toml",
        "yazelix/yazi/package.toml",
        "yazelix/yazi/theme.toml",
        "yazelix/yazi/yazi.toml",
        "yazelix/zellij/config.kdl",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let mut errors = Vec::new();
    if actual_paths != expected_paths {
        errors.push(format!(
            "Home Manager native files must use the canonical Yazelix paths; expected={expected_paths:?}, actual={actual_paths:?}"
        ));
    }
    for (field, message) in [
        (
            "valid",
            "Home Manager must accept exactly one of native-file text or source",
        ),
        (
            "rejectsZero",
            "Home Manager must reject a declared native file with neither text nor source",
        ),
        (
            "rejectsTwo",
            "Home Manager must reject a declared native file with both text and source",
        ),
    ] {
        if object.get(field).and_then(JsonValue::as_bool) != Some(true) {
            errors.push(message.to_string());
        }
    }
    Ok(errors)
}

fn build_home_manager_native_file_contract_expr(repo_root: &Path) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"x86_64-linux\"; };".to_string(),
        "  lib = pkgs.lib.extend (_: super: { hm = { dag = { entryAfter = after: data: { inherit after data; }; }; }; });".to_string(),
        "  fakePackage = pkgs.runCommand \"yazelix\" {} ''mkdir -p $out/bin $out/libexec $out/toolbin; touch $out/bin/yzx $out/libexec/yzx_core $out/libexec/yzx_control'';".to_string(),
        "  nativeSource = builtins.toFile \"native-config\" \"source\";".to_string(),
        format!(
            "  yazelixModule = import (builtins.toPath \"{}\") {{ defaultPackageFor = _system: fakePackage; }};",
            module_path
        ),
        "  baseModules = [ yazelixModule".to_string(),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, false));
    lines.extend([
        "  ];".to_string(),
        "  evaluate = extra: lib.evalModules { specialArgs = { inherit pkgs; }; modules = baseModules ++ [ extra ]; };".to_string(),
        "  validEval = evaluate { config.programs.yazelix = { enable = true; config = {".to_string(),
        "    cursors.source = nativeSource;".to_string(),
        "    mars.text = \"\"; zellij.text = \"\"; starship.text = \"\";".to_string(),
        "    helix = { config.text = \"\"; languages.text = \"\"; module.text = \"\"; init.text = \"\"; };".to_string(),
        "    yazi = { config.text = \"\"; init.text = \"\"; keymap.text = \"\"; package.text = \"\"; theme.text = \"\"; };".to_string(),
        "    nu = { env.text = \"\"; config.text = \"\"; };".to_string(),
        "  }; }; };".to_string(),
        "  zeroEval = evaluate { config.programs.yazelix = { enable = true; config.mars = {}; }; };".to_string(),
        "  twoEval = evaluate { config.programs.yazelix = { enable = true; config.mars = { text = \"\"; source = nativeSource; }; }; };".to_string(),
        "  assertionsHold = evaluation: builtins.all (item: item.assertion) evaluation.config.assertions;".to_string(),
        "in {".to_string(),
        "  paths = builtins.attrNames validEval.config.xdg.configFile;".to_string(),
        "  valid = assertionsHold validEval;".to_string(),
        "  rejectsZero = !(assertionsHold zeroEval);".to_string(),
        "  rejectsTwo = !(assertionsHold twoEval);".to_string(),
        "}".to_string(),
    ]);
    lines.join("\n")
}

fn load_home_manager_managed_config_toml(
    repo_root: &Path,
    settings_value: Option<&str>,
) -> Result<Option<String>, String> {
    let expr = build_home_manager_managed_config_toml_expr(repo_root, settings_value);
    let result = run_nix_eval(repo_root, &expr)?;
    match result {
        JsonValue::Null => Ok(None),
        JsonValue::String(raw) => Ok(Some(raw)),
        _ => Err(
            "Home Manager managed settings evaluation did not return null or a TOML string"
                .to_string(),
        ),
    }
}

fn build_home_manager_managed_config_toml_expr(
    repo_root: &Path,
    settings_value: Option<&str>,
) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"x86_64-linux\"; };".to_string(),
        "  lib = pkgs.lib.extend (_: super: { hm = { dag = { entryAfter = after: data: { inherit after data; }; }; }; });".to_string(),
        "  fakePackage = pkgs.runCommand \"yazelix\" {} ''mkdir -p $out/bin $out/libexec $out/toolbin; touch $out/bin/yzx $out/libexec/yzx_core $out/libexec/yzx_control'';".to_string(),
        format!(
            "  yazelixModule = import (builtins.toPath \"{}\") {{ defaultPackageFor = _system: fakePackage; }};",
            module_path
        ),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; };".to_string(),
        "    modules = [".to_string(),
        "      yazelixModule".to_string(),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, true));
    if let Some(value) = settings_value {
        lines.push(format!(
            "      {{ config.programs.yazelix.config.settings = {value}; }}"
        ));
    }
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        "  files = eval.config.xdg.configFile;".to_string(),
        "in if builtins.hasAttr \"yazelix/config.toml\" files then builtins.readFile files.\"yazelix/config.toml\".source else null".to_string(),
    ]);
    lines.join("\n")
}

fn config_surface_diff_base() -> String {
    if let Ok(base_ref) = env::var("GITHUB_BASE_REF") {
        let trimmed = base_ref.trim();
        if !trimmed.is_empty() {
            return format!("origin/{trimmed}");
        }
    }
    "HEAD~1".to_string()
}

fn previous_settings_contract_version(
    repo_root: &Path,
    git_ref: &str,
) -> Result<Option<u64>, String> {
    let Some(source) = load_file_from_git_ref(
        repo_root,
        git_ref,
        "rust_core/yazelix_core/src/settings_contract.rs",
    )?
    else {
        return Ok(None);
    };
    Ok(settings_contract_version_from_source(&source))
}

fn settings_contract_version_from_source(source: &str) -> Option<u64> {
    source.lines().find_map(|line| {
        if !line.contains("SETTINGS_CONTRACT_CURRENT_VERSION") {
            return None;
        }
        line.split_once('=')
            .and_then(|(_, value)| value.trim().trim_end_matches(';').trim().parse().ok())
    })
}

fn git_ref_exists(repo_root: &Path, git_ref: &str) -> Result<bool, String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root.display().to_string(),
            "rev-parse",
            "--verify",
            git_ref,
        ])
        .output()
        .map_err(|error| format!("Failed to run `git rev-parse` for {git_ref}: {error}"))?;
    Ok(output.status.success())
}

fn load_file_from_git_ref(
    repo_root: &Path,
    git_ref: &str,
    relative_path: &str,
) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root.display().to_string(),
            "show",
            &format!("{git_ref}:{relative_path}"),
        ])
        .output()
        .map_err(|error| {
            format!("Failed to run `git show` for {git_ref}:{relative_path}: {error}")
        })?;
    if !output.status.success() {
        return Ok(None);
    }
    String::from_utf8(output.stdout)
        .map(Some)
        .map_err(|error| format!("Failed to decode {git_ref}:{relative_path} as UTF-8: {error}"))
}

fn validate_nova_keybinding_registry_defaults(repo_root: &Path) -> Result<Vec<String>, String> {
    let contract = read_toml_file(&repo_root.join(MAIN_CONTRACT_RELATIVE_PATH))?;
    let mut errors = Vec::new();
    for (field_path, action_id) in [
        ("keybindings.config", "top_popup"),
        ("keybindings.agent", "open_codex_agent_right"),
        ("keybindings.git", "bottom_popup"),
        ("keybindings.menu", "menu"),
    ] {
        let contract_default = contract
            .get("fields")
            .and_then(TomlValue::as_table)
            .and_then(|fields| fields.get(field_path))
            .and_then(TomlValue::as_table)
            .and_then(|field| field.get("default"))
            .and_then(TomlValue::as_str);
        let registry_default = ZELLIJ_ACTIONS
            .iter()
            .find(|spec| spec.action.local_id == action_id)
            .map(|spec| spec.action.default_keys);
        match (contract_default, registry_default) {
            (Some(contract_default), Some([registry_default]))
                if contract_default == *registry_default => {}
            (Some(contract_default), Some(registry_defaults)) => errors.push(format!(
                "main_config_contract.toml {field_path} default `{contract_default}` does not match Classic action `{action_id}` defaults [{}]",
                registry_defaults.join(", ")
            )),
            (None, _) => errors.push(format!(
                "main_config_contract.toml is missing string default for `{field_path}`"
            )),
            (_, None) => errors.push(format!(
                "Classic action registry is missing Nova bridge action `{action_id}` for `{field_path}`"
            )),
        }
    }
    Ok(errors)
}

fn json_map_str_field<'a>(value: &'a JsonMap<String, JsonValue>, field: &str) -> &'a str {
    value
        .get(field)
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
}

fn json_map_bool_field(value: &JsonMap<String, JsonValue>, field: &str) -> bool {
    value
        .get(field)
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
}

fn format_json_string(value: &str) -> String {
    format_json_value(&JsonValue::String(value.to_string()))
}

fn validate_home_manager_desktop_entry_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let entry = load_home_manager_desktop_entry_contract(repo_root)?;
    let is_present = json_map_bool_field(&entry, "present");
    let actual_exec = json_map_str_field(&entry, "exec");
    let actual_name = json_map_str_field(&entry, "name");
    let actual_startup_wm_class = json_map_str_field(&entry, "startupWmClass");
    let expected_name = terminal_desktop_entry_name(HOME_MANAGER_DEFAULT_TERMINAL);
    let expected_exec = expected_home_manager_profile_desktop_exec(HOME_MANAGER_DEFAULT_TERMINAL);
    let expected_startup_wm_class =
        expected_home_manager_startup_wm_class(HOME_MANAGER_DEFAULT_TERMINAL);
    let mut errors = Vec::new();

    if !is_present {
        errors.push(format!(
            "Home Manager Linux {} desktop entry must be generated",
            terminal_display_name(HOME_MANAGER_DEFAULT_TERMINAL)
        ));
    }

    if actual_name != expected_name {
        errors.push(format!(
            "Home Manager {} desktop entry name mismatch: expected {}, got {}",
            terminal_display_name(HOME_MANAGER_DEFAULT_TERMINAL),
            expected_name,
            format_json_string(actual_name)
        ));
    }

    if entry.get("terminal").and_then(JsonValue::as_bool) != Some(false) {
        errors.push(
            "Home Manager desktop entry must set terminal = false so the host desktop launches yzx directly"
                .to_string(),
        );
    }

    if actual_exec != expected_exec {
        errors.push(format!(
            "Home Manager desktop entry Exec mismatch: expected {}, got {}",
            format_json_string(&expected_exec),
            format_json_string(actual_exec)
        ));
    }

    if actual_startup_wm_class != expected_startup_wm_class {
        errors.push(format!(
            "Home Manager desktop entry StartupWMClass mismatch: expected {}, got {}",
            format_json_string(&expected_startup_wm_class),
            format_json_string(actual_startup_wm_class)
        ));
    }

    validate_home_manager_darwin_without_desktop_entry_option(repo_root)?;

    Ok(errors)
}

fn expected_home_manager_profile_desktop_exec(terminal: &str) -> String {
    let launch_command = "/tmp/profile/bin/yzx desktop launch";
    match terminal {
        "mars" => format!(
            "env MARS_APP_ID={} {launch_command}",
            terminal_desktop_entry_id(terminal)
        ),
        _ => launch_command.to_string(),
    }
}

fn expected_home_manager_startup_wm_class(terminal: &str) -> String {
    match terminal {
        "mars" => terminal_desktop_entry_id(terminal),
        _ => "com.yazelix.Yazelix".to_string(),
    }
}

fn validate_home_manager_activation_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let script = load_home_manager_activation_contract(repo_root, false)?;
    let managed_script = load_home_manager_activation_contract(repo_root, true)?;
    let mut errors = validate_home_manager_activation_script(&script, false);
    errors.extend(validate_home_manager_activation_script(
        &managed_script,
        true,
    ));

    Ok(errors)
}

fn validate_home_manager_activation_script(script: &str, settings_owned: bool) -> Vec<String> {
    let mut errors = Vec::new();
    let owner_label = if settings_owned {
        "Home Manager-owned"
    } else {
        "ratconfig-owned"
    };

    if script.contains("terminal-materialization.generate") {
        errors.push(format!(
            "{owner_label} Home Manager activation must not regenerate the native Mars override"
        ));
    }

    errors
}

fn load_home_manager_activation_contract(
    repo_root: &Path,
    settings_owned: bool,
) -> Result<String, String> {
    let expr = build_home_manager_activation_expr(repo_root, settings_owned);
    let result = run_nix_eval(repo_root, &expr)?;
    result.as_str().map(str::to_string).ok_or_else(|| {
        "Home Manager activation evaluation did not return a JSON string".to_string()
    })
}

fn build_home_manager_activation_expr(repo_root: &Path, settings_owned: bool) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"x86_64-linux\"; };".to_string(),
        "  lib = pkgs.lib.extend (_: super: { hm = { dag = { entryAfter = after: data: { inherit after data; }; }; }; });".to_string(),
        "  fakePackage = pkgs.runCommand \"yazelix\" {} ''mkdir -p $out/bin $out/libexec $out/toolbin; touch $out/bin/yzx $out/libexec/yzx_core $out/libexec/yzx_control'';".to_string(),
        format!(
            "  yazelixModule = import (builtins.toPath \"{}\") {{ defaultPackageFor = _system: fakePackage; }};",
            module_path
        ),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; };".to_string(),
        "    modules = [".to_string(),
        "      yazelixModule".to_string(),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, true));
    if settings_owned {
        lines.push("      { config.programs.yazelix.config.settings = {}; }".to_string());
    }
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        "in eval.config.home.activation.yazelixGeneratedRuntimeConfigs.data".to_string(),
    ]);
    lines.join("\n")
}

fn validate_generated_state_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let fixture = setup_config_state_fixture(repo_root)?;
    let mut errors = Vec::new();

    let validation = (|| -> Result<(), String> {
        let baseline = compute_fixture_state(&fixture, &fixture.runtime_root)?;
        record_fixture_state(&fixture, &baseline)?;

        mutate_fixture_config(
            &fixture.main_config_path,
            "welcome.enabled",
            TomlValue::Boolean(false),
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

fn validate_startup_snapshot_env_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let relative_path = "rust_core/yazelix_core/src/launch_commands/enter.rs";
    let path = repo_root.join(relative_path);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    Ok(validate_startup_snapshot_env_contract_content(
        relative_path,
        &content,
    ))
}

fn validate_startup_snapshot_env_contract_content(label: &str, content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for required in [
        "write_session_config_snapshot_for_launch",
        "\"YAZELIX_SESSION_CONFIG_PATH\"",
        "\"YAZELIX_STATUS_BAR_CACHE_PATH\"",
        "command_status_with_overrides",
    ] {
        if !content.contains(required) {
            errors.push(format!(
                "{} must keep Rust-owned startup snapshot env contract token `{}`",
                label, required
            ));
        }
    }

    errors
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
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> {};".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  fakePackage = pkgs.runCommand \"yazelix\" {} ''mkdir -p $out/bin $out/libexec $out/toolbin; touch $out/bin/yzx $out/libexec/yzx_core $out/libexec/yzx_control'';".to_string(),
        format!(
            "  yazelixModule = import (builtins.toPath \"{}\") {{ defaultPackageFor = _system: fakePackage; }};",
            module_path
        ),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; };".to_string(),
        "    modules = [".to_string(),
        "      yazelixModule".to_string(),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, false));
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        "  collect = prefix: attrs: builtins.foldl' (result: name:".to_string(),
        "    let value = attrs.${name}; path = if prefix == \"\" then name else \"${prefix}.${name}\";".to_string(),
        "    in result // (if value ? declarations then { \"${path}\" = map builtins.toString value.declarations; } else collect path value)".to_string(),
        "  ) {} (builtins.attrNames attrs);".to_string(),
        "in collect \"\" eval.options.programs.yazelix".to_string(),
    ]);
    lines.join("\n")
}

fn standalone_home_manager_eval_fixture_module(
    include_desktop_entries_option: bool,
    enable_yazelix: bool,
) -> Vec<String> {
    let mut lines = vec![
        "      ({ lib, ... }: {".to_string(),
        "        options.assertions = lib.mkOption { type = lib.types.listOf lib.types.anything; default = []; };".to_string(),
        "        options.xdg.configHome = lib.mkOption { type = lib.types.str; default = \"/tmp/config\"; };".to_string(),
        "        options.xdg.dataHome = lib.mkOption { type = lib.types.str; default = \"/tmp/data\"; };".to_string(),
        "        options.xdg.dataFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.xdg.configFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
    ];
    if include_desktop_entries_option {
        lines.push("        options.xdg.desktopEntries = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string());
    }
    lines.extend([
        "        options.home.packages = lib.mkOption { type = lib.types.listOf lib.types.package; default = []; };".to_string(),
        "        options.home.activation = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.home.profileDirectory = lib.mkOption { type = lib.types.str; default = \"/tmp/profile\"; };".to_string(),
        "        options.home.sessionVariables = lib.mkOption { type = lib.types.attrsOf lib.types.str; default = {}; };".to_string(),
    ]);
    if enable_yazelix {
        lines.push("        config.programs.yazelix.enable = true;".to_string());
    }
    lines.push("      })".to_string());
    lines
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
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"x86_64-linux\"; };".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  fakePackage = pkgs.runCommand \"yazelix\" {} ''mkdir -p $out/bin $out/libexec $out/toolbin; touch $out/bin/yzx $out/libexec/yzx_core $out/libexec/yzx_control'';".to_string(),
        format!(
            "  yazelixModule = import (builtins.toPath \"{}\") {{ defaultPackageFor = _system: fakePackage; }};",
            module_path
        ),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; };".to_string(),
        "    modules = [".to_string(),
        "      yazelixModule".to_string(),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, true));
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        "  entryKey = \"com.yazelix.Yazelix.Mars\";".to_string(),
        "  entries = eval.config.xdg.desktopEntries;".to_string(),
        "  entry = if builtins.hasAttr entryKey entries then builtins.getAttr entryKey entries else {};".to_string(),
        "in {".to_string(),
        "  present = builtins.hasAttr entryKey entries;".to_string(),
        "  name = entry.name or \"\";".to_string(),
        "  exec = entry.exec or \"\";".to_string(),
        "  terminal = entry.terminal or false;".to_string(),
        "  startupWmClass = (entry.settings or {}).StartupWMClass or \"\";".to_string(),
        "}".to_string(),
    ]);
    lines.join("\n")
}

fn validate_home_manager_darwin_without_desktop_entry_option(
    repo_root: &Path,
) -> Result<(), String> {
    let expr = build_home_manager_darwin_without_desktop_entry_option_expr(repo_root);
    let _ = run_nix_eval(repo_root, &expr)?;
    Ok(())
}

fn build_home_manager_darwin_without_desktop_entry_option_expr(repo_root: &Path) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"aarch64-darwin\"; };".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  fakePackage = pkgs.runCommand \"yazelix\" {} ''mkdir -p $out/bin $out/libexec $out/toolbin; touch $out/bin/yzx $out/libexec/yzx_core $out/libexec/yzx_control'';".to_string(),
        format!(
            "  yazelixModule = import (builtins.toPath \"{}\") {{ defaultPackageFor = _system: fakePackage; }};",
            module_path
        ),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; };".to_string(),
        "    modules = [".to_string(),
        "      yazelixModule".to_string(),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(false, true));
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        "in { enabled = eval.config.programs.yazelix.enable; }".to_string(),
    ]);
    lines.join("\n")
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
    let home_root = fixture_root.join("home");
    fs::create_dir_all(&runtime_root)
        .map_err(|error| format!("Failed to create {}: {}", runtime_root.display(), error))?;
    fs::create_dir_all(&runtime_root_alt)
        .map_err(|error| format!("Failed to create {}: {}", runtime_root_alt.display(), error))?;
    fs::create_dir_all(&config_root)
        .map_err(|error| format!("Failed to create {}: {}", config_root.display(), error))?;
    fs::create_dir_all(&home_root)
        .map_err(|error| format!("Failed to create {}: {}", home_root.display(), error))?;

    for relative_path in [MAIN_TEMPLATE_RELATIVE_PATH, MAIN_CONTRACT_RELATIVE_PATH] {
        copy_fixture_file(repo_root, &runtime_root, relative_path)?;
        copy_fixture_file(repo_root, &runtime_root_alt, relative_path)?;
    }

    let main_config_path = config_root.join("config.toml");
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
    let mut table = read_config_table(config_path, "read_generated_state_fixture_config")
        .map_err(|error| error.message().to_string())?;
    set_nested_toml_value(&mut table, &split_field_path(field_path), value);
    fs::write(
        config_path,
        render_config_value(&toml_to_json(&TomlValue::Table(table)))
            .map_err(|error| error.message().to_string())?,
    )
    .map_err(|error| format!("Failed to write {}: {}", config_path.display(), error))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: maintainer
    // Regression: a required field declared under the wrong object used to pass the config-surface validator.
    #[test]
    fn template_schema_structure_rejects_wrong_required_locality_and_unknown_defaults() {
        let schema = serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "core": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["zellij_proxy"],
                    "properties": { "debug": { "type": "boolean" } }
                },
                "popups": { "type": "object" }
            }
        });
        let template =
            serde_json::json!({ "core": { "debug": false, "unknown": true }, "popups": {} });
        let mut errors = Vec::new();

        validate_schema_structure(&schema, Some(&template), "$", &mut errors);

        assert!(errors.iter().any(|error| {
            error.contains("`$.core` requires `zellij_proxy` outside that object's properties")
        }));
        assert!(errors.iter().any(|error| {
            error.contains("$.core.unknown") && error.contains("additionalProperties")
        }));
        assert!(errors.iter().any(|error| {
            error.contains("`$.popups`") && error.contains("additionalProperties")
        }));
    }

    // Test lane: maintainer
    // Regression: startup lost `YAZELIX_SESSION_CONFIG_PATH` and `YAZELIX_STATUS_BAR_CACHE_PATH` before the Zellij handoff.
    #[test]
    fn startup_snapshot_env_contract_requires_rust_handoff_tokens() {
        let missing_status_cache = r#"
fn prepare_rust_startup() {
    write_session_config_snapshot_for_launch();
    "YAZELIX_SESSION_CONFIG_PATH";
    command_status_with_overrides();
}
"#;

        let errors = validate_startup_snapshot_env_contract_content(
            "rust_core/yazelix_core/src/launch_commands/enter.rs",
            missing_status_cache,
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("YAZELIX_STATUS_BAR_CACHE_PATH"));

        let complete = format!("{missing_status_cache}\n\"YAZELIX_STATUS_BAR_CACHE_PATH\";\n");
        assert!(
            validate_startup_snapshot_env_contract_content(
                "rust_core/yazelix_core/src/launch_commands/enter.rs",
                &complete
            )
            .is_empty()
        );
    }

    // Test lane: maintainer
    // Defends: every main config contract field declares a closed runtime apply mode before config UI and doctor consume it.
    #[test]
    fn main_contract_apply_mode_validator_rejects_missing_or_unknown_modes() {
        let mut field = TomlTable::new();
        let mut errors = Vec::new();
        validate_main_contract_apply_mode("core.debug_mode", &field, &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("missing apply_mode"));

        field.insert(
            "apply_mode".to_string(),
            TomlValue::String("restart_later".to_string()),
        );
        errors.clear();
        validate_main_contract_apply_mode("core.debug_mode", &field, &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unsupported apply_mode"));

        field.insert(
            "apply_mode".to_string(),
            TomlValue::String("tab_session_restart".to_string()),
        );
        errors.clear();
        validate_main_contract_apply_mode("core.debug_mode", &field, &mut errors);
        assert!(errors.is_empty());
    }

    // Test lane: maintainer
    // Regression: Home Manager must not freeze packaged defaults when no semantic options are declared.
    #[test]
    fn home_manager_sparse_config_validator_rejects_materialized_defaults() {
        assert!(
            validate_home_manager_sparse_config_content(Path::new("config.toml"), "")
                .unwrap()
                .is_empty()
        );
        let errors = validate_home_manager_sparse_config_content(
            Path::new("config.toml"),
            "[core]\ndebug_mode = false\n",
        )
        .unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("empty sparse config.toml"));
    }

    // Test lane: maintainer
    // Defends: removing accepted settings values requires a ratconfig migration or manual blocker.
    #[test]
    fn main_contract_semantic_diff_reports_removed_enum_values() {
        let previous = toml::from_str::<TomlTable>(
            r#"
[contract]
ratconfig_contract_version = 9

[fields."zellij.widget_tray"]
kind = "string_list"
default = ["editor", "cursor"]
parser_key = "zellij_widget_tray"
parser_behavior = "direct"
validation = "enum_string_list"
allowed_values = ["editor", "cursor"]
"#,
        )
        .unwrap();
        let current = toml::from_str::<TomlTable>(
            r#"
[contract]
ratconfig_contract_version = 9

[fields."zellij.widget_tray"]
kind = "string_list"
default = ["editor"]
parser_key = "zellij_widget_tray"
parser_behavior = "direct"
validation = "enum_string_list"
allowed_values = ["editor"]
"#,
        )
        .unwrap();

        let changes = semantic_main_config_contract_changes(&previous, &current);

        assert!(changes.iter().any(|change| {
            change.contains("removed `zellij.widget_tray.allowed_values` value `cursor`")
        }));
        assert!(
            changes
                .iter()
                .any(|change| change.contains("changed `zellij.widget_tray.default`"))
        );
    }

    // Test lane: maintainer
    // Defends: adding an accepted enum value stays backward-compatible and does not force a ratconfig bump.
    #[test]
    fn main_contract_semantic_diff_allows_added_enum_values() {
        let previous = toml::from_str::<TomlTable>(
            r#"
[fields."core.welcome_style"]
kind = "string"
default = "random"
parser_key = "welcome_style"
parser_behavior = "direct"
validation = "enum"
allowed_values = ["static", "random"]
"#,
        )
        .unwrap();
        let current = toml::from_str::<TomlTable>(
            r#"
[fields."core.welcome_style"]
kind = "string"
default = "random"
parser_key = "welcome_style"
parser_behavior = "direct"
validation = "enum"
allowed_values = ["static", "random", "boids"]
"#,
        )
        .unwrap();

        assert!(semantic_main_config_contract_changes(&previous, &current).is_empty());
    }

    // Test lane: maintainer
    // Defends: source parsing for the Git diff guard follows the Rust-owned ratconfig version constant.
    #[test]
    fn settings_contract_version_parser_reads_public_const() {
        let source = "pub const SETTINGS_CONTRACT_CURRENT_VERSION: u64 = 42;";

        assert_eq!(settings_contract_version_from_source(source), Some(42));
    }

    // Test lane: maintainer
    // Regression: CI must reject a contract shrink when the ratconfig version did not advance.
    #[test]
    fn ratconfig_diff_guard_rejects_contract_change_without_version_bump() {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        fs::create_dir_all(root.join("config_metadata")).unwrap();
        fs::create_dir_all(root.join("rust_core/yazelix_core/src")).unwrap();
        let previous_contract = r#"
[contract]
ratconfig_contract_version = 9

[fields."zellij.widget_tray"]
kind = "string_list"
default = ["editor", "cursor"]
parser_key = "zellij_widget_tray"
parser_behavior = "direct"
validation = "enum_string_list"
allowed_values = ["editor", "cursor"]
"#;
        fs::write(
            root.join("config_metadata/main_config_contract.toml"),
            previous_contract,
        )
        .unwrap();
        fs::write(
            root.join("rust_core/yazelix_core/src/settings_contract.rs"),
            "const SETTINGS_CONTRACT_CURRENT_VERSION: u64 = 9;\n",
        )
        .unwrap();
        run_fixture_git(root, &["init", "--quiet"]);
        run_fixture_git(root, &["config", "user.email", "codex@example.com"]);
        run_fixture_git(root, &["config", "user.name", "Codex"]);
        run_fixture_git(root, &["add", "-A"]);
        run_fixture_git(root, &["commit", "--quiet", "-m", "baseline"]);

        let current_contract = toml::from_str::<TomlTable>(
            r#"
[contract]
ratconfig_contract_version = 9

[fields."zellij.widget_tray"]
kind = "string_list"
default = ["editor"]
parser_key = "zellij_widget_tray"
parser_behavior = "direct"
validation = "enum_string_list"
allowed_values = ["editor"]
"#,
        )
        .unwrap();

        let errors =
            validate_ratconfig_contract_diff_guard_from_base(root, &current_contract, 9, "HEAD")
                .unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("without a ratconfig migration or manual blocker"));
        assert!(errors[0].contains("removed `zellij.widget_tray.allowed_values` value `cursor`"));
    }

    fn run_fixture_git(repo_root: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(repo_root)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git command failed: {args:?}");
    }

    // Test lane: maintainer
    // Defends: the four Nova root keybindings stay aligned with the Classic bridge actions
    // they drive before the source swap.
    #[test]
    fn nova_keybinding_registry_validator_reports_missing_and_mismatched_defaults() {
        let mut contract = TomlTable::new();
        let mut fields = TomlTable::new();
        let mut keybinding = TomlTable::new();
        keybinding.insert(
            "default".to_string(),
            TomlValue::String("Alt x".to_string()),
        );
        fields.insert(
            "keybindings.config".to_string(),
            TomlValue::Table(keybinding),
        );
        contract.insert("fields".to_string(), TomlValue::Table(fields));

        let repo = tempfile::tempdir().unwrap();
        let metadata_dir = repo.path().join("config_metadata");
        fs::create_dir_all(&metadata_dir).unwrap();
        fs::write(
            metadata_dir.join("main_config_contract.toml"),
            toml::to_string(&contract).unwrap(),
        )
        .unwrap();

        let errors = validate_nova_keybinding_registry_defaults(repo.path()).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("keybindings.config default `Alt x`"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing string default for `keybindings.agent`"))
        );
    }
}
