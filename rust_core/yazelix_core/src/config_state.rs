use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use crate::settings_surface::read_config_table;
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;

const MATERIALIZED_STATE_SCHEMA_VERSION: u64 = 2;
const GENERATED_STATE_MATERIALIZER_SCHEMA_VERSION: u64 = 2;

#[derive(Debug, Clone)]
pub struct ComputeConfigStateRequest {
    pub config_path: PathBuf,
    pub default_config_path: PathBuf,
    pub contract_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub state_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RecordConfigStateRequest {
    pub config_file: String,
    pub managed_config_path: PathBuf,
    pub state_path: PathBuf,
    pub runtime_dir: Option<PathBuf>,
    pub config_hash: String,
    pub runtime_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigStateData {
    pub config: JsonMap<String, JsonValue>,
    pub config_file: String,
    pub needs_refresh: bool,
    pub refresh_reason: String,
    pub config_changed: bool,
    pub inputs_changed: bool,
    pub inputs_require_refresh: bool,
    pub config_hash: String,
    pub runtime_hash: String,
    pub combined_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordConfigStateData {
    pub recorded: bool,
}

#[derive(Debug, Clone)]
enum CachedState {
    Missing,
    Structured {
        config_hash: String,
        runtime_hash: String,
        runtime_source_last_modified_date: Option<String>,
        materializer_schema_version: Option<u64>,
    },
}

pub fn compute_config_state(
    request: &ComputeConfigStateRequest,
) -> Result<ConfigStateData, CoreError> {
    let normalize_data = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: true,
    })?;
    let raw_config = read_config_table(&request.config_path, "read_config")?;
    let contract = read_toml_table(&request.contract_path, "read_config_contract")?;
    let rebuild_paths = load_rebuild_required_paths(&contract);
    let rebuild_config = extract_rebuild_config(&raw_config, &rebuild_paths);
    let rebuild_config_toml = toml::to_string(&rebuild_config).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_rebuild_config",
            format!("Could not serialize rebuild-required config: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    // The zellij.kdl override sidecar is merged into the generated config.kdl at
    // materialization time, so its content is a real rebuild input. Fold it into
    // the config hash (only when present) so editing it alone is detected as
    // drift instead of being silently missed by the freshness check.
    let sidecar_fingerprint = config_override_sidecar_fingerprint(&request.config_path)?;
    let config_hash = sha256_hex(&format!("{rebuild_config_toml}{sidecar_fingerprint}"));
    let runtime_hash = compute_runtime_refresh_hash(&request.runtime_dir)?;
    let combined_hash = sha256_hex(&format!("{config_hash}:{runtime_hash}"));
    let cached_state = load_recorded_materialized_state(&request.state_path)?;

    let has_structured_cache = matches!(cached_state, CachedState::Structured { .. });
    let (cached_config_hash, cached_runtime_hash) = match &cached_state {
        CachedState::Structured {
            config_hash,
            runtime_hash,
            ..
        } => (config_hash.as_str(), runtime_hash.as_str()),
        CachedState::Missing => ("", ""),
    };

    reject_known_runtime_downgrade(&cached_state, &request.runtime_dir)?;

    let config_changed = has_structured_cache && config_hash != cached_config_hash;
    let inputs_changed = has_structured_cache && runtime_hash != cached_runtime_hash;
    let inputs_require_refresh = match &cached_state {
        CachedState::Structured { .. } => config_changed || inputs_changed,
        CachedState::Missing => true,
    };
    let needs_refresh = inputs_require_refresh;
    let refresh_reason = refresh_reason(
        needs_refresh,
        inputs_require_refresh,
        has_structured_cache,
        config_changed,
        inputs_changed,
    );

    Ok(ConfigStateData {
        config: normalize_data.normalized_config,
        config_file: normalize_data.config_file,
        needs_refresh,
        refresh_reason,
        config_changed,
        inputs_changed,
        inputs_require_refresh,
        config_hash,
        runtime_hash,
        combined_hash,
    })
}

pub fn record_config_state(
    request: &RecordConfigStateRequest,
) -> Result<RecordConfigStateData, CoreError> {
    if !is_default_managed_surface(&request.config_file, &request.managed_config_path) {
        return Ok(RecordConfigStateData { recorded: false });
    }

    let runtime_source = request
        .runtime_dir
        .as_deref()
        .map(runtime_source_metadata)
        .transpose()?
        .flatten();
    let mut state = json!({
        "schema_version": MATERIALIZED_STATE_SCHEMA_VERSION,
        "materializer_schema_version": GENERATED_STATE_MATERIALIZER_SCHEMA_VERSION,
        "config_hash": request.config_hash,
        "runtime_hash": request.runtime_hash,
    });
    if let Some(runtime_source) = runtime_source {
        if let Some(object) = state.as_object_mut() {
            object.insert("runtime_source".to_string(), runtime_source);
        }
    }
    write_json_atomic(&request.state_path, &state)?;
    Ok(RecordConfigStateData { recorded: true })
}

fn read_toml_table(path: &Path, code: &str) -> Result<toml::Table, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            code,
            "Could not read Yazelix config-state input",
            "Ensure the explicit config, contract, and state paths exist and are readable.",
            path.to_string_lossy(),
            source,
        )
    })?;
    toml::from_str::<toml::Table>(&raw).map_err(|source| {
        CoreError::toml(
            "invalid_toml",
            "Could not parse Yazelix TOML input",
            "Fix the TOML syntax in the reported file and retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

pub(crate) fn compute_runtime_refresh_hash(runtime_dir: &Path) -> Result<String, CoreError> {
    let identity_path = runtime_dir.join("runtime_identity.json");
    if !identity_path.exists() {
        return Ok(sha256_hex(&format!(
            "runtime_path:{}",
            path_to_string(runtime_dir)
        )));
    }

    let raw = fs::read_to_string(&identity_path).map_err(|source| {
        CoreError::io(
            "runtime_refresh_identity",
            "Could not read Yazelix runtime identity for generated-state freshness.",
            "Reinstall Yazelix from a current package so runtime_identity.json is readable.",
            identity_path.to_string_lossy(),
            source,
        )
    })?;
    let identity = serde_json::from_str::<JsonValue>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_runtime_refresh_identity",
            format!(
                "Yazelix runtime identity is invalid JSON at {}.",
                identity_path.display()
            ),
            "Reinstall Yazelix from a current package so runtime_identity.json is valid.",
            json!({
                "path": identity_path.display().to_string(),
                "error": source.to_string(),
            }),
        )
    })?;
    let object = identity.as_object().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_runtime_refresh_identity_shape",
            format!(
                "Yazelix runtime identity must be a JSON object at {}.",
                identity_path.display()
            ),
            "Reinstall Yazelix from a current package so runtime_identity.json has the supported shape.",
            json!({ "path": identity_path.display().to_string() }),
        )
    })?;

    let refresh_identity = json!({
        "generated_state_materializer_schema_version": GENERATED_STATE_MATERIALIZER_SCHEMA_VERSION,
        "runtime_dir": path_to_string(runtime_dir),
        "schema_version": object.get("schema_version").cloned().unwrap_or(JsonValue::Null),
        "version": object.get("version").cloned().unwrap_or(JsonValue::Null),
        "source": object.get("source").cloned().unwrap_or(JsonValue::Null),
        "inputs": object.get("inputs").cloned().unwrap_or(JsonValue::Null),
    });
    serde_json::to_string(&refresh_identity)
        .map(|serialized| sha256_hex(&serialized))
        .map_err(|source| {
            CoreError::classified(
                ErrorClass::Internal,
                "serialize_runtime_refresh_identity",
                format!("Could not serialize runtime refresh identity: {source}"),
                "Report this as a Yazelix internal error.",
                json!({ "path": identity_path.display().to_string() }),
            )
        })
}

