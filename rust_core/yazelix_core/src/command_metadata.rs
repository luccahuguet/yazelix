use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxCommandCategory {
    Config,
    Development,
    Help,
    Integration,
    Session,
    System,
    Workspace,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxParameterKind {
    Switch,
    Named,
    Positional,
    Rest,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct YzxCommandParameter {
    pub kind: YzxParameterKind,
    pub name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<&'static str>,
    pub shape: &'static str,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct YzxCommandMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub category: YzxCommandCategory,
    pub parameters: &'static [YzxCommandParameter],
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct YzxCommandMetadataData {
    pub commands: Vec<YzxCommandMetadata>,
    pub extern_content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YzxExternBridgeSyncRequest {
    pub runtime_dir: PathBuf,
    pub state_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct YzxExternBridgeSyncData {
    pub extern_path: String,
    pub fingerprint_path: String,
    pub status: YzxExternBridgeSyncStatus,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxExternBridgeSyncStatus {
    Reused,
    Updated,
}

#[derive(Debug, Deserialize)]
struct YzxExternBridgeState {
    schema_version: u8,
    source_fingerprint: String,
    extern_hash: String,
}

#[derive(Debug, Serialize)]
struct YzxExternSourceFingerprint {
    schema_version: u8,
    renderer_version: &'static str,
    runtime_dir: String,
    runtime_marker: YzxExternFileFingerprint,
    yzx_core: YzxExternFileFingerprint,
}

#[derive(Debug, Serialize)]
struct YzxExternFileFingerprint {
    path: String,
    exists: bool,
    size: Option<u64>,
    modified: Option<String>,
}

const YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION: u8 = 3;
const YZX_EXTERN_BRIDGE_RENDERER_VERSION: &str = "v3-rust-sync";
const YZX_EXTERN_BRIDGE_PLACEHOLDER: &str = "# Yazelix generated Nushell extern bridge (empty)\n";

const VERSION_FLAGS: &[YzxCommandParameter] = &[
    switch("version", Some("V")),
    switch("version-short", Some("v")),
];
const ENV_FLAGS: &[YzxCommandParameter] = &[switch("no-shell", Some("n"))];
const RUN_REST: &[YzxCommandParameter] = &[rest("argv")];
const LAUNCH_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    switch("home", None),
    named("terminal", Some("t"), "string", true),
    switch("verbose", None),
];
const ENTER_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    switch("home", None),
    switch("verbose", None),
];
const UPDATE_NIX_FLAGS: &[YzxCommandParameter] = &[switch("yes", None), switch("verbose", None)];
const CWD_ARGS: &[YzxCommandParameter] = &[positional("target", "string", true)];
const REVEAL_ARGS: &[YzxCommandParameter] = &[positional("target", "string", false)];
const STATUS_FLAGS: &[YzxCommandParameter] = &[switch("versions", Some("V")), switch("json", None)];
const DOCTOR_FLAGS: &[YzxCommandParameter] = &[
    switch("verbose", Some("v")),
    switch("fix", Some("f")),
    switch("json", None),
];
const CONFIG_RESET_FLAGS: &[YzxCommandParameter] = &[switch("force", None)];
const IMPORT_FLAGS: &[YzxCommandParameter] = &[switch("force", None)];
const EDIT_ARGS: &[YzxCommandParameter] = &[rest("query"), switch("print", None)];
const EDIT_CONFIG_FLAGS: &[YzxCommandParameter] = &[switch("print", None)];
const POPUP_ARGS: &[YzxCommandParameter] = &[rest("program")];
const SCREEN_ARGS: &[YzxCommandParameter] = &[positional("style", "string", true)];
const DEV_UPDATE_FLAGS: &[YzxCommandParameter] = &[
    switch("yes", None),
    switch("no-canary", None),
    named("activate", None, "string", true),
    named("home-manager-dir", None, "string", true),
    named("home-manager-input", None, "string", true),
    named("home-manager-attr", None, "string", true),
    switch("canary-only", None),
    named("canaries", None, "string", true),
];
const DEV_BUMP_ARGS: &[YzxCommandParameter] = &[positional("version", "string", false)];
const DEV_SYNC_FLAGS: &[YzxCommandParameter] = &[switch("dry-run", None)];
const DEV_BUILD_FLAGS: &[YzxCommandParameter] = &[switch("sync", None)];
const DEV_TEST_FLAGS: &[YzxCommandParameter] = &[
    switch("verbose", Some("v")),
    switch("new-window", Some("n")),
    switch("lint-only", None),
    switch("profile", None),
    switch("sweep", None),
    switch("visual", None),
    switch("all", Some("a")),
    named("delay", None, "int", true),
];
const DEV_PROFILE_FLAGS: &[YzxCommandParameter] = &[
    switch("cold", Some("c")),
    switch("desktop", None),
    switch("launch", None),
    switch("clear-cache", None),
    named("terminal", Some("t"), "string", true),
    switch("verbose", None),
];
const DEV_LINT_ARGS: &[YzxCommandParameter] =
    &[named("format", Some("f"), "string", true), rest("paths")];
const HM_PREPARE_FLAGS: &[YzxCommandParameter] = &[switch("apply", None), switch("yes", None)];

pub fn yzx_command_metadata() -> Vec<YzxCommandMetadata> {
    let mut commands = vec![
        cmd(
            "yzx",
            "Show Yazelix help or version information",
            YzxCommandCategory::Help,
            VERSION_FLAGS,
        ),
        cmd(
            "yzx config",
            "Show the active Yazelix configuration",
            YzxCommandCategory::Config,
            &[],
        ),
        cmd(
            "yzx config reset",
            "Replace the main Yazelix config with a fresh shipped template",
            YzxCommandCategory::Config,
            CONFIG_RESET_FLAGS,
        ),
        cmd(
            "yzx cwd",
            "Retarget the current Yazelix tab workspace directory",
            YzxCommandCategory::Workspace,
            CWD_ARGS,
        ),
        cmd(
            "yzx desktop install",
            "Install the user-local Yazelix desktop entry and icons",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx desktop launch",
            "Launch Yazelix from the desktop entry fast path",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx desktop uninstall",
            "Remove the user-local Yazelix desktop entry and icons",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx dev",
            "Development and maintainer commands",
            YzxCommandCategory::Development,
            &[],
        ),
        cmd(
            "yzx dev build_pane_orchestrator",
            "Build the Zellij pane-orchestrator wasm",
            YzxCommandCategory::Development,
            DEV_BUILD_FLAGS,
        ),
        cmd(
            "yzx dev bump",
            "Bump the tracked Yazelix version and create release metadata",
            YzxCommandCategory::Development,
            DEV_BUMP_ARGS,
        ),
        cmd(
            "yzx dev lint_nu",
            "Lint Nushell scripts with repo-tuned nu-lint config",
            YzxCommandCategory::Development,
            DEV_LINT_ARGS,
        ),
        cmd(
            "yzx dev profile",
            "Profile launch sequence and identify bottlenecks",
            YzxCommandCategory::Development,
            DEV_PROFILE_FLAGS,
        ),
        cmd(
            "yzx dev sync_issues",
            "Sync GitHub issue lifecycle into Beads locally",
            YzxCommandCategory::Development,
            DEV_SYNC_FLAGS,
        ),
        cmd(
            "yzx dev test",
            "Run Yazelix test suite",
            YzxCommandCategory::Development,
            DEV_TEST_FLAGS,
        ),
        cmd(
            "yzx dev update",
            "Refresh maintainer flake inputs and run update canaries",
            YzxCommandCategory::Development,
            DEV_UPDATE_FLAGS,
        ),
        cmd(
            "yzx doctor",
            "Run health checks and diagnostics",
            YzxCommandCategory::System,
            DOCTOR_FLAGS,
        ),
        cmd(
            "yzx edit",
            "Open a Yazelix-managed config surface in the configured editor",
            YzxCommandCategory::Config,
            EDIT_ARGS,
        ),
        cmd(
            "yzx edit config",
            "Open the main Yazelix config in the configured editor",
            YzxCommandCategory::Config,
            EDIT_CONFIG_FLAGS,
        ),
        cmd(
            "yzx enter",
            "Start Yazelix in the current terminal",
            YzxCommandCategory::Session,
            ENTER_FLAGS,
        ),
        cmd(
            "yzx env",
            "Load the Yazelix environment without UI",
            YzxCommandCategory::Session,
            ENV_FLAGS,
        ),
        cmd(
            "yzx home_manager",
            "Show Yazelix Home Manager takeover helpers",
            YzxCommandCategory::Integration,
            &[],
        ),
        cmd(
            "yzx home_manager prepare",
            "Preview or archive manual-install artifacts before Home Manager takeover",
            YzxCommandCategory::Integration,
            HM_PREPARE_FLAGS,
        ),
        cmd(
            "yzx import",
            "Import native config files into Yazelix-managed override paths",
            YzxCommandCategory::Config,
            &[],
        ),
        cmd(
            "yzx import helix",
            "Import the native Helix config into Yazelix-managed overrides",
            YzxCommandCategory::Config,
            IMPORT_FLAGS,
        ),
        cmd(
            "yzx import yazi",
            "Import native Yazi config files into Yazelix-managed overrides",
            YzxCommandCategory::Config,
            IMPORT_FLAGS,
        ),
        cmd(
            "yzx import zellij",
            "Import the native Zellij config into Yazelix-managed overrides",
            YzxCommandCategory::Config,
            IMPORT_FLAGS,
        ),
        cmd(
            "yzx keys",
            "Show Yazelix-owned keybindings and remaps",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys helix",
            "Alias for yzx keys hx",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys hx",
            "Explain how to discover Helix keybindings and commands",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys nu",
            "Show a small curated subset of useful Nushell keybindings",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys nushell",
            "Alias for yzx keys nu",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys yazi",
            "Explain how to view Yazi's built-in keybindings",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx keys yzx",
            "Alias for the default Yazelix keybinding view",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx launch",
            "Launch Yazelix",
            YzxCommandCategory::Session,
            LAUNCH_FLAGS,
        ),
        cmd(
            "yzx menu",
            "Interactive command palette for Yazelix",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx popup",
            "Open or toggle the configured Yazelix popup program in Zellij",
            YzxCommandCategory::Workspace,
            POPUP_ARGS,
        ),
        cmd(
            "yzx restart",
            "Restart Yazelix",
            YzxCommandCategory::Session,
            &[],
        ),
        cmd(
            "yzx reveal",
            "Reveal a file or directory in the managed Yazi sidebar",
            YzxCommandCategory::Workspace,
            REVEAL_ARGS,
        ),
        cmd(
            "yzx run",
            "Run a command in the Yazelix environment and exit",
            YzxCommandCategory::Session,
            RUN_REST,
        ),
        cmd(
            "yzx screen",
            "Show an animated Yazelix full-terminal screen",
            YzxCommandCategory::Workspace,
            SCREEN_ARGS,
        ),
        cmd(
            "yzx sponsor",
            "Open the Yazelix sponsor page or print its URL",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx status",
            "Canonical inspection command",
            YzxCommandCategory::System,
            STATUS_FLAGS,
        ),
        cmd(
            "yzx tutor",
            "Show the Yazelix guided overview",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor helix",
            "Alias for yzx tutor hx",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor hx",
            "Launch Helix's built-in tutorial",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor nu",
            "Launch Nushell's built-in tutorial in a fresh Nushell process",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx tutor nushell",
            "Alias for yzx tutor nu",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx update",
            "Show supported update owners",
            YzxCommandCategory::System,
            &[],
        ),
        cmd(
            "yzx update home_manager",
            "Refresh the current Home Manager flake input for Yazelix",
            YzxCommandCategory::System,
            &[],
        ),
        cmd(
            "yzx update nix",
            "Upgrade Determinate Nix through determinate-nixd",
            YzxCommandCategory::System,
            UPDATE_NIX_FLAGS,
        ),
        cmd(
            "yzx update upstream",
            "Upgrade the active Yazelix package in the default Nix profile",
            YzxCommandCategory::System,
            &[],
        ),
        cmd(
            "yzx whats_new",
            "Show the current Yazelix upgrade summary",
            YzxCommandCategory::Help,
            &[],
        ),
        cmd(
            "yzx why",
            "Elevator pitch: Why Yazelix",
            YzxCommandCategory::Help,
            &[],
        ),
    ];
    commands.sort_by(|left, right| left.name.cmp(right.name));
    commands
}

pub fn yzx_command_metadata_data() -> YzxCommandMetadataData {
    let commands = yzx_command_metadata();
    let extern_content = render_yzx_externs(&commands);
    YzxCommandMetadataData {
        commands,
        extern_content,
    }
}

pub fn sync_yzx_extern_bridge(
    request: &YzxExternBridgeSyncRequest,
) -> Result<YzxExternBridgeSyncData, CoreError> {
    let extern_path = generated_yzx_extern_path(&request.state_dir);
    let fingerprint_path = generated_yzx_extern_fingerprint_path(&request.state_dir);
    let source_fingerprint = compute_yzx_extern_source_fingerprint(&request.runtime_dir)?;

    if yzx_extern_bridge_is_current(&extern_path, &fingerprint_path, &source_fingerprint)? {
        return Ok(YzxExternBridgeSyncData {
            extern_path: path_string(&extern_path),
            fingerprint_path: path_string(&fingerprint_path),
            status: YzxExternBridgeSyncStatus::Reused,
        });
    }

    let has_existing_bridge = extern_path.exists();
    if !has_existing_bridge {
        write_text_atomic(&extern_path, YZX_EXTERN_BRIDGE_PLACEHOLDER)?;
    }

    ensure_valid_runtime_dir(&request.runtime_dir)?;

    let extern_content = yzx_command_metadata_data().extern_content;
    if extern_content.trim().is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Internal,
            "empty_yzx_extern_bridge",
            "Rust yzx command metadata produced an empty Nushell extern bridge.",
            "Report this as a Yazelix internal error.",
            json!({}),
        ));
    }

    write_text_atomic(&extern_path, &extern_content)?;
    write_yzx_extern_bridge_state(&fingerprint_path, &source_fingerprint, &extern_content)?;

    Ok(YzxExternBridgeSyncData {
        extern_path: path_string(&extern_path),
        fingerprint_path: path_string(&fingerprint_path),
        status: YzxExternBridgeSyncStatus::Updated,
    })
}

pub fn render_yzx_help(commands: &[YzxCommandMetadata]) -> String {
    let width = commands
        .iter()
        .map(|command| command.name.len())
        .max()
        .unwrap_or(3);
    let rows = commands
        .iter()
        .map(|command| {
            format!(
                "  {:width$}  {}",
                command.name,
                command.description,
                width = width
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    [
        "Show Yazelix help or version information".to_string(),
        String::new(),
        "Usage:".to_string(),
        "  yzx [--version]".to_string(),
        "  yzx <command> [args...]".to_string(),
        String::new(),
        "Commands:".to_string(),
        rows,
        String::new(),
        "Flags:".to_string(),
        "  -h, --help           Display help for this command".to_string(),
        "  -V, --version        Show Yazelix version".to_string(),
        "  -v, --version-short  Show Yazelix version".to_string(),
    ]
    .join("\n")
}

pub fn render_yzx_externs(commands: &[YzxCommandMetadata]) -> String {
    let header = [
        "# Generated by Yazelix from Rust-owned yzx command metadata.",
        "# Restores Nushell completion/signature knowledge for the external yzx CLI.",
        "",
    ]
    .join("\n");
    let body = commands
        .iter()
        .map(render_extern_block)
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{header}{body}\n")
}

const fn switch(name: &'static str, short: Option<&'static str>) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Switch,
        name,
        short,
        shape: "string",
        optional: true,
    }
}

const fn named(
    name: &'static str,
    short: Option<&'static str>,
    shape: &'static str,
    optional: bool,
) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Named,
        name,
        short,
        shape,
        optional,
    }
}

