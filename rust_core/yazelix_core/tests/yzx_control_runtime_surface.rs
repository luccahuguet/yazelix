// Test lane: default

use serde_json::Value;
use std::fs;
use std::process::Command;

mod support;

use support::commands::{apply_managed_config_env, yzx_control_command};
use support::fixtures::{managed_config_fixture, prepend_path, repo_root, write_executable_script};

fn yzx_control_command_in_fixture(
    fixture: &support::fixtures::ManagedConfigFixture,
) -> assert_cmd::Command {
    let mut command = yzx_control_command();
    apply_managed_config_env(&mut command, fixture);
    command
}

fn write_default_profile_manifest(fixture: &support::fixtures::ManagedConfigFixture, raw: &str) {
    let manifest_path = fixture.home_dir.join(".nix-profile").join("manifest.json");
    fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
    fs::write(manifest_path, raw).unwrap();
}

// Regression: workspace startup scrubs inherited GTK/GIO loader variables so host GUI apps do not load incompatible Nix modules.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn start_yazelix_scrubs_gui_loader_env_before_control_handoff() {
    let repo = repo_root();
    let temp = tempfile::tempdir().unwrap();
    let runtime_dir = temp.path().join("runtime");
    let home_dir = temp.path().join("home");
    let posix_dir = runtime_dir.join("shells").join("posix");

    fs::create_dir_all(&posix_dir).unwrap();
    fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
    fs::create_dir_all(&home_dir).unwrap();

    write_executable_script(
        &posix_dir.join("start_yazelix.sh"),
        &fs::read_to_string(repo.join("shells/posix/start_yazelix.sh")).unwrap(),
    );
    fs::write(
        posix_dir.join("runtime_env.sh"),
        fs::read_to_string(repo.join("shells/posix/runtime_env.sh")).unwrap(),
    )
    .unwrap();
    write_executable_script(&runtime_dir.join("libexec/nu"), "#!/bin/sh\nexit 0\n");

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
}

// Defends: the public Rust-owned `yzx config --path` route still bootstraps the managed config surface and returns its canonical path.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_config_path_bootstraps_missing_managed_config() {
    let fixture = managed_config_fixture("");
    fs::remove_file(&fixture.managed_config).unwrap();

    let output = yzx_control_command_in_fixture(&fixture)
        .arg("config")
        .arg("--path")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(fixture.managed_config.is_file());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        fixture.managed_config.to_string_lossy()
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Creating yazelix.toml from yazelix_default.toml"));
    assert!(stderr.contains("yazelix.toml created"));
}

// Defends: the public Rust-owned `yzx status --json` surface keeps the typed runtime summary instead of a wrapper-shaped blob.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_status_json_reports_typed_summary() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "nu"

[terminal]
terminals = ["ghostty"]
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
            .ends_with("yazelix.toml")
    );
    assert_eq!(summary["default_shell"], "nu");
    assert_eq!(summary["terminals"], serde_json::json!(["ghostty"]));
    assert!(summary["generated_state_repair_needed"].is_boolean());
    assert!(summary["generated_state_materialization_status"].is_string());
}

// Defends: `yzx inspect --json` is the canonical runtime truth report for diagnostics and agents.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_inspect_json_reports_runtime_truth_without_zellij_session() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "nu"

[terminal]
terminals = ["ghostty"]
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
            .ends_with("yazelix.toml")
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

// Defends: the default human `yzx status` output groups fields into readable sections instead of leaking raw internal summary keys.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_status_human_output_groups_sections_and_human_labels() {
    let fixture = managed_config_fixture(
        r#"[shell]
default_shell = "nu"

[terminal]
terminals = ["ghostty"]
"#,
    );
    let output = yzx_control_command_in_fixture(&fixture)
        .arg("status")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("Runtime\n"));
    assert!(stdout.contains("\nGenerated State\n"));
    assert!(stdout.contains("\nWorkspace\n"));
    assert!(stdout.contains("Config file"));
    assert!(stdout.contains("Default shell"));
    assert!(stdout.contains("Repair needed"));
    assert!(stdout.contains("Persistent sessions"));
    assert!(!stdout.contains("Yazelix status"));
    assert!(!stdout.contains('\u{1b}'));
    assert!(!stdout.contains("generated_state_materialization_status"));
    assert!(!stdout.contains("generated_state_materialization_reason"));
    assert!(!stdout.contains("default_shell"));
}

