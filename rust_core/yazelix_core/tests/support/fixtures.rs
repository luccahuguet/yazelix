use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

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
        repo.join("config_metadata/main_config_contract.toml"),
        runtime_dir.join("config_metadata/main_config_contract.toml"),
    )
    .unwrap();
    fs::write(runtime_dir.join(".taplo.toml"), "[format]\n").unwrap();
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

pub fn managed_config_fixture(raw_config: &str) -> ManagedConfigFixture {
    let repo = repo_root();
    let temp = TempDir::new().unwrap();
    let home_dir = temp.path().join("home");
    let runtime_dir = temp.path().join("runtime");
    let config_dir = home_dir.join(".config").join("yazelix");
    let state_dir = home_dir.join(".local").join("share").join("yazelix");
    let managed_config = config_dir.join("user_configs").join("yazelix.toml");

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
