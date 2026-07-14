use std::{fs, path::Path};

use toml::{Value as TomlValue, map::Map as TomlMap};

use crate::common::*;

const REVEAL_KEY: &str = "A-r";
const REVEAL_COMMAND: &str = r#":sh yzx reveal "%{buffer_name}""#;

pub(crate) fn write_effective_helix_config(
    packaged_path: &Path,
    user_path: &Path,
    output_path: &Path,
) -> Result<()> {
    let mut config = read_toml_config(packaged_path, "packaged Helix config")?;
    if user_path.is_file() {
        let user_config = read_toml_config(user_path, "user Helix config")?;
        deep_merge_toml(&mut config, &user_config);
    }
    enforce_reveal_binding(&mut config)?;
    let output = toml::to_string_pretty(&config)
        .map_err(|err| error(format!("could not serialize effective Helix config: {err}")))?;
    atomic_write(output_path, &output)
}

fn read_toml_config(path: &Path, label: &str) -> Result<TomlValue> {
    let raw = fs::read_to_string(path)
        .map_err(|err| error(format!("could not read {label} {}: {err}", path.display())))?;
    toml::from_str(&raw)
        .map_err(|err| error(format!("could not parse {label} {}: {err}", path.display())))
}

fn enforce_reveal_binding(config: &mut TomlValue) -> Result<()> {
    let root = config
        .as_table_mut()
        .ok_or_else(|| error("effective Helix config root must be a TOML table"))?;
    let keys = table_entry(root, "keys", "[keys]")?;
    let normal = table_entry(keys, "normal", "[keys.normal]")?;
    normal.insert(
        REVEAL_KEY.to_string(),
        TomlValue::String(REVEAL_COMMAND.to_string()),
    );
    Ok(())
}

fn table_entry<'a>(
    table: &'a mut TomlMap<String, TomlValue>,
    key: &str,
    label: &str,
) -> Result<&'a mut TomlMap<String, TomlValue>> {
    table
        .entry(key.to_string())
        .or_insert_with(|| TomlValue::Table(TomlMap::new()))
        .as_table_mut()
        .ok_or_else(|| error(format!("{label} must be a TOML table")))
}
