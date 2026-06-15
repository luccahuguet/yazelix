// Test lane: default

use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

mod support;

use support::commands::{yzx_control_bin_path, yzx_control_command_in_fixture};
use support::fixtures::{
    managed_config_fixture, prepend_path, repo_root, write_executable_script,
    write_session_config_snapshot,
};

fn write_default_profile_manifest(fixture: &support::fixtures::ManagedConfigFixture, raw: &str) {
    let manifest_path = fixture.home_dir.join(".nix-profile").join("manifest.json");
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(manifest_path, raw).unwrap();
}

fn copy_dir_recursive(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).unwrap();
        }
    }
}

fn seed_startup_materialization_runtime_assets(fixture: &support::fixtures::ManagedConfigFixture) {
    let repo = repo_root();
    copy_dir_recursive(
        &repo.join("configs").join("zellij"),
        &fixture.runtime_dir.join("configs").join("zellij"),
    );
    copy_dir_recursive(
        &repo.join("configs").join("yazi"),
        &fixture.runtime_dir.join("configs").join("yazi"),
    );
    fs::write(
        fixture
            .runtime_dir
            .join("configs/yazi/yazelix_starship.toml"),
        "[character]\n",
    )
    .unwrap();
    fs::create_dir_all(fixture.runtime_dir.join("configs/zellij/plugins")).unwrap();
    for wasm in [
        "yazelix_pane_orchestrator.wasm",
        "zjstatus.wasm",
        "yzpp.wasm",
    ] {
        fs::write(
            fixture
                .runtime_dir
                .join("configs/zellij/plugins")
                .join(wasm),
            wasm,
        )
        .unwrap();
    }
    write_executable_script(
        &fixture
            .runtime_dir
            .join("libexec")
            .join("yazelix_zellij_bar_widget"),
        "#!/bin/sh\nprintf '%s\\n' '{\"schema_version\":3,\"plugin_block\":\"pane size=1 { plugin location=\\\"file:/tmp/zjstatus.wasm\\\" }\"}'\n",
    );
    fs::create_dir_all(fixture.runtime_dir.join("shells").join("posix")).unwrap();
}

