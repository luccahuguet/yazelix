// Test lane: maintainer

use assert_cmd::Command;
use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{TempDir, tempdir};
use yazelix_core::{
    ghostty_cursor_registry::CursorRegistry,
    settings_surface::{read_settings_jsonc_value, render_default_settings_jsonc},
    user_config_paths::shared_cursor_config,
};
use yazelix_cursors::render_cursor_settings_jsonc;

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
    let managed_config = config_dir.join("settings.jsonc");
    let managed_zellij_config = config_dir.join("zellij.kdl");
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
    let runtime_libexec_dir = runtime_dir.join("libexec");
    let runtime_contract_dir = runtime_dir.join("config_metadata");
    let runtime_ghostty_shader_dir = runtime_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    let runtime_yzxterm_package_dir = runtime_dir.join("share").join("yazelix-terminal");
    let runtime_yzxterm_baseline_dir = runtime_yzxterm_package_dir.join("baseline");
    let runtime_yzxterm_shader_profile_dir =
        runtime_yzxterm_package_dir.join("profiles").join("shaders");
    let runtime_yzxterm_emoji_dir = runtime_yzxterm_package_dir.join("emoji");
    fs::create_dir_all(managed_config.parent().unwrap()).unwrap();
    fs::create_dir_all(managed_zellij_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&zellij_layout_dir).unwrap();
    fs::create_dir_all(&runtime_yazi_dir).unwrap();
    fs::create_dir_all(&runtime_fragment_dir).unwrap();
    fs::create_dir_all(&runtime_plugin_dir).unwrap();
    fs::create_dir_all(&runtime_shell_dir).unwrap();
    fs::create_dir_all(&runtime_libexec_dir).unwrap();
    fs::create_dir_all(&runtime_contract_dir).unwrap();
    fs::create_dir_all(&runtime_ghostty_shader_dir).unwrap();
    fs::create_dir_all(&runtime_yzxterm_package_dir).unwrap();
    fs::create_dir_all(&runtime_yzxterm_baseline_dir).unwrap();
    fs::create_dir_all(&runtime_yzxterm_shader_profile_dir).unwrap();
    fs::create_dir_all(&runtime_yzxterm_emoji_dir).unwrap();
    write_runtime_contract_assets(repo, &runtime_dir);
    fs::write(
        runtime_shell_dir.join("yazelix_nu.sh"),
        "#!/bin/sh\nexec nu \"$@\"\n",
    )
    .unwrap();
    write_fake_zellij_bar_widget(&runtime_libexec_dir.join("yazelix_zellij_bar_widget"));
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
    for fragment in [
        "swap_sidebar_open.kdl",
        "swap_sidebar_closed.kdl",
        "swap_agent_open.kdl",
        "swap_agent_closed.kdl",
    ] {
        fs::write(runtime_fragment_dir.join(fragment), "").unwrap();
    }
    fs::write(
        runtime_plugin_dir.join("yazelix_pane_orchestrator.wasm"),
        b"wasm",
    )
    .unwrap();
    fs::write(runtime_plugin_dir.join("zjstatus.wasm"), b"wasm").unwrap();
    fs::write(runtime_plugin_dir.join("yzpp.wasm"), b"wasm").unwrap();
    write_runtime_cursor_shader_assets(&runtime_ghostty_shader_dir);
    write_yzxterm_package_profile_set(&runtime_yzxterm_package_dir, None);
    write_yzxterm_package_profile_set(
        &runtime_yzxterm_emoji_dir.join("twitter"),
        Some("Twitter Color Emoji"),
    );
    write_yzxterm_package_profile_set(
        &runtime_yzxterm_emoji_dir.join("serenityos"),
        Some("SerenityOS Emoji"),
    );
    fs::write(
        &managed_config,
        render_default_settings_jsonc(&runtime_dir.join("settings_default.jsonc")).unwrap(),
    )
    .unwrap();
    fs::write(&managed_zellij_config, "theme \"default\"\n").unwrap();
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

