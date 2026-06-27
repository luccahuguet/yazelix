use crate::bridge::{CoreError, ErrorClass};
use std::fs;
use std::path::Path;

pub const SUPPORTED_TERMINALS: &[&str] = &["mars"];
const KNOWN_SESSION_TERMINALS: &[&str] = &[
    "ghostty", "mars", "rio", "wezterm", "ratty", "kitty", "foot",
];
pub const SESSION_TERMINAL_ENV: &str = "YAZELIX_SESSION_TERMINAL";
pub const UNKNOWN_SESSION_TERMINAL: &str = "unknown";

pub fn normalize_terminal_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() || !SUPPORTED_TERMINALS.contains(&trimmed.as_str()) {
        return None;
    }
    Some(trimmed)
}

fn normalize_session_terminal_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() || !KNOWN_SESSION_TERMINALS.contains(&trimmed.as_str()) {
        return None;
    }
    Some(trimmed)
}

pub fn terminal_command_name(terminal: &str) -> &str {
    terminal
}

pub fn terminal_display_name(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "rio" => "Rio".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "mars" => "Mars".to_string(),
        "ratty" => "Ratty".to_string(),
        "kitty" => "Kitty".to_string(),
        "foot" => "Foot".to_string(),
        other => other.to_string(),
    }
}

pub fn terminal_desktop_label(terminal: &str) -> String {
    terminal_display_name(terminal)
}

pub fn terminal_desktop_id_suffix(terminal: &str) -> String {
    match terminal {
        "ghostty" => "Ghostty".to_string(),
        "rio" => "Rio".to_string(),
        "wezterm" => "WezTerm".to_string(),
        "mars" => "Mars".to_string(),
        "ratty" => "Ratty".to_string(),
        "kitty" => "Kitty".to_string(),
        "foot" => "Foot".to_string(),
        other => other.to_string(),
    }
}

pub fn terminal_desktop_entry_id(terminal: &str) -> String {
    format!(
        "com.yazelix.Yazelix.{}",
        terminal_desktop_id_suffix(terminal)
    )
}

pub fn terminal_startup_wm_class(terminal: &str) -> String {
    match terminal {
        "mars" => terminal_desktop_entry_id(terminal),
        _ => "com.yazelix.Yazelix".to_string(),
    }
}

pub fn terminal_desktop_entry_file_name(terminal: &str) -> String {
    format!("{}.desktop", terminal_desktop_entry_id(terminal))
}

pub fn terminal_desktop_entry_name(terminal: &str) -> String {
    format!("New Yazelix - {}", terminal_desktop_label(terminal))
}

pub fn terminal_window_title(terminal: &str, session_name: Option<&str>) -> String {
    let base = format!("Yazelix - {}", terminal_display_name(terminal));
    match session_name
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .filter(|name| !name.eq_ignore_ascii_case("unknown"))
    {
        Some(name) => format!("{base} - {name}"),
        None => base,
    }
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
                "Reinstall Yazelix with the supported packaged terminal variant: {}.",
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

pub fn current_session_terminal_label_from_env() -> String {
    detect_session_terminal_from_env(|key| std::env::var(key).ok())
        .unwrap_or_else(|| UNKNOWN_SESSION_TERMINAL.to_string())
}

pub(crate) fn detect_session_terminal_from_env<F>(mut get_env: F) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    for key in [SESSION_TERMINAL_ENV, "MARS"] {
        if let Some(terminal) = get_env(key).and_then(|value| normalize_session_terminal_id(&value))
        {
            return Some(terminal);
        }
    }

    if let Some(terminal) = get_env("TERM_PROGRAM").and_then(|value| terminal_program_id(&value)) {
        return Some(terminal);
    }

    for (key, terminal) in [
        ("GHOSTTY_RESOURCES_DIR", "ghostty"),
        ("GHOSTTY_BIN_DIR", "ghostty"),
        ("WEZTERM_EXECUTABLE", "wezterm"),
        ("WEZTERM_PANE", "wezterm"),
        ("KITTY_WINDOW_ID", "kitty"),
        ("KITTY_PID", "kitty"),
    ] {
        if get_env(key)
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return Some(terminal.to_string());
        }
    }

    get_env("TERM").and_then(|value| terminal_term_id(&value))
}

fn terminal_program_id(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "ghostty" => Some("ghostty".to_string()),
        "mars" | "mars-terminal" | "mars_terminal" => Some("mars".to_string()),
        "rio" => Some("rio".to_string()),
        "wezterm" | "wezterm-gui" => Some("wezterm".to_string()),
        "ratty" => Some("ratty".to_string()),
        "kitty" => Some("kitty".to_string()),
        "foot" => Some("foot".to_string()),
        _ => None,
    }
}

fn terminal_term_id(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "xterm-ghostty" | "ghostty" => Some("ghostty".to_string()),
        "mars" | "xterm-mars" => Some("mars".to_string()),
        "rio" | "xterm-rio" => Some("rio".to_string()),
        "wezterm" | "xterm-wezterm" => Some("wezterm".to_string()),
        "ratty" | "xterm-ratty" => Some("ratty".to_string()),
        "xterm-kitty" | "kitty" => Some("kitty".to_string()),
        value if value.starts_with("foot") => Some("foot".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;

    // Defends: launcher labels describe the action while window titles describe the running terminal state.
    #[test]
    fn desktop_entry_name_and_window_title_are_distinct() {
        assert_eq!(
            terminal_desktop_entry_name("ghostty"),
            "New Yazelix - Ghostty"
        );
        assert_eq!(terminal_window_title("ghostty", None), "Yazelix - Ghostty");
        assert_eq!(
            terminal_window_title("ghostty", Some("work")),
            "Yazelix - Ghostty - work"
        );
        assert_eq!(
            terminal_window_title("rio", Some("work")),
            "Yazelix - Rio - work"
        );
        assert_eq!(terminal_startup_wm_class("ghostty"), "com.yazelix.Yazelix");
    }

    // Defends: managed launches and current-terminal enter sessions report the actual host terminal instead of the configured runtime variant.
    #[test]
    fn detects_session_terminal_from_explicit_and_common_host_env() {
        let lookup = |pairs: &[(&str, &str)], key: &str| {
            pairs
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| (*value).to_string())
        };

        assert_eq!(
            detect_session_terminal_from_env(|key| lookup(
                &[("MARS", "ghostty"), ("TERM_PROGRAM", "WezTerm")],
                key
            )),
            Some("ghostty".to_string())
        );
        assert_eq!(
            detect_session_terminal_from_env(|key| lookup(&[("TERM_PROGRAM", "WezTerm")], key)),
            Some("wezterm".to_string())
        );
        assert_eq!(
            detect_session_terminal_from_env(|key| lookup(&[("TERM_PROGRAM", "mars")], key)),
            Some("mars".to_string())
        );
        assert_eq!(
            detect_session_terminal_from_env(|key| lookup(
                &[("TERM", "xterm-256color"), ("KITTY_WINDOW_ID", "4")],
                key
            )),
            Some("kitty".to_string())
        );
        assert_eq!(
            detect_session_terminal_from_env(|key| lookup(&[("TERM", "xterm-256color")], key)),
            None
        );
    }
}