// Regression: workspace startup scrubs inherited GTK/GIO loader variables so host GUI apps do not load incompatible Nix modules.
#[test]
fn start_yazelix_scrubs_gui_loader_env_before_control_handoff() {
    let repo = repo_root();
    let temp = tempfile::tempdir().unwrap();
    let runtime_dir = temp.path().join("runtime");
    let home_dir = temp.path().join("home");
    let posix_dir = runtime_dir.join("shells").join("posix");

    fs::create_dir_all(&posix_dir).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(runtime_dir.join("runtime_features")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();
    fs::write(
        runtime_dir
            .join("runtime_features")
            .join("zellij_kitty_passthrough"),
        "",
    )
    .unwrap();

    write_executable_script(
        &posix_dir.join("start_yazelix.sh"),
        &fs::read_to_string(repo.join("shells/posix/start_yazelix.sh")).unwrap(),
    );
    fs::write(
        posix_dir.join("runtime_env.sh"),
        fs::read_to_string(repo.join("shells/posix/runtime_env.sh")).unwrap(),
    )
    .unwrap();
    write_executable_script(&runtime_dir.join("libexec/hx"), "#!/bin/sh\nexit 0\n");

    let capture = temp.path().join("capture_env.sh");
    write_executable_script(
        &capture,
        r#"#!/bin/sh
set -eu
printf 'argv=%s\n' "${1:-}"
for key in GIO_EXTRA_MODULES GIO_MODULE_DIR GSETTINGS_SCHEMA_DIR GI_TYPELIB_PATH GTK_PATH GTK_EXE_PREFIX GTK_DATA_PREFIX GDK_PIXBUF_MODULE_FILE GDK_PIXBUF_MODULEDIR; do
  eval "value=\${$key-unset}"
  printf '%s=%s\n' "$key" "$value"
done
printf 'YAZELIX_RUNTIME_DIR=%s\n' "$YAZELIX_RUNTIME_DIR"
printf 'YAZELIX_MANAGED_HELIX_BINARY=%s\n' "$YAZELIX_MANAGED_HELIX_BINARY"
printf 'YAZELIX_NU_BIN=%s\n' "${YAZELIX_NU_BIN-unset}"
printf 'YAZI_ZELLIJ_KITTY_PASSTHROUGH=%s\n' "${YAZI_ZELLIJ_KITTY_PASSTHROUGH-unset}"
"#,
    );

    let output = Command::new(posix_dir.join("start_yazelix.sh"))
        .env_clear()
        .env("HOME", &home_dir)
        .env("PATH", "/usr/bin:/bin")
        .env("YAZELIX_YZX_CONTROL_BIN", &capture)
        .env("GIO_EXTRA_MODULES", "/nix/store/bad/lib/gio/modules")
        .env("GIO_MODULE_DIR", "/nix/store/bad/lib/gio/modules")
        .env(
            "GSETTINGS_SCHEMA_DIR",
            "/nix/store/bad/share/gsettings-schemas",
        )
        .env("GI_TYPELIB_PATH", "/nix/store/bad/lib/girepository-1.0")
        .env("GTK_PATH", "/nix/store/bad/lib/gtk")
        .env("GTK_EXE_PREFIX", "/nix/store/bad")
        .env("GTK_DATA_PREFIX", "/nix/store/bad")
        .env("GDK_PIXBUF_MODULE_FILE", "/nix/store/bad/loaders.cache")
        .env("GDK_PIXBUF_MODULEDIR", "/nix/store/bad/loaders")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("argv=enter"));
    for key in [
        "GIO_EXTRA_MODULES",
        "GIO_MODULE_DIR",
        "GSETTINGS_SCHEMA_DIR",
        "GI_TYPELIB_PATH",
        "GTK_PATH",
        "GTK_EXE_PREFIX",
        "GTK_DATA_PREFIX",
        "GDK_PIXBUF_MODULE_FILE",
        "GDK_PIXBUF_MODULEDIR",
    ] {
        assert!(
            stdout.contains(&format!("{key}=unset")),
            "expected {key} to be scrubbed from:\n{stdout}"
        );
    }
    assert!(stdout.contains(&format!(
        "YAZELIX_RUNTIME_DIR={}",
        runtime_dir.to_string_lossy()
    )));
    assert!(stdout.contains(&format!(
        "YAZELIX_MANAGED_HELIX_BINARY={}",
        runtime_dir.join("libexec/hx").to_string_lossy()
    )));
    assert!(stdout.contains("YAZELIX_NU_BIN=unset"));
    assert!(stdout.contains("YAZI_ZELLIJ_KITTY_PASSTHROUGH=unset"));
}

// Regression: the Ghostty launch wrapper must not expose runtime-private libexec helpers such as nix ahead of the host Nix.
#[test]
fn ghostty_wrapper_keeps_runtime_libexec_private_for_host_nix() {
    let repo = repo_root();
    let temp = tempfile::tempdir().unwrap();
    let runtime_dir = temp.path().join("runtime");
    let home_dir = temp.path().join("home");
    let host_bin = home_dir.join("host-bin");
    let posix_dir = runtime_dir.join("shells").join("posix");
    let libexec_dir = runtime_dir.join("libexec");

    fs::create_dir_all(&posix_dir).unwrap();
    fs::create_dir_all(&libexec_dir).unwrap();
    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("bin")).unwrap();
    fs::create_dir_all(&host_bin).unwrap();

    write_executable_script(
        &posix_dir.join("yazelix_ghostty.sh"),
        &fs::read_to_string(repo.join("shells/posix/yazelix_ghostty.sh")).unwrap(),
    );
    let runtime_nix = libexec_dir.join("nix");
    write_executable_script(&runtime_nix, "#!/bin/sh\nprintf 'runtime nix\\n'\n");
    write_executable_script(&host_bin.join("nix"), "#!/bin/sh\nprintf 'host nix\\n'\n");
    let capture = temp.path().join("capture_path.sh");
    write_executable_script(
        &capture,
        r#"#!/bin/sh
set -eu
printf 'PATH=%s\n' "$PATH"
printf 'nix=%s\n' "$(command -v nix)"
"#,
    );

    let output = Command::new(posix_dir.join("yazelix_ghostty.sh"))
        .env_clear()
        .env("PATH", &host_bin)
        .arg(&capture)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        !stdout.contains(&format!("{}", libexec_dir.display())),
        "runtime libexec leaked into PATH:\n{stdout}"
    );
    assert!(stdout.contains(&format!("nix={}", host_bin.join("nix").display())));
}

