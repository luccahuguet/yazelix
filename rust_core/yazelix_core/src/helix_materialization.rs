use crate::bridge::{CoreError, ErrorClass};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

pub(crate) const MANAGED_REVEAL_COMMAND: &str = ":sh yzx reveal \"%{buffer_name}\"";
const REVEAL_KEY: &str = "A-r";

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
    pub template_path: String,
    pub user_config_merged: bool,
    pub reveal_binding_enforced: bool,
    pub import_notice: Option<HelixImportNotice>,
}

struct PreparedHelixConfig {
    template_path: PathBuf,
    user_config_path: PathBuf,
    config: TomlValue,
    user_config_merged: bool,
}

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

    fs::write(&generated_path, output).map_err(|source| {
        CoreError::io(
            "write_helix_config",
            "Could not write the managed Helix config",
            "Check permissions for the Yazelix state directory and retry.",
            generated_path.to_string_lossy(),
            source,
        )
    })?;

    let import_notice = build_import_notice(request, &prepared.user_config_path)?;

    Ok(HelixMaterializationData {
        generated_path: generated_path.to_string_lossy().into_owned(),
        template_path: prepared.template_path.to_string_lossy().into_owned(),
        user_config_merged: prepared.user_config_merged,
        reveal_binding_enforced: true,
        import_notice,
    })
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

    let user_config_path = config_dir
        .join("user_configs")
        .join("helix")
        .join("config.toml");

    let user_config_merged = if user_config_path.exists() {
        let user_content = fs::read_to_string(&user_config_path).map_err(|source| {
            CoreError::io(
                "read_helix_user_config",
                "Could not read the user Helix config override",
                "Check permissions for ~/.config/yazelix/user_configs/helix/config.toml and retry.",
                user_config_path.to_string_lossy(),
                source,
            )
        })?;
        let user_config: TomlValue = toml::from_str(&user_content).map_err(|source| {
            CoreError::toml(
                "parse_helix_user_config",
                "Could not parse the user Helix config override as TOML",
                "Fix the TOML syntax in ~/.config/yazelix/user_configs/helix/config.toml and retry.",
                user_config_path.to_string_lossy(),
                source,
            )
        })?;
        deep_merge_toml(&mut config, &user_config);
        true
    } else {
        false
    };

    enforce_reveal_binding(&mut config);

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

fn enforce_reveal_binding(config: &mut TomlValue) {
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
                REVEAL_KEY.into(),
                TomlValue::String(MANAGED_REVEAL_COMMAND.into()),
            );
        }
        _ => {}
    }
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
