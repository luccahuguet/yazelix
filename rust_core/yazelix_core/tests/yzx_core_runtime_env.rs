// Test lane: maintainer

use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::process::Command;
use tempfile::tempdir;

mod support;

use support::commands::yzx_core_command;
use support::envelopes::{error_envelope, ok_envelope};

const YAZELIX_LAZYGIT_CONFIG: &str = "os:\n  edit: '$EDITOR -- {{filename}}'\n  editAtLine: '$EDITOR -- {{filename}}:{{line}}'\n  editAtLineAndWait: '$EDITOR -- {{filename}}:{{line}}'\n  editInTerminal: true\n  openDirInEditor: '$EDITOR -- {{dir}}'\n";

// Defends: runtime-env.compute returns one machine-readable env envelope with filtered PATH entries and managed Helix wrapping.
// Contract: CRCP-002
#[test]
fn runtime_env_compute_prints_machine_readable_env_envelope() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");

    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("bin")).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    let runtime_libexec = runtime_dir.join("libexec");
    let runtime_toolbin = runtime_dir.join("toolbin");
    let runtime_bin = runtime_dir.join("bin");
    let request = json!({
        "runtime_dir": runtime_dir,
        "home_dir": home_dir,
        "current_path": format!(
            "{}:{}:/usr/local/bin:{}:/usr/bin",
            runtime_libexec.to_string_lossy(),
            runtime_toolbin.to_string_lossy(),
            runtime_bin.to_string_lossy(),
        ),
        "helix_external": {
            "binary": "/tmp/custom/bin/hx",
            "runtime_path": "/tmp/helix-runtime"
        },
    });

    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    let expected_wrapper = runtime_dir
        .join("shells")
        .join("posix")
        .join("yazelix_hx.sh")
        .to_string_lossy()
        .to_string();
    let expected_home = tmp
        .path()
        .join("home")
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("yazi")
        .to_string_lossy()
        .to_string();

    assert_eq!(envelope["schema_version"], 1);
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["editor_kind"], "helix");
    assert_eq!(
        envelope["data"]["path_entries"],
        json!([
            runtime_toolbin.to_string_lossy().to_string(),
            runtime_bin.to_string_lossy().to_string(),
            "/usr/local/bin",
            "/usr/bin"
        ])
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["PATH"],
        envelope["data"]["path_entries"]
    );
    assert_eq!(envelope["data"]["runtime_env"]["EDITOR"], expected_wrapper);
    assert_eq!(envelope["data"]["runtime_env"]["VISUAL"], expected_wrapper);
    assert_eq!(
        envelope["data"]["runtime_env"]["YAZELIX_MANAGED_HELIX_BINARY"],
        "/tmp/custom/bin/hx"
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["HELIX_RUNTIME"],
        "/tmp/helix-runtime"
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["YAZI_CONFIG_HOME"],
        expected_home
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["ZELLIJ_DEFAULT_LAYOUT"],
        "yzx_side"
    );
}

// Defends: runtime-env.compute rejects malformed JSON request payloads with one usage envelope.
// Contract: CRCP-002
#[test]
fn runtime_env_compute_rejects_invalid_request_json() {
    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg("{")
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 64);
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(envelope["error"]["class"], "usage");
    assert_eq!(envelope["error"]["code"], "invalid_request_json");
}

// Defends: custom Helix binaries cannot bypass the required binary/runtime pair contract at runtime-env assembly.
// Contract: CRCP-002
#[test]
fn runtime_env_compute_rejects_bare_custom_helix_binary() {
    let tmp = tempdir().unwrap();
    let request = json!({
        "runtime_dir": tmp.path().join("runtime"),
        "home_dir": tmp.path().join("home"),
        "editor_command": "/tmp/custom/bin/hx"
    });

    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(request.to_string())
        .output()
        .unwrap();

    let envelope: Value = error_envelope(&output, 65);
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "helix_external_required");
}

