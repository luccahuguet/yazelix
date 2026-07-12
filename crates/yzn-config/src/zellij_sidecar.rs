use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use ratconfig::ConfigUiDiagnostic;
use serde_json::{Value as JsonValue, json};

use crate::{catalog::*, common::*};

pub(crate) type ZellijSidecar = BTreeMap<&'static str, JsonValue>;
pub(crate) fn packaged_zellij_defaults() -> ZellijSidecar {
    BTreeMap::from([
        ("pane_frames", json!(true)),
        ("mouse_mode", json!(true)),
        ("scroll_buffer_size", json!(10000)),
        ("copy_on_select", json!(true)),
        ("copy_clipboard", json!("system")),
        ("styled_underlines", json!(true)),
        ("show_startup_tips", json!(true)),
        ("ui.pane_frames.rounded_corners", json!(false)),
    ])
}
pub(crate) fn write_zellij_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let spec = require_zellij_field(field_path)?;
    let (raw, _) = read_editable_zellij_sidecar(path)?;
    atomic_write(path, &patch_zellij_field(&raw, spec, value)?)?;
    // Best-effort: patch the watched runtime config without wiping launch patches.
    let _ = refresh_active_zellij_runtime_field(spec.path, value);
    Ok(())
}
pub(crate) fn unset_zellij_config_field(path: &Path, field_path: &str) -> Result<()> {
    let spec = require_zellij_field(field_path)?;
    if !path_entry_exists(path)? {
        return Ok(());
    }
    let (raw, mut config) = read_editable_zellij_sidecar(path)?;
    config.remove(spec.path);
    if config.is_empty() {
        fs::remove_file(path)?;
    } else {
        atomic_write(path, &remove_zellij_field(&raw, spec))?;
    }
    let defaults = packaged_zellij_defaults();
    let default = defaults
        .get(spec.path)
        .expect("known Zellij field has a packaged default");
    // Best-effort: restore the watched runtime field without wiping launch patches.
    let _ = refresh_active_zellij_runtime_field(spec.path, default);
    Ok(())
}

pub(crate) fn read_zellij_sidecar(path: &Path) -> Result<String> {
    if path_entry_exists(path)? {
        Ok(fs::read_to_string(path)?)
    } else {
        Ok(String::new())
    }
}
fn read_editable_zellij_sidecar(path: &Path) -> Result<(String, ZellijSidecar)> {
    let raw = read_zellij_sidecar(path)?;
    let (config, diagnostics) = parse_zellij_sidecar(&raw);
    if let Some(diagnostic) = diagnostics.iter().find(|diagnostic| diagnostic.blocking) {
        return Err(error(format!(
            "cannot update zellij/config.kdl: {}",
            diagnostic.headline
        )));
    }
    Ok((raw, config))
}

fn refresh_active_zellij_runtime_field(field_path: &str, value: &JsonValue) -> Result<()> {
    let Some(runtime_config) = active_zellij_runtime_config_path() else {
        return Ok(());
    };
    let raw = fs::read_to_string(&runtime_config)?;
    atomic_write(
        &runtime_config,
        &patch_zellij_field_in_text(&raw, field_path, value)?,
    )
}

fn active_zellij_runtime_config_path() -> Option<PathBuf> {
    env::var_os("ZELLIJ_SESSION_NAME")
        .or_else(|| env::var_os("YAZELIX_ZELLIJ_SESSION_NAME"))
        .filter(|value| !value.is_empty())?;
    let path = PathBuf::from(env::var_os("YAZELIX_STATE_DIR").filter(|value| !value.is_empty())?)
        .join("zellij/config.kdl");
    path.is_file().then_some(path)
}

