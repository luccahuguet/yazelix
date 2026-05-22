mod writer;

use crate::action_registry::YAZI_ACTIONS;
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::control_plane::config_dir_from_env;
use crate::user_config_paths;
use crate::yazi_render_plan::{YaziRenderPlanRequest, compute_yazi_render_plan};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;
pub use writer::YaziManagedFileStatus;

const YAZI_KEYBINDINGS_CONFIG_KEY: &str = "yazi_keybindings";

#[derive(Debug, Clone)]
pub struct YaziMaterializationRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub yazi_config_dir: PathBuf,
    pub sync_static_assets: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct YaziMaterializationData {
    pub merged_config_dir: String,
    pub resolved_theme: String,
    pub sort_by: String,
    pub missing_plugins: Vec<String>,
    pub synced_static_assets: bool,
    pub user_config_merged: bool,
    pub user_keymap_merged: bool,
    pub user_init_appended: bool,
    pub managed_files: Vec<YaziManagedFileStatus>,
}

struct UserOverridePaths {
    yazi_toml: PathBuf,
    keymap_toml: PathBuf,
    init_lua: PathBuf,
    plugins_dir: PathBuf,
    flavors_dir: PathBuf,
}

pub fn generate_yazi_materialization(
    request: &YaziMaterializationRequest,
) -> Result<YaziMaterializationData, CoreError> {
    let normalized = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: true,
    })?;
    let render_plan = compute_yazi_render_plan(&build_yazi_render_plan_request(
        &normalized.normalized_config,
    ))?;
    let yazi_keybindings = resolve_yazi_keybindings(&normalized.normalized_config)?;
    let config_dir = config_dir_from_env()?;
    crate::managed_user_config_stubs::ensure_yazi_surface_stub(&config_dir)?;
    let user_paths = resolve_user_override_paths(&config_dir)?;
    let user_yazi_config = read_optional_managed_toml_override(
        &user_paths.yazi_toml,
        "read_yazi_user_config",
        "Could not read the managed Yazi override config",
        "Fix the managed override file or remove it, then retry.",
    )?;
    let user_keymap = read_optional_managed_toml_override(
        &user_paths.keymap_toml,
        "read_yazi_keymap_override",
        "Could not read the managed Yazi keymap override",
        "Fix the managed Yazi keymap override or remove it, then retry.",
    )?;
    let user_init_lua = read_optional_managed_text_override(
        &user_paths.init_lua,
        "read_yazi_init_override",
        "Could not read the managed Yazi init.lua override",
        "Fix the managed Yazi init.lua override or remove it, then retry.",
    )?;
    let source_dir = request.runtime_dir.join("configs").join("yazi");
    let semantic_keymap = build_semantic_yazi_keymap(&yazi_keybindings);
    let written = writer::write_yazi_config_pack(&writer::YaziConfigPackWriteRequest {
        source_dir: &source_dir,
        output_dir: &request.yazi_config_dir,
        runtime_dir: &request.runtime_dir,
        render_plan: &render_plan,
        user_yazi_config: user_yazi_config.as_ref(),
        user_keymap: user_keymap.as_ref(),
        user_init_lua: user_init_lua.as_deref(),
        user_plugins_dir: &user_paths.plugins_dir,
        user_flavors_dir: &user_paths.flavors_dir,
        semantic_keymap: &semantic_keymap,
        sync_static_assets: request.sync_static_assets,
    })?;

    Ok(YaziMaterializationData {
        merged_config_dir: request.yazi_config_dir.to_string_lossy().to_string(),
        resolved_theme: render_plan.resolved_theme,
        sort_by: render_plan.sort_by,
        missing_plugins: written.missing_plugins,
        synced_static_assets: written.synced_static_assets,
        user_config_merged: user_yazi_config.is_some(),
        user_keymap_merged: user_keymap.is_some(),
        user_init_appended: written.user_init_appended,
        managed_files: written.managed_files,
    })
}

pub fn generated_yazi_static_assets_missing(
    runtime_dir: &Path,
    yazi_config_dir: &Path,
) -> Result<bool, CoreError> {
    writer::bundled_yazi_assets_missing(
        &runtime_dir.join("configs").join("yazi"),
        yazi_config_dir,
        runtime_dir,
    )
}