// Defends: runtime-env.compute can build the canonical runtime env from process env plus optional config JSON without Nu request assembly.
// Contract: CRCP-002
#[test]
fn runtime_env_compute_from_env_accepts_config_json() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");

    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("bin")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    let runtime_toolbin = runtime_dir.join("toolbin");
    let runtime_bin = runtime_dir.join("bin");
    let config_json = json!({
        "helix_external": {
            "binary": "/tmp/managed/bin/hx",
            "runtime_path": "/tmp/managed-helix-runtime"
        },
    });

    let output = yzx_core_command()
        .env_clear()
        .env("HOME", &home_dir)
        .env(
            "PATH",
            format!("{}:{}", runtime_toolbin.display(), runtime_bin.display()),
        )
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .arg("runtime-env.compute")
        .arg("--from-env")
        .arg("--config-json")
        .arg(config_json.to_string())
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    let expected_wrapper = runtime_dir
        .join("shells")
        .join("posix")
        .join("yazelix_hx.sh")
        .to_string_lossy()
        .to_string();

    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["command"], "runtime-env.compute");
    assert_eq!(
        envelope["data"]["runtime_env"]["PATH"],
        json!([
            runtime_toolbin.to_string_lossy().to_string(),
            runtime_bin.to_string_lossy().to_string(),
        ])
    );
    assert_eq!(envelope["data"]["runtime_env"]["EDITOR"], expected_wrapper);
    assert_eq!(envelope["data"]["runtime_env"]["VISUAL"], expected_wrapper);
    assert_eq!(
        envelope["data"]["runtime_env"]["YAZELIX_MANAGED_HELIX_BINARY"],
        "/tmp/managed/bin/hx"
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["ZELLIJ_DEFAULT_LAYOUT"],
        "yzx_side"
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["HELIX_RUNTIME"],
        "/tmp/managed-helix-runtime"
    );
}

// Defends: the shipped Lazygit config uses Yazelix's exported editor command instead of the literal `helix` preset command.
#[test]
fn shipped_lazygit_config_uses_runtime_editor_not_helix_preset() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root");
    let config = fs::read_to_string(repo_root.join("configs/lazygit/yazelix_config.yml")).unwrap();

    assert!(config.contains("edit: '$EDITOR -- {{filename}}'"));
    assert!(config.contains("editAtLine: '$EDITOR -- {{filename}}:{{line}}'"));
    assert!(config.contains("editAtLineAndWait: '$EDITOR -- {{filename}}:{{line}}'"));
    assert!(config.contains("editInTerminal: true"));
    assert!(!config.contains("editPreset"));
    assert!(!config.contains("helix"));
}

// Defends: built-in Lazygit gets Yazelix's runtime editor config without replacing the user's Lazygit config.
#[test]
fn runtime_env_compute_adds_lazygit_base_config_before_user_config() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");
    let runtime_lazygit_config = runtime_dir
        .join("configs")
        .join("lazygit")
        .join("yazelix_config.yml");
    let user_lazygit_config = home_dir.join(".config").join("lazygit").join("config.yml");

    fs::create_dir_all(runtime_lazygit_config.parent().unwrap()).unwrap();
    fs::create_dir_all(user_lazygit_config.parent().unwrap()).unwrap();
    fs::write(&runtime_lazygit_config, YAZELIX_LAZYGIT_CONFIG).unwrap();
    fs::write(&user_lazygit_config, "gui:\n  showIcons: true\n").unwrap();

    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(
            json!({
                "runtime_dir": runtime_dir,
                "home_dir": home_dir
            })
            .to_string(),
        )
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    assert_eq!(
        envelope["data"]["runtime_env"]["LG_CONFIG_FILE"],
        format!(
            "{},{}",
            runtime_lazygit_config.to_string_lossy(),
            user_lazygit_config.to_string_lossy()
        )
    );
}

// Defends: explicit Lazygit config-file lists are preserved after the Yazelix base config so user overrides can still win.
#[test]
fn runtime_env_compute_preserves_existing_lazygit_config_file_list() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");
    let runtime_lazygit_config = runtime_dir
        .join("configs")
        .join("lazygit")
        .join("yazelix_config.yml");

    fs::create_dir_all(runtime_lazygit_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&home_dir).unwrap();
    fs::write(&runtime_lazygit_config, YAZELIX_LAZYGIT_CONFIG).unwrap();

    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(
            json!({
                "runtime_dir": runtime_dir,
                "home_dir": home_dir,
                "current_lazygit_config_file": "/tmp/base.yml,/tmp/theme.yml"
            })
            .to_string(),
        )
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    assert_eq!(
        envelope["data"]["runtime_env"]["LG_CONFIG_FILE"],
        format!(
            "{},/tmp/base.yml,/tmp/theme.yml",
            runtime_lazygit_config.to_string_lossy()
        )
    );
}

