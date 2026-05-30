use crate::active_config_surface::primary_config_paths;
use crate::atomic_fs::{write_bytes_atomic, write_text_atomic};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::user_config_paths;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

pub(crate) const MANAGED_REVEAL_COMMAND: &str = ":sh yzx reveal \"%{buffer_name}\"";
pub(crate) const MANAGED_COMMAND_MODE_KEY: &str = ":";
pub(crate) const MANAGED_COMMAND_MODE_COMMAND: &str = "command_mode";
pub(crate) const REVEAL_KEY: &str = "A-r";
const STEEL_CONFIG_MODULE: &str = "helix.scm";
const STEEL_INIT_MODULE: &str = "init.scm";
const STEEL_PLUGIN_ROOT: &str = "steel_plugins";
const STEEL_SUPPORT_FILES: &[&str] = &["cogs/keymaps.scm", "cogs/labelled-buffers.scm"];

#[derive(Debug, Clone)]
pub struct HelixMaterializationRequest {
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelixImportNotice {
    pub marker_path: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HelixMaterializationData {
    pub generated_path: String,
    pub generated_steel_config_dir: String,
    pub generated_steel_module_path: String,
    pub generated_steel_init_path: String,
    pub template_path: String,
    pub user_config_merged: bool,
    pub reveal_binding_enforced: bool,
    pub enabled_steel_plugins: Vec<String>,
    pub steel_plugin_files: Vec<String>,
    pub import_notice: Option<HelixImportNotice>,
}

struct PreparedHelixConfig {
    template_path: PathBuf,
    user_config_path: PathBuf,
    config: TomlValue,
    user_config_merged: bool,
}

#[derive(Debug, Clone, Copy)]
struct SteelPluginSpec {
    id: &'static str,
    normalized_config_key: &'static str,
    source_relative_path: &'static str,
    helix_module_lines: &'static [&'static str],
    init_lines: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SteelPluginSelection {
    recentf: bool,
    splash: bool,
    spacemacs_theme: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SteelMaterializationData {
    config_dir: PathBuf,
    helix_module_path: PathBuf,
    init_path: PathBuf,
    enabled_plugins: Vec<String>,
    copied_plugin_files: Vec<String>,
}

const STEEL_PLUGIN_SPECS: &[SteelPluginSpec] = &[
    SteelPluginSpec {
        id: "recentf",
        normalized_config_key: "helix_plugin_recentf",
        source_relative_path: "cogs/recentf.scm",
        helix_module_lines: &[
            "(require (only-in \"cogs/recentf.scm\" recentf-open-files))",
            "(provide recentf-open-files)",
        ],
        init_lines: &[
            "(require (only-in \"cogs/recentf.scm\" recentf-snapshot))",
            "(recentf-snapshot)",
        ],
    },
    SteelPluginSpec {
        id: "splash",
        normalized_config_key: "helix_plugin_splash",
        source_relative_path: "splash.scm",
        helix_module_lines: &[
            "(require (only-in \"splash.scm\" show-splash))",
            "(provide show-splash)",
        ],
        init_lines: &[
            "(require (only-in \"splash.scm\" show-splash))",
            "(when (equal? (command-line) '(\"hx\"))",
            "  (show-splash))",
        ],
    },
    SteelPluginSpec {
        id: "spacemacs_theme",
        normalized_config_key: "helix_plugin_spacemacs_theme",
        source_relative_path: "cogs/themes/spacemacs.scm",
        helix_module_lines: &[],
        init_lines: &["(require \"cogs/themes/spacemacs.scm\")"],
    },
];

pub(crate) fn build_managed_helix_contract_json(
    runtime_dir: &Path,
    config_dir: &Path,
) -> Result<JsonValue, CoreError> {
    let prepared = prepare_managed_helix_config(runtime_dir, config_dir)?;
    serde_json::to_value(prepared.config).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_helix_contract_json",
            format!("Could not serialize the expected Helix config contract as JSON: {source}"),
            "This is an internal error. File a bug report if it persists.",
            serde_json::json!({
                "runtime_dir": runtime_dir.to_string_lossy(),
                "config_dir": config_dir.to_string_lossy(),
            }),
        )
    })
}

