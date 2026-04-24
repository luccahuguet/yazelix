use crossterm::style::Stylize;
use std::io::IsTerminal;

pub fn colors_enabled() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var_os("FORCE_COLOR").is_some() {
        return true;
    }
    std::io::stdout().is_terminal()
}

pub fn section_title(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.yellow().bold())
    } else {
        text.to_string()
    }
}

pub fn label(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.dark_yellow())
    } else {
        text.to_string()
    }
}

pub fn accent(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.cyan().bold())
    } else {
        text.to_string()
    }
}

pub fn muted(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.cyan())
    } else {
        text.to_string()
    }
}

pub fn success(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.green().bold())
    } else {
        text.to_string()
    }
}

pub fn warning(text: &str, color: bool) -> String {
    if color {
        format!("{}", text.yellow().bold())
    } else {
        text.to_string()
    }
}