fn runtime_source_metadata(runtime_dir: &Path) -> Result<Option<JsonValue>, CoreError> {
    let identity_path = runtime_dir.join("runtime_identity.json");
    if !identity_path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&identity_path).map_err(|source| {
        CoreError::io(
            "read_runtime_source_metadata",
            "Could not read Yazelix runtime identity for generated-state metadata.",
            "Reinstall Yazelix from a current package so runtime_identity.json is readable.",
            identity_path.to_string_lossy(),
            source,
        )
    })?;
    let identity = serde_json::from_str::<JsonValue>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "invalid_runtime_source_metadata",
            format!(
                "Yazelix runtime identity is invalid JSON at {}.",
                identity_path.display()
            ),
            "Reinstall Yazelix from a current package so runtime_identity.json is valid.",
            json!({
                "path": identity_path.display().to_string(),
                "error": source.to_string(),
            }),
        )
    })?;
    Ok(identity
        .get("source")
        .and_then(JsonValue::as_object)
        .map(|source| JsonValue::Object(source.clone())))
}

fn runtime_source_last_modified_date(runtime_dir: &Path) -> Result<Option<String>, CoreError> {
    Ok(runtime_source_metadata(runtime_dir)?
        .and_then(|source| source.get("last_modified_date").cloned())
        .and_then(|value| value.as_str().map(str::to_string)))
}

