// Test lane: maintainer

use assert_cmd::Command;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};
use yazelix_core::active_config_surface::TOML_TOOLING_CONFIG_FILENAME;

mod support;

use support::commands::yzx_core_command;
use support::envelopes::{error_envelope, ok_envelope};
use support::fixtures::{repo_root, write_runtime_contract_assets};

fn doctor_config_request(config_dir: &Path, runtime_dir: &Path) -> String {
    serde_json::json!({
        "config_dir": config_dir.to_string_lossy(),
        "runtime_dir": runtime_dir.to_string_lossy(),
    })
    .to_string()
}
fn prepare_doctor_config_runtime_fixture(repo: &Path, tmp: &TempDir) -> PathBuf {
    let runtime_dir = tmp.path().join("runtime");
    write_runtime_contract_assets(repo, &runtime_dir);
    runtime_dir
}
struct RuntimeMaterializationFixture {
    home_dir: PathBuf,
    runtime_dir: PathBuf,
    config_dir: PathBuf,
    state_dir: PathBuf,
    managed_config: PathBuf,
    state_path: PathBuf,
    yazi_dir: PathBuf,
    zellij_dir: PathBuf,
    zellij_layout_dir: PathBuf,
}
fn prepare_runtime_materialization_fixture(
    repo: &Path,
    tmp: &TempDir,
) -> RuntimeMaterializationFixture {
    let home_dir = tmp.path().join("home");
    let runtime_dir = tmp.path().join("runtime");
    let config_dir = home_dir.join(".config").join("yazelix");
    let state_dir = home_dir.join(".local").join("share").join("yazelix");
    let managed_config = config_dir.join("user_configs").join("yazelix.toml");
    let managed_zellij_config = config_dir
        .join("user_configs")
        .join("zellij")
        .join("config.kdl");
    let state_path = state_dir.join("state").join("rebuild_hash");
    let yazi_dir = state_dir.join("configs").join("yazi");
    let zellij_dir = state_dir.join("configs").join("zellij");
    let zellij_layout_dir = zellij_dir.join("layouts");
    let runtime_yazi_dir = runtime_dir.join("configs").join("yazi");
    let runtime_zellij_dir = runtime_dir.join("configs").join("zellij");
    let runtime_layout_dir = runtime_zellij_dir.join("layouts");
    let runtime_fragment_dir = runtime_layout_dir.join("fragments");
    let runtime_plugin_dir = runtime_zellij_dir.join("plugins");
    let runtime_shell_dir = runtime_dir.join("shells").join("posix");
    let runtime_contract_dir = runtime_dir.join("config_metadata");
    let runtime_ghostty_shader_dir = runtime_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    fs::create_dir_all(managed_config.parent().unwrap()).unwrap();
    fs::create_dir_all(managed_zellij_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&zellij_layout_dir).unwrap();
    fs::create_dir_all(&runtime_yazi_dir).unwrap();
    fs::create_dir_all(&runtime_fragment_dir).unwrap();
    fs::create_dir_all(&runtime_plugin_dir).unwrap();
    fs::create_dir_all(&runtime_shell_dir).unwrap();
    fs::create_dir_all(&runtime_contract_dir).unwrap();
    fs::create_dir_all(&runtime_ghostty_shader_dir).unwrap();
    write_runtime_contract_assets(repo, &runtime_dir);
    fs::write(
        runtime_shell_dir.join("yazelix_nu.sh"),
        "#!/bin/sh\nexec nu \"$@\"\n",
    )
    .unwrap();
    fs::write(
        runtime_yazi_dir.join("yazelix_yazi.toml"),
        "[manager]\nsort_by = \"alphabetical\"\n[opener]\nedit = []\n",
    )
    .unwrap();
    fs::write(runtime_yazi_dir.join("yazelix_keymap.toml"), "").unwrap();
    fs::write(runtime_yazi_dir.join("yazelix_theme.toml"), "").unwrap();
    fs::write(
        runtime_yazi_dir.join("yazelix_starship.toml"),
        "format = \"$all\"\n",
    )
    .unwrap();
    fs::write(runtime_zellij_dir.join("yazelix_overrides.kdl"), "").unwrap();
    fs::write(runtime_layout_dir.join("yzx_side.kdl"), "layout { pane }\n").unwrap();
    fs::write(
        runtime_layout_dir.join("yzx_side_closed.kdl"),
        "layout { pane }\n",
    )
    .unwrap();
    fs::write(
        runtime_layout_dir.join("yzx_side_closed.swap.kdl"),
        "swap_tiled_layout name=\"single_open\" {}\n",
    )
    .unwrap();
    fs::write(
        runtime_layout_dir.join("yzx_no_side.kdl"),
        "layout { pane }\n",
    )
    .unwrap();
    for fragment in [
        "zjstatus_tab_template.kdl",
        "keybinds_common.kdl",
        "swap_sidebar_open.kdl",
        "swap_sidebar_closed.kdl",
    ] {
        fs::write(runtime_fragment_dir.join(fragment), "").unwrap();
    }
    fs::write(
        runtime_plugin_dir.join("yazelix_pane_orchestrator.wasm"),
        b"wasm",
    )
    .unwrap();
    fs::write(runtime_plugin_dir.join("zjstatus.wasm"), b"wasm").unwrap();
    copy_dir_all(
        &repo
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty")
            .join("shaders"),
        &runtime_ghostty_shader_dir,
    );
    fs::copy(runtime_dir.join("yazelix_default.toml"), &managed_config).unwrap();
    fs::write(&managed_zellij_config, "keybinds {}\n").unwrap();
    RuntimeMaterializationFixture {
        home_dir,
        runtime_dir,
        config_dir,
        state_dir,
        managed_config,
        state_path,
        yazi_dir,
        zellij_dir,
        zellij_layout_dir,
    }
}