fn build_yazi_render_plan_request(
    normalized: &JsonMap<String, JsonValue>,
) -> YaziRenderPlanRequest {
    let yazi_plugins = normalized
        .get("yazi_plugins")
        .and_then(|value| match value {
            JsonValue::Null => None,
            JsonValue::Array(values) => Some(
                values
                    .iter()
                    .filter_map(JsonValue::as_str)
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        });

    YaziRenderPlanRequest {
        yazi_theme: normalized
            .get("yazi_theme")
            .and_then(JsonValue::as_str)
            .unwrap_or("default")
            .to_string(),
        yazi_sort_by: normalized
            .get("yazi_sort_by")
            .and_then(JsonValue::as_str)
            .unwrap_or("alphabetical")
            .to_string(),
        yazi_plugins,
    }
}

fn resolve_user_override_paths(config_dir: &Path) -> Result<UserOverridePaths, CoreError> {
    Ok(UserOverridePaths {
        yazi_toml: resolve_managed_yazi_user_file(config_dir, "yazi.toml")?,
        keymap_toml: resolve_managed_yazi_user_file(config_dir, "keymap.toml")?,
        init_lua: resolve_managed_yazi_user_file(config_dir, "init.lua")?,
        plugins_dir: resolve_managed_yazi_user_dir(
            config_dir,
            user_config_paths::yazi_plugins_dir(config_dir),
            user_config_paths::flat_yazi_plugins_dir(config_dir),
            "Yazi plugins directory",
        )?,
        flavors_dir: user_config_paths::yazi_flavors_dir(config_dir),
    })
}

fn resolve_managed_yazi_user_file(
    config_dir: &Path,
    file_name: &str,
) -> Result<PathBuf, CoreError> {
    let (current_path, flat_path, old_managed_path) = match file_name {
        "yazi.toml" => (
            user_config_paths::yazi_config(config_dir),
            user_config_paths::flat_yazi_config(config_dir),
            user_config_paths::legacy_yazi_config(config_dir),
        ),
        "keymap.toml" => (
            user_config_paths::yazi_keymap(config_dir),
            user_config_paths::flat_yazi_keymap(config_dir),
            user_config_paths::legacy_yazi_keymap(config_dir),
        ),
        "init.lua" => (
            user_config_paths::yazi_init(config_dir),
            user_config_paths::flat_yazi_init(config_dir),
            user_config_paths::legacy_yazi_init(config_dir),
        ),
        _ => unreachable!("supported Yazi override file"),
    };
    let current_path = user_config_paths::resolve_current_config_file(
        &current_path,
        &old_managed_path,
        &format!("Yazi {file_name} override"),
    )?;
    let legacy_path = config_dir
        .join("configs")
        .join("yazi")
        .join("user")
        .join(file_name);
    reject_old_flat_yazi_path(
        &flat_path,
        &current_path,
        &format!("Yazi {file_name} override"),
    )?;

    let current_exists = current_path.exists();
    let legacy_exists = legacy_path.exists();

    if current_exists && legacy_exists {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "duplicate_yazi_user_override",
            format!(
                "Yazelix found duplicate Yazi user config files for {file_name}.\nmanaged path: {}\nlegacy path: {}\n\nKeep only the managed ~/.config/yazelix/yazi/ copy. Move or delete the legacy configs/yazi/user file so Yazelix has one clear owner.",
                current_path.to_string_lossy(),
                legacy_path.to_string_lossy(),
            ),
            "Keep only the managed ~/.config/yazelix/yazi/ copy, then retry.",
            json!({
                "file_name": file_name,
                "current_path": current_path.to_string_lossy(),
                "legacy_path": legacy_path.to_string_lossy(),
            }),
        ));
    }

    if legacy_exists {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "legacy_yazi_user_override",
            format!(
                "Yazelix found a legacy Yazi user config file for {file_name}.\nlegacy path: {}\nmanaged path: {}\n\nYazelix no longer relocates configs/yazi/user overrides during normal config generation.\nUse `yzx import yazi` to move native or legacy overrides into `~/.config/yazelix/`, or move the file manually.",
                legacy_path.to_string_lossy(),
                current_path.to_string_lossy(),
            ),
            "Move the override into `~/.config/yazelix/yazi/` with `yzx import yazi`, then retry.",
            json!({
                "file_name": file_name,
                "current_path": current_path.to_string_lossy(),
                "legacy_path": legacy_path.to_string_lossy(),
            }),
        ));
    }

    Ok(current_path)
}

