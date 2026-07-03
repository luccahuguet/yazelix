use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use ratconfig::toml_adapter::{get_toml_path, parse_toml_value};
use ratconfig::ConfigUiFileAction;
use serde_json::Value as JsonValue;

use crate::{
    catalog::*,
    common::*,
    native_config::{write_mars_config_field, write_starship_config_field},
    paths::ConfigPaths,
    root_config::{default_config_value, write_config_field},
    zellij_sidecar::{write_zellij_config_field, zellij_field_value, ZellijSidecar},
};

pub(crate) struct FileActionSpec {
    source_id: &'static str,
    action_id: &'static str,
    tab: &'static str,
    label: &'static str,
    description: &'static str,
    path: PathBuf,
    starter: &'static str,
}
pub(crate) fn build_file_actions(paths: &ConfigPaths) -> Vec<ConfigUiFileAction> {
    file_action_specs(paths)
        .into_iter()
        .map(|spec| ConfigUiFileAction {
            source_id: spec.source_id.to_string(),
            action_id: spec.action_id.to_string(),
            tab: spec.tab.to_string(),
            label: spec.label.to_string(),
            description: spec.description.to_string(),
            exists: spec.path.exists(),
            read_only: path_read_only(&spec.path),
            create_if_missing: true,
            disabled_reason: None,
            path: spec.path,
        })
        .collect()
}
fn file_action_specs(paths: &ConfigPaths) -> [FileActionSpec; 8] {
    [
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_CONFIG,
            tab: TAB_HELIX,
            label: "helix/config.toml",
            description: "Open the managed Helix TOML config file.",
            path: paths.helix_config.clone(),
            starter: HELIX_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_LANGUAGES,
            tab: TAB_HELIX,
            label: "helix/languages.toml",
            description: "Open the managed Helix language override file.",
            path: paths.helix_languages.clone(),
            starter: HELIX_LANGUAGES_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_MODULE,
            tab: TAB_HELIX,
            label: "helix/helix.scm",
            description: "Open the managed Helix Steel module file.",
            path: paths.helix_module.clone(),
            starter: HELIX_MODULE_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_HELIX,
            action_id: ACTION_HELIX_INIT,
            tab: TAB_HELIX,
            label: "helix/init.scm",
            description: "Open the managed Helix Steel startup file.",
            path: paths.helix_init.clone(),
            starter: HELIX_INIT_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_NU_ENV,
            tab: TAB_ADVANCED,
            label: "nu/env.nu",
            description: "Open the user Nushell environment file.",
            path: paths.nu_env.clone(),
            starter: NU_ENV_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_NU_CONFIG,
            tab: TAB_ADVANCED,
            label: "nu/config.nu",
            description: "Open the user Nushell config file.",
            path: paths.nu_config.clone(),
            starter: NU_CONFIG_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_YAZI_INIT,
            tab: TAB_ADVANCED,
            label: "yazi/init.lua",
            description: "Open the managed Yazi user init.lua file.",
            path: paths.yazi_init.clone(),
            starter: YAZI_INIT_STARTER,
        },
        FileActionSpec {
            source_id: SOURCE_ADVANCED,
            action_id: ACTION_YAZI_KEYMAP,
            tab: TAB_ADVANCED,
            label: "yazi/keymap.toml",
            description: "Open the managed Yazi user keymap.toml file.",
            path: paths.yazi_keymap.clone(),
            starter: YAZI_KEYMAP_STARTER,
        },
    ]
}
pub(crate) fn write_source_field(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
    value: &JsonValue,
) -> Result<()> {
    match source_id {
        SOURCE_CONFIG => {
            reject_read_only_source(&paths.root, source_id)?;
            write_config_field(&paths.root, field_path, value)
        }
        SOURCE_MARS => {
            reject_read_only_source(&paths.mars, source_id)?;
            write_mars_config_field(&paths.mars, field_path, value)
        }
        SOURCE_ZELLIJ => {
            reject_read_only_source(&paths.zellij, source_id)?;
            write_zellij_config_field(&paths.zellij, field_path, value)
        }
        SOURCE_STARSHIP => {
            reject_read_only_source(&paths.starship, source_id)?;
            write_starship_config_field(&paths.starship, field_path, value)
        }
        _ => Err(error(format!("unknown config source: {source_id}"))),
    }
}
pub(crate) fn write_source_default(
    paths: &ConfigPaths,
    source_id: &str,
    field_path: &str,
) -> Result<()> {
    let value = match source_id {
        SOURCE_CONFIG => default_config_value(field_path)?,
        SOURCE_MARS => {
            let default = parse_toml_value(DEFAULT_MARS_CONFIG_TOML)
                .map_err(|error| boxed_debug("invalid default Mars config", error))?;
            get_toml_path(&default, field_path)
                .cloned()
                .ok_or_else(|| error(format!("unknown Mars config path: {field_path}")))?
        }
        SOURCE_ZELLIJ => zellij_field_value(&ZellijSidecar::default(), field_path),
        SOURCE_STARSHIP => {
            let default = parse_toml_value(DEFAULT_STARSHIP_CONFIG_TOML)
                .map_err(|error| boxed_debug("invalid default Starship config", error))?;
            get_toml_path(&default, field_path)
                .cloned()
                .ok_or_else(|| error(format!("unknown Starship config path: {field_path}")))?
        }
        _ => return Err(error(format!("unknown config source: {source_id}"))),
    };
    write_source_field(paths, source_id, field_path, &value)
}
pub(crate) fn open_file_action(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
    create_if_missing: bool,
) -> Result<()> {
    let editor = configured_editor()?;
    prepare_file_action(paths, source_id, action_id, path, create_if_missing)?;
    let status = Command::new(&editor).arg(path).status().map_err(|error| {
        io::Error::other(format!(
            "failed to launch editor `{}`: {error}",
            editor.display()
        ))
    })?;
    if !status.success() {
        return Err(error(format!(
            "editor `{}` exited with status {status}",
            editor.display()
        )));
    }
    Ok(())
}
pub(crate) fn prepare_file_action(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
    create_if_missing: bool,
) -> Result<()> {
    let spec = file_action_spec(paths, source_id, action_id, path)?;
    let is_helix_steel_action = spec.source_id == SOURCE_HELIX
        && matches!(spec.action_id, ACTION_HELIX_MODULE | ACTION_HELIX_INIT);
    if spec.path.exists() {
        if is_helix_steel_action {
            ensure_helix_steel_pair(paths)?;
        }
        return Ok(());
    }
    if !create_if_missing {
        return Err(error(format!("config file is missing: {}", path.display())));
    }
    atomic_write(&spec.path, spec.starter)?;
    if is_helix_steel_action {
        ensure_helix_steel_pair(paths)?;
    }
    Ok(())
}
fn file_action_spec(
    paths: &ConfigPaths,
    source_id: &str,
    action_id: &str,
    path: &Path,
) -> Result<FileActionSpec> {
    let Some(spec) = file_action_specs(paths)
        .into_iter()
        .find(|spec| spec.source_id == source_id && spec.action_id == action_id)
    else {
        return Err(error(format!("unknown file action: {action_id}")));
    };
    if spec.path != path {
        return Err(error(format!(
            "file action `{action_id}` does not own {}",
            path.display()
        )));
    }
    Ok(spec)
}
fn ensure_helix_steel_pair(paths: &ConfigPaths) -> Result<()> {
    if !paths.helix_module.exists() {
        atomic_write(&paths.helix_module, HELIX_MODULE_STARTER)?;
    }
    if !paths.helix_init.exists() {
        atomic_write(&paths.helix_init, HELIX_INIT_STARTER)?;
    }
    Ok(())
}
fn configured_editor() -> Result<PathBuf> {
    ["YAZELIX_NEXT_EDITOR", "VISUAL", "EDITOR"]
        .into_iter()
        .find_map(|key| env::var_os(key).filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .ok_or_else(|| error("no editor configured; set YAZELIX_NEXT_EDITOR, VISUAL, or EDITOR"))
}
pub(crate) fn edit_text_externally(input: &str) -> Result<String> {
    edit_text_with_editor(input, &configured_editor()?)
}
pub(crate) fn edit_text_with_editor(input: &str, editor: &Path) -> Result<String> {
    let path = external_text_edit_path();
    fs::write(&path, input)?;
    let status = Command::new(editor).arg(&path).status().map_err(|error| {
        io::Error::other(format!(
            "failed to launch editor `{}`: {error}",
            editor.display()
        ))
    })?;
    if !status.success() {
        let _ = fs::remove_file(&path);
        return Err(error(format!(
            "editor `{}` exited with status {status}",
            editor.display()
        )));
    }

    let read_result = fs::read_to_string(&path);
    let _ = fs::remove_file(&path);
    let mut text = read_result?;
    if text.ends_with('\n') {
        text.pop();
        if text.ends_with('\r') {
            text.pop();
        }
    }
    Ok(text)
}
fn external_text_edit_path() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    env::temp_dir().join(format!("yzn-config-edit-{}-{nonce}.txt", process::id()))
}