// Defends: the public Rust-owned `yzx config --path` route still bootstraps the managed config surface and returns its canonical path.
#[test]
fn yzx_control_config_path_bootstraps_missing_managed_config() {
    let fixture = managed_config_fixture("");
    fs::remove_file(&fixture.managed_config).unwrap();
    let settings_path = fixture.config_dir.join("settings.jsonc");

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("config")
        .arg("--path")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(settings_path.is_file());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        settings_path.to_string_lossy()
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.is_empty());
}

// Defends: setup-only launch preflight preserves managed-pane shell UX without invoking the deleted Nushell setup script.
#[test]
fn yzx_enter_setup_only_generates_managed_initializers_and_extern_bridge() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "fish"
"#,
    );
    let plugin_path = fixture
        .runtime_dir
        .join("configs/zellij/plugins/zjstatus.wasm");
    fs::create_dir_all(plugin_path.parent().unwrap()).unwrap();
    fs::write(&plugin_path, "").unwrap();

    let empty_path = fixture.home_dir.join("empty-path");
    fs::create_dir_all(&empty_path).unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", &empty_path)
        .arg("enter")
        .arg("--setup-only")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Setting up Yazelix generated environment files"));
    assert!(stdout.contains("Generated 0 shell initializers"));
    assert!(stdout.contains("Setup complete"));

    assert!(fixture.config_dir.join("shell_nu.nu").exists());
    assert!(fixture.config_dir.join("shell_bash.sh").exists());
    assert!(fixture.config_dir.join("shell_fish.fish").exists());
    assert!(!fixture.config_dir.join("shell_zsh.zsh").exists());

    let generated = fixture.home_dir.join(".local/share/yazelix/initializers");
    assert!(generated.join("nushell/yazelix_init.nu").exists());
    assert!(generated.join("bash/yazelix_init.sh").exists());
    assert!(generated.join("fish/yazelix_init.fish").exists());
    assert!(!generated.join("zsh/yazelix_init.zsh").exists());

    assert!(
        fixture
            .state_dir
            .join("initializers/nushell/yazelix_extern.nu")
            .exists()
    );
    assert!(fixture.state_dir.join("logs").is_dir());
}

// Defends: host-owned xonsh default shell fails before launch when the host does not provide xonsh.
#[test]
fn yzx_enter_setup_only_reports_missing_host_xonsh_default_shell() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "xonsh"
"#,
    );
    let empty_path = fixture.home_dir.join("empty-path");
    fs::create_dir_all(&empty_path).unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", &empty_path)
        .arg("enter")
        .arg("--setup-only")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Configured shell.default_shell is xonsh"));
    assert!(stderr.contains("Install xonsh on the host"));
    assert!(!fixture.config_dir.join("shell_xonsh.xsh").exists());
}