fn resolve_managed_yazi_user_dir(
    config_dir: &Path,
    current_path: PathBuf,
    flat_path: PathBuf,
    label: &str,
) -> Result<PathBuf, CoreError> {
    reject_old_flat_yazi_path(&flat_path, &current_path, label)?;
    let legacy_path = config_dir.join("configs").join("yazi").join("user");
    if legacy_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "legacy_yazi_user_override",
            format!(
                "Yazelix found a legacy Yazi user config directory.\nlegacy path: {}\nmanaged path: {}\n\nYazelix now stores Yazi-owned files under ~/.config/yazelix/yazi/.",
                legacy_path.to_string_lossy(),
                user_config_paths::yazi_config_dir(config_dir).to_string_lossy(),
            ),
            "Move the old configs/yazi/user contents into `~/.config/yazelix/yazi/`, then retry.",
            json!({
                "current_path": user_config_paths::yazi_config_dir(config_dir).to_string_lossy(),
                "legacy_path": legacy_path.to_string_lossy(),
            }),
        ));
    }
    Ok(current_path)
}

fn reject_old_flat_yazi_path(
    flat_path: &Path,
    current_path: &Path,
    label: &str,
) -> Result<(), CoreError> {
    if !flat_path.exists() {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Config,
        "flat_yazi_user_override",
        format!(
            "Yazelix found an old flat {label} at {}.\nmanaged Yazi-home path: {}\n\nYazelix now stores Yazi-owned files under ~/.config/yazelix/yazi/ so package.toml, plugins/, flavors/, and native file names share one managed home.",
            flat_path.to_string_lossy(),
            current_path.to_string_lossy(),
        ),
        "Move the old flat Yazi path into `~/.config/yazelix/yazi/`, then retry.",
        json!({
            "current_path": current_path.to_string_lossy(),
            "flat_path": flat_path.to_string_lossy(),
            "label": label,
        }),
    ))
}

fn read_optional_managed_toml_override(
    path: &Path,
    code: &str,
    message: &str,
    remediation: &str,
) -> Result<Option<toml::Table>, CoreError> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(path).map_err(|source| {
        CoreError::io(code, message, remediation, path.to_string_lossy(), source)
    })?;
    toml::from_str::<toml::Table>(&raw)
        .map(Some)
        .map_err(|source| {
            CoreError::toml(
                "invalid_toml",
                message,
                remediation,
                path.to_string_lossy(),
                source,
            )
        })
}

fn read_optional_managed_text_override(
    path: &Path,
    code: &str,
    message: &str,
    remediation: &str,
) -> Result<Option<String>, CoreError> {
    if !path.exists() {
        return Ok(None);
    }
    std::fs::read_to_string(path)
        .map(Some)
        .map_err(|source| CoreError::io(code, message, remediation, path.to_string_lossy(), source))
}

fn default_yazi_keybindings() -> BTreeMap<String, Vec<String>> {
    YAZI_ACTIONS
        .iter()
        .map(|spec| {
            (
                spec.action.local_id.to_string(),
                spec.action
                    .default_keys
                    .iter()
                    .map(|key| (*key).to_string())
                    .collect(),
            )
        })
        .collect()
}

fn resolve_yazi_keybindings(
    config: &JsonMap<String, JsonValue>,
) -> Result<BTreeMap<String, Vec<String>>, CoreError> {
    let mut resolved = default_yazi_keybindings();
    let Some(value) = config.get(YAZI_KEYBINDINGS_CONFIG_KEY) else {
        validate_yazi_keybindings(&resolved)?;
        return Ok(resolved);
    };
    let Some(object) = value.as_object() else {
        return Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_yazi_keybindings",
            "yazi.keybindings must be an object whose values are lists of Yazi key strings.",
            "Use settings such as `\"open_zoxide_in_editor\": [\"<A-z>\"]`, or remove yazi.keybindings to use Yazelix defaults.",
            json!({ "actual": value }),
        ));
    };

    for (action, raw_keys) in object {
        if !is_supported_yazi_keybinding_action(action) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_yazi_keybinding_action",
                format!("Unsupported Yazi keybinding action: {action}."),
                "Use one of the supported Yazelix Yazi action ids, or remove the unsupported keybinding entry.",
                json!({
                    "action": action,
                    "supported_actions": supported_yazi_keybinding_actions(),
                }),
            ));
        }
        let Some(values) = raw_keys.as_array() else {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_yazi_keybinding_keys",
                format!("yazi.keybindings.{action} must be a list of Yazi key strings."),
                "Use a list such as `[\"<A-z>\"]`, or an empty list to disable that Yazelix action binding.",
                json!({ "action": action, "actual": raw_keys }),
            ));
        };
        let mut keys = Vec::with_capacity(values.len());
        for value in values {
            let Some(raw_key) = value.as_str() else {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_yazi_keybinding_key",
                    format!("yazi.keybindings.{action} contains a non-string key."),
                    "Use Yazi key strings such as \"<A-z>\" or \"g\".",
                    json!({ "action": action, "actual": value }),
                ));
            };
            let key = raw_key.trim();
            if key.is_empty() || key.contains('\n') || key.contains('\r') {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "invalid_yazi_keybinding_key",
                    format!("yazi.keybindings.{action} contains an invalid key string."),
                    "Use a non-empty single-line Yazi key string such as \"<A-z>\".",
                    json!({ "action": action, "actual": raw_key }),
                ));
            }
            keys.push(key.to_string());
        }
        resolved.insert(action.clone(), keys);
    }

    validate_yazi_keybindings(&resolved)?;
    Ok(resolved)
}