fn write_cursor_sidecar(fixture: &RuntimeMaterializationFixture, raw: &str) {
    let cursor_path = fixture
        .config_dir
        .join("user_configs")
        .join("yazelix_cursors.toml");
    fs::create_dir_all(cursor_path.parent().unwrap()).unwrap();
    fs::write(cursor_path, raw).unwrap();
}

fn cursor_sidecar_path(fixture: &RuntimeMaterializationFixture) -> PathBuf {
    fixture
        .config_dir
        .join("user_configs")
        .join("yazelix_cursors.toml")
}

fn copy_dir_all(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = dst.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_all(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).unwrap();
        }
    }
}

fn runtime_materialization_request(fixture: &RuntimeMaterializationFixture) -> Value {
    json!({
        "config_path": fixture.managed_config,
        "default_config_path": fixture.runtime_dir.join("yazelix_default.toml"),
        "contract_path": fixture.runtime_dir.join("config_metadata/main_config_contract.toml"),
        "runtime_dir": fixture.runtime_dir,
        "state_path": fixture.state_path,
        "yazi_config_dir": fixture.yazi_dir,
        "zellij_config_dir": fixture.zellij_dir,
        "zellij_layout_dir": fixture.zellij_layout_dir,
        "layout_override": Value::Null,
    })
}
fn runtime_materialization_command(
    fixture: &RuntimeMaterializationFixture,
    helper_command: &str,
) -> Command {
    let xdg_config_home = fixture.home_dir.join(".config");
    let xdg_data_home = fixture.home_dir.join(".local").join("share");
    let mut command = yzx_core_command();
    command
        .arg(helper_command)
        .env("HOME", &fixture.home_dir)
        .env("XDG_CONFIG_HOME", xdg_config_home)
        .env("XDG_DATA_HOME", xdg_data_home)
        .env("YAZELIX_CONFIG_DIR", &fixture.config_dir)
        .env("YAZELIX_STATE_DIR", &fixture.state_dir)
        .env("YAZELIX_RUNTIME_DIR", &fixture.runtime_dir);
    command
}
// Defends: config.normalize emits a single machine-readable success envelope for valid config input.
// Contract: CRCP-001
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn config_normalize_prints_one_success_json_envelope() {
    let repo = repo_root();
    let output = yzx_core_command()
        .arg("config.normalize")
        .arg("--config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .output()
        .unwrap();
    let envelope: Value = ok_envelope(&output);
    assert_eq!(envelope["command"], "config.normalize");
    assert_eq!(
        envelope["data"]["normalized_config"]["terminal_config_mode"],
        "yazelix"
    );
}

// Regression: config.normalize accepts every documented widget_tray value, including the cursor widget added to the bar renderer.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn config_normalize_accepts_cursor_widget_tray_entry() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("yazelix.toml");
    fs::write(
        &config_path,
        "[zellij]\nwidget_tray = [\"editor\", \"cursor\", \"cpu\"]\n",
    )
    .unwrap();

    let output = yzx_core_command()
        .arg("config.normalize")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    assert_eq!(
        envelope["data"]["normalized_config"]["zellij_widget_tray"],
        json!(["editor", "cursor", "cpu"])
    );
}

// Defends: config.normalize emits a single machine-readable config error envelope for invalid input.
// Contract: CRCP-001
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn config_normalize_prints_one_error_json_envelope() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("yazelix.toml");
    fs::write(&config_path, "[shell]\ndefault_shell = \"powershell\"\n").unwrap();
    let output = yzx_core_command()
        .arg("config.normalize")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 65);
    assert_eq!(envelope["command"], "config.normalize");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "unsupported_config");
}

// Defends: moved Ghostty cursor fields fail with a sidecar-specific transition hint instead of being silently ignored.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn config_normalize_reports_moved_ghostty_cursor_fields() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("yazelix.toml");
    fs::write(&config_path, "[terminal]\nghostty_trail_duration = 4.5\n").unwrap();
    let output = yzx_core_command()
        .arg("config.normalize")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 65);
    assert_eq!(envelope["command"], "config.normalize");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "unsupported_config");
    let diagnostic = &envelope["error"]["details"]["blocking_diagnostics"][0];
    assert_eq!(
        diagnostic["headline"],
        "Moved cursor config field at terminal.ghostty_trail_duration"
    );
    assert!(
        diagnostic["detail_lines"][1]
            .as_str()
            .unwrap()
            .contains("user_configs/yazelix_cursors.toml")
    );
}

// Defends: unknown helper commands report the requested command in the usage error envelope.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
#[test]
fn unsupported_command_reports_requested_command_in_error_envelope() {
    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config.unknown")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["command"], "config.unknown");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(envelope["error"]["code"], "invalid_arguments");
}

// Defends: config-surface.resolve bootstraps the canonical managed config and TOML tooling support through the Rust active-config owner.
// Contract: CRCP-004
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn config_surface_resolve_bootstraps_managed_config_and_toml_tooling_support() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-surface.resolve")
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--config-dir")
        .arg(&config_dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "config-surface.resolve");
    assert_eq!(envelope["status"], "ok");

    let managed_config = config_dir.join("user_configs").join("yazelix.toml");
    let managed_toml_tooling_config = config_dir.join(TOML_TOOLING_CONFIG_FILENAME);
    assert_eq!(
        envelope["data"]["config_file"],
        managed_config.to_string_lossy().to_string()
    );
    assert_eq!(
        fs::read_to_string(&managed_config).unwrap(),
        fs::read_to_string(runtime_dir.join("yazelix_default.toml")).unwrap()
    );
    assert_eq!(
        fs::read_to_string(&managed_toml_tooling_config).unwrap(),
        fs::read_to_string(runtime_dir.join(TOML_TOOLING_CONFIG_FILENAME)).unwrap()
    );
}

