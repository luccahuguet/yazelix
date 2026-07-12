use crate::bridge::{CoreError, ErrorClass};
use std::fs;
use std::path::Path;
use yazelix_terminal_support::terminal_support;

pub const SESSION_TERMINAL_ENV: &str = "YAZELIX_SESSION_TERMINAL";
pub const UNKNOWN_SESSION_TERMINAL: &str = "unknown";

/// Launchable terminal ids in launch-preference order. Sourced from the
/// `yazelix_terminal_support` child (single source of truth); the former
/// hand-maintained `SUPPORTED_TERMINALS`/`KNOWN_SESSION_TERMINALS` consts and
/// per-terminal match tables have been removed.
pub fn supported_terminals() -> &'static [String] {
    terminal_support().supported_terminals()
}

pub fn is_supported(terminal: &str) -> bool {
    terminal_support().is_supported(terminal)
}

/// Packaged default terminal id, sourced from the `yazelix_terminal_support`
/// child (single source of truth). Use this instead of hardcoding a terminal
/// name in user-facing remediation copy.
pub fn default_terminal() -> &'static str {
    terminal_support().default_terminal()
}

pub fn normalize_terminal_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() || !terminal_support().is_supported(&trimmed) {
        return None;
    }
    Some(trimmed)
}

fn normalize_session_terminal_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim().to_ascii_lowercase();
    if trimmed.is_empty() || !terminal_support().is_session_known(&trimmed) {
        return None;
    }
    Some(trimmed)
}

pub fn terminal_command_name(terminal: &str) -> &str {
    terminal
}

pub fn terminal_display_name(terminal: &str) -> String {
    terminal_support().display_label(terminal)
}

pub fn terminal_desktop_label(terminal: &str) -> String {
    terminal_display_name(terminal)
}

pub fn terminal_desktop_id_suffix(terminal: &str) -> String {
    terminal_support().desktop_suffix(terminal)
}

pub fn terminal_desktop_entry_id(terminal: &str) -> String {
    terminal_support().desktop_entry_id(terminal)
}

pub fn terminal_startup_wm_class(terminal: &str) -> String {
    terminal_support().startup_wm_class(terminal)
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
                "Could not read Yazelix packaged terminal metadata at {}.",
                runtime_variant_path.display()
            ),
            "Reinstall Yazelix so the runtime exposes its packaged terminal metadata.",
            runtime_variant_path.to_string_lossy(),
            source,
        )
    })?;

    normalize_terminal_id(&raw).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "unsupported_terminal_variant",
            format!(
                "Unsupported Yazelix packaged terminal metadata: {}.",
                raw.trim()
            ),
            format!(
                "Reinstall Yazelix with the supported packaged terminal: {}.",
                supported_terminals().join(", ")
            ),
            serde_json::json!({
                "runtime_variant_path": runtime_variant_path,
                "runtime_variant": raw.trim(),
                "supported_terminals": supported_terminals(),
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
    let support = terminal_support();

    for key in support.session_id_env_keys() {
        if let Some(terminal) =
            get_env(key.as_str()).and_then(|value| normalize_session_terminal_id(&value))
        {
            return Some(terminal);
        }
    }

    if let Some(terminal) =
        get_env("TERM_PROGRAM").and_then(|value| support.terminal_for_term_program(&value))
    {
        return Some(terminal);
    }

    for (marker, terminal) in support.env_marker_probes() {
        if get_env(marker.as_str())
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
        {
            return Some(terminal);
        }
    }

    get_env("TERM").and_then(|value| support.terminal_for_term(&value))
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

    // Defends: managed launches and current-terminal enter sessions report the actual host terminal instead of the packaged runtime label.
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
