use super::{STEEL_CONFIG_MODULE, STEEL_INIT_MODULE, SteelCommandMetadata};
use crate::active_config_surface::primary_config_paths;
use crate::atomic_fs::{write_bytes_atomic, write_text_atomic};
use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::helix_steel_plugins::{
    SteelPluginConfig, SteelPluginManifest, SteelPluginManifestCommand, SteelPluginManifestError,
    parse_steel_plugin_config, parse_steel_plugin_manifest_array,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value as TomlValue;

const STEEL_PLUGIN_ROOT: &str = "steel_plugins";
const STEEL_PLUGIN_MANIFEST: &str = "manifest.toml";

#[derive(Debug, Clone, Copy)]
struct SteelCommandSpec {
    name: &'static str,
    owner: &'static str,
    description: &'static str,
    visibility: SteelCommandVisibility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SteelCommandVisibility {
    Public,
    Internal,
}

impl SteelCommandVisibility {
    fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
        }
    }
}

struct ActiveSteelCommand {
    name: String,
    owner: String,
    description: String,
    visibility: SteelCommandVisibility,
}

impl ActiveSteelCommand {
    fn metadata(self) -> SteelCommandMetadata {
        SteelCommandMetadata {
            name: self.name,
            owner: self.owner,
            description: self.description,
            visibility: self.visibility.as_str().to_string(),
        }
    }

    fn is_public(&self) -> bool {
        self.visibility == SteelCommandVisibility::Public
    }
}

impl SteelCommandSpec {
    fn active(self) -> ActiveSteelCommand {
        ActiveSteelCommand {
            name: self.name.to_string(),
            owner: self.owner.to_string(),
            description: self.description.to_string(),
            visibility: self.visibility,
        }
    }
}

pub(super) struct SteelMaterializationData {
    pub(super) config_dir: PathBuf,
    pub(super) helix_module_path: PathBuf,
    pub(super) init_path: PathBuf,
    pub(super) enabled_plugins: Vec<String>,
    pub(super) copied_plugin_files: Vec<String>,
    pub(super) commands: Vec<SteelCommandMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SteelPluginSource {
    Bundled,
    User,
}

struct ActiveSteelPlugin {
    manifest: SteelPluginManifest,
    source: SteelPluginSource,
}

const BASE_STEEL_COMMANDS: &[SteelCommandSpec] = &[
    SteelCommandSpec {
        name: "eval-buffer",
        owner: "helix/ext",
        description: "Evaluate the current Steel buffer",
        visibility: SteelCommandVisibility::Public,
    },
    SteelCommandSpec {
        name: "evalp",
        owner: "helix/ext",
        description: "Evaluate one Steel expression",
        visibility: SteelCommandVisibility::Public,
    },
    SteelCommandSpec {
        name: "yzx-new-shell",
        owner: "yazelix",
        description: "Open a Yazelix terminal pane at the current Helix file or workspace",
        visibility: SteelCommandVisibility::Public,
    },
];

pub(super) fn load_steel_plugin_selection(
    runtime_dir: &Path,
    config_dir: &Path,
) -> Result<SteelPluginConfig, CoreError> {
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

    parse_steel_plugin_config(normalized.get("helix_steel_plugins"))
        .map_err(helix_steel_plugin_manifest_error)
}

pub(super) fn materialize_steel_config(
    runtime_dir: &Path,
    config_dir: &Path,
    generated_dir: &Path,
    selection: &SteelPluginConfig,
    show_splash: bool,
) -> Result<SteelMaterializationData, CoreError> {
    let repository = load_bundled_steel_plugin_repository(runtime_dir)?;
    let active_plugins = active_steel_plugins(&repository, selection)?;
    validate_active_steel_plugins(&active_plugins)?;
    let mut copied_plugin_files = Vec::new();
    let mut copied_relative_paths: BTreeMap<String, PathBuf> = BTreeMap::new();

    for plugin in &active_plugins {
        for relative_path in std::iter::once(&plugin.manifest.source_relative_path)
            .chain(plugin.manifest.support_files.iter())
        {
            let source = resolve_active_steel_plugin_source(
                runtime_dir,
                config_dir,
                plugin.source,
                relative_path,
                &plugin.manifest.id,
            )?;
            if let Some(previous_source) = copied_relative_paths.get(relative_path) {
                if previous_source != &source {
                    return Err(CoreError::classified(
                        ErrorClass::Config,
                        "duplicate_helix_steel_plugin_file",
                        format!(
                            "Helix Steel plugin `{}` wants to copy `{relative_path}` from a different source.",
                            plugin.manifest.id
                        ),
                        "Use unique generated Steel file paths across bundled and custom plugins.",
                        serde_json::json!({
                            "plugin_id": &plugin.manifest.id,
                            "relative_path": relative_path,
                            "previous_source": previous_source.to_string_lossy(),
                            "source": source.to_string_lossy(),
                        }),
                    ));
                }
                continue;
            }
            let target = generated_dir.join(relative_path);
            copy_steel_plugin_file(&source, &target)?;
            copied_relative_paths.insert(relative_path.clone(), source);
            copied_plugin_files.push(target.to_string_lossy().into_owned());
        }
    }

    let helix_module_path = generated_dir.join(STEEL_CONFIG_MODULE);
    let init_path = generated_dir.join(STEEL_INIT_MODULE);
    write_text_atomic(
        &helix_module_path,
        &render_steel_helix_module(&active_plugins, show_splash),
    )?;
    write_text_atomic(&init_path, &render_steel_init_module())?;

    Ok(SteelMaterializationData {
        config_dir: generated_dir.to_path_buf(),
        helix_module_path,
        init_path,
        enabled_plugins: active_plugins
            .iter()
            .map(|plugin| plugin.manifest.id.clone())
            .collect(),
        copied_plugin_files,
        commands: active_steel_commands(&active_plugins),
    })
}

fn helix_steel_plugin_manifest_error(error: SteelPluginManifestError) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        "invalid_helix_steel_plugin_manifest",
        format!(
            "Invalid Helix Steel plugin manifest at {}: {}",
            error.path, error.message
        ),
        "Fix helix.steel_plugins in ~/.config/yazelix/settings.jsonc and retry.",
        serde_json::json!({
            "path": error.path,
            "message": error.message,
        }),
    )
}