// Defends: selecting host-owned xonsh as the default shell still generates the xonsh hook surfaces.
#[test]
fn yzx_enter_setup_only_accepts_host_xonsh_default_shell() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "xonsh"
"#,
    );
    let plugin_path = fixture
        .runtime_dir
        .join("configs/zellij/plugins/zjstatus.wasm");
    fs::create_dir_all(plugin_path.parent().unwrap()).unwrap();
    fs::write(&plugin_path, "").unwrap();

    let fake_bin = fixture.home_dir.join("fake-bin");
    write_executable_script(&fake_bin.join("xonsh"), "#!/bin/sh\nexit 0\n");

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", &fake_bin)
        .arg("enter")
        .arg("--setup-only")
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Generated 0 shell initializers"));
    assert!(fixture.config_dir.join("shell_xonsh.xsh").exists());

    let generated = fixture.home_dir.join(".local/share/yazelix/initializers");
    assert!(generated.join("xonsh/yazelix_init.xsh").exists());
}

// Regression: Rust startup handoff creates the launch snapshot and recomputes runtime env from helix.external without invoking the deleted Nu bridge.
#[test]
fn yzx_enter_uses_rust_startup_snapshot_env_without_nu_bridge() {
    let fixture = managed_config_fixture(
        r#"[helix]
external = { binary = "/custom/helix/bin/hx", runtime_path = "/custom/helix/runtime" }

[editor]
command = ""
"#,
    );
    seed_startup_materialization_runtime_assets(&fixture);
    let fake_bin = fixture.home_dir.join("fake-bin");
    let workspace = fixture.home_dir.join("workspace");
    let zellij_log = fixture.home_dir.join("zellij-startup.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&workspace).unwrap();

    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"setup\" ] && [ \"$2\" = \"--dump-config\" ]; then\n  exit 0\nfi\nprintf 'argv=%s\\n' \"$*\" > \"{}\"\nprintf 'cwd=%s\\n' \"$(pwd)\" >> \"{}\"\nfor key in YAZELIX_SESSION_CONFIG_PATH YAZELIX_STATUS_BAR_CACHE_PATH YAZELIX_MANAGED_HELIX_BINARY HELIX_RUNTIME EDITOR VISUAL; do\n  eval \"value=\\${{$key-}}\"\n  printf '%s=%s\\n' \"$key\" \"$value\" >> \"{}\"\ndone\n[ -f \"$YAZELIX_SESSION_CONFIG_PATH\" ] && printf 'snapshot_exists=1\\n' >> \"{}\"\n",
            zellij_log.display(),
            zellij_log.display(),
            zellij_log.display(),
            zellij_log.display(),
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("YAZELIX_MANAGED_HELIX_BINARY", "/stale/hx")
        .env("HELIX_RUNTIME", "/stale/runtime")
        .env("EDITOR", "/stale/editor")
        .env("VISUAL", "/stale/editor")
        .env("YAZELIX_STARTUP_PROFILE_SKIP_WELCOME", "true")
        .arg("enter")
        .arg("--with")
        .arg("core.welcome_duration_seconds=0.25")
        .arg("--path")
        .arg(&workspace)
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Welcome screen skipped"), "{stdout}");
    let welcome_logs = fs::read_dir(fixture.state_dir.join("logs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("welcome_"))
        })
        .collect::<Vec<_>>();
    assert_eq!(welcome_logs.len(), 1, "{welcome_logs:?}");
    assert!(
        fs::read_to_string(&welcome_logs[0])
            .unwrap()
            .contains("Welcome to Yazelix v-test")
    );

    let log = fs::read_to_string(zellij_log).unwrap();
    let snapshot_path = log
        .lines()
        .find_map(|line| line.strip_prefix("YAZELIX_SESSION_CONFIG_PATH="))
        .unwrap();
    let snapshot: Value = serde_json::from_str(
        &fs::read_to_string(snapshot_path).expect("startup snapshot should be readable"),
    )
    .unwrap();
    assert_eq!(
        snapshot["normalized_config"]["helix_external"]["binary"],
        "/custom/helix/bin/hx"
    );
    assert_eq!(
        snapshot["normalized_config"]["welcome_duration_seconds"],
        serde_json::json!(0.25)
    );
    let expected_editor = fixture
        .runtime_dir
        .join("shells/posix/yazelix_hx.sh")
        .to_string_lossy()
        .to_string();
    assert!(log.contains("argv=--config-dir "), "{log}");
    assert!(log.contains(" options --default-cwd "), "{log}");
    assert!(
        log.contains(&workspace.to_string_lossy().to_string()),
        "{log}"
    );
    assert!(log.contains("snapshot_exists=1"), "{log}");
    assert!(
        log.contains("YAZELIX_MANAGED_HELIX_BINARY=/custom/helix/bin/hx"),
        "{log}"
    );
    assert!(log.contains("HELIX_RUNTIME=/custom/helix/runtime"), "{log}");
    assert!(log.contains(&format!("EDITOR={expected_editor}")), "{log}");
    assert!(log.contains(&format!("VISUAL={expected_editor}")), "{log}");
    assert!(!log.contains("nushell/scripts"), "{log}");
    assert!(!log.contains("/stale/"), "{log}");
}

// Defends: the public Rust-owned `yzx status --json` surface keeps the typed runtime summary instead of a wrapper-shaped blob.
#[test]
fn yzx_control_status_json_reports_typed_summary() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "nu"
"#,
    );
    let output = yzx_control_command_in_fixture(&fixture)
        .arg("status")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let summary = &report["summary"];
    assert_eq!(report["title"], "Yazelix status");
    assert!(
        summary["config_file"]
            .as_str()
            .unwrap()
            .ends_with("settings.jsonc")
    );
    assert_eq!(summary["default_shell"], "nu");
    assert_eq!(summary["terminals"], serde_json::json!(["ghostty"]));
    assert!(summary["generated_state_repair_needed"].is_boolean());
    assert!(summary["generated_state_materialization_status"].is_string());
    assert_eq!(summary["session_config_snapshot"]["status"], "not_set");
}

