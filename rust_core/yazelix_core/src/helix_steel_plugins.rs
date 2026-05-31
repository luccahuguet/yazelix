use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SteelPluginConfig {
    pub enabled: Vec<String>,
    pub extra: Vec<SteelPluginManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SteelPluginManifest {
    pub id: String,
    pub source_relative_path: String,
    pub support_files: Vec<String>,
    pub public_commands: Vec<SteelPluginManifestCommand>,
    pub internal_commands: Vec<SteelPluginManifestCommand>,
    pub startup_commands: Vec<String>,
    pub startup_condition: Option<SteelPluginStartupCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SteelPluginManifestCommand {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SteelPluginStartupCondition {
    ShowSplash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SteelPluginManifestError {
    pub path: String,
    pub message: String,
}

impl SteelPluginManifest {
    pub(crate) fn command_names(&self) -> Vec<&str> {
        self.public_commands
            .iter()
            .chain(self.internal_commands.iter())
            .map(|command| command.name.as_str())
            .collect()
    }

    pub(crate) fn startup_condition_matches(&self, show_splash: bool) -> bool {
        match self.startup_condition {
            Some(SteelPluginStartupCondition::ShowSplash) => show_splash,
            None => true,
        }
    }
}

pub(crate) fn parse_steel_plugin_config(
    value: Option<&JsonValue>,
) -> Result<SteelPluginConfig, SteelPluginManifestError> {
    let Some(value) = value else {
        return Ok(SteelPluginConfig {
            enabled: Vec::new(),
            extra: Vec::new(),
        });
    };
    let JsonValue::Object(config) = value else {
        return Err(manifest_error(
            "helix.steel_plugins",
            "helix.steel_plugins must be an object with enabled and extra fields.",
        ));
    };

    for key in config.keys() {
        if !matches!(key.as_str(), "enabled" | "extra") {
            return Err(manifest_error(
                format!("helix.steel_plugins.{key}"),
                format!("Unknown Helix Steel plugin config field `{key}`."),
            ));
        }
    }

    let enabled = enabled_plugin_ids(config.get("enabled"))?;
    let extra =
        parse_steel_plugin_manifest_array(config.get("extra"), "helix.steel_plugins.extra")?;

    Ok(SteelPluginConfig { enabled, extra })
}

pub(crate) fn parse_steel_plugin_manifest_array(
    value: Option<&JsonValue>,
    path: &str,
) -> Result<Vec<SteelPluginManifest>, SteelPluginManifestError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let JsonValue::Array(raw_plugins) = value else {
        return Err(manifest_error(
            path,
            format!("{path} must be a list of plugin manifest objects."),
        ));
    };

    let mut manifests = Vec::new();
    let mut ids = BTreeSet::new();
    let mut sources = BTreeSet::new();
    for (index, raw_plugin) in raw_plugins.iter().enumerate() {
        let plugin_path = format!("{path}[{index}]");
        let JsonValue::Object(plugin) = raw_plugin else {
            return Err(manifest_error(
                &plugin_path,
                format!("Each {path} entry must be an object."),
            ));
        };

        validate_manifest_keys(plugin, &plugin_path)?;
        let id = required_manifest_string(plugin.get("id"), &format!("{plugin_path}.id"))?;
        validate_manifest_id(&id, &format!("{plugin_path}.id"))?;
        if !ids.insert(id.clone()) {
            return Err(manifest_error(
                format!("{plugin_path}.id"),
                format!("Duplicate Helix Steel plugin id `{id}`."),
            ));
        }

        let source_relative_path =
            required_manifest_string(plugin.get("source"), &format!("{plugin_path}.source"))?;
        validate_manifest_source_path(&source_relative_path, &format!("{plugin_path}.source"))?;
        if !sources.insert(source_relative_path.clone()) {
            return Err(manifest_error(
                format!("{plugin_path}.source"),
                format!("Duplicate Helix Steel plugin source `{source_relative_path}`."),
            ));
        }

        let support_files = manifest_source_path_list(
            plugin.get("support_files"),
            &format!("{plugin_path}.support_files"),
        )?;
        let command_descriptions =
            command_descriptions(plugin.get("command_descriptions"), &plugin_path)?;
        let mut declared_command_names = BTreeSet::new();
        let public_commands = manifest_command_list(
            plugin.get("public_commands"),
            &format!("{plugin_path}.public_commands"),
            &id,
            &command_descriptions,
            &mut declared_command_names,
        )?;
        let internal_commands = manifest_command_list(
            plugin.get("internal_commands"),
            &format!("{plugin_path}.internal_commands"),
            &id,
            &command_descriptions,
            &mut declared_command_names,
        )?;
        for described in command_descriptions.keys() {
            if !declared_command_names.contains(described) {
                return Err(manifest_error(
                    format!("{plugin_path}.command_descriptions.{described}"),
                    format!(
                        "Description for undeclared Helix Steel command `{described}` in plugin `{id}`."
                    ),
                ));
            }
        }

        let startup_commands = manifest_startup_commands(
            plugin.get("startup_commands"),
            &format!("{plugin_path}.startup_commands"),
            &id,
            &declared_command_names,
        )?;
        let startup_condition = manifest_startup_condition(
            plugin.get("startup_condition"),
            &format!("{plugin_path}.startup_condition"),
        )?;

        manifests.push(SteelPluginManifest {
            id,
            source_relative_path,
            support_files,
            public_commands,
            internal_commands,
            startup_commands,
            startup_condition,
        });
    }

    Ok(manifests)
}

fn enabled_plugin_ids(value: Option<&JsonValue>) -> Result<Vec<String>, SteelPluginManifestError> {
    let ids = string_array(value, "helix.steel_plugins.enabled")?;
    let mut seen = BTreeSet::new();
    for id in &ids {
        validate_manifest_id(id, "helix.steel_plugins.enabled")?;
        if !seen.insert(id.clone()) {
            return Err(manifest_error(
                "helix.steel_plugins.enabled",
                format!("Duplicate enabled Helix Steel plugin id `{id}`."),
            ));
        }
    }
    Ok(ids)
}

fn validate_manifest_keys(
    plugin: &serde_json::Map<String, JsonValue>,
    path: &str,
) -> Result<(), SteelPluginManifestError> {
    for key in plugin.keys() {
        if !matches!(
            key.as_str(),
            "id" | "source"
                | "support_files"
                | "public_commands"
                | "internal_commands"
                | "startup_commands"
                | "startup_condition"
                | "command_descriptions"
        ) {
            return Err(manifest_error(
                format!("{path}.{key}"),
                format!("Unknown Helix Steel plugin manifest field `{key}`."),
            ));
        }
    }
    Ok(())
}

fn required_manifest_string(
    value: Option<&JsonValue>,
    path: &str,
) -> Result<String, SteelPluginManifestError> {
    let Some(value) = value else {
        return Err(manifest_error(path, format!("{path} is required.")));
    };
    let Some(raw) = value.as_str() else {
        return Err(manifest_error(path, format!("{path} must be a string.")));
    };
    Ok(raw.to_string())
}

fn validate_manifest_id(id: &str, path: &str) -> Result<(), SteelPluginManifestError> {
    if id.is_empty() || id.trim() != id {
        return Err(manifest_error(
            path,
            "Helix Steel plugin id must be non-empty and untrimmed.",
        ));
    }
    if !id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        return Err(manifest_error(
            path,
            "Helix Steel plugin id may only contain ASCII letters, numbers, dots, hyphens, and underscores.",
        ));
    }
    Ok(())
}

fn validate_manifest_source_path(source: &str, path: &str) -> Result<(), SteelPluginManifestError> {
    if source.is_empty() || source.trim() != source {
        return Err(manifest_error(
            path,
            "Helix Steel plugin source must be non-empty and untrimmed.",
        ));
    }
    if source.contains('\\') || Path::new(source).is_absolute() {
        return Err(manifest_error(
            path,
            "Helix Steel plugin source must be a relative path below helix/steel_plugins.",
        ));
    }
    if !source.ends_with(".scm") {
        return Err(manifest_error(
            path,
            "Helix Steel plugin source must point to a .scm file.",
        ));
    }
    for segment in source.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(manifest_error(
                path,
                "Helix Steel plugin source cannot contain empty, current, or parent path segments.",
            ));
        }
        if !segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
        {
            return Err(manifest_error(
                path,
                "Helix Steel plugin source segments may only contain ASCII letters, numbers, dots, hyphens, and underscores.",
            ));
        }
    }
    Ok(())
}