// Defends: config.normalize rejects removed config surfaces without mutating the active config file or creating backup churn.
// Contract: CRCP-001
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn config_normalize_rejects_removed_surfaces_without_rewriting() {
    let repo = repo_root();

    for (label, raw_config, expected_field) in [
        ("legacy_ascii", "[ascii]\nmode = \"animated\"\n", "ascii"),
        (
            "removed_enable_atuin",
            "[shell]\nenable_atuin = true\n",
            "shell.enable_atuin",
        ),
        (
            "legacy_packs",
            "[packs]\nenabled = [\"git\"]\nuser_packages = [\"docker\"]\n\n[packs.declarations]\ngit = [\"gh\", \"prek\"]\n",
            "packs",
        ),
        (
            "removed_persistent_sessions",
            "[zellij]\npersistent_sessions = true\n",
            "zellij.persistent_sessions",
        ),
        (
            "removed_session_name",
            "[zellij]\nsession_name = \"demo\"\n",
            "zellij.session_name",
        ),
        (
            "removed_initial_sidebar_state",
            "[editor]\ninitial_sidebar_state = \"closed\"\n",
            "editor.initial_sidebar_state",
        ),
    ] {
        let tmp = tempdir().unwrap();
        let config_dir = tmp.path().join("config").join("user_configs");
        fs::create_dir_all(&config_dir).unwrap();
        let config_path = config_dir.join("yazelix.toml");
        fs::write(&config_path, raw_config).unwrap();
        let original = fs::read_to_string(&config_path).unwrap();

        let output = Command::cargo_bin("yzx_core")
            .unwrap()
            .arg("config.normalize")
            .arg("--config")
            .arg(&config_path)
            .arg("--default-config")
            .arg(repo.join("yazelix_default.toml"))
            .arg("--contract")
            .arg(repo.join("config_metadata/main_config_contract.toml"))
            .output()
            .unwrap();

        assert_eq!(output.status.code(), Some(65), "{label}");
        assert!(output.stdout.is_empty(), "{label}");
        let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
        assert_eq!(envelope["command"], "config.normalize", "{label}");
        assert_eq!(envelope["status"], "error", "{label}");
        assert_eq!(envelope["error"]["class"], "config", "{label}");
        assert_eq!(envelope["error"]["code"], "unsupported_config", "{label}");
        let reported_field = envelope["error"]["details"]["field"]
            .as_str()
            .map(str::to_string)
            .or_else(|| {
                envelope["error"]["details"]["blocking_diagnostics"][0]["path"]
                    .as_str()
                    .map(str::to_string)
            });
        assert_eq!(reported_field.as_deref(), Some(expected_field), "{label}");
        assert_eq!(
            fs::read_to_string(&config_path).unwrap(),
            original,
            "{label}"
        );
        assert!(fs::read_dir(&config_dir).unwrap().count() == 1, "{label}");
    }
}

// Defends: config-state.compute returns a machine-readable state envelope with stable hash fields.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn config_state_compute_prints_machine_readable_state_envelope() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let state_path = tmp.path().join("state/rebuild_hash");

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.compute")
        .arg("--config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&repo)
        .arg("--state-path")
        .arg(&state_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "config-state.compute");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["config"]["default_shell"], "nu");
    assert_eq!(
        envelope["data"]["config_hash"],
        "cfba8d137ac98997cbf9437838509db79f49ea26e7e1f806b2a9a1da7580f7a8"
    );
    assert_eq!(envelope["data"]["needs_refresh"], true);
}

// Defends: config-state.record persists state only for the managed main config surface.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn config_state_record_writes_only_managed_surface_state() {
    let tmp = tempdir().unwrap();
    let managed_config = tmp.path().join("config/user_configs/yazelix.toml");
    let state_path = tmp.path().join("state/rebuild_hash");

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.record")
        .arg("--config-file")
        .arg(&managed_config)
        .arg("--managed-config")
        .arg(&managed_config)
        .arg("--state-path")
        .arg(&state_path)
        .arg("--config-hash")
        .arg("cfg")
        .arg("--runtime-hash")
        .arg("runtime")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "config-state.record");
    assert_eq!(envelope["data"]["recorded"], true);
    let recorded: Value = serde_json::from_str(&fs::read_to_string(state_path).unwrap()).unwrap();
    assert_eq!(
        recorded,
        serde_json::json!({"config_hash":"cfg","runtime_hash":"runtime"})
    );
}