fn write_yzxterm_package_profile_set(root: &Path, emoji_family: Option<&str>) {
    let baseline_dir = root.join("baseline");
    let shader_profile_dir = root.join("profiles").join("shaders");
    fs::create_dir_all(root).unwrap();
    fs::create_dir_all(&baseline_dir).unwrap();
    fs::create_dir_all(&shader_profile_dir).unwrap();
    let fonts = emoji_family
        .map(|family| {
            format!(
                r#"
[fonts]
symbol-map = [{{ start = "1F000", end = "1FB00", font-family = "{family}" }}]
"#
            )
        })
        .unwrap_or_default();
    fs::write(
        root.join("config.toml"),
        format!(
            r##"confirm-before-quit = false
{fonts}
[renderer]
backend = "Webgpu"
custom-shader = ["/nix/store/demo/cursor_trail_dusk.glsl"]

[window]
decorations = "Disabled"

[effects]
trail-cursor = true
"##
        ),
    )
    .unwrap();
    fs::write(
        baseline_dir.join("config.toml"),
        format!(
            r##"confirm-before-quit = false
{fonts}
[renderer]
backend = "Webgpu"

[window]
decorations = "Disabled"
"##
        ),
    )
    .unwrap();
    fs::write(
        shader_profile_dir.join("config.toml"),
        format!(
            r##"confirm-before-quit = false
{fonts}
[renderer]
backend = "Webgpu"
custom-shader = ["/nix/store/demo/cursor_trail_dusk.glsl"]

[window]
decorations = "Disabled"

[effects]
trail-cursor = true
"##
        ),
    )
    .unwrap();
}

