// Test lane: default
use super::helix_config::prepare_managed_helix_config;
use super::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use toml::Value as TomlValue;

fn normal_binding(config: &TomlValue, key: &str) -> Option<String> {
    config
        .get("keys")?
        .get("normal")?
        .get(key)?
        .as_str()
        .map(str::to_owned)
}

fn steel_command_names(data: &HelixMaterializationData, visibility: &str) -> Vec<String> {
    data.steel_commands
        .iter()
        .filter(|command| command.visibility == visibility)
        .map(|command| command.name.clone())
        .collect()
}

const STEEL_PLUGIN_MANIFEST_FIXTURE: &str = r#"[[plugins]]
id = "recentf"
source = "cogs/recentf.scm"
public_commands = ["recentf-open-files"]
internal_commands = ["recentf-snapshot"]
startup_commands = ["recentf-snapshot"]
command_descriptions = { "recentf-open-files" = "Open a picker for recently visited Helix files", "recentf-snapshot" = "Refresh and persist the recent-file cache" }

[[plugins]]
id = "splash"
source = "splash.scm"
internal_commands = ["show-splash"]
startup_commands = ["show-splash"]
startup_condition = "show_splash"
command_descriptions = { "show-splash" = "Render the optional Yazelix splash overlay" }

[[plugins]]
id = "spacemacs_theme"
source = "cogs/themes/spacemacs.scm"
internal_commands = ["activate-spacemacs-theme"]
startup_commands = ["activate-spacemacs-theme"]
command_descriptions = { "activate-spacemacs-theme" = "Activate the registered Spacemacs Steel theme" }

[[plugins]]
id = "keymaps"
source = "cogs/keymaps.scm"

[[plugins]]
id = "labelled_buffers"
source = "cogs/labelled-buffers.scm"
support_files = ["cogs/keymaps.scm"]
"#;

fn write_runtime_layout(runtime_dir: &Path) {
    fs::create_dir_all(runtime_dir.join("configs/helix")).unwrap();
    fs::create_dir_all(runtime_dir.join("config_metadata")).unwrap();
    fs::create_dir_all(runtime_dir.join("configs/helix/steel_plugins/cogs/themes")).unwrap();

    for (path, content) in [
        (
            "configs/helix/yazelix_config.toml",
            include_str!("../../../../configs/helix/yazelix_config.toml"),
        ),
        (
            "config_default.toml",
            include_str!("../../../../config_default.toml"),
        ),
        (
            "config_metadata/main_config_contract.toml",
            include_str!("../../../../config_metadata/main_config_contract.toml"),
        ),
        (
            "configs/helix/steel_plugins/manifest.toml",
            STEEL_PLUGIN_MANIFEST_FIXTURE,
        ),
        (
            "configs/helix/steel_plugins/cogs/recentf.scm",
            "(provide recentf-open-files recentf-snapshot)\n",
        ),
        (
            "configs/helix/steel_plugins/cogs/keymaps.scm",
            "(provide keymap)\n",
        ),
        (
            "configs/helix/steel_plugins/cogs/labelled-buffers.scm",
            "(provide open-labelled-buffer)\n",
        ),
        (
            "configs/helix/steel_plugins/splash.scm",
            "(provide show-splash)\n",
        ),
        (
            "configs/helix/steel_plugins/cogs/themes/spacemacs.scm",
            "(provide built-theme activate-spacemacs-theme)\n",
        ),
    ] {
        fs::write(runtime_dir.join(path), content).unwrap();
    }
}

// Regression: Yazi-to-Helix open sends command text through `:` after Escape, so managed Helix materialization must reclaim command mode even when user overrides remap it.
#[test]
fn managed_helix_reclaims_colon_command_mode_binding() {
    let tmp = TempDir::new().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let config_dir = tmp.path().join("config");
    let template_dir = runtime_dir.join("configs").join("helix");
    fs::create_dir_all(&template_dir).unwrap();
    fs::create_dir_all(config_dir.join("helix")).unwrap();
    fs::write(
        template_dir.join("yazelix_config.toml"),
        "[keys.normal]\n\":\" = \"command_mode\"\nA-r = \":noop\"\n",
    )
    .unwrap();
    fs::write(
        config_dir.join("helix/config.toml"),
        "[keys.normal]\n\":\" = \"no_op\"\nA-r = \":noop\"\n",
    )
    .unwrap();

    let prepared = prepare_managed_helix_config(&runtime_dir, &config_dir).unwrap();

    assert_eq!(
        normal_binding(&prepared.config, MANAGED_COMMAND_MODE_KEY).as_deref(),
        Some(MANAGED_COMMAND_MODE_COMMAND)
    );
    assert_eq!(
        normal_binding(&prepared.config, REVEAL_KEY).as_deref(),
        Some(MANAGED_REVEAL_COMMAND)
    );
}

