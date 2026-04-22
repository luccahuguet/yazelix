// Test lane: default

use assert_cmd::Command;
use tempfile::TempDir;

// Defends: the Rust-owned `yzx why` leaf keeps the existing elevator-pitch copy instead of drifting through wrapper churn.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn yzx_control_why_prints_elevator_pitch() {
    let output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("why")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Yazelix is a reproducible terminal IDE"));
    assert!(stdout.contains("Zero"));
    assert!(stdout.contains("Get everything running in <10 minutes."));
}

// Defends: the Rust-owned `yzx sponsor` leaf still falls back to printing the sponsor URL when no opener is available.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_sponsor_falls_back_to_printed_url_without_openers() {
    let empty_path = TempDir::new().unwrap();
    let output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("sponsor")
        .env("PATH", empty_path.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Support Yazelix:"));
    assert!(stdout.contains("https://github.com/sponsors/luccahuguet"));
    assert!(!stdout.contains("Opened sponsor page."));
}

// Defends: the Rust-owned `yzx keys` root keeps the sectioned discoverability output instead of collapsing into a flat dump.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_keys_root_preserves_discoverability_sections() {
    let output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Yazelix keybindings"));
    assert!(stdout.contains("Workspace actions"));
    assert!(stdout.contains("Command access"));
    assert!(stdout.contains("yzx keys yazi"));
    assert!(stdout.contains("yzx keys hx"));
    assert!(stdout.contains("yzx keys nu"));
}

// Defends: the Rust-owned `yzx keys` leaves preserve alias parity and tool-specific guidance instead of routing every leaf to the same generic output.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_keys_aliases_and_leaf_views_preserve_guidance() {
    let root_output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .output()
        .unwrap();
    let yzx_output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .arg("yzx")
        .output()
        .unwrap();
    let hx_output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .arg("hx")
        .output()
        .unwrap();
    let helix_output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .arg("helix")
        .output()
        .unwrap();
    let nu_output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .arg("nu")
        .output()
        .unwrap();
    let nushell_output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .arg("nushell")
        .output()
        .unwrap();
    let yazi_output = Command::cargo_bin("yzx_control")
        .unwrap()
        .arg("keys")
        .arg("yazi")
        .output()
        .unwrap();

    assert!(root_output.status.success());
    assert!(yzx_output.status.success());
    assert!(hx_output.status.success());
    assert!(helix_output.status.success());
    assert!(nu_output.status.success());
    assert!(nushell_output.status.success());
    assert!(yazi_output.status.success());

    let root_stdout = String::from_utf8(root_output.stdout).unwrap();
    let yzx_stdout = String::from_utf8(yzx_output.stdout).unwrap();
    let hx_stdout = String::from_utf8(hx_output.stdout).unwrap();
    let helix_stdout = String::from_utf8(helix_output.stdout).unwrap();
    let nu_stdout = String::from_utf8(nu_output.stdout).unwrap();
    let nushell_stdout = String::from_utf8(nushell_output.stdout).unwrap();
    let yazi_stdout = String::from_utf8(yazi_output.stdout).unwrap();

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