fn write_fake_zellij_bar_widget(path: &Path) {
    fs::write(
        path,
        r#"#!/bin/sh
[ "$1" = "render-yazelix-runtime" ] || exit 11
[ "$2" = "--json" ] || exit 12
printf '%s\n' '{"schema_version":2,"plugin_block":"plugin location=\"file:/fake/zjstatus.wasm\" {}"}'
"#,
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
}

fn write_cursor_sidecar(fixture: &RuntimeMaterializationFixture, raw: &str) {
    let cursor_path = shared_cursor_config(&fixture.config_dir);
    fs::create_dir_all(cursor_path.parent().unwrap()).unwrap();
    let registry = CursorRegistry::parse_str(&cursor_path, raw).unwrap();
    fs::write(cursor_path, render_cursor_settings_jsonc(&registry)).unwrap();
}

fn write_managed_config_toml(fixture: &RuntimeMaterializationFixture, raw: &str) {
    let value = toml::from_str::<toml::Value>(raw).unwrap();
    let json = serde_json::to_value(value).unwrap();
    fs::write(
        &fixture.managed_config,
        format!("{}\n", serde_json::to_string_pretty(&json).unwrap()),
    )
    .unwrap();
}

fn write_runtime_cursor_shader_assets(shader_dir: &Path) {
    fs::create_dir_all(shader_dir.join("upstream_effects")).unwrap();
    fs::create_dir_all(shader_dir.join("variants")).unwrap();
    fs::write(
        shader_dir.join("cursor_trail_common.glsl"),
        "void renderMonoColorTrail(out vec4 fragColor, in vec2 fragCoord, vec4 color0, vec4 color1, float duration, float width, float scale) {}\n",
    )
    .unwrap();
    fs::write(
        shader_dir.join("variants").join("reef.glsl"),
        "void mainImage(out vec4 fragColor, in vec2 fragCoord) {}\n",
    )
    .unwrap();
    for (file, duration) in [
        ("cursor_tail.glsl", "0.09"),
        ("cursor_warp.glsl", "0.09"),
        ("cursor_sweep.glsl", "0.09"),
        ("ripple_cursor.glsl", "0.15"),
        ("rectangle_boom_cursor.glsl", "0.15"),
        ("sonic_boom_cursor.glsl", "0.15"),
        ("ripple_rectangle_cursor.glsl", "0.15"),
    ] {
        fs::write(
            shader_dir.join("upstream_effects").join(file),
            format!(
                "vec4 COLOR = iCurrentCursorColor;\n\
                 vec4 TRAIL_COLOR = iCurrentCursorColor;\n\
                 const float BLUR = 1.0;\n\
                 const float MAX_RADIUS = 1.0;\n\
                 const float MAX_SIZE = 1.0;\n\
                 const float MAX_TRAIL_LENGTH = 1.0;\n\
                 const float TRAIL_LENGTH = 1.0;\n\
                 const float TRAIL_SIZE = 1.0;\n\
                 const float RING_THICKNESS = 1.0;\n\
                 const float DURATION = {duration};\n"
            ),
        )
        .unwrap();
    }
}

fn runtime_materialization_request(fixture: &RuntimeMaterializationFixture) -> Value {
    json!({
        "config_path": fixture.managed_config,
        "default_config_path": fixture.runtime_dir.join("settings_default.jsonc"),
        "contract_path": fixture.runtime_dir.join("config_metadata/main_config_contract.toml"),
        "runtime_dir": fixture.runtime_dir,
        "state_path": fixture.state_path,
        "yazi_config_dir": fixture.yazi_dir,
        "zellij_config_dir": fixture.zellij_dir,
        "zellij_layout_dir": fixture.zellij_layout_dir,
        "zellij_permissions_cache_path": fixture.home_dir.join(".cache/zellij/permissions.kdl"),
        "layout_override": Value::Null,
    })
}

fn runtime_materialization_canonical_settings_request(
    fixture: &RuntimeMaterializationFixture,
) -> Value {
    let settings_path = fixture.config_dir.join("settings.jsonc");
    let rendered =
        render_default_settings_jsonc(&fixture.runtime_dir.join("settings_default.jsonc")).unwrap();
    fs::write(&settings_path, rendered).unwrap();

    let mut request = runtime_materialization_request(fixture);
    request
        .as_object_mut()
        .unwrap()
        .insert("config_path".into(), json!(settings_path));
    request
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
#[test]
fn config_normalize_prints_one_success_json_envelope() {
    let repo = repo_root();
    let output = yzx_core_command()
        .arg("config.normalize")
        .arg("--config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
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
        .arg(repo.join("settings_default.jsonc"))
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
        .arg(repo.join("settings_default.jsonc"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 65);
    assert_eq!(envelope["command"], "config.normalize");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "unsupported_config");
}

// Defends: config-surface.resolve bootstraps the canonical managed config through the Rust active-config owner.
// Contract: CRCP-004
#[test]
fn config_surface_resolve_bootstraps_managed_config() {
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

    let managed_config = config_dir.join("settings.jsonc");
    assert_eq!(
        envelope["data"]["config_file"],
        managed_config.to_string_lossy().to_string()
    );
    let managed_value = read_settings_jsonc_value(&managed_config).unwrap();
    assert!(managed_value.get("core").is_some());
    assert!(managed_value.get("cursors").is_none());
    assert!(shared_cursor_config(&config_dir).exists());
}

// Defends: config.normalize rejects removed config surfaces without mutating the active config file or creating backup churn.
// Contract: CRCP-001
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
        (
            "removed_enable_sidebar",
            "[editor]\nenable_sidebar = false\n",
            "editor.enable_sidebar",
        ),
    ] {
        let tmp = tempdir().unwrap();
        let config_dir = tmp.path().join("config");
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
            .arg(repo.join("settings_default.jsonc"))
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

// Defends: config-state.compute returns a machine-readable state envelope with a content hash.
#[test]
fn config_state_compute_prints_machine_readable_state_envelope() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let state_path = tmp.path().join("state/rebuild_hash");

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.compute")
        .arg("--config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
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
    let config_hash = envelope["data"]["config_hash"].as_str().unwrap();
    assert_eq!(config_hash.len(), 64);
    assert!(config_hash.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(envelope["data"]["needs_refresh"], true);
}

// Defends: config-state.record persists state only for the managed main config surface.
#[test]
fn config_state_record_writes_only_managed_surface_state() {
    let tmp = tempdir().unwrap();
    let managed_config = tmp.path().join("config/settings.jsonc");
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
#[test]
fn runtime_materialization_plan_reports_missing_artifacts_with_current_state() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let managed_config = tmp.path().join("config/settings.jsonc");
    let state_path = tmp.path().join("state/rebuild_hash");
    let yazi_dir = tmp.path().join("configs/yazi");
    let zellij_dir = tmp.path().join("configs/zellij");
    let zellij_layout_dir = zellij_dir.join("layouts");

    fs::create_dir_all(managed_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&zellij_layout_dir).unwrap();
    fs::copy(repo.join("settings_default.jsonc"), &managed_config).unwrap();

    let state_output = Command::cargo_bin("yzx_core")
        .unwrap()
        .arg("config-state.compute")
        .arg("--config")
        .arg(&managed_config)
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
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
        "default_config_path": repo.join("settings_default.jsonc"),
        "contract_path": repo.join("config_metadata/main_config_contract.toml"),
        "runtime_dir": repo,
        "state_path": state_path,
        "yazi_config_dir": yazi_dir,
        "zellij_config_dir": zellij_dir,
        "zellij_layout_dir": zellij_layout_dir,
        "zellij_permissions_cache_path": tmp.path().join("home/.cache/zellij/permissions.kdl"),
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
        6
    );
}

// Defends: startup can create a launch-scoped immutable config snapshot through the packaged Rust helper.
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
    assert_eq!(snapshot["normalized_config"]["default_shell"], "bash");
}

// Defends: runtime-materialization.materialize becomes the single Rust owner for generate-plus-record of managed runtime artifacts.
#[test]
fn runtime_materialization_materialize_writes_generated_artifacts_and_records_state() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    let request = runtime_materialization_canonical_settings_request(&fixture);
    let output = runtime_materialization_command(&fixture, "runtime-materialization.materialize")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
    let generated_layout = fs::read_to_string(fixture.zellij_layout_dir.join("yzx_side.kdl"))
        .expect("generated layout");
    let generation_fingerprint = envelope["data"]["zellij"]["generation_fingerprint"]
        .as_str()
        .expect("generation fingerprint");
    assert!(generated_layout.contains("GENERATED ZELLIJ LAYOUT (YAZELIX)"));
    assert!(generated_layout.contains(generation_fingerprint));
    let permissions =
        fs::read_to_string(fixture.home_dir.join(".cache/zellij/permissions.kdl")).unwrap();
    assert!(permissions.contains("zjstatus.wasm"));
    assert!(permissions.contains("yazelix_pane_orchestrator.wasm"));
    assert!(permissions.contains("ReadCliPipes"));
    assert!(permissions.contains("MessageAndLaunchOtherPlugins"));
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
    assert_eq!(second["data"]["zellij"]["seeded_plugin_permissions"], true);
}

// Regression: generated Zellij config is disposable launch state, so a plain native config in that path is overwritten even when hashes are current.
#[test]
fn runtime_materialization_materialize_replaces_plain_generated_zellij_config() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    let request = runtime_materialization_canonical_settings_request(&fixture);

    let initial_output =
        runtime_materialization_command(&fixture, "runtime-materialization.materialize")
            .arg("--request-json")
            .arg(request.to_string())
            .output()
            .unwrap();
    assert!(
        initial_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&initial_output.stderr)
    );
    fs::write(
        fixture.zellij_dir.join("config.kdl"),
        "keybinds clear-defaults=true {\n    normal {}\n}\n",
    )
    .unwrap();

    let repair_output =
        runtime_materialization_command(&fixture, "runtime-materialization.materialize")
            .arg("--request-json")
            .arg(request.to_string())
            .output()
            .unwrap();

    assert!(
        repair_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&repair_output.stderr)
    );
    let regenerated = fs::read_to_string(fixture.zellij_dir.join("config.kdl")).unwrap();
    assert!(regenerated.contains("GENERATED ZELLIJ CONFIG (YAZELIX)"));
    assert!(regenerated.contains("yazelix_pane_orchestrator"));
    assert!(regenerated.contains("yzpp"));
}