// Defends: runtime-materialization.plan reports missing artifacts without forcing refresh when hashes are current.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_plan_reports_missing_artifacts_with_current_state() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let managed_config = tmp.path().join("config/user_configs/yazelix.toml");
    let state_path = tmp.path().join("state/rebuild_hash");
    let yazi_dir = tmp.path().join("configs/yazi");
    let zellij_dir = tmp.path().join("configs/zellij");
    let zellij_layout_dir = zellij_dir.join("layouts");

    fs::create_dir_all(managed_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&zellij_layout_dir).unwrap();
    fs::copy(repo.join("yazelix_default.toml"), &managed_config).unwrap();

    let state_output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.compute")
        .arg("--config")
        .arg(&managed_config)
        .arg("--default-config")
        .arg(repo.join("yazelix_default.toml"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&repo)
        .arg("--state-path")
        .arg(&state_path)
        .output()
        .unwrap();
    assert!(state_output.status.success());
    let state_envelope: Value = serde_json::from_slice(&state_output.stdout).unwrap();

    let record_output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.record")
        .arg("--config-file")
        .arg(&managed_config)
        .arg("--managed-config")
        .arg(&managed_config)
        .arg("--state-path")
        .arg(&state_path)
        .arg("--config-hash")
        .arg(state_envelope["data"]["config_hash"].as_str().unwrap())
        .arg("--runtime-hash")
        .arg(state_envelope["data"]["runtime_hash"].as_str().unwrap())
        .output()
        .unwrap();
    assert!(record_output.status.success());

    let request = json!({
        "config_path": managed_config,
        "default_config_path": repo.join("yazelix_default.toml"),
        "contract_path": repo.join("config_metadata/main_config_contract.toml"),
        "runtime_dir": repo,
        "state_path": state_path,
        "yazi_config_dir": yazi_dir,
        "zellij_config_dir": zellij_dir,
        "zellij_layout_dir": zellij_layout_dir,
        "layout_override": Value::Null,
    });
    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-materialization.plan")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-materialization.plan");
    assert_eq!(envelope["data"]["status"], "repair_missing_artifacts");
    assert_eq!(envelope["data"]["needs_refresh"], false);
    assert_eq!(
        envelope["data"]["missing_artifacts"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
}

// Defends: startup can create a launch-scoped immutable config snapshot through the packaged Rust helper.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn session_config_snapshot_write_command_writes_launch_scoped_snapshot() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    let request = json!({
        "state_dir": fixture.state_dir,
        "snapshot_id": "launch-test",
        "source_config_file": fixture.managed_config,
        "source_config_hash": "cfg-hash",
        "runtime_dir": fixture.runtime_dir,
        "runtime_hash": "runtime-hash",
        "normalized_config": {
            "popup_program": ["gitui"],
            "default_shell": "bash",
            "terminals": ["ghostty", "wezterm"]
        },
    });

    let output = runtime_materialization_command(&fixture, "session-config-snapshot.write")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "session-config-snapshot.write");
    assert_eq!(envelope["status"], "ok");
    let snapshot_path = fixture
        .state_dir
        .join("sessions/launch-test/config_snapshot.json");
    assert_eq!(
        envelope["data"]["snapshot_path"],
        snapshot_path.to_string_lossy().to_string()
    );
    assert!(snapshot_path.exists());
    let snapshot: Value =
        serde_json::from_str(&fs::read_to_string(snapshot_path).unwrap()).unwrap();
    assert_eq!(snapshot["schema_version"], 1);
    assert_eq!(snapshot["snapshot_id"], "launch-test");
    assert_eq!(snapshot["source_config"]["hash"], "cfg-hash");
    assert_eq!(snapshot["runtime"]["hash"], "runtime-hash");
    assert_eq!(snapshot["facts"]["popup_program"], json!(["gitui"]));
    assert_eq!(snapshot["normalized_config"]["default_shell"], "bash");
}

// Defends: runtime-materialization.materialize becomes the single Rust owner for generate-plus-record of managed runtime artifacts.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_materialize_writes_generated_artifacts_and_records_state() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    let request = runtime_materialization_request(&fixture);
    let output = runtime_materialization_command(&fixture, "runtime-materialization.materialize")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-materialization.materialize");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["plan"]["status"], "refresh_required");
    assert_eq!(envelope["data"]["apply"]["recorded"], true);
    assert_eq!(
        envelope["data"]["zellij"]["seeded_plugin_permissions"],
        true
    );
    assert!(fixture.yazi_dir.join("yazi.toml").exists());
    assert!(fixture.yazi_dir.join("keymap.toml").exists());
    assert!(fixture.yazi_dir.join("init.lua").exists());
    assert!(fixture.zellij_dir.join("config.kdl").exists());
    assert!(fixture.zellij_layout_dir.join("yzx_side.kdl").exists());
    assert!(
        fixture
            .zellij_layout_dir
            .join("yzx_side_closed.kdl")
            .exists()
    );
    assert!(
        fixture
            .zellij_layout_dir
            .join("yzx_side_closed.swap.kdl")
            .exists()
    );
    let permissions =
        fs::read_to_string(fixture.home_dir.join(".cache/zellij/permissions.kdl")).unwrap();
    assert!(permissions.contains("zjstatus.wasm"));
    assert!(permissions.contains("yazelix_pane_orchestrator.wasm"));
    assert!(permissions.contains("ReadCliPipes"));
    let recorded: Value =
        serde_json::from_str(&fs::read_to_string(&fixture.state_path).unwrap()).unwrap();
    assert_eq!(
        recorded["config_hash"],
        envelope["data"]["plan"]["config_hash"]
    );
    assert_eq!(
        recorded["runtime_hash"],
        envelope["data"]["plan"]["runtime_hash"]
    );
    let second_output =
        runtime_materialization_command(&fixture, "runtime-materialization.materialize")
            .arg("--request-json")
            .arg(request.to_string())
            .output()
            .unwrap();
    assert!(second_output.status.success());
    let second: Value = serde_json::from_slice(&second_output.stdout).unwrap();
    assert_eq!(second["data"]["zellij"]["reused"], true);
    assert_eq!(second["data"]["zellij"]["seeded_plugin_permissions"], true);
}