pub fn generate_helix_materialization(
    request: &HelixMaterializationRequest,
) -> Result<HelixMaterializationData, CoreError> {
    crate::managed_user_config_stubs::ensure_helix_surface_stub(&request.config_dir)?;
    let prepared = prepare_managed_helix_config(&request.runtime_dir, &request.config_dir)?;
    let plugin_selection = load_steel_plugin_selection(&request.runtime_dir, &request.config_dir)?;

    let generated_dir = request.state_dir.join("configs").join("helix");
    fs::create_dir_all(&generated_dir).map_err(|source| {
        CoreError::io(
            "create_helix_output_dir",
            "Could not create the managed Helix output directory",
            "Check permissions for the Yazelix state directory and retry.",
            generated_dir.to_string_lossy(),
            source,
        )
    })?;

    let generated_path = generated_dir.join("config.toml");
    let output = toml::to_string_pretty(&prepared.config).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_helix_config",
            format!("Could not serialize the merged Helix config as TOML: {source}"),
            "This is an internal error. File a bug report if it persists.",
            serde_json::json!({ "path": generated_path.to_string_lossy() }),
        )
    })?;

    write_text_atomic(&generated_path, &output)?;

    let import_notice = build_import_notice(request, &prepared.user_config_path)?;
    let steel = materialize_steel_config(
        &request.runtime_dir,
        &request.config_dir,
        &generated_dir,
        &plugin_selection,
    )?;

    Ok(HelixMaterializationData {
        generated_path: generated_path.to_string_lossy().into_owned(),
        generated_steel_config_dir: steel.config_dir.to_string_lossy().into_owned(),
        generated_steel_module_path: steel.helix_module_path.to_string_lossy().into_owned(),
        generated_steel_init_path: steel.init_path.to_string_lossy().into_owned(),
        template_path: prepared.template_path.to_string_lossy().into_owned(),
        user_config_merged: prepared.user_config_merged,
        reveal_binding_enforced: true,
        enabled_steel_plugins: steel.enabled_plugins,
        steel_plugin_files: steel.copied_plugin_files,
        import_notice,
    })
}

fn load_steel_plugin_selection(
    runtime_dir: &Path,
    config_dir: &Path,
) -> Result<SteelPluginSelection, CoreError> {
    let paths = primary_config_paths(runtime_dir, config_dir);
    let config_path = if paths.user_config.exists() {
        paths.user_config
    } else {
        paths.default_config_path.clone()
    };
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        include_missing: false,
    })?
    .normalized_config;

    Ok(SteelPluginSelection {
        recentf: normalized_bool(&normalized, "helix_plugin_recentf"),
        splash: normalized_bool(&normalized, "helix_plugin_splash"),
        spacemacs_theme: normalized_bool(&normalized, "helix_plugin_spacemacs_theme"),
    })
}

fn normalized_bool(config: &JsonMap<String, JsonValue>, key: &str) -> bool {
    config
        .get(key)
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
}

fn prepare_managed_helix_config(
    runtime_dir: &Path,
    config_dir: &Path,
) -> Result<PreparedHelixConfig, CoreError> {
    let template_path = runtime_dir
        .join("configs")
        .join("helix")
        .join("yazelix_config.toml");
    if !template_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Io,
            "missing_helix_template",
            format!(
                "Missing Yazelix Helix template at: {}",
                template_path.display()
            ),
            "Reinstall Yazelix so the runtime includes configs/helix/yazelix_config.toml.",
            serde_json::json!({ "template_path": template_path.to_string_lossy() }),
        ));
    }

    let template_content = fs::read_to_string(&template_path).map_err(|source| {
        CoreError::io(
            "read_helix_template",
            "Could not read the Yazelix Helix config template",
            "Check permissions for the Yazelix runtime directory and retry.",
            template_path.to_string_lossy(),
            source,
        )
    })?;

    let mut config: TomlValue = toml::from_str(&template_content).map_err(|source| {
        CoreError::toml(
            "parse_helix_template",
            "Could not parse the Yazelix Helix config template as TOML",
            "Reinstall Yazelix so the runtime includes a valid Helix config template.",
            template_path.to_string_lossy(),
            source,
        )
    })?;

    let user_config_path = user_config_paths::resolve_current_config_file(
        &user_config_paths::helix_config(config_dir),
        &user_config_paths::legacy_helix_config(config_dir),
        "Helix override",
    )?;

    let user_config_merged = if user_config_path.exists() {
        let user_content = fs::read_to_string(&user_config_path).map_err(|source| {
            CoreError::io(
                "read_helix_user_config",
                "Could not read the user Helix config override",
                "Check permissions for ~/.config/yazelix/helix.toml and retry.",
                user_config_path.to_string_lossy(),
                source,
            )
        })?;
        let user_config: TomlValue = toml::from_str(&user_content).map_err(|source| {
            CoreError::toml(
                "parse_helix_user_config",
                "Could not parse the user Helix config override as TOML",
                "Fix the TOML syntax in ~/.config/yazelix/helix.toml and retry.",
                user_config_path.to_string_lossy(),
                source,
            )
        })?;
        deep_merge_toml(&mut config, &user_config);
        true
    } else {
        false
    };

    enforce_managed_normal_bindings(&mut config);

    Ok(PreparedHelixConfig {
        template_path,
        user_config_path,
        config,
        user_config_merged,
    })
}

