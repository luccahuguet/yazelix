use crate::bridge::{CoreError, ErrorClass};
use crate::helix_external::{
    HelixExternalPair, is_custom_helix_binary_command, is_helix_command, non_empty_string,
};
use crate::helix_steel_plugins::parse_steel_plugin_config;
use crate::settings_surface::{read_config_table, read_sparse_config_table};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

const FLEXIBLE_NUMERIC_PATHS: &[&str] = &["core.welcome_duration_seconds"];
const MOVED_CURSOR_CONFIG_FIELDS: &[&str] = &[
    "terminal.ghostty_trail_color",
    "terminal.ghostty_trail_effect",
    "terminal.ghostty_trail_duration",
    "terminal.ghostty_mode_effect",
    "terminal.ghostty_trail_glow",
];
const REMOVED_PERSISTENT_SESSION_FIELDS: &[&str] =
    &["zellij.persistent_sessions", "zellij.session_name"];
const REMOVED_TERMINAL_SELECTION_FIELDS: &[&str] = &["terminal.terminals"];
const REMOVED_POPUP_PROGRAM_FIELDS: &[&str] = &["zellij.popup_program"];
const MOVED_CUSTOM_POPUP_FIELDS: &[&str] = &["zellij.popup_commands.btm", "zellij.keybindings.btm"];
const REMOVED_GENERIC_POPUP_ACTION_FIELDS: &[&str] = &["zellij.keybindings.popup"];
const REPLACED_HELIX_RUNTIME_FIELDS: &[&str] = &["helix.runtime_path"];

#[derive(Debug, Clone)]
pub struct NormalizeConfigRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct NormalizeConfigData {
    pub normalized_config: JsonMap<String, JsonValue>,
    pub config_file: String,
    pub diagnostic_report: ConfigDiagnosticReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiagnosticReport {
    pub config_path: String,
    pub schema_diagnostics: Vec<ConfigDiagnostic>,
    pub doctor_diagnostics: Vec<ConfigDiagnostic>,
    pub blocking_diagnostics: Vec<ConfigDiagnostic>,
    pub issue_count: usize,
    pub blocking_count: usize,
    pub fixable_count: usize,
    pub has_blocking: bool,
    pub has_fixable_config_issues: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiagnostic {
    pub category: String,
    pub path: String,
    pub status: String,
    pub blocking: bool,
    pub fix_available: bool,
    pub headline: String,
    pub detail_lines: Vec<String>,
}

#[derive(Debug, Clone)]
struct SchemaFinding {
    kind: &'static str,
    path: String,
    message: String,
}

#[derive(Debug, Clone)]
struct ContractField {
    path: String,
    parser_key: String,
    parser_behavior: String,
    validation: String,
    allowed_values: Vec<String>,
    allowed_symbols: Vec<String>,
    min: Option<f64>,
    max: Option<f64>,
}

pub fn normalize_config(
    request: &NormalizeConfigRequest,
) -> Result<NormalizeConfigData, CoreError> {
    let config = read_sparse_config_table(&request.config_path, "read_config")?;
    let (default_config, fields, diagnostic_report) = prepare_classic_config_table(
        &config,
        &request.default_config_path,
        &request.contract_path,
        &request.config_path,
    )?;
    let config_file = request.config_path.to_string_lossy().to_string();

    let mut normalized_config = JsonMap::new();
    for field in fields.values() {
        let normalized = normalize_field(field, &config, &default_config)?;
        normalized_config.insert(field.parser_key.clone(), normalized);
    }
    normalized_config.insert(
        "config_file".to_string(),
        JsonValue::String(config_file.clone()),
    );

    Ok(NormalizeConfigData {
        normalized_config,
        config_file,
        diagnostic_report,
    })
}

fn prepare_classic_config_table(
    config: &toml::Table,
    default_config_path: &Path,
    contract_path: &Path,
    config_path: &Path,
) -> Result<
    (
        toml::Table,
        BTreeMap<String, ContractField>,
        ConfigDiagnosticReport,
    ),
    CoreError,
> {
    let default_config = read_config_table(default_config_path, "read_default_config")?;
    let contract = read_toml_table(contract_path, "read_config_contract")?;
    let fields = load_contract_fields(&contract)?;
    let diagnostic_report = build_diagnostic_report(config, &default_config, &fields, config_path)?;
    if diagnostic_report.has_blocking {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "unsupported_config",
            format!(
                "Yazelix found stale or unsupported config entries in {}.",
                diagnostic_report.config_path
            ),
            "Update the reported config fields manually, then retry. Use `yzx reset config` only as a blunt fallback.",
            serde_json::to_value(&diagnostic_report).unwrap_or_else(|_| json!({})),
        ));
    }
    Ok((default_config, fields, diagnostic_report))
}

pub(crate) fn validate_classic_config_table(
    config: &toml::Table,
    default_config_path: &Path,
    contract_path: &Path,
    config_path: &Path,
) -> Result<(), CoreError> {
    let (defaults, fields, _) =
        prepare_classic_config_table(config, default_config_path, contract_path, config_path)?;
    for field in fields.values() {
        normalize_field(field, config, &defaults)?;
    }
    Ok(())
}

fn read_toml_table(path: &Path, code: &str) -> Result<toml::Table, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            code,
            "Could not read Yazelix config input",
            "Ensure the explicit config, default, and contract paths exist and are readable.",
            path.to_string_lossy(),
            source,
        )
    })?;
    toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_toml",
            "Could not parse Yazelix TOML input",
            "Fix the TOML syntax in the reported file and retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

