// Test lane: default
//! `yzx why` and `yzx sponsor` implemented in Rust for `yzx_control`.

use crate::bridge::CoreError;
use std::process::Command;

const SPONSOR_URL: &str = "https://github.com/sponsors/luccahuguet";
const WHY_LINES: &[&str] = &[
    "Yazelix is a reproducible terminal IDE (Yazi + Zellij + Helix) with:",
    "• Zero‑conflict keybindings, zjstatus, smooth Yazi↔editor flows",
    "• Top terminals (Ghostty/WezTerm/Kitty/Alacritty) and shells (Bash/Zsh/Fish/Nushell)",
    "• One‑file config (Nix) with sane defaults and curated packs",
    "• Remote‑ready over SSH; same superterminal on barebones hosts",
    "• Git and tooling preconfigured (lazygit, starship, zoxide, carapace)",
    "Get everything running in <10 minutes. No extra deps, only Nix.",
    "Install once, get the same environment everywhere.",
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct LeafHelpArgs {
    help: bool,
}

pub fn run_yzx_why(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_leaf_help_args(args, "yzx why")?;
    if parsed.help {
        print_why_help();
        return Ok(0);
    }

    for line in WHY_LINES {
        println!("{line}");
    }
    Ok(0)
}

pub fn run_yzx_sponsor(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_leaf_help_args(args, "yzx sponsor")?;
    if parsed.help {
        print_sponsor_help();
        return Ok(0);
    }

    if open_support_url(SPONSOR_URL) {
        println!("Opened sponsor page.");
    } else {
        println!("Support Yazelix:");
        println!("{SPONSOR_URL}");
    }

    Ok(0)
}

fn parse_leaf_help_args(args: &[String], route: &str) -> Result<LeafHelpArgs, CoreError> {
    let mut parsed = LeafHelpArgs::default();

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" | "help" => parsed.help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for {route}: {other}. Try `{route} --help`."
                )));
            }
        }
    }

    Ok(parsed)
}

fn print_why_help() {
    println!("Show the Yazelix elevator pitch");
    println!();
    println!("Usage:");
    println!("  yzx why");
}

fn print_sponsor_help() {
    println!("Open the Yazelix sponsor page or print its URL");
    println!();
    println!("Usage:");
    println!("  yzx sponsor");
}

fn open_support_url(url: &str) -> bool {
    open_support_url_with(url, try_open_url_with_command)
}

fn open_support_url_with(url: &str, mut try_open: impl FnMut(&str, &str) -> bool) -> bool {
    for command in ["xdg-open", "open"] {
        if try_open(command, url) {
            return true;
        }
    }

    false
}

fn try_open_url_with_command(command: &str, url: &str) -> bool {
    Command::new(command)
        .arg(url)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the Rust support leaf parser only accepts help flags and rejects accidental extra argv for the public `yzx why` and `yzx sponsor` routes.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parses_leaf_help_flags() {
        assert_eq!(
            parse_leaf_help_args(&["--help".into()], "yzx why").unwrap(),
            LeafHelpArgs { help: true }
        );
        assert_eq!(
            parse_leaf_help_args(&["help".into()], "yzx sponsor").unwrap(),
            LeafHelpArgs { help: true }
        );
        assert!(parse_leaf_help_args(&["extra".into()], "yzx why").is_err());
    }

    // Regression: sponsor opening should stop at the first successful opener instead of probing extra commands after success.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn sponsor_opening_stops_at_first_successful_command() {
        let mut attempts = Vec::new();
        let opened = open_support_url_with(SPONSOR_URL, |command, url| {
            attempts.push((command.to_string(), url.to_string()));
            command == "xdg-open"
        });

        assert!(opened);
        assert_eq!(
            attempts,
            vec![("xdg-open".to_string(), SPONSOR_URL.to_string())]
        );
    }

    // Regression: sponsor opening must try the macOS `open` fallback when `xdg-open` is unavailable or fails, then report failure only if neither opener works.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn sponsor_opening_tries_fallback_opener_and_reports_failure_if_needed() {
        let mut attempts = Vec::new();
        let opened = open_support_url_with(SPONSOR_URL, |command, url| {
            attempts.push((command.to_string(), url.to_string()));
            false
        });

        assert!(!opened);
        assert_eq!(
            attempts,
            vec![
                ("xdg-open".to_string(), SPONSOR_URL.to_string()),
                ("open".to_string(), SPONSOR_URL.to_string()),
            ]
        );
    }
}
