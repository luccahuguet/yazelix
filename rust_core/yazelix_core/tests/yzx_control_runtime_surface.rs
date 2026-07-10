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

fn assert_security_wrapper_path(stdout: &str, wrapper_dir: &Path) {
    let wrapper_dir = wrapper_dir.to_string_lossy();
    let resolved_sudo = stdout
        .lines()
        .find_map(|line| line.strip_prefix("sudo="))
        .expect("capture should include resolved sudo");
    assert_eq!(resolved_sudo, format!("{wrapper_dir}/sudo"));

    let path = stdout
        .lines()
        .find_map(|line| line.strip_prefix("PATH="))
        .expect("capture should include PATH");
    let entries = path.split(':').collect::<Vec<_>>();
    assert_eq!(entries.first().copied(), Some(wrapper_dir.as_ref()));
    assert_eq!(
        entries
            .iter()
            .filter(|entry| **entry == wrapper_dir.as_ref())
            .count(),
        1
    );
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

// Regression: detached macOS launchers can inherit a PATH without dirname/readlink, so POSIX
// bootstrap must seed system tool dirs before runtime_env.sh is sourced.
// Regression: NixOS security wrappers must remain ahead of the raw current-system programs after
// every POSIX entrypoint finishes bootstrap.
#[test]
fn posix_bootstrap_entrypoints_resolve_runtime_and_preserve_security_wrappers() {
    let repo = repo_root();
    let temp = tempfile::tempdir().unwrap();
    let runtime_dir = temp.path().join("runtime");
    let home_dir = temp.path().join("home");
    let posix_dir = runtime_dir.join("shells").join("posix");
    let narrow_path = temp.path().join("private_tmp");
    let raw_system_bin = temp.path().join("run/current-system/sw/bin");
    let wrapper_dir = temp.path().join("run/wrappers/bin");

    fs::create_dir_all(&posix_dir).unwrap();
    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&narrow_path).unwrap();
    fs::create_dir_all(&raw_system_bin).unwrap();
    fs::create_dir_all(&wrapper_dir).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(runtime_dir.join("toolbin")).unwrap();
    fs::create_dir_all(runtime_dir.join("nushell/config")).unwrap();
    write_executable_script(&raw_system_bin.join("sudo"), "#!/bin/sh\nexit 99\n");
    write_executable_script(&wrapper_dir.join("sudo"), "#!/bin/sh\nexit 0\n");

    let entrypoint_source = |name: &str| {
        fs::read_to_string(repo.join("shells/posix").join(name))
            .unwrap()
            .replace(
                "/run/current-system/sw/bin",
                &raw_system_bin.to_string_lossy(),
            )
    };
    write_executable_script(
        &posix_dir.join("yzx_cli.sh"),
        &entrypoint_source("yzx_cli.sh"),
    );
    write_executable_script(
        &posix_dir.join("start_yazelix.sh"),
        &entrypoint_source("start_yazelix.sh"),
    );
    write_executable_script(
        &posix_dir.join("yazelix_nu.sh"),
        &entrypoint_source("yazelix_nu.sh"),
    );
    write_executable_script(
        &posix_dir.join("yazelix_hx.sh"),
        &entrypoint_source("yazelix_hx.sh"),
    );
    fs::write(
        posix_dir.join("runtime_env.sh"),
        fs::read_to_string(repo.join("shells/posix/runtime_env.sh"))
            .unwrap()
            .replace("/run/wrappers/bin", &wrapper_dir.to_string_lossy()),
    )
    .unwrap();
    fs::write(runtime_dir.join("nushell/config/config.nu"), "").unwrap();
    fs::write(runtime_dir.join("nushell/config/stack_prompt_guard.nu"), "").unwrap();

    let yzx_capture = temp.path().join("capture_yzx.sh");
    write_executable_script(
        &yzx_capture,
        r#"#!/bin/sh
printf 'yzx_argv=%s\n' "${1:-}"
printf 'YAZELIX_RUNTIME_DIR=%s\n' "$YAZELIX_RUNTIME_DIR"
printf 'sudo=%s\n' "$(command -v sudo)"
printf 'PATH=%s\n' "$PATH"
"#,
    );
    let yzx_output = Command::new(posix_dir.join("yzx_cli.sh"))
        .env_clear()
        .env("HOME", &home_dir)
        .env("USER", "yazelix-test")
        .env("PATH", &narrow_path)
        .env("YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT", "1")
        .env("YAZELIX_YZX_BIN", &yzx_capture)
        .arg("--version")
        .output()
        .unwrap();

    assert_eq!(
        yzx_output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&yzx_output.stderr)
    );
    let yzx_stdout = String::from_utf8(yzx_output.stdout).unwrap();
    assert!(yzx_stdout.contains("yzx_argv=--version"));
    assert!(yzx_stdout.contains(&format!(
        "YAZELIX_RUNTIME_DIR={}",
        runtime_dir.to_string_lossy()
    )));
    assert_security_wrapper_path(&yzx_stdout, &wrapper_dir);

    let control_capture = temp.path().join("capture_control.sh");
    write_executable_script(
        &control_capture,
        r#"#!/bin/sh
printf 'control_argv=%s\n' "${1:-}"
printf 'YAZELIX_RUNTIME_DIR=%s\n' "$YAZELIX_RUNTIME_DIR"
printf 'sudo=%s\n' "$(command -v sudo)"
printf 'PATH=%s\n' "$PATH"
"#,
    );
    let start_output = Command::new(posix_dir.join("start_yazelix.sh"))
        .env_clear()
        .env("HOME", &home_dir)
        .env("USER", "yazelix-test")
        .env("PATH", &narrow_path)
        .env("YAZELIX_YZX_CONTROL_BIN", &control_capture)
        .output()
        .unwrap();

    assert_eq!(
        start_output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&start_output.stderr)
    );
    let start_stdout = String::from_utf8(start_output.stdout).unwrap();
    assert!(start_stdout.contains("control_argv=enter"));
    assert!(start_stdout.contains(&format!(
        "YAZELIX_RUNTIME_DIR={}",
        runtime_dir.to_string_lossy()
    )));
    assert_security_wrapper_path(&start_stdout, &wrapper_dir);

    write_executable_script(
        &runtime_dir.join("libexec/nu"),
        r#"#!/bin/sh
printf 'nu_argv=%s\n' "$*"
printf 'YAZELIX_RUNTIME_DIR=%s\n' "$YAZELIX_RUNTIME_DIR"
printf 'sudo=%s\n' "$(command -v sudo)"
printf 'PATH=%s\n' "$PATH"
"#,
    );
    let nu_output = Command::new(posix_dir.join("yazelix_nu.sh"))
        .env_clear()
        .env("HOME", &home_dir)
        .env("USER", "yazelix-test")
        .env("PATH", &narrow_path)
        .arg("--version")
        .output()
        .unwrap();

    assert_eq!(
        nu_output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&nu_output.stderr)
    );
    let nu_stdout = String::from_utf8(nu_output.stdout).unwrap();
    assert!(nu_stdout.contains("nu_argv=--login --env-config /dev/null --config "));
    assert!(nu_stdout.contains("--version"));
    assert!(nu_stdout.contains(&format!(
        "YAZELIX_RUNTIME_DIR={}",
        runtime_dir.to_string_lossy()
    )));
    assert_security_wrapper_path(&nu_stdout, &wrapper_dir);

    write_executable_script(
        &runtime_dir.join("libexec/yzx_core"),
        r#"#!/bin/sh
printf '%s\n' '{"data":{"import_notice":{"lines":[]},"generated_path":"/tmp/generated-helix.toml","generated_steel_config_dir":"/tmp/generated-steel","managed_helix_config_dir":"/tmp/managed-helix"}}'
"#,
    );
    write_executable_script(
        &runtime_dir.join("toolbin/jq"),
        r#"#!/bin/sh
case "$2" in
  '.data.import_notice.lines[]?') exit 0 ;;
  '.data.generated_path // ""') printf '%s\n' '/tmp/generated-helix.toml' ;;
  '.data.generated_steel_config_dir // ""') printf '%s\n' '/tmp/generated-steel' ;;
  '.data.managed_helix_config_dir // ""') printf '%s\n' '/tmp/managed-helix' ;;
  *) printf 'unexpected jq args: %s\n' "$*" >&2; exit 1 ;;