fn load_contract_fields(
    contract: &toml::Table,
) -> Result<BTreeMap<String, ContractField>, CoreError> {
    let fields_table = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "missing_contract_fields",
                "The Yazelix config contract is missing its fields table",
                "Reinstall Yazelix so the runtime includes the current config contract.",
                json!({}),
            )
        })?;

    let mut fields = BTreeMap::new();
    for (path, raw_field) in fields_table {
        let field_table = raw_field.as_table().ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_contract_field",
                format!("Config contract field {path} must be a TOML table"),
                "Reinstall Yazelix so the runtime includes a valid config contract.",
                json!({ "field": path }),
            )
        })?;

        let parser_key = field_table
            .get("parser_key")
            .and_then(TomlValue::as_str)
            .unwrap_or(path)
            .to_string();
        let parser_behavior = field_table
            .get("parser_behavior")
            .and_then(TomlValue::as_str)
            .unwrap_or("direct")
            .to_string();
        let validation = field_table
            .get("validation")
            .and_then(TomlValue::as_str)
            .unwrap_or("")
            .to_string();
        let allowed_values = string_array(field_table.get("allowed_values"));
        let allowed_symbols = string_array(field_table.get("allowed_symbols"));
        let min = field_table.get("min").and_then(toml_number_as_f64);
        let max = field_table.get("max").and_then(toml_number_as_f64);

        fields.insert(
            path.clone(),
            ContractField {
                path: path.clone(),
                parser_key,
                parser_behavior,
                validation,
                allowed_values,
                allowed_symbols,
                min,
                max,
            },
        );
    }

    Ok(fields)
}

fn string_array(value: Option<&TomlValue>) -> Vec<String> {
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

fn toml_number_as_f64(value: &TomlValue) -> Option<f64> {
    value
        .as_float()
        .or_else(|| value.as_integer().map(|integer| integer as f64))
}

fn build_diagnostic_report(
    user_config: &toml::Table,
    default_config: &toml::Table,
    fields: &BTreeMap<String, ContractField>,
    config_path: &Path,
) -> Result<ConfigDiagnosticReport, CoreError> {
    let findings = {
        let mut findings = compare_configs(
            &TomlValue::Table(default_config.clone()),
            &TomlValue::Table(user_config.clone()),
            &[],
            fields,
        );
        findings.extend(validate_enum_values(user_config, fields));
        findings.extend(validate_helix_external_pair(user_config));
        findings.extend(validate_helix_steel_plugins(user_config));
        findings
    };

    let schema_diagnostics = findings
        .into_iter()
        .map(make_schema_diagnostic)
        .collect::<Vec<_>>();
    let doctor_diagnostics = schema_diagnostics.clone();
    let fixable_count = doctor_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.fix_available)
        .count();
    let blocking_diagnostics = doctor_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.blocking)
        .cloned()
        .collect::<Vec<_>>();

    Ok(ConfigDiagnosticReport {
        config_path: config_path.to_string_lossy().to_string(),
        issue_count: doctor_diagnostics.len(),
        blocking_count: blocking_diagnostics.len(),
        fixable_count,
        has_blocking: !blocking_diagnostics.is_empty(),
        has_fixable_config_issues: fixable_count > 0,
        schema_diagnostics,
        doctor_diagnostics,
        blocking_diagnostics,
    })
}

fn classify_value(value: &TomlValue) -> &'static str {
    match value {
        TomlValue::Table(_) => "record",
        TomlValue::Array(_) => "list",
        TomlValue::String(_) => "string",
        TomlValue::Boolean(_) => "bool",
        TomlValue::Integer(_) => "int",
        TomlValue::Float(_) => "float",
        TomlValue::Datetime(_) => "datetime",
    }
}

fn compare_configs(
    default: &TomlValue,
    user: &TomlValue,
    path: &[&str],
    fields: &BTreeMap<String, ContractField>,
) -> Vec<SchemaFinding> {
    let default_kind = classify_value(default);
    let user_kind = classify_value(user);
    let formatted_path = format_config_path(path);

    if let TomlValue::Table(default_table) = default {
        let TomlValue::Table(user_table) = user else {
            return vec![SchemaFinding {
                kind: "type_mismatch",
                path: formatted_path.clone(),
                message: format!(
                    "Type mismatch at {formatted_path}: expected {default_kind}, found {user_kind}"
                ),
            }];
        };

        let mut findings = Vec::new();
        for key in user_table.keys() {
            if path.is_empty() && key == "cursors" {
                continue;
            }
            if !default_table.contains_key(key) {
                let mut finding_path = path.to_vec();
                finding_path.push(key);
                let formatted = format_config_path(&finding_path);
                if contract_allows_config_path(&formatted, fields) {
                    continue;
                }
                findings.push(SchemaFinding {
                    kind: "unknown_field",
                    path: formatted.clone(),
                    message: format!("Unknown config field: {formatted}"),
                });
            }
        }

        for (key, default_value) in default_table {
            if let Some(user_value) = user_table.get(key) {
                let mut nested_path = path.to_vec();
                nested_path.push(key);
                findings.extend(compare_configs(
                    default_value,
                    user_value,
                    &nested_path,
                    fields,
                ));
            }
        }
        return findings;
    }

    if default_kind != user_kind {
        let flexible_numeric = FLEXIBLE_NUMERIC_PATHS.contains(&formatted_path.as_str())
            && matches!(default_kind, "int" | "float")
            && matches!(user_kind, "int" | "float");
        if !flexible_numeric {
            return vec![SchemaFinding {
                kind: "type_mismatch",
                path: formatted_path.clone(),
                message: format!(
                    "Type mismatch at {formatted_path}: expected {default_kind}, found {user_kind}"
                ),
            }];
        }
    }

    Vec::new()
}

fn contract_allows_config_path(path: &str, fields: &BTreeMap<String, ContractField>) -> bool {
    fields
        .keys()
        .any(|field_path| field_path == path || field_path.starts_with(&format!("{path}.")))
}