// Defends: runtime-materialization.repair repairs missing managed artifacts through the Rust lifecycle owner instead of bouncing back into a Nu coordinator.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_repair_regenerates_missing_artifacts_end_to_end() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    let request = runtime_materialization_request(&fixture);

    let initial_output =
        runtime_materialization_command(&fixture, "runtime-materialization.materialize")
            .arg("--request-json")
            .arg(request.to_string())
            .output()
            .unwrap();
    assert!(initial_output.status.success());
    fs::remove_file(fixture.yazi_dir.join("yazi.toml")).unwrap();

    let repair_request = json!({
        "plan": request,
        "force": false,
    });
    let repair_output = runtime_materialization_command(&fixture, "runtime-materialization.repair")
        .arg("--request-json")
        .arg(repair_request.to_string())
        .output()
        .unwrap();

    assert!(repair_output.status.success());
    assert!(repair_output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&repair_output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-materialization.repair");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["status"], "repaired_missing_artifacts");
    assert_eq!(
        envelope["data"]["plan"]["status"],
        "repair_missing_artifacts"
    );
    assert_eq!(envelope["data"]["repair"]["action"], "regenerate");
    assert!(envelope["data"]["materialization"].is_object());
    assert!(fixture.yazi_dir.join("yazi.toml").exists());
}

// Defends: runtime-materialization.repair --summary keeps the Home Manager activation path human-readable instead of dumping the full JSON envelope.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn runtime_materialization_repair_summary_prints_one_human_line() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);

    let output = runtime_materialization_command(&fixture, "runtime-materialization.repair")
        .arg("--from-env")
        .arg("--force")
        .arg("--summary")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "✅ Generated runtime state repaired.\n"
    );
    assert!(fixture.yazi_dir.join("yazi.toml").exists());
    assert!(fixture.zellij_dir.join("config.kdl").exists());
}

// Defends: runtime-materialization.repair migrates moved Ghostty cursor fields into the cursor sidecar before strict config validation.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_repair_migrates_moved_ghostty_cursor_fields() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(
        &fixture.managed_config,
        [
            "[terminal]",
            "terminals = [\"ghostty\"]",
            "ghostty_trail_color = \"orchid\"",
            "ghostty_trail_effect = \"sweep\"",
            "ghostty_trail_duration = 1.5",
            "ghostty_mode_effect = \"sonic_boom\"",
            "ghostty_trail_glow = \"high\"",
        ]
        .join("\n"),
    )
    .unwrap();

    let repair_request = json!({
        "plan": runtime_materialization_request(&fixture),
        "force": true,
    });
    let output = runtime_materialization_command(&fixture, "runtime-materialization.repair")
        .arg("--request-json")
        .arg(repair_request.to_string())
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "stdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-materialization.repair");
    assert_eq!(envelope["data"]["status"], "repaired");
    assert_eq!(
        envelope["data"]["migration"]["action"],
        "moved_ghostty_cursor_fields"
    );
    assert_eq!(
        envelope["data"]["migration"]["moved_fields"],
        json!([
            "terminal.ghostty_trail_color",
            "terminal.ghostty_trail_effect",
            "terminal.ghostty_trail_duration",
            "terminal.ghostty_mode_effect",
            "terminal.ghostty_trail_glow",
        ])
    );

    let migrated_main = fs::read_to_string(&fixture.managed_config).unwrap();
    let migrated_main: toml::Table = toml::from_str(&migrated_main).unwrap();
    let terminal = migrated_main
        .get("terminal")
        .and_then(toml::Value::as_table)
        .unwrap();
    assert_eq!(
        terminal.get("terminals").unwrap().as_array().unwrap().len(),
        1
    );
    assert!(terminal.get("ghostty_trail_color").is_none());
    assert!(terminal.get("ghostty_trail_effect").is_none());
    assert!(terminal.get("ghostty_trail_duration").is_none());
    assert!(terminal.get("ghostty_mode_effect").is_none());
    assert!(terminal.get("ghostty_trail_glow").is_none());

    let cursor_sidecar = fs::read_to_string(cursor_sidecar_path(&fixture)).unwrap();
    let cursor_sidecar: toml::Table = toml::from_str(&cursor_sidecar).unwrap();
    let settings = cursor_sidecar
        .get("settings")
        .and_then(toml::Value::as_table)
        .unwrap();
    assert_eq!(settings["trail"].as_str().unwrap(), "orchid");
    assert_eq!(settings["trail_effect"].as_str().unwrap(), "sweep");
    assert_eq!(settings["duration"].as_float().unwrap(), 1.5);
    assert_eq!(settings["mode_effect"].as_str().unwrap(), "sonic_boom");
    assert_eq!(settings["glow"].as_str().unwrap(), "high");
}

// Defends: cursor-field auto-migration does not turn runtime repair into a tolerant path for unrelated unsupported config.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_repair_keeps_unrelated_unknown_fields_blocking() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(
        &fixture.managed_config,
        [
            "[terminal]",
            "ghostty_trail_color = \"orchid\"",
            "not_a_real_terminal_option = true",
        ]
        .join("\n"),
    )
    .unwrap();

    let repair_request = json!({
        "plan": runtime_materialization_request(&fixture),
        "force": true,
    });
    let output = runtime_materialization_command(&fixture, "runtime-materialization.repair")
        .arg("--request-json")
        .arg(repair_request.to_string())
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 65);
    assert_eq!(envelope["command"], "runtime-materialization.repair");
    assert_eq!(envelope["error"]["code"], "unsupported_config");
    assert_eq!(
        envelope["error"]["details"]["blocking_diagnostics"][0]["path"],
        "terminal.not_a_real_terminal_option"
    );
}