// Defends: runtime-materialization.repair repairs missing managed artifacts through the Rust lifecycle owner instead of bouncing back into a Nu coordinator.
#[test]
fn runtime_materialization_repair_regenerates_missing_artifacts_end_to_end() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    let request = runtime_materialization_canonical_settings_request(&fixture);

    let initial_output =
        runtime_materialization_command(&fixture, "runtime-materialization.materialize")
            .arg("--request-json")
            .arg(request.to_string())
            .output()
            .unwrap();
    assert!(
        initial_output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&initial_output.stderr)
    );
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

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "✅ Generated runtime state repaired.\n"
    );
    assert!(fixture.yazi_dir.join("yazi.toml").exists());
    assert!(fixture.zellij_dir.join("config.kdl").exists());
}

// Defends: runtime-materialization.repair --summary keeps activation failures human-readable instead of dumping the raw JSON envelope.
#[test]
fn runtime_materialization_repair_summary_prints_human_config_error() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    write_managed_config_toml(
        &fixture,
        &["[terminal]", "not_a_real_terminal_option = true"].join("\n"),
    );

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
        "The selected Yazelix terminal command is available"
    );
}

// Defends: startup-launch-preflight.evaluate returns a single startup summary without Nu-side check selection.
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
#[test]
fn install_ownership_evaluate_from_env_resolves_stable_profile_wrapper() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let home = tmp.path().join("home");
    let config_dir = home.join(".config").join("yazelix");
    let xdg_data = home.join(".local").join("share");
    let state_dir = xdg_data.join("yazelix");
    let profile_yzx = home.join(".nix-profile").join("bin").join("yzx");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
    fs::write(config_dir.join("yazelix.toml"), "[core]\n").unwrap();
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