fn reject_known_runtime_downgrade(
    cached_state: &CachedState,
    runtime_dir: &Path,
) -> Result<(), CoreError> {
    let CachedState::Structured {
        runtime_source_last_modified_date: Some(cached_date),
        materializer_schema_version: Some(cached_schema),
        ..
    } = cached_state
    else {
        return Ok(());
    };
    if *cached_schema < GENERATED_STATE_MATERIALIZER_SCHEMA_VERSION {
        return Ok(());
    }
    let Some(current_date) = runtime_source_last_modified_date(runtime_dir)? else {
        return Ok(());
    };
    if current_date < *cached_date {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "generated_state_runtime_downgrade",
            format!(
                "Refusing to regenerate Yazelix generated state from an older runtime source ({current_date}) than the recorded generated state ({cached_date})."
            ),
            "Launch or repair with the newer Yazelix runtime that last generated this state, or deliberately reset generated state before using an older runtime.",
            json!({
                "current_runtime_source_last_modified_date": current_date,
                "recorded_runtime_source_last_modified_date": cached_date,
                "current_materializer_schema_version": GENERATED_STATE_MATERIALIZER_SCHEMA_VERSION,
                "recorded_materializer_schema_version": cached_schema,
            }),
        ));
    }
    Ok(())
}

fn load_rebuild_required_paths(contract: &toml::Table) -> Vec<String> {
    let mut paths = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(path, field)| {
                    let table = field.as_table()?;
                    let rebuild_required = table
                        .get("rebuild_required")
                        .and_then(TomlValue::as_bool)
                        .unwrap_or(false);
                    let generated_runtime_refresh =
                        table.get("apply_mode").and_then(TomlValue::as_str)
                            == Some("generated_runtime_refresh");
                    (rebuild_required || generated_runtime_refresh).then(|| path.clone())
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if let Some(extra_paths) = contract
        .get("rebuild")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("extra_paths"))
        .and_then(TomlValue::as_array)
    {
        paths.extend(
            extra_paths
                .iter()
                .filter_map(TomlValue::as_str)
                .map(ToOwned::to_owned),
        );
    }

    paths
}

/// Fingerprint of the user's `zellij.kdl` override sidecar for the freshness
/// hash. The sidecar lives next to `settings.jsonc` and is merged into the
/// generated `config.kdl`, so its content must participate in drift detection.
/// Returns an empty string when the sidecar is absent so sidecar-less configs
/// keep their previous hash (no spurious one-time re-materialization).
fn config_override_sidecar_fingerprint(config_path: &Path) -> Result<String, CoreError> {
    let Some(parent) = config_path.parent() else {
        return Ok(String::new());
    };
    let sidecar_path = parent.join(crate::user_config_paths::ZELLIJ_CONFIG);
    match fs::read_to_string(&sidecar_path) {
        Ok(content) => Ok(format!("\n---zellij-override-sidecar---\n{content}")),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(CoreError::io(
            "read_zellij_override_sidecar",
            "Could not read the Yazelix zellij.kdl override sidecar for generated-state freshness.",
            "Fix permissions on ~/.config/yazelix/zellij.kdl, or remove it if unused.",
            sidecar_path.to_string_lossy(),
            source,
        )),
    }
}