// Defends: runtime-materialization.repair --summary keeps activation failures human-readable instead of dumping the raw JSON envelope.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_materialization_repair_summary_prints_human_config_error() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(
        &fixture.managed_config,
        ["[terminal]", "not_a_real_terminal_option = true"].join("\n"),
    )
    .unwrap();

    let repair_request = json!({
        "plan": runtime_materialization_request(&fixture),
        "force": true,
    });
    let output = runtime_materialization_command(&fixture, "runtime-materialization.repair")
        .arg("--request-json")
        .arg(repair_request.to_string())
        .arg("--summary")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(65));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Yazelix generated runtime repair failed"));
    assert!(stderr.contains("Blocking config issues: 1"));
    assert!(stderr.contains("- Unknown config field at terminal.not_a_real_terminal_option"));
    assert!(stderr.contains("- Remove or rename this field manually"));
    assert!(!stderr.trim_start().starts_with('{'));
    assert!(!stderr.contains("\"schema_version\""));
    assert!(!stderr.contains("\"blocking_diagnostics\""));
}

// Defends: runtime-contract.evaluate emits one machine-readable checks envelope for batched preflight requests.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_contract_evaluate_prints_machine_readable_checks_envelope() {
    let tmp = tempdir().unwrap();
    let host_bin = tmp.path().join("host-bin");
    fs::create_dir_all(&host_bin).unwrap();
    fs::write(host_bin.join("ghostty"), "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(host_bin.join("ghostty"))
            .unwrap()
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(host_bin.join("ghostty"), permissions).unwrap();
    }

    let request = serde_json::json!({
        "working_dir": {
            "kind": "launch",
            "path": tmp.path().join("missing-dir").to_string_lossy().to_string()
        },
        "terminal_support": {
            "owner_surface": "launch",
            "requested_terminal": "",
            "terminals": ["ghostty"],
            "command_search_paths": [host_bin.to_string_lossy().to_string()]
        }
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-contract.evaluate")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "runtime-contract.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(
        envelope["data"]["checks"][0]["message"],
        format!(
            "Launch directory does not exist: {}",
            tmp.path().join("missing-dir").to_string_lossy()
        )
    );
    assert_eq!(
        envelope["data"]["checks"][1]["message"],
        "A configured terminal command is available"
    );
}

// Defends: startup-launch-preflight.evaluate returns a single startup summary without Nu-side check selection.
// Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
#[test]
fn startup_launch_preflight_evaluate_prints_startup_summary_envelope() {
    let tmp = tempdir().unwrap();
    let work = tmp.path().join("work");
    fs::create_dir_all(&work).unwrap();
    let script = tmp.path().join("inner.nu");
    fs::write(&script, "").unwrap();

    let request = serde_json::json!({
        "startup": {
            "working_dir": work.to_string_lossy().to_string(),
            "runtime_script": {
                "id": "startup_runtime_script",
                "label": "startup script",
                "owner_surface": "startup",
                "path": script.to_string_lossy().to_string()
            }
        }
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("startup-launch-preflight.evaluate")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "startup-launch-preflight.evaluate");
    assert_eq!(envelope["data"]["kind"], "startup");
    assert_eq!(
        envelope["data"]["working_dir"].as_str().unwrap(),
        work.to_string_lossy()
    );
    assert_eq!(
        envelope["data"]["script_path"].as_str().unwrap(),
        script.to_string_lossy()
    );
}

// Defends: runtime-contract.evaluate rejects malformed request JSON as a usage-surface error.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn runtime_contract_evaluate_reports_invalid_request_json_as_usage_error() {
    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("runtime-contract.evaluate")
        .arg("--request-json")
        .arg("{not-json")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(64));
    assert!(output.stdout.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["command"], "runtime-contract.evaluate");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(envelope["error"]["code"], "invalid_request_json");
}

// Defends: install-ownership.evaluate emits one machine-readable report envelope for explicit request paths.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn install_ownership_evaluate_prints_ok_envelope() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let home = tmp.path().join("home");
    let xdg_data = home.join(".local/share");
    let xdg_config = home.join(".config");
    let state = xdg_data.join("yazelix");
    let main = home.join(".config/yazelix/yazelix.toml");
    fs::create_dir_all(main.parent().unwrap()).unwrap();
    fs::write(&main, "[core]\n").unwrap();

    let request = serde_json::json!({
        "runtime_dir": repo.to_string_lossy(),
        "home_dir": home.to_string_lossy(),
        "user": null,
        "xdg_config_home": xdg_config.to_string_lossy(),
        "xdg_data_home": xdg_data.to_string_lossy(),
        "yazelix_state_dir": state.to_string_lossy(),
        "main_config_path": main.to_string_lossy(),
        "invoked_yzx_path": null,
        "redirected_from_stale_yzx_path": null,
        "shell_resolved_yzx_path": null,
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("install-ownership.evaluate")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "install-ownership.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert!(envelope["data"]["install_owner"].is_string());
    assert!(envelope["data"]["desktop_entry_freshness"]["message"].is_string());
}

// Defends: install-ownership.evaluate can build the env-derived request in Rust without a Nushell bridge.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn install_ownership_evaluate_from_env_resolves_stable_profile_wrapper() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let home = tmp.path().join("home");
    let config_dir = home.join(".config").join("yazelix");
    let xdg_data = home.join(".local").join("share");
    let state_dir = xdg_data.join("yazelix");
    let profile_yzx = home.join(".nix-profile").join("bin").join("yzx");

    fs::create_dir_all(config_dir.join("user_configs")).unwrap();
    fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
    fs::write(
        config_dir.join("user_configs").join("yazelix.toml"),
        "[core]\n",
    )
    .unwrap();
    fs::write(&profile_yzx, "#!/bin/sh\nexit 0\n").unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("install-ownership.evaluate")
        .arg("--from-env")
        .arg("--runtime-dir")
        .arg(&repo)
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", &xdg_data)
        .env("YAZELIX_CONFIG_DIR", &config_dir)
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("USER", "alice")
        .env_remove("YAZELIX_INVOKED_YZX_PATH")
        .env_remove("YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "install-ownership.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["install_owner"], "profile");
    assert_eq!(
        envelope["data"]["stable_yzx_wrapper"],
        profile_yzx.to_string_lossy().to_string()
    );
    assert_eq!(
        envelope["data"]["desktop_launcher_path"],
        profile_yzx.to_string_lossy().to_string()
    );
}