fn validate_yazi_keybindings(keybindings: &BTreeMap<String, Vec<String>>) -> Result<(), CoreError> {
    let mut seen = BTreeMap::<String, String>::new();
    for spec in YAZI_ACTIONS {
        let Some(keys) = keybindings.get(spec.action.local_id) else {
            continue;
        };
        if keys.is_empty() && !spec.action.disable_policy.empty_binding_list_allowed() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "disabled_required_yazi_keybinding",
                format!("Yazi action {} cannot be disabled.", spec.action.local_id),
                "Remove the empty list or assign at least one Yazi key string.",
                json!({ "action": spec.action.local_id }),
            ));
        }
        for key in keys {
            if let Some(existing_action) =
                seen.insert(key.clone(), spec.action.local_id.to_string())
            {
                return Err(CoreError::classified(
                    ErrorClass::Config,
                    "duplicate_yazi_keybinding",
                    format!(
                        "Yazi keybinding {key:?} is assigned to both {existing_action} and {}.",
                        spec.action.local_id
                    ),
                    "Give each Yazelix Yazi action a distinct key, or set one action to an empty list to disable its binding.",
                    json!({
                        "key": key,
                        "first_action": existing_action,
                        "second_action": spec.action.local_id,
                    }),
                ));
            }
        }
    }
    Ok(())
}

fn is_supported_yazi_keybinding_action(action: &str) -> bool {
    YAZI_ACTIONS
        .iter()
        .any(|spec| spec.action.local_id == action)
}

fn supported_yazi_keybinding_actions() -> Vec<&'static str> {
    YAZI_ACTIONS
        .iter()
        .map(|spec| spec.action.local_id)
        .collect()
}

fn build_semantic_yazi_keymap(yazi_keybindings: &BTreeMap<String, Vec<String>>) -> toml::Table {
    let mut keymap = toml::Table::new();
    for spec in YAZI_ACTIONS {
        let Some(keys) = yazi_keybindings.get(spec.action.local_id) else {
            continue;
        };
        if keys.is_empty() {
            continue;
        }
        let section = keymap
            .entry(spec.section.to_string())
            .or_insert_with(|| TomlValue::Table(toml::Table::new()));
        let section = section
            .as_table_mut()
            .expect("new semantic Yazi keymap section is a table");
        let entries = section
            .entry(spec.keymap_list.to_string())
            .or_insert_with(|| TomlValue::Array(Vec::new()));
        let entries = entries
            .as_array_mut()
            .expect("new semantic Yazi keymap list is an array");
        for key in keys {
            entries.push(TomlValue::Table(yazi_keymap_entry(spec, key)));
        }
    }
    keymap
}