// Regression: status stays usable from an older running window whose live config contains newer unsupported fields.
#[test]
fn yzx_control_status_json_reports_config_problem_without_aborting() {
    let fixture = managed_config_fixture(
        r#"[terminal]
ghostty_trail_color = "random"
"#,
    );
    let snapshot = write_session_config_snapshot(
        &fixture,
        &[
            ("default_shell", serde_json::json!("bash")),
            ("terminals", serde_json::json!(["cached-term"])),
        ],
    );
    let snapshot_display = snapshot.to_string_lossy().to_string();

    let output = yzx_control_command_in_fixture(&fixture)
        .env("YAZELIX_SESSION_CONFIG_PATH", snapshot)
        .arg("status")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["summary"]["generated_state_materialization_status"],
        "config_problem"
    );
    assert_eq!(report["summary"]["default_shell"], "bash");
    assert_eq!(
        report["summary"]["terminals"],
        serde_json::json!(["cached-term"])
    );
    assert_eq!(
        report["summary"]["config_diagnostic_report"]["blocking_count"],
        1
    );
    assert_eq!(report["summary"]["session_config_snapshot"]["status"], "ok");
    assert_eq!(
        report["summary"]["session_config_snapshot"]["path"],
        snapshot_display
    );
    assert_eq!(
        report["summary"]["session_config_snapshot"]["source_config_file"],
        fixture.managed_config.to_string_lossy().to_string()
    );
}

// Regression: status reports a bad active snapshot as a readable diagnostic instead of hiding the snapshot problem.
#[test]
fn yzx_control_status_json_reports_bad_session_snapshot_diagnostic() {
    let fixture = managed_config_fixture("");
    let missing_snapshot = fixture
        .state_dir
        .join("sessions/missing/config_snapshot.json");

    let output = yzx_control_command_in_fixture(&fixture)
        .env("YAZELIX_SESSION_CONFIG_PATH", &missing_snapshot)
        .arg("status")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["summary"]["session_config_snapshot"]["status"],
        "error"
    );
    assert_eq!(
        report["summary"]["session_config_snapshot"]["path"],
        missing_snapshot.to_string_lossy().to_string()
    );
    assert_eq!(
        report["summary"]["session_config_snapshot"]["error_code"],
        "session_config_snapshot_read"
    );
    assert!(
        report["summary"]["session_config_snapshot"]["message"]
            .as_str()
            .unwrap()
            .contains("Could not read the Yazelix session config snapshot")
    );
}