// Defends: terminal-materialization.generate resolves the active packaged terminal from runtime metadata.
#[test]
fn terminal_materialization_generate_from_env_writes_generated_configs() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);

    write_managed_config_toml(
        &fixture,
        &["[terminal]", "transparency = \"low\""].join("\n"),
    );
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

    let output = runtime_materialization_command(&fixture, "terminal-materialization.generate")
        .arg("--from-env")
        .env_remove("YAZELIX_TERMINAL_PROFILE")
        .env_remove("YAZELIX_TERMINAL_EFFECTS")
        .env_remove("YAZELIX_TERMINAL_EMOJI_FONT")
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
        !fixture
            .state_dir
            .join("configs/terminal_emulators/rio")
            .exists()
    );
    assert!(
        !fixture
            .state_dir
            .join("configs/terminal_emulators/yzxterm")
            .exists()
    );
    assert!(
        !fixture
            .state_dir
            .join("configs/terminal_emulators/ratty")
            .exists()
    );
    assert!(
        !fixture
            .state_dir
            .join("configs/terminal_emulators/kitty")
            .exists()
    );
    assert!(
        !fixture
            .state_dir
            .join("configs/terminal_emulators/foot")
            .exists()
    );
}

// Defends: vanilla Rio runtime metadata materializes a Rio-native config at the path launch binds through RIO_CONFIG_HOME.
#[test]
fn terminal_materialization_rio_uses_rio_config_toml() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "rio\n").unwrap();

    write_managed_config_toml(
        &fixture,
        &["[terminal]", "transparency = \"low\""].join("\n"),
    );
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
    assert_eq!(envelope["data"]["generated"][0]["terminal"], "rio");

    let rio_config = fs::read_to_string(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("rio")
            .join("config.toml"),
    )
    .unwrap();
    assert!(rio_config.contains("placeholder = \"Yazelix - Rio\""));
    assert!(rio_config.contains("content = \"{{ TITLE || RELATIVE_PATH }}\""));
    assert!(rio_config.contains("opacity = 0.90"));
    assert!(rio_config.contains("opacity-cells = true"));
    assert!(rio_config.contains("[effects]\ntrail-cursor = true"));
    assert!(rio_config.contains("mode = \"Plain\""));
}

// Defends: Linux Foot runtime metadata materializes a Foot-native config at the active launch path.
#[test]
fn terminal_materialization_foot_uses_foot_ini() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "foot\n").unwrap();

    write_managed_config_toml(
        &fixture,
        &["[terminal]", "transparency = \"low\""].join("\n"),
    );
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
    assert_eq!(envelope["data"]["generated"][0]["terminal"], "foot");

    let foot_config = fs::read_to_string(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("foot")
            .join("foot.ini"),
    )
    .unwrap();
    assert!(foot_config.contains("font=FiraCode Nerd Font:size=14"));
    assert!(foot_config.contains("alpha=0.90"));
    assert!(foot_config.contains("[csd]"));
    assert!(foot_config.contains("preferred=none"));
    assert!(foot_config.contains("size=0"));
    assert!(foot_config.contains("[colors-dark]"));
}

