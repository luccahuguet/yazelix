use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::OnceLock,
};

const DEFAULT_CONFIG: &str = "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n";
static TEST_NU: OnceLock<PathBuf> = OnceLock::new();

fn default_config(extra: &str) -> String {
    format!("{DEFAULT_CONFIG}{extra}")
}

pub fn write_config_home(config_home: &Path, contents: impl AsRef<[u8]>) -> PathBuf {
    fs::create_dir_all(config_home).unwrap();
    let config = config_home.join("config.toml");
    fs::write(&config, contents).unwrap();
    config
}

pub fn write_executable(path: &Path, contents: impl AsRef<[u8]>) {
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

pub fn set_test_nu(path: &Path) {
    assert!(
        path.is_absolute(),
        "test Nu path must be absolute: {}",
        path.display()
    );
    assert!(
        path.is_file(),
        "test Nu path does not exist: {}",
        path.display()
    );
    TEST_NU
        .set(path.to_path_buf())
        .unwrap_or_else(|_| panic!("test Nu path was initialized more than once"));
}

pub fn write_nu_executable(path: &Path, body: impl AsRef<str>) {
    let nu = TEST_NU
        .get()
        .expect("set_test_nu must run before creating fixtures");
    write_executable(
        path,
        format!("#!{} --no-config-file\n{}", nu.display(), body.as_ref()),
    );
}

pub fn successful_output(command: &mut Command, context: &str) -> Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{context} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

pub fn successful_stdout(command: &mut Command, context: &str) -> String {
    String::from_utf8_lossy(&successful_output(command, context).stdout).into_owned()
}

pub fn expect_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context} is missing {needle:?}\n{}",
        excerpt(haystack)
    );
}

pub fn expect_order(haystack: &str, needles: &[&str], context: &str) {
    let mut offset = 0;
    for needle in needles {
        let Some(index) = haystack[offset..].find(needle) else {
            panic!(
                "{context} is missing {needle:?} after byte {offset}\n{}",
                excerpt(haystack)
            );
        };
        offset += index + needle.len();
    }
}

pub fn excerpt(text: &str) -> String {
    const LIMIT: usize = 4000;
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(LIMIT).collect();
    let omitted = chars.count();
    if omitted == 0 {
        head
    } else {
        format!("{head}...\n[{omitted} chars omitted]")
    }
}

pub fn binary_text(path: &Path) -> String {
    String::from_utf8_lossy(&fs::read(path).unwrap()).into_owned()
}

pub fn embedded_store_path(text: &str, suffix: &str) -> PathBuf {
    let end = text
        .find(suffix)
        .unwrap_or_else(|| panic!("binary text is missing path suffix {suffix}"))
        + suffix.len();
    let start = text[..end]
        .rfind("/nix/store/")
        .unwrap_or_else(|| panic!("binary text is missing /nix/store path for {suffix}"));
    PathBuf::from(&text[start..end])
}

pub struct RuntimeCase {
    pub config_home: PathBuf,
    pub state_dir: PathBuf,
}

impl RuntimeCase {
    pub fn new(root: &Path, name: &str) -> Self {
        Self {
            config_home: root.join(format!("{name}-config")),
            state_dir: root.join(format!("{name}-state")),
        }
    }

    pub fn write_config(&self, contents: impl AsRef<[u8]>) -> PathBuf {
        write_config_home(&self.config_home, contents)
    }

    pub fn write_default_config(&self, extra: &str) -> PathBuf {
        self.write_config(default_config(extra))
    }

    pub fn yzx_command(&self, yzx_bin: &Path, command: &str) -> Command {
        let mut yzx = Command::new(yzx_bin);
        yzx.arg(command)
            .env("YAZELIX_CONFIG_HOME", &self.config_home)
            .env("YAZELIX_STATE_DIR", &self.state_dir)
            .env_remove("ZELLIJ_SESSION_NAME");
        yzx
    }

    pub fn run_yzx(&self, yzx_bin: &Path, command: &str, context: &str) -> String {
        successful_stdout(&mut self.yzx_command(yzx_bin, command), context)
    }
}

pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    pub fn new() -> Self {
        let mut path = env::temp_dir();
        path.push(format!("yzx-check-{}-{}", std::process::id(), unix_nanos()));
        fs::create_dir(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn unix_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
