use crate::user_config_paths;
use serde::Serialize;
use std::path::{Path, PathBuf};

const HOME_MANAGER_FILES_MARKER: &str = "-home-manager-files/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeConfigStatusCode {
    CanonicalSettings,
    ManagedDefault,
    ManagedOverride,
    ImportedOverride,
    NativeReadOnly,
    NativeAvailable,
    NativeMissing,
    NativeRequiredMissing,
    HomeManagerReadOnly,
    GeneratedRuntime,
    NativeUserOwned,
    NotInspected,
}

impl NativeConfigStatusCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CanonicalSettings => "canonical_settings",
            Self::ManagedDefault => "managed_default",
            Self::ManagedOverride => "managed_override",
            Self::ImportedOverride => "imported_override",
            Self::NativeReadOnly => "native_read_only",
            Self::NativeAvailable => "native_available",
            Self::NativeMissing => "native_missing",
            Self::NativeRequiredMissing => "native_required_missing",
            Self::HomeManagerReadOnly => "home_manager_read_only",
            Self::GeneratedRuntime => "generated_runtime",
            Self::NativeUserOwned => "native_user_owned",
            Self::NotInspected => "not_inspected",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::CanonicalSettings => "Canonical Yazelix settings",
            Self::ManagedDefault => "Yazelix default",
            Self::ManagedOverride => "Yazelix-managed override",
            Self::ImportedOverride => "Imported into Yazelix",
            Self::NativeReadOnly => "Native read-only source",
            Self::NativeAvailable => "Native config available to import",
            Self::NativeMissing => "Native config missing",
            Self::NativeRequiredMissing => "Required native config missing",
            Self::HomeManagerReadOnly => "Home Manager-managed",
            Self::GeneratedRuntime => "Generated runtime output",
            Self::NativeUserOwned => "User-owned native config",
            Self::NotInspected => "Not inspected",
        }
    }

    pub const fn doctor_severity(self) -> &'static str {
        match self {
            Self::NativeRequiredMissing => "error",
            Self::NativeReadOnly | Self::HomeManagerReadOnly => "warning",
            Self::CanonicalSettings | Self::ManagedOverride => "ok",
            _ => "info",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeConfigStatusEntry {
    pub surface: String,
    pub tool: String,
    pub description: String,
    pub status: String,
    pub label: String,
    pub active_path: Option<String>,
    pub managed_path: Option<String>,
    pub native_paths: Vec<String>,
    pub generated_path: Option<String>,
    pub allowed_action: String,
    pub read_only_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NativeConfigStatusRequest {
    pub xdg_config_home: PathBuf,
    pub config_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub state_dir: PathBuf,
    pub active_terminal: String,
    pub settings_home_manager_read_only: bool,
}

pub fn xdg_config_home_from_env(home_dir: &Path) -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| home_dir.join(".config"))
}

pub fn path_present(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok()
}

pub fn path_owned_by_home_manager(path: &Path) -> bool {
    std::fs::read_link(path)
        .ok()
        .map(|target| target.to_string_lossy().contains(HOME_MANAGER_FILES_MARKER))
        .unwrap_or(false)
}

pub fn classify_native_config_statuses(
    request: &NativeConfigStatusRequest,
) -> Vec<NativeConfigStatusEntry> {
    let mut entries = Vec::new();
    entries.push(settings_status(request));
    entries.extend(zellij_statuses(request));
    entries.extend(yazi_statuses(request));
    entries.extend(helix_statuses(request));
    entries.extend(terminal_statuses(request));
    entries.extend(shell_statuses(request));
    entries
}

pub fn highest_doctor_severity(entries: &[NativeConfigStatusEntry]) -> &'static str {
    if entries
        .iter()
        .filter_map(status_code_for_entry)
        .any(|status| status.doctor_severity() == "error")
    {
        return "error";
    }
    if entries
        .iter()
        .filter_map(status_code_for_entry)
        .any(|status| status.doctor_severity() == "warning")
    {
        return "warning";
    }
    if entries
        .iter()
        .filter_map(status_code_for_entry)
        .any(|status| status.doctor_severity() == "info")
    {
        return "info";
    }
    "ok"
}

pub fn status_code_for_entry(entry: &NativeConfigStatusEntry) -> Option<NativeConfigStatusCode> {
    all_status_codes()
        .iter()
        .copied()
        .find(|status| status.as_str() == entry.status)
}