fn command_descriptions(
    value: Option<&JsonValue>,
    path: &str,
) -> Result<BTreeMap<String, String>, SteelPluginManifestError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    let JsonValue::Object(raw_descriptions) = value else {
        return Err(manifest_error(
            format!("{path}.command_descriptions"),
            "command_descriptions must be an object of command names to descriptions.",
        ));
    };
    let mut descriptions = BTreeMap::new();
    for (name, raw_description) in raw_descriptions {
        validate_command_name(name, &format!("{path}.command_descriptions.{name}"))?;
        let Some(description) = raw_description.as_str() else {
            return Err(manifest_error(
                format!("{path}.command_descriptions.{name}"),
                "Command description must be a string.",
            ));
        };
        descriptions.insert(name.clone(), description.to_string());
    }
    Ok(descriptions)
}

fn manifest_command_list(
    value: Option<&JsonValue>,
    path: &str,
    plugin_id: &str,
    descriptions: &BTreeMap<String, String>,
    declared_command_names: &mut BTreeSet<String>,
) -> Result<Vec<SteelPluginManifestCommand>, SteelPluginManifestError> {
    let names = string_array(value, path)?;
    let mut commands = Vec::new();
    for name in names {
        validate_command_name(&name, path)?;
        if !declared_command_names.insert(name.clone()) {
            return Err(manifest_error(
                path,
                format!("Duplicate Helix Steel command `{name}` in plugin `{plugin_id}`."),
            ));
        }
        commands.push(SteelPluginManifestCommand {
            description: descriptions
                .get(&name)
                .cloned()
                .unwrap_or_else(|| format!("Steel command from {plugin_id}")),
            name,
        });
    }
    Ok(commands)
}