// Defends: the durable managed Helix source config lives under helix/config.toml; the old flat helix.toml surface must fail fast instead of silently winning.
#[test]
fn managed_helix_rejects_flat_legacy_helix_toml() {
    let tmp = TempDir::new().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let config_dir = tmp.path().join("config");
    let template_dir = runtime_dir.join("configs").join("helix");
    fs::create_dir_all(&template_dir).unwrap();
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        template_dir.join("yazelix_config.toml"),
        "theme = \"default\"\n",
    )
    .unwrap();
    fs::write(config_dir.join("helix.toml"), "theme = \"legacy\"\n").unwrap();

    let error = match prepare_managed_helix_config(&runtime_dir, &config_dir) {
        Ok(_) => panic!("flat legacy helix.toml unexpectedly accepted"),
        Err(error) => error,
    };

    assert!(
        error
            .message()
            .contains("old Helix override config surface")
    );
    assert!(
        error
            .remediation()
            .contains("Move ~/.config/yazelix/helix.toml to ~/.config/yazelix/helix/config.toml")
    );
}

// Defends: Helix materialization creates Steel entrypoint files and loads only the default non-history-writing Steel plugins from runtime-owned sources.
#[test]
fn helix_materialization_writes_default_steel_entrypoints() {
    let tmp = TempDir::new().unwrap();
    let runtime_dir = tmp.path().join("runtime");
    let config_dir = tmp.path().join("config");
    let state_dir = tmp.path().join("state");
    fs::create_dir_all(&config_dir).unwrap();
    write_runtime_layout(&runtime_dir);

    let data = generate_helix_materialization(&HelixMaterializationRequest {
        runtime_dir,
        config_dir: config_dir.clone(),
        state_dir: state_dir.clone(),
        show_splash: true,
    })
    .unwrap();

    let steel_dir = state_dir.join("configs/helix");
    assert_eq!(
        data.enabled_steel_plugins,
        vec!["splash", "spacemacs_theme"]
    );
    assert_eq!(
        data.generated_steel_config_dir,
        steel_dir.to_string_lossy().to_string()
    );
    assert_eq!(
        data.managed_helix_config_dir,
        config_dir.join("helix").to_string_lossy().to_string()
    );
    assert!(config_dir.join("helix").exists());
    assert!(!steel_dir.join("cogs/recentf.scm").exists());
    assert!(steel_dir.join("splash.scm").exists());
    assert!(steel_dir.join("cogs/themes/spacemacs.scm").exists());
    assert!(!steel_dir.join("cogs/keymaps.scm").exists());
    assert!(!steel_dir.join("cogs/labelled-buffers.scm").exists());

    let generated_helix = fs::read_to_string(state_dir.join("configs/helix/helix.scm")).unwrap();
    assert!(generated_helix.contains("(require (only-in \"helix/ext.scm\" eval-buffer evalp))"));
    assert!(generated_helix.contains("(provide eval-buffer evalp yzx-new-shell)"));
    assert!(
        generated_helix
            .contains("(require (only-in \"helix/static.scm\" cx->current-file get-helix-cwd))")
    );
    assert!(
        generated_helix.contains("(require (only-in \"helix/commands.scm\" run-shell-command))")
    );
    assert!(generated_helix.contains("yzx-new-shell"));
    assert!(
        generated_helix
            .contains("(string-append \"'\" (string-replace value \"'\" \"'\\\\''\") \"'\"))")
    );
    assert!(generated_helix.contains("yzx_control\\\" zellij open-terminal"));
    assert!(!generated_helix.contains("recentf-open-files"));
    assert!(!generated_helix.contains("recentf-snapshot"));
    assert!(generated_helix.contains("(require (only-in \"splash.scm\" show-splash))"));
    assert!(generated_helix.contains("(show-splash)"));
    assert!(
        generated_helix
            .contains("(require (only-in \"cogs/themes/spacemacs.scm\" activate-spacemacs-theme))")
    );
    assert!(generated_helix.contains("(activate-spacemacs-theme)"));
    assert_eq!(
        steel_command_names(&data, "public"),
        vec![
            "eval-buffer".to_string(),
            "evalp".to_string(),
            "yzx-new-shell".to_string()
        ]
    );
    assert_eq!(
        steel_command_names(&data, "internal"),
        vec![
            "show-splash".to_string(),
            "activate-spacemacs-theme".to_string()
        ]
    );

    let generated_init = fs::read_to_string(state_dir.join("configs/helix/init.scm")).unwrap();
    assert!(!generated_init.contains("prefix-in"));
    assert!(!generated_init.contains("yazelix."));
    assert!(!generated_init.contains("show-splash"));
}
