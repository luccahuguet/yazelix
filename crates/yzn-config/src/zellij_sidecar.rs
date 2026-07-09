use std::{
    env, fs,
    path::{Path, PathBuf},
};

use ratconfig::ConfigUiDiagnostic;
use serde_json::{Value as JsonValue, json};

use crate::{catalog::*, common::*};

pub(crate) struct ZellijSidecar {
    pub(crate) pane_frames: bool,
    mouse_mode: bool,
    scroll_buffer_size: i64,
    copy_on_select: bool,
    copy_clipboard: String,
    styled_underlines: bool,
    show_startup_tips: bool,
    rounded_corners: bool,
}
impl Default for ZellijSidecar {
    fn default() -> Self {
        Self {
            pane_frames: true,
            mouse_mode: true,
            scroll_buffer_size: 10000,
            copy_on_select: true,
            copy_clipboard: "system".to_string(),
            styled_underlines: true,
            show_startup_tips: true,
            rounded_corners: false,
        }
    }
}
pub(crate) fn write_zellij_config_field(
    path: &Path,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    if !ZELLIJ_FIELDS.iter().any(|spec| spec.path == field_path) {
        return Err(error(format!("unknown Zellij config path: {field_path}")));
    }
    let raw = fs::read_to_string(path)?;
    let (mut config, diagnostics) = parse_zellij_sidecar(&raw);
    if let Some(diagnostic) = diagnostics.iter().find(|diagnostic| diagnostic.blocking) {
        return Err(error(format!(
            "cannot update zellij/config.kdl: {}",
            diagnostic.headline
        )));
    }
    set_zellij_field_value(&mut config, field_path, value)?;
    atomic_write(path, &render_zellij_sidecar(&config))
}

/// When `yzn config` runs inside a managed session, also patch the active runtime
/// Zellij config Zellij is watching. Preserves launch-time integration patches.
/// Returns `Ok(true)` if the active file was updated, `Ok(false)` if no session runtime.
pub(crate) fn refresh_active_zellij_runtime_field(
    field_path: &str,
    value: &JsonValue,
) -> Result<bool> {
    let Some(runtime_config) = active_zellij_runtime_config_path() else {
        return Ok(false);
    };
    let raw = fs::read_to_string(&runtime_config)?;
    let patched = patch_zellij_field_in_text(&raw, field_path, value)?;
    atomic_write(&runtime_config, &patched)?;
    Ok(true)
}

fn active_zellij_runtime_config_path() -> Option<PathBuf> {
    let in_session = env::var_os("ZELLIJ_SESSION_NAME")
        .or_else(|| env::var_os("YAZELIX_ZELLIJ_SESSION_NAME"))
        .filter(|value| !value.is_empty())
        .is_some();
    if !in_session {
        return None;
    }
    let state_dir = env::var_os("YAZELIX_STATE_DIR").filter(|value| !value.is_empty())?;
    let path = PathBuf::from(state_dir).join("zellij/config.kdl");
    path.is_file().then_some(path)
}

pub(crate) fn patch_zellij_field_in_text(
    text: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<String> {
    let token = zellij_field_token(field_path)?;
    let assignment = render_zellij_field_assignment(field_path, value)?;
    let mut out = String::with_capacity(text.len() + assignment.len());
    let mut replaced = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let indent_len = line.len() - trimmed.len();
        let line_token = trimmed
            .split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
            .next()
            .unwrap_or("");
        let is_block = trimmed.contains('{');
        if !replaced && line_token == token && !is_block {
            out.push_str(&line[..indent_len]);
            out.push_str(&assignment);
            out.push('\n');
            replaced = true;
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !replaced {
        if !out.ends_with('\n') && !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&assignment);
        out.push('\n');
    }
    Ok(out)
}

fn zellij_field_token(field_path: &str) -> Result<&'static str> {
    match field_path {
        "pane_frames" => Ok("pane_frames"),
        "mouse_mode" => Ok("mouse_mode"),
        "scroll_buffer_size" => Ok("scroll_buffer_size"),
        "copy_on_select" => Ok("copy_on_select"),
        "copy_clipboard" => Ok("copy_clipboard"),
        "styled_underlines" => Ok("styled_underlines"),
        "show_startup_tips" => Ok("show_startup_tips"),
        "ui.pane_frames.rounded_corners" => Ok("rounded_corners"),
        _ => Err(error(format!("unknown Zellij config path: {field_path}"))),
    }
}

