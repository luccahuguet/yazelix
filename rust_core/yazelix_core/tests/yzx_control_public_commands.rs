// Test lane: default

use tempfile::TempDir;

mod support;

use support::commands::yzx_control_command;
use support::envelopes::stdout_text;

// Defends: the Rust-owned `yzx why` leaf keeps the existing elevator-pitch copy instead of drifting through wrapper churn.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn yzx_control_why_prints_elevator_pitch() {
    let output = yzx_control_command().arg("why").output().unwrap();
    let stdout = stdout_text(output);
    assert!(stdout.contains("Yazelix is a reproducible terminal IDE"));
    assert!(stdout.contains("Zero"));
    assert!(stdout.contains("Get everything running in <10 minutes."));
}

// Defends: the Rust-owned `yzx sponsor` leaf still falls back to printing the sponsor URL when no opener is available.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_sponsor_falls_back_to_printed_url_without_openers() {
    let empty_path = TempDir::new().unwrap();
    let output = yzx_control_command()
        .arg("sponsor")
        .env("PATH", empty_path.path())
        .output()
        .unwrap();

    let stdout = stdout_text(output);
    assert!(stdout.contains("Support Yazelix:"));
    assert!(stdout.contains("https://github.com/sponsors/luccahuguet"));
    assert!(!stdout.contains("Opened sponsor page."));
}

// Defends: the Rust-owned `yzx keys` root keeps the sectioned discoverability output instead of collapsing into a flat dump.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_keys_root_preserves_discoverability_sections() {
    let output = yzx_control_command().arg("keys").output().unwrap();
    let stdout = stdout_text(output);
    assert!(stdout.starts_with("Workspace actions\n"));
    assert!(stdout.contains("Workspace actions"));
    assert!(stdout.contains("Command access"));
    assert!(stdout.contains("Keybinding"));
    assert!(stdout.contains("yzx keys yazi"));
    assert!(stdout.contains("yzx keys hx"));
    assert!(stdout.contains("yzx keys nu"));
    assert!(!stdout.contains('\u{1b}'));
    assert!(!stdout.contains("Yazelix keybindings"));
    assert!(!stdout.contains("╭"));
    assert!(!stdout.contains("│"));
}

// Defends: the Rust-owned `yzx keys` leaves preserve alias parity and tool-specific guidance instead of routing every leaf to the same generic output.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_keys_aliases_and_leaf_views_preserve_guidance() {
    let root_stdout = stdout_text(yzx_control_command().arg("keys").output().unwrap());
    let yzx_stdout = stdout_text(
        yzx_control_command()
            .arg("keys")
            .arg("yzx")
            .output()
            .unwrap(),
    );
    let hx_stdout = stdout_text(
        yzx_control_command()
            .arg("keys")
            .arg("hx")
            .output()
            .unwrap(),
    );
    let helix_stdout = stdout_text(
        yzx_control_command()
            .arg("keys")
            .arg("helix")
            .output()
            .unwrap(),
    );
    let nu_stdout = stdout_text(
        yzx_control_command()
            .arg("keys")
            .arg("nu")
            .output()
            .unwrap(),
    );
    let nushell_stdout = stdout_text(
        yzx_control_command()
            .arg("keys")
            .arg("nushell")
            .output()
            .unwrap(),
    );
    let yazi_stdout = stdout_text(
        yzx_control_command()
            .arg("keys")
            .arg("yazi")
            .output()
            .unwrap(),
    );

    assert_eq!(root_stdout, yzx_stdout);
    assert_eq!(hx_stdout, helix_stdout);
    assert_eq!(nu_stdout, nushell_stdout);
    assert!(yazi_stdout.contains("Yazi keybindings"));
    assert!(yazi_stdout.contains("Alt+p"));
    assert!(yazi_stdout.contains("Focus the Yazi pane and press `~`"));
    assert!(hx_stdout.contains("Helix keybindings"));
    assert!(hx_stdout.contains("Press `<space>?`"));
    assert!(nu_stdout.contains("Nushell keybindings"));
    assert!(nu_stdout.contains("history"));
}
