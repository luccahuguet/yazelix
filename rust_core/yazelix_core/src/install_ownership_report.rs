//! Install ownership, launcher provenance, and doctor install-artifact classification.
//! Bead: yazelix-ulb2.4.1

use crate::config_state::compute_runtime_refresh_hash;
use crate::desktop_exec::{parse_env_assignment, split_desktop_exec_tokens};
use crate::terminal_variant::{SUPPORTED_TERMINALS, terminal_desktop_entry_file_name};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const HOME_MANAGER_FILES_MARKER: &str = "-home-manager-files/";
const MANUAL_DESKTOP_ICON_SIZES: &[&str] = &["48x48", "64x64", "128x128", "256x256"];
const RETIRED_TERMINAL_DESKTOP_ENTRY_TERMINALS: &[&str] =
    &["ghostty", "kitty", "rio", "wezterm", "foot", "ratty"];
pub const HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH: &str = "archive_path";
pub const HOME_MANAGER_PREPARE_ACTION_REMOVE_PROFILE_ENTRY: &str = "remove_profile_entry";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallOwnershipEvaluateRequest {
    pub runtime_dir: PathBuf,
    pub home_dir: PathBuf,
    #[serde(default)]
    pub user: Option<String>,
    pub xdg_config_home: PathBuf,
    pub xdg_data_home: PathBuf,
    pub yazelix_state_dir: PathBuf,
    pub main_config_path: PathBuf,
    #[serde(default)]
    pub invoked_yzx_path: Option<String>,
    #[serde(default)]
    pub redirected_from_stale_yzx_path: Option<String>,
    #[serde(default)]
    pub shell_resolved_yzx_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DoctorInstallResult {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub fix_available: bool,
}

impl DoctorInstallResult {
    fn new(status: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            message: message.into(),
            details: None,
            fix_available: false,
        }
    }

    fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HomeManagerPrepareArtifact {
    pub id: String,
    pub class: String,
    pub label: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StandaloneYazelixProfileEntry {
    pub name: String,
    pub remove_target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HomeManagerDesktopLauncher {
    pub terminal: String,
    pub path: String,
    pub launch_mode: String,
    pub active_runtime: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exec: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InstallOwnershipEvaluateData {
    pub install_owner: String,
    pub has_home_manager_managed_install: bool,
    pub is_manual_runtime_reference_path: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stable_yzx_wrapper: Option<String>,
    pub desktop_launcher_path: String,
    pub home_manager_profile_yzx_candidates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_home_manager_profile_yzx: Option<String>,
    pub home_manager_desktop_launchers: Vec<HomeManagerDesktopLauncher>,
    pub standalone_profile_yazelix_entries: Vec<StandaloneYazelixProfileEntry>,
    pub prepare_artifacts: Vec<HomeManagerPrepareArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_manager_profile_collision: Option<DoctorInstallResult>,
    pub install_owner_diagnostic: DoctorInstallResult,
    pub desktop_entry_freshness: DoctorInstallResult,
    pub wrapper_shadowing: Vec<DoctorInstallResult>,
}

pub fn evaluate_install_ownership_report(
    request: &InstallOwnershipEvaluateRequest,
) -> InstallOwnershipEvaluateData {
    let profile_candidates =
        home_manager_yzx_profile_paths(&request.home_dir, request.user.as_deref());
    let existing_profile = first_existing_profile_yzx(&profile_candidates);
    let has_hm = has_home_manager_managed_install(&request.main_config_path)
        || has_home_manager_profile_yazelix(&request.home_dir, existing_profile.as_ref());
    let stable = resolve_stable_yzx_wrapper_path(&request.home_dir, has_hm, &existing_profile);
    let desktop_launcher = resolve_desktop_launcher_path(&request.runtime_dir, stable.as_deref());
    let desktop_launcher_str = path_to_string(&desktop_launcher);
    let install_owner = detect_install_owner(has_hm, existing_profile.as_ref(), &request.home_dir);
    let home_manager_desktop_launchers =
        collect_home_manager_desktop_launchers(request, existing_profile.as_deref());
    let is_manual_runtime_ref = is_manual_runtime_reference_path(
        &request.yazelix_state_dir.join("runtime").join("current"),
    );
    let standalone_profile_yazelix_entries =
        collect_standalone_profile_yazelix_entries(&request.home_dir);
    let prepare_artifacts = collect_home_manager_prepare_artifacts(
        request,
        has_hm,
        &standalone_profile_yazelix_entries,
    );
    let home_manager_profile_collision =
        check_home_manager_profile_collision(has_hm, &standalone_profile_yazelix_entries);
    let install_owner_diagnostic = build_install_owner_diagnostic(
        request,
        &install_owner,
        stable.as_deref(),
        existing_profile.as_ref(),
        &desktop_launcher_str,
        &home_manager_desktop_launchers,
        is_manual_runtime_ref,
    );
    let desktop_entry_freshness = check_desktop_entry_freshness(
        request,
        &install_owner,
        &profile_candidates,
        &desktop_launcher_str,
    );
    let wrapper_shadowing = check_wrapper_shadowing(request, &existing_profile, &request.home_dir);

    InstallOwnershipEvaluateData {
        install_owner,
        has_home_manager_managed_install: has_hm,
        is_manual_runtime_reference_path: is_manual_runtime_ref,
        stable_yzx_wrapper: stable.map(|p| path_to_string(&p)),
        desktop_launcher_path: desktop_launcher_str,
        home_manager_profile_yzx_candidates: profile_candidates
            .iter()
            .map(path_to_string)
            .collect(),
        existing_home_manager_profile_yzx: existing_profile.map(|p| path_to_string(&p)),
        home_manager_desktop_launchers,
        standalone_profile_yazelix_entries,
        prepare_artifacts,
        home_manager_profile_collision,
        install_owner_diagnostic,
        desktop_entry_freshness,
        wrapper_shadowing,
    }
}

fn path_to_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().into_owned()
}

fn read_symlink_target(path: &Path) -> Option<PathBuf> {
    fs::read_link(path).ok()
}

fn is_home_manager_symlink_target(target: Option<&Path>) -> bool {
    let Some(t) = target else {
        return false;
    };
    let s = t.to_string_lossy();
    s.contains(HOME_MANAGER_FILES_MARKER)
}

pub(crate) fn has_home_manager_managed_install(main_config: &Path) -> bool {
    is_home_manager_symlink_target(read_symlink_target(main_config).as_deref())
}

fn home_manager_yzx_profile_paths(home_dir: &Path, user: Option<&str>) -> Vec<PathBuf> {
    let mut out = vec![home_dir.join(".nix-profile").join("bin").join("yzx")];
    if let Some(u) = user {
        let u = u.trim();
        if !u.is_empty() {
            let per_user = PathBuf::from("/etc/profiles/per-user")
                .join(u)
                .join("bin")
                .join("yzx");
            if !out.contains(&per_user) {
                out.push(per_user);
            }
        }
    }
    out
}

fn profile_path_exists_or_symlink(path: &Path) -> bool {
    path.exists() || is_symlink(path)
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

fn first_existing_profile_yzx(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates
        .iter()
        .find(|p| profile_path_exists_or_symlink(p))
        .cloned()
}

fn manual_yzx_wrapper_path(home_dir: &Path) -> PathBuf {
    home_dir.join(".local").join("bin").join("yzx")
}

fn resolve_stable_yzx_wrapper_path(
    home_dir: &Path,
    has_hm_managed: bool,
    existing_profile: &Option<PathBuf>,
) -> Option<PathBuf> {
    let manual = manual_yzx_wrapper_path(home_dir);
    if has_hm_managed {
        return existing_profile.clone();
    }
    if manual.exists() {
        return Some(manual);
    }
    existing_profile.clone()
}

fn get_runtime_yzx_cli_path(runtime_dir: &Path) -> PathBuf {
    let packaged = runtime_dir.join("bin").join("yzx");
    if packaged.exists() {
        packaged
    } else {
        runtime_dir.join("shells").join("posix").join("yzx_cli.sh")
    }
}

fn resolve_desktop_launcher_path(runtime_dir: &Path, stable: Option<&Path>) -> PathBuf {
    if let Some(s) = stable {
        s.to_path_buf()
    } else {
        get_runtime_yzx_cli_path(runtime_dir)
    }
}

fn detect_install_owner(
    has_hm_managed: bool,
    existing_profile: Option<&PathBuf>,
    home_dir: &Path,
) -> String {
    if has_hm_managed {
        return "home-manager".into();
    }
    if existing_profile.is_some() {
        return "profile".into();
    }
    let profile_apps = home_dir
        .join(".nix-profile")
        .join("share")
        .join("applications");
    if !existing_desktop_entry_paths(&profile_apps).is_empty() {
        return "profile".into();
    }
    "manual".into()
}

fn is_manual_runtime_reference_path(candidate: &Path) -> bool {
    let Some(target) = read_symlink_target(candidate) else {
        return false;
    };
    !is_home_manager_symlink_target(Some(target.as_path()))
}

fn symlink_target_looks_like_legacy_yazelix_wrapper(target: Option<&Path>) -> bool {
    let Some(t) = target else {
        return false;
    };
    let s = t.to_string_lossy();
    s.contains("yazelix-runtime") && s.ends_with("/bin/yzx")
}

fn file_contents_look_like_legacy_yazelix_wrapper(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    raw.contains("shells/posix/yzx_cli.sh")
        || raw.contains("Stable Yazelix CLI entrypoint for external tools and editors.")
        || (raw.contains("YAZELIX_BOOTSTRAP_RUNTIME_DIR") && raw.contains("Yazelix"))
}

fn is_legacy_manual_yzx_wrapper_path(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    if symlink_target_looks_like_legacy_yazelix_wrapper(read_symlink_target(path).as_deref()) {
        return true;
    }
    file_contents_look_like_legacy_yazelix_wrapper(path)
}

fn desktop_entry_file_names() -> Vec<String> {
    let mut names = vec![
        "com.yazelix.Yazelix.desktop".to_string(),
        "yazelix.desktop".to_string(),
    ];
    names.extend(
        SUPPORTED_TERMINALS
            .iter()
            .chain(RETIRED_TERMINAL_DESKTOP_ENTRY_TERMINALS.iter())
            .map(|terminal| terminal_desktop_entry_file_name(terminal)),
    );
    names.sort();
    names.dedup();
    names
}

fn desktop_entry_paths(apps_dir: &Path) -> Vec<PathBuf> {
    desktop_entry_file_names()
        .into_iter()
        .map(|name| apps_dir.join(name))
        .collect()
}

fn existing_desktop_entry_paths(apps_dir: &Path) -> Vec<PathBuf> {
    desktop_entry_paths(apps_dir)
        .into_iter()
        .filter(|path| path.exists())
        .collect()
}

fn existing_local_desktop_entry_paths(xdg_data_home: &Path) -> Vec<PathBuf> {
    existing_desktop_entry_paths(&xdg_data_home.join("applications"))
}

fn profile_applications_dir(home_dir: &Path) -> PathBuf {
    home_dir
        .join(".nix-profile")
        .join("share")
        .join("applications")
}

fn terminal_for_desktop_entry_path(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    for terminal in SUPPORTED_TERMINALS
        .iter()
        .chain(RETIRED_TERMINAL_DESKTOP_ENTRY_TERMINALS.iter())
    {
        if file_name == terminal_desktop_entry_file_name(terminal) {
            return (*terminal).to_string();
        }
    }
    if matches!(file_name, "com.yazelix.Yazelix.desktop" | "yazelix.desktop") {
        return "default".into();
    }
    "unknown".into()
}

fn exec_references_runtime_dir(exec: &str, runtime_dir: &Path) -> bool {
    let runtime = path_to_string(runtime_dir);
    !runtime.is_empty() && exec.contains(&runtime)
}

fn is_home_manager_direct_terminal_exec_for_runtime(exec: &str, runtime_dir: &Path) -> bool {
    exec.contains("YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1")
        && exec_references_runtime_dir(exec, runtime_dir)
        && exec.trim_end().ends_with(" desktop launch")
}

fn home_manager_desktop_launcher_mode(
    exec: Option<&str>,
    profile_yzx: Option<&Path>,
    runtime_dir: &Path,
) -> String {
    let Some(exec) = exec else {
        return "missing_exec".into();
    };
    if is_home_manager_direct_terminal_exec_for_runtime(exec, runtime_dir) {
        return "direct_runtime_package".into();
    }
    if let Some(profile_yzx) = profile_yzx {
        if exec.contains(&path_to_string(profile_yzx)) {
            return "primary_profile_wrapper".into();
        }
    }
    if exec.contains("YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1") {
        return "direct_runtime_package".into();
    }
    "profile_desktop_entry".into()
}

fn collect_home_manager_desktop_launchers(
    request: &InstallOwnershipEvaluateRequest,
    profile_yzx: Option<&Path>,
) -> Vec<HomeManagerDesktopLauncher> {
    let profile_apps = profile_applications_dir(&request.home_dir);
    let mut launchers = existing_desktop_entry_paths(&profile_apps)
        .into_iter()
        .map(|path| {
            let exec = get_desktop_entry_exec(&path);
            HomeManagerDesktopLauncher {
                terminal: terminal_for_desktop_entry_path(&path),
                launch_mode: home_manager_desktop_launcher_mode(
                    exec.as_deref(),
                    profile_yzx,
                    &request.runtime_dir,
                ),
                active_runtime: exec
                    .as_deref()
                    .is_some_and(|exec| exec_references_runtime_dir(exec, &request.runtime_dir)),
                path: path_to_string(&path),
                exec,
            }
        })
        .collect::<Vec<_>>();
    launchers.sort_by(|left, right| left.path.cmp(&right.path));
    launchers
}

fn is_manual_desktop_entry_path(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    if !raw.contains("Name=Yazelix") {
        return false;
    }
    if raw.contains("X-Yazelix-Managed=true") {
        return true;
    }
    raw.lines().any(|line| {
        let t = line.trim();
        t.starts_with("Exec=") && t.contains(" desktop launch")
    })
}

fn collect_manual_desktop_icon_artifacts(xdg_data_home: &Path) -> Vec<HomeManagerPrepareArtifact> {
    let mut out = Vec::new();
    for size in MANUAL_DESKTOP_ICON_SIZES {
        let icon_path = xdg_data_home
            .join("icons")
            .join("hicolor")
            .join(*size)
            .join("apps")
            .join("yazelix.png");
        if icon_path.exists() {
            out.push(HomeManagerPrepareArtifact {
                id: format!("desktop_icon_{size}"),
                class: "cleanup".into(),
                label: format!("manual desktop icon ({size})"),
                path: path_to_string(&icon_path),
                action: Some(HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH.into()),
                remove_target: None,
            });
        }
    }
    out
}

fn default_profile_manifest_paths(home_dir: &Path) -> Vec<PathBuf> {
    vec![
        home_dir.join(".nix-profile").join("manifest.json"),
        home_dir
            .join(".local")
            .join("state")
            .join("nix")
            .join("profiles")
            .join("profile")
            .join("manifest.json"),
    ]
}

fn read_default_profile_manifest(home_dir: &Path) -> Option<JsonValue> {
    default_profile_manifest_paths(home_dir)
        .into_iter()
        .find_map(|manifest_path| {
            let raw = fs::read_to_string(manifest_path).ok()?;
            serde_json::from_str::<JsonValue>(&raw).ok()
        })
}

fn default_profile_has_active_home_manager_path(home_dir: &Path) -> bool {
    let Some(parsed) = read_default_profile_manifest(home_dir) else {
        return false;
    };
    parsed
        .get("elements")
        .and_then(JsonValue::as_object)
        .and_then(|elements| elements.get("home-manager-path"))
        .is_some_and(|entry| {
            entry
                .get("active")
                .and_then(JsonValue::as_bool)
                .unwrap_or(true)
        })
}

fn has_home_manager_profile_yazelix(
    home_dir: &Path,
    existing_profile_yzx: Option<&PathBuf>,
) -> bool {
    if existing_profile_yzx.is_none() || !default_profile_has_active_home_manager_path(home_dir) {
        return false;
    }
    let profile_apps = home_dir
        .join(".nix-profile")
        .join("share")
        .join("applications");
    !existing_desktop_entry_paths(&profile_apps).is_empty()
}

fn is_yazelix_profile_entry(name: &str, entry: &JsonValue) -> bool {
    let attr_path = entry["attrPath"].as_str().unwrap_or("").trim();
    let original_url = entry["originalUrl"].as_str().unwrap_or("").trim();
    let resolved_url = entry["url"].as_str().unwrap_or("").trim();
    let store_paths = entry["storePaths"].as_array().cloned().unwrap_or_default();

    name.trim().starts_with("yazelix")
        || attr_path.split('.').any(|token| token == "yazelix")
        || original_url.contains("luccahuguet/yazelix")
        || resolved_url.contains("luccahuguet/yazelix")
        || store_paths.iter().any(|path| {
            let value = path.as_str().unwrap_or("").trim();
            value.contains("-yazelix-") || value.ends_with("-yazelix")
        })
}

fn read_standalone_profile_yazelix_entries(
    manifest_path: &Path,
) -> Vec<StandaloneYazelixProfileEntry> {
    let Ok(raw) = fs::read_to_string(manifest_path) else {
        return Vec::new();
    };
    let Ok(parsed) = serde_json::from_str::<JsonValue>(&raw) else {
        return Vec::new();
    };
    let Some(elements) = parsed.get("elements").and_then(JsonValue::as_object) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    for (name, entry) in elements {
        if entry
            .get("active")
            .and_then(JsonValue::as_bool)
            .is_some_and(|active| !active)
        {
            continue;
        }
        if !is_yazelix_profile_entry(name, entry) {
            continue;
        }

        let store_path = entry
            .get("storePaths")
            .and_then(JsonValue::as_array)
            .and_then(|paths| paths.first())
            .and_then(JsonValue::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        entries.push(StandaloneYazelixProfileEntry {
            name: name.trim().to_string(),
            remove_target: name.trim().to_string(),
            store_path,
        });
    }
    entries
}

fn collect_standalone_profile_yazelix_entries(
    home_dir: &Path,
) -> Vec<StandaloneYazelixProfileEntry> {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    for manifest_path in default_profile_manifest_paths(home_dir) {
        for entry in read_standalone_profile_yazelix_entries(&manifest_path) {
            if seen.insert(entry.remove_target.clone()) {
                entries.push(entry);
            }
        }
    }
    entries.sort_by(|left, right| left.remove_target.cmp(&right.remove_target));
    entries
}

fn collect_home_manager_prepare_artifacts(
    request: &InstallOwnershipEvaluateRequest,
    has_hm: bool,
    standalone_profile_yazelix_entries: &[StandaloneYazelixProfileEntry],
) -> Vec<HomeManagerPrepareArtifact> {
    let mut artifacts = Vec::new();
    let main = &request.main_config_path;
    if main.exists() && !has_hm {
        artifacts.push(HomeManagerPrepareArtifact {
            id: "main_config".into(),
            class: "blocker".into(),
            label: "managed settings.jsonc surface".into(),
            path: path_to_string(main),
            action: Some(HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH.into()),
            remove_target: None,
        });
    }
    for entry in standalone_profile_yazelix_entries {
        let path = entry
            .store_path
            .as_ref()
            .map(|store_path| {
                format!(
                    "default Nix profile entry `{}` -> {}",
                    entry.name, store_path
                )
            })
            .unwrap_or_else(|| format!("default Nix profile entry `{}`", entry.name));
        artifacts.push(HomeManagerPrepareArtifact {
            id: format!("standalone_profile_yazelix_{}", entry.name),
            class: "blocker".into(),
            label: "standalone default-profile Yazelix package".into(),
            path,
            action: Some(HOME_MANAGER_PREPARE_ACTION_REMOVE_PROFILE_ENTRY.into()),
            remove_target: Some(entry.remove_target.clone()),
        });
    }
    for desktop_entry in existing_local_desktop_entry_paths(&request.xdg_data_home)
        .into_iter()
        .filter(|path| is_manual_desktop_entry_path(path))
    {
        artifacts.push(HomeManagerPrepareArtifact {
            id: format!(
                "desktop_entry_{}",
                desktop_entry
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("unknown")
                    .replace('.', "_")
            ),
            class: "cleanup".into(),
            label: "manual desktop entry".into(),
            path: path_to_string(&desktop_entry),
            action: Some(HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH.into()),
            remove_target: None,
        });
    }
    artifacts.extend(collect_manual_desktop_icon_artifacts(
        &request.xdg_data_home,
    ));
    let manual_wrapper = manual_yzx_wrapper_path(&request.home_dir);
    if is_legacy_manual_yzx_wrapper_path(&manual_wrapper) {
        artifacts.push(HomeManagerPrepareArtifact {
            id: "manual_yzx_wrapper".into(),
            class: "cleanup".into(),
            label: "legacy ~/.local/bin/yzx wrapper".into(),
            path: path_to_string(&manual_wrapper),
            action: Some(HOME_MANAGER_PREPARE_ACTION_ARCHIVE_PATH.into()),
            remove_target: None,
        });
    }
    artifacts
}

fn check_home_manager_profile_collision(
    has_hm: bool,
    standalone_profile_yazelix_entries: &[StandaloneYazelixProfileEntry],
) -> Option<DoctorInstallResult> {
    if !has_hm || standalone_profile_yazelix_entries.is_empty() {
        return None;
    }

    let remove_targets = standalone_profile_yazelix_entries
        .iter()
        .map(|entry| entry.remove_target.as_str())
        .collect::<Vec<_>>();
    let details = format!(
        "Home Manager now owns this Yazelix install, but the default Nix profile still contains standalone Yazelix package entries.\nRemove them with `yzx home_manager prepare --apply` before the next `home-manager switch`, or run `nix profile remove {}` yourself.",
        remove_targets.join(" ")
    );
    Some(
        DoctorInstallResult::new(
            "warn",
            "The default Nix profile still contains standalone Yazelix packages alongside the Home Manager install",
        )
        .with_details(details),
    )
}

fn build_install_owner_diagnostic(
    request: &InstallOwnershipEvaluateRequest,
    install_owner: &str,
    stable_yzx_wrapper: Option<&Path>,
    existing_profile_yzx: Option<&PathBuf>,
    desktop_launcher: &str,
    home_manager_desktop_launchers: &[HomeManagerDesktopLauncher],
    is_manual_runtime_reference_path: bool,
) -> DoctorInstallResult {
    let mut details = vec![
        format!("Runtime root: {}", path_to_string(&request.runtime_dir)),
        format!("Desktop launcher target: {desktop_launcher}"),
    ];
    if let Some(wrapper) = stable_yzx_wrapper {
        details.push(format!("Stable yzx wrapper: {}", path_to_string(wrapper)));
    }
    if let Some(profile) = existing_profile_yzx {
        details.push(format!(
            "Profile yzx candidate: {}",
            path_to_string(profile)
        ));
    }
    if !home_manager_desktop_launchers.is_empty() {
        details.push("Home Manager desktop launchers:".into());
        for launcher in home_manager_desktop_launchers {
            let active = if launcher.active_runtime {
                " active runtime"
            } else {
                ""
            };
            details.push(format!(
                "  - {}: {} ({}){}",
                launcher.terminal, launcher.path, launcher.launch_mode, active
            ));
        }
    }
    if is_manual_runtime_reference_path {
        details.push(
            "Legacy runtime/current points at a manual runtime reference; reinstall into a profile or move to Home Manager before relying on owner updates."
                .into(),
        );
    }

    let (message, update_detail) = match install_owner {
        "home-manager" => (
            "Install owner: Home Manager",
            "Owner update command: `yzx update home_manager` from the owning Home Manager flake, then run the printed `home-manager switch` command.",
        ),
        "profile" => (
            "Install owner: default Nix profile",
            "Owner update command: `yzx update upstream`.",
        ),
        _ => (
            "Install owner: unmanaged runtime root",
            "No package owner was detected. Install with `nix profile add --refresh github:luccahuguet/yazelix#yazelix`, or enable the Home Manager module and use `yzx update home_manager` afterward.",
        ),
    };
    details.push(update_detail.into());

    DoctorInstallResult::new("info", message).with_details(details.join("\n"))
}

fn get_desktop_entry_exec(desktop_path: &Path) -> Option<String> {
    if !desktop_path.exists() {
        return None;
    }
    let Ok(raw) = fs::read_to_string(desktop_path) else {
        return None;
    };
    raw.lines()
        .find(|l| l.trim().starts_with("Exec="))
        .map(|l| {
            l.trim()
                .strip_prefix("Exec=")
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
}

fn get_desktop_entry_terminal_value(desktop_path: &Path) -> Option<String> {
    if !desktop_path.exists() {
        return None;
    }
    let Ok(raw) = fs::read_to_string(desktop_path) else {
        return None;
    };
    raw.lines()
        .find(|l| l.trim().starts_with("Terminal="))
        .map(|l| {
            l.trim()
                .strip_prefix("Terminal=")
                .unwrap_or("")
                .trim()
                .to_string()
        })
}

fn desktop_entry_terminal_enabled(desktop_path: &Path) -> bool {
    get_desktop_entry_terminal_value(desktop_path)
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false)
}

fn expected_desktop_entry_execs(
    install_owner: &str,
    profile_paths: &[PathBuf],
    desktop_launcher: &str,
) -> Vec<String> {
    let launcher_paths: Vec<PathBuf> = if install_owner == "home-manager" {
        profile_paths.to_vec()
    } else {
        vec![PathBuf::from(desktop_launcher)]
    };
    let mut out = Vec::new();
    for p in launcher_paths {
        let s = path_to_string(&p);
        out.push(format!("\"{s}\" desktop launch"));
        out.push(format!("{s} desktop launch"));
    }
    out.sort();
    out.dedup();
    out
}

fn desktop_entry_exec_matches_expected(
    exec: Option<&str>,
    expected: &[String],
    active_runtime_dir: &Path,
) -> bool {
    let Some(e) = exec else {
        return false;
    };
    expected.iter().any(|x| x == e)
        || desktop_exec_launcher_path(e).is_ok_and(|launcher| {
            let path = path_to_string(&launcher);
            let direct = format!("{path} desktop launch");
            let quoted = format!("\"{path}\" desktop launch");
            expected.iter().any(|x| x == &direct || x == &quoted)
        })
        || is_home_manager_direct_terminal_exec_for_runtime(e, active_runtime_dir)
}

fn desktop_exec_launcher_path(exec: &str) -> Result<PathBuf, String> {
    let tokens = split_desktop_exec_tokens(exec).map_err(|err| err.message().to_string())?;
    let mut index = 0;
    if tokens.first().map(String::as_str) == Some("env") {
        index = 1;
        while let Some(token) = tokens.get(index) {
            if parse_env_assignment(token).is_none() {
                break;
            }
            index += 1;
        }
    }
    let launcher = tokens
        .get(index)
        .ok_or_else(|| "Exec line does not contain a launcher path".to_string())?;
    let trailing = &tokens[index + 1..];
    if trailing != ["desktop", "launch"] {
        return Err("Exec line is not a supported `yzx desktop launch` command".to_string());
    }
    let launcher = PathBuf::from(launcher);
    if !launcher.is_absolute() {
        return Err("Exec launcher path is not absolute".to_string());
    }
    Ok(launcher)
}

fn runtime_dir_for_yzx_launcher(launcher: &Path) -> Option<PathBuf> {
    if launcher.file_name().and_then(|name| name.to_str()) != Some("yzx") {
        return None;
    }
    let bin_dir = launcher.parent()?;
    if bin_dir.file_name().and_then(|name| name.to_str()) != Some("bin") {
        return None;
    }
    bin_dir.parent().map(Path::to_path_buf)
}

fn validate_home_manager_profile_desktop_entry(
    exec: Option<&str>,
    expected: &[String],
    active_runtime_dir: &Path,
    active_runtime_hash: &str,
) -> Result<(), String> {
    let exec = exec.ok_or_else(|| "desktop entry has no Exec line".to_string())?;
    if desktop_entry_exec_matches_expected(Some(exec), expected, active_runtime_dir) {
        return Ok(());
    }

    let launcher = desktop_exec_launcher_path(exec)?;
    if fs::symlink_metadata(&launcher).is_err() {
        return Err(format!(
            "launcher path does not exist: {}",
            launcher.display()
        ));
    }
    let launcher_runtime_dir = runtime_dir_for_yzx_launcher(&launcher)
        .ok_or_else(|| "Exec launcher is not a packaged bin/yzx path".to_string())?;
    let launcher_runtime_hash = compute_runtime_refresh_hash(&launcher_runtime_dir)
        .map_err(|err| err.message().to_string())?;
    if launcher_runtime_hash == active_runtime_hash {
        return Ok(());
    }

    Err(format!(
        "launcher runtime {} does not match the active Home Manager runtime identity",
        launcher_runtime_dir.display()
    ))
}

fn stale_home_manager_profile_desktop_entries(
    request: &InstallOwnershipEvaluateRequest,
    install_owner: &str,
    profile_entries: &[PathBuf],
    expected: &[String],
) -> Vec<String> {
    if install_owner != "home-manager" {
        return Vec::new();
    }
    let active_runtime_hash = match compute_runtime_refresh_hash(&request.runtime_dir) {
        Ok(hash) => hash,
        Err(err) => {
            return vec![format!(
                "Active runtime {}: {}",
                request.runtime_dir.display(),
                err.message()
            )];
        }
    };

    profile_entries
        .iter()
        .filter_map(|path| {
            let exec = get_desktop_entry_exec(path);
            let mut reasons = Vec::new();
            if let Err(reason) = validate_home_manager_profile_desktop_entry(
                exec.as_deref(),
                expected,
                &request.runtime_dir,
                &active_runtime_hash,
            ) {
                reasons.push(reason);
            }
            if !desktop_entry_terminal_enabled(path) {
                let terminal = get_desktop_entry_terminal_value(path)
                    .unwrap_or_else(|| "<missing>".to_string());
                reasons.push(format!(
                    "Terminal={terminal} cannot show prelaunch failures"
                ));
            }
            if reasons.is_empty() {
                return None;
            }
            Some(format!(
                "{}\n  Exec: {}\n  Problem: {}",
                path_to_string(path),
                exec.as_deref().unwrap_or("<missing>"),
                reasons.join("; ")
            ))
        })
        .collect()
}

fn check_desktop_entry_freshness(
    request: &InstallOwnershipEvaluateRequest,
    install_owner: &str,
    profile_paths: &[PathBuf],
    desktop_launcher: &str,
) -> DoctorInstallResult {
    let local_entries = existing_local_desktop_entry_paths(&request.xdg_data_home);
    let profile_apps = profile_applications_dir(&request.home_dir);
    let profile_entries = existing_desktop_entry_paths(&profile_apps);
    let expected = expected_desktop_entry_execs(install_owner, profile_paths, desktop_launcher);
    let local_path = local_entries.first().cloned();
    let profile_path = profile_entries
        .iter()
        .find(|path| {
            get_desktop_entry_exec(path)
                .as_deref()
                .is_some_and(|exec| exec_references_runtime_dir(exec, &request.runtime_dir))
        })
        .cloned()
        .or_else(|| {
            profile_entries
                .iter()
                .find(|path| {
                    desktop_entry_exec_matches_expected(
                        get_desktop_entry_exec(path).as_deref(),
                        &expected,
                        &request.runtime_dir,
                    )
                })
                .cloned()
        })
        .or_else(|| profile_entries.first().cloned());
    let local_exists = local_path.is_some();
    let profile_exists = profile_path.is_some();
    let desktop_path: Option<PathBuf> = if local_exists {
        local_path.clone()
    } else if profile_exists {
        profile_path.clone()
    } else {
        None
    };
    let Some(dp) = desktop_path else {
        let details = if install_owner == "home-manager" {
            format!(
                "Home Manager-managed desktop entries usually resolve under {}. Reapply your Home Manager configuration if it is missing.",
                path_to_string(&profile_apps)
            )
        } else {
            "Run `yzx desktop install` if you want application-launcher integration.".into()
        };
        return DoctorInstallResult::new("info", "Yazelix desktop entry not installed")
            .with_details(details);
    };

    let local_exec = local_path.as_deref().and_then(get_desktop_entry_exec);
    let profile_exec = profile_path.as_deref().and_then(get_desktop_entry_exec);

    if install_owner == "home-manager"
        && local_exists
        && profile_exists
        && !desktop_entry_exec_matches_expected(
            local_exec.as_deref(),
            &expected,
            &request.runtime_dir,
        )
        && desktop_entry_exec_matches_expected(
            profile_exec.as_deref(),
            &expected,
            &request.runtime_dir,
        )
    {
        return DoctorInstallResult::new(
            "warning",
            "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry",
        )
        .with_details(format!(
                "Shadowing local entry: {}\nLocal Exec: {}\nHome Manager entry: {}\nProfile Exec: {}\nRemove the shadowing local entry with `yzx desktop uninstall`, then reapply your Home Manager configuration if the profile desktop entry is missing or stale.",
                path_to_string(local_path.as_ref().expect("local desktop path")),
                local_exec.as_deref().unwrap_or("<missing>"),
                path_to_string(profile_path.as_ref().expect("profile desktop path")),
                profile_exec.as_deref().unwrap_or("<missing>"),
            ));
    }

    let stale_profile_entries = stale_home_manager_profile_desktop_entries(
        request,
        install_owner,
        &profile_entries,
        &expected,
    );
    if !stale_profile_entries.is_empty() {
        return DoctorInstallResult::new(
            "warning",
            "Home Manager Yazelix desktop launcher paths are stale",
        )
        .with_details(format!(
            "{}\nRepair by reapplying your Home Manager configuration.",
            stale_profile_entries.join("\n")
        ));
    }

    let desktop_exec = if Some(&dp) == local_path.as_ref() {
        local_exec.clone()
    } else {
        profile_exec.clone()
    };
    let repair_hint = if Some(&dp) == profile_path.as_ref() {
        "Repair by reapplying your Home Manager configuration."
    } else {
        "Repair with `yzx desktop install`."
    };

    if desktop_exec.is_none() {
        return DoctorInstallResult::new("warning", "Yazelix desktop entry is invalid")
            .with_details(format!(
                "The installed desktop entry has no Exec line. {repair_hint}"
            ));
    }

    let de = desktop_exec.as_deref().unwrap();
    if !desktop_entry_exec_matches_expected(Some(de), &expected, &request.runtime_dir) {
        return DoctorInstallResult::new(
            "warning",
            "Yazelix desktop entry does not use the expected launcher path",
        )
        .with_details(format!(
            "Desktop entry Exec: {de}\nExpected one of: {}\n{repair_hint}",
            expected.join(", ")
        ));
    }

    if !desktop_entry_terminal_enabled(&dp) {
        let terminal_value =
            get_desktop_entry_terminal_value(&dp).unwrap_or_else(|| "<missing>".into());
        return DoctorInstallResult::new(
            "warning",
            "Yazelix desktop entry cannot show prelaunch failures",
        )
        .with_details(format!(
                "Desktop entry: {}\nTerminal: {}\nTerminal=false can hide config and generated-state errors that happen before the packaged terminal is spawned. Yazelix desktop entries should use Terminal=true as a starter window until a dedicated graphical prelaunch surface exists.\n{repair_hint}",
                path_to_string(&dp),
                terminal_value
            ));
    }

    DoctorInstallResult::new(
        "ok",
        "Yazelix desktop entry uses the expected launcher path",
    )
    .with_details(path_to_string(&dp))
}

fn absolutize_base(home_dir: &Path, path: &Path) -> PathBuf {
    let p = if path.is_absolute() {
        path.to_path_buf()
    } else {
        home_dir.join(path)
    };
    std::path::absolute(&p).unwrap_or(p)
}

fn resolved_invoked_or_shell(request: &InstallOwnershipEvaluateRequest) -> Option<PathBuf> {
    if let Some(s) = request.shell_resolved_yzx_path.as_deref() {
        let t = s.trim();
        if !t.is_empty() {
            return Some(absolutize_base(&request.home_dir, Path::new(t)));
        }
    }
    if let Some(s) = request.invoked_yzx_path.as_deref() {
        let t = s.trim();
        if !t.is_empty() {
            return Some(absolutize_base(&request.home_dir, Path::new(t)));
        }
    }
    None
}

fn check_wrapper_shadowing(
    request: &InstallOwnershipEvaluateRequest,
    existing_profile: &Option<PathBuf>,
    home_dir: &Path,
) -> Vec<DoctorInstallResult> {
    let mut out = Vec::new();
    let redirected_from = request
        .redirected_from_stale_yzx_path
        .as_deref()
        .unwrap_or("")
        .trim();
    if !redirected_from.is_empty() {
        if let Some(profile_wrapper) = existing_profile {
            let rf = absolutize_base(home_dir, Path::new(redirected_from));
            let pw = absolutize_base(home_dir, profile_wrapper.as_path());
            let details_lines = vec![
                format!("Stale host-shell invocation: {}", path_to_string(&rf)),
                format!("Current profile-owned yzx: {}", path_to_string(&pw)),
                "Yazelix redirected this invocation to the current profile command so the requested action could still run".into(),
                "A stale host-shell function or alias is still shadowing `yzx` in at least one shell startup file".into(),
                "Open a fresh shell after removing the old Yazelix-managed shell block, or bypass host-shell functions with `command yzx` until cleanup is complete".into(),
            ];
            out.push(
                DoctorInstallResult::new(
                    "warning",
                    "A stale host-shell yzx function or alias is shadowing the current profile command",
                )
                .with_details(details_lines.join("\n")),
            );
            return out;
        }
    }

    let manual_wrapper = manual_yzx_wrapper_path(home_dir);
    if !is_legacy_manual_yzx_wrapper_path(&manual_wrapper) {
        return out;
    }
    let Some(profile_wrapper) = existing_profile else {
        return out;
    };
    let Some(shell_resolved) = resolved_invoked_or_shell(request) else {
        return out;
    };
    let expanded_manual = absolutize_base(home_dir, &manual_wrapper);
    let expanded_profile = absolutize_base(home_dir, profile_wrapper.as_path());
    if shell_resolved != expanded_manual {
        return out;
    }

    let details_lines = vec![
        format!("Shell-resolved yzx: {}", path_to_string(&shell_resolved)),
        format!("Legacy local wrapper: {}", path_to_string(&expanded_manual)),
        format!("Profile-owned yzx: {}", path_to_string(&expanded_profile)),
        "Choose one clear owner for this install".into(),
        "If you are migrating to Home Manager, run `yzx home_manager prepare --apply`, then rerun `home-manager switch`".into(),
        "If a profile install owns this runtime, remove the stale `~/.local/bin/yzx` wrapper and keep the profile-owned `yzx` command".into(),
    ];
    out.push(
        DoctorInstallResult::new(
            "warning",
            "A stale user-local yzx wrapper shadows the profile-owned Yazelix command",
        )
        .with_details(details_lines.join("\n")),
    );
    out
}

// Test lane: default

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_default_profile_manifest(home_dir: &Path, raw: &str) {
        let manifest_path = home_dir.join(".nix-profile").join("manifest.json");
        std::fs::create_dir_all(manifest_path.parent().unwrap()).unwrap();
        std::fs::write(manifest_path, raw).unwrap();
    }

    fn test_request(
        tmp: &TempDir,
        home: &Path,
        xdg_data: &Path,
        main_config_path: PathBuf,
    ) -> InstallOwnershipEvaluateRequest {
        InstallOwnershipEvaluateRequest {
            runtime_dir: tmp.path().join("runtime"),
            home_dir: home.to_path_buf(),
            user: Some("test-user".into()),
            xdg_config_home: home.join(".config"),
            xdg_data_home: xdg_data.to_path_buf(),
            yazelix_state_dir: xdg_data.join("yazelix"),
            main_config_path,
            invoked_yzx_path: None,
            redirected_from_stale_yzx_path: None,
            shell_resolved_yzx_path: None,
        }
    }

    // Defends: Home Manager ownership detection still recognizes the store-symlink marker path.
    #[test]
    fn home_manager_ownership_detects_store_symlink_marker() {
        let tmp = TempDir::new().unwrap();
        let cfg = tmp.path().join("yazelix.toml");
        #[cfg(unix)]
        {
            let target = tmp.path().join("-home-manager-files").join("yazelix.toml");
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(&target, "").unwrap();
            std::os::unix::fs::symlink(&target, &cfg).unwrap();
        }
        #[cfg(not(unix))]
        {
            let _ = cfg;
            return;
        }
        assert!(has_home_manager_managed_install(&cfg));
    }

    // Regression: dangling Home Manager symlinks must still classify the install as Home Manager-owned.
    #[test]
    fn evaluate_install_ownership_keeps_home_manager_owner_for_dangling_main_config_symlink() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let main_config = home.join(".config/yazelix/yazelix.toml");
        let profile_yzx = home.join(".nix-profile/bin/yzx");

        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        std::fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
        std::fs::write(&profile_yzx, "#!/bin/sh\n").unwrap();

        #[cfg(unix)]
        {
            let dangling_target = tmp
                .path()
                .join("hm-marker")
                .join("-home-manager-files")
                .join("missing-yazelix.toml");
            std::os::unix::fs::symlink(&dangling_target, &main_config).unwrap();
        }
        #[cfg(not(unix))]
        {
            let _ = (&main_config, &xdg_data);
            return;
        }

        let report =
            evaluate_install_ownership_report(&test_request(&tmp, &home, &xdg_data, main_config));

        assert!(report.has_home_manager_managed_install);
        assert_eq!(report.install_owner, "home-manager");
        assert_eq!(
            report.existing_home_manager_profile_yzx,
            Some(path_to_string(&profile_yzx))
        );
        assert_eq!(
            report.stable_yzx_wrapper,
            Some(path_to_string(&profile_yzx))
        );
    }

    // Regression: profile candidate ordering must keep ~/.nix-profile ahead of /etc/profiles/per-user.
    #[test]
    fn home_manager_profile_candidates_preserve_home_profile_preference() {
        let home = PathBuf::from("/tmp/home");
        let candidates = home_manager_yzx_profile_paths(&home, Some("alice"));
        assert_eq!(
            candidates,
            vec![
                home.join(".nix-profile").join("bin").join("yzx"),
                PathBuf::from("/etc/profiles/per-user/alice/bin/yzx"),
            ]
        );
    }

    // Regression: a plain profile-owned yzx path must not be misclassified as Home Manager ownership.
    #[test]
    fn evaluate_install_ownership_classifies_plain_profile_install_as_profile() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let main_config = home.join(".config/yazelix/yazelix.toml");
        let profile_yzx = home.join(".nix-profile/bin/yzx");

        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        std::fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
        std::fs::write(&main_config, "[core]\n").unwrap();
        std::fs::write(&profile_yzx, "#!/bin/sh\n").unwrap();

        let report =
            evaluate_install_ownership_report(&test_request(&tmp, &home, &xdg_data, main_config));

        assert!(!report.has_home_manager_managed_install);
        assert_eq!(report.install_owner, "profile");
        assert_eq!(
            report.existing_home_manager_profile_yzx,
            Some(path_to_string(&profile_yzx))
        );
        assert_eq!(
            report.stable_yzx_wrapper,
            Some(path_to_string(&profile_yzx))
        );
        assert_eq!(
            report.install_owner_diagnostic.message,
            "Install owner: default Nix profile"
        );
        assert!(
            report
                .install_owner_diagnostic
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("yzx update upstream")
        );
    }

    // Regression: Home Manager owns the Yazelix runtime even when manage_config=false leaves settings.jsonc mutable.
    #[test]
    fn evaluate_install_ownership_detects_home_manager_profile_without_managed_config() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let main_config = home.join(".config/yazelix/settings.jsonc");
        let profile_yzx = home.join(".nix-profile/bin/yzx");
        let profile_desktop = home
            .join(".nix-profile/share/applications")
            .join("yazelix.desktop");

        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        std::fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
        std::fs::create_dir_all(profile_desktop.parent().unwrap()).unwrap();
        std::fs::write(&main_config, "{}\n").unwrap();
        std::fs::write(&profile_yzx, "#!/bin/sh\n").unwrap();
        std::fs::write(
            &profile_desktop,
            format!(
                "[Desktop Entry]\nName=Yazelix\nTerminal=true\nExec={} desktop launch\n",
                profile_yzx.display()
            ),
        )
        .unwrap();
        write_default_profile_manifest(
            &home,
            r#"{"elements":{"home-manager-path":{"active":true,"storePaths":["/nix/store/test-home-manager-path"]}},"version":3}"#,
        );

        let report =
            evaluate_install_ownership_report(&test_request(&tmp, &home, &xdg_data, main_config));

        assert!(!has_home_manager_managed_install(
            &home.join(".config/yazelix/settings.jsonc")
        ));
        assert!(report.has_home_manager_managed_install);
        assert_eq!(report.install_owner, "home-manager");
        assert_eq!(
            report.install_owner_diagnostic.message,
            "Install owner: Home Manager"
        );
    }

    // Regression: pre-terminal config failures are invisible from GUI launchers unless the desktop entry opens a starter terminal window.
    #[test]
    fn desktop_freshness_accepts_terminal_true_and_warns_on_terminal_false() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let apps = xdg_data.join("applications");
        let main_config = home.join(".config/yazelix/yazelix.toml");
        let profile_yzx = home.join(".nix-profile/bin/yzx");
        let desktop = apps.join("com.yazelix.Yazelix.desktop");

        std::fs::create_dir_all(&apps).unwrap();
        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        std::fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
        std::fs::write(&main_config, "[core]\n").unwrap();
        std::fs::write(&profile_yzx, "#!/bin/sh\n").unwrap();

        let exec = format!("\"{}\" desktop launch", profile_yzx.display());
        std::fs::write(
            &desktop,
            format!("[Desktop Entry]\nName=Yazelix\nTerminal=true\nExec={exec}\n"),
        )
        .unwrap();

        let request = test_request(&tmp, &home, &xdg_data, main_config);
        let fresh = evaluate_install_ownership_report(&request);
        assert_eq!(fresh.desktop_entry_freshness.status, "ok");

        std::fs::write(
            &desktop,
            format!(
                "[Desktop Entry]\nName=Yazelix\nTerminal=true\nExec=env SAMPLE_FLAG=1 {exec}\n"
            ),
        )
        .unwrap();
        let env_prefixed = evaluate_install_ownership_report(&request);
        assert_eq!(env_prefixed.desktop_entry_freshness.status, "ok");

        std::fs::write(
            &desktop,
            format!("[Desktop Entry]\nName=Yazelix\nTerminal=false\nExec={exec}\n"),
        )
        .unwrap();
        let stale = evaluate_install_ownership_report(&request);
        assert_eq!(stale.desktop_entry_freshness.status, "warning");
        assert_eq!(
            stale.desktop_entry_freshness.message,
            "Yazelix desktop entry cannot show prelaunch failures"
        );
    }

    // Defends: desktop freshness still warns when a user-local desktop entry shadows the profile-owned entry.
    #[test]
    fn desktop_freshness_warns_on_shadowing_local_desktop() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("h");
        let xdg = home.join(".local/share");
        std::fs::create_dir_all(xdg.join("applications")).unwrap();
        let profile_apps = home.join(".nix-profile/share/applications");
        std::fs::create_dir_all(&profile_apps).unwrap();
        let local_desktop = xdg.join("applications/com.yazelix.Yazelix.desktop");
        let profile_desktop = profile_apps.join("yazelix.desktop");
        let profile_yzx = home.join(".nix-profile/bin/yzx");
        std::fs::create_dir_all(profile_yzx.parent().unwrap()).unwrap();
        let mut f = std::fs::File::create(&profile_yzx).unwrap();
        f.write_all(b"#!/bin/sh\n").unwrap();

        let good_exec = format!("\"{}\" desktop launch", profile_yzx.display());
        std::fs::write(
            &profile_desktop,
            format!("[Desktop Entry]\nName=Yazelix\nTerminal=false\nExec={good_exec}\n"),
        )
        .unwrap();
        std::fs::write(
            &local_desktop,
            "[Desktop Entry]\nName=Yazelix\nTerminal=true\nExec=\"/old/bin/yzx\" desktop launch\n",
        )
        .unwrap();

        let main_config = home.join(".config/yazelix/yazelix.toml");
        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        #[cfg(unix)]
        {
            let hm_cfg_target = tmp
                .path()
                .join("hm-marker")
                .join("-home-manager-files")
                .join("yazelix.toml");
            std::fs::create_dir_all(hm_cfg_target.parent().unwrap()).unwrap();
            std::fs::write(&hm_cfg_target, "").unwrap();
            std::os::unix::fs::symlink(&hm_cfg_target, &main_config).unwrap();
        }

        let req = InstallOwnershipEvaluateRequest {
            runtime_dir: tmp.path().join("rt"),
            home_dir: home.clone(),
            user: None,
            xdg_config_home: home.join(".config"),
            xdg_data_home: xdg.clone(),
            yazelix_state_dir: xdg.join("yazelix"),
            main_config_path: main_config,
            invoked_yzx_path: None,
            redirected_from_stale_yzx_path: None,
            shell_resolved_yzx_path: None,
        };
        let r = evaluate_install_ownership_report(&req);
        assert_eq!(r.install_owner, "home-manager");
        assert_eq!(
            r.install_owner_diagnostic.message,
            "Install owner: Home Manager"
        );
        assert!(
            r.install_owner_diagnostic
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("yzx update home_manager")
        );
        assert_eq!(
            r.desktop_entry_freshness.message,
            "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry"
        );
    }

    // Regression: Home Manager prepare archives old generic and new variant-specific local desktop entries.
    #[test]
    fn prepare_artifacts_include_all_local_yazelix_desktop_entries() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let apps = xdg_data.join("applications");
        let main_config = home.join(".config/yazelix/yazelix.toml");
        std::fs::create_dir_all(&apps).unwrap();
        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        std::fs::write(&main_config, "[core]\n").unwrap();

        for name in [
            "com.yazelix.Yazelix.desktop",
            "yazelix.desktop",
            "com.yazelix.Yazelix.Ghostty.desktop",
        ] {
            std::fs::write(
                apps.join(name),
                "[Desktop Entry]\nName=Yazelix\nTerminal=false\nExec=\"/old/bin/yzx\" desktop launch\nX-Yazelix-Managed=true\n",
            )
            .unwrap();
        }

        let report =
            evaluate_install_ownership_report(&test_request(&tmp, &home, &xdg_data, main_config));

        let desktop_cleanup_paths = report
            .prepare_artifacts
            .iter()
            .filter(|artifact| artifact.label == "manual desktop entry")
            .map(|artifact| artifact.path.clone())
            .collect::<HashSet<_>>();
        assert!(
            desktop_cleanup_paths
                .contains(&path_to_string(&apps.join("com.yazelix.Yazelix.desktop")))
        );
        assert!(desktop_cleanup_paths.contains(&path_to_string(&apps.join("yazelix.desktop"))));
        assert!(desktop_cleanup_paths.contains(&path_to_string(
            &apps.join("com.yazelix.Yazelix.Ghostty.desktop")
        )));
    }

    // Regression: Home Manager prepare must surface standalone default-profile Yazelix entries as explicit removal blockers.
    #[test]
    fn prepare_artifacts_include_standalone_profile_yazelix_entry() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let main_config = home.join(".config/yazelix/yazelix.toml");
        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        std::fs::write(&main_config, "[core]\n").unwrap();
        write_default_profile_manifest(
            &home,
            r#"{"elements":{"yazelix":{"active":true,"storePaths":["/nix/store/test-yazelix"]}},"version":3}"#,
        );

        let report =
            evaluate_install_ownership_report(&test_request(&tmp, &home, &xdg_data, main_config));

        assert_eq!(report.standalone_profile_yazelix_entries.len(), 1);
        assert_eq!(
            report.standalone_profile_yazelix_entries[0].remove_target,
            "yazelix"
        );
        let artifact = report
            .prepare_artifacts
            .iter()
            .find(|artifact| artifact.remove_target.as_deref() == Some("yazelix"))
            .expect("standalone profile artifact");
        assert_eq!(
            artifact.action.as_deref(),
            Some(HOME_MANAGER_PREPARE_ACTION_REMOVE_PROFILE_ENTRY)
        );
        assert_eq!(artifact.class, "blocker");
    }

    // Defends: old mutable config inputs are stale-config diagnostics, not Home Manager takeover artifacts.
    #[test]
    fn prepare_artifacts_exclude_unsupported_old_config_inputs() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let config_dir = home.join(".config/yazelix");
        let main_config = config_dir.join("settings.jsonc");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(&main_config, "{}\n").unwrap();
        std::fs::write(config_dir.join("yazelix.toml"), "[core]\n").unwrap();
        std::fs::write(config_dir.join("cursors.toml"), "[cursor]\n").unwrap();

        let report =
            evaluate_install_ownership_report(&test_request(&tmp, &home, &xdg_data, main_config));

        let artifact_ids = report
            .prepare_artifacts
            .iter()
            .map(|artifact| artifact.id.as_str())
            .collect::<HashSet<_>>();
        assert!(artifact_ids.contains("main_config"));
        assert!(!artifact_ids.contains("old_main_config"));
        assert!(!artifact_ids.contains("old_cursor_config"));
    }

    // Regression: doctor ownership diagnostics must flag mixed Home Manager/profile installs instead of leaving the collision to Home Manager's package error.
    #[test]
    fn evaluate_install_ownership_reports_home_manager_profile_collision() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let xdg_data = home.join(".local/share");
        let main_config = home.join(".config/yazelix/yazelix.toml");
        std::fs::create_dir_all(main_config.parent().unwrap()).unwrap();
        write_default_profile_manifest(
            &home,
            r#"{"elements":{"yazelix":{"active":true,"storePaths":["/nix/store/test-yazelix"]}},"version":3}"#,
        );

        #[cfg(unix)]
        {
            let hm_cfg_target = tmp
                .path()
                .join("hm-marker")
                .join("-home-manager-files")
                .join("yazelix.toml");
            std::fs::create_dir_all(hm_cfg_target.parent().unwrap()).unwrap();
            std::fs::write(&hm_cfg_target, "").unwrap();
            std::os::unix::fs::symlink(&hm_cfg_target, &main_config).unwrap();
        }
        #[cfg(not(unix))]
        {
            let _ = (&main_config, &xdg_data);
            return;
        }

        let report =
            evaluate_install_ownership_report(&test_request(&tmp, &home, &xdg_data, main_config));

        let collision = report
            .home_manager_profile_collision
            .as_ref()
            .expect("mixed ownership warning");
        assert_eq!(collision.status, "warn");
        assert!(collision.message.contains("default Nix profile"));
        assert!(
            collision
                .details
                .as_deref()
                .unwrap_or_default()
                .contains("yzx home_manager prepare --apply")
        );
    }
}