// Defends: terminal-materialization.generate can resolve config/runtime/state request roots from process env without Nu path assembly.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn terminal_materialization_generate_from_env_writes_generated_configs() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);

    fs::write(
        &fixture.managed_config,
        [
            "[terminal]",
            "terminals = [\"ghostty\", \"kitty\"]",
            "transparency = \"low\"",
        ]
        .join("\n"),
    )
    .unwrap();

    let output = runtime_materialization_command(&fixture, "terminal-materialization.generate")
        .arg("--from-env")
        .arg("--terminals-json")
        .arg(json!(["ghostty", "kitty"]).to_string())
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "stdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.stderr.is_empty());

    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "terminal-materialization.generate");
    assert_eq!(envelope["status"], "ok");
    assert!(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty")
            .exists()
    );
    assert!(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("kitty")
            .join("kitty.conf")
            .exists()
    );
}

// Defends: Kitty cursor fallback is controlled by the cursor sidecar's binary kitty_enable_cursor setting.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn terminal_materialization_uses_cursor_sidecar_for_kitty_toggle() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);

    fs::write(
        &fixture.managed_config,
        ["[terminal]", "terminals = [\"kitty\"]"].join("\n"),
    )
    .unwrap();
    write_cursor_sidecar(
        &fixture,
        r##"
schema_version = 1
enabled_cursors = ["snow"]

[settings]
trail = "snow"
trail_effect = "none"
mode_effect = "none"
glow = "medium"
duration = 1.0
kitty_enable_cursor = false

[[cursor]]
name = "snow"
family = "mono"
color = "#ffffff"
"##,
    );

    let output = runtime_materialization_command(&fixture, "terminal-materialization.generate")
        .arg("--from-env")
        .arg("--terminals-json")
        .arg(json!(["kitty"]).to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let kitty_config = fs::read_to_string(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("kitty")
            .join("kitty.conf"),
    )
    .unwrap();
    assert!(kitty_config.contains("# cursor_trail 0  # disabled by yazelix_cursors.toml"));
}

// Defends: ghostty-materialization.generate can resolve config/runtime/state request roots from process env without Nu path assembly.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn ghostty_materialization_generate_from_env_uses_normalized_config() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    copy_dir_all(
        &repo
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty")
            .join("shaders"),
        &fixture
            .runtime_dir
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty")
            .join("shaders"),
    );

    fs::write(
        &fixture.managed_config,
        ["[terminal]", "transparency = \"high\""].join("\n"),
    )
    .unwrap();
    write_cursor_sidecar(
        &fixture,
        r##"
schema_version = 1
enabled_cursors = ["forest"]

[settings]
trail = "forest"
trail_effect = "tail"
mode_effect = "ripple"
glow = "high"
duration = 1.5
kitty_enable_cursor = true

[[cursor]]
name = "forest"
family = "mono"
color = "#3bd17a"
"##,
    );

    let output = runtime_materialization_command(&fixture, "ghostty-materialization.generate")
        .arg("--from-env")
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "stdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(output.stderr.is_empty());

    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "ghostty-materialization.generate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["cursor_state"]["selected_color"], "forest");
    assert_eq!(
        envelope["data"]["cursor_state"]["selected_trail_effect"],
        "tail"
    );
    assert_eq!(
        envelope["data"]["cursor_state"]["selected_mode_effect"],
        "ripple"
    );
    assert_eq!(
        envelope["data"]["cursor_state"]["trail_duration"],
        serde_json::json!(1.5)
    );
    let ghostty_dir = fixture
        .state_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty");
    let generated_config = fs::read_to_string(ghostty_dir.join("config")).unwrap();
    assert!(generated_config.contains("# Ghostty trail duration multiplier: 1.5"));
    let forest_shader =
        fs::read_to_string(ghostty_dir.join("shaders").join("cursor_trail_forest.glsl")).unwrap();
    assert!(forest_shader.contains("const float DURATION = 0.375;"));
    let tail_shader = fs::read_to_string(
        ghostty_dir
            .join("shaders")
            .join("generated_effects")
            .join("tail.glsl"),
    )
    .unwrap();
    assert!(tail_shader.contains("const float DURATION = 0.135;"));
    let ripple_shader = fs::read_to_string(
        ghostty_dir
            .join("shaders")
            .join("generated_effects")
            .join("ripple.glsl"),
    )
    .unwrap();
    assert!(ripple_shader.contains("const float DURATION = 0.15;"));
    assert!(ghostty_dir.exists());
}