// Regression: yzxterm-only sessions keep active cursor color without injecting cursor shaders.
#[test]
fn terminal_materialization_yzxterm_only_uses_rio_trail_without_cursor_shaders() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "yzxterm\n").unwrap();

    write_managed_config_toml(
        &fixture,
        &["[terminal]", "transparency = \"none\""].join("\n"),
    );
    write_cursor_sidecar(
        &fixture,
        r##"
schema_version = 1
enabled_cursors = ["snow"]

[settings]
trail = "snow"
trail_effect = "warp"
mode_effect = "ripple_rectangle"
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
        .env_remove("YAZELIX_TERMINAL_PROFILE")
        .env_remove("YAZELIX_TERMINAL_EFFECTS")
        .env_remove("YAZELIX_TERMINAL_EMOJI_FONT")
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
    assert_eq!(
        envelope["data"]["cursor"]["cursor_state"]["selected_color"],
        "snow"
    );

    let yzxterm_config = fs::read_to_string(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("yzxterm")
            .join("config.toml"),
    )
    .unwrap();
    assert!(yzxterm_config.contains("cursor = \"#ffffff\""));
    assert!(!yzxterm_config.contains("custom-shader"));
    assert!(!yzxterm_config.contains("cursor_trail_snow.glsl"));
    assert!(!yzxterm_config.contains("generated_effects/warp.glsl"));
    assert!(!yzxterm_config.contains("generated_effects/ripple_rectangle.glsl"));
    assert!(!yzxterm_config.contains("cursor_trail_dusk.glsl"));
}

// Regression: Yazelix-managed yzxterm launches pass YAZELIX_TERMINAL_CONFIG, so the runtime must materialize the requested Rio decoration shader itself.
#[test]
fn terminal_materialization_yzxterm_shader_profile_injects_rio_decoration_shader() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "yzxterm\n").unwrap();

    write_managed_config_toml(
        &fixture,
        &["[terminal]", "transparency = \"medium\""].join("\n"),
    );
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

    let output = runtime_materialization_command(&fixture, "terminal-materialization.generate")
        .env("YAZELIX_TERMINAL_PROFILE", "shaders")
        .env_remove("YAZELIX_TERMINAL_EMOJI_FONT")
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
    assert_eq!(envelope["command"], "terminal-materialization.generate");
    assert_eq!(envelope["status"], "ok");

    let yzxterm_config = fs::read_to_string(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("yzxterm")
            .join("config.toml"),
    )
    .unwrap();
    assert!(yzxterm_config.contains("backend = \"Webgpu\""));
    assert!(yzxterm_config.contains("opacity = 0.85"));
    assert!(yzxterm_config.contains("opacity-cells = true"));
    assert!(yzxterm_config.contains("trail-cursor = true"));
    assert!(yzxterm_config.contains("cursor = \"#3bd17a\""));
    assert!(yzxterm_config.contains("custom-shader = ["));
    assert!(yzxterm_config.contains("cursor_trail_forest.glsl"));
    assert!(!yzxterm_config.contains("generated_effects/tail.glsl"));
    assert!(!yzxterm_config.contains("generated_effects/ripple.glsl"));
    assert!(!yzxterm_config.contains("/nix/store/demo/cursor_trail_dusk.glsl"));
}

// Defends: yzxterm generated configs can select a child-owned emoji font profile root without losing main-owned transparency, cursor color, or shader edits.
#[test]
fn terminal_materialization_yzxterm_emoji_font_selects_child_config_root() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "yzxterm\n").unwrap();

    write_managed_config_toml(
        &fixture,
        &["[terminal]", "transparency = \"medium\""].join("\n"),
    );
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

    let output = runtime_materialization_command(&fixture, "terminal-materialization.generate")
        .env("YAZELIX_TERMINAL_PROFILE", "shaders")
        .env("YAZELIX_TERMINAL_EMOJI_FONT", "twitter")
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

    let yzxterm_config = fs::read_to_string(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("yzxterm")
            .join("config.toml"),
    )
    .unwrap();
    assert!(yzxterm_config.contains("Twitter Color Emoji"));
    assert!(!yzxterm_config.contains("SerenityOS Emoji"));
    assert!(yzxterm_config.contains("opacity = 0.85"));
    assert!(yzxterm_config.contains("cursor = \"#3bd17a\""));
    assert!(yzxterm_config.contains("custom-shader = ["));
    assert!(yzxterm_config.contains("cursor_trail_forest.glsl"));
}

