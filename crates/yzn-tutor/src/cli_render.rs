use std::io::IsTerminal;

pub(crate) fn colors_enabled() -> bool {
    colors_enabled_from_inputs(
        std::env::var_os("NO_COLOR").is_some(),
        std::env::var_os("FORCE_COLOR").is_some(),
        std::env::var_os("TERM")
            .as_deref()
            .and_then(std::ffi::OsStr::to_str),
        std::io::stdout().is_terminal(),
    )
}

fn colors_enabled_from_inputs(
    no_color: bool,
    force_color: bool,
    term: Option<&str>,
    stdout_is_terminal: bool,
) -> bool {
    if no_color {
        return false;
    }
    if force_color {
        return true;
    }
    term != Some("dumb") && stdout_is_terminal
}

pub(crate) fn accent(text: &str, color: bool) -> String {
    styled(text, "36", true, color)
}

pub(crate) fn section_title(text: &str, color: bool) -> String {
    styled(text, "33", true, color)
}

pub(crate) fn label(text: &str, color: bool) -> String {
    styled(text, "33", false, color)
}

pub(crate) fn muted(text: &str, color: bool) -> String {
    styled(text, "36", false, color)
}

pub(crate) fn inline_code(text: &str, color: bool) -> String {
    styled(text, "35", true, color)
}

fn styled(text: &str, color_code: &str, bold: bool, color: bool) -> String {
    if !color {
        return text.to_string();
    }
    let weight = if bold { "1" } else { "22" };
    format!("\x1b[{weight};{color_code}m{text}\x1b[0m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_decision_honors_no_color_force_color_and_term_dumb() {
        assert!(!colors_enabled_from_inputs(
            true,
            true,
            Some("xterm-256color"),
            true
        ));
        assert!(colors_enabled_from_inputs(false, true, Some("dumb"), false));
        assert!(!colors_enabled_from_inputs(
            false,
            false,
            Some("dumb"),
            true
        ));
        assert!(colors_enabled_from_inputs(
            false,
            false,
            Some("xterm-256color"),
            true
        ));
        assert!(!colors_enabled_from_inputs(
            false,
            false,
            Some("xterm-256color"),
            false
        ));
    }
}