fn deep_merge_toml(base: &mut TomlValue, user: &TomlValue) {
    match (base, user) {
        (TomlValue::Table(base_map), TomlValue::Table(user_map)) => {
            for (key, user_val) in user_map {
                if let Some(base_val) = base_map.get_mut(key) {
                    deep_merge_toml(base_val, user_val);
                } else {
                    base_map.insert(key.clone(), user_val.clone());
                }
            }
        }
        (base_val, user_val) => {
            *base_val = user_val.clone();
        }
    }
}

fn enforce_managed_normal_bindings(config: &mut TomlValue) {
    let table = match config {
        TomlValue::Table(t) => t,
        _ => return,
    };

    let keys_table = table
        .entry("keys")
        .or_insert_with(|| TomlValue::Table(toml::map::Map::new()));

    let normal_table = match keys_table {
        TomlValue::Table(t) => t
            .entry("normal")
            .or_insert_with(|| TomlValue::Table(toml::map::Map::new())),
        _ => return,
    };

    match normal_table {
        TomlValue::Table(t) => {
            t.insert(
                MANAGED_COMMAND_MODE_KEY.into(),
                TomlValue::String(MANAGED_COMMAND_MODE_COMMAND.into()),
            );
            t.insert(
                REVEAL_KEY.into(),
                TomlValue::String(MANAGED_REVEAL_COMMAND.into()),
            );
        }
        _ => {}
    }
}

fn materialize_steel_config(
    runtime_dir: &Path,
    config_dir: &Path,
    generated_dir: &Path,
    selection: &SteelPluginSelection,
) -> Result<SteelMaterializationData, CoreError> {
    let selected = selected_steel_plugins(selection);
    let mut copied_plugin_files = Vec::new();

    for relative_path in STEEL_SUPPORT_FILES {
        let source = resolve_steel_file_source(runtime_dir, config_dir, relative_path)?;
        let target = generated_dir.join(relative_path);
        copy_steel_plugin_file(&source, &target)?;
        copied_plugin_files.push(target.to_string_lossy().into_owned());
    }

    for spec in &selected {
        let source = resolve_steel_file_source(runtime_dir, config_dir, spec.source_relative_path)?;
        let target = generated_dir.join(spec.source_relative_path);
        copy_steel_plugin_file(&source, &target)?;
        copied_plugin_files.push(target.to_string_lossy().into_owned());
    }

    let helix_module_path = generated_dir.join(STEEL_CONFIG_MODULE);
    let init_path = generated_dir.join(STEEL_INIT_MODULE);
    write_text_atomic(&helix_module_path, &render_steel_helix_module(&selected))?;
    write_text_atomic(&init_path, &render_steel_init_module(&selected))?;

    Ok(SteelMaterializationData {
        config_dir: generated_dir.to_path_buf(),
        helix_module_path,
        init_path,
        enabled_plugins: selected
            .iter()
            .map(|spec| spec.id.to_string())
            .collect::<Vec<_>>(),
        copied_plugin_files,
    })
}

fn selected_steel_plugins(selection: &SteelPluginSelection) -> Vec<&'static SteelPluginSpec> {
    STEEL_PLUGIN_SPECS
        .iter()
        .filter(|spec| match spec.normalized_config_key {
            "helix_plugin_recentf" => selection.recentf,
            "helix_plugin_splash" => selection.splash,
            "helix_plugin_spacemacs_theme" => selection.spacemacs_theme,
            _ => false,
        })
        .collect()
}

fn resolve_steel_file_source(
    runtime_dir: &Path,
    config_dir: &Path,
    relative_path: &str,
) -> Result<PathBuf, CoreError> {
    let user_source = config_dir
        .join("helix")
        .join(STEEL_PLUGIN_ROOT)
        .join(relative_path);
    if user_source.exists() {
        return Ok(user_source);
    }

    let runtime_source = runtime_dir
        .join("configs")
        .join("helix")
        .join(STEEL_PLUGIN_ROOT)
        .join(relative_path);
    if runtime_source.exists() {
        return Ok(runtime_source);
    }

    Err(CoreError::classified(
        ErrorClass::Config,
        "missing_helix_steel_file",
        format!(
            "Helix Steel support file `{}` is missing from the managed plugin sources.",
            relative_path
        ),
        format!(
            "Install {} under {} or {}, then retry.",
            relative_path,
            config_dir.join("helix").join(STEEL_PLUGIN_ROOT).display(),
            runtime_dir
                .join("configs")
                .join("helix")
                .join(STEEL_PLUGIN_ROOT)
                .display()
        ),
        serde_json::json!({
            "source_relative_path": relative_path,
            "user_source": user_source.to_string_lossy(),
            "runtime_source": runtime_source.to_string_lossy(),
        }),
    ))
}