// Defends: mutable settings.jsonc can select the yzxterm child-owned emoji style without depending on a Home Manager launch env override.
#[test]
fn terminal_materialization_yzxterm_emoji_style_selects_child_config_root() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "yzxterm\n").unwrap();

    write_managed_config_toml(
        &fixture,
        &[
            "[terminal]",
            "transparency = \"medium\"",
            "emoji_style = \"serenityos\"",
        ]
        .join("\n"),
    );
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

    let output = runtime_materialization_command(&fixture, "terminal-materialization.generate")
        .env("YAZELIX_TERMINAL_PROFILE", "shaders")
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

    let yzxterm_config = fs::read_to_string(
        fixture
            .state_dir
            .join("configs")
            .join("terminal_emulators")
            .join("yzxterm")
            .join("config.toml"),
    )
    .unwrap();
    assert!(yzxterm_config.contains("SerenityOS Emoji"));
    assert!(!yzxterm_config.contains("Twitter Color Emoji"));
    assert!(yzxterm_config.contains("opacity = 0.85"));
    assert!(yzxterm_config.contains("cursor = \"#3bd17a\""));
    assert!(yzxterm_config.contains("custom-shader = ["));
    assert!(yzxterm_config.contains("cursor_trail_forest.glsl"));
}

// Defends: invalid yzxterm emoji font preset names fail clearly instead of silently using the default package config.
#[test]
fn terminal_materialization_yzxterm_rejects_unknown_emoji_font() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "yzxterm\n").unwrap();
    write_managed_config_toml(&fixture, "[terminal]\n");
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
        .env("YAZELIX_TERMINAL_EMOJI_FONT", "whatsapp")
        .arg("--from-env")
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 64);
    assert_eq!(envelope["command"], "terminal-materialization.generate");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(
        envelope["error"]["message"],
        "Unsupported YAZELIX_TERMINAL_EMOJI_FONT: whatsapp. Use noto, twitter, or serenityos."
    );
}

// Regression: yzxterm shader activation must replace stale copied shader assets after a runtime update instead of reusing the old shader directory.
#[test]
fn terminal_materialization_yzxterm_shader_profile_replaces_stale_shader_assets() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "yzxterm\n").unwrap();

    write_managed_config_toml(&fixture, "[terminal]\n");
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
    let shader_dir = fixture
        .state_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    fs::create_dir_all(&shader_dir).unwrap();
    fs::write(shader_dir.join("stale_only.glsl"), "old runtime shader").unwrap();
    fs::write(
        shader_dir.join("cursor_trail_forest.glsl"),
        "old cursor shader",
    )
    .unwrap();

    let output = runtime_materialization_command(&fixture, "terminal-materialization.generate")
        .env("YAZELIX_TERMINAL_PROFILE", "shaders")
        .env_remove("YAZELIX_TERMINAL_EMOJI_FONT")
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

    assert!(!shader_dir.join("stale_only.glsl").exists());
    let forest_shader = fs::read_to_string(shader_dir.join("cursor_trail_forest.glsl")).unwrap();
    assert!(!forest_shader.contains("old cursor shader"));
}

// Defends: Kitty cursor fallback is controlled by the settings cursor registry's binary kitty_enable_cursor setting.
#[test]
fn terminal_materialization_uses_cursor_sidecar_for_kitty_toggle() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    fs::write(fixture.runtime_dir.join("runtime_variant"), "kitty\n").unwrap();

    write_managed_config_toml(&fixture, "[terminal]\n");
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
    assert!(kitty_config.contains("# cursor_trail 0  # disabled in settings.jsonc"));
}

