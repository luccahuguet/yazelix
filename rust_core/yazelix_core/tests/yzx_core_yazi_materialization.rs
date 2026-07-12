// Test lane: maintainer

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use tempfile::{TempDir, tempdir};
use yazelix_core::{
    CoreError, YaziMaterializationData, YaziMaterializationRequest, generate_yazi_materialization,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn prepare_managed_config(config_root: &Path, repo: &Path) -> PathBuf {
    let config_path = config_root.join("config.toml");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::copy(repo.join("config_default.toml"), &config_path).unwrap();
    config_path
}

fn prepare_runtime_fixture(runtime_dir: &Path) {
    let yazi_dir = runtime_dir.join("configs").join("yazi");
    fs::create_dir_all(yazi_dir.join("plugins")).unwrap();
    fs::create_dir_all(yazi_dir.join("flavors")).unwrap();

    fs::write(
        yazi_dir.join("yazelix_starship.toml"),
        "# YAZELIX STARSHIP CONFIG FOR YAZI SIDEBAR\n",
    )
    .unwrap();

    for plugin in [
        "sidebar-status",
        "auto-layout",
        "sidebar-state",
        "git",
        "starship",
    ] {
        let plugin_dir = yazi_dir.join("plugins").join(format!("{plugin}.yazi"));
        fs::create_dir_all(&plugin_dir).unwrap();
        let body = if plugin == "auto-layout" {
            "return '__YAZELIX_RUNTIME_DIR__/libexec/yzx_control zellij open-editor-cwd'\n"
        } else {
            "return 'ok'\n"
        };
        fs::write(plugin_dir.join("main.lua"), body).unwrap();
    }

    let tokyo_night = yazi_dir.join("flavors").join("tokyo-night.yazi");
    fs::create_dir_all(&tokyo_night).unwrap();
    fs::write(tokyo_night.join("flavor.toml"), "[mgr]\n").unwrap();
}

fn run_yazi_materialization_generate(
    config_path: &Path,
    repo: &Path,
    runtime_dir: &Path,
    output_dir: &Path,
    sync_static_assets: bool,
) -> Result<YaziMaterializationData, CoreError> {
    let _guard = yazi_materialization_env_lock().lock().unwrap();
    let previous_config_dir = std::env::var_os("YAZELIX_CONFIG_DIR");
    let config_dir = config_path
        .parent()
        .expect("managed settings path has a config dir");
    // Tests serialize calls before mutating process env for this control-plane boundary.
    unsafe {
        std::env::set_var("YAZELIX_CONFIG_DIR", config_dir);
    }

    let result = generate_yazi_materialization(&YaziMaterializationRequest {
        config_path: config_path.to_path_buf(),
        default_config_path: repo.join("config_default.toml"),
        contract_path: repo.join("config_metadata/main_config_contract.toml"),
        runtime_dir: runtime_dir.to_path_buf(),
        yazi_config_dir: output_dir.to_path_buf(),
        sync_static_assets,
    });

    match previous_config_dir {
        Some(value) => unsafe {
            std::env::set_var("YAZELIX_CONFIG_DIR", value);
        },
        None => unsafe {
            std::env::remove_var("YAZELIX_CONFIG_DIR");
        },
    }

    result
}

fn yazi_materialization_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct YaziMaterializationFixture {
    temp: TempDir,
    repo: PathBuf,
    config_root: PathBuf,
    output_dir: PathBuf,
    runtime_dir: PathBuf,
    config_path: PathBuf,
}

impl YaziMaterializationFixture {
    fn new() -> Self {
        let repo = repo_root();
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let config_root = home.join(".config").join("yazelix");
        let output_dir = temp.path().join("state").join("configs").join("yazi");
        let runtime_dir = temp.path().join("runtime");
        let config_path = prepare_managed_config(&config_root, &repo);
        prepare_runtime_fixture(&runtime_dir);

        Self {
            temp,
            repo,
            config_root,
            output_dir,
            runtime_dir,
            config_path,
        }
    }

    fn generate(&self, sync_static_assets: bool) -> Result<YaziMaterializationData, CoreError> {
        run_yazi_materialization_generate(
            &self.config_path,
            &self.repo,
            &self.runtime_dir,
            &self.output_dir,
            sync_static_assets,
        )
    }

    fn user_yazi_dir(&self) -> PathBuf {
        self.config_root.join("yazi")
    }
}

// Regression: the zoxide editor plugin must not bake a generated Nix store path to yzx_control into copied Lua assets.
#[test]
fn bundled_zoxide_editor_resolves_yzx_control_from_runtime_env() {
    let plugin =
        fs::read_to_string(repo_root().join("configs/yazi/plugins/zoxide-editor.yazi/main.lua"))
            .unwrap();

    assert!(plugin.contains(r#"os.getenv("YAZELIX_RUNTIME_DIR")"#));
    assert!(plugin.contains(r#""open-editor-cwd""#));
    assert!(!plugin.contains("__YAZELIX_RUNTIME_DIR__/libexec/yzx_control"));
}

// Regression: the sidebar state plugin runs under managed Yazi's graphics-filtered env and must restore the saved Zellij session before piping to the pane orchestrator.
#[test]
fn bundled_sidebar_state_plugin_restores_saved_zellij_session_for_pipe_commands() {
    let plugin =
        fs::read_to_string(repo_root().join("configs/yazi/plugins/sidebar-state.yazi/main.lua"))
            .unwrap();

    assert!(plugin.contains(r#"os.getenv("YAZELIX_ZELLIJ_SESSION_NAME")"#));
    assert!(plugin.contains(r#"command:env("ZELLIJ_SESSION_NAME", session_name)"#));
    assert!(plugin.contains(r#""register_sidebar_yazi_state""#));
}

// Defends: Yazi materialization Rust-owns the generated surface, bundled assets, and runtime placeholder rendering end-to-end.
#[test]
fn yazi_materialization_generate_writes_managed_surface_and_assets() {
    let fixture = YaziMaterializationFixture::new();

    let data = fixture.generate(true).unwrap();

    assert_eq!(data.resolved_theme, "default");
    assert_eq!(data.sort_by, "alphabetical");
    assert!(data.synced_static_assets);
    assert_eq!(data.missing_plugins, Vec::<String>::new());

    let yazi_toml = fs::read_to_string(fixture.output_dir.join("yazi.toml")).unwrap();
    let init_lua = fs::read_to_string(fixture.output_dir.join("init.lua")).unwrap();
    let runtime_placeholder_plugin = fs::read_to_string(
        fixture
            .output_dir
            .join("plugins")
            .join("auto-layout.yazi")
            .join("main.lua"),
    )
    .unwrap();

    assert!(yazi_toml.contains("[manager]"));
    assert!(yazi_toml.contains("sort_by = \"alphabetical\""));
    assert!(yazi_toml.contains(fixture.runtime_dir.to_string_lossy().as_ref()));
    assert!(yazi_toml.contains("yzx_control zellij open-editor %s"));
    assert!(yazi_toml.contains("url = \"*\""));
    assert!(yazi_toml.contains("group = \"git\""));
    assert!(!yazi_toml.contains("name = \"*\""));
    assert!(
        init_lua.contains(
            fixture
                .output_dir
                .join("yazelix_starship.toml")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert!(!runtime_placeholder_plugin.contains("__YAZELIX_RUNTIME_DIR__"));
    assert!(runtime_placeholder_plugin.contains(fixture.runtime_dir.to_string_lossy().as_ref()));
}

// Regression: warm Yazi materialization must repair stale bundled static assets without recopied assets on a clean no-op path.
#[test]
fn yazi_materialization_repairs_stale_bundled_assets_without_warm_recopy() {
    let fixture = YaziMaterializationFixture::new();

    fixture.generate(true).unwrap();

    let plugin_main = fixture
        .output_dir
        .join("plugins")
        .join("sidebar-state.yazi")
        .join("main.lua");
    let sentinel = fixture
        .output_dir
        .join("plugins")
        .join("sidebar-state.yazi")
        .join("warm_skip_sentinel");
    fs::write(&sentinel, "warm asset marker\n").unwrap();

    let warm = fixture.generate(false).unwrap();
    assert!(!warm.synced_static_assets);
    assert!(sentinel.exists());

    fs::write(&plugin_main, "return 'stale generated plugin'\n").unwrap();
    let repair = fixture.generate(false).unwrap();
    assert!(repair.synced_static_assets);
    assert_eq!(fs::read_to_string(plugin_main).unwrap(), "return 'ok'\n");
}

// Regression: Nix-packaged bundled plugin directories can be symlinks into the package source, and static sync must copy their real contents.
#[cfg(unix)]
#[test]
fn yazi_materialization_syncs_symlinked_bundled_plugin_dirs() {
    use std::os::unix::fs::symlink;

    let fixture = YaziMaterializationFixture::new();

    let source_plugins = fixture.runtime_dir.join("configs/yazi/plugins");
    let source_plugin = source_plugins.join("sidebar-state.yazi");
    let real_plugin = fixture
        .temp
        .path()
        .join("package_source/sidebar-state.yazi");
    fs::remove_dir_all(&source_plugin).unwrap();
    fs::create_dir_all(&real_plugin).unwrap();
    fs::write(
        real_plugin.join("main.lua"),
        "return 'current symlinked plugin'\n",
    )
    .unwrap();
    symlink(&real_plugin, &source_plugin).unwrap();

    let target_plugin = fixture.output_dir.join("plugins/sidebar-state.yazi");
    fs::create_dir_all(&target_plugin).unwrap();
    fs::write(target_plugin.join("main.lua"), "return 'stale plugin'\n").unwrap();

    fixture.generate(true).unwrap();

    assert_eq!(
        fs::read_to_string(target_plugin.join("main.lua")).unwrap(),
        "return 'current symlinked plugin'\n"
    );
}

// Regression: `yzx import yazi` places native plugin directories under Yazelix-managed config,
// and materialization preserves that native tree without a retired root plugin catalog.
#[test]
fn yazi_materialization_generate_copies_managed_user_plugins() {
    let fixture = YaziMaterializationFixture::new();

    let plugin_dir = fixture
        .user_yazi_dir()
        .join("plugins")
        .join("clipboard.yazi");
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(plugin_dir.join("main.lua"), "return {}\n").unwrap();

    let data = fixture.generate(false).unwrap();
    assert_eq!(data.missing_plugins, Vec::<String>::new());
    assert!(
        fixture
            .output_dir
            .join("plugins/clipboard.yazi/main.lua")
            .exists()
    );
}

// Regression: native Yazi array settings such as mgr.ratio replace Yazelix defaults instead of appending to them.
#[test]
fn yazi_materialization_generate_replaces_user_yazi_array_settings() {
    let fixture = YaziMaterializationFixture::new();
    fs::create_dir_all(fixture.user_yazi_dir()).unwrap();
    fs::write(
        fixture.user_yazi_dir().join("yazi.toml"),
        r#"[mgr]
ratio = [1, 4, 0]
"#,
    )
    .unwrap();

    fixture.generate(false).unwrap();
    let generated_yazi = fs::read_to_string(fixture.output_dir.join("yazi.toml")).unwrap();
    let parsed_yazi: toml::Value = toml::from_str(&generated_yazi).unwrap();
    assert_eq!(
        parsed_yazi
            .get("mgr")
            .and_then(|section| section.get("ratio"))
            .and_then(toml::Value::as_array),
        Some(&vec![1.into(), 4.into(), 0.into()])
    );
}

// Regression: user Yazi keymap sections that are absent from Yazelix's bundled base keymap still survive materialization.
#[test]
fn yazi_materialization_generate_preserves_user_keymap_sections_beyond_mgr() {
    let fixture = YaziMaterializationFixture::new();
    let user_yazi_dir = fixture.user_yazi_dir();
    fs::create_dir_all(&user_yazi_dir).unwrap();
    fs::write(
        user_yazi_dir.join("keymap.toml"),
        r#"
[[input.append_keymap]]
on = ["<Esc>"]
run = "close"
desc = "Close input"

[[input.prepend_keymap]]
on = ["<C-a>"]
run = "move -999"
desc = "Move to start"

[[cmp.append_keymap]]
on = ["<Tab>"]
run = "confirm"
desc = "Confirm completion"

[[cmp.prepend_keymap]]
on = ["<BackTab>"]
run = "arrow -1"
desc = "Previous completion"
"#,
    )
    .unwrap();

    fixture.generate(false).unwrap();
    let generated_keymap = fs::read_to_string(fixture.output_dir.join("keymap.toml")).unwrap();
    let parsed_keymap: toml::Value = toml::from_str(&generated_keymap).unwrap();

    assert_eq!(
        parsed_keymap
            .get("mgr")
            .and_then(|section| section.get("append_keymap"))
            .and_then(toml::Value::as_array)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        parsed_keymap
            .get("input")
            .and_then(|section| section.get("append_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("close")
    );
    assert_eq!(
        parsed_keymap
            .get("input")
            .and_then(|section| section.get("prepend_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("move -999")
    );
    assert_eq!(
        parsed_keymap
            .get("cmp")
            .and_then(|section| section.get("append_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("confirm")
    );
    assert_eq!(
        parsed_keymap
            .get("cmp")
            .and_then(|section| section.get("prepend_keymap"))
            .and_then(toml::Value::as_array)
            .and_then(|entries| entries.first())
            .and_then(|entry| entry.get("run"))
            .and_then(toml::Value::as_str),
        Some("arrow -1")
    );
}

// Defends: Yazi materialization rejects legacy override ownership instead of silently adopting configs/yazi/user.
#[test]
fn yazi_materialization_generate_rejects_legacy_override_surface() {
    let fixture = YaziMaterializationFixture::new();
    let legacy_override_dir = fixture
        .config_root
        .join("configs")
        .join("yazi")
        .join("user");
    fs::create_dir_all(&legacy_override_dir).unwrap();
    fs::write(legacy_override_dir.join("init.lua"), "return 'legacy'\n").unwrap();

    let error = fixture.generate(false).unwrap_err();
    assert_eq!(error.class().as_str(), "config");
    assert_eq!(error.code(), "legacy_yazi_user_override");
    let message = error.message();
    assert!(message.contains("yzx import yazi"));
    assert!(message.contains("~/.config/yazelix/"));
}

// Regression: old flat Yazi sidecars are not silently ignored after the managed Yazi home moved under ~/.config/yazelix/yazi/.
#[test]
fn yazi_materialization_generate_rejects_old_flat_yazi_sidecars() {
    let fixture = YaziMaterializationFixture::new();
    fs::write(fixture.config_root.join("yazi_keymap.toml"), "[mgr]\n").unwrap();

    let error = fixture.generate(false).unwrap_err();
    assert_eq!(error.code(), "flat_yazi_user_override");
    let message = error.message();
    assert!(message.contains("yazi_keymap.toml"));
    assert!(message.contains("~/.config/yazelix/yazi/"));
}
