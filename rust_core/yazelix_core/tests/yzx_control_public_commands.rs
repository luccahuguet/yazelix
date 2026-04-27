// Test lane: default

use tempfile::TempDir;

mod support;

use support::commands::{apply_managed_config_env, yzx_control_command};
use support::envelopes::stdout_text;
use support::fixtures::managed_config_fixture;

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

// Defends: the Rust-owned `yzx onboard` command exposes a non-interactive help path without entering prompt mode.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_onboard_help_prints_prompt_contract() {
    let output = yzx_control_command()
        .arg("onboard")
        .arg("--help")
        .output()
        .unwrap();
    let stdout = stdout_text(output);

    assert!(stdout.contains("Generate a focused first-run Yazelix config"));
    assert!(stdout.contains("yzx onboard [--force] [--dry-run]"));
    assert!(stdout.contains("--force"));
    assert!(stdout.contains("--dry-run"));
}

// Defends: `yzx edit cursors --print` resolves the user-owned Ghostty cursor sidecar without launching an editor.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_edit_cursors_prints_cursor_sidecar_path() {
    let fixture = managed_config_fixture("");
    let expected_path = fixture
        .config_dir
        .join("user_configs")
        .join("yazelix_cursors.toml");
    let mut command = yzx_control_command();
    apply_managed_config_env(&mut command, &fixture)
        .arg("edit")
        .arg("cursors")
        .arg("--print");

    let stdout = stdout_text(command.output().unwrap());

    assert_eq!(stdout, format!("{}\n", expected_path.display()));
    assert!(expected_path.exists());
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