// Defends: non-Helix editor.command values must not be overridden by Yazelix's LazyGit Helix preset.
#[test]
fn runtime_env_compute_does_not_force_lazygit_helix_for_neovim() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");
    let runtime_lazygit_config = runtime_dir
        .join("configs")
        .join("lazygit")
        .join("yazelix_config.yml");
    let user_lazygit_config = home_dir.join(".config").join("lazygit").join("config.yml");

    fs::create_dir_all(runtime_lazygit_config.parent().unwrap()).unwrap();
    fs::create_dir_all(user_lazygit_config.parent().unwrap()).unwrap();
    fs::write(&runtime_lazygit_config, YAZELIX_LAZYGIT_CONFIG).unwrap();
    fs::write(&user_lazygit_config, "gui:\n  showIcons: true\n").unwrap();

    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(
            json!({
                "runtime_dir": runtime_dir,
                "home_dir": home_dir,
                "editor_command": "nvim"
            })
            .to_string(),
        )
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    assert_eq!(envelope["data"]["editor_kind"], "neovim");
    assert_eq!(envelope["data"]["runtime_env"]["EDITOR"], "nvim");
    assert_eq!(envelope["data"]["runtime_env"]["VISUAL"], "nvim");
    assert!(envelope["data"]["runtime_env"]["LG_CONFIG_FILE"].is_null());
}

// Defends: default bundled Helix uses the private raw binary behind the public managed `hx` wrapper.
#[test]
fn runtime_env_compute_points_default_helix_wrapper_at_private_binary() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");
    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("bin")).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    let output = yzx_core_command()
        .arg("runtime-env.compute")
        .arg("--request-json")
        .arg(
            json!({
                "runtime_dir": runtime_dir,
                "home_dir": home_dir
            })
            .to_string(),
        )
        .output()
        .unwrap();

    let envelope: Value = ok_envelope(&output);
    assert_eq!(
        envelope["data"]["runtime_env"]["EDITOR"],
        tmp.path()
            .join("runtime/shells/posix/yazelix_hx.sh")
            .to_string_lossy()
            .to_string()
    );
    assert_eq!(
        envelope["data"]["runtime_env"]["YAZELIX_MANAGED_HELIX_BINARY"],
        tmp.path()
            .join("runtime/libexec/hx")
            .to_string_lossy()
            .to_string()
    );
}

