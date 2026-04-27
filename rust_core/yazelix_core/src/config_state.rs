use crate::bridge::{CoreError, ErrorClass};
use crate::config_normalize::{NormalizeConfigRequest, normalize_config};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;

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
    },
}

pub fn compute_config_state(
    request: &ComputeConfigStateRequest,
) -> Result<ConfigStateData, CoreError> {
    let normalize_data = normalize_config(&NormalizeConfigRequest {
        config_path: request.config_path.clone(),
        default_config_path: request.default_config_path.clone(),
        contract_path: request.contract_path.clone(),
        include_missing: false,
    })?;
    let raw_config = read_toml_table(&request.config_path, "read_config")?;
    let contract = read_toml_table(&request.contract_path, "read_config_contract")?;
    let rebuild_paths = load_rebuild_required_paths(&contract);
    let rebuild_config = extract_rebuild_config(&raw_config, &rebuild_paths);
    let config_hash = sha256_hex(&toml::to_string(&rebuild_config).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "serialize_rebuild_config",
            format!("Could not serialize rebuild-required config: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?);
    let runtime_hash = sha256_hex(&path_to_string(&request.runtime_dir));
    let combined_hash = sha256_hex(&format!("{config_hash}:{runtime_hash}"));
    let cached_state = load_recorded_materialized_state(&request.state_path)?;

    let has_structured_cache = matches!(cached_state, CachedState::Structured { .. });
    let (cached_config_hash, cached_runtime_hash) = match &cached_state {
        CachedState::Structured {
            config_hash,
            runtime_hash,
        } => (config_hash.as_str(), runtime_hash.as_str()),
        CachedState::Missing => ("", ""),
    };

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

    let state = json!({
        "config_hash": request.config_hash,
        "runtime_hash": request.runtime_hash,
    });
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

fn load_rebuild_required_paths(contract: &toml::Table) -> Vec<String> {
    let mut paths = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .map(|fields| {
            fields
                .iter()
                .filter_map(|(path, field)| {
                    field
                        .as_table()
                        .and_then(|table| table.get("rebuild_required"))
                        .and_then(TomlValue::as_bool)
                        .filter(|required| *required)
                        .map(|_| path.clone())
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
    let digest = Sha256::digest(input.as_bytes());
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push_str(&format!("{byte:02x}"));
    }
    output
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
            default_config_path: repo.join("yazelix_default.toml"),
            contract_path: repo.join("config_metadata/main_config_contract.toml"),
            runtime_dir,
            state_path,
        }
    }

    fn write_user_config(dir: &Path, contents: &str) -> PathBuf {
        let path = dir.join("yazelix.toml");
        fs::write(&path, contents).expect("write config");
        path
    }

    // Invariant: config-state hashing stays stable for the default config when no prior state exists.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn computes_default_rebuild_hash_without_recorded_state() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let state_path = dir.path().join("state/rebuild_hash");
        let config_path = repo_root().join("yazelix_default.toml");
        let state = compute_config_state(&request_for(
            config_path,
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .unwrap();

        assert_eq!(
            state.config_hash,
            "cfba8d137ac98997cbf9437838509db79f49ea26e7e1f806b2a9a1da7580f7a8"
        );
        assert!(state.needs_refresh);
    }

    // Regression: malformed legacy state cache content must be treated as missing instead of trusted.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn treats_malformed_state_cache_as_missing() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let state_path = dir.path().join("state/rebuild_hash");

        fs::create_dir_all(state_path.parent().unwrap()).unwrap();
        fs::write(&state_path, "legacy-raw-hash-or-garbage").unwrap();

        let state = compute_config_state(&request_for(
            repo_root().join("yazelix_default.toml"),
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
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn ignores_non_rebuild_config_changes_but_flags_rebuild_changes() {
        let dir = tempdir().expect("tempdir");
        let runtime_dir = repo_root();
        let state_path = dir.path().join("state/rebuild_hash");
        let config_path = write_user_config(
            dir.path(),
            "[core]\nskip_welcome_screen = false\n\n[editor]\ncommand = \"hx\"\n\n[terminal]\nterminals = [\"ghostty\"]\n",
        );
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
            config_hash: baseline.config_hash.clone(),
            runtime_hash: baseline.runtime_hash.clone(),
        })
        .unwrap();

        fs::write(
            &config_path,
            "[core]\nskip_welcome_screen = true\n\n[editor]\ncommand = \"hx\"\n\n[terminal]\nterminals = [\"ghostty\"]\n",
        )
        .unwrap();
        let runtime_only = compute_config_state(&request_for(
            config_path.clone(),
            runtime_dir.clone(),
            state_path.clone(),
        ))
        .unwrap();
        assert_eq!(baseline.config_hash, runtime_only.config_hash);
        assert!(!runtime_only.needs_refresh);

        fs::write(
            &config_path,
            "[core]\nskip_welcome_screen = true\n\n[editor]\ncommand = \"nvim\"\n\n[terminal]\nterminals = [\"ghostty\"]\n",
        )
        .unwrap();
        let rebuild_changed =
            compute_config_state(&request_for(config_path, runtime_dir, state_path)).unwrap();
        assert_ne!(runtime_only.config_hash, rebuild_changed.config_hash);
        assert!(rebuild_changed.needs_refresh);
        assert_eq!(
            rebuild_changed.refresh_reason,
            "config changed since last generated-state repair"
        );
    }

    // Defends: recording generated-state hashes never takes ownership of unmanaged config surfaces.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn records_only_the_managed_main_config_surface() {
        let dir = tempdir().expect("tempdir");
        let state_path = dir.path().join("state/rebuild_hash");
        let managed = dir.path().join("config/user_configs/yazelix.toml");
        let unmanaged = dir.path().join("other/yazelix.toml");

        let skipped = record_config_state(&RecordConfigStateRequest {
            config_file: unmanaged.to_string_lossy().to_string(),
            managed_config_path: managed.clone(),
            state_path: state_path.clone(),
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
            config_hash: "cfg".to_string(),
            runtime_hash: "runtime".to_string(),
        })
        .unwrap();
        assert!(recorded.recorded);
        let stored = fs::read_to_string(state_path).unwrap();
        assert_eq!(
            serde_json::from_str::<JsonValue>(&stored).unwrap(),
            json!({"config_hash":"cfg","runtime_hash":"runtime"})
        );
    }
}