fn manifest_source_path_list(
    value: Option<&JsonValue>,
    path: &str,
) -> Result<Vec<String>, SteelPluginManifestError> {
    let values = string_array(value, path)?;
    let mut seen = BTreeSet::new();
    for value in &values {
        validate_manifest_source_path(value, path)?;
        if !seen.insert(value.clone()) {
            return Err(manifest_error(
                path,
                format!("Duplicate Helix Steel support file `{value}`."),
            ));
        }
    }
    Ok(values)
}

fn manifest_startup_commands(
    value: Option<&JsonValue>,
    path: &str,
    plugin_id: &str,
    declared_command_names: &BTreeSet<String>,
) -> Result<Vec<String>, SteelPluginManifestError> {
    let names = string_array(value, path)?;
    let mut startup_commands = Vec::new();
    let mut seen = BTreeSet::new();
    for name in names {
        validate_command_name(&name, path)?;
        if !declared_command_names.contains(&name) {
            return Err(manifest_error(
                path,
                format!(
                    "Startup command `{name}` in plugin `{plugin_id}` must also be declared as public or internal."
                ),
            ));
        }
        if !seen.insert(name.clone()) {
            return Err(manifest_error(
                path,
                format!("Duplicate startup command `{name}` in plugin `{plugin_id}`."),
            ));
        }
        startup_commands.push(name);
    }
    Ok(startup_commands)
}

fn manifest_startup_condition(
    value: Option<&JsonValue>,
    path: &str,
) -> Result<Option<SteelPluginStartupCondition>, SteelPluginManifestError> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let Some(raw) = value.as_str() else {
        return Err(manifest_error(path, format!("{path} must be a string.")));
    };
    match raw {
        "show_splash" => Ok(Some(SteelPluginStartupCondition::ShowSplash)),
        _ => Err(manifest_error(
            path,
            format!("Unknown Helix Steel startup condition `{raw}`."),
        )),
    }
}

fn string_array(
    value: Option<&JsonValue>,
    path: &str,
) -> Result<Vec<String>, SteelPluginManifestError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let JsonValue::Array(raw_values) = value else {
        return Err(manifest_error(path, format!("{path} must be a list.")));
    };
    raw_values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| manifest_error(path, format!("{path} entries must all be strings.")))
        })
        .collect()
}

fn validate_command_name(name: &str, path: &str) -> Result<(), SteelPluginManifestError> {
    if name.is_empty() || name.trim() != name {
        return Err(manifest_error(
            path,
            "Helix Steel command names must be non-empty and untrimmed.",
        ));
    }
    if !name.chars().all(|ch| {
        ch.is_ascii_alphanumeric()
            || matches!(
                ch,
                '-' | '_' | '.' | '!' | '?' | '+' | '*' | '/' | '<' | '=' | '>'
            )
    }) {
        return Err(manifest_error(
            path,
            "Helix Steel command names may only contain ASCII letters, numbers, and safe Scheme symbol punctuation.",
        ));
    }
    Ok(())
}

fn manifest_error(path: impl Into<String>, message: impl Into<String>) -> SteelPluginManifestError {
    SteelPluginManifestError {
        path: path.into(),
        message: message.into(),
    }
}
