use crate::bridge::{CoreError, ErrorClass};
use std::fs;
use std::path::Path;

pub const SUPPORTED_TERMINALS: &[&str] =
    &["ghostty", "yzxterm", "rio", "wezterm", "ratty", "kitty"];

pub fn normalize_terminal_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() || !SUPPORTED_TERMINALS.contains(&trimmed.as_str()) {
        return None;
    }
    Some(trimmed)
}

pub fn terminal_command_name(terminal: &str) -> &str {
    match terminal {
        "yzxterm" => "yazelix-terminal-desktop",
        other => other,
    }
}

pub fn terminal_display_name(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "rio" => "Rio".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "yzxterm" => "Yazelix Terminal".to_string(),
        "ratty" => "Ratty".to_string(),
        "kitty" => "Kitty".to_string(),
        other => other.to_string(),
    }
}

pub fn terminal_desktop_label(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "rio" => "Rio".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "yzxterm" => "yzxterm".to_string(),
        "ratty" => "Ratty".to_string(),
        "kitty" => "Kitty".to_string(),
        other => other.to_string(),
    }
}

pub fn terminal_desktop_id_suffix(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "rio" => "Rio".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "yzxterm" => "Yzxterm".to_string(),
        "ratty" => "Ratty".to_string(),
        "kitty" => "Kitty".to_string(),
        other => other.to_string(),
    }
}

pub fn terminal_desktop_entry_id(terminal: &str) -> String {
    format!(
        "com.yazelix.Yazelix.{}",
        terminal_desktop_id_suffix(terminal)
    )
}

pub fn terminal_desktop_entry_file_name(terminal: &str) -> String {
    format!("{}.desktop", terminal_desktop_entry_id(terminal))
}

pub fn terminal_desktop_entry_name(terminal: &str) -> String {
    format!("Yazelix - {}", terminal_desktop_label(terminal))
}

pub fn active_terminal_from_runtime_dir(runtime_dir: &Path) -> Result<String, CoreError> {
    let runtime_variant_path = runtime_dir.join("runtime_variant");
    let raw = fs::read_to_string(&runtime_variant_path).map_err(|source| {
        CoreError::io(
            "read_runtime_variant",
            format!(
                "Could not read Yazelix terminal variant metadata at {}.",
                runtime_variant_path.display()
            ),
            "Reinstall Yazelix so the runtime exposes its selected terminal variant.",
            runtime_variant_path.to_string_lossy(),
            source,
        )
    })?;

    normalize_terminal_id(&raw).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "unsupported_terminal_variant",
            format!(
                "Unsupported Yazelix terminal variant metadata: {}.",
                raw.trim()
            ),
            format!(
                "Reinstall Yazelix with one supported terminal variant: {}.",
                SUPPORTED_TERMINALS.join(", ")
            ),
            serde_json::json!({
                "runtime_variant_path": runtime_variant_path,
                "runtime_variant": raw.trim(),
                "supported_terminals": SUPPORTED_TERMINALS,
            }),
        )
    })
}