fn all_status_codes() -> &'static [NativeConfigStatusCode] {
    &[
        NativeConfigStatusCode::CanonicalSettings,
        NativeConfigStatusCode::ManagedDefault,
        NativeConfigStatusCode::ManagedOverride,
        NativeConfigStatusCode::ImportedOverride,
        NativeConfigStatusCode::NativeReadOnly,
        NativeConfigStatusCode::NativeAvailable,
        NativeConfigStatusCode::NativeMissing,
        NativeConfigStatusCode::NativeRequiredMissing,
        NativeConfigStatusCode::HomeManagerReadOnly,
        NativeConfigStatusCode::GeneratedRuntime,
        NativeConfigStatusCode::NativeUserOwned,
        NativeConfigStatusCode::NotInspected,
    ]
}

fn entry(
    surface: impl Into<String>,
    tool: impl Into<String>,
    description: impl Into<String>,
    status: NativeConfigStatusCode,
) -> NativeConfigStatusEntry {
    NativeConfigStatusEntry {
        surface: surface.into(),
        tool: tool.into(),
        description: description.into(),
        status: status.as_str().to_string(),
        label: status.label().to_string(),
        active_path: None,
        managed_path: None,
        native_paths: Vec::new(),
        generated_path: None,
        allowed_action: "none".to_string(),
        read_only_reason: None,
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn path_strings(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|path| path_string(path)).collect()
}

fn settings_status(request: &NativeConfigStatusRequest) -> NativeConfigStatusEntry {
    let settings_path = user_config_paths::main_config(&request.config_dir);
    let mut status = if request.settings_home_manager_read_only {
        entry(
            "settings.main",
            "yazelix",
            "Canonical semantic settings source",
            NativeConfigStatusCode::HomeManagerReadOnly,
        )
    } else {
        entry(
            "settings.main",
            "yazelix",
            "Canonical semantic settings source",
            NativeConfigStatusCode::CanonicalSettings,
        )
    };
    status.active_path = Some(path_string(&settings_path));
    status.managed_path = Some(path_string(&settings_path));
    status.allowed_action = if request.settings_home_manager_read_only {
        "edit_home_manager".to_string()
    } else {
        "edit_managed".to_string()
    };
    status.read_only_reason = request
        .settings_home_manager_read_only
        .then(|| "Home Manager owns the active settings file".to_string());
    status
}

fn zellij_statuses(request: &NativeConfigStatusRequest) -> Vec<NativeConfigStatusEntry> {
    let managed = user_config_paths::zellij_config(&request.config_dir);
    let plugins = user_config_paths::zellij_plugins(&request.config_dir);
    let native = request.xdg_config_home.join("zellij").join("config.kdl");
    let generated = request
        .state_dir
        .join("configs")
        .join("zellij")
        .join("config.kdl");
    let mut input = if path_present(&managed) {
        entry(
            "zellij.input",
            "zellij",
            "Guarded Zellij preferences; present rows stay explicit in Nova unless removed",
            NativeConfigStatusCode::ManagedOverride,
        )
    } else {
        entry(
            "zellij.input",
            "zellij",
            "Zellij defaults plus Yazelix overlays",
            NativeConfigStatusCode::ManagedDefault,
        )
    };
    input.active_path = path_present(&managed).then(|| path_string(&managed));
    input.managed_path = Some(path_string(&managed));
    input.native_paths = vec![path_string(&native)];
    input.allowed_action = if path_present(&managed) {
        "edit_managed".to_string()
    } else {
        "create_or_import_native".to_string()
    };

    let mut plugin_input = entry(
        "zellij.plugins",
        "zellij",
        "Third-party Zellij plugins; a present file stays explicit in Nova",
        if path_present(&plugins) {
            NativeConfigStatusCode::ManagedOverride
        } else {
            NativeConfigStatusCode::ManagedDefault
        },
    );
    plugin_input.active_path = path_present(&plugins).then(|| path_string(&plugins));
    plugin_input.managed_path = Some(path_string(&plugins));
    plugin_input.allowed_action = "edit_managed".to_string();

    vec![
        input,
        plugin_input,
        generated_entry(
            "zellij.generated",
            "zellij",
            "Merged generated Zellij runtime config",
            generated,
        ),
    ]
}

fn yazi_statuses(request: &NativeConfigStatusRequest) -> Vec<NativeConfigStatusEntry> {
    let files = [
        (
            "yazi.config",
            "Yazi main override",
            user_config_paths::yazi_config(&request.config_dir),
            request.xdg_config_home.join("yazi").join("yazi.toml"),
        ),
        (
            "yazi.keymap",
            "Yazi keymap override",
            user_config_paths::yazi_keymap(&request.config_dir),
            request.xdg_config_home.join("yazi").join("keymap.toml"),
        ),
        (
            "yazi.init",
            "Yazi init.lua override",
            user_config_paths::yazi_init(&request.config_dir),
            request.xdg_config_home.join("yazi").join("init.lua"),
        ),
        (
            "yazi.package",
            "Yazi package manifest",
            user_config_paths::yazi_package(&request.config_dir),
            request.xdg_config_home.join("yazi").join("package.toml"),
        ),
        (
            "yazi.plugins",
            "Yazi plugin directory",
            user_config_paths::yazi_plugins_dir(&request.config_dir),
            request.xdg_config_home.join("yazi").join("plugins"),
        ),
        (
            "yazi.flavors",
            "Yazi flavor directory",
            user_config_paths::yazi_flavors_dir(&request.config_dir),
            request.xdg_config_home.join("yazi").join("flavors"),
        ),
    ];
    let mut entries = files
        .into_iter()
        .map(|(surface, description, managed, native)| {
            optional_managed_import_status("yazi", surface, description, managed, vec![native])
        })
        .collect::<Vec<_>>();
    entries.push(generated_entry(
        "yazi.generated",
        "yazi",
        "Generated Yazi runtime config directory",
        request.state_dir.join("configs").join("yazi"),
    ));
    entries
}

fn helix_statuses(request: &NativeConfigStatusRequest) -> Vec<NativeConfigStatusEntry> {
    let managed = user_config_paths::helix_config(&request.config_dir);
    let native = request.xdg_config_home.join("helix").join("config.toml");
    vec![
        optional_managed_import_status(
            "helix",
            "helix.input",
            "Managed Helix override",
            managed,
            vec![native],
        ),
        generated_entry(
            "helix.generated",
            "helix",
            "Generated managed Helix config",
            request
                .state_dir
                .join("configs")
                .join("helix")
                .join("config.toml"),
        ),
    ]
}

fn terminal_statuses(request: &NativeConfigStatusRequest) -> Vec<NativeConfigStatusEntry> {
    let terminal = request.active_terminal.as_str();
    let managed = user_config_paths::mars_config(&request.config_dir);
    let packaged = user_config_paths::packaged_mars_config(&request.runtime_dir);
    let managed_present = path_present(&managed);
    let home_manager = managed_present && path_owned_by_home_manager(&managed);
    let status = if home_manager {
        NativeConfigStatusCode::HomeManagerReadOnly
    } else if managed_present {
        NativeConfigStatusCode::ManagedOverride
    } else {
        NativeConfigStatusCode::ManagedDefault
    };
    let mut input = entry(
        format!("terminal.{terminal}.input"),
        terminal.to_string(),
        "Native Mars override",
        status,
    );
    input.active_path = Some(path_string(if managed_present {
        &managed
    } else {
        &packaged
    }));
    input.managed_path = Some(path_string(&managed));
    input.allowed_action = if home_manager {
        "edit_home_manager"
    } else {
        "edit_managed"
    }
    .to_string();
    input.read_only_reason = home_manager.then(|| {
        "The Mars override is owned by Home Manager; edit programs.yazelix.config.mars.".to_string()
    });
    vec![input]
}

fn shell_statuses(request: &NativeConfigStatusRequest) -> Vec<NativeConfigStatusEntry> {
    let shells = [
        ("bash", user_config_paths::SHELL_BASH_HOOK),
        ("zsh", user_config_paths::SHELL_ZSH_HOOK),
        ("fish", user_config_paths::SHELL_FISH_HOOK),
        ("nu", user_config_paths::SHELL_NU_HOOK),
        ("xonsh", user_config_paths::SHELL_XONSH_HOOK),
    ];
    let mut entries = shells
        .into_iter()
        .map(|(shell, file)| {
            let managed = request.config_dir.join(file);
            let mut status = if path_present(&managed) {
                entry(
                    format!("shell.{shell}.hook"),
                    shell,
                    "Yazelix-managed shell hook",
                    NativeConfigStatusCode::ManagedOverride,
                )
            } else {
                entry(
                    format!("shell.{shell}.hook"),
                    shell,
                    "Default Yazelix shell behavior",
                    NativeConfigStatusCode::ManagedDefault,
                )
            };
            status.active_path = path_present(&managed).then(|| path_string(&managed));
            status.managed_path = Some(path_string(&managed));
            status.allowed_action = "edit_managed".to_string();
            status
        })
        .collect::<Vec<_>>();
    entries.push(entry(
        "shell.native_rc",
        "shell",
        "User shell rc files are not inspected or imported",
        NativeConfigStatusCode::NotInspected,
    ));
    entries
}

fn optional_managed_import_status(
    tool: &str,
    surface: &str,
    description: &str,
    managed: PathBuf,
    native_paths: Vec<PathBuf>,
) -> NativeConfigStatusEntry {
    let native_existing = native_paths.iter().find(|path| path.exists()).cloned();
    let mut status = if path_present(&managed) {
        entry(
            surface,
            tool,
            description,
            NativeConfigStatusCode::ManagedOverride,
        )
    } else if native_existing.is_some() {
        entry(
            surface,
            tool,
            description,
            NativeConfigStatusCode::NativeAvailable,
        )
    } else {
        entry(
            surface,
            tool,
            description,
            NativeConfigStatusCode::ManagedDefault,
        )
    };
    status.active_path = path_present(&managed).then(|| path_string(&managed));
    status.managed_path = Some(path_string(&managed));
    status.native_paths = path_strings(&native_paths);
    status.allowed_action = match status.status.as_str() {
        "managed_override" => "edit_managed".to_string(),
        "native_available" => "import_native".to_string(),
        _ => "edit_managed".to_string(),
    };
    status
}

fn generated_entry(
    surface: impl Into<String>,
    tool: impl Into<String>,
    description: impl Into<String>,
    path: PathBuf,
) -> NativeConfigStatusEntry {
    let mut status = entry(
        surface,
        tool,
        description,
        NativeConfigStatusCode::GeneratedRuntime,
    );
    status.generated_path = Some(path_string(&path));
    status.active_path = Some(path_string(&path));
    status.allowed_action = "open_read_only".to_string();
    status.read_only_reason = Some("Generated runtime output should not be edited directly".into());
    status
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn request(tmp: &TempDir) -> NativeConfigStatusRequest {
        NativeConfigStatusRequest {
            xdg_config_home: tmp.path().join("home").join(".config"),
            config_dir: tmp.path().join("config").join("yazelix"),
            runtime_dir: tmp.path().join("runtime"),
            state_dir: tmp.path().join("state"),
            active_terminal: "mars".to_string(),
            settings_home_manager_read_only: false,
        }
    }

    fn find<'a>(
        entries: &'a [NativeConfigStatusEntry],
        surface: &str,
    ) -> &'a NativeConfigStatusEntry {
        entries
            .iter()
            .find(|entry| entry.surface == surface)
            .unwrap_or_else(|| panic!("missing status surface {surface}"))
    }

    // Defends: plain Zellij config remains an explicit import source and is never selected implicitly.
    #[test]
    fn zellij_native_config_is_not_an_active_fallback() {
        let tmp = TempDir::new().unwrap();
        let req = request(&tmp);
        let native = req.xdg_config_home.join("zellij").join("config.kdl");
        fs::create_dir_all(native.parent().unwrap()).unwrap();
        fs::write(&native, "keybinds {}\n").unwrap();

        let entries = classify_native_config_statuses(&req);
        let zellij = find(&entries, "zellij.input");

        assert_eq!(zellij.status, "managed_default");
        assert_eq!(zellij.active_path, None);
        assert_eq!(zellij.native_paths, vec![path_string(&native)]);
        assert_eq!(zellij.allowed_action, "create_or_import_native");
    }

    // Defends: absent user Mars override reports the packaged base as the active default.
    #[test]
    fn absent_mars_config_reports_packaged_default() {
        let tmp = TempDir::new().unwrap();
        let req = request(&tmp);

        let entries = classify_native_config_statuses(&req);
        let terminal = find(&entries, "terminal.mars.input");

        assert_eq!(terminal.status, "managed_default");
        assert!(terminal.generated_path.is_none());
        assert!(
            terminal
                .active_path
                .as_deref()
                .is_some_and(|path| path.ends_with("share/mars/config.toml"))
        );
    }

    // Defends: a Home Manager-installed Mars override is reported as the active read-only owner.
    #[cfg(unix)]
    #[test]
    fn home_manager_mars_config_is_active_and_read_only() {
        let tmp = TempDir::new().unwrap();
        let req = request(&tmp);
        let managed = user_config_paths::mars_config(&req.config_dir);
        let store = tmp
            .path()
            .join("profile-home-manager-files")
            .join("mars-config.toml");
        fs::create_dir_all(store.parent().unwrap()).unwrap();
        fs::create_dir_all(managed.parent().unwrap()).unwrap();
        fs::write(&store, "[window]\nopacity = 0.5\n").unwrap();
        std::os::unix::fs::symlink(&store, &managed).unwrap();

        let entries = classify_native_config_statuses(&req);
        let terminal = find(&entries, "terminal.mars.input");

        assert_eq!(terminal.status, "home_manager_read_only");
        assert_eq!(
            terminal.active_path.as_deref(),
            Some(path_string(&managed).as_str())
        );
        assert_eq!(terminal.allowed_action, "edit_home_manager");
    }

    // Defends: native Yazi and Helix files are import candidates only; Yazelix does not silently read them as runtime input.
    #[test]
    fn native_yazi_and_helix_configs_are_available_to_import() {
        let tmp = TempDir::new().unwrap();
        let req = request(&tmp);
        let yazi = req.xdg_config_home.join("yazi").join("yazi.toml");
        let yazi_package = req.xdg_config_home.join("yazi").join("package.toml");
        let yazi_flavors = req.xdg_config_home.join("yazi").join("flavors");
        let helix = req.xdg_config_home.join("helix").join("config.toml");
        fs::create_dir_all(yazi.parent().unwrap()).unwrap();
        fs::create_dir_all(&yazi_flavors).unwrap();
        fs::create_dir_all(helix.parent().unwrap()).unwrap();
        fs::write(&yazi, "[manager]\n").unwrap();
        fs::write(&yazi_package, "[plugin]\n").unwrap();
        fs::write(&helix, "[editor]\n").unwrap();

        let entries = classify_native_config_statuses(&req);

        assert_eq!(find(&entries, "yazi.config").status, "native_available");
        assert_eq!(find(&entries, "yazi.package").status, "native_available");
        assert_eq!(find(&entries, "yazi.flavors").status, "native_available");
        assert_eq!(find(&entries, "helix.input").status, "native_available");
    }

    // Defends: host-owned xonsh still exposes Yazelix's managed xonsh hook surface.
    #[test]
    fn xonsh_shell_hook_is_reported_as_managed_shell_surface() {
        let tmp = TempDir::new().unwrap();
        let req = request(&tmp);
        let hook = req.config_dir.join(user_config_paths::SHELL_XONSH_HOOK);
        fs::create_dir_all(hook.parent().unwrap()).unwrap();
        fs::write(&hook, "# xonsh hook\n").unwrap();

        let entries = classify_native_config_statuses(&req);
        let xonsh = find(&entries, "shell.xonsh.hook");

        assert_eq!(xonsh.status, "managed_override");
        assert_eq!(
            xonsh.active_path.as_deref(),
            Some(path_string(&hook).as_str())
        );
        assert_eq!(xonsh.allowed_action, "edit_managed");
    }

    // Defends: declarative settings ownership uses the shared contract label consumed by both doctor and config UI.
    #[test]
    fn home_manager_settings_are_read_only() {
        let tmp = TempDir::new().unwrap();
        let mut req = request(&tmp);
        req.settings_home_manager_read_only = true;

        let entries = classify_native_config_statuses(&req);
        let settings = find(&entries, "settings.main");

        assert_eq!(settings.status, "home_manager_read_only");
        assert_eq!(settings.label, "Home Manager-managed");
        assert_eq!(
            settings.read_only_reason.as_deref(),
            Some("Home Manager owns the active settings file")
        );
    }
}
