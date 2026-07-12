//! Temporary, Classic-owned transaction that leaves the canonical root Nova-native.

use crate::atomic_fs::write_text_atomic;
use crate::backup_timestamp::compact_utc_backup_timestamp;
use crate::bridge::{CoreError, ErrorClass};
use crate::classic_nova_root_translation::{
    ClassicNovaDisposition, ClassicNovaReportEntry, PACKAGED_NON_POPUP_CHORDS, POPUP_ROLE_MAPPINGS,
    chord_matches, executable, translate_classic_root, valid_chord, valid_popup_id,
};
use crate::config_normalize::validate_classic_config_table;
use crate::native_config_status::{path_owned_by_home_manager, path_present};
use crate::settings_contract::reconcile_settings_contract_text;
use crate::settings_surface::{json_value_to_toml_table, parse_config_value};
use crate::user_config_paths;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use toml::{Table, Value};

const NOVA_BAR_WIDGETS: &[&str] = &[
    "session",
    "editor",
    "shell",
    "term",
    "claude_usage",
    "codex_usage",
    "opencode_go_usage",
    "cpu",
    "ram",
];
const RESERVED_POPUP_TITLES: &[&str] = &["config_popup", "agent_popup", "git_popup", "menu_popup"];
const LEGACY_NATIVE_ZELLIJ_FIELDS: &[&str] = &[
    "disable_tips",
    "pane_frames",
    "rounded_corners",
    "default_mode",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaRoot {
    open: Option<NovaOpen>,
    shell: Option<NovaShell>,
    editor: Option<NovaEditor>,
    agent: Option<NovaAgent>,
    welcome: Option<NovaWelcome>,
    popup: Option<NovaPopupMargins>,
    keybindings: Option<NovaKeybindings>,
    bar: Option<NovaBar>,
    popups: Option<BTreeMap<String, NovaCustomPopup>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaOpen {
    log_level: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaShell {
    program: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaEditor {
    command: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaAgent {
    command: Option<String>,
    args: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaWelcome {
    #[serde(rename = "enabled")]
    _enabled: Option<bool>,
    style: Option<String>,
    duration_seconds: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaPopupMargins {
    side_margin: Option<i64>,
    vertical_margin: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaKeybindings {
    config: Option<String>,
    agent: Option<String>,
    git: Option<String>,
    menu: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaBar {
    widgets: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NovaCustomPopup {
    command: String,
    args: Option<Vec<String>>,
    title: Option<String>,
    keybinding: String,
    #[serde(rename = "keep_alive")]
    _keep_alive: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ClassicNovaMigrationRequest {
    pub config_dir: PathBuf,
    pub classic_default_config: PathBuf,
    pub classic_contract: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassicNovaMigrationStatus {
    Absent,
    NovaUnchanged,
    Migrated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassicNovaMigrationSource {
    ConfigToml,
    SettingsJsonc,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassicNovaMigrationReport {
    pub schema_version: u8,
    pub source: ClassicNovaMigrationSource,
    pub source_path: String,
    pub backup_path: String,
    pub target_path: String,
    pub entries: Vec<ClassicNovaReportEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassicNovaMigrationOutcome {
    pub status: ClassicNovaMigrationStatus,
    pub config_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub report_path: Option<PathBuf>,
}

pub fn migrate_classic_root_to_nova(
    request: &ClassicNovaMigrationRequest,
) -> Result<ClassicNovaMigrationOutcome, CoreError> {
    migrate_with(request, &compact_utc_backup_timestamp(), &RealTransactionIo)
}

trait TransactionIo {
    fn copy(&self, source: &Path, target: &Path) -> io::Result<()>;
    fn write_atomic(&self, path: &Path, contents: &str) -> Result<(), CoreError>;
    fn remove(&self, path: &Path) -> io::Result<()>;
}

struct RealTransactionIo;

impl TransactionIo for RealTransactionIo {
    fn copy(&self, source: &Path, target: &Path) -> io::Result<()> {
        fs::copy(source, target).map(|_| ())
    }

    fn write_atomic(&self, path: &Path, contents: &str) -> Result<(), CoreError> {
        write_text_atomic(path, contents)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }
}

fn migrate_with(
    request: &ClassicNovaMigrationRequest,
    timestamp: &str,
    transaction_io: &impl TransactionIo,
) -> Result<ClassicNovaMigrationOutcome, CoreError> {
    let config = user_config_paths::main_config(&request.config_dir);
    let legacy = user_config_paths::legacy_settings_config(&request.config_dir);
    let config_present = path_present(&config);
    let legacy_present = path_present(&legacy);
    if config_present && legacy_present {
        return Err(migration_error(
            "classic_nova_root_coexistence",
            "Both config.toml and the retired settings.jsonc exist; Yazelix cannot choose a migration source.",
            "Keep one intended root source, move the other aside, then retry.",
            json!({ "config": config, "legacy": legacy }),
        ));
    }
    if !config_present && !legacy_present {
        return Ok(outcome(
            ClassicNovaMigrationStatus::Absent,
            config,
            None,
            None,
        ));
    }

    let (source, source_kind, classic, mut extra_entries) = if config_present {
        let raw = read_source(&config)?;
        let root = parse_toml_root(&config, &raw)?;
        reject_embedded_cursor_settings(&root, &config)?;
        if has_classic_evidence(&root) && has_nova_evidence(&root) {
            return Err(migration_error(
                "mixed_classic_nova_root",
                "config.toml mixes Classic-only and Nova-only settings.",
                "Restore one coherent schema from version control or a backup, then retry.",
                json!({ "path": config }),
            ));
        }
        let nova_validation = validate_nova_root(&root);
        if nova_validation.is_ok() {
            return Ok(outcome(
                ClassicNovaMigrationStatus::NovaUnchanged,
                config,
                None,
                None,
            ));
        }
        if has_nova_evidence(&root) {
            return Err(migration_error(
                "invalid_nova_root",
                format!(
                    "config.toml uses Nova paths but is invalid: {}.",
                    nova_validation.expect_err("checked above")
                ),
                "Fix the reported Nova field, then retry.",
                json!({ "path": config }),
            ));
        }
        match validate_classic_config_table(
            &root,
            &request.classic_default_config,
            &request.classic_contract,
            &config,
        ) {
            Ok(_) => {}
            Err(classic_error) if !has_classic_evidence(&root) => {
                return Err(migration_error(
                    "ambiguous_root_schema",
                    "config.toml is neither a valid Nova root nor an identifiable Classic root.",
                    "Fix the reported fields or restore a coherent config.toml backup, then retry.",
                    json!({
                        "path": config,
                        "classic_error": classic_error.message(),
                        "nova_error": nova_validation.expect_err("checked above"),
                    }),
                ));
            }
            Err(error) => return Err(error),
        }
        (
            config.clone(),
            ClassicNovaMigrationSource::ConfigToml,
            root,
            Vec::new(),
        )
    } else {
        ensure_mutable_regular_source(&legacy)?;
        let raw = read_source(&legacy)?;
        let reconciled =
            reconcile_settings_contract_text(&legacy, &raw, &request.classic_default_config)?;
        let mut value = parse_config_value(&legacy, &reconciled.text)?;
        let object = value.as_object_mut().ok_or_else(|| {
            migration_error(
                "legacy_settings_not_object",
                "The retired settings.jsonc root is not an object.",
                "Restore a valid Yazelix settings.jsonc backup, then retry.",
                json!({ "path": legacy }),
            )
        })?;
        object.remove("ratconfig");
        if object.contains_key("cursors") {
            return Err(migration_error(
                "embedded_cursor_settings_unsupported",
                "The retired settings.jsonc contains cursor settings that do not belong in the Nova root.",
                "Move cursor settings to ~/.config/yazelix/cursors.toml, then retry.",
                json!({ "path": legacy }),
            ));
        }
        let extras = remove_legacy_native_zellij_fields(object);
        let root = json_value_to_toml_table(&value, &legacy)?;
        validate_classic_config_table(
            &root,
            &request.classic_default_config,
            &request.classic_contract,
            &legacy,
        )?;
        (
            legacy.clone(),
            ClassicNovaMigrationSource::SettingsJsonc,
            root,
            extras,
        )
    };

    ensure_mutable_regular_source(&source)?;
    let translation = translate_classic_root(&classic);
    if let Some(entry) = translation.report.iter().find(|entry| {
        entry.disposition == ClassicNovaDisposition::Rejected
            && (entry.detail.contains("conflict") || entry.detail.contains("duplicate"))
    }) {
        return Err(migration_error(
            "classic_nova_mapping_collision",
            format!("Cannot migrate {}: {}.", entry.source_path, entry.detail),
            "Resolve the conflicting Classic popup ids or keybindings, then retry.",
            json!({ "path": source, "entry": entry }),
        ));
    }
    validate_nova_root(&translation.root).map_err(|detail| {
        migration_error(
            "invalid_translated_nova_root",
            format!("The translated root violates the published Nova schema: {detail}."),
            "Report this as a Yazelix migration bug; the original source was not changed.",
            json!({ "path": source }),
        )
    })?;
    extra_entries.extend(translation.report);
    extra_entries.sort_by(|left, right| left.source_path.cmp(&right.source_path));

    let rendered = toml::to_string_pretty(&translation.root).map_err(|error| {
        migration_error(
            "render_nova_root",
            format!("Could not render the translated Nova root: {error}."),
            "Report this as a Yazelix migration bug; the original source was not changed.",
            json!({ "path": source }),
        )
    })?;
    let backup = backup_path(&source, timestamp);
    let report_path = report_path(&backup);
    ensure_destination_absent(&backup)?;
    ensure_destination_absent(&report_path)?;
    if source_kind == ClassicNovaMigrationSource::SettingsJsonc && path_present(&config) {
        return Err(migration_error(
            "classic_nova_target_collision",
            "config.toml appeared while the migration was being prepared.",
            "Move the competing config.toml aside, then retry.",
            json!({ "path": config }),
        ));
    }

    let report = ClassicNovaMigrationReport {
        schema_version: 1,
        source: source_kind,
        source_path: source.display().to_string(),
        backup_path: backup.display().to_string(),
        target_path: config.display().to_string(),
        entries: extra_entries,
    };
    let report_text = format!(
        "{}\n",
        serde_json::to_string_pretty(&report).map_err(|error| {
            migration_error(
                "render_classic_nova_report",
                format!("Could not render the migration report: {error}."),
                "Report this as a Yazelix migration bug; the original source was not changed.",
                json!({ "path": source }),
            )
        })?
    );

    transaction_io.copy(&source, &backup).map_err(|error| {
        io_error(
            "backup_classic_nova_root",
            &source,
            "Could not back up the Classic root before migration",
            error,
        )
    })?;
    transaction_io.write_atomic(&report_path, &report_text)?;
    transaction_io.write_atomic(&config, &rendered)?;
    if source_kind == ClassicNovaMigrationSource::SettingsJsonc {
        if let Err(error) = transaction_io.remove(&source) {
            let _ = fs::remove_file(&config);
            return Err(io_error(
                "retire_classic_settings_jsonc",
                &source,
                "Could not retire settings.jsonc after writing config.toml",
                error,
            ));
        }
    }

    Ok(outcome(
        ClassicNovaMigrationStatus::Migrated,
        config,
        Some(backup),
        Some(report_path),
    ))
}

fn remove_legacy_native_zellij_fields(
    root: &mut serde_json::Map<String, JsonValue>,
) -> Vec<ClassicNovaReportEntry> {
    let Some(zellij) = root.get_mut("zellij").and_then(JsonValue::as_object_mut) else {
        return Vec::new();
    };
    LEGACY_NATIVE_ZELLIJ_FIELDS
        .iter()
        .filter_map(|field| {
            zellij.remove(*field).map(|_| ClassicNovaReportEntry {
                source_path: format!("zellij.{field}"),
                disposition: ClassicNovaDisposition::Manual,
                target_paths: Vec::new(),
                detail: "verify this retired native preference in zellij/config.kdl; the root migration never mutates sidecars".to_string(),
            })
        })
        .collect()
}

fn validate_nova_root(root: &Table) -> Result<(), String> {
    let nova = Value::Table(root.clone())
        .try_into::<NovaRoot>()
        .map_err(|error| error.to_string())?;
    validate_optional_enum(
        "open.log_level",
        nova.open.and_then(|table| table.log_level),
        &["off", "error", "info", "debug"],
    )?;
    validate_optional_enum(
        "shell.program",
        nova.shell.and_then(|table| table.program),
        &["nu", "bash", "fish", "zsh"],
    )?;
    if let Some(command) = nova.editor.and_then(|table| table.command) {
        validate_executable("editor.command", &command)?;
    }
    if let Some(agent) = nova.agent {
        let command = agent.command.as_deref().unwrap_or("auto");
        validate_executable("agent.command", command)?;
        if command == "auto" && agent.args.as_ref().is_some_and(|args| !args.is_empty()) {
            return Err("agent.args requires a custom agent.command".to_string());
        }
    }
    if let Some(welcome) = nova.welcome {
        validate_optional_enum(
            "welcome.style",
            welcome.style,
            &[
                "static",
                "logo",
                "boids",
                "boids_predator",
                "boids_schools",
                "mandelbrot",
                "game_of_life_gliders",
                "game_of_life_oscillators",
                "game_of_life_bloom",
                "random",
            ],
        )?;
        if welcome
            .duration_seconds
            .is_some_and(|duration| !(1..=60).contains(&duration))
        {
            return Err("welcome.duration_seconds must be between 1 and 60".to_string());
        }
    }
    if let Some(popup) = nova.popup {
        if popup.side_margin.is_some_and(|margin| margin < 0)
            || popup.vertical_margin.is_some_and(|margin| margin < 0)
        {
            return Err("popup margins must be zero or greater".to_string());
        }
    }
    if let Some(widgets) = nova.bar.and_then(|bar| bar.widgets) {
        if let Some(widget) = widgets
            .iter()
            .find(|widget| !NOVA_BAR_WIDGETS.contains(&widget.as_str()))
        {
            return Err(format!("bar.widgets contains unsupported value {widget:?}"));
        }
    }
    validate_nova_popups(nova.keybindings, nova.popups.unwrap_or_default())
}

fn validate_nova_popups(
    keybindings: Option<NovaKeybindings>,
    popups: BTreeMap<String, NovaCustomPopup>,
) -> Result<(), String> {
    let mut chords = POPUP_ROLE_MAPPINGS
        .iter()
        .map(|(_, role, default)| ((*role).to_string(), (*default).to_string()))
        .collect::<BTreeMap<_, _>>();
    if let Some(keybindings) = keybindings {
        for (role, chord) in [
            ("config", keybindings.config),
            ("agent", keybindings.agent),
            ("git", keybindings.git),
            ("menu", keybindings.menu),
        ] {
            if let Some(chord) = chord {
                validate_chord(&format!("keybindings.{role}"), &chord)?;
                chords.insert(role.to_string(), chord);
            }
        }
    }
    let mut used_chords = BTreeMap::new();
    for (role, chord) in &chords {
        if let Some(existing) = used_chords.insert(chord.to_ascii_lowercase(), role.clone()) {
            return Err(format!(
                "keybindings.{role} conflicts with keybindings.{existing}"
            ));
        }
    }

    let mut titles = BTreeMap::new();
    for (id, popup) in popups {
        if !valid_popup_id(&id) || matches!(id.as_str(), "config" | "agent" | "git" | "menu") {
            return Err(format!("popups.{id} has an invalid or reserved id"));
        }
        let path = format!("popups.{id}");
        validate_executable(&format!("{path}.command"), &popup.command)?;
        if popup
            .args
            .as_ref()
            .is_some_and(|args| args.iter().any(|argument| argument.trim().is_empty()))
        {
            return Err(format!("{path}.args must not contain empty strings"));
        }
        let title = popup
            .title
            .unwrap_or_else(|| format!("{id}_popup"))
            .trim()
            .to_string();
        if title.is_empty() {
            return Err(format!("{path}.title must be a non-empty string"));
        }
        if RESERVED_POPUP_TITLES.contains(&title.as_str()) {
            return Err(format!(
                "{path}.title conflicts with a packaged popup title"
            ));
        }
        if let Some(existing) = titles.insert(title.clone(), id.clone()) {
            return Err(format!(
                "{path}.title conflicts with popups.{existing}.title"
            ));
        }
        validate_chord(&format!("{path}.keybinding"), &popup.keybinding)?;
        if let Some(existing) =
            used_chords.insert(popup.keybinding.to_ascii_lowercase(), path.clone())
        {
            return Err(format!("{path}.keybinding conflicts with {existing}"));
        }
    }
    Ok(())
}

fn validate_optional_enum(
    path: &str,
    value: Option<String>,
    allowed: &[&str],
) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    if allowed.contains(&value.as_str()) {
        Ok(())
    } else {
        Err(format!("{path} has unsupported value {value:?}"))
    }
}

fn validate_executable(path: &str, value: &str) -> Result<(), String> {
    if executable(value) {
        Ok(())
    } else {
        Err(format!("{path} must be one executable without arguments"))
    }
}

fn validate_chord(path: &str, chord: &str) -> Result<(), String> {
    if !valid_chord(chord) {
        return Err(format!("{path} must be a key chord like Alt Shift A"));
    }
    if PACKAGED_NON_POPUP_CHORDS
        .iter()
        .any(|packaged| chord_matches(packaged, chord))
    {
        return Err(format!("{path} conflicts with packaged key {chord}"));
    }
    Ok(())
}

fn value_at<'a>(root: &'a Table, path: &str) -> Option<&'a Value> {
    let mut segments = path.split('.');
    let mut value = root.get(segments.next()?)?;
    for segment in segments {
        value = value.as_table()?.get(segment)?;
    }
    Some(value)
}

fn has_classic_evidence(root: &Table) -> bool {
    ["core", "helix", "workspace", "appearance", "zellij", "yazi"]
        .iter()
        .any(|key| root.contains_key(*key))
        || value_at(root, "shell.default_shell").is_some()
        || value_at(root, "editor.hide_sidebar_on_file_open").is_some()
}

fn has_nova_evidence(root: &Table) -> bool {
    [
        "open",
        "agent",
        "welcome",
        "popup",
        "keybindings",
        "bar",
        "popups",
    ]
    .iter()
    .any(|key| root.contains_key(*key))
        || value_at(root, "shell.program").is_some()
}

fn reject_embedded_cursor_settings(root: &Table, path: &Path) -> Result<(), CoreError> {
    if !root.contains_key("cursors") {
        return Ok(());
    }
    Err(migration_error(
        "embedded_cursor_settings_unsupported",
        "config.toml contains cursor settings that do not belong in the Nova root.",
        "Move cursor settings to ~/.config/yazelix/cursors.toml, then retry.",
        json!({ "path": path }),
    ))
}

fn parse_toml_root(path: &Path, raw: &str) -> Result<Table, CoreError> {
    toml::from_str(raw).map_err(|error| {
        CoreError::toml(
            "invalid_classic_nova_root_toml",
            "Could not parse config.toml before schema classification",
            "Fix the TOML syntax or restore a valid backup, then retry.",
            path.display().to_string(),
            error,
        )
    })
}

fn read_source(path: &Path) -> Result<String, CoreError> {
    fs::read_to_string(path).map_err(|error| {
        io_error(
            "read_classic_nova_root",
            path,
            "Could not read the root migration source",
            error,
        )
    })
}

fn ensure_mutable_regular_source(path: &Path) -> Result<(), CoreError> {
    if path_owned_by_home_manager(path) {
        return Err(migration_error(
            "home_manager_owned_root_migration",
            format!("{} is owned by Home Manager.", path.display()),
            "Declare Nova-native programs.yazelix.config.settings values and run your normal Home Manager switch.",
            json!({ "path": path }),
        ));
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        io_error(
            "inspect_classic_nova_root",
            path,
            "Could not inspect the root migration source",
            error,
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(migration_error(
            "ambiguous_root_file_owner",
            format!("{} is not a regular user-owned file.", path.display()),
            "Replace it explicitly with one writable regular file or update its declarative owner.",
            json!({ "path": path }),
        ));
    }
    if metadata.permissions().readonly() {
        return Err(migration_error(
            "read_only_root_migration",
            format!("{} is read-only.", path.display()),
            "Make the user-owned source writable or update its declarative owner, then retry.",
            json!({ "path": path }),
        ));
    }
    Ok(())
}

fn backup_path(source: &Path, timestamp: &str) -> PathBuf {
    let name = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config");
    source.with_file_name(format!("{name}.backup-{timestamp}"))
}

fn report_path(backup: &Path) -> PathBuf {
    let name = backup
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.backup");
    backup.with_file_name(format!("{name}.migration_report.json"))
}

fn ensure_destination_absent(path: &Path) -> Result<(), CoreError> {
    if !path_present(path) {
        return Ok(());
    }
    Err(migration_error(
        "classic_nova_migration_artifact_collision",
        format!(
            "Refusing to replace existing migration artifact {}.",
            path.display()
        ),
        "Preserve or move the existing backup/report, then retry.",
        json!({ "path": path }),
    ))
}

fn outcome(
    status: ClassicNovaMigrationStatus,
    config_path: PathBuf,
    backup_path: Option<PathBuf>,
    report_path: Option<PathBuf>,
) -> ClassicNovaMigrationOutcome {
    ClassicNovaMigrationOutcome {
        status,
        config_path,
        backup_path,
        report_path,
    }
}

fn migration_error(
    code: &'static str,
    message: impl Into<String>,
    remediation: impl Into<String>,
    details: JsonValue,
) -> CoreError {
    CoreError::classified(ErrorClass::Config, code, message, remediation, details)
}

fn io_error(code: &'static str, path: &Path, message: &str, error: io::Error) -> CoreError {
    CoreError::io(
        code,
        message,
        "Fix permissions or free disk space, then retry; the original source remains available.",
        path.display().to_string(),
        error,
    )
}

#[cfg(test)]
#[path = "classic_nova_root_migration_tests.rs"]
mod tests;