fn extract_rebuild_config(config: &toml::Table, rebuild_paths: &[String]) -> toml::Table {
    let root = TomlValue::Table(config.clone());
    let mut rebuild_config = TomlValue::Table(toml::Table::new());
    for path in rebuild_paths {
        let segments = path.split('.').collect::<Vec<_>>();
        if let Some(value) = get_nested_value(&root, &segments).cloned() {
            set_nested_value(&mut rebuild_config, &segments, value);
        }
    }

    rebuild_config.as_table().cloned().unwrap_or_default()
}

fn get_nested_value<'a>(value: &'a TomlValue, path: &[&str]) -> Option<&'a TomlValue> {
    let mut current = value;
    for segment in path {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

fn set_nested_value(value: &mut TomlValue, path: &[&str], new_value: TomlValue) {
    if path.is_empty() {
        *value = new_value;
        return;
    }
    let Some(table) = value.as_table_mut() else {
        return;
    };
    if path.len() == 1 {
        table.insert(path[0].to_string(), new_value);
        return;
    }
    let entry = table
        .entry(path[0].to_string())
        .or_insert_with(|| TomlValue::Table(toml::Table::new()));
    set_nested_value(entry, &path[1..], new_value);
}

fn load_recorded_materialized_state(path: &Path) -> Result<CachedState, CoreError> {
    if !path.exists() {
        return Ok(CachedState::Missing);
    }

    let raw_state = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "read_materialized_state",
            "Could not read Yazelix generated-state cache",
            "Remove the generated-state cache file and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    let trimmed = raw_state.trim();
    if trimmed.is_empty() {
        return Ok(CachedState::Missing);
    }

    let Ok(value) = serde_json::from_str::<JsonValue>(trimmed) else {
        return Ok(CachedState::Missing);
    };

    match value {
        JsonValue::Object(record) => Ok(CachedState::Structured {
            config_hash: record
                .get("config_hash")
                .and_then(JsonValue::as_str)
                .unwrap_or("")
                .to_string(),
            runtime_hash: record
                .get("runtime_hash")
                .and_then(JsonValue::as_str)
                .unwrap_or("")
                .to_string(),
            runtime_source_last_modified_date: record
                .get("runtime_source")
                .and_then(JsonValue::as_object)
                .and_then(|source| source.get("last_modified_date"))
                .and_then(JsonValue::as_str)
                .map(str::to_string),
            materializer_schema_version: record
                .get("materializer_schema_version")
                .and_then(JsonValue::as_u64),
        }),
        _ => Ok(CachedState::Missing),
    }
}

fn refresh_reason(
    needs_refresh: bool,
    inputs_require_refresh: bool,
    has_structured_cache: bool,
    config_changed: bool,
    inputs_changed: bool,
) -> String {
    if !needs_refresh {
        String::new()
    } else if inputs_require_refresh && !has_structured_cache {
        "config or runtime inputs changed since last generated-state repair".to_string()
    } else if inputs_require_refresh && config_changed && inputs_changed {
        "config and runtime inputs changed since last generated-state repair".to_string()
    } else if inputs_require_refresh && config_changed {
        "config changed since last generated-state repair".to_string()
    } else if inputs_require_refresh && inputs_changed {
        "runtime inputs changed since last generated-state repair".to_string()
    } else {
        "config or runtime inputs changed since last generated-state repair".to_string()
    }
}

fn is_default_managed_surface(config_file: &str, managed_config_path: &Path) -> bool {
    if config_file.is_empty() {
        return true;
    }

    let normalized_config_file = normalize_without_symlink(Path::new(config_file));
    let normalized_managed = normalize_without_symlink(managed_config_path);
    if normalized_config_file == normalized_managed {
        return true;
    }

    normalized_config_file.parent() == normalized_managed.parent()
        && normalized_config_file.file_name() == normalized_managed.file_name()
}

fn normalize_without_symlink(path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn write_json_atomic(path: &Path, value: &JsonValue) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Io,
            "invalid_state_path",
            "Generated-state cache path has no parent directory",
            "Report this as a Yazelix internal error.",
            json!({ "path": path.to_string_lossy() }),
        )
    })?;
    fs::create_dir_all(parent).map_err(|source| {
        CoreError::io(
            "create_state_dir",
            "Could not create Yazelix generated-state cache directory",
            "Check permissions for the Yazelix state directory and retry.",
            parent.to_string_lossy(),
            source,
        )
    })?;

    let temporary_path = path.with_file_name(format!(
        ".{}.yazelix-tmp-{}-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("rebuild_hash"),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    ));
    let serialized = serde_json::to_string(value).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_materialized_state",
            format!("Could not serialize generated-state cache: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    fs::write(&temporary_path, serialized).map_err(|source| {
        CoreError::io(
            "write_materialized_state",
            "Could not write Yazelix generated-state cache",
            "Check permissions for the Yazelix state directory and retry.",
            temporary_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temporary_path, path).map_err(|source| {
        CoreError::io(
            "rename_materialized_state",
            "Could not replace Yazelix generated-state cache",
            "Check permissions for the Yazelix state directory and retry.",
            path.to_string_lossy(),
            source,
        )
    })?;
    Ok(())
}

fn sha256_hex(input: &str) -> String {
    format!("{:x}", Sha256::digest(input.as_bytes()))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use tempfile::tempdir;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root")
    }

    fn request_for(
        config_path: PathBuf,
        runtime_dir: PathBuf,
        state_path: PathBuf,
    ) -> ComputeConfigStateRequest {
        let repo = repo_root();
        ComputeConfigStateRequest {
            config_path,
            default_config_path: repo.join("settings_default.jsonc"),
            contract_path: repo.join("config_metadata/main_config_contract.toml"),
            runtime_dir,
            state_path,
        }
    }

    fn default_settings_jsonc() -> JsonValue {
        crate::settings_surface::read_settings_jsonc_value(
            &repo_root().join("settings_default.jsonc"),
        )
        .expect("default settings")
    }

    fn write_settings_config(dir: &Path, value: &JsonValue) -> PathBuf {
        let path = dir.join("settings.jsonc");
        fs::write(
            &path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(value).expect("settings json")
            ),
        )
        .expect("write config");
        path
    }

    fn write_runtime_identity(runtime_dir: &Path, variant: &str, source_revision: &str) {
        write_runtime_identity_with_date(runtime_dir, variant, source_revision, "20260620000000");
    }

    fn write_runtime_identity_with_date(
        runtime_dir: &Path,
        variant: &str,
        source_revision: &str,
        last_modified_date: &str,
    ) {
        fs::create_dir_all(runtime_dir).expect("runtime dir");
        fs::write(
            runtime_dir.join("runtime_identity.json"),
            serde_json::to_string(&json!({
                "schema_version": 1,
                "version": "v17.7",
                "runtime_variant": variant,
                "source": {
                    "revision": source_revision,
                    "short_revision": &source_revision[..7.min(source_revision.len())],
                    "last_modified_date": last_modified_date,
                },
                "inputs": {
                    "nixpkgs": {
                        "revision": "input-revision",
                        "short_revision": "input-r",
                        "last_modified_date": "20260619000000",
                    }
                }
            }))
            .expect("identity json"),
        )
        .expect("write runtime identity");
    }

    // Invariant: config-state hashing stays stable for the default config when no prior state exists.
    #[test]
    fn computes_default_rebuild_hash_without_recorded_state() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let state_path = dir.path().join("state/rebuild_hash");
        let config_path = repo_root().join("settings_default.jsonc");
        let state = compute_config_state(&request_for(
            config_path,
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .unwrap();

        assert_eq!(
            state.config_hash,
            "dcb3fc2de8ad8c2e1ef4e0232226b635f1086fc9920e248670984237cbe9c88b"
        );
        assert!(state.needs_refresh);
    }

    // Defends: the zellij.kdl override sidecar participates in the config freshness
    // hash, so editing it alone is detected as drift instead of silently missed.
    #[test]
    fn zellij_override_sidecar_participates_in_config_hash() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = dir.path().join("runtime");
        write_runtime_identity(&runtime_dir, "kitty", "0123456789abcdef");
        let state_path = dir.path().join("state/rebuild_hash");
        let config_dir = dir.path().join("config");
        fs::create_dir_all(&config_dir).expect("config dir");
        let config_path = write_settings_config(&config_dir, &default_settings_jsonc());
        let sidecar_path = config_dir.join(crate::user_config_paths::ZELLIJ_CONFIG);

        let baseline = compute_config_state(&request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .expect("baseline state");

        // Adding a sidecar changes the config hash.
        fs::write(&sidecar_path, "scrollback_lines_to_serialize 100000\n").expect("write sidecar");
        let with_sidecar = compute_config_state(&request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .expect("sidecar state");
        assert_ne!(with_sidecar.config_hash, baseline.config_hash);

        // Editing the sidecar changes the hash again.
        fs::write(&sidecar_path, "scrollback_lines_to_serialize 5000\n").expect("edit sidecar");
        let edited = compute_config_state(&request_for(config_path, runtime_dir, state_path))
            .expect("edited state");
        assert_ne!(edited.config_hash, with_sidecar.config_hash);
    }

    // Regression: malformed legacy state cache content must be treated as missing instead of trusted.
    #[test]
    fn treats_malformed_state_cache_as_missing() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let state_path = dir.path().join("state/rebuild_hash");

        fs::create_dir_all(state_path.parent().unwrap()).unwrap();
        fs::write(&state_path, "legacy-raw-hash-or-garbage").unwrap();

        let state = compute_config_state(&request_for(
            repo_root().join("settings_default.jsonc"),
            runtime_dir,
            state_path,
        ))
        .unwrap();

        assert!(state.needs_refresh);
        assert_eq!(
            state.refresh_reason,
            "config or runtime inputs changed since last generated-state repair"
        );
    }

    // Defends: config-state hashing ignores non-rebuild settings while still invalidating on rebuild-driving changes.
    #[test]
    fn ignores_non_rebuild_config_changes_but_flags_rebuild_changes() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let state_path = dir.path().join("state/rebuild_hash");
        let mut config = default_settings_jsonc();
        config["core"]["skip_welcome_screen"] = json!(false);
        config["editor"]["command"] = json!("hx");
        let config_path = write_settings_config(dir.path(), &config);
        let baseline = compute_config_state(&request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .unwrap();
        record_config_state(&RecordConfigStateRequest {
            config_file: config_path.to_string_lossy().to_string(),
            managed_config_path: config_path.clone(),
            state_path: state_path.clone(),
            runtime_dir: Some(runtime_dir.clone()),
            config_hash: baseline.config_hash.clone(),
            runtime_hash: baseline.runtime_hash.clone(),
        })
        .unwrap();

        config["core"]["skip_welcome_screen"] = json!(true);
        write_settings_config(dir.path(), &config);
        let runtime_only = compute_config_state(&request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .unwrap();
        assert_eq!(baseline.config_hash, runtime_only.config_hash);
        assert!(!runtime_only.needs_refresh);

        config["editor"]["command"] = json!("nvim");
        write_settings_config(dir.path(), &config);
        let rebuild_changed =
            compute_config_state(&request_for(config_path, runtime_dir, state_path)).unwrap();
        assert_ne!(runtime_only.config_hash, rebuild_changed.config_hash);
        assert!(rebuild_changed.needs_refresh);
        assert_eq!(
            rebuild_changed.refresh_reason,
            "config changed since last generated-state repair"
        );
    }

    // Regression: generated-runtime apply fields must be part of the materialized-state hash.
    #[test]
    fn flags_generated_runtime_refresh_config_changes() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let state_path = dir.path().join("state/rebuild_hash");
        let mut config = default_settings_jsonc();
        let config_path = write_settings_config(dir.path(), &config);
        let baseline = compute_config_state(&request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .unwrap();
        record_config_state(&RecordConfigStateRequest {
            config_file: config_path.to_string_lossy().to_string(),
            managed_config_path: config_path.clone(),
            state_path: state_path.clone(),
            runtime_dir: Some(runtime_dir.clone()),
            config_hash: baseline.config_hash.clone(),
            runtime_hash: baseline.runtime_hash.clone(),
        })
        .unwrap();

        config["zellij"]["widget_tray"] = json!([
            "session",
            "editor",
            "shell",
            "term",
            "workspace",
            "codex_usage"
        ]);
        write_settings_config(dir.path(), &config);
        let changed = compute_config_state(&request_for(config_path, runtime_dir, state_path))
            .expect("config state");

        assert_ne!(baseline.config_hash, changed.config_hash);
        assert!(changed.needs_refresh);
        assert_eq!(
            changed.refresh_reason,
            "config changed since last generated-state repair"
        );
    }

    // Regression: generated runtime files embed absolute runtime paths, so an
    // otherwise identical package at a new store path must refresh generated state.
    #[test]
    fn runtime_refresh_hash_includes_runtime_store_path() {
        let dir = tempdir().expect("tempdir");
        let current_runtime = dir.path().join("store/current-yazelix-mars");
        let sibling_runtime = dir.path().join("store/current-yazelix-mars-copy");
        let old_runtime = dir.path().join("store/old-yazelix-mars");
        write_runtime_identity(
            &current_runtime,
            "mars",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        write_runtime_identity(
            &sibling_runtime,
            "mars",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );
        write_runtime_identity(
            &old_runtime,
            "mars",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        );

        let current_hash = compute_runtime_refresh_hash(&current_runtime).unwrap();
        let sibling_hash = compute_runtime_refresh_hash(&sibling_runtime).unwrap();
        let old_hash = compute_runtime_refresh_hash(&old_runtime).unwrap();

        assert_ne!(current_hash, sibling_hash);
        assert_ne!(current_hash, old_hash);
    }

    // Regression: generated-state repair must be additive; current code refuses a known older runtime source.
    #[test]
    fn compute_config_state_rejects_known_runtime_downgrade() {
        let dir = tempdir().expect("tempdir");
        let state_path = dir.path().join("state/rebuild_hash");
        let config_path = repo_root().join("settings_default.jsonc");
        let newer_runtime = dir.path().join("store/newer-yazelix-mars");
        let older_runtime = dir.path().join("store/older-yazelix-mars");
        write_runtime_identity_with_date(
            &newer_runtime,
            "mars",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "20260702000000",
        );
        write_runtime_identity_with_date(
            &older_runtime,
            "mars",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "20260701000000",
        );

        let newer_state = compute_config_state(&request_for(
            config_path.clone(),
            newer_runtime.clone(),
            state_path.clone(),
        ))
        .unwrap();
        record_config_state(&RecordConfigStateRequest {
            config_file: config_path.to_string_lossy().to_string(),
            managed_config_path: config_path.clone(),
            state_path: state_path.clone(),
            runtime_dir: Some(newer_runtime),
            config_hash: newer_state.config_hash,
            runtime_hash: newer_state.runtime_hash,
        })
        .unwrap();

        let error =
            compute_config_state(&request_for(config_path, older_runtime, state_path)).unwrap_err();

        assert_eq!(error.class().as_str(), "runtime");
        assert_eq!(error.code(), "generated_state_runtime_downgrade");
    }

    // Defends: recording generated-state hashes never takes ownership of unmanaged config surfaces.
    #[test]
    fn records_only_the_managed_main_config_surface() {
        let dir = tempdir().expect("tempdir");
        let state_path = dir.path().join("state/rebuild_hash");
        let managed = dir.path().join("config/yazelix.toml");
        let unmanaged = dir.path().join("other/yazelix.toml");

        let skipped = record_config_state(&RecordConfigStateRequest {
            config_file: unmanaged.to_string_lossy().to_string(),
            managed_config_path: managed.clone(),
            state_path: state_path.clone(),
            runtime_dir: None,
            config_hash: "cfg".to_string(),
            runtime_hash: "runtime".to_string(),
        })
        .unwrap();
        assert!(!skipped.recorded);
        assert!(!state_path.exists());

        let recorded = record_config_state(&RecordConfigStateRequest {
            config_file: managed.to_string_lossy().to_string(),
            managed_config_path: managed,
            state_path: state_path.clone(),
            runtime_dir: None,
            config_hash: "cfg".to_string(),
            runtime_hash: "runtime".to_string(),
        })
        .unwrap();
        assert!(recorded.recorded);
        let stored = fs::read_to_string(state_path).unwrap();
        assert_eq!(
            serde_json::from_str::<JsonValue>(&stored).unwrap(),
            json!({
                "schema_version": MATERIALIZED_STATE_SCHEMA_VERSION,
                "materializer_schema_version": GENERATED_STATE_MATERIALIZER_SCHEMA_VERSION,
                "config_hash":"cfg",
                "runtime_hash":"runtime"
            })
        );
    }
}