// Defends: the public Rust-owned `yzx status --json --versions` surface still attaches the optional tool matrix under one machine-readable report.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_status_json_versions_includes_tool_matrix() {
    let fixture = managed_config_fixture(
        r#"[terminal]
terminals = ["ghostty"]
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_executable_script(
        &fake_bin.join("nix"),
        "#!/bin/sh\nprintf 'nix (Nix) 2.28.3\\n'\n",
    );
    write_executable_script(&fake_bin.join("nu"), "#!/bin/sh\nprintf '0.105.1\\n'\n");

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .arg("status")
        .arg("--json")
        .arg("--versions")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let versions = report["versions"].as_object().unwrap();
    let tools = versions["tools"].as_array().unwrap();
    let nix_entry = tools
        .iter()
        .find(|entry| entry["tool"] == "nix")
        .expect("nix entry");

    assert_eq!(report["title"], "Yazelix status");
    assert_eq!(versions["title"], "Yazelix Tool Versions");
    assert_eq!(nix_entry["runtime"], "2.28.3");
}

// Defends: the human `yzx status --versions` output keeps the grouped status surface plus the tool-version matrix without leaking ANSI escapes to redirected output.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_status_versions_human_output_keeps_tool_matrix() {
    let fixture = managed_config_fixture(
        r#"[terminal]
terminals = ["ghostty"]
"#,
    );
    let fake_bin = fixture.home_dir.join("fake-bin");
    fs::create_dir_all(&fake_bin).unwrap();
    write_executable_script(
        &fake_bin.join("nix"),
        "#!/bin/sh\nprintf 'nix (Nix) 2.28.3\\n'\n",
    );
    write_executable_script(&fake_bin.join("nu"), "#!/bin/sh\nprintf '0.105.1\\n'\n");

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .arg("status")
        .arg("--versions")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("Runtime\n"));
    assert!(stdout.contains("\nYazelix Tool Versions\n"));
    assert!(stdout.contains("nix"));
    assert!(stdout.contains("2.28.3"));
    assert!(!stdout.contains('\u{1b}'));
}

// Defends: the Rust-owned `yzx update upstream` route still fails early for Home Manager-owned installs instead of probing the profile path.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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
        .join("yazelix.toml");
    fs::create_dir_all(hm_store_config.parent().unwrap()).unwrap();
    fs::write(&hm_store_config, "[core]\nwelcome_style = \"random\"\n").unwrap();
    fs::remove_file(&fixture.managed_config).unwrap();
    std::os::unix::fs::symlink(&hm_store_config, &fixture.managed_config).unwrap();

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

// Regression: `yzx update upstream` must allow a plain profile-owned install instead of misclassifying ~/.nix-profile as Home Manager ownership.
// Strength: defect=2 behavior=2 resilience=2 cost=2 uniqueness=2 total=10/10
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
    assert!(!stdout.contains("appears to be Home Manager-owned"));
    assert_eq!(
        fs::read_to_string(upgrade_log).unwrap(),
        "profile upgrade --refresh yazelix\n"
    );
}

// Regression: `yzx home_manager prepare --apply` must remove standalone profile-owned Yazelix entries as part of the takeover flow instead of only archiving files.
// Strength: defect=2 behavior=2 resilience=2 cost=2 uniqueness=2 total=10/10
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
            .any(|name| { name.starts_with("yazelix.toml.home-manager-prepare-backup-") })
    );
}

// Regression: `yzx update home_manager` must explain that `path:` inputs are still lock-pinned in flake.lock.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_update_home_manager_mentions_path_input_locking() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let update_log = fixture.home_dir.join("nix-update.log");
    let flake_dir = fixture.home_dir.join("hm-flake");
    fs::create_dir_all(&fake_bin).unwrap();
    fs::create_dir_all(&flake_dir).unwrap();
    fs::write(flake_dir.join("flake.nix"), "{ outputs = { self }: {}; }\n").unwrap();
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
    assert!(stdout.contains("Requested update path: Home Manager flake input."));
    assert!(stdout.contains("This still matters for `path:` inputs"));
    assert!(stdout.contains("pins a snapshot of that local path until you refresh it"));
    assert!(stdout.contains("home-manager switch"));
    assert_eq!(
        fs::read_to_string(update_log).unwrap(),
        "flake update yazelix\n"
    );
}

// Regression: `yzx update home_manager` must detect local git-backed `path:` inputs and print the exact safer `git+file:` replacement instead of normalizing the slow path snapshot UX.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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

// Defends: the public Rust-owned `yzx run` route preserves child dash flags end to end instead of stealing them as wrapper options.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yzx_control_run_preserves_child_dash_flags_end_to_end() {
    let fixture = managed_config_fixture("");
    let fake_bin = fixture.home_dir.join("fake-bin");
    let command_log = fixture.home_dir.join("child-command.log");
    fs::create_dir_all(&fake_bin).unwrap();
    write_executable_script(
        &fake_bin.join("cargo"),
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > \"{}\"\n",
            command_log.display()
        ),
    );

    let output = yzx_control_command_in_fixture(&fixture)
        .env("PATH", prepend_path(&fake_bin))
        .arg("run")
        .arg("cargo")
        .arg("--verbose")
        .arg("check")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(command_log).unwrap().trim(),
        "--verbose check"
    );
}