const fn positional(
    name: &'static str,
    shape: &'static str,
    optional: bool,
) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Positional,
        name,
        short: None,
        shape,
        optional,
    }
}

const fn rest(name: &'static str) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Rest,
        name,
        short: None,
        shape: "string",
        optional: true,
    }
}

const fn cmd(
    name: &'static str,
    description: &'static str,
    category: YzxCommandCategory,
    parameters: &'static [YzxCommandParameter],
) -> YzxCommandMetadata {
    YzxCommandMetadata {
        name,
        description,
        category,
        parameters,
    }
}

fn render_extern_block(command: &YzxCommandMetadata) -> String {
    if command.parameters.is_empty() {
        return format!("export extern \"{}\" []", command.name);
    }

    let parameters = command
        .parameters
        .iter()
        .map(render_parameter)
        .collect::<Vec<_>>()
        .join("\n");
    format!("export extern \"{}\" [\n{}\n]", command.name, parameters)
}

fn render_parameter(parameter: &YzxCommandParameter) -> String {
    match parameter.kind {
        YzxParameterKind::Switch => render_flag(parameter),
        YzxParameterKind::Named => render_named(parameter),
        YzxParameterKind::Positional => render_positional(parameter),
        YzxParameterKind::Rest => format!("    ...{}: {}", parameter.name, parameter.shape),
    }
}