fn yazi_keymap_entry(spec: &crate::action_registry::YaziActionSpec, key: &str) -> toml::Table {
    let mut entry = toml::Table::new();
    entry.insert(
        "on".to_string(),
        TomlValue::Array(vec![TomlValue::String(key.to_string())]),
    );
    entry.insert(
        "run".to_string(),
        TomlValue::String(spec.action.generated_command.to_string()),
    );
    entry.insert(
        "desc".to_string(),
        TomlValue::String(spec.description.to_string()),
    );
    entry
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::tempdir;

    // Regression: Yazi TOML sidecar arrays replace generated defaults, while opener.edit stays Yazelix-owned.
    #[test]
    fn deep_merge_replaces_arrays_and_preserves_base_edit_opener() {
        let base = toml::from_str::<toml::Table>(
            r#"
[opener]
edit = [{ run = "base-edit" }]

[mgr]
ratio = [1, 4, 3]

[[plugin.prepend_fetchers]]
url = "*"
run = "git"
group = "git"
"#,
        )
        .unwrap();
        let user = toml::from_str::<toml::Table>(
            r#"
[opener]
edit = [{ run = "user-edit" }]
open = [{ run = "user-open" }]

[mgr]
ratio = [1, 4, 0]

[[plugin.prepend_fetchers]]
url = "*/"
run = "extra"
group = "extra"
"#,
        )
        .unwrap();

        let mut merged = writer::merge_yazi_toml_config(base.clone(), user);
        writer::preserve_yazelix_edit_opener(&base, &mut merged);

        assert_eq!(
            merged
                .get("opener")
                .and_then(TomlValue::as_table)
                .and_then(|opener| opener.get("edit"))
                .unwrap(),
            base.get("opener")
                .and_then(TomlValue::as_table)
                .and_then(|opener| opener.get("edit"))
                .unwrap()
        );
        assert_eq!(
            merged
                .get("mgr")
                .and_then(TomlValue::as_table)
                .and_then(|mgr| mgr.get("ratio"))
                .and_then(TomlValue::as_array)
                .unwrap(),
            &vec![1.into(), 4.into(), 0.into()]
        );
        assert_eq!(
            merged
                .get("plugin")
                .and_then(TomlValue::as_table)
                .and_then(|plugin| plugin.get("prepend_fetchers"))
                .and_then(TomlValue::as_array)
                .and_then(|fetchers| fetchers.first())
                .and_then(TomlValue::as_table)
                .and_then(|fetcher| fetcher.get("group"))
                .and_then(TomlValue::as_str),
            Some("extra")
        );
    }

    // Defends: missing bundled asset targets are detected so warm Yazi generation can self-heal deleted files.
    #[test]
    fn bundled_asset_detection_flags_missing_targets() {
        let temp = tempdir().unwrap();
        let source_root = temp
            .path()
            .join("runtime/configs/yazi/plugins/example.yazi");
        let target_root = temp.path().join("state/configs/yazi/plugins");
        fs::create_dir_all(&source_root).unwrap();
        fs::write(source_root.join("main.lua"), "print('hi')").unwrap();
        fs::create_dir_all(&target_root).unwrap();

        assert!(
            writer::asset_tree_missing_targets(
                &temp.path().join("runtime/configs/yazi/plugins"),
                &target_root,
                &temp.path().join("runtime"),
            )
            .unwrap()
        );

        let target_plugin = target_root.join("example.yazi");
        fs::create_dir_all(&target_plugin).unwrap();
        fs::write(target_plugin.join("main.lua"), "print('stale')").unwrap();

        assert!(
            writer::asset_tree_missing_targets(
                &temp.path().join("runtime/configs/yazi/plugins"),
                &target_root,
                &temp.path().join("runtime"),
            )
            .unwrap()
        );
    }

    // Regression: sidebar-state must not rely on a fixed startup delay before its only Zellij registration attempt.
    #[test]
    fn sidebar_state_registers_with_orchestrator_asynchronously_with_bounded_retry() {
        let source = include_str!("../../../configs/yazi/plugins/sidebar-state.yazi/main.lua");

        assert!(source.contains("ya.async(function()"));
        assert!(source.contains("REGISTER_RETRY_DELAYS_SECONDS"));
        assert!(source.contains("REGISTER_RETRYABLE_RESULTS"));
        assert!(source.contains("pipe_sidebar_state_registration(payload)"));
        assert!(source.contains("generation ~= sidebar_state_generation"));
        assert!(!source.contains("STARTUP_REGISTER_DELAY_SECONDS"));
        assert!(!source.contains("os.execute"));
    }

    // Regression: the bundled Yazi Starship config must copy into the generated surface without becoming read-only or drift-prone.
    #[test]
    fn syncs_starship_config_into_generated_surface() {
        let tmp = tempdir().unwrap();
        let source = tmp.path().join("yazelix_starship.toml");
        let target = tmp.path().join("generated").join("yazelix_starship.toml");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&source, "# YAZELIX STARSHIP CONFIG FOR YAZI SIDEBAR\n").unwrap();

        writer::sync_starship_config(&source, &target).unwrap();

        let copied = fs::read_to_string(&target).unwrap();
        assert!(copied.contains("YAZELIX STARSHIP CONFIG FOR YAZI SIDEBAR"));
        assert!(!fs::metadata(target).unwrap().permissions().readonly());
    }
}