// Defends: doctor-helix.evaluate emits one machine-readable report envelope for a minimal request.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn doctor_helix_evaluate_prints_ok_envelope() {
    let tmp = tempdir().unwrap();
    let home = tmp.path().join("home");
    let user_rt = home.join(".config/helix/runtime");
    fs::create_dir_all(user_rt.parent().unwrap()).unwrap();

    let request = serde_json::json!({
        "home_dir": home.to_string_lossy(),
        "runtime_dir": tmp.path().join("runtime").to_string_lossy(),
        "config_dir": tmp.path().join("config").to_string_lossy(),
        "user_config_helix_runtime_dir": user_rt.to_string_lossy(),
        "hx_exe_path": null,
        "include_runtime_health": false,
        "editor_command": null,
        "managed_helix_user_config_path": home.join("managed.toml").to_string_lossy(),
        "native_helix_config_path": home.join("native.toml").to_string_lossy(),
        "generated_helix_config_path": home.join("generated.toml").to_string_lossy(),
        "expected_managed_config": null,
        "build_managed_config_error": null,
        "reveal_binding_expected": ":sh yzx reveal \"%{buffer_name}\"",
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("doctor-helix.evaluate")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "doctor-helix.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["runtime_conflicts"]["status"], "ok");
    assert!(
        envelope["data"]["managed_integration"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

// Defends: doctor-runtime.evaluate emits one machine-readable report envelope for a minimal request.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn doctor_runtime_evaluate_prints_ok_envelope() {
    let tmp = tempdir().unwrap();
    let home = tmp.path().join("home");
    let state = home.join(".local/share/yazelix");
    let rt = tmp.path().join("runtime");
    fs::create_dir_all(rt.join("bin")).unwrap();
    fs::write(rt.join("yazelix_default.toml"), "").unwrap();
    fs::write(rt.join("bin").join("yzx"), "").unwrap();
    fs::create_dir_all(rt.join("libexec").join("nu")).unwrap();

    let request = serde_json::json!({
        "runtime_dir": rt.to_string_lossy(),
        "yazelix_state_dir": state.to_string_lossy(),
        "has_home_manager_managed_install": false,
        "is_manual_runtime_reference_path": false,
        "shared_runtime": null,
    });

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("doctor-runtime.evaluate")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "doctor-runtime.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(
        envelope["data"]["distribution"]["capability_mode"],
        "package_runtime"
    );
    assert!(
        envelope["data"]["shared_runtime_preflight"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

// Defends: doctor-config.evaluate reports duplicate root/user config ownership as a config-surface error finding.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn doctor_config_evaluate_reports_duplicate_config_surfaces() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");
    let user_config_dir = config_dir.join("user_configs");
    fs::create_dir_all(&user_config_dir).unwrap();
    fs::write(user_config_dir.join("yazelix.toml"), "[shell]\n").unwrap();
    fs::write(config_dir.join("yazelix.toml"), "[shell]\n").unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("doctor-config.evaluate")
        .arg("--request-json")
        .arg(doctor_config_request(&config_dir, &runtime_dir))
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "doctor-config.evaluate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(
        envelope["data"]["findings"][0]["message"],
        "Could not reconcile Yazelix config surfaces"
    );
    assert_eq!(envelope["data"]["findings"][0]["status"], "error");
    let details = envelope["data"]["findings"][0]["details"].as_str().unwrap();
    assert!(details.contains("user_configs main:"));
    assert!(details.contains("legacy main:"));
}

// Defends: doctor-config.evaluate preserves the stale-schema warning contract and includes the diagnostic report payload.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn doctor_config_evaluate_reports_stale_schema_warning() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");
    let user_config_dir = config_dir.join("user_configs");
    fs::create_dir_all(&user_config_dir).unwrap();
    fs::write(
        user_config_dir.join("yazelix.toml"),
        "[editor]\nsidebar_width_percent = 99\n",
    )
    .unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("doctor-config.evaluate")
        .arg("--request-json")
        .arg(doctor_config_request(&config_dir, &runtime_dir))
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "doctor-config.evaluate");
    assert_eq!(
        envelope["data"]["findings"][0]["message"],
        "Using custom yazelix.toml configuration"
    );
    assert_eq!(envelope["data"]["findings"][1]["status"], "warning");
    assert_eq!(
        envelope["data"]["findings"][1]["message"],
        "Stale or unsupported yazelix.toml entries detected (1 issues)"
    );
    assert_eq!(
        envelope["data"]["findings"][1]["config_diagnostic_report"]["issue_count"],
        1
    );
    assert_eq!(
        envelope["data"]["findings"][1]["config_diagnostic_report"]["doctor_diagnostics"][0]["headline"],
        "Invalid config value at editor.sidebar_width_percent"
    );
    let details = envelope["data"]["findings"][1]["details"].as_str().unwrap();
    assert!(details.contains("Config report for:"));
    assert!(details.contains("Issues: 1"));
    assert!(details.contains("Invalid config value at editor.sidebar_width_percent"));
}

// Regression: malformed TOML must stay on the validation-error path instead of being downgraded into the stale-schema warning row.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn doctor_config_evaluate_keeps_invalid_toml_as_error() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");
    let user_config_dir = config_dir.join("user_configs");
    fs::create_dir_all(&user_config_dir).unwrap();
    fs::write(user_config_dir.join("yazelix.toml"), "[editor\n").unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("doctor-config.evaluate")
        .arg("--request-json")
        .arg(doctor_config_request(&config_dir, &runtime_dir))
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        envelope["data"]["findings"][1]["message"],
        "Could not validate yazelix.toml against the current schema"
    );
    assert_eq!(envelope["data"]["findings"][1]["status"], "error");
    assert!(
        envelope["data"]["findings"][1]["details"]
            .as_str()
            .unwrap()
            .contains("Could not parse Yazelix TOML input")
    );
}

// Defends: doctor-config.evaluate keeps the default-template doctor row fixable instead of bootstrapping config eagerly.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn doctor_config_evaluate_reports_default_template_as_fixable() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("doctor-config.evaluate")
        .arg("--request-json")
        .arg(doctor_config_request(&config_dir, &runtime_dir))
        .output()
        .unwrap();

    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "doctor-config.evaluate");
    assert_eq!(envelope["data"]["findings"][0]["status"], "info");
    assert_eq!(
        envelope["data"]["findings"][0]["message"],
        "Using default configuration (yazelix_default.toml)"
    );
    assert_eq!(envelope["data"]["findings"][0]["fix_available"], true);
}
