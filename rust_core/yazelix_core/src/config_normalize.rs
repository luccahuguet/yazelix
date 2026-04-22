use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

const FLEXIBLE_NUMERIC_PATHS: &[&str] = &["core.welcome_duration_seconds"];

#[derive(Debug, Clone)]
pub struct NormalizeConfigRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub include_missing: bool,
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
    default: Option<TomlValue>,
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
    let config = read_toml_table(&request.config_path, "read_config")?;
    let default_config = read_toml_table(&request.default_config_path, "read_default_config")?;
    let contract = read_toml_table(&request.contract_path, "read_config_contract")?;
    let fields = load_contract_fields(&contract)?;
    let config_file = request.config_path.to_string_lossy().to_string();

    let diagnostic_report = build_diagnostic_report(
        &config,
        &default_config,
        &fields,
        &request.config_path,
        request.include_missing,
    )?;
    if diagnostic_report.has_blocking {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "unsupported_config",
            format!(
                "Yazelix found stale or unsupported config entries in {}.",
                diagnostic_report.config_path
            ),
            "Update the reported config fields manually, then retry. Use `yzx config reset` only as a blunt fallback.",
            serde_json::to_value(&diagnostic_report).unwrap_or_else(|_| json!({})),
        ));
    }

    let mut normalized_config = JsonMap::new();
    for field in fields.values() {
        let normalized = normalize_field(field, &config)?;
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
                default: field_table.get("default").cloned(),
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
    include_missing: bool,
) -> Result<ConfigDiagnosticReport, CoreError> {
    let mut reference = TomlValue::Table(default_config.clone());
    for field in fields.values() {
        if let Some(default_value) = &field.default {
            set_nested_value(
                &mut reference,
                &field.path.split('.').collect::<Vec<_>>(),
                default_value.clone(),
            );
        }
    }

    let should_validate_like_startup = config_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "yazelix.toml")
        .unwrap_or(false);

    let findings = if should_validate_like_startup {
        let mut findings = compare_configs(&reference, &TomlValue::Table(user_config.clone()), &[]);
        if !include_missing {
            findings.retain(|finding| finding.kind != "missing_field");
        }
        findings.extend(validate_enum_values(user_config, fields));
        findings
    } else {
        Vec::new()
    };

    let schema_diagnostics = findings
        .into_iter()
        .map(make_schema_diagnostic)
        .collect::<Vec<_>>();
    let doctor_diagnostics = schema_diagnostics.clone();
    let blocking_diagnostics = doctor_diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.blocking)
        .cloned()
        .collect::<Vec<_>>();

    Ok(ConfigDiagnosticReport {
        config_path: config_path.to_string_lossy().to_string(),
        issue_count: doctor_diagnostics.len(),
        blocking_count: blocking_diagnostics.len(),
        fixable_count: 0,
        has_blocking: !blocking_diagnostics.is_empty(),
        has_fixable_config_issues: false,
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

fn compare_configs(default: &TomlValue, user: &TomlValue, path: &[&str]) -> Vec<SchemaFinding> {
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
            if !default_table.contains_key(key) {
                let mut finding_path = path.to_vec();
                finding_path.push(key);
                let formatted = format_config_path(&finding_path);
                findings.push(SchemaFinding {
                    kind: "unknown_field",
                    path: formatted.clone(),
                    message: format!("Unknown config field: {formatted}"),
                });
            }
        }

        for key in default_table.keys() {
            if !user_table.contains_key(key) {
                let mut finding_path = path.to_vec();
                finding_path.push(key);
                let formatted = format_config_path(&finding_path);
                findings.push(SchemaFinding {
                    kind: "missing_field",
                    path: formatted.clone(),
                    message: format!("Missing config field: {formatted}"),
                });
            }
        }

        for (key, default_value) in default_table {
            if let Some(user_value) = user_table.get(key) {
                let mut nested_path = path.to_vec();
                nested_path.push(key);
                findings.extend(compare_configs(default_value, user_value, &nested_path));
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
        let user_root = TomlValue::Table(user_config.clone());
        let Some(value) = get_nested_value(&user_root, &path).cloned() else {
            continue;
        };

        if field.validation == "enum_string_list" {
            if let TomlValue::Array(values) = &value {
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
                let rendered = toml_value_to_lossy_string(&value);
                if !field.allowed_values.contains(&rendered) {
                    findings.push(invalid_enum_finding(
                        &field.path,
                        &field.allowed_values,
                        &rendered,
                    ));
                }
            }
        } else {
            let rendered = toml_value_to_lossy_string(&value);
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
    let blocking = finding.kind != "missing_field";
    let mut diagnostic = ConfigDiagnostic {
        category: "schema".to_string(),
        path: finding.path.clone(),
        status: finding.kind.to_string(),
        blocking,
        fix_available: false,
        headline: String::new(),
        detail_lines: Vec::new(),
    };

    match finding.kind {
        "unknown_field" => {
            diagnostic.headline = format!("Unknown config field at {}", finding.path);
            diagnostic.detail_lines = vec![
                finding.message,
                "Next: Remove or rename this field manually.".to_string(),
                "Next: Run `yzx doctor --verbose` to review the full config report.".to_string(),
                "Next: Use `yzx config reset` only as a blunt fallback.".to_string(),
            ];
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
                "Next: Use `yzx config reset` only as a blunt fallback.".to_string(),
            ];
        }
        "missing_field" => {
            diagnostic.headline = format!("Missing config field at {}", finding.path);
            diagnostic.detail_lines = vec![
                finding.message,
                "Next: Add the field from the current template if you want your config to stay fully in sync.".to_string(),
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
) -> Result<JsonValue, CoreError> {
    let value = get_nested_value(
        &TomlValue::Table(raw_config.clone()),
        &field.path.split('.').collect::<Vec<_>>(),
    )
    .cloned()
    .or_else(|| field.default.clone())
    .unwrap_or_else(|| TomlValue::String(String::new()));

    match field.parser_behavior.as_str() {
        "compact_badge_text" => Ok(JsonValue::String(compact_badge_text(&value))),
        "empty_string_to_null" => {
            let value = toml_value_to_lossy_string(&value);
            if value.is_empty() {
                Ok(JsonValue::Null)
            } else {
                Ok(JsonValue::String(value))
            }
        }
        "bool_to_string" => {
            let value = value.as_bool().ok_or_else(|| {
                invalid_value_error(
                    &field.path,
                    &toml_value_to_lossy_string(&value),
                    "a boolean",
                )
            })?;
            Ok(JsonValue::String(
                if value { "true" } else { "false" }.to_string(),
            ))
        }
        _ => normalize_direct_field(field, &value),
    }
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
    let remediation = if field_path == "terminal.config_mode" {
        "Use `terminal.config_mode = \"yazelix\"` for the supported managed path, or `\"user\"` only when you want Yazelix to load the terminal's native config file."
    } else {
        "Update yazelix.toml with a supported value, or run `yzx config reset` to restore the template."
    };

    CoreError::classified(
        ErrorClass::Config,
        "invalid_config_value",
        format!("Invalid {field_path} value '{actual_value}'. Expected {expectation}."),
        remediation,
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

fn get_nested_value<'a>(value: &'a TomlValue, path: &[&str]) -> Option<&'a TomlValue> {
    let mut current = value;
    for segment in path {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

fn set_nested_value(value: &mut TomlValue, path: &[&str], new_value: TomlValue) {
    if path.is_empty() {
        *value = new_value;
        return;
    }
    let Some(table) = value.as_table_mut() else {
        return;
    };
    if path.len() == 1 {
        table.insert(path[0].to_string(), new_value);
        return;
    }
    let entry = table
        .entry(path[0].to_string())
        .or_insert_with(|| TomlValue::Table(toml::Table::new()));
    set_nested_value(entry, &path[1..], new_value);
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
            default_config_path: repo.join("yazelix_default.toml"),
            contract_path: repo.join("config_metadata/main_config_contract.toml"),
            include_missing: false,
        }
    }

    fn write_user_config(contents: &str) -> PathBuf {
        let dir = tempdir().expect("tempdir").keep();
        let path = dir.join("yazelix.toml");
        fs::write(&path, contents).expect("write config");
        path
    }

    // Defends: config normalization keeps the parser-owned default keys and value transforms stable.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn normalizes_default_config_with_parser_keys_and_transforms() {
        let repo = repo_root();
        let data = normalize_config(&request_for(repo.join("yazelix_default.toml"))).unwrap();
        let config = data.normalized_config;

        assert_eq!(config.get("default_shell").unwrap(), "nu");
        assert_eq!(config.get("helix_runtime_path").unwrap(), &JsonValue::Null);
        assert_eq!(config.get("zellij_pane_frames").unwrap(), "true");
        assert_eq!(config.get("welcome_duration_seconds").unwrap(), 1.0);
        assert_eq!(config.len(), 36);
    }

    // Defends: compact badge text normalization trims and truncates user input consistently.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=1 total=7/10
    #[test]
    fn applies_compact_badge_text_behavior() {
        let path = write_user_config("[zellij]\ncustom_text = \"  [hello]  world demo  \"\n");
        let data = normalize_config(&request_for(path)).unwrap();

        assert_eq!(
            data.normalized_config.get("zellij_custom_text").unwrap(),
            "hello wo"
        );
    }

    // Defends: removed config surfaces fail as unsupported config instead of being silently accepted.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rejects_removed_unknown_config_surfaces_without_migration() {
        let path = write_user_config("[shell]\nenable_atuin = true\n");
        let error = normalize_config(&request_for(path)).unwrap_err();

        assert_eq!(error.class().as_str(), "config");
        assert_eq!(error.code(), "unsupported_config");
        let details = error.details();
        assert_eq!(
            details["blocking_diagnostics"][0]["headline"],
            "Unknown config field at shell.enable_atuin"
        );
    }

    // Defends: invalid enum values produce structured diagnostics instead of generic parse failures.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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

    // Regression: doctor-style config reports can request missing fields explicitly without changing startup defaults.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn includes_missing_fields_when_requested() {
        let path = write_user_config("[shell]\ndefault_shell = \"nu\"\n");
        let mut request = request_for(path);
        request.include_missing = true;

        let data = normalize_config(&request).unwrap();
        let report = data.diagnostic_report;
        let missing_field = report
            .schema_diagnostics
            .iter()
            .find(|diagnostic| diagnostic.status == "missing_field")
            .expect("missing field diagnostic");

        assert!(report.issue_count > 0);
        assert_eq!(missing_field.status, "missing_field");
        assert!(
            missing_field
                .headline
                .starts_with("Missing config field at ")
        );
    }
}