fn copy_steel_plugin_file(source: &Path, target: &Path) -> Result<(), CoreError> {
    let bytes = fs::read(source).map_err(|source_error| {
        CoreError::io(
            "read_helix_steel_plugin",
            "Could not read a managed Helix Steel plugin source file",
            "Reinstall Yazelix so the runtime includes readable Helix Steel plugin files.",
            source.to_string_lossy(),
            source_error,
        )
    })?;
    write_bytes_atomic(target, &bytes)
}

fn render_steel_helix_module(selected: &[&SteelPluginSpec]) -> String {
    let mut lines = vec![
        ";; Yazelix-managed Helix Steel command module.",
        ";; Generated at launch from settings.jsonc.",
        "",
    ];
    for spec in selected {
        lines.push(";;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;");
        lines.push("");
        lines.extend(spec.helix_module_lines.iter().copied());
        lines.push("");
    }
    lines.join("\n") + "\n"
}

fn render_steel_init_module(selected: &[&SteelPluginSpec]) -> String {
    let mut lines = vec![
        ";; Yazelix-managed Helix Steel init file.",
        ";; Generated at launch from settings.jsonc.",
        "",
    ];
    for spec in selected {
        lines.push(";;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;");
        lines.push("");
        lines.extend(spec.init_lines.iter().copied());
        lines.push("");
    }
    lines.join("\n") + "\n"
}

fn build_import_notice(
    request: &HelixMaterializationRequest,
    user_config_path: &Path,
) -> Result<Option<HelixImportNotice>, CoreError> {
    let native_config_path = resolve_native_helix_config_path()?;

    if !native_config_path.exists() {
        return Ok(None);
    }

    if user_config_path.exists() {
        return Ok(None);
    }

    let notice_dir = request.state_dir.join("state").join("helix");
    fs::create_dir_all(&notice_dir).map_err(|source| {
        CoreError::io(
            "create_helix_notice_dir",
            "Could not create the Helix notice state directory",
            "Check permissions for the Yazelix state directory and retry.",
            notice_dir.to_string_lossy(),
            source,
        )
    })?;

    let marker_path = notice_dir.join("import_notice_seen");
    if marker_path.exists() {
        return Ok(None);
    }

    fs::write(&marker_path, "").map_err(|source| {
        CoreError::io(
            "write_helix_notice_marker",
            "Could not write the Helix import notice marker",
            "Check permissions for the Yazelix state directory and retry.",
            marker_path.to_string_lossy(),
            source,
        )
    })?;

    Ok(Some(HelixImportNotice {
        marker_path: marker_path.to_string_lossy().into_owned(),
        lines: vec![
            "ℹ️  Yazelix is using its managed Helix config.".into(),
            format!(
                "   Personal Helix config detected at: {}",
                native_config_path.display()
            ),
            "   If you want Yazelix-managed Helix sessions to reuse it, run: yzx import helix"
                .into(),
        ],
    }))
}

