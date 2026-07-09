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

use yazelix_core::{RuntimeEnvComputeData, RuntimeEnvComputeRequest, compute_runtime_env};

const YAZELIX_LAZYGIT_CONFIG: &str = "os:\n  edit: '$EDITOR -- {{filename}}'\n  editAtLine: '$EDITOR -- {{filename}}:{{line}}'\n  editAtLineAndWait: '$EDITOR -- {{filename}}:{{line}}'\n  editInTerminal: true\n  openDirInEditor: '$EDITOR -- {{dir}}'\n";

fn runtime_env_data(request: Value) -> RuntimeEnvComputeData {
    let request: RuntimeEnvComputeRequest = serde_json::from_value(request).unwrap();
    compute_runtime_env(&request).unwrap()
}

// Defends: runtime env computation returns filtered PATH entries and managed Helix wrapping.
// Contract: CRCP-002
#[test]
fn runtime_env_compute_returns_filtered_env() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let home_dir = tmp.path().join("home");

    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("bin")).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(home_dir.join(".nix-profile").join("bin")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    let runtime_libexec = runtime_dir.join("libexec");
    let runtime_toolbin = runtime_dir.join("toolbin");
    let runtime_bin = runtime_dir.join("bin");
    let nix_profile_bin = home_dir.join(".nix-profile").join("bin");
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

    let data = runtime_env_data(request);
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

    assert_eq!(data.editor_kind, "helix");
    assert_eq!(data.path_entries[0], nix_profile_bin.to_string_lossy());
    let runtime_toolbin_index = data
        .path_entries
        .iter()
        .position(|entry| entry == runtime_toolbin.to_string_lossy().as_ref())
        .unwrap();
    let runtime_bin_index = data
        .path_entries
        .iter()
        .position(|entry| entry == runtime_bin.to_string_lossy().as_ref())
        .unwrap();
    assert!(runtime_toolbin_index < runtime_bin_index);
    assert!(
        !data
            .path_entries
            .contains(&runtime_libexec.to_string_lossy().to_string())
    );
    let usr_local_bin_index = data
        .path_entries
        .iter()
        .position(|entry| entry == "/usr/local/bin")
        .unwrap();
    let usr_bin_index = data
        .path_entries
        .iter()
        .position(|entry| entry == "/usr/bin")
        .unwrap();
    assert!(usr_local_bin_index < usr_bin_index);
    assert_eq!(data.runtime_env["PATH"], json!(data.path_entries));
    assert_eq!(data.runtime_env["EDITOR"], expected_wrapper);
    assert_eq!(data.runtime_env["VISUAL"], expected_wrapper);
    assert_eq!(
        data.runtime_env["YAZELIX_MANAGED_HELIX_BINARY"],
        "/tmp/custom/bin/hx"
    );
    assert_eq!(data.runtime_env["HELIX_RUNTIME"], "/tmp/helix-runtime");
    assert_eq!(data.runtime_env["YAZI_CONFIG_HOME"], expected_home);
    assert_eq!(data.runtime_env["ZELLIJ_DEFAULT_LAYOUT"], "yzx_side");
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

    let request: RuntimeEnvComputeRequest = serde_json::from_value(request).unwrap();
    let error = compute_runtime_env(&request).unwrap_err();

    assert_eq!(error.class().as_str(), "config");
    assert_eq!(error.code(), "helix_external_required");
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

    let data = runtime_env_data(json!({
        "runtime_dir": runtime_dir,
        "home_dir": home_dir
    }));
    assert_eq!(
        data.runtime_env["LG_CONFIG_FILE"],
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

    let data = runtime_env_data(json!({
        "runtime_dir": runtime_dir,
        "home_dir": home_dir,
        "current_lazygit_config_file": "/tmp/base.yml,/tmp/theme.yml"
    }));
    assert_eq!(
        data.runtime_env["LG_CONFIG_FILE"],
        format!(
            "{},/tmp/base.yml,/tmp/theme.yml",
            runtime_lazygit_config.to_string_lossy()
        )
    );
}

// Defends: updated runtimes must not inherit stale Yazelix LazyGit config files from already-open shells.
#[test]
fn runtime_env_compute_strips_inherited_yazelix_lazygit_configs() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let old_runtime_dir = tmp.path().join("old_runtime");
    let home_dir = tmp.path().join("home");
    let runtime_lazygit_config = runtime_dir
        .join("configs")
        .join("lazygit")
        .join("yazelix_config.yml");
    let old_runtime_lazygit_config = old_runtime_dir
        .join("configs")
        .join("lazygit")
        .join("yazelix_config.yml");
    let user_lazygit_config = home_dir.join(".config").join("lazygit").join("config.yml");

    fs::create_dir_all(runtime_lazygit_config.parent().unwrap()).unwrap();
    fs::create_dir_all(old_runtime_lazygit_config.parent().unwrap()).unwrap();
    fs::create_dir_all(user_lazygit_config.parent().unwrap()).unwrap();
    fs::write(&runtime_lazygit_config, YAZELIX_LAZYGIT_CONFIG).unwrap();
    fs::write(&old_runtime_lazygit_config, "os:\n  editPreset: helix\n").unwrap();
    fs::write(&user_lazygit_config, "gui:\n  showIcons: true\n").unwrap();

    let data = runtime_env_data(json!({
        "runtime_dir": runtime_dir,
        "home_dir": home_dir,
        "current_lazygit_config_file": format!(
            "{},{},{}",
            old_runtime_lazygit_config.to_string_lossy(),
            old_runtime_lazygit_config.to_string_lossy(),
            user_lazygit_config.to_string_lossy(),
        )
    }));
    assert_eq!(
        data.runtime_env["LG_CONFIG_FILE"],
        format!(
            "{},{}",
            runtime_lazygit_config.to_string_lossy(),
            user_lazygit_config.to_string_lossy()
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

    let data = runtime_env_data(json!({
        "runtime_dir": runtime_dir,
        "home_dir": home_dir,
        "editor_command": "nvim"
    }));

    assert_eq!(data.editor_kind, "neovim");
    assert_eq!(data.runtime_env["EDITOR"], "nvim");
    assert_eq!(data.runtime_env["VISUAL"], "nvim");
    assert!(data.runtime_env.get("LG_CONFIG_FILE").is_none());
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

    let data = runtime_env_data(json!({
        "runtime_dir": runtime_dir,
        "home_dir": home_dir
    }));
    assert_eq!(
        data.runtime_env["EDITOR"],
        tmp.path()
            .join("runtime/shells/posix/yazelix_hx.sh")
            .to_string_lossy()
            .to_string()
    );
    assert_eq!(
        data.runtime_env["YAZELIX_MANAGED_HELIX_BINARY"],
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

    let fake_hx = runtime_dir.join("libexec").join("hx");
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
// Defends: helix.external is an integrated Yazelix-Helix fork surface; upstream Helix fails clearly because it does not support --config-dir.
#[test]
fn managed_helix_wrapper_rejects_external_binary_without_yazelix_config_dir_support() {
    let tmp = tempdir().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let config_dir = tmp.path().join("config").join("yazelix");
    let state_dir = tmp.path().join("state").join("yazelix");
    let generated_dir = state_dir.join("configs").join("helix");
    let generated_config = generated_dir.join("config.toml");
    let managed_helix_config_dir = config_dir.join("helix");

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
        "#!/bin/sh\nprintf '%s\\n' '{\"data\":{\"import_notice\":{\"lines\":[]},\"generated_path\":\"ignored-by-fake-jq\",\"managed_helix_config_dir\":\"ignored-by-fake-jq\",\"generated_steel_config_dir\":\"ignored-by-fake-jq\"}}'\n",
    )
    .unwrap();
    make_executable(&runtime_dir.join("libexec/yzx_core"));

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

    let upstream_hx = tmp.path().join("upstream-hx");
    fs::write(
        &upstream_hx,
        "#!/bin/sh\nprintf '%s\\n' 'Error: could not parse arguments' >&2\nprintf '%s\\n' 'Caused by:' >&2\nprintf '%s\\n' '    unexpected double dash argument: --config-dir' >&2\nexit 1\n",
    )
    .unwrap();
    make_executable(&upstream_hx);

    let output = Command::new(runtime_dir.join("shells/posix/yazelix_hx.sh"))
        .env_clear()
        .env("HOME", tmp.path().join("home"))
        .env("PATH", "/usr/bin:/bin")
        .env("YAZELIX_RUNTIME_DIR", &runtime_dir)
        .env("YAZELIX_CONFIG_DIR", &config_dir)
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("YAZELIX_MANAGED_HELIX_BINARY", &upstream_hx)
        .arg("README.md")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("helix.external points at a Helix binary that is not Yazelix-compatible"),
        "{stderr}"
    );
    assert!(
        stderr.contains("Vanilla/upstream Helix does not support Yazelix's --config-dir option"),
        "{stderr}"
    );
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