fn load_bundled_steel_plugin_repository(
    runtime_dir: &Path,
) -> Result<Vec<SteelPluginManifest>, CoreError> {
    let manifest_path = runtime_dir
        .join("configs")
        .join("helix")
        .join(STEEL_PLUGIN_ROOT)
        .join(STEEL_PLUGIN_MANIFEST);
    let raw = fs::read_to_string(&manifest_path).map_err(|source| {
        CoreError::io(
            "read_helix_steel_plugin_manifest",
            "Could not read the bundled Helix Steel plugin repository manifest",
            "Reinstall Yazelix so the runtime includes configs/helix/steel_plugins/manifest.toml.",
            manifest_path.to_string_lossy(),
            source,
        )
    })?;
    let manifest: TomlValue = toml::from_str(&raw).map_err(|source| {
        CoreError::toml(
            "parse_helix_steel_plugin_manifest",
            "Could not parse the bundled Helix Steel plugin repository manifest",
            "Reinstall Yazelix so the runtime includes a valid Helix Steel plugin manifest.",
            manifest_path.to_string_lossy(),
            source,
        )
    })?;
    let manifest = serde_json::to_value(manifest).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_helix_steel_plugin_manifest",
            format!("Could not convert the Helix Steel plugin manifest to JSON: {source}"),
            "This is an internal error. File a bug report if it persists.",
            serde_json::json!({ "manifest_path": manifest_path.to_string_lossy() }),
        )
    })?;
    parse_steel_plugin_manifest_array(
        manifest.get("plugins"),
        "configs.helix.steel_plugins.manifest.plugins",
    )
    .map_err(helix_steel_plugin_manifest_error)
}

fn active_steel_plugins(
    repository: &[SteelPluginManifest],
    selection: &SteelPluginConfig,
) -> Result<Vec<ActiveSteelPlugin>, CoreError> {
    let mut plugins = Vec::new();
    for id in &selection.enabled {
        let Some(manifest) = repository.iter().find(|plugin| plugin.id == *id) else {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unknown_helix_steel_plugin_id",
                format!("Helix Steel plugin `{id}` is not in the bundled plugin repository."),
                "Remove the id from helix.steel_plugins.enabled, or add it as helix.steel_plugins.extra with a source file.",
                serde_json::json!({
                    "plugin_id": id,
                    "available_plugins": repository.iter().map(|plugin| plugin.id.as_str()).collect::<Vec<_>>(),
                }),
            ));
        };
        plugins.push(ActiveSteelPlugin {
            manifest: manifest.clone(),
            source: SteelPluginSource::Bundled,
        });
    }
    plugins.extend(
        selection
            .extra
            .iter()
            .cloned()
            .map(|manifest| ActiveSteelPlugin {
                manifest,
                source: SteelPluginSource::User,
            }),
    );
    Ok(plugins)
}