esac
"#,
    );
    write_executable_script(
        &runtime_dir.join("libexec/hx"),
        r#"#!/bin/sh
printf 'hx_argv=%s\n' "$*"
printf 'sudo=%s\n' "$(command -v sudo)"
printf 'PATH=%s\n' "$PATH"
"#,
    );
    let hx_output = Command::new(posix_dir.join("yazelix_hx.sh"))
        .env_clear()
        .env("HOME", &home_dir)
        .env("USER", "yazelix-test")
        .env("PATH", &narrow_path)
        .env(
            "YAZELIX_MANAGED_HELIX_BINARY",
            runtime_dir.join("libexec/hx"),
        )
        .arg("--version")
        .output()
        .unwrap();

    assert_eq!(
        hx_output.status.code(),
        Some(0),
        "stderr:\n{}",
        String::from_utf8_lossy(&hx_output.stderr)
    );
    let hx_stdout = String::from_utf8(hx_output.stdout).unwrap();
    assert!(hx_stdout.contains("hx_argv=--config-dir /tmp/managed-helix"));
    assert!(hx_stdout.contains("-c /tmp/generated-helix.toml --version"));
    assert_security_wrapper_path(&hx_stdout, &wrapper_dir);
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

// Regression: `yzx enter` uses the actual host terminal for the status bar instead of the packaged runtime label.
#[test]
fn yzx_enter_uses_detected_host_terminal_for_status_bar_label() {
    let fixture = managed_config_fixture("");
    fs::write(fixture.runtime_dir.join("runtime_variant"), "mars\n").unwrap();
    fs::write(
        fixture.runtime_dir.join("runtime_identity.json"),
        r#"{"schema_version":1,"version":"v-test","runtime_variant":"mars"}"#,
    )
    .unwrap();
    seed_startup_materialization_runtime_assets(&fixture);

    let fake_bin = fixture.home_dir.join("fake-bin");
    let workspace = fixture.home_dir.join("workspace");
    let zellij_log = fixture.home_dir.join("zellij-startup.log");
    let bar_request = fixture.home_dir.join("zellij-bar-request.json");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&workspace).unwrap();

    write_executable_script(
        &fixture
            .runtime_dir
            .join("libexec")
            .join("yazelix_zellij_bar_widget"),
        &format!(
            "#!/bin/sh\n[ \"$1\" = \"render-yazelix-runtime\" ] || exit 11\n[ \"$2\" = \"--json\" ] || exit 12\nprintf '%s' \"$3\" > '{}'\nprintf '%s\\n' '{{\"schema_version\":3,\"plugin_block\":\"CHILD_PLUGIN_BLOCK\"}}'\n",
            bar_request.display()
        ),
    );
    write_executable_script(
        &fake_bin.join("zellij"),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"setup\" ] && [ \"$2\" = \"--dump-config\" ]; then exit 0; fi\nprintf 'YAZELIX_SESSION_TERMINAL=%s\\n' \"${{YAZELIX_SESSION_TERMINAL-unset}}\" > '{}'\nexit 0\n",
            zellij_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .env("TERM_PROGRAM", "WezTerm")
        .env("YAZELIX_STARTUP_PROFILE_SKIP_WELCOME", "true")
        .arg("enter")
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
    let request: Value = serde_json::from_str(&fs::read_to_string(bar_request).unwrap()).unwrap();
    assert_eq!(request["terminal_label"], "wezterm");
    assert_ne!(request["terminal_label"], "mars");

    let log = fs::read_to_string(zellij_log).unwrap();
    assert!(log.contains("YAZELIX_SESSION_TERMINAL=wezterm"), "{log}");
}

