use crossterm::{
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
};
use std::io::IsTerminal;

pub fn colors_enabled() -> bool {
    let term = std::env::var_os("TERM");
    colors_enabled_from_inputs(
        std::env::var_os("NO_COLOR").is_some(),
        std::env::var_os("FORCE_COLOR").is_some(),
        term.as_deref().and_then(std::ffi::OsStr::to_str),
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
    if term == Some("dumb") {
        return false;
    }
    stdout_is_terminal
}

pub fn section_title(text: &str, color: bool) -> String {
    if color {
        styled(text, Color::Yellow, true)
    } else {
        text.to_string()
    }
}

pub fn label(text: &str, color: bool) -> String {
    if color {
        styled(text, Color::DarkYellow, false)
    } else {
        text.to_string()
    }
}

pub fn accent(text: &str, color: bool) -> String {
    if color {
        styled(text, Color::Cyan, true)
    } else {
        text.to_string()
    }
}

pub fn muted(text: &str, color: bool) -> String {
    if color {
        styled(text, Color::Cyan, false)
    } else {
        text.to_string()
    }
}

pub fn inline_code(text: &str, color: bool) -> String {
    if color {
        styled(text, Color::Magenta, true)
    } else {
        text.to_string()
    }
}

pub fn success(text: &str, color: bool) -> String {
    if color {
        styled(text, Color::Green, true)
    } else {
        text.to_string()
    }
}

pub fn warning(text: &str, color: bool) -> String {
    if color {
        styled(text, Color::Yellow, true)
    } else {
        text.to_string()
    }
}

fn styled(text: &str, foreground: Color, bold: bool) -> String {
    crossterm::style::force_color_output(true);
    let mut output = Vec::new();
    if bold {
        queue!(output, SetAttribute(Attribute::Bold)).expect("ANSI style writes to memory");
    }
    queue!(
        output,
        SetForegroundColor(foreground),
        Print(text),
        SetAttribute(Attribute::NormalIntensity),
        SetForegroundColor(Color::Reset)
    )
    .expect("ANSI style writes to memory");
    String::from_utf8(output).expect("ANSI style commands emit UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: default
    // Defends: the shared CLI color gate honors explicit plain-output environments before emitting ANSI.
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