fn validate_enum_values(
    user_config: &toml::Table,
    fields: &BTreeMap<String, ContractField>,
) -> Vec<SchemaFinding> {
    let mut findings = Vec::new();
    for field in fields.values() {
        if field.validation != "enum" && field.validation != "enum_string_list" {
            continue;
        }
        let path = field.path.split('.').collect::<Vec<_>>();
        let Some(value) = get_nested_table_value(user_config, &path) else {
            continue;
        };

        if field.validation == "enum_string_list" {
            if let TomlValue::Array(values) = value {
                for value in values {
                    let rendered = toml_value_to_lossy_string(value);
                    if !field.allowed_values.contains(&rendered) {
                        findings.push(invalid_enum_finding(
                            &field.path,
                            &field.allowed_values,
                            &rendered,
                        ));
                    }
                }
            } else {
                let rendered = toml_value_to_lossy_string(value);
                if !field.allowed_values.contains(&rendered) {
                    findings.push(invalid_enum_finding(
                        &field.path,
                        &field.allowed_values,
                        &rendered,
                    ));
                }
            }
        } else {
            let rendered = toml_value_to_lossy_string(value);
            if !field.allowed_values.contains(&rendered) {
                findings.push(invalid_enum_finding(
                    &field.path,
                    &field.allowed_values,
                    &rendered,
                ));
            }
        }
    }
    findings
}

fn validate_helix_external_pair(user_config: &toml::Table) -> Vec<SchemaFinding> {
    let mut findings = Vec::new();
    let editor_command = get_nested_table_value(user_config, &["editor", "command"])
        .and_then(TomlValue::as_str)
        .and_then(non_empty_string);
    let external = get_nested_table_value(user_config, &["helix", "external"]);

    if let Some(external) = external {
        let Some(table) = external.as_table() else {
            findings.push(SchemaFinding {
                kind: "type_mismatch",
                path: "helix.external".to_string(),
                message: "Type mismatch at helix.external: expected record, found non-record"
                    .to_string(),
            });
            return findings;
        };

        for key in table.keys() {
            if key != "binary" && key != "runtime_path" {
                findings.push(SchemaFinding {
                    kind: "unknown_field",
                    path: format!("helix.external.{key}"),
                    message: format!("Unknown config field: helix.external.{key}"),
                });
            }
        }

        let binary = table
            .get("binary")
            .and_then(TomlValue::as_str)
            .and_then(non_empty_string);
        let runtime_path = table
            .get("runtime_path")
            .and_then(TomlValue::as_str)
            .and_then(non_empty_string);

        match (binary.as_deref(), runtime_path.as_deref()) {
            (Some(_), Some(_)) => {}
            (Some(_), None) => findings.push(helix_external_pair_finding(
                "helix.external.runtime_path",
                "helix.external.binary is set, so helix.external.runtime_path is required.",
            )),
            (None, Some(_)) => findings.push(helix_external_pair_finding(
                "helix.external.binary",
                "helix.external.runtime_path is set, so helix.external.binary is required.",
            )),
            (None, None) => findings.push(helix_external_pair_finding(
                "helix.external",
                "helix.external must be null or contain both binary and runtime_path.",
            )),
        }

        if binary.is_some()
            && runtime_path.is_some()
            && editor_command
                .as_deref()
                .is_some_and(|editor| !is_helix_command(editor))
        {
            findings.push(helix_external_pair_finding(
                "editor.command",
                "helix.external is set but editor.command points at a non-Helix editor.",
            ));
        }
    } else if editor_command
        .as_deref()
        .is_some_and(is_custom_helix_binary_command)
    {
        findings.push(helix_external_pair_finding(
            "editor.command",
            "A custom Helix binary requires helix.external with both binary and runtime_path.",
        ));
    }

    findings
}

fn helix_external_pair_finding(path: &str, message: &str) -> SchemaFinding {
    SchemaFinding {
        kind: "invalid_helix_external_pair",
        path: path.to_string(),
        message: message.to_string(),
    }
}

fn validate_helix_steel_plugins(user_config: &toml::Table) -> Vec<SchemaFinding> {
    let Some(value) = get_nested_table_value(user_config, &["helix", "steel_plugins"]) else {
        return Vec::new();
    };
    match parse_steel_plugin_config(Some(&toml_to_json(value))) {
        Ok(_) => Vec::new(),
        Err(error) => vec![SchemaFinding {
            kind: "invalid_helix_steel_plugins",
            path: error.path,
            message: error.message,
        }],
    }
}

fn invalid_enum_finding(path: &str, allowed_values: &[String], value: &str) -> SchemaFinding {
    SchemaFinding {
        kind: "invalid_enum",
        path: path.to_string(),
        message: format!(
            "Invalid value for {path}: {value} (allowed: [{}])",
            allowed_values.join(", ")
        ),
    }
}