fn resolve_native_helix_config_path() -> Result<PathBuf, CoreError> {
    let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.into())
            }
        })
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|_| PathBuf::from("."))
        });

    Ok(xdg_config_home.join("helix").join("config.toml"))
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn normal_binding(config: &TomlValue, key: &str) -> Option<String> {
        config
            .get("keys")?
            .get("normal")?
            .get(key)?
            .as_str()
            .map(str::to_owned)
    }

    fn write_runtime_layout(runtime_dir: &Path) {
        let template_dir = runtime_dir.join("configs").join("helix");
        fs::create_dir_all(&template_dir).unwrap();
        fs::write(
            template_dir.join("yazelix_config.toml"),
            include_str!("../../../configs/helix/yazelix_config.toml"),
        )
        .unwrap();
        fs::write(
            runtime_dir.join("settings_default.jsonc"),
            include_str!("../../../settings_default.jsonc"),
        )
        .unwrap();
        fs::create_dir_all(runtime_dir.join("config_metadata")).unwrap();
        fs::write(
            runtime_dir
                .join("config_metadata")
                .join("main_config_contract.toml"),
            include_str!("../../../config_metadata/main_config_contract.toml"),
        )
        .unwrap();
        fs::create_dir_all(template_dir.join("steel_plugins/cogs/themes")).unwrap();
        fs::write(
            template_dir.join("steel_plugins/cogs/recentf.scm"),
            include_str!("../../../configs/helix/steel_plugins/cogs/recentf.scm"),
        )
        .unwrap();
        fs::write(
            template_dir.join("steel_plugins/cogs/keymaps.scm"),
            include_str!("../../../configs/helix/steel_plugins/cogs/keymaps.scm"),
        )
        .unwrap();
        fs::write(
            template_dir.join("steel_plugins/cogs/labelled-buffers.scm"),
            include_str!("../../../configs/helix/steel_plugins/cogs/labelled-buffers.scm"),
        )
        .unwrap();
        fs::write(
            template_dir.join("steel_plugins/splash.scm"),
            include_str!("../../../configs/helix/steel_plugins/splash.scm"),
        )
        .unwrap();
        fs::write(
            template_dir.join("steel_plugins/cogs/themes/spacemacs.scm"),
            include_str!("../../../configs/helix/steel_plugins/cogs/themes/spacemacs.scm"),
        )
        .unwrap();
    }

    // Regression: Yazi-to-Helix open sends command text through `:` after Escape, so managed Helix materialization must reclaim command mode even when user overrides remap it.
    #[test]
    fn managed_helix_reclaims_colon_command_mode_binding() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let config_dir = tmp.path().join("config");
        let template_dir = runtime_dir.join("configs").join("helix");
        fs::create_dir_all(&template_dir).unwrap();
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            template_dir.join("yazelix_config.toml"),
            "[keys.normal]\n\":\" = \"command_mode\"\nA-r = \":noop\"\n",
        )
        .unwrap();
        fs::write(
            config_dir.join("helix.toml"),
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

    // Defends: Helix materialization creates Steel entrypoint files and loads the default curated Steel plugins from runtime-owned sources.
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
            config_dir,
            state_dir: state_dir.clone(),
        })
        .unwrap();

        let steel_dir = state_dir.join("configs/helix");
        assert_eq!(data.enabled_steel_plugins, vec!["recentf", "splash"]);
        assert_eq!(
            data.generated_steel_config_dir,
            steel_dir.to_string_lossy().to_string()
        );
        assert!(steel_dir.join("cogs/recentf.scm").exists());
        assert!(steel_dir.join("splash.scm").exists());
        assert!(steel_dir.join("cogs/keymaps.scm").exists());
        assert!(steel_dir.join("cogs/labelled-buffers.scm").exists());
    }

    // Defends: declarative helix.plugins toggles copy selected Steel plugin files into the generated Helix config directory and load them from init.scm.
    #[test]
    fn helix_materialization_copies_enabled_user_steel_plugin() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let config_dir = tmp.path().join("config");
        let state_dir = tmp.path().join("state");
        write_runtime_layout(&runtime_dir);
        fs::create_dir_all(config_dir.join("helix/steel_plugins/cogs")).unwrap();
        fs::write(
            config_dir.join("settings.jsonc"),
            r#"{
  "helix": {
    "plugins": {
      "recentf": true,
      "splash": false
    }
  }
}
"#,
        )
        .unwrap();
        fs::write(
            config_dir.join("helix/steel_plugins/cogs/recentf.scm"),
            "(provide recentf-open-files recentf-snapshot)\n",
        )
        .unwrap();

        let data = generate_helix_materialization(&HelixMaterializationRequest {
            runtime_dir,
            config_dir,
            state_dir: state_dir.clone(),
        })
        .unwrap();

        let generated_recentf = state_dir.join("configs/helix/cogs/recentf.scm");
        let generated_init = fs::read_to_string(state_dir.join("configs/helix/init.scm")).unwrap();
        let generated_helix =
            fs::read_to_string(state_dir.join("configs/helix/helix.scm")).unwrap();

        assert_eq!(data.enabled_steel_plugins, vec!["recentf"]);
        assert!(generated_recentf.exists());
        assert!(!state_dir.join("configs/helix/splash.scm").exists());
        assert_eq!(
            fs::read_to_string(&generated_recentf).unwrap(),
            "(provide recentf-open-files recentf-snapshot)\n"
        );
        assert!(generated_init.contains("(recentf-snapshot)"));
        assert!(!generated_init.contains("show-splash"));
        assert!(generated_helix.contains("(provide recentf-open-files)"));
    }
}