fn render_flag(parameter: &YzxCommandParameter) -> String {
    match parameter.short {
        Some(short) => format!("    --{}(-{})", parameter.name, short),
        None => format!("    --{}", parameter.name),
    }
}

fn render_named(parameter: &YzxCommandParameter) -> String {
    match parameter.short {
        Some(short) => format!("    --{}(-{}): {}", parameter.name, short, parameter.shape),
        None => format!("    --{}: {}", parameter.name, parameter.shape),
    }
}

fn render_positional(parameter: &YzxCommandParameter) -> String {
    if parameter.optional {
        format!("    {}?: {}", parameter.name, parameter.shape)
    } else {
        format!("    {}: {}", parameter.name, parameter.shape)
    }
}

fn generated_yzx_extern_path(state_dir: &Path) -> PathBuf {
    state_dir
        .join("initializers")
        .join("nushell")
        .join("yazelix_extern.nu")
}

fn generated_yzx_extern_fingerprint_path(state_dir: &Path) -> PathBuf {
    generated_yzx_extern_path(state_dir)
        .parent()
        .expect("generated extern path has a parent")
        .join("yazelix_extern.fingerprint.json")
}

fn ensure_valid_runtime_dir(runtime_dir: &Path) -> Result<(), CoreError> {
    let sentinel = runtime_dir.join("yazelix_default.toml");
    if sentinel.is_file() {
        return Ok(());
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "invalid_runtime_dir",
        format!(
            "Yazelix runtime directory is missing its default config sentinel at {}.",
            sentinel.display()
        ),
        "Reinstall Yazelix or run from a valid source checkout.",
        json!({ "runtime_dir": path_string(runtime_dir), "sentinel": path_string(&sentinel) }),
    ))
}