fn make_schema_diagnostic(finding: SchemaFinding) -> ConfigDiagnostic {
    let blocking = true;
    let fix_available = false;
    let mut diagnostic = ConfigDiagnostic {
        category: "schema".to_string(),
        path: finding.path.clone(),
        status: finding.kind.to_string(),
        blocking,
        fix_available,
        headline: String::new(),
        detail_lines: Vec::new(),
    };

    match finding.kind {
        "unknown_field" => {
            if MOVED_CURSOR_CONFIG_FIELDS.contains(&finding.path.as_str()) {
                diagnostic.headline = format!("Moved cursor config field at {}", finding.path);
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Move this cursor setting into ~/.config/yazelix/cursors.toml."
                        .to_string(),
                    "Next: Remove the old terminal.ghostty_* field from ~/.config/yazelix/config.toml."
                        .to_string(),
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                        .to_string(),
                ];
            } else if REPLACED_HELIX_RUNTIME_FIELDS.contains(&finding.path.as_str()) {
                diagnostic.headline =
                    format!("Replaced Helix runtime config field at {}", finding.path);
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Next: Replace helix.runtime_path with helix.external = { binary = \"/path/to/hx\", runtime_path = \"/path/to/helix/runtime\" }.".to_string(),
                    "Next: Leave helix.external as null to use Yazelix's bundled Helix."
                        .to_string(),
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                        .to_string(),
                ];
            } else if REMOVED_PERSISTENT_SESSION_FIELDS.contains(&finding.path.as_str()) {
                diagnostic.headline = format!(
                    "Removed persistent-session config field at {}",
                    finding.path
                );
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Next: Remove zellij.persistent_sessions and zellij.session_name from ~/.config/yazelix/config.toml.".to_string(),
                    "Next: Yazelix now starts independent windows; use raw Zellij session management outside Yazelix if you need it.".to_string(),
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                        .to_string(),
                ];
            } else if REMOVED_TERMINAL_SELECTION_FIELDS.contains(&finding.path.as_str()) {
                diagnostic.headline = format!(
                    "Removed terminal selection config field at {}",
                    finding.path
                );
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Next: Remove terminal.terminals from ~/.config/yazelix/config.toml."
                        .to_string(),
                    "Next: Yazelix packages Mars; configure other terminal emulators to start Yazelix with `yzx enter`.".to_string(),
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                        .to_string(),
                ];
            } else if REMOVED_POPUP_PROGRAM_FIELDS.contains(&finding.path.as_str()) {
                diagnostic.headline =
                    format!("Removed popup program config field at {}", finding.path);
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Next: Remove zellij.popup_program from ~/.config/yazelix/config.toml."
                        .to_string(),
                    "Next: Add persistent popup commands through zellij.custom_popups instead."
                        .to_string(),
                    "Next: Use `yzx popup <program> [args...]` for one-off transient popups."
                        .to_string(),
                ];
            } else if MOVED_CUSTOM_POPUP_FIELDS.contains(&finding.path.as_str()) {
                diagnostic.headline =
                    format!("Moved custom popup config field at {}", finding.path);
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Next: Move the Zenith popup to zellij.custom_popups with { \"id\": \"zenith\", \"command\": [\"zenith\"], \"keybindings\": [\"Alt Shift I\"], \"keep_alive\": true }.".to_string(),
                    "Next: Keep zellij.popup_commands limited to bottom_popup, top_popup, and menu.".to_string(),
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                        .to_string(),
                ];
            } else if REMOVED_GENERIC_POPUP_ACTION_FIELDS.contains(&finding.path.as_str()) {
                diagnostic.headline =
                    format!("Removed generic popup keybinding at {}", finding.path);
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Next: Remove zellij.keybindings.popup from ~/.config/yazelix/config.toml."
                        .to_string(),
                    "Next: Add a named persistent popup through zellij.custom_popups, or run `yzx popup <program> [args...]` for one-off popups.".to_string(),
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                        .to_string(),
                ];
            } else {
                diagnostic.headline = format!("Unknown config field at {}", finding.path);
                diagnostic.detail_lines = vec![
                    finding.message,
                    "Next: Remove or rename this field manually.".to_string(),
                    "Next: Run `yzx doctor --verbose` to review the full config report."
                        .to_string(),
                    "Next: Use `yzx reset config` only as a blunt fallback.".to_string(),
                ];
            }
        }
        "type_mismatch" => {
            diagnostic.headline = format!("Wrong config type at {}", finding.path);
            diagnostic.detail_lines = vec![
                finding.message,
                "Next: Update the value to the expected type manually.".to_string(),
                "Next: Run `yzx doctor --verbose` to review the full config report.".to_string(),
            ];
        }
        "invalid_enum" => {
            diagnostic.headline = format!("Unsupported config value at {}", finding.path);
            diagnostic.detail_lines = vec![
                finding.message,
                "Next: Replace this value with one of the supported options.".to_string(),
                "Next: Run `yzx doctor --verbose` to review the full config report.".to_string(),
                "Next: Use `yzx reset config` only as a blunt fallback.".to_string(),
            ];
        }
        "invalid_helix_external_pair" => {
            diagnostic.headline = format!("Invalid Helix external pair at {}", finding.path);
            diagnostic.detail_lines = vec![
                finding.message,
                "Next: Set helix.external to null, or provide both binary and runtime_path."
                    .to_string(),
                "Next: Do not set a custom Helix binary through editor.command alone.".to_string(),
            ];
        }
        "invalid_helix_steel_plugins" => {
            diagnostic.headline =
                format!("Invalid Helix Steel plugin manifest at {}", finding.path);
            diagnostic.detail_lines = vec![
                finding.message,
                "Next: Keep helix.steel_plugins as an object with enabled bundled ids and extra plugin manifests.".to_string(),
                "Next: Source paths must be relative .scm files below ~/.config/yazelix/helix/steel_plugins.".to_string(),
            ];
        }
        _ => {
            diagnostic.headline = format!("Config issue at {}", finding.path);
            diagnostic.detail_lines = vec![finding.message];
        }
    }

    diagnostic
}

fn normalize_field(
    field: &ContractField,
    raw_config: &toml::Table,
    default_config: &toml::Table,
) -> Result<JsonValue, CoreError> {
    if field.parser_behavior == "helix_external_pair" {
        return normalize_helix_external_field(field, raw_config, default_config);
    }

    let path = field.path.split('.').collect::<Vec<_>>();
    let value = get_nested_table_value(raw_config, &path)
        .or_else(|| get_nested_table_value(default_config, &path))
        .ok_or_else(|| missing_packaged_default_error(&field.path))?;

    match field.parser_behavior.as_str() {
        "compact_badge_text" => Ok(JsonValue::String(compact_badge_text(value))),
        "empty_string_to_null" => {
            let value = toml_value_to_lossy_string(value);
            if value.is_empty() {
                Ok(JsonValue::Null)
            } else {
                Ok(JsonValue::String(value))
            }
        }
        "bool_to_string" => {
            let value = value.as_bool().ok_or_else(|| {
                invalid_value_error(&field.path, &toml_value_to_lossy_string(value), "a boolean")
            })?;
            Ok(JsonValue::String(
                if value { "true" } else { "false" }.to_string(),
            ))
        }
        _ => normalize_direct_field(field, value),
    }
}

fn normalize_helix_external_field(
    field: &ContractField,
    raw_config: &toml::Table,
    default_config: &toml::Table,
) -> Result<JsonValue, CoreError> {
    let path = field.path.split('.').collect::<Vec<_>>();
    let value = get_nested_table_value(raw_config, &path)
        .or_else(|| get_nested_table_value(default_config, &path));
    let Some(value) = value else {
        return Ok(JsonValue::Null);
    };
    let Some(table) = value.as_table() else {
        return Err(invalid_value_error(
            &field.path,
            &toml_value_to_lossy_string(value),
            "null or an object with binary and runtime_path",
        ));
    };
    let pair = HelixExternalPair::normalized(
        table
            .get("binary")
            .and_then(TomlValue::as_str)
            .unwrap_or(""),
        table
            .get("runtime_path")
            .and_then(TomlValue::as_str)
            .unwrap_or(""),
    )
    .ok_or_else(|| {
        invalid_value_error(
            &field.path,
            &toml_value_to_lossy_string(value),
            "null or an object with both binary and runtime_path",
        )
    })?;
    Ok(pair.as_json())
}

