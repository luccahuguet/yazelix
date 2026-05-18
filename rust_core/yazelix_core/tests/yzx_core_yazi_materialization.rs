// Test lane: maintainer

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn prepare_managed_config(
    config_root: &std::path::Path,
    repo: &std::path::Path,
    body: &str,
) -> PathBuf {
    let config_path = if body.is_empty() {
        config_root.join("settings.jsonc")
    } else {
        config_root.join("yazelix.toml")
    };
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    if body.is_empty() {
        fs::copy(repo.join("settings_default.jsonc"), &config_path).unwrap();
    } else {
        fs::write(&config_path, body).unwrap();
    }
    config_path
}

fn prepare_runtime_fixture(runtime_dir: &std::path::Path) {
    let yazi_dir = runtime_dir.join("configs").join("yazi");
    fs::create_dir_all(yazi_dir.join("plugins")).unwrap();
    fs::create_dir_all(yazi_dir.join("flavors")).unwrap();

    fs::write(
        yazi_dir.join("yazelix_yazi.toml"),
        r#"[mgr]
ratio = [1, 4, 3]

[opener]
edit = [
  { run = '__YAZELIX_RUNTIME_DIR__/libexec/yzx_control zellij open-editor %s', desc = "Open File with configured editor (with Zellij integration)" },
]

[[plugin.prepend_fetchers]]
id = "git"
name = "*"
run = "git"
"#,
    )
    .unwrap();
    fs::write(
        yazi_dir.join("yazelix_keymap.toml"),
        r#"[[mgr.prepend_keymap]]
on = ["g", "l"]
run = "plugin lazygit"
desc = "Open lazygit"
"#,
    )
    .unwrap();
    fs::write(
        yazi_dir.join("yazelix_theme.toml"),
        "[status]\noverall = { bg = \"black\" }\n",
    )
    .unwrap();
    fs::write(
        yazi_dir.join("yazelix_starship.toml"),
        "# YAZELIX STARSHIP CONFIG FOR YAZI SIDEBAR\n",
    )
    .unwrap();

    for plugin in [
        "sidebar-status",
        "auto-layout",
        "sidebar-state",
        "git",
        "starship",
    ] {
        let plugin_dir = yazi_dir.join("plugins").join(format!("{plugin}.yazi"));
        fs::create_dir_all(&plugin_dir).unwrap();
        let body = if plugin == "auto-layout" {
            "return '__YAZELIX_RUNTIME_DIR__/libexec/yzx_control zellij open-editor-cwd'\n"
        } else {
            "return 'ok'\n"
        };
        fs::write(plugin_dir.join("main.lua"), body).unwrap();
    }

    let tokyo_night = yazi_dir.join("flavors").join("tokyo-night.yazi");
    fs::create_dir_all(&tokyo_night).unwrap();
    fs::write(tokyo_night.join("flavor.toml"), "[mgr]\n").unwrap();
}

