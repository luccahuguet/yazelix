use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::{Path, PathBuf},
};

use ratconfig::{ConfigUiDiagnostic, ConfigUiDiagnosticScope, ConfigUiFieldId};
use serde_json::{Value as JsonValue, json};

use crate::{catalog::*, common::*};

pub(crate) type ZellijSidecar = BTreeMap<&'static str, JsonValue>;
pub(crate) type ZellijInvalidInputs = BTreeMap<&'static str, String>;

#[derive(Default)]
struct ZellijParseState {
    config: ZellijSidecar,
    invalid_inputs: ZellijInvalidInputs,
    diagnostics: Vec<ConfigUiDiagnostic>,
    seen: BTreeSet<&'static str>,
}
pub(crate) fn packaged_zellij_defaults() -> ZellijSidecar {
    BTreeMap::from([
        ("theme", json!("default")),
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
pub(crate) fn packaged_zellij_theme_choices() -> Vec<String> {
    std::iter::once("default")
        .chain(include_str!("../zellij-themes.txt").lines())
        .map(str::to_string)
        .collect()
}
pub(crate) fn write_zellij_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    let spec = require_zellij_field(field_path)?;
    if spec.path == "theme" && value.as_str() == Some("default") {
        return unset_zellij_config_field(path, field_path);
    }
    let raw = read_editable_zellij_sidecar(path)?;
    atomic_write(path, &patch_zellij_field(&raw, spec, value)?)?;
    // Best-effort: patch the watched runtime config without wiping launch patches.
    let _ = refresh_active_zellij_runtime_field(spec, Some(value));
    Ok(())
}
pub(crate) fn unset_zellij_config_field(path: &Path, field_path: &str) -> Result<()> {
    let spec = require_zellij_field(field_path)?;
    if path_entry_exists(path)? {
        let raw = read_editable_zellij_sidecar(path)?;
        let updated = remove_zellij_field(&raw, spec);
        if zellij_sidecar_has_settings(&updated) {
            atomic_write(path, &updated)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    let defaults = packaged_zellij_defaults();
    let default = defaults
        .get(spec.path)
        .expect("known Zellij field has a packaged default");
    // Best-effort: restore the watched runtime field without wiping launch patches.
    let _ = refresh_active_zellij_runtime_field(spec, (spec.path != "theme").then_some(default));
    Ok(())
}

pub(crate) fn read_zellij_sidecar(path: &Path) -> Result<String> {
    read_optional_text(path)
}
fn read_editable_zellij_sidecar(path: &Path) -> Result<String> {
    let raw = read_zellij_sidecar(path)?;
    let (_, _, diagnostics) = parse_zellij_sidecar(&raw);
    if let Some(diagnostic) = diagnostics.iter().find(|diagnostic| {
        diagnostic.blocking && !matches!(&diagnostic.scope, ConfigUiDiagnosticScope::Field(_))
    }) {
        return Err(error(format!(
            "cannot update zellij/config.kdl: {}",
            diagnostic.headline
        )));
    }
    Ok(raw)
}

fn refresh_active_zellij_runtime_field(spec: &FieldSpec, value: Option<&JsonValue>) -> Result<()> {
    let Some(runtime_config) = active_zellij_runtime_config_path() else {
        return Ok(());
    };
    let raw = fs::read_to_string(&runtime_config)?;
    let updated = match value {
        Some(value) => patch_zellij_field(&raw, spec, value)?,
        None => remove_zellij_field(&raw, spec),
    };
    atomic_write(&runtime_config, &updated)
}

fn active_zellij_runtime_config_path() -> Option<PathBuf> {
    env::var_os("ZELLIJ_SESSION_NAME")
        .or_else(|| env::var_os("YAZELIX_ZELLIJ_SESSION_NAME"))
        .filter(|value| !value.is_empty())?;
    let path = PathBuf::from(env::var_os("YAZELIX_STATE_DIR").filter(|value| !value.is_empty())?)
        .join("zellij/config.kdl");
    path.is_file().then_some(path)
}

#[cfg(test)]
pub(crate) fn patch_zellij_field_in_text(
    text: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<String> {
    patch_zellij_field(text, require_zellij_field(field_path)?, value)
}
#[cfg(test)]
pub(crate) fn unset_zellij_field_in_text(text: &str, field_path: &str) -> Result<String> {
    Ok(remove_zellij_field(text, require_zellij_field(field_path)?))
}
fn patch_zellij_field(text: &str, spec: &FieldSpec, value: &JsonValue) -> Result<String> {
    let assignment = zellij_field_assignment(spec, value)?;
    let token = spec.path.rsplit('.').next().unwrap();
    let mut out = String::with_capacity(text.len() + assignment.len());
    let mut replaced = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let syntax = zellij_line(line);
        let indent = &line[..line.len() - trimmed.len()];
        if zellij_line_token(syntax.content) == token && syntax.braces == (0, 0) {
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
                let syntax = zellij_line(line);
                if depth == 0 && zellij_line_token(syntax.content) != "ui" {
                    return true;
                }
                let (opens, closes) = syntax.braces;
                depth += opens as i32 - closes as i32;
                false
            })
            .collect();
    }
    let token = spec.path.rsplit('.').next().unwrap();
    text.split_inclusive('\n')
        .filter(|line| {
            let syntax = zellij_line(line);
            zellij_line_token(syntax.content) != token || syntax.braces != (0, 0)
        })
        .collect()
}
struct ZellijLine<'a> {
    content: &'a str,
    braces: (usize, usize),
    leaf: bool,
    quote_closed: bool,
}
fn zellij_line(line: &str) -> ZellijLine<'_> {
    let line = line.trim_start();
    let mut opens = 0;
    let mut closes = 0;
    let mut semicolons = 0;
    let mut structural_syntax = false;
    let mut quoted = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    let mut end = line.len();
    while let Some((index, ch)) = chars.next() {
        match ch {
            '#' if index == 0 => {
                end = 0;
                break;
            }
            '\\' if quoted && !escaped => {
                escaped = true;
                continue;
            }
            '\\' if !quoted => structural_syntax = true,
            '"' if !escaped => quoted = !quoted,
            '/' if !quoted => match chars.peek().map(|(_, next)| *next) {
                Some('/') => {
                    end = index;
                    break;
                }
                Some('*' | '-') => structural_syntax = true,
                _ => {}
            },
            '*' if !quoted && chars.peek().is_some_and(|(_, next)| *next == '/') => {
                structural_syntax = true
            }
            '{' if !quoted => opens += 1,
            '}' if !quoted => closes += 1,
            ';' if !quoted => semicolons += 1,
            _ => {}
        }
        escaped = false;
    }
    let content = &line[..end];
    ZellijLine {
        content,
        braces: (opens, closes),
        leaf: !structural_syntax
            && opens == 0
            && closes == 0
            && (semicolons == 0 || semicolons == 1 && content.trim_end().ends_with(';')),
        quote_closed: !quoted,
    }
}
fn zellij_sidecar_has_settings(text: &str) -> bool {
    text.lines().any(|line| {
        let line = zellij_line(line).content.trim();
        !line.is_empty()
    })
}
fn zellij_line_token(line: &str) -> &str {
    line.split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
        .next()
        .unwrap_or("")
}
fn zellij_block_open(line: &str, token: &str) -> bool {
    line.strip_prefix(token)
        .is_some_and(|suffix| suffix.trim() == "{")
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
pub(crate) fn parse_zellij_sidecar(
    raw: &str,
) -> (ZellijSidecar, ZellijInvalidInputs, Vec<ConfigUiDiagnostic>) {
    let mut state = ZellijParseState::default();
    let mut block_depth = 0;

    for (index, raw_line) in raw.lines().enumerate() {
        let line_number = index + 1;
        let syntax = zellij_line(raw_line);
        if !syntax.quote_closed {
            state.diagnostics.push(zellij_diagnostic(
                line_number,
                "unterminated Zellij quoted string",
                "Close the quoted value before editing from yzx config.",
            ));
            continue;
        }
        let mut line = syntax.content.trim();
        let mut closed = false;
        while let Some(rest) = line.strip_prefix('}') {
            closed = true;
            if block_depth == 0 {
                state.diagnostics.push(zellij_diagnostic(
                    line_number,
                    "unmatched Zellij block close",
                    "Remove the extra closing brace before editing from yzx config.",
                ));
            } else {
                block_depth -= 1;
            }
            line = rest.trim_start();
        }
        if closed {
            if !line.is_empty() && line != ";" {
                state.diagnostics.push(zellij_diagnostic(
                    line_number,
                    "unsupported content after Zellij block close",
                    "Put the next node on a new line before editing from yzx config.",
                ));
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let braces = syntax.braces;
        let token = zellij_line_token(line);
        if token.is_empty() {
            continue;
        }
        match block_depth {
            0 => parse_zellij_top_level_line(&mut state, line, token, &syntax, line_number),
            1 => {
                if token == "pane_frames" && zellij_block_open(line, token) {
                    block_depth = 2;
                } else {
                    state.diagnostics.push(zellij_diagnostic(
                        line_number,
                        format!("unsupported Zellij ui node `{token}`"),
                        "The managed editor only supports ui.pane_frames.rounded_corners.",
                    ));
                }
            }
            2 => {
                if token == "rounded_corners" {
                    parse_zellij_config_value(
                        &mut state,
                        zellij_field("ui.pane_frames.rounded_corners").expect("known field"),
                        line,
                        braces,
                        line_number,
                    );
                } else {
                    state.diagnostics.push(zellij_diagnostic(
                        line_number,
                        format!("unsupported Zellij pane frame node `{token}`"),
                        "The managed editor only supports rounded_corners in this block.",
                    ));
                }
            }
            _ => unreachable!("Zellij parser only tracks ui and pane_frames"),
        }
        if block_depth == 0 && token == "ui" && zellij_block_open(line, token) {
            block_depth = 1;
        }
    }

    if block_depth > 0 {
        state.diagnostics.push(zellij_diagnostic(
            raw.lines().count().max(1),
            "unterminated Zellij block",
            "The managed editor only supports complete multiline ui.pane_frames blocks.",
        ));
    }

    (state.config, state.invalid_inputs, state.diagnostics)
}
fn parse_zellij_top_level_line(
    state: &mut ZellijParseState,
    line: &str,
    token: &str,
    syntax: &ZellijLine<'_>,
    line_number: usize,
) {
    if token == "ui" {
        if !state.seen.insert("ui") {
            state.diagnostics.push(zellij_diagnostic(
                line_number,
                "duplicate Zellij node `ui`",
                "Keep one ui block before editing from yzx config.",
            ));
        } else if !zellij_block_open(line, token) {
            state.diagnostics.push(zellij_diagnostic(
                line_number,
                "unsupported Zellij ui form",
                "The managed editor expects ui as a block.",
            ));
        }
        return;
    }
    if ZELLIJ_FORBIDDEN_TOP_LEVEL.contains(&token) {
        state.diagnostics.push(zellij_diagnostic(
            line_number,
            format!("guarded Zellij node `{token}`"),
            "This node belongs to the managed runtime and cannot live in the editable sidecar.",
        ));
        return;
    }
    let Some(spec) = top_level_zellij_field(token) else {
        if ZELLIJ_FIELDS
            .iter()
            .any(|spec| spec.path.rsplit('.').next() == Some(token))
        {
            state.diagnostics.push(zellij_diagnostic(
                line_number,
                format!("ambiguous Zellij node `{token}`"),
                "This token is managed inside a structured block and cannot also be preserved at top level.",
            ));
        } else if syntax.leaf {
            state.diagnostics.push(zellij_unvalidated_diagnostic(
                line_number,
                token,
                "This native leaf node is preserved unchanged; Zellij owns its validity.",
            ));
        } else {
            state.diagnostics.push(zellij_diagnostic(
                line_number,
                format!("unsupported Zellij node `{token}`"),
                "Edit structured native configuration by hand.",
            ));
        }
        return;
    };

    parse_zellij_config_value(state, spec, line, syntax.braces, line_number);
}
fn zellij_scalar_value<'a>(
    line: &'a str,
    token: &str,
    braces: (usize, usize),
) -> Option<(&'a str, bool)> {
    if braces != (0, 0) {
        return None;
    }
    let value = line.strip_prefix(token)?.trim();
    let value = value.strip_suffix(';').unwrap_or(value).trim_end();
    if value.is_empty() {
        return None;
    }
    if let Some(quoted) = value.strip_prefix('"') {
        let mut escaped = false;
        for (index, ch) in quoted.char_indices() {
            if ch == '\\' && !escaped {
                escaped = true;
                continue;
            }
            if ch == '"' && !escaped {
                return quoted[index + ch.len_utf8()..]
                    .trim()
                    .is_empty()
                    .then_some((&quoted[..index], true));
            }
            escaped = false;
        }
        None
    } else {
        (!value.contains(['"', '\\', '=', ';']) && value.split_whitespace().count() == 1)
            .then_some((value, false))
    }
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
    state: &mut ZellijParseState,
    spec: &FieldSpec,
    line: &str,
    braces: (usize, usize),
    line_number: usize,
) {
    if !state.seen.insert(spec.path) {
        state.diagnostics.push(zellij_diagnostic(
            line_number,
            format!("duplicate Zellij node `{}`", spec.path),
            "Keep one assignment before editing from yzx config.",
        ));
        return;
    }
    let token = spec.path.rsplit('.').next().unwrap();
    let Some((value, quoted)) = zellij_scalar_value(line, token, braces) else {
        state.diagnostics.push(zellij_diagnostic(
            line_number,
            format!("unsupported Zellij form for `{}`", spec.path),
            "The managed editor only supports one scalar value for this setting.",
        ));
        return;
    };
    let parsed = parse_kdl_json_value(value, quoted, spec)
        .and_then(|value| normalize_zellij_field_value(spec, &value));
    match parsed {
        Ok(value) => {
            state.config.insert(spec.path, value);
        }
        Err(error) => {
            state.invalid_inputs.insert(
                spec.path,
                if quoted {
                    serde_json::to_string(value).expect("string JSON")
                } else {
                    value.to_string()
                },
            );
            state.diagnostics.push(zellij_field_diagnostic(
                line_number,
                spec,
                if matches!(spec.kind, "boolean" | "integer" | "string") {
                    format!("invalid Zellij value for `{}`", spec.path)
                } else {
                    format!("unsupported Zellij value kind `{}`", spec.kind)
                },
                error.to_string(),
            ));
        }
    }
}
fn parse_kdl_json_value(value: &str, quoted: bool, spec: &FieldSpec) -> Result<JsonValue> {
    match spec.kind {
        "boolean" if !quoted => match value {
            "true" => Ok(json!(true)),
            "false" => Ok(json!(false)),
            _ => Err(error("Expected true or false.")),
        },
        "integer" if !quoted => value
            .parse::<i64>()
            .map(JsonValue::from)
            .map_err(|_| error("Expected an integer.")),
        "string" if quoted => {
            validate_zellij_string(value)?;
            Ok(json!(value))
        }
        "string" => Err(error("Expected a quoted string.")),
        "boolean" | "integer" => Err(error(format!("Expected an unquoted {}.", spec.kind))),
        _ => Err(error(
            "The managed editor only supports boolean, integer, and string KDL values.",
        )),
    }
}
fn validate_zellij_string(value: &str) -> Result<()> {
    if value.contains(['\\', '"']) || value.chars().any(char::is_control) {
        Err(error(
            "The managed editor supports quoted Zellij strings without escapes.",
        ))
    } else {
        Ok(())
    }
}
fn zellij_diagnostic(
    line_number: usize,
    headline: impl Into<String>,
    detail: impl Into<String>,
) -> ConfigUiDiagnostic {
    ConfigUiDiagnostic {
        path: format!("zellij/config.kdl:{line_number}"),
        status: "blocked".to_string(),
        headline: headline.into(),
        blocking: true,
        scope: ConfigUiDiagnosticScope::Source {
            source_id: SOURCE_ZELLIJ.to_string(),
        },
        detail_lines: vec![detail.into()],
    }
}
fn zellij_field_diagnostic(
    line_number: usize,
    spec: &FieldSpec,
    headline: String,
    detail: impl Into<String>,
) -> ConfigUiDiagnostic {
    ConfigUiDiagnostic {
        path: format!("zellij/config.kdl:{line_number}"),
        status: "invalid".to_string(),
        headline,
        blocking: true,
        scope: ConfigUiDiagnosticScope::Field(ConfigUiFieldId::new(SOURCE_ZELLIJ, spec.path)),
        detail_lines: vec![detail.into()],
    }
}
fn zellij_unvalidated_diagnostic(
    line_number: usize,
    path: &str,
    detail: impl Into<String>,
) -> ConfigUiDiagnostic {
    ConfigUiDiagnostic {
        path: format!("zellij/config.kdl:{line_number}"),
        status: "unvalidated".to_string(),
        headline: format!("unvalidated Zellij node `{path}`"),
        blocking: false,
        scope: ConfigUiDiagnosticScope::Source {
            source_id: SOURCE_ZELLIJ.to_string(),
        },
        detail_lines: vec![detail.into()],
    }
}
fn normalize_zellij_field_value(spec: &FieldSpec, value: &JsonValue) -> Result<JsonValue> {
    match spec.kind {
        "boolean" => Ok(json!(json_bool(spec.path, value)?)),
        "integer" => Ok(json!(json_positive_i64(spec.path, value)?)),
        "string" => {
            let value = spec.json_choice(value)?;
            validate_zellij_string(value)?;
            Ok(json!(value))
        }
        _ => Err(error(format!("unsupported Zellij value for {}", spec.path))),
    }
}