fn render_zellij_field_assignment(field_path: &str, value: &JsonValue) -> Result<String> {
    let token = zellij_field_token(field_path)?;
    match field_path {
        "scroll_buffer_size" => Ok(format!("{token} {}", json_positive_i64(field_path, value)?)),
        "copy_clipboard" => {
            let choice = zellij_field(field_path)
                .expect("known field")
                .json_choice(value)?;
            Ok(format!("{token} {}", kdl_string(choice)))
        }
        _ => Ok(format!(
            "{token} {}",
            kdl_bool(json_bool(field_path, value)?)
        )),
    }
}
pub(crate) fn parse_zellij_sidecar(raw: &str) -> (ZellijSidecar, Vec<ConfigUiDiagnostic>) {
    let mut config = ZellijSidecar::default();
    let mut diagnostics = Vec::new();
    let mut stack: Vec<&str> = Vec::new();

    for (index, raw_line) in raw.lines().enumerate() {
        let line_number = index + 1;
        let mut line = raw_line
            .split_once("//")
            .map_or(raw_line, |(content, _)| content)
            .trim();
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
        let Some(token) = line
            .split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
            .next()
            .filter(|token| !token.is_empty())
        else {
            continue;
        };
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
    if let Err(error) = set_zellij_field_value(config, spec.path, &value) {
        diagnostics.push(zellij_diagnostic(
            line_number,
            format!("invalid Zellij value for `{}`", spec.path),
            error.to_string(),
        ));
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
pub(crate) fn zellij_field_value(config: &ZellijSidecar, path: &str) -> JsonValue {
    match path {
        "pane_frames" => json!(config.pane_frames),
        "mouse_mode" => json!(config.mouse_mode),
        "scroll_buffer_size" => json!(config.scroll_buffer_size),
        "copy_on_select" => json!(config.copy_on_select),
        "copy_clipboard" => json!(config.copy_clipboard),
        "styled_underlines" => json!(config.styled_underlines),
        "show_startup_tips" => json!(config.show_startup_tips),
        "ui.pane_frames.rounded_corners" => json!(config.rounded_corners),
        _ => JsonValue::Null,
    }
}
fn set_zellij_field_value(config: &mut ZellijSidecar, path: &str, value: &JsonValue) -> Result<()> {
    match path {
        "pane_frames" => config.pane_frames = json_bool(path, value)?,
        "mouse_mode" => config.mouse_mode = json_bool(path, value)?,
        "scroll_buffer_size" => config.scroll_buffer_size = json_positive_i64(path, value)?,
        "copy_on_select" => config.copy_on_select = json_bool(path, value)?,
        "copy_clipboard" => {
            config.copy_clipboard = zellij_field(path)
                .expect("known field")
                .json_choice(value)?
                .to_string();
        }
        "styled_underlines" => config.styled_underlines = json_bool(path, value)?,
        "show_startup_tips" => config.show_startup_tips = json_bool(path, value)?,
        "ui.pane_frames.rounded_corners" => config.rounded_corners = json_bool(path, value)?,
        _ => return Err(error(format!("unknown Zellij config path: {path}"))),
    }
    Ok(())
}
pub(crate) fn render_zellij_sidecar(config: &ZellijSidecar) -> String {
    format!(
        "\
pane_frames {}
mouse_mode {}
scroll_buffer_size {}
copy_on_select {}
copy_clipboard \"{}\"
styled_underlines {}
show_startup_tips {}

ui {{
    pane_frames {{
        rounded_corners {}
    }}
}}
",
        kdl_bool(config.pane_frames),
        kdl_bool(config.mouse_mode),
        config.scroll_buffer_size,
        kdl_bool(config.copy_on_select),
        config.copy_clipboard,
        kdl_bool(config.styled_underlines),
        kdl_bool(config.show_startup_tips),
        kdl_bool(config.rounded_corners),
    )
}