// Defends: ghostty-materialization.generate can resolve config/runtime/state request roots from process env without Nu path assembly.
#[test]
fn ghostty_materialization_generate_from_env_uses_normalized_config() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let fixture = prepare_runtime_materialization_fixture(&repo, &tmp);
    write_runtime_cursor_shader_assets(
        &fixture
            .runtime_dir
            .join("configs")
            .join("terminal_emulators")
            .join("ghostty")
            .join("shaders"),
    );

    write_managed_config_toml(
        &fixture,
        &["[terminal]", "transparency = \"high\""].join("\n"),
    );
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
    assert!(generated_config.contains(&format!(
        "custom-shader = {}",
        ghostty_dir
            .join("shaders")
            .join("cursor_trail_forest.glsl")
            .display()
    )));
    assert!(!generated_config.contains("custom-shader = ./shaders/"));
    assert!(!generated_config.contains("{name}"));
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

// Defends: doctor-config.evaluate reports stale old config inputs as a config-surface error finding.
#[test]
fn doctor_config_evaluate_reports_stale_old_config_inputs() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");
    let user_config_dir = config_dir.join("user_configs");
    fs::create_dir_all(&user_config_dir).unwrap();
    fs::write(config_dir.join("settings.jsonc"), "{}\n").unwrap();
    fs::write(
        user_config_dir.join("yazelix.toml"),
        "[shell]\ndefault_shell = \"bash\"\n",
    )
    .unwrap();
    fs::write(
        config_dir.join("yazelix.toml"),
        "[shell]\ndefault_shell = \"nu\"\n",
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
    assert_eq!(envelope["status"], "ok");
    assert_eq!(
        envelope["data"]["findings"][0]["message"],
        "Could not reconcile Yazelix config surfaces"
    );
    assert_eq!(envelope["data"]["findings"][0]["status"], "error");
    let details = envelope["data"]["findings"][0]["details"].as_str().unwrap();
    assert!(details.contains("canonical settings:"));
    assert!(details.contains("old flat main:"));
    assert!(details.contains("old nested main:"));
}

// Defends: doctor-config.evaluate preserves the stale-schema warning contract and includes the diagnostic report payload.
#[test]
fn doctor_config_evaluate_reports_stale_schema_warning() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    let mut settings =
        read_settings_jsonc_value(&runtime_dir.join("settings_default.jsonc")).unwrap();
    settings["workspace"]["left_sidebar"]["width_percent"] = json!(99);
    fs::write(
        config_dir.join("settings.jsonc"),
        format!("{}\n", serde_json::to_string_pretty(&settings).unwrap()),
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
        "Using custom settings.jsonc configuration"
    );
    assert_eq!(envelope["data"]["findings"][1]["status"], "warning");
    assert_eq!(
        envelope["data"]["findings"][1]["message"],
        "Stale or unsupported settings.jsonc entries detected (1 issues)"
    );
    assert_eq!(
        envelope["data"]["findings"][1]["config_diagnostic_report"]["issue_count"],
        1
    );
    assert_eq!(
        envelope["data"]["findings"][1]["config_diagnostic_report"]["doctor_diagnostics"][0]["headline"],
        "Invalid config value at workspace.left_sidebar.width_percent"
    );
    let details = envelope["data"]["findings"][1]["details"].as_str().unwrap();
    assert!(details.contains("Config report for:"));
    assert!(details.contains("Issues: 1"));
    assert!(details.contains("Invalid config value at workspace.left_sidebar.width_percent"));
}

// Regression: malformed JSONC must stay on the validation-error path instead of being downgraded into the stale-schema warning row.
#[test]
fn doctor_config_evaluate_keeps_invalid_jsonc_as_error() {
    let repo = repo_root();
    let tmp = tempdir().unwrap();
    let runtime_dir = prepare_doctor_config_runtime_fixture(&repo, &tmp);
    let config_dir = tmp.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("settings.jsonc"), "{ \"editor\": ").unwrap();

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
        "Could not validate settings.jsonc against the current schema"
    );
    assert_eq!(envelope["data"]["findings"][1]["status"], "error");
    assert!(
        envelope["data"]["findings"][1]["details"]
            .as_str()
            .unwrap()
            .contains("Could not parse Yazelix settings JSONC")
    );
}

// Defends: doctor-config.evaluate keeps the default-template doctor row fixable instead of bootstrapping config eagerly.
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
        "Using default configuration (settings_default.jsonc)"
    );
    assert_eq!(envelope["data"]["findings"][0]["fix_available"], true);
}