// Defends: host Ghostty users can generate the cursor include from the normal Yazelix package without installing the standalone cursor package.
#[test]
fn yzx_cursors_ghostty_setup_uses_runtime_private_yzc() {
    let fixture = managed_config_fixture("");
    let shader_root = fixture
        .runtime_dir
        .join("configs/terminal_emulators/ghostty/shaders");
    fs::create_dir_all(&shader_root).unwrap();
    let yzc_log = fixture.home_dir.join("yzc.log");
    write_executable_script(
        &fixture.runtime_dir.join("libexec/yzc"),
        &format!(
            r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "{}"
config_dir=
share_dir=
while [ "$#" -gt 0 ]; do
  case "$1" in
    --config-dir)
      config_dir="$2"
      shift 2
      ;;
    --share-dir)
      share_dir="$2"
      shift 2
      ;;
    generate)
      test "$2" = ghostty
      test -d "$share_dir/shaders"
      mkdir -p "$config_dir"
      printf '# generated ghostty include\n' > "$config_dir/ghostty.conf"
      exit 0
      ;;
    init)
      mkdir -p "$config_dir"
      printf '{{}}\n' > "$config_dir/settings.jsonc"
      exit 0
      ;;
    *)
      echo "unexpected yzc arg: $1" >&2
      exit 99
      ;;
  esac
