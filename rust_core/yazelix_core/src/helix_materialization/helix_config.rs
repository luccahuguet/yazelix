use super::{
    MANAGED_COMMAND_MODE_COMMAND, MANAGED_COMMAND_MODE_KEY, MANAGED_REVEAL_COMMAND, REVEAL_KEY,
};
use crate::bridge::{CoreError, ErrorClass};
use crate::user_config_paths;
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml::Value as TomlValue;

pub(super) struct PreparedHelixConfig {
    pub(super) template_path: PathBuf,
    pub(super) user_config_path: PathBuf,
    pub(super) config: TomlValue,
    pub(super) user_config_merged: bool,
}

pub(super) fn prepare_managed_helix_config(
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

    let current_config_path = user_config_paths::helix_config(config_dir);
    let flat_config_path = user_config_paths::flat_helix_config(config_dir);
    let legacy_config_path = user_config_paths::legacy_helix_config(config_dir);
    let user_config_path = user_config_paths::resolve_current_config_file_against_legacy_paths(
        &current_config_path,
        &[&flat_config_path, &legacy_config_path],
        "Helix override",
    )?;

    let user_config_merged = if user_config_path.exists() {
        let user_content = fs::read_to_string(&user_config_path).map_err(|source| {
            CoreError::io(
                "read_helix_user_config",
                "Could not read the user Helix config override",
                "Check permissions for ~/.config/yazelix/helix/config.toml and retry.",
                user_config_path.to_string_lossy(),
                source,
            )
        })?;
        let user_config: TomlValue = toml::from_str(&user_content).map_err(|source| {
            CoreError::toml(
                "parse_helix_user_config",
                "Could not parse the user Helix config override as TOML",
                "Fix the TOML syntax in ~/.config/yazelix/helix/config.toml and retry.",
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

    if let TomlValue::Table(t) = normal_table {
        t.insert(
            MANAGED_COMMAND_MODE_KEY.into(),
            TomlValue::String(MANAGED_COMMAND_MODE_COMMAND.into()),
        );
        t.insert(
            REVEAL_KEY.into(),
            TomlValue::String(MANAGED_REVEAL_COMMAND.into()),
        );
    }
}