// Defends: yazi-materialization.generate Rust-owns the generated Yazi surface, bundled assets, and runtime placeholder rendering end-to-end.
#[test]
fn yazi_materialization_generate_writes_managed_surface_and_assets() {
    let repo = repo_root();
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let config_root = home.join(".config").join("yazelix");
    let output_dir = temp.path().join("state").join("configs").join("yazi");
    let runtime_dir = temp.path().join("runtime");
    let config_path = prepare_managed_config(
        &config_root,
        &repo,
        r#"[yazi]
theme = "tokyo-night"
sort_by = "modified"
plugins = ["git", "starship"]
"#,
    );
    prepare_runtime_fixture(&runtime_dir);

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local").join("share"))
        .env("YAZELIX_CONFIG_DIR", &config_root)
        .arg("yazi-materialization.generate")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--yazi-config-dir")
        .arg(&output_dir)
        .arg("--sync-static-assets")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["command"], "yazi-materialization.generate");
    assert_eq!(envelope["status"], "ok");
    assert_eq!(envelope["data"]["resolved_theme"], "tokyo-night");
    assert_eq!(envelope["data"]["sort_by"], "modified");
    assert_eq!(envelope["data"]["synced_static_assets"], true);
    assert_eq!(envelope["data"]["missing_plugins"], serde_json::json!([]));

    let yazi_toml = fs::read_to_string(output_dir.join("yazi.toml")).unwrap();
    let init_lua = fs::read_to_string(output_dir.join("init.lua")).unwrap();
    let runtime_placeholder_plugin = fs::read_to_string(
        output_dir
            .join("plugins")
            .join("auto-layout.yazi")
            .join("main.lua"),
    )
    .unwrap();

    assert!(yazi_toml.contains("[manager]"));
    assert!(yazi_toml.contains("sort_by = \"modified\""));
    assert!(yazi_toml.contains(runtime_dir.to_string_lossy().as_ref()));
    assert!(yazi_toml.contains("yzx_control zellij open-editor %s"));
    assert!(
        init_lua.contains(
            output_dir
                .join("yazelix_starship.toml")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert!(!runtime_placeholder_plugin.contains("__YAZELIX_RUNTIME_DIR__"));
    assert!(runtime_placeholder_plugin.contains(runtime_dir.to_string_lossy().as_ref()));
}

// Regression: native Yazi array settings such as mgr.ratio replace Yazelix defaults instead of appending to them.
#[test]
fn yazi_materialization_generate_replaces_user_yazi_array_settings() {
    let repo = repo_root();
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let config_root = home.join(".config").join("yazelix");
    let output_dir = temp.path().join("state").join("configs").join("yazi");
    let runtime_dir = temp.path().join("runtime");
    let config_path = prepare_managed_config(&config_root, &repo, "");
    prepare_runtime_fixture(&runtime_dir);
    fs::write(
        config_root.join("yazi.toml"),
        r#"[mgr]
ratio = [1, 4, 0]
"#,
    )
    .unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local").join("share"))
        .env("YAZELIX_CONFIG_DIR", &config_root)
        .arg("yazi-materialization.generate")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--yazi-config-dir")
        .arg(&output_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated_yazi = fs::read_to_string(output_dir.join("yazi.toml")).unwrap();
    let parsed_yazi: toml::Value = toml::from_str(&generated_yazi).unwrap();
    assert_eq!(
        parsed_yazi
            .get("mgr")
            .and_then(|section| section.get("ratio"))
            .and_then(toml::Value::as_array),
        Some(&vec![1.into(), 4.into(), 0.into()])
    );
}

// Defends: semantic Yazi integration keybinding remaps replace generated Yazelix-owned bindings without editing the native yazi_keymap.toml sidecar.
#[test]
fn yazi_materialization_generate_applies_semantic_keybinding_remaps() {
    let repo = repo_root();
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let config_root = home.join(".config").join("yazelix");
    let output_dir = temp.path().join("state").join("configs").join("yazi");
    let runtime_dir = temp.path().join("runtime");
    let config_path = prepare_managed_config(
        &config_root,
        &repo,
        r#"[yazi.keybindings]
open_directory_as_workspace_pane = []
open_zoxide_in_editor = ["<A-x>", "<A-s>"]
"#,
    );
    prepare_runtime_fixture(&runtime_dir);

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local").join("share"))
        .env("YAZELIX_CONFIG_DIR", &config_root)
        .arg("yazi-materialization.generate")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--yazi-config-dir")
        .arg(&output_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated_keymap = fs::read_to_string(output_dir.join("keymap.toml")).unwrap();
    let parsed_keymap: toml::Value = toml::from_str(&generated_keymap).unwrap();
    let mgr_append = parsed_keymap
        .get("mgr")
        .and_then(|section| section.get("append_keymap"))
        .and_then(toml::Value::as_array)
        .expect("mgr append keymap");

    assert_eq!(mgr_append.len(), 2);
    assert_eq!(
        mgr_append[0]
            .get("on")
            .and_then(toml::Value::as_array)
            .and_then(|keys| keys.first())
            .and_then(toml::Value::as_str),
        Some("<A-x>")
    );
    assert_eq!(
        mgr_append[1]
            .get("on")
            .and_then(toml::Value::as_array)
            .and_then(|keys| keys.first())
            .and_then(toml::Value::as_str),
        Some("<A-s>")
    );
    assert_eq!(
        mgr_append[0].get("run").and_then(toml::Value::as_str),
        Some("plugin zoxide-editor")
    );
    assert!(!generated_keymap.contains("open-terminal"));
    assert!(!generated_keymap.contains("<A-p>"));
}

// Defends: duplicate semantic Yazi keys fail before a generated keymap can contain ambiguous Yazelix-owned integration bindings.
#[test]
fn yazi_materialization_generate_rejects_duplicate_semantic_keybindings() {
    let repo = repo_root();
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let config_root = home.join(".config").join("yazelix");
    let output_dir = temp.path().join("state").join("configs").join("yazi");
    let runtime_dir = temp.path().join("runtime");
    let config_path = prepare_managed_config(
        &config_root,
        &repo,
        r#"[yazi.keybindings]
open_directory_as_workspace_pane = ["<A-x>"]
open_zoxide_in_editor = ["<A-x>"]
"#,
    );
    prepare_runtime_fixture(&runtime_dir);

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local").join("share"))
        .env("YAZELIX_CONFIG_DIR", &config_root)
        .arg("yazi-materialization.generate")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--yazi-config-dir")
        .arg(&output_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(65));
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["error"]["code"], "duplicate_yazi_keybinding");
    assert_eq!(envelope["error"]["details"]["key"], "<A-x>");
}

// Regression: user Yazi keymap sections that are absent from Yazelix's bundled base keymap still survive materialization.
#[test]
fn yazi_materialization_generate_preserves_user_keymap_sections_beyond_mgr() {
    let repo = repo_root();
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let config_root = home.join(".config").join("yazelix");
    let output_dir = temp.path().join("state").join("configs").join("yazi");
    let runtime_dir = temp.path().join("runtime");
    let config_path = prepare_managed_config(&config_root, &repo, "");
    let user_yazi_dir = config_root.clone();
    prepare_runtime_fixture(&runtime_dir);
    fs::create_dir_all(&user_yazi_dir).unwrap();
    fs::write(
        user_yazi_dir.join("yazi_keymap.toml"),
        r#"
[[input.append_keymap]]
on = ["<Esc>"]
run = "close"
desc = "Close input"

[[input.prepend_keymap]]
on = ["<C-a>"]
run = "move -999"
desc = "Move to start"

[[cmp.append_keymap]]
on = ["<Tab>"]
run = "confirm"
desc = "Confirm completion"

[[cmp.prepend_keymap]]
on = ["<BackTab>"]
run = "arrow -1"
desc = "Previous completion"
"#,
    )
    .unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local").join("share"))
        .env("YAZELIX_CONFIG_DIR", &config_root)
        .arg("yazi-materialization.generate")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--yazi-config-dir")
        .arg(&output_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated_keymap = fs::read_to_string(output_dir.join("keymap.toml")).unwrap();
    let parsed_keymap: toml::Value = toml::from_str(&generated_keymap).unwrap();

    assert_eq!(
        parsed_keymap
            .get("mgr")
            .and_then(|section| section.get("append_keymap"))
            .and_then(toml::Value::as_array)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        parsed_keymap
            .get("input")
            .and_then(|section| section.get("append_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("close")
    );
    assert_eq!(
        parsed_keymap
            .get("input")
            .and_then(|section| section.get("prepend_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("move -999")
    );
    assert_eq!(
        parsed_keymap
            .get("cmp")
            .and_then(|section| section.get("append_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("confirm")
    );
    assert_eq!(
        parsed_keymap
            .get("cmp")
            .and_then(|section| section.get("prepend_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("arrow -1")
    );
}

// Defends: yazi-materialization.generate rejects legacy Yazi override ownership instead of silently adopting configs/yazi/user.
#[test]
fn yazi_materialization_generate_rejects_legacy_override_surface() {
    let repo = repo_root();
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let config_root = home.join(".config").join("yazelix");
    let output_dir = temp.path().join("state").join("configs").join("yazi");
    let runtime_dir = temp.path().join("runtime");
    let config_path = prepare_managed_config(&config_root, &repo, "");
    let legacy_override_dir = config_root.join("configs").join("yazi").join("user");
    prepare_runtime_fixture(&runtime_dir);
    fs::create_dir_all(&legacy_override_dir).unwrap();
    fs::write(legacy_override_dir.join("init.lua"), "return 'legacy'\n").unwrap();

    let output = Command::cargo_bin("yzx_core")
        .unwrap()
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local").join("share"))
        .env("YAZELIX_CONFIG_DIR", &config_root)
        .arg("yazi-materialization.generate")
        .arg("--config")
        .arg(&config_path)
        .arg("--default-config")
        .arg(repo.join("settings_default.jsonc"))
        .arg("--contract")
        .arg(repo.join("config_metadata/main_config_contract.toml"))
        .arg("--runtime-dir")
        .arg(&runtime_dir)
        .arg("--yazi-config-dir")
        .arg(&output_dir)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(65));
    let envelope: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(envelope["command"], "yazi-materialization.generate");
    assert_eq!(envelope["status"], "error");
    assert_eq!(envelope["error"]["class"], "config");
    assert_eq!(envelope["error"]["code"], "legacy_yazi_user_override");
    let message = envelope["error"]["message"].as_str().unwrap();
    assert!(message.contains("yzx import yazi"));
    assert!(message.contains("~/.config/yazelix/"));
}