fn compute_yzx_extern_source_fingerprint(runtime_dir: &Path) -> Result<String, CoreError> {
    let runtime_marker = runtime_dir.join("yazelix_default.toml");
    let yzx_core = std::env::current_exe().map_err(|source| {
        CoreError::io(
            "resolve_yzx_core_exe",
            "Could not resolve the running yzx_core helper path",
            "Retry the command and report this as a Yazelix internal error if it persists.",
            "<current_exe>",
            source,
        )
    })?;
    let payload = YzxExternSourceFingerprint {
        schema_version: YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION,
        renderer_version: YZX_EXTERN_BRIDGE_RENDERER_VERSION,
        runtime_dir: path_string(runtime_dir),
        runtime_marker: fingerprint_file(&runtime_marker),
        yzx_core: fingerprint_file(&yzx_core),
    };
    let serialized = serde_json::to_vec(&payload).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_yzx_extern_source_fingerprint",
            format!("Could not serialize yzx extern bridge source fingerprint: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    Ok(sha256_hex(&serialized))
}

fn fingerprint_file(path: &Path) -> YzxExternFileFingerprint {
    let Ok(metadata) = fs::metadata(path) else {
        return YzxExternFileFingerprint {
            path: path_string(path),
            exists: false,
            size: None,
            modified: None,
        };
    };

    YzxExternFileFingerprint {
        path: path_string(path),
        exists: true,
        size: Some(metadata.len()),
        modified: metadata.modified().ok().and_then(system_time_string),
    }
}

fn yzx_extern_bridge_is_current(
    extern_path: &Path,
    fingerprint_path: &Path,
    source_fingerprint: &str,
) -> Result<bool, CoreError> {
    let Some(state) = read_yzx_extern_bridge_state(fingerprint_path)? else {
        return Ok(false);
    };
    let Some(extern_hash) = hash_file_contents(extern_path)? else {
        return Ok(false);
    };

    Ok(
        state.schema_version == YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION
            && state.source_fingerprint == source_fingerprint
            && state.extern_hash == extern_hash,
    )
}

fn read_yzx_extern_bridge_state(path: &Path) -> Result<Option<YzxExternBridgeState>, CoreError> {
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(CoreError::io(
                "read_yzx_extern_bridge_state",
                "Could not read the generated yzx extern bridge fingerprint",
                "Check permissions for the Yazelix state directory and retry.",
                path.to_string_lossy(),
                source,
            ));
        }
    };

    Ok(serde_json::from_str(&raw).ok())
}