fn validate_active_steel_plugins(active_plugins: &[ActiveSteelPlugin]) -> Result<(), CoreError> {
    let mut plugin_ids = BTreeSet::new();
    for plugin in active_plugins {
        if !plugin_ids.insert(plugin.manifest.id.clone()) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "duplicate_helix_steel_plugin_id",
                format!(
                    "Helix Steel plugin id `{}` conflicts with another plugin.",
                    plugin.manifest.id
                ),
                "Choose a unique id in helix.steel_plugins and retry.",
                serde_json::json!({ "plugin_id": &plugin.manifest.id }),
            ));
        }
    }

    let mut source_paths = BTreeSet::new();
    for plugin in active_plugins {
        if !source_paths.insert(plugin.manifest.source_relative_path.clone()) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "duplicate_helix_steel_plugin_source",
                format!(
                    "Helix Steel plugin source `{}` is already loaded by another active plugin.",
                    plugin.manifest.source_relative_path
                ),
                "Use a distinct source path for each active Helix Steel plugin and retry.",
                serde_json::json!({
                    "plugin_id": &plugin.manifest.id,
                    "source_relative_path": &plugin.manifest.source_relative_path,
                }),
            ));
        }
    }

    let mut command_names = BTreeSet::new();
    for command in active_steel_command_specs(active_plugins) {
        if !command_names.insert(command.name.clone()) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "duplicate_helix_steel_plugin_command",
                format!(
                    "Helix Steel command `{}` is declared more than once.",
                    command.name
                ),
                "Keep every public and internal Helix Steel command name unique across built-in and custom plugins.",
                serde_json::json!({ "command": command.name }),
            ));
        }
    }

    Ok(())
}

fn active_steel_command_specs(active_plugins: &[ActiveSteelPlugin]) -> Vec<ActiveSteelCommand> {
    let mut commands = BASE_STEEL_COMMANDS
        .iter()
        .copied()
        .map(SteelCommandSpec::active)
        .collect::<Vec<_>>();
    for plugin in active_plugins {
        commands.extend(plugin.manifest.public_commands.iter().map(|command| {
            active_plugin_steel_command(plugin, command, SteelCommandVisibility::Public)
        }));
        commands.extend(plugin.manifest.internal_commands.iter().map(|command| {
            active_plugin_steel_command(plugin, command, SteelCommandVisibility::Internal)
        }));
    }
    commands
}

fn active_plugin_steel_command(
    plugin: &ActiveSteelPlugin,
    command: &SteelPluginManifestCommand,
    visibility: SteelCommandVisibility,
) -> ActiveSteelCommand {
    ActiveSteelCommand {
        name: command.name.clone(),
        owner: plugin.manifest.id.clone(),
        description: command.description.clone(),
        visibility,
    }
}

fn active_steel_commands(active_plugins: &[ActiveSteelPlugin]) -> Vec<SteelCommandMetadata> {
    active_steel_command_specs(active_plugins)
        .into_iter()
        .map(ActiveSteelCommand::metadata)
        .collect()
}

fn resolve_active_steel_plugin_source(
    runtime_dir: &Path,
    config_dir: &Path,
    source_kind: SteelPluginSource,
    relative_path: &str,
    plugin_id: &str,
) -> Result<PathBuf, CoreError> {
    let source = match source_kind {
        SteelPluginSource::Bundled => runtime_dir
            .join("configs")
            .join("helix")
            .join(STEEL_PLUGIN_ROOT)
            .join(relative_path),
        SteelPluginSource::User => config_dir
            .join("helix")
            .join(STEEL_PLUGIN_ROOT)
            .join(relative_path),
    };
    if source.is_file() {
        return Ok(source);
    }

    let (code, remediation) = match source_kind {
        SteelPluginSource::Bundled => (
            "missing_helix_steel_plugin_repository_source",
            "Reinstall Yazelix so the runtime includes all files declared by configs/helix/steel_plugins/manifest.toml.",
        ),
        SteelPluginSource::User => (
            "missing_helix_steel_plugin_manifest_source",
            "Create the source below ~/.config/yazelix/helix/steel_plugins, or remove the manifest from helix.steel_plugins.extra.",
        ),
    };
    Err(CoreError::classified(
        ErrorClass::Config,
        code,
        format!("Helix Steel plugin `{plugin_id}` declares missing source `{relative_path}`."),
        remediation,
        serde_json::json!({
            "plugin_id": plugin_id,
            "source_relative_path": relative_path,
            "source": source.to_string_lossy(),
        }),
    ))
}