// Defends: `yzx inspect --json` is the canonical runtime truth report for diagnostics and agents.
#[test]
fn yzx_control_inspect_json_reports_runtime_truth_without_zellij_session() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "nu"
"#,
    );
    let output = yzx_control_command_in_fixture(&fixture)
        .env(
            "YAZELIX_INVOKED_YZX_PATH",
            "/nix/store/example-yazelix/bin/yzx",
        )
        .arg("inspect")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schema_version"], 1);
    assert_eq!(report["title"], "Yazelix inspect");
    assert_eq!(report["runtime"]["version"], "v-test");
    assert_eq!(report["runtime"]["exists"], true);
    assert_eq!(
        report["runtime"]["invoked_yzx_path"],
        "/nix/store/example-yazelix/bin/yzx"
    );
    assert!(
        report["config"]["file"]
            .as_str()
            .unwrap()
            .ends_with("settings.jsonc")
    );
    assert_eq!(
        report["config"]["session_config_snapshot"]["status"],
        "not_set"
    );
    assert!(report["generated_state"]["repair_needed"].is_boolean());
    assert_eq!(report["session"]["available"], false);
    assert_eq!(report["session"]["reason"], "not_in_zellij");
    assert_eq!(report["install"]["install_owner"], "manual");
    assert!(
        report["tool_versions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["tool"] == "nix" && entry["runtime"].as_str().is_some() })
    );
}

// Regression: inspect remains the diagnostic escape hatch when config validation is what failed.
#[test]
fn yzx_control_inspect_json_embeds_config_problem_without_aborting() {
    let fixture = managed_config_fixture(
        r#"[terminal]
ghostty_trail_color = "random"
"#,
    );
    let snapshot = write_session_config_snapshot(&fixture, &[]);
    let snapshot_display = snapshot.to_string_lossy().to_string();

    let output = yzx_control_command_in_fixture(&fixture)
        .env("YAZELIX_SESSION_CONFIG_PATH", snapshot)
        .arg("inspect")
        .arg("--json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["config"]["status"], "unsupported_config");
    assert_eq!(
        report["generated_state"]["materialization_status"],
        "config_problem"
    );
    assert_eq!(report["config"]["diagnostic_report"]["blocking_count"], 1);
    assert_eq!(report["config"]["session_config_snapshot"]["status"], "ok");
    assert_eq!(
        report["config"]["session_config_snapshot"]["path"],
        snapshot_display
    );
}

// Defends: the Rust-owned `yzx update upstream` route still fails early for Home Manager-owned installs instead of probing the profile path.
#[test]
fn yzx_control_update_upstream_rejects_home_manager_owned_install() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_executable_script(&fake_bin.join("nix"), "#!/bin/sh\nexit 0\n");

    let hm_store_config = fixture
        .home_dir
        .join("hm-store")
        .join("abc-home-manager-files")
        .join("settings.jsonc");
    fs::create_dir_all(hm_store_config.parent().unwrap()).unwrap();
    fs::write(
        &hm_store_config,
        "{\"core\":{\"welcome_style\":\"random\"}}\n",
    )
    .unwrap();
    fs::remove_file(&fixture.managed_config).unwrap();
    std::os::unix::fs::symlink(&hm_store_config, fixture.config_dir.join("settings.jsonc"))
        .unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .arg("update")
        .arg("upstream")
        .output()
        .unwrap();

    assert_ne!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("appears to be Home Manager-owned"));
    assert!(stdout.contains("yzx update home_manager"));
    assert!(stdout.contains("home-manager switch"));
}

