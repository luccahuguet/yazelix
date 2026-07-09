use serde_json::{Number as JsonNumber, Value as JsonValue};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::{Table as TomlTable, Value as TomlValue};

pub(super) fn require_path_missing_abs(path: &Path, label: &str, errors: &mut Vec<String>) {
    if path.exists() {
        errors.push(format!("Unexpected {}: {}", label, path.display()));
    }
}

pub(super) fn require_path_exists_abs(path: &Path, label: &str, errors: &mut Vec<String>) {
    if !path.exists() {
        errors.push(format!("Missing {}: {}", label, path.display()));
    }
}

pub(super) fn require_list_contains(
    items: &[String],
    expected: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    if !items.iter().any(|item| item == expected) {
        errors.push(format!(
            "{} is missing expected entry `{}`. Found: {}",
            label,
            expected,
            items.join(", ")
        ));
    }
}

pub(super) fn require_list_not_contains(
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

pub(super) fn run_repo_command(
    repo_root: &Path,
    program: &str,
    args: &[&str],
) -> Result<Output, String> {
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

pub(super) fn build_flake_output_path(
    repo_root: &Path,
    attr: &str,
    label: &str,
) -> Result<PathBuf, String> {
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

pub(super) fn build_nix_file_output_path(
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

fn command_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

pub(super) fn command_output_summary(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = command_stderr(output);
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => "No subprocess output captured".to_string(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr),
    }
}

pub(super) fn format_command(program: &str, args: &[&str]) -> String {
    std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn run_nix_eval(repo_root: &Path, expr: &str) -> Result<JsonValue, String> {
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

pub(super) fn create_unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
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

pub(super) fn prepare_temp_home(temp_home: &Path) -> Result<(), String> {
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

pub(super) fn relative_display(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
}

pub(super) fn read_toml_file(path: &Path) -> Result<TomlTable, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    parse_toml_from_str(&raw, &path.display().to_string())
}

pub(super) fn parse_toml_from_str(raw: &str, label: &str) -> Result<TomlTable, String> {
    toml::from_str::<TomlTable>(raw)
        .map_err(|error| format!("Failed to parse {} as TOML: {}", label, error))
}

pub(super) fn split_field_path(path: &str) -> Vec<&str> {
    path.split('.').collect()
}

pub(super) fn get_nested_toml_value<'a>(
    table: &'a TomlTable,
    path: &[&str],
) -> Option<&'a TomlValue> {
    if path.is_empty() {
        return None;
    }
    let mut current = table.get(path[0])?;
    for segment in &path[1..] {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

pub(super) fn set_nested_toml_value(table: &mut TomlTable, path: &[&str], value: TomlValue) {
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

pub(super) fn toml_to_json(value: &TomlValue) -> JsonValue {
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

pub(super) fn json_values_equal(left: &JsonValue, right: &JsonValue) -> bool {
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

pub(super) fn toml_values_equal(left: &TomlValue, right: &TomlValue) -> bool {
    json_values_equal(&toml_to_json(left), &toml_to_json(right))
}

pub(super) fn format_json_value(value: &JsonValue) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unrenderable json>".to_string())
}

pub(super) fn format_toml_value(value: &TomlValue) -> String {
    format_json_value(&toml_to_json(value))
}

pub(super) fn as_string_list(value: Option<&TomlValue>) -> Vec<String> {
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

pub(super) fn sorted_keys(table: &TomlTable) -> Vec<String> {
    let mut keys = table.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

pub(super) fn escape_nix_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