done
exit 99
"#,
            yzc_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .args(["cursors", "ghostty", "setup"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let cursor_dir = fixture.home_dir.join(".config/yazelix_cursors");
    let include_path = cursor_dir.join("ghostty.conf");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Ghostty cursor include generated:"),
        "{stdout}"
    );
    assert!(
        stdout.contains(&format!("config-file = {}", include_path.display())),
        "{stdout}"
    );
    assert!(include_path.exists());

    let log = fs::read_to_string(yzc_log).unwrap();
    assert!(
        log.contains(&format!(
            "--config-dir {} --share-dir {} generate ghostty",
            cursor_dir.display(),
            fixture
                .runtime_dir
                .join("configs/terminal_emulators/ghostty")
                .display()
        )),
        "{log}"
    );
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
    assert_eq!(summary["terminals"], serde_json::json!(["mars"]));
    assert!(summary["generated_state_repair_needed"].is_boolean());
    assert!(summary["generated_state_materialization_status"].is_string());
    assert_eq!(summary["session_config_snapshot"]["status"], "not_set");
}

// Regression: status stays usable from an older running window whose live config contains newer unsupported fields.
#[test]
fn yzx_control_status_json_reports_config_problem_without_aborting() {
    let fixture = managed_config_fixture(
        r#"[editor]
future_option = true
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
    assert_eq!(report["runtime"]["variant"], "mars");
    assert_eq!(report["runtime"]["variant_source"], "runtime_identity_json");
    assert_eq!(
        report["runtime"]["invoked_yzx_path"],
        "/nix/store/example-yazelix/bin/yzx"
    );
    assert_eq!(
        report["self_description"]["query_surface"],
        "yzx inspect --json"
    );
    assert_eq!(
        report["config"]["session_config_snapshot"]["status"],
        "not_set"
    );
    assert_eq!(report["session"]["available"], false);
    assert_eq!(report["session"]["reason"], "not_in_zellij");
    assert_eq!(report["install"]["install_owner"], "manual");
}

// Regression: inspect remains the diagnostic escape hatch when config validation is what failed.
#[test]
fn yzx_control_inspect_json_embeds_config_problem_without_aborting() {
    let fixture = managed_config_fixture(
        r#"[editor]
future_option = true
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