fn copy_steel_plugin_file(source: &Path, target: &Path) -> Result<(), CoreError> {
    let bytes = fs::read(source).map_err(|source_error| {
        CoreError::io(
            "read_helix_steel_plugin",
            "Could not read a managed Helix Steel plugin source file",
            "Check permissions for the source plugin file, or reinstall Yazelix if it is runtime-owned.",
            source.to_string_lossy(),
            source_error,
        )
    })?;
    write_bytes_atomic(target, &bytes)
}

fn render_steel_helix_module(active_plugins: &[ActiveSteelPlugin], show_splash: bool) -> String {
    let commands = active_steel_command_specs(active_plugins);
    let public_commands = commands
        .iter()
        .filter(|command| command.is_public())
        .collect::<Vec<_>>();
    let provide_line = format!(
        "(provide {})",
        public_commands
            .iter()
            .map(|command| command.name.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut lines = vec![
        ";; Yazelix-managed Helix Steel command module.".to_string(),
        ";; Generated at launch from settings.jsonc.".to_string(),
        "".to_string(),
        ";; Public commands:".to_string(),
    ];
    for command in &public_commands {
        lines.push(format!(
            ";; - {} [{}]: {}",
            command.name, command.owner, command.description
        ));
    }
    lines.extend([
        "".to_string(),
        "(require (only-in \"helix/ext.scm\" eval-buffer evalp))".to_string(),
        "(require (only-in \"helix/static.scm\" cx->current-file get-helix-cwd))".to_string(),
        "(require (only-in \"helix/commands.scm\" run-shell-command))".to_string(),
        "(require (only-in \"helix/misc.scm\" set-error!))".to_string(),
        "".to_string(),
    ]);
    lines.push(provide_line);
    lines.extend([
        "".to_string(),
        "(define (yazelix-posix-quote value)".to_string(),
        "  (string-append \"'\" (string-replace value \"'\" \"'\\\\''\") \"'\"))".to_string(),
        "".to_string(),
        "(define (yzx-new-shell-command target)".to_string(),
        "  (string-append \"\\\"$YAZELIX_RUNTIME_DIR/libexec/yzx_control\\\" zellij open-terminal \" (yazelix-posix-quote target)))".to_string(),
        "".to_string(),
        ";;@doc".to_string(),
        ";;Open a Yazelix terminal pane at the current Helix file or workspace.".to_string(),
        "(define (yzx-new-shell)".to_string(),
        "  (let ([current-file (cx->current-file)]".to_string(),
        "        [current-workspace (get-helix-cwd)])".to_string(),
        "    (cond".to_string(),
        "      [(string? current-file)".to_string(),
        "       (run-shell-command (yzx-new-shell-command current-file))]".to_string(),
        "      [(string? current-workspace)".to_string(),
        "       (run-shell-command (yzx-new-shell-command current-workspace))]".to_string(),
        "      [else".to_string(),
        "       (set-error! \"Yazelix could not resolve a target path for opening a shell\")])))"
            .to_string(),
        "".to_string(),
    ]);
    for plugin in active_plugins {
        lines.push(";;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;".to_string());
        lines.push("".to_string());
        lines.extend(render_steel_plugin_module_lines(
            &plugin.manifest,
            show_splash,
        ));
        lines.push("".to_string());
    }
    lines.join("\n") + "\n"
}

fn render_steel_plugin_module_lines(spec: &SteelPluginManifest, show_splash: bool) -> Vec<String> {
    let command_names = spec.command_names();
    let mut lines = if command_names.is_empty() {
        vec![format!("(require \"{}\")", spec.source_relative_path)]
    } else {
        vec![format!(
            "(require (only-in \"{}\" {}))",
            spec.source_relative_path,
            command_names.join(" ")
        )]
    };
    if spec.startup_condition_matches(show_splash) {
        lines.extend(
            spec.startup_commands
                .iter()
                .map(|command| format!("({command})")),
        );
    }
    lines
}

fn render_steel_init_module() -> String {
    ";; Yazelix-managed Helix Steel init file.\n;; Generated at launch from settings.jsonc.\n\n"
        .to_string()
}
