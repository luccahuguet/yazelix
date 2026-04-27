// Test lane: default
//! Versioned immutable config snapshots for live Yazelix windows.

use crate::bridge::{CoreError, ErrorClass};
use crate::session_facts::SessionFactsData;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const SESSION_CONFIG_SNAPSHOT_SCHEMA_VERSION: u64 = 1;
pub const SESSION_CONFIG_SNAPSHOT_FILE_NAME: &str = "config_snapshot.json";
pub const SESSION_CONFIG_SNAPSHOT_PATH_ENV: &str = "YAZELIX_SESSION_CONFIG_PATH";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionConfigSnapshotData {
    pub schema_version: u64,
    pub snapshot_id: String,
    pub created_at_unix_seconds: u64,
    pub source_config: SessionConfigSourceMetadata,
    pub runtime: SessionRuntimeMetadata,
    pub normalized_config: JsonMap<String, JsonValue>,
    pub facts: SessionFactsData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionConfigSourceMetadata {
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionRuntimeMetadata {
    pub dir: String,
    pub hash: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct SessionConfigSnapshotWriteRequest<'a> {
    pub path: &'a Path,
    pub snapshot_id: &'a str,
    pub source_config_file: &'a str,
    pub source_config_hash: &'a str,
    pub runtime_dir: &'a Path,
    pub runtime_hash: &'a str,
    pub runtime_version: &'a str,
    pub normalized_config: &'a JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SessionConfigSnapshotCreateRequest {
    pub state_dir: PathBuf,
    pub snapshot_id: String,
    pub source_config_file: String,
    pub source_config_hash: String,
    pub runtime_dir: PathBuf,
    pub runtime_hash: String,
    pub normalized_config: JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionConfigSnapshotWriteData {
    pub snapshot_path: String,
    pub snapshot_id: String,
    pub snapshot: SessionConfigSnapshotData,
}

pub fn session_config_snapshot_path(session_dir: &Path) -> PathBuf {
    session_dir.join(SESSION_CONFIG_SNAPSHOT_FILE_NAME)
}

pub fn session_config_snapshot_path_from_env() -> Option<PathBuf> {
    std::env::var(SESSION_CONFIG_SNAPSHOT_PATH_ENV)
        .ok()
        .and_then(|value| non_empty_string(value.as_str()))
        .map(PathBuf::from)
}

pub fn load_session_config_snapshot_from_env() -> Result<SessionConfigSnapshotData, CoreError> {
    let Some(path) = session_config_snapshot_path_from_env() else {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_session_config_snapshot",
            format!("{SESSION_CONFIG_SNAPSHOT_PATH_ENV} is not set for this Yazelix pane."),
            "Restart this Yazelix window so it inherits a launch-time config snapshot.",
            json!({ "env": SESSION_CONFIG_SNAPSHOT_PATH_ENV }),
        ));
    };
    load_session_config_snapshot_from_path(&path)
}

pub fn load_session_facts_from_snapshot_path(path: &Path) -> Result<SessionFactsData, CoreError> {
    load_session_config_snapshot_from_path(path).map(|snapshot| snapshot.facts)
}

pub fn load_session_config_snapshot_from_path(
    path: &Path,
) -> Result<SessionConfigSnapshotData, CoreError> {
    let raw = fs::read_to_string(path).map_err(|source| {
        CoreError::io(
            "session_config_snapshot_read",
            "Could not read the Yazelix session config snapshot.",
            "Restart this Yazelix window so it can create a fresh config snapshot.",
            path.to_string_lossy(),
            source,
        )
    })?;
    let snapshot = serde_json::from_str::<SessionConfigSnapshotData>(&raw).map_err(|source| {
        CoreError::classified(
            ErrorClass::Runtime,
            "session_config_snapshot_parse",
            format!(
                "Could not parse the Yazelix session config snapshot {}: {source}",
                path.display()
            ),
            "Restart this Yazelix window so it can create a fresh config snapshot.",
            json!({ "path": path.to_string_lossy() }),
        )
    })?;
    validate_session_config_snapshot(path, snapshot)
}

pub fn write_session_config_snapshot(
    request: &SessionConfigSnapshotWriteRequest<'_>,
) -> Result<SessionConfigSnapshotData, CoreError> {
    let snapshot = build_session_config_snapshot(request, unix_seconds()?);
    write_json_atomic(request.path, &snapshot)?;
    Ok(snapshot)
}

pub fn write_session_config_snapshot_for_launch(
    request: &SessionConfigSnapshotCreateRequest,
    runtime_version: &str,
) -> Result<SessionConfigSnapshotWriteData, CoreError> {
    validate_snapshot_id(&request.snapshot_id)?;
    let session_dir = request
        .state_dir
        .join("sessions")
        .join(&request.snapshot_id);
    let snapshot_path = session_config_snapshot_path(&session_dir);
    let snapshot = write_session_config_snapshot(&SessionConfigSnapshotWriteRequest {
        path: &snapshot_path,
        snapshot_id: &request.snapshot_id,
        source_config_file: &request.source_config_file,
        source_config_hash: &request.source_config_hash,
        runtime_dir: &request.runtime_dir,
        runtime_hash: &request.runtime_hash,
        runtime_version,
        normalized_config: &request.normalized_config,
    })?;
    Ok(SessionConfigSnapshotWriteData {
        snapshot_path: snapshot_path.to_string_lossy().to_string(),
        snapshot_id: snapshot.snapshot_id.clone(),
        snapshot,
    })
}

fn build_session_config_snapshot(
    request: &SessionConfigSnapshotWriteRequest<'_>,
    created_at_unix_seconds: u64,
) -> SessionConfigSnapshotData {
    SessionConfigSnapshotData {
        schema_version: SESSION_CONFIG_SNAPSHOT_SCHEMA_VERSION,
        snapshot_id: request.snapshot_id.trim().to_string(),
        created_at_unix_seconds,
        source_config: SessionConfigSourceMetadata {
            path: request.source_config_file.to_string(),
            hash: request.source_config_hash.to_string(),
        },
        runtime: SessionRuntimeMetadata {
            dir: request.runtime_dir.to_string_lossy().to_string(),
            hash: request.runtime_hash.to_string(),
            version: request.runtime_version.to_string(),
        },
        normalized_config: request.normalized_config.clone(),
        facts: SessionFactsData::from_normalized_config(request.normalized_config),
    }
}

fn validate_session_config_snapshot(
    path: &Path,
    snapshot: SessionConfigSnapshotData,
) -> Result<SessionConfigSnapshotData, CoreError> {
    if snapshot.schema_version != SESSION_CONFIG_SNAPSHOT_SCHEMA_VERSION {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "session_config_snapshot_schema",
            format!(
                "Unsupported Yazelix session config snapshot schema {} at {}.",
                snapshot.schema_version,
                path.display()
            ),
            "Restart this Yazelix window so it can create a fresh config snapshot.",
            json!({
                "path": path.to_string_lossy(),
                "expected_schema_version": SESSION_CONFIG_SNAPSHOT_SCHEMA_VERSION,
                "actual_schema_version": snapshot.schema_version,
            }),
        ));
    }
    if snapshot.snapshot_id.trim().is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "session_config_snapshot_identity",
            format!(
                "Yazelix session config snapshot {} is missing its snapshot id.",
                path.display()
            ),
            "Restart this Yazelix window so it can create a fresh config snapshot.",
            json!({ "path": path.to_string_lossy() }),
        ));
    }
    Ok(snapshot)
}