pub(crate) fn patch_zellij_field_in_text(
    text: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<String> {
    patch_zellij_field(text, require_zellij_field(field_path)?, value)
}
fn patch_zellij_field(text: &str, spec: &FieldSpec, value: &JsonValue) -> Result<String> {
    let assignment = zellij_field_assignment(spec, value)?;
    let token = spec.path.rsplit('.').next().unwrap();
    let mut out = String::with_capacity(text.len() + assignment.len());
    let mut replaced = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let content = trimmed.split_once("//").map_or(trimmed, |(text, _)| text);
        let indent = &line[..line.len() - trimmed.len()];
        if zellij_line_token(content) == token && !content.contains('{') {
            out.push_str(indent);
            out.push_str(&assignment);
            out.push('\n');
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        if !out.is_empty() && spec.path.contains('.') && !out.ends_with("\n\n") {
            out.push('\n');
        }
        if spec.path.contains('.') {
            out.push_str(&format!(
                "ui {{\n    pane_frames {{\n        {assignment}\n    }}\n}}\n"
            ));
        } else {
            out.push_str(&assignment);
            out.push('\n');
        }
    }
    Ok(out)
}
fn remove_zellij_field(text: &str, spec: &FieldSpec) -> String {
    if spec.path.contains('.') {
        let mut depth = 0;
        return text
            .split_inclusive('\n')
            .filter(|line| {
                let content = zellij_line_content(line);
                if depth == 0 && zellij_line_token(content) != "ui" {
                    return true;
                }
                depth += zellij_brace_delta(content);
                false
            })
            .collect();
    }
    let token = spec.path.rsplit('.').next().unwrap();
    text.split_inclusive('\n')
        .filter(|line| {
            let content = zellij_line_content(line);
            zellij_line_token(content) != token || content.contains('{')
        })
        .collect()
}
fn zellij_line_content(line: &str) -> &str {
    let line = line.trim_start();
    line.split_once("//").map_or(line, |(text, _)| text)
}
fn zellij_line_token(line: &str) -> &str {
    line.split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
        .next()
        .unwrap_or("")
}
fn zellij_brace_delta(line: &str) -> i32 {
    if line.starts_with('#') {
        return 0;
    }
    line.matches('{').count() as i32 - line.matches('}').count() as i32
}
fn zellij_field_assignment(spec: &FieldSpec, value: &JsonValue) -> Result<String> {
    let token = spec.path.rsplit('.').next().unwrap();
    let rhs = match normalize_zellij_field_value(spec, value)? {
        JsonValue::Bool(flag) => kdl_bool(flag).to_string(),
        JsonValue::Number(number) => number.to_string(),
        JsonValue::String(text) => kdl_string(&text),
        _ => {
            return Err(error(format!("unsupported Zellij value for {}", spec.path)));
        }
    };
    Ok(format!("{token} {rhs}"))
}
pub(crate) fn parse_zellij_sidecar(raw: &str) -> (ZellijSidecar, Vec<ConfigUiDiagnostic>) {
    let mut config = ZellijSidecar::new();
    let mut diagnostics = Vec::new();
    let mut stack: Vec<&str> = Vec::new();

    for (index, raw_line) in raw.lines().enumerate() {
        let line_number = index + 1;
        let mut line = zellij_line_content(raw_line).trim();
        while let Some(rest) = line.strip_prefix('}') {
            if stack.pop().is_none() {
                diagnostics.push(zellij_diagnostic(
                    line_number,
                    "unmatched Zellij block close".to_string(),
                    "Remove the extra closing brace before editing from yzn config.",
                ));
            }
            line = rest.trim_start();
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let token = zellij_line_token(line);
        if token.is_empty() {
            continue;
        }
        match stack.as_slice() {
            [] => {
                parse_zellij_top_level_line(&mut config, &mut diagnostics, line, token, line_number)
            }
            ["ui"] => {
                if token == "pane_frames" && line.contains('{') {
                    stack.push("pane_frames");
                } else {
                    diagnostics.push(zellij_diagnostic(
                        line_number,
                        format!("unsupported Zellij ui node `{token}`"),
                        "The managed editor only supports ui.pane_frames.rounded_corners.",
                    ));
                }
            }
            ["ui", "pane_frames"] => {
                if token == "rounded_corners" {
                    parse_zellij_config_value(
                        &mut config,
                        zellij_field("ui.pane_frames.rounded_corners").expect("known field"),
                        line,
                        token,
                        line_number,
                        &mut diagnostics,
                    );
                } else {
                    diagnostics.push(zellij_diagnostic(
                        line_number,
                        format!("unsupported Zellij pane frame node `{token}`"),
                        "The managed editor only supports rounded_corners in this block.",
                    ));
                }
            }
            _ => unreachable!("Zellij parser stack only contains ui and pane_frames"),
        }
        if stack.is_empty() && token == "ui" && line.contains('{') {
            stack.push("ui");
        }
    }

    if !stack.is_empty() {
        diagnostics.push(zellij_diagnostic(
            raw.lines().count().max(1),
            "unterminated Zellij block".to_string(),
            "The managed editor only supports complete multiline ui.pane_frames blocks.",
        ));
    }

    (config, diagnostics)
}
fn parse_zellij_top_level_line(
    config: &mut ZellijSidecar,
    diagnostics: &mut Vec<ConfigUiDiagnostic>,
    line: &str,
    token: &str,
    line_number: usize,
) {
    if token == "ui" {
        if !line.contains('{') {
            diagnostics.push(zellij_diagnostic(
                line_number,
                "unsupported Zellij ui form".to_string(),
                "The managed editor expects ui as a block.",
            ));
        }
        return;
    }
    if ZELLIJ_FORBIDDEN_TOP_LEVEL.contains(&token) {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("guarded Zellij node `{token}`"),
            "This node belongs to the managed runtime and cannot live in the editable sidecar.",
        ));
        return;
    }
    let Some(spec) = top_level_zellij_field(token) else {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("unsupported Zellij node `{token}`"),
            "Remove this node before editing from yzn config, or keep editing the sidecar by hand.",
        ));
        return;
    };

    parse_zellij_config_value(config, spec, line, token, line_number, diagnostics);
}
fn top_level_zellij_field(token: &str) -> Option<&'static FieldSpec> {
    zellij_field(token).filter(|spec| !spec.path.contains('.'))
}
fn zellij_field(path: &str) -> Option<&'static FieldSpec> {
    ZELLIJ_FIELDS.iter().find(|spec| spec.path == path)
}
fn require_zellij_field(path: &str) -> Result<&'static FieldSpec> {
    zellij_field(path).ok_or_else(|| error(format!("unknown Zellij config path: {path}")))
}
fn parse_zellij_config_value(
    config: &mut ZellijSidecar,
    spec: &FieldSpec,
    line: &str,
    token: &str,
    line_number: usize,
    diagnostics: &mut Vec<ConfigUiDiagnostic>,
) {
    let Some(value) = parse_kdl_json_value(line, token, spec, line_number, diagnostics) else {
        return;
    };
    match normalize_zellij_field_value(spec, &value) {
        Ok(value) => {
            config.insert(spec.path, value);
        }
        Err(error) => diagnostics.push(zellij_diagnostic(
            line_number,
            format!("invalid Zellij value for `{}`", spec.path),
            error.to_string(),
        )),
    }
}
fn parse_kdl_json_value(
    line: &str,
    token: &str,
    spec: &FieldSpec,
    line_number: usize,
    diagnostics: &mut Vec<ConfigUiDiagnostic>,
) -> Option<JsonValue> {
    let Some(value) = first_kdl_value(line, token) else {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("missing Zellij value for `{}`", spec.path),
            format!("Expected {}.", zellij_expected_value(spec.kind)),
        ));
        return None;
    };
    match spec.kind {
        "boolean" => match value {
            "true" => Some(json!(true)),
            "false" => Some(json!(false)),
            _ => {
                diagnostics.push(zellij_diagnostic(
                    line_number,
                    format!("invalid Zellij value for `{}`", spec.path),
                    "Expected true or false.",
                ));
                None
            }
        },
        "integer" => value.parse::<i64>().map(JsonValue::from).map_or_else(
            |_| {
                diagnostics.push(zellij_diagnostic(
                    line_number,
                    format!("invalid Zellij value for `{}`", spec.path),
                    "Expected an integer.",
                ));
                None
            },
            Some,
        ),
        "string" => Some(json!(value)),
        _ => {
            diagnostics.push(zellij_diagnostic(
                line_number,
                format!("unsupported Zellij value kind `{}`", spec.kind),
                "The managed editor only supports boolean, integer, and string KDL values.",
            ));
            None
        }
    }
}
fn zellij_expected_value(kind: &str) -> &'static str {
    match kind {
        "boolean" => "true or false",
        "integer" => "an integer",
        "string" => "a string",
        _ => "a supported scalar",
    }
}
fn first_kdl_value<'a>(line: &'a str, token: &str) -> Option<&'a str> {
    line.strip_prefix(token)?
        .split_whitespace()
        .next()
        .map(|value| {
            value
                .trim_end_matches(';')
                .trim_end_matches('{')
                .trim_matches('"')
        })
}
fn zellij_diagnostic(
    line_number: usize,
    headline: String,
    detail: impl Into<String>,
) -> ConfigUiDiagnostic {
    ConfigUiDiagnostic {
        path: format!("zellij/config.kdl:{line_number}"),
        status: "blocked".to_string(),
        headline,
        blocking: true,
        detail_lines: vec![detail.into()],
    }
}
fn normalize_zellij_field_value(spec: &FieldSpec, value: &JsonValue) -> Result<JsonValue> {
    match spec.kind {
        "boolean" => Ok(json!(json_bool(spec.path, value)?)),
        "integer" => Ok(json!(json_positive_i64(spec.path, value)?)),
        "string" => Ok(json!(spec.json_choice(value)?)),
        _ => Err(error(format!("unsupported Zellij value for {}", spec.path))),
    }
}