#[cfg(unix)]
// Defends: the managed Helix wrapper passes Yazelix's generated config directory to Helix, so native ~/.config/helix cannot win.
#[test]
fn managed_helix_wrapper_passes_generated_config_dir() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let config_dir = tmp.path().join("config").join("yazelix");
    let state_dir = tmp.path().join("state").join("yazelix");
    let generated_dir = state_dir.join("configs").join("helix");
    let generated_config = generated_dir.join("config.toml");
    let managed_helix_config_dir = config_dir.join("helix");
    let log_path = tmp.path().join("hx.log");
    let core_log_path = tmp.path().join("yzx_core.log");
    let stale_core_log_path = tmp.path().join("stale_yzx_core.log");

    fs::create_dir_all(runtime_dir.join("shells/posix")).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(&generated_dir).unwrap();
    fs::write(&generated_config, "theme = \"default\"\n").unwrap();

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("repo root");
    fs::copy(
        repo_root.join("shells/posix/yazelix_hx.sh"),
        runtime_dir.join("shells/posix/yazelix_hx.sh"),
    )
    .unwrap();
    make_executable(&runtime_dir.join("shells/posix/yazelix_hx.sh"));

    fs::write(
        runtime_dir.join("libexec/yzx_core"),
        format!(
            "#!/bin/sh\n: > '{}'\nfor arg in \"$@\"; do printf 'core-arg=%s\\n' \"$arg\" >> '{}'; done\nprintf '%s\\n' '{{\"data\":{{\"import_notice\":{{\"lines\":[]}},\"generated_path\":\"ignored-by-fake-jq\",\"managed_helix_config_dir\":\"ignored-by-fake-jq\",\"generated_steel_config_dir\":\"ignored-by-fake-jq\"}}}}'\n",
            core_log_path.display(),
            core_log_path.display()
        ),
    )
    .unwrap();
    make_executable(&runtime_dir.join("libexec/yzx_core"));

    let stale_yzx_core = tmp.path().join("stale_yzx_core");
    fs::write(
        &stale_yzx_core,
        format!(
            "#!/bin/sh\nprintf 'used stale core\\n' > '{}'\nexit 99\n",
            stale_core_log_path.display()
        ),
    )
    .unwrap();
    make_executable(&stale_yzx_core);

    fs::write(
        runtime_dir.join("toolbin/jq"),
        format!(
            "#!/bin/sh\ncase \"$2\" in\n  '.data.import_notice.lines[]?') exit 0 ;;\n  '.data.generated_path // \"\"') printf '%s\\n' '{}' ;;\n  '.data.managed_helix_config_dir // \"\"') printf '%s\\n' '{}' ;;\n  '.data.generated_steel_config_dir // \"\"') printf '%s\\n' '{}' ;;\n  *) exit 1 ;;\nesac\n",
            generated_config.display(),
            managed_helix_config_dir.display(),
            generated_dir.display()
        ),
    )
    .unwrap();
    make_executable(&runtime_dir.join("toolbin/jq"));

    let fake_hx = tmp.path().join("hx");
    fs::write(
        &fake_hx,
        format!(
            "#!/bin/sh\nprintf 'HELIX_STEEL_CONFIG=%s\\n' \"$HELIX_STEEL_CONFIG\" > '{}'\nprintf 'arg=%s\\n' \"$@\" >> '{}'\n",
            log_path.display(),
            log_path.display()
        ),
    )
    .unwrap();
    make_executable(&fake_hx);

    let output = Command::new(runtime_dir.join("shells/posix/yazelix_hx.sh"))
        .env_clear()
        .env("HOME", tmp.path().join("home"))
        .env("PATH", "/usr/bin:/bin")
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .env("YAZELIX_CONFIG_DIR", &config_dir)
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("YAZELIX_MANAGED_HELIX_BINARY", &fake_hx)
        .env("YAZELIX_YZX_CORE_BIN", &stale_yzx_core)
        .arg("README.md")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wrapper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&log_path).unwrap(),
        format!(
            "HELIX_STEEL_CONFIG={}\narg=--config-dir\narg={}\narg=-c\narg={}\narg=README.md\n",
            generated_dir.display(),
            managed_helix_config_dir.display(),
            generated_config.display()
        )
    );
    assert!(
        fs::read_to_string(&core_log_path)
            .unwrap()
            .contains("core-arg=--show-splash\ncore-arg=false\n")
    );
    assert!(!stale_core_log_path.exists());

    let output = Command::new(runtime_dir.join("shells/posix/yazelix_hx.sh"))
        .env_clear()
        .env("HOME", tmp.path().join("home"))
        .env("PATH", "/usr/bin:/bin")
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .env("YAZELIX_CONFIG_DIR", &config_dir)
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("YAZELIX_MANAGED_HELIX_BINARY", &fake_hx)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wrapper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&log_path).unwrap(),
        format!(
            "HELIX_STEEL_CONFIG={}\narg=--config-dir\narg={}\narg=-c\narg={}\n",
            generated_dir.display(),
            managed_helix_config_dir.display(),
            generated_config.display()
        )
    );
    assert!(
        fs::read_to_string(&core_log_path)
            .unwrap()
            .contains("core-arg=--show-splash\ncore-arg=true\n")
    );

    let project_dir = tmp.path().join("project");
    fs::create_dir_all(&project_dir).unwrap();
    let output = Command::new(runtime_dir.join("shells/posix/yazelix_hx.sh"))
        .env_clear()
        .env("HOME", tmp.path().join("home"))
        .env("PATH", "/usr/bin:/bin")
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .env("YAZELIX_CONFIG_DIR", &config_dir)
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("YAZELIX_MANAGED_HELIX_BINARY", &fake_hx)
        .arg(&project_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "wrapper failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&log_path).unwrap(),
        format!(
            "HELIX_STEEL_CONFIG={}\narg=--config-dir\narg={}\narg=-c\narg={}\narg={}\n",
            generated_dir.display(),
            managed_helix_config_dir.display(),
            generated_config.display(),
            project_dir.display()
        )
    );
    assert!(
        fs::read_to_string(&core_log_path)
            .unwrap()
            .contains("core-arg=--show-splash\ncore-arg=false\n")
    );
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