fn validate_snapshot_id(snapshot_id: &str) -> Result<(), CoreError> {
    let trimmed = snapshot_id.trim();
    if trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
    {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "invalid_session_config_snapshot_id",
            format!("Invalid Yazelix session config snapshot id: {snapshot_id:?}."),
            "Use a non-empty launch id without path separators.",
            json!({ "snapshot_id": snapshot_id }),
        ));
    }
    Ok(())
}

fn unix_seconds() -> Result<u64, CoreError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|source| {
            CoreError::classified(
                ErrorClass::Runtime,
                "session_config_snapshot_time",
                format!("System clock error while writing the Yazelix config snapshot: {source}"),
                "Check the system clock and retry the launch.",
                json!({}),
            )
        })
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "session_config_snapshot_mkdir",
                "Could not create the Yazelix session config snapshot directory.",
                "Check permissions under the Yazelix state directory, then retry.",
                parent.to_string_lossy(),
                source,
            )
        })?;
    }

    let raw = serde_json::to_string_pretty(value).map_err(|source| {
        CoreError::classified(
            ErrorClass::Internal,
            "session_config_snapshot_serialize",
            format!("Could not serialize the Yazelix session config snapshot: {source}"),
            "Report this as a Yazelix internal error.",
            json!({}),
        )
    })?;
    let temp_path = path.with_extension(format!("json.tmp.{}", std::process::id()));
    fs::write(&temp_path, raw).map_err(|source| {
        CoreError::io(
            "session_config_snapshot_write",
            "Could not write the Yazelix session config snapshot.",
            "Check permissions under the Yazelix state directory, then retry.",
            temp_path.to_string_lossy(),
            source,
        )
    })?;
    fs::rename(&temp_path, path).map_err(|source| {
        CoreError::io(
            "session_config_snapshot_replace",
            "Could not replace the Yazelix session config snapshot.",
            "Check permissions under the Yazelix state directory, then retry.",
            path.to_string_lossy(),
            source,
        )
    })
}