fn hash_file_contents(path: &Path) -> Result<Option<String>, CoreError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(sha256_hex(&bytes))),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(CoreError::io(
            "read_yzx_extern_bridge",
            "Could not read the generated yzx extern bridge",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )),
    }
}

fn write_yzx_extern_bridge_state(
    fingerprint_path: &Path,
    source_fingerprint: &str,
    extern_content: &str,
) -> Result<(), CoreError> {
    let state = serde_json::json!({
        "schema_version": YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION,
        "source_fingerprint": source_fingerprint,
        "extern_hash": sha256_hex(extern_content.as_bytes()),
    });
    let serialized = serde_json::to_string(&state).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_yzx_extern_bridge_state",
            format!("Could not serialize yzx extern bridge fingerprint: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    write_text_atomic(fingerprint_path, &format!("{serialized}\n"))
}

fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_yzx_extern_bridge_path",
            "Generated yzx extern bridge path has no parent directory.",
            "Report this as a Yazelix internal error.",
            json!({ "path": path_string(path) }),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "create_yzx_extern_bridge_parent",
            "Could not create parent directory for the generated yzx extern bridge",
            "Check permissions for the Yazelix state directory and retry.",
            parent.to_string_lossy(),
            source,
        )
    })?;

    let temporary_path = path.with_file_name(format!(
        ".{}.yazelix-tmp-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("yazelix_extern"),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    fs::write(&temporary_path, content).map_err(|source| {
        CoreError::io(
            "write_yzx_extern_bridge_temp",
            "Could not write temporary generated yzx extern bridge",
            "Check permissions for the Yazelix state directory and retry.",
            temporary_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|source| {
        CoreError::io(
            "rename_yzx_extern_bridge_temp",
            "Could not replace generated yzx extern bridge",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

fn sha256_hex(input: &[u8]) -> String {
    let digest = Sha256::digest(input);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn system_time_string(value: SystemTime) -> Option<String> {
    value
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_nanos().to_string())
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Defends: Rust metadata is the public source for migrated control-plane leaves that no longer live in the Nushell command tree.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn metadata_includes_rust_owned_control_plane_commands() {
        let names = yzx_command_metadata()
            .into_iter()
            .map(|command| command.name)
            .collect::<Vec<_>>();
        assert!(names.contains(&"yzx env"));
        assert!(names.contains(&"yzx run"));
        assert!(names.contains(&"yzx update"));
        assert!(names.contains(&"yzx update nix"));
    }

    // Defends: generated Nushell externs come from Rust metadata, including Rust-only leaves exactly once.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_externs_for_rust_only_leaves_once() {
        let data = yzx_command_metadata_data();
        assert_eq!(
            data.extern_content
                .matches("export extern \"yzx env\"")
                .count(),
            1
        );
        assert_eq!(
            data.extern_content
                .matches("export extern \"yzx run\"")
                .count(),
            1
        );
        assert!(data.extern_content.contains("--no-shell(-n)"));
        assert!(data.extern_content.contains("...argv: string"));
    }

    // Regression: the generated yzx extern bridge lifecycle is Rust-owned and reuses current fingerprints without a Nushell wrapper.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn sync_yzx_extern_bridge_writes_and_reuses_current_bridge() {
        let runtime = TempDir::new().unwrap();
        let state = TempDir::new().unwrap();
        fs::write(runtime.path().join("yazelix_default.toml"), "").unwrap();

        let request = YzxExternBridgeSyncRequest {
            runtime_dir: runtime.path().to_path_buf(),
            state_dir: state.path().to_path_buf(),
        };

        let first = sync_yzx_extern_bridge(&request).unwrap();
        let second = sync_yzx_extern_bridge(&request).unwrap();
        let extern_content = fs::read_to_string(first.extern_path).unwrap();

        assert_eq!(first.status, YzxExternBridgeSyncStatus::Updated);
        assert_eq!(second.status, YzxExternBridgeSyncStatus::Reused);
        assert!(extern_content.contains("export extern \"yzx env\""));
        assert!(Path::new(&second.fingerprint_path).is_file());
    }

    // Regression: failed generated yzx extern refreshes must not replace a previous valid bridge with the placeholder.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn sync_yzx_extern_bridge_preserves_existing_bridge_on_invalid_runtime() {
        let runtime = TempDir::new().unwrap();
        let invalid_runtime = TempDir::new().unwrap();
        let state = TempDir::new().unwrap();
        fs::write(runtime.path().join("yazelix_default.toml"), "").unwrap();

        let request = YzxExternBridgeSyncRequest {
            runtime_dir: runtime.path().to_path_buf(),
            state_dir: state.path().to_path_buf(),
        };
        let data = sync_yzx_extern_bridge(&request).unwrap();
        let generated_content = fs::read_to_string(&data.extern_path).unwrap();
        fs::write(&data.fingerprint_path, "stale fingerprint").unwrap();

        let failed = sync_yzx_extern_bridge(&YzxExternBridgeSyncRequest {
            runtime_dir: invalid_runtime.path().to_path_buf(),
            state_dir: state.path().to_path_buf(),
        });
        let after_failed_refresh = fs::read_to_string(&data.extern_path).unwrap();

        assert!(failed.is_err());
        assert_eq!(after_failed_refresh, generated_content);
        assert!(!after_failed_refresh.contains("generated Nushell extern bridge (empty)"));
    }
}