// Regression: `yzx update upstream` must allow a plain profile-owned install and report no-op upgrades instead of silently returning.
#[test]
fn yzx_control_update_upstream_accepts_profile_owned_install() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let profile_yzx = fixture
        .home_dir
        .join(".nix-profile")
        .join("bin")
        .join("yzx");
    let upgrade_log = fixture.home_dir.join("nix-upgrade.log");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
    fs::write(&profile_yzx, "#!/bin/sh\nexit 0\n").unwrap();

    write_executable_script(
        &fake_bin.join("nix"),
        &format!(
            "#!/bin/sh
if [ \"$1\" = profile ] && [ \"$2\" = list ] && [ \"$3\" = --json ]; then
  cat <<'EOF'
{{\"elements\":{{\"yazelix\":{{\"storePaths\":[\"{}\"]}}}}}}
EOF
  exit 0
fi
if [ \"$1\" = profile ] && [ \"$2\" = upgrade ] && [ \"$3\" = --refresh ] && [ \"$4\" = yazelix ]; then
  printf '%s\\n' \"$*\" > \"{}\"
  exit 0
fi
echo unexpected nix invocation: \"$*\" >&2
exit 99
",
            fixture.runtime_dir.display(),
            upgrade_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .arg("update")
        .arg("upstream")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Requested update path: default Nix profile."));
    assert!(stdout.contains("nix profile upgrade --refresh yazelix"));
    assert!(stdout.contains("Yazelix is already up to date."));
    assert!(!stdout.contains("appears to be Home Manager-owned"));
    assert_eq!(
        fs::read_to_string(upgrade_log).unwrap(),
        "profile upgrade --refresh yazelix\n"
    );
}

// Regression: `yzx update upstream` must stream Nix progress while the upgrade is still running instead of buffering child output until completion.
#[test]
fn yzx_control_update_upstream_streams_nix_output_before_exit() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let profile_yzx = fixture
        .home_dir
        .join(".nix-profile")
        .join("bin")
        .join("yzx");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
    fs::write(&profile_yzx, "#!/bin/sh\nexit 0\n").unwrap();

    write_executable_script(
        &fake_bin.join("nix"),
        &format!(
            "#!/bin/sh
if [ \"$1\" = profile ] && [ \"$2\" = list ] && [ \"$3\" = --json ]; then
  cat <<'EOF'
{{\"elements\":{{\"yazelix\":{{\"storePaths\":[\"{}\"]}}}}}}
EOF
  exit 0
fi
if [ \"$1\" = profile ] && [ \"$2\" = upgrade ] && [ \"$3\" = --refresh ] && [ \"$4\" = yazelix ]; then
  printf 'fake nix progress\\n'
  sleep 1
  printf 'fake nix done\\n'
  exit 0
fi
echo unexpected nix invocation: \"$*\" >&2
exit 99
",
            fixture.runtime_dir.display()
        ),
    );

    let mut child = Command::new(yzx_control_bin_path())
        .env_clear()
        .env("HOME", &fixture.home_dir)
        .env("PATH", prepend_path(&fake_bin))
        .env("XDG_CONFIG_HOME", fixture.xdg_config_home())
        .env("XDG_DATA_HOME", fixture.xdg_data_home())
        .env("YAZELIX_RUNTIME_DIR", &fixture.runtime_dir)
        .env("YAZELIX_CONFIG_DIR", &fixture.config_dir)
        .env("YAZELIX_STATE_DIR", &fixture.state_dir)
        .arg("update")
        .arg("upstream")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        let bytes = stdout.read_line(&mut line).unwrap();
        assert_ne!(
            bytes, 0,
            "upstream update exited before streaming nix output"
        );
        if line.contains("fake nix progress") {
            break;
        }
    }

    assert!(
        child.try_wait().unwrap().is_none(),
        "upstream update buffered nix output until after the child process exited"
    );
    assert_eq!(child.wait().unwrap().code(), Some(0));
}

// Regression: `yzx home_manager prepare --apply` must remove standalone profile-owned Yazelix entries as part of the takeover flow instead of only archiving files.
#[test]
fn yzx_control_home_manager_prepare_apply_removes_profile_entry_blockers() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let remove_log = fixture.home_dir.join("nix-remove.log");
    fs::create_dir_all(&fake_bin).unwrap();
    write_default_profile_manifest(
        &fixture,
        r#"{"elements":{"yazelix":{"active":true,"storePaths":["/nix/store/test-yazelix"]}},"version":3}"#,
    );
    write_executable_script(
        &fake_bin.join("nix"),
        &format!(
            "#!/bin/sh
if [ \"$1\" = profile ] && [ \"$2\" = remove ] && [ \"$3\" = yazelix ]; then
  printf '%s\\n' \"$*\" > \"{}\"
  exit 0
fi
echo unexpected nix invocation: \"$*\" >&2
exit 99
",
            remove_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .arg("home_manager")
        .arg("prepare")
        .arg("--apply")
        .arg("--yes")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Archived manual-install artifacts for Home Manager takeover"));
    assert!(stdout.contains("Removed standalone default-profile Yazelix entries"));
    assert!(stdout.contains("home-manager switch"));
    assert_eq!(
        fs::read_to_string(remove_log).unwrap(),
        "profile remove yazelix\n"
    );
    assert!(!fixture.managed_config.exists());
    let archived_paths = fs::read_dir(fixture.managed_config.parent().unwrap())
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    assert!(
        archived_paths
            .iter()
            .any(|name| { name.starts_with("settings.jsonc.home-manager-prepare-backup-") })
    );
}

// Regression: `yzx update home_manager` must detect local git-backed `path:` inputs and print the exact safer `git+file:` replacement instead of normalizing the slow path snapshot UX.
#[test]
fn yzx_control_update_home_manager_recommends_git_file_for_local_path_input() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let update_log = fixture.home_dir.join("nix-update.log");
    let flake_dir = fixture.home_dir.join("hm-flake");
    let local_checkout = fixture.home_dir.join("local-yazelix");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&flake_dir).unwrap();
    fs::create_dir_all(local_checkout.join(".git")).unwrap();
    fs::write(flake_dir.join("flake.nix"), "{ outputs = { self }: {}; }\n").unwrap();
    fs::write(
        flake_dir.join("flake.lock"),
        format!(
            r#"{{
  "nodes": {{
    "root": {{
      "inputs": {{
        "yazelix": "yazelix"
      }}
    }},
    "yazelix": {{
      "locked": {{
        "path": "{}",
        "type": "path"
      }},
      "original": {{
        "path": "{}",
        "type": "path"
      }}
    }}
  }},
  "root": "root",
  "version": 7
}}
"#,
            local_checkout.display(),
            local_checkout.display()
        ),
    )
    .unwrap();
    write_executable_script(
        &fake_bin.join("nix"),
        &format!(
            "#!/bin/sh
if [ \"$1\" = flake ] && [ \"$2\" = update ] && [ \"$3\" = yazelix ]; then
  printf '%s\\n' \"$*\" > \"{}\"
  exit 0
fi
echo unexpected nix invocation: \"$*\" >&2
exit 99
",
            update_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .current_dir(&flake_dir)
        .env("PATH", prepend_path(&fake_bin))
        .arg("update")
        .arg("home_manager")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("pinned as a local `path:` source"));
    assert!(stdout.contains("snapshots the whole directory"));
    assert!(stdout.contains("prefer `git+file:`"));
    assert!(stdout.contains(&format!(
        "url = \"git+file://{}\";",
        local_checkout.display()
    )));
    assert!(stdout.contains("home-manager switch"));
    assert_eq!(
        fs::read_to_string(update_log).unwrap(),
        "flake update yazelix\n"
    );
}