fn non_empty_string(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::tempdir;

    fn write_request<'a>(
        path: &'a Path,
        config: &'a JsonMap<String, JsonValue>,
    ) -> SessionConfigSnapshotWriteRequest<'a> {
        SessionConfigSnapshotWriteRequest {
            path,
            snapshot_id: "launch-123",
            source_config_file: "/home/user/.config/yazelix/user_configs/yazelix.toml",
            source_config_hash: "cfg-hash",
            runtime_dir: Path::new("/nix/store/yazelix"),
            runtime_hash: "runtime-hash",
            runtime_version: "v16.1",
            normalized_config: config,
        }
    }

    // Defends: live windows get a complete immutable config snapshot, not only a narrow facts cache.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn session_config_snapshot_roundtrips_full_config_and_facts_projection() {
        let dir = tempdir().unwrap();
        let path = session_config_snapshot_path(dir.path());
        let config = JsonMap::from_iter([
            ("editor_command".to_string(), json!("nvim")),
            ("popup_program".to_string(), json!(["gitui"])),
            ("default_shell".to_string(), json!("bash")),
            ("terminals".to_string(), json!(["ghostty", "wezterm"])),
        ]);

        let written = write_session_config_snapshot(&write_request(&path, &config)).unwrap();
        let loaded = load_session_config_snapshot_from_path(&path).unwrap();
        let facts = load_session_facts_from_snapshot_path(&path).unwrap();

        assert_eq!(
            written.schema_version,
            SESSION_CONFIG_SNAPSHOT_SCHEMA_VERSION
        );
        assert_eq!(loaded.snapshot_id, "launch-123");
        assert_eq!(
            loaded.source_config.path,
            "/home/user/.config/yazelix/user_configs/yazelix.toml"
        );
        assert_eq!(loaded.source_config.hash, "cfg-hash");
        assert_eq!(loaded.runtime.hash, "runtime-hash");
        assert_eq!(loaded.runtime.version, "v16.1");
        assert_eq!(loaded.normalized_config["editor_command"], json!("nvim"));
        assert_eq!(loaded.facts.popup_program, vec!["gitui"]);
        assert_eq!(facts.default_shell, "bash");
        assert_eq!(facts.terminals, vec!["ghostty", "wezterm"]);
    }

    // Defends: stale or corrupt snapshot files fail clearly instead of falling back to mutable user config.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn session_config_snapshot_loader_rejects_corrupt_and_wrong_schema_files() {
        let dir = tempdir().unwrap();
        let corrupt = dir.path().join("corrupt.json");
        fs::write(&corrupt, "{not json").unwrap();
        let corrupt_error = load_session_config_snapshot_from_path(&corrupt).unwrap_err();
        assert_eq!(corrupt_error.code(), "session_config_snapshot_parse");

        let wrong_schema = dir.path().join("wrong_schema.json");
        fs::write(
            &wrong_schema,
            json!({
                "schema_version": 999,
                "snapshot_id": "old",
                "created_at_unix_seconds": 1,
                "source_config": { "path": "/config", "hash": "cfg" },
                "runtime": { "dir": "/runtime", "hash": "runtime", "version": "v-old" },
                "normalized_config": {},
                "facts": SessionFactsData::default(),
            })
            .to_string(),
        )
        .unwrap();
        let schema_error = load_session_config_snapshot_from_path(&wrong_schema).unwrap_err();
        assert_eq!(schema_error.code(), "session_config_snapshot_schema");
        assert_eq!(
            schema_error.details()["expected_schema_version"],
            json!(SESSION_CONFIG_SNAPSHOT_SCHEMA_VERSION)
        );
    }

    // Defends: launch-time snapshot creation uses one state-scoped file per launch id and rejects path-like ids.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn session_config_snapshot_launch_writer_scopes_path_to_snapshot_id() {
        let dir = tempdir().unwrap();
        let config = JsonMap::from_iter([("default_shell".to_string(), json!("nu"))]);
        let request = SessionConfigSnapshotCreateRequest {
            state_dir: dir.path().to_path_buf(),
            snapshot_id: "launch-456".to_string(),
            source_config_file: "/config/yazelix.toml".to_string(),
            source_config_hash: "cfg".to_string(),
            runtime_dir: PathBuf::from("/runtime"),
            runtime_hash: "runtime".to_string(),
            normalized_config: config,
        };

        let data = write_session_config_snapshot_for_launch(&request, "v16.1").unwrap();

        assert_eq!(data.snapshot_id, "launch-456");
        assert_eq!(
            data.snapshot_path,
            dir.path()
                .join("sessions/launch-456/config_snapshot.json")
                .to_string_lossy()
        );
        assert_eq!(data.snapshot.runtime.version, "v16.1");

        let mut bad = request;
        bad.snapshot_id = "../escape".to_string();
        let error = write_session_config_snapshot_for_launch(&bad, "v16.1").unwrap_err();
        assert_eq!(error.code(), "invalid_session_config_snapshot_id");
    }
}