fn missing_packaged_default_error(path: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_packaged_config_default",
        format!("The packaged Yazelix config is missing the required default for {path}."),
        "Reinstall Yazelix so config_default.toml matches the current config contract.",
        json!({ "path": path }),
    )
}

fn normalize_direct_field(
    field: &ContractField,
    value: &TomlValue,
) -> Result<JsonValue, CoreError> {
    match field.validation.as_str() {
        "enum" => {
            let normalized = toml_value_to_lossy_string(value).to_lowercase();
            if !field.allowed_values.contains(&normalized) {
                return Err(invalid_value_error(
                    &field.path,
                    &normalized,
                    &format!("one of: {}", field.allowed_values.join(", ")),
                ));
            }
            Ok(JsonValue::String(normalized))
        }
        "enum_string_list" => {
            let Some(values) = value.as_array() else {
                return Err(invalid_value_error(
                    &field.path,
                    &toml_value_to_lossy_string(value),
                    &format!(
                        "a list with values from: {}",
                        field.allowed_values.join(", ")
                    ),
                ));
            };
            let mut normalized = Vec::new();
            for value in values {
                let rendered = toml_value_to_lossy_string(value);
                if !field.allowed_values.contains(&rendered) {
                    return Err(invalid_value_error(
                        &field.path,
                        &rendered,
                        &format!(
                            "a list with values from: {}",
                            field.allowed_values.join(", ")
                        ),
                    ));
                }
                normalized.push(JsonValue::String(rendered));
            }
            Ok(JsonValue::Array(normalized))
        }
        "float_range" => {
            let Some(parsed) = toml_number_as_f64(value) else {
                return Err(invalid_value_error(
                    &field.path,
                    &toml_value_to_lossy_string(value),
                    &range_expectation(field),
                ));
            };
            validate_range(field, parsed, value)?;
            json_number(parsed).map(JsonValue::Number)
        }
        "int_range" => {
            let parsed = match value {
                TomlValue::Integer(integer) => *integer,
                TomlValue::String(raw) => raw.trim().parse::<i64>().map_err(|_| {
                    invalid_value_error(&field.path, raw, &range_expectation(field))
                })?,
                _ => {
                    return Err(invalid_value_error(
                        &field.path,
                        &toml_value_to_lossy_string(value),
                        &range_expectation(field),
                    ));
                }
            };
            validate_range(field, parsed as f64, value)?;
            Ok(JsonValue::Number(JsonNumber::from(parsed)))
        }
        "symbol_or_positive_int_string" => {
            let normalized = toml_value_to_lossy_string(value).to_lowercase();
            if field.allowed_symbols.contains(&normalized) {
                return Ok(JsonValue::String(normalized));
            }
            let parsed = normalized.parse::<i64>().map_err(|_| {
                invalid_value_error(
                    &field.path,
                    &normalized,
                    &format!(
                        "one of: {}, or a positive integer",
                        field.allowed_symbols.join(", ")
                    ),
                )
            })?;
            if parsed < 1 {
                return Err(invalid_value_error(
                    &field.path,
                    &normalized,
                    "a positive integer",
                ));
            }
            Ok(JsonValue::String(normalized))
        }
        _ => Ok(toml_to_json(value)),
    }
}

fn validate_range(
    field: &ContractField,
    parsed: f64,
    original: &TomlValue,
) -> Result<(), CoreError> {
    let min = field.min.unwrap_or(f64::MIN);
    let max = field.max.unwrap_or(f64::MAX);
    if parsed < min || parsed > max {
        return Err(invalid_value_error(
            &field.path,
            &toml_value_to_lossy_string(original),
            &range_expectation(field),
        ));
    }
    Ok(())
}

fn range_expectation(field: &ContractField) -> String {
    match (field.min, field.max) {
        (Some(min), Some(max)) if min.fract() == 0.0 && max.fract() == 0.0 => {
            format!("an integer from {} to {}", min as i64, max as i64)
        }
        (Some(min), Some(max)) => format!("a number from {min} to {max}"),
        _ => "a supported number".to_string(),
    }
}

fn invalid_value_error(field_path: &str, actual_value: &str, expectation: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        "invalid_config_value",
        format!("Invalid {field_path} value '{actual_value}'. Expected {expectation}."),
        "Update config.toml with a supported value, or run `yzx reset config` to remove the explicit overrides.",
        json!({
            "field": field_path,
            "actual": actual_value,
            "expectation": expectation,
        }),
    )
}

