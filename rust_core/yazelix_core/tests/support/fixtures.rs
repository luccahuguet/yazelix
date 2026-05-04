use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use yazelix_core::active_config_surface::TOML_TOOLING_CONFIG_FILENAME;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct ManagedConfigFixture {
    pub _temp: TempDir,
    pub home_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub state_dir: PathBuf,
    pub managed_config: PathBuf,
}

impl ManagedConfigFixture {
    pub fn default_config_path(&self) -> PathBuf {
        self.runtime_dir.join("yazelix_default.toml")
    }

    pub fn contract_path(&self) -> PathBuf {
        self.runtime_dir
            .join("config_metadata")
            .join("main_config_contract.toml")
    }

    pub fn xdg_config_home(&self) -> PathBuf {
        self.home_dir.join(".config")
    }

    pub fn xdg_data_home(&self) -> PathBuf {
        self.home_dir.join(".local").join("share")
    }
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

pub fn write_runtime_contract_assets(repo: &Path, runtime_dir: &Path) {
    fs::create_dir_all(runtime_dir.join("config_metadata")).unwrap();
    fs::create_dir_all(runtime_dir.join("nushell/scripts/utils")).unwrap();
    fs::copy(
        repo.join("yazelix_default.toml"),
        runtime_dir.join("yazelix_default.toml"),
    )
    .unwrap();
    fs::copy(
        repo.join("yazelix_cursors_default.toml"),
        runtime_dir.join("yazelix_cursors_default.toml"),
    )
    .unwrap();
    fs::copy(
        repo.join("config_metadata/main_config_contract.toml"),
        runtime_dir.join("config_metadata/main_config_contract.toml"),
    )
    .unwrap();
    fs::write(runtime_dir.join(TOML_TOOLING_CONFIG_FILENAME), "[format]\n").unwrap();
    fs::write(
        runtime_dir.join("nushell/scripts/utils/constants.nu"),
        "export const YAZELIX_VERSION = \"v-test\"\n",
    )
    .unwrap();
}

pub fn write_executable_script(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
    #[cfg(unix)]
    {
        let permissions = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }
}

pub fn prepend_path(dir: &Path) -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    if current.is_empty() {
        dir.to_string_lossy().to_string()
    } else {
        format!("{}:{current}", dir.to_string_lossy())
    }
}

pub fn write_session_facts_cache(
    fixture: &ManagedConfigFixture,
    overrides: &[(&str, serde_json::Value)],
) -> PathBuf {
    let mut facts = serde_json::Map::from_iter([
        ("enable_sidebar".to_string(), serde_json::json!(true)),
        (
            "hide_sidebar_on_file_open".to_string(),
            serde_json::json!(false),
        ),
        ("yazi_command".to_string(), serde_json::json!("yazi")),
        ("ya_command".to_string(), serde_json::json!("ya")),
        ("popup_program".to_string(), serde_json::json!(["lazygit"])),
        ("popup_width_percent".to_string(), serde_json::json!(90)),
        ("popup_height_percent".to_string(), serde_json::json!(90)),
        (
            "game_of_life_cell_style".to_string(),
            serde_json::json!("full_block"),
        ),
        ("default_shell".to_string(), serde_json::json!("nu")),
        ("terminals".to_string(), serde_json::json!(["ghostty"])),
    ]);
    for (key, value) in overrides {
        facts.insert((*key).to_string(), value.clone());
    }
    let cache = serde_json::json!({
        "schema_version": 1,
        "source_config_file": "test-cache",
        "normalized_config": {},
        "facts": facts,
    });
    let path = fixture.state_dir.join("sessions/test/session_facts.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, serde_json::to_string_pretty(&cache).unwrap()).unwrap();
    path
}

pub fn write_session_config_snapshot(
    fixture: &ManagedConfigFixture,
    overrides: &[(&str, serde_json::Value)],
) -> PathBuf {
    write_session_config_snapshot_with_id(fixture, "test", overrides)
}

pub fn write_session_config_snapshot_with_id(
    fixture: &ManagedConfigFixture,
    snapshot_id: &str,
    overrides: &[(&str, serde_json::Value)],
) -> PathBuf {
    let mut normalized_config = serde_json::Map::from_iter([
        ("enable_sidebar".to_string(), serde_json::json!(true)),
        (
            "hide_sidebar_on_file_open".to_string(),
            serde_json::json!(false),
        ),
        ("yazi_command".to_string(), serde_json::json!("yazi")),
        ("yazi_ya_command".to_string(), serde_json::json!("ya")),
        ("popup_program".to_string(), serde_json::json!(["lazygit"])),
        ("popup_width_percent".to_string(), serde_json::json!(90)),
        ("popup_height_percent".to_string(), serde_json::json!(90)),
        (
            "game_of_life_cell_style".to_string(),
            serde_json::json!("full_block"),
        ),
        ("default_shell".to_string(), serde_json::json!("nu")),
        ("terminals".to_string(), serde_json::json!(["ghostty"])),
    ]);
    let mut facts = serde_json::Map::from_iter([
        ("enable_sidebar".to_string(), serde_json::json!(true)),
        (
            "hide_sidebar_on_file_open".to_string(),
            serde_json::json!(false),
        ),
        ("yazi_command".to_string(), serde_json::json!("yazi")),
        ("ya_command".to_string(), serde_json::json!("ya")),
        ("popup_program".to_string(), serde_json::json!(["lazygit"])),
        ("popup_width_percent".to_string(), serde_json::json!(90)),
        ("popup_height_percent".to_string(), serde_json::json!(90)),
        (
            "game_of_life_cell_style".to_string(),
            serde_json::json!("full_block"),
        ),
        ("default_shell".to_string(), serde_json::json!("nu")),
        ("terminals".to_string(), serde_json::json!(["ghostty"])),
    ]);
    for (key, value) in overrides {
        facts.insert((*key).to_string(), value.clone());
        let normalized_key = if *key == "ya_command" {
            "yazi_ya_command"
        } else {
            key
        };
        normalized_config.insert(normalized_key.to_string(), value.clone());
    }
    let snapshot = serde_json::json!({
        "schema_version": 1,
        "snapshot_id": snapshot_id,
        "created_at_unix_seconds": 1,
        "source_config": {
            "path": fixture.managed_config,
            "hash": "test-config-hash",
        },
        "runtime": {
            "dir": fixture.runtime_dir,
            "hash": "test-runtime-hash",
            "version": "v-test",
        },
        "normalized_config": normalized_config,
        "facts": facts,
    });
    let path = fixture
        .state_dir
        .join("sessions")
        .join(snapshot_id)
        .join("config_snapshot.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, serde_json::to_string_pretty(&snapshot).unwrap()).unwrap();
    path
}

pub fn managed_config_fixture(raw_config: &str) -> ManagedConfigFixture {
    let repo = repo_root();
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let runtime_dir = temp.path().join("runtime");
    let config_dir = home_dir.join(".config").join("yazelix");
    let state_dir = home_dir.join(".local").join("share").join("yazelix");
    let managed_config = config_dir.join("yazelix.toml");

    fs::create_dir_all(managed_config.parent().unwrap()).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir_all(&home_dir).unwrap();
    write_runtime_contract_assets(&repo, &runtime_dir);
    fs::write(&managed_config, raw_config).unwrap();

    ManagedConfigFixture {
        _temp: temp,
        home_dir,
        runtime_dir,
        config_dir,
        state_dir,
        managed_config,
    }
}