fn compact_badge_text(value: &TomlValue) -> String {
    let mut compact = toml_value_to_lossy_string(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    compact.retain(|character| !matches!(character, '[' | ']' | '{' | '}' | '"' | '\\'));
    compact.chars().take(8).collect()
}

fn get_nested_table_value<'a>(table: &'a toml::Table, path: &[&str]) -> Option<&'a TomlValue> {
    let (first, rest) = path.split_first()?;
    let mut current = table.get(*first)?;
    for segment in rest {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

fn format_config_path(path: &[&str]) -> String {
    if path.is_empty() {
        "<root>".to_string()
    } else {
        path.join(".")
    }
}

fn toml_value_to_lossy_string(value: &TomlValue) -> String {
    match value {
        TomlValue::String(value) => value.clone(),
        TomlValue::Integer(value) => value.to_string(),
        TomlValue::Float(value) => value.to_string(),
        TomlValue::Boolean(value) => value.to_string(),
        TomlValue::Datetime(value) => value.to_string(),
        TomlValue::Array(_) | TomlValue::Table(_) => value.to_string(),
    }
}

fn toml_to_json(value: &TomlValue) -> JsonValue {
    match value {
        TomlValue::String(value) => JsonValue::String(value.clone()),
        TomlValue::Integer(value) => JsonValue::Number(JsonNumber::from(*value)),
        TomlValue::Float(value) => json_number(*value)
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

fn json_number(value: f64) -> Result<JsonNumber, CoreError> {
    JsonNumber::from_f64(value).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Config,
            "invalid_number",
            "Yazelix config number cannot be represented as JSON",
            "Update the reported config field with a finite supported number.",
            json!({ "value": value.to_string() }),
        )
    })
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::tempdir;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    fn request_for(config_path: PathBuf) -> NormalizeConfigRequest {
        let repo = repo_root();
        NormalizeConfigRequest {
            config_path,
            default_config_path: repo.join("config_default.toml"),
            contract_path: repo.join("config_metadata/main_config_contract.toml"),
        }
    }

    fn write_user_config(contents: &str) -> PathBuf {
        let dir = tempdir().expect("tempdir").keep();
        let path = dir.join("yazelix.toml");
        fs::write(&path, contents).expect("write config");
        path
    }

    fn default_main_config() -> JsonValue {
        crate::settings_surface::read_config_value(&repo_root().join("config_default.toml"))
            .expect("default settings")
    }

    fn write_main_config(value: &JsonValue) -> PathBuf {
        let dir = tempdir().expect("tempdir").keep();
        let path = dir.join("config.toml");
        fs::write(
            &path,
            crate::settings_surface::render_config_value(value).expect("settings TOML"),
        )
        .expect("write settings");
        path
    }

    fn blocking_diagnostic_for<'a>(
        details: &'a JsonValue,
        path: &str,
    ) -> &'a serde_json::Map<String, JsonValue> {
        details["blocking_diagnostics"]
            .as_array()
            .expect("blocking diagnostics")
            .iter()
            .find(|diagnostic| diagnostic["path"] == path)
            .and_then(JsonValue::as_object)
            .expect("diagnostic for path")
    }

    // Defends: config normalization keeps parser-owned defaults and value transforms stable.
    #[test]
    fn normalizes_default_config_with_parser_keys_and_transforms() {
        let repo = repo_root();
        let data = normalize_config(&request_for(repo.join("config_default.toml"))).unwrap();
        let config = data.normalized_config;

        assert_eq!(config.get("default_shell").unwrap(), "nu");
        assert_eq!(config.get("appearance_mode").unwrap(), "dark");
        assert_eq!(config.get("helix_external").unwrap(), &JsonValue::Null);
        assert_eq!(config.get("game_of_life_cell_style").unwrap(), "full_block");
        assert_eq!(config.get("welcome_duration_seconds").unwrap(), 4.0);

        let contract = read_toml_table(
            &repo.join("config_metadata/main_config_contract.toml"),
            "test",
        )
        .unwrap();
        let fields = load_contract_fields(&contract).unwrap();
        assert_eq!(config.len(), fields.len() + 1);

        let partial_config = write_user_config("[appearance]\nmode = \"light\"\n");
        let partial = normalize_config(&request_for(partial_config)).unwrap();
        assert_eq!(
            partial.normalized_config.get("appearance_mode").unwrap(),
            "light"
        );
    }

    // Defends: an absent user config inherits every packaged semantic default without creating a file.
    #[test]
    fn absent_config_normalizes_to_packaged_defaults_without_writing() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");

        let inherited = normalize_config(&request_for(path.clone())).unwrap();
        let packaged =
            normalize_config(&request_for(repo_root().join("config_default.toml"))).unwrap();
        let mut inherited_values = inherited.normalized_config;
        let mut packaged_values = packaged.normalized_config;
        inherited_values.remove("config_file");
        packaged_values.remove("config_file");

        assert_eq!(inherited_values, packaged_values);
        assert_eq!(inherited.diagnostic_report.issue_count, 0);
        assert!(!path.exists());
    }

    // Defends: packaged defaults, rather than duplicated contract metadata, own inherited values.
    #[test]
    fn absent_field_tracks_the_packaged_default_value() {
        let dir = tempdir().expect("tempdir");
        let config_path = dir.path().join("config.toml");
        let default_config_path = dir.path().join("config_default.toml");
        let default_config = fs::read_to_string(repo_root().join("config_default.toml"))
            .expect("read packaged defaults")
            .replacen("mode = \"dark\"", "mode = \"light\"", 1);
        fs::write(&default_config_path, default_config).expect("write changed packaged defaults");

        let data = normalize_config(&NormalizeConfigRequest {
            config_path: config_path.clone(),
            default_config_path,
            contract_path: repo_root().join("config_metadata/main_config_contract.toml"),
        })
        .unwrap();

        assert_eq!(
            data.normalized_config.get("appearance_mode").unwrap(),
            "light"
        );
        assert!(!config_path.exists());
    }

    // Regression: a damaged packaged default must fail instead of inventing an empty value.
    #[test]
    fn missing_required_packaged_default_fails_fast() {
        let dir = tempdir().expect("tempdir");
        let default_config_path = dir.path().join("config_default.toml");
        let default_config = fs::read_to_string(repo_root().join("config_default.toml"))
            .expect("read packaged defaults")
            .replacen("mode = \"dark\"\n", "", 1);
        fs::write(&default_config_path, default_config).expect("write damaged packaged defaults");

        let error = normalize_config(&NormalizeConfigRequest {
            config_path: dir.path().join("config.toml"),
            default_config_path,
            contract_path: repo_root().join("config_metadata/main_config_contract.toml"),
        })
        .unwrap_err();

        assert_eq!(error.code(), "missing_packaged_config_default");
        assert_eq!(error.details()["path"], "appearance.mode");
    }

    // Defends: host-owned xonsh is a valid default-shell enum without being bundled by Yazelix.
    #[test]
    fn accepts_xonsh_as_default_shell_enum_value() {
        let path = write_user_config("[shell]\ndefault_shell = \"xonsh\"\n");
        let data = normalize_config(&request_for(path)).unwrap();

        assert_eq!(
            data.normalized_config.get("default_shell").unwrap(),
            "xonsh"
        );
    }

    // Defends: the shipped main settings template is a complete strict config.toml surface.
    #[test]
    fn strict_default_config_toml_has_no_diagnostics() {
        let repo = repo_root();
        let data = normalize_config(&request_for(repo.join("config_default.toml"))).unwrap();

        assert_eq!(data.diagnostic_report.issue_count, 0);
        assert_eq!(data.diagnostic_report.blocking_count, 0);
        assert_eq!(
            data.normalized_config.get("helix_external").unwrap(),
            &JsonValue::Null
        );
    }

    // Defends: custom Helix forks must be configured as one binary/runtime pair, not as a bare runtime path or bare editor binary.
    #[test]
    fn rejects_incomplete_helix_external_pair_and_old_runtime_field() {
        let mut binary_only_config = default_main_config();
        binary_only_config["helix"]["external"] = json!({ "binary": "/tmp/hx" });
        let binary_only = write_main_config(&binary_only_config);
        let error = normalize_config(&request_for(binary_only)).unwrap_err();
        assert_eq!(error.code(), "unsupported_config");
        let details = error.details();
        let diagnostic = blocking_diagnostic_for(&details, "helix.external.runtime_path");
        assert_eq!(diagnostic["status"], "invalid_helix_external_pair");

        let mut old_runtime_config = default_main_config();
        old_runtime_config["helix"]
            .as_object_mut()
            .unwrap()
            .remove("external");
        old_runtime_config["helix"]["runtime_path"] = json!("/tmp/runtime");
        let old_runtime = write_main_config(&old_runtime_config);
        let error = normalize_config(&request_for(old_runtime)).unwrap_err();
        assert_eq!(error.code(), "unsupported_config");
        let details = error.details();
        let diagnostic = blocking_diagnostic_for(&details, "helix.runtime_path");
        assert!(
            diagnostic["detail_lines"]
                .as_array()
                .unwrap()
                .iter()
                .any(|line| line.as_str().unwrap().contains("helix.external = { binary"))
        );

        let mut conflicting_editor_config = default_main_config();
        conflicting_editor_config["editor"]["command"] = json!("nvim");
        conflicting_editor_config["helix"]["external"] = json!({
            "binary": "/tmp/hx",
            "runtime_path": "/tmp/runtime"
        });
        let conflicting_editor = write_main_config(&conflicting_editor_config);
        let error = normalize_config(&request_for(conflicting_editor)).unwrap_err();
        assert_eq!(error.code(), "unsupported_config");
        let details = error.details();
        let diagnostic = blocking_diagnostic_for(&details, "editor.command");
        assert_eq!(diagnostic["status"], "invalid_helix_external_pair");
    }

    // Defends: a complete external Helix pair normalizes as one object for runtime env consumers.
    #[test]
    fn normalizes_helix_external_pair() {
        let mut config = default_main_config();
        config["helix"]["external"] = json!({
            "binary": "/tmp/hx",
            "runtime_path": "/tmp/runtime"
        });
        let path = write_main_config(&config);
        let data = normalize_config(&request_for(path)).unwrap();
        assert_eq!(
            data.normalized_config.get("helix_external").unwrap(),
            &json!({
                "binary": "/tmp/hx",
                "runtime_path": "/tmp/runtime",
            })
        );
    }

    // Defends: custom Helix Steel plugin manifests reject unsafe source paths before materialization.
    #[test]
    fn rejects_invalid_helix_steel_plugin_manifest_shape() {
        let mut config = default_main_config();
        config["helix"]["steel_plugins"] = json!({
            "enabled": ["recentf"],
            "extra": [{
                "id": "bad_plugin",
                "source": "../bad.scm",
                "public_commands": ["bad-open"]
            }]
        });
        let path = write_main_config(&config);
        let error = normalize_config(&request_for(path)).unwrap_err();
        assert_eq!(error.code(), "unsupported_config");
        let details = error.details();
        let diagnostic = blocking_diagnostic_for(&details, "helix.steel_plugins.extra[0].source");
        assert_eq!(diagnostic["status"], "invalid_helix_steel_plugins");
    }

    // Defends: compact badge text normalization trims and truncates user input consistently.
    #[test]
    fn applies_compact_badge_text_behavior() {
        let path = write_user_config("[zellij]\ncustom_text = \"  [hello]  world demo  \"\n");
        let data = normalize_config(&request_for(path)).unwrap();

        assert_eq!(
            data.normalized_config.get("zellij_custom_text").unwrap(),
            "hello wo"
        );
    }

    // Defends: compact tab-label mode flows through the main config contract as a typed Zellij setting.
    #[test]
    fn normalizes_zellij_tab_label_mode() {
        let path = write_user_config("[zellij]\ntab_label_mode = \"compact\"\n");
        let data = normalize_config(&request_for(path)).unwrap();

        assert_eq!(
            data.normalized_config.get("zellij_tab_label_mode").unwrap(),
            "compact"
        );

        let bad_path = write_user_config("[zellij]\ntab_label_mode = \"tiny\"\n");
        let error = normalize_config(&request_for(bad_path)).unwrap_err();
        assert_eq!(error.code(), "unsupported_config");
    }

    // Defends: status-bar widget chrome flows through the main config contract as bounded enum settings.
    #[test]
    fn normalizes_zellij_widget_chrome() {
        let path =
            write_user_config("[zellij]\nwidget_frame = \"round\"\nwidget_separator = \"pipe\"\n");
        let data = normalize_config(&request_for(path)).unwrap();

        assert_eq!(
            data.normalized_config.get("zellij_widget_frame").unwrap(),
            "round"
        );
        assert_eq!(
            data.normalized_config
                .get("zellij_widget_separator")
                .unwrap(),
            "pipe"
        );

        let bad_path = write_user_config("[zellij]\nwidget_separator = \"comma\"\n");
        let error = normalize_config(&request_for(bad_path)).unwrap_err();
        assert_eq!(error.code(), "unsupported_config");
    }

    // Defends: semantic Zellij keybinding remaps flow through the main config contract as a typed action map without taking over default merging from Zellij materialization.
    #[test]
    fn normalizes_zellij_keybinding_map() {
        let path = write_user_config(
            r#"
[zellij.keybindings]
menu = ["Alt Space"]
toggle_left_sidebar = []
"#,
        );
        let data = normalize_config(&request_for(path)).unwrap();
        let keybindings = data
            .normalized_config
            .get("zellij_keybindings")
            .and_then(JsonValue::as_object)
            .expect("zellij keybindings");

        assert_eq!(keybindings["menu"], json!(["Alt Space"]));
        assert_eq!(keybindings["toggle_left_sidebar"], json!([]));
        assert!(!keybindings.contains_key("toggle_editor_right_sidebar_focus"));
    }

    // Defends: curated native Zellij key policy remaps flow through the main config contract as a typed action map.
    #[test]
    fn normalizes_zellij_native_keybinding_map() {
        let path = write_user_config(
            r#"
[zellij.native_keybindings]
scroll_mode = ["Ctrl Alt x"]
scroll_mode_unbind = []
"#,
        );
        let data = normalize_config(&request_for(path)).unwrap();
        let keybindings = data
            .normalized_config
            .get("zellij_native_keybindings")
            .and_then(JsonValue::as_object)
            .expect("zellij native keybindings");

        assert_eq!(keybindings["scroll_mode"], json!(["Ctrl Alt x"]));
        assert_eq!(keybindings["scroll_mode_unbind"], json!([]));
    }

    // Defends: semantic Yazi integration keybinding remaps flow through the main config contract as a typed action map.
    #[test]
    fn normalizes_yazi_keybinding_map() {
        let path = write_user_config(
            r#"
[yazi.keybindings]
open_zoxide_in_editor = ["<A-x>"]
open_directory_as_workspace_pane = []
"#,
        );
        let data = normalize_config(&request_for(path)).unwrap();
        let keybindings = data
            .normalized_config
            .get("yazi_keybindings")
            .and_then(JsonValue::as_object)
            .expect("yazi keybindings");

        assert_eq!(keybindings["open_zoxide_in_editor"], json!(["<A-x>"]));
        assert_eq!(keybindings["open_directory_as_workspace_pane"], json!([]));
    }

    // Defends: removed config surfaces fail as unsupported config instead of being silently accepted.
    #[test]
    fn rejects_removed_unknown_config_surfaces_without_migration() {
        for (raw_config, expected_headline) in [
            (
                "[ascii]\nmode = \"animated\"\n",
                "Unknown config field at ascii",
            ),
            (
                "[shell]\nenable_atuin = true\n",
                "Unknown config field at shell.enable_atuin",
            ),
            (
                "[packs]\nenabled = [\"git\"]\nuser_packages = [\"docker\"]\n\n[packs.declarations]\ngit = [\"gh\", \"prek\"]\n",
                "Unknown config field at packs",
            ),
            (
                "[zellij]\npersistent_sessions = true\n",
                "Removed persistent-session config field at zellij.persistent_sessions",
            ),
            (
                "[zellij]\nsession_name = \"demo\"\n",
                "Removed persistent-session config field at zellij.session_name",
            ),
            (
                "[editor]\ninitial_sidebar_state = \"closed\"\n",
                "Unknown config field at editor.initial_sidebar_state",
            ),
            (
                "[editor]\nenable_sidebar = false\n",
                "Unknown config field at editor.enable_sidebar",
            ),
        ] {
            let path = write_user_config(raw_config);
            let error = normalize_config(&request_for(path)).unwrap_err();

            assert_eq!(error.class().as_str(), "config");
            assert_eq!(error.code(), "unsupported_config");
            let details = error.details();
            assert_eq!(
                details["blocking_diagnostics"][0]["headline"],
                expected_headline
            );
        }
    }

    // Defends: invalid enum values produce structured diagnostics instead of generic parse failures.
    #[test]
    fn rejects_invalid_enum_values_with_structured_diagnostics() {
        let path = write_user_config("[shell]\ndefault_shell = \"powershell\"\n");
        let error = normalize_config(&request_for(path)).unwrap_err();

        assert_eq!(error.class().as_str(), "config");
        let details = error.details();
        assert_eq!(
            details["blocking_diagnostics"][0]["headline"],
            "Unsupported config value at shell.default_shell"
        );
    }

    // Regression: dynamic status widgets normalize, while the retired cursor widget is rejected before Zellij layout generation.
    #[test]
    fn normalizes_dynamic_widget_tray_and_rejects_retired_cursor_value() {
        let valid_path =
            write_user_config("[zellij]\nwidget_tray = [\"editor\", \"workspace\", \"cpu\"]\n");
        let data = normalize_config(&request_for(valid_path)).unwrap();
        assert_eq!(
            data.normalized_config.get("zellij_widget_tray").unwrap(),
            &json!(["editor", "workspace", "cpu"])
        );

        let path = write_user_config("[zellij]\nwidget_tray = [\"editor\", \"cursor\"]\n");
        let error = normalize_config(&request_for(path)).unwrap_err();

        assert_eq!(error.class().as_str(), "config");
        assert_eq!(error.code(), "unsupported_config");
        let details = error.details();
        assert_eq!(
            details["blocking_diagnostics"][0]["headline"],
            "Unsupported config value at zellij.widget_tray"
        );
    }

    // Regression: wrong bool types in config.toml are blocking user errors, not implicit coercions.
    #[test]
    fn rejects_wrong_settings_bool_type() {
        let mut value = default_main_config();
        value["core"]["debug_mode"] = json!("false");
        let request = request_for(write_main_config(&value));

        let error = normalize_config(&request).unwrap_err();
        assert_eq!(error.code(), "unsupported_config");
        let details = error.details();
        let diagnostic = blocking_diagnostic_for(&details, "core.debug_mode");

        assert_eq!(diagnostic["status"], "type_mismatch");
        assert_eq!(diagnostic["blocking"], json!(true));
        assert_eq!(diagnostic["fix_available"], json!(false));
    }
}
