//! Install ownership, launcher provenance, and doctor install-artifact classification.
//! Bead: yazelix-ulb2.4.1

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const HOME_MANAGER_FILES_MARKER: &str = "-home-manager-files/";
const MANUAL_DESKTOP_ICON_SIZES: &[&str] = &["48x48", "64x64", "128x128", "256x256"];

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HomeManagerPrepareArtifact {
    pub id: String,
    pub class: String,
    pub label: String,
    pub path: String,
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
    pub prepare_artifacts: Vec<HomeManagerPrepareArtifact>,
    pub desktop_entry_freshness: DoctorInstallResult,
    pub wrapper_shadowing: Vec<DoctorInstallResult>,
}

pub fn evaluate_install_ownership_report(
    request: &InstallOwnershipEvaluateRequest,
) -> InstallOwnershipEvaluateData {
    let has_hm = has_home_manager_managed_install(&request.main_config_path);
    let profile_candidates =
        home_manager_yzx_profile_paths(&request.home_dir, request.user.as_deref());
    let existing_profile = first_existing_profile_yzx(&profile_candidates);
    let stable = resolve_stable_yzx_wrapper_path(&request.home_dir, has_hm, &existing_profile);
    let desktop_launcher = resolve_desktop_launcher_path(&request.runtime_dir, stable.as_deref());
    let desktop_launcher_str = path_to_string(&desktop_launcher);
    let install_owner = detect_install_owner(has_hm, existing_profile.as_ref(), &request.home_dir);
    let is_manual_runtime_ref = is_manual_runtime_reference_path(
        &request.yazelix_state_dir.join("runtime").join("current"),
    );
    let prepare_artifacts = collect_home_manager_prepare_artifacts(request, has_hm);
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
        prepare_artifacts,
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
        return "home-manager".into();
    }
    let profile_desktop = home_dir
        .join(".nix-profile")
        .join("share")
        .join("applications")
        .join("yazelix.desktop");
    if profile_desktop.exists() {
        "home-manager".into()
    } else {
        "manual".into()
    }
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

fn manual_desktop_entry_path(xdg_data_home: &Path) -> PathBuf {
    xdg_data_home
        .join("applications")
        .join("com.yazelix.Yazelix.desktop")
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
            });
        }
    }
    out
}

fn collect_home_manager_prepare_artifacts(
    request: &InstallOwnershipEvaluateRequest,
    has_hm: bool,
) -> Vec<HomeManagerPrepareArtifact> {
    let mut artifacts = Vec::new();
    let main = &request.main_config_path;
    if main.exists() && !has_hm {
        artifacts.push(HomeManagerPrepareArtifact {
            id: "main_config".into(),
            class: "blocker".into(),
            label: "managed yazelix.toml surface".into(),
            path: path_to_string(main),
        });
    }
    let desktop_entry = manual_desktop_entry_path(&request.xdg_data_home);
    if is_manual_desktop_entry_path(&desktop_entry) {
        artifacts.push(HomeManagerPrepareArtifact {
            id: "desktop_entry".into(),
            class: "cleanup".into(),
            label: "manual desktop entry".into(),
            path: path_to_string(&desktop_entry),
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
        });
    }
    artifacts
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

fn desktop_entry_exec_matches_expected(exec: Option<&str>, expected: &[String]) -> bool {
    let Some(e) = exec else {
        return false;
    };
    expected.iter().any(|x| x == e)
}

fn check_desktop_entry_freshness(
    request: &InstallOwnershipEvaluateRequest,
    install_owner: &str,
    profile_paths: &[PathBuf],
    desktop_launcher: &str,
) -> DoctorInstallResult {
    let local_path = manual_desktop_entry_path(&request.xdg_data_home);
    let profile_path = request
        .home_dir
        .join(".nix-profile")
        .join("share")
        .join("applications")
        .join("yazelix.desktop");
    let local_exists = local_path.exists();
    let profile_exists = profile_path.exists();
    let desktop_path: Option<PathBuf> = if local_exists {
        Some(local_path.clone())
    } else if profile_exists {
        Some(profile_path.clone())
    } else {
        None
    };
    let expected = expected_desktop_entry_execs(install_owner, profile_paths, desktop_launcher);

    let Some(dp) = desktop_path else {
        let details = if install_owner == "home-manager" {
            format!(
                "Home Manager-managed desktop entries usually resolve through {}. Reapply your Home Manager configuration if it is missing.",
                path_to_string(&profile_path)
            )
        } else {
            "Run `yzx desktop install` if you want application-launcher integration.".into()
        };
        return DoctorInstallResult {
            status: "info".into(),
            message: "Yazelix desktop entry not installed".into(),
            details: Some(details),
            fix_available: false,
        };
    };

    let local_exec = if local_exists {
        get_desktop_entry_exec(&local_path)
    } else {
        None
    };
    let profile_exec = if profile_exists {
        get_desktop_entry_exec(&profile_path)
    } else {
        None
    };

    if install_owner == "home-manager"
        && local_exists
        && profile_exists
        && !desktop_entry_exec_matches_expected(local_exec.as_deref(), &expected)
        && desktop_entry_exec_matches_expected(profile_exec.as_deref(), &expected)
    {
        return DoctorInstallResult {
            status: "warning".into(),
            message:
                "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry"
                    .into(),
            details: Some(format!(
                "Shadowing local entry: {}\nLocal Exec: {}\nHome Manager entry: {}\nProfile Exec: {}\nRemove the shadowing local entry with `yzx desktop uninstall`, then reapply your Home Manager configuration if the profile desktop entry is missing or stale.",
                path_to_string(&local_path),
                local_exec.as_deref().unwrap_or("<missing>"),
                path_to_string(&profile_path),
                profile_exec.as_deref().unwrap_or("<missing>"),
            )),
            fix_available: false,
        };
    }

    let desktop_exec = if dp == local_path {
        local_exec.clone()
    } else {
        profile_exec.clone()
    };
    let repair_hint = if dp == profile_path {
        "Repair by reapplying your Home Manager configuration."
    } else {
        "Repair with `yzx desktop install`."
    };

    if desktop_exec.is_none() {
        return DoctorInstallResult {
            status: "warning".into(),
            message: "Yazelix desktop entry is invalid".into(),
            details: Some(format!(
                "The installed desktop entry has no Exec line. {repair_hint}"
            )),
            fix_available: false,
        };
    }

    let de = desktop_exec.as_deref().unwrap();
    if !desktop_entry_exec_matches_expected(Some(de), &expected) {
        return DoctorInstallResult {
            status: "warning".into(),
            message: "Yazelix desktop entry does not use the expected launcher path".into(),
            details: Some(format!(
                "Desktop entry Exec: {de}\nExpected one of: {}\n{repair_hint}",
                expected.join(", ")
            )),
            fix_available: false,
        };
    }

    if !desktop_entry_terminal_enabled(&dp) {
        let terminal_value =
            get_desktop_entry_terminal_value(&dp).unwrap_or_else(|| "<missing>".into());
        return DoctorInstallResult {
            status: "warning".into(),
            message: "Yazelix desktop entry is not terminal-backed".into(),
            details: Some(format!(
                "Desktop entry: {}\nTerminal: {}\nDesktop launch failures before terminal handoff can disappear without a visible terminal surface.\n{repair_hint}",
                path_to_string(&dp),
                terminal_value
            )),
            fix_available: false,
        };
    }

    DoctorInstallResult {
        status: "ok".into(),
        message: "Yazelix desktop entry uses the expected launcher path".into(),
        details: Some(path_to_string(&dp)),
        fix_available: false,
    }
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
            out.push(DoctorInstallResult {
                status: "warning".into(),
                message: "A stale host-shell yzx function or alias is shadowing the current profile command"
                    .into(),
                details: Some(details_lines.join("\n")),
                fix_available: false,
            });
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
    out.push(DoctorInstallResult {
        status: "warning".into(),
        message: "A stale user-local yzx wrapper shadows the profile-owned Yazelix command".into(),
        details: Some(details_lines.join("\n")),
        fix_available: false,
    });
    out
}

// Test lane: default

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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

        let report = evaluate_install_ownership_report(&InstallOwnershipEvaluateRequest {
            runtime_dir: tmp.path().join("runtime"),
            home_dir: home.clone(),
            user: Some("test-user".into()),
            xdg_config_home: home.join(".config"),
            xdg_data_home: xdg_data.clone(),
            yazelix_state_dir: xdg_data.join("yazelix"),
            main_config_path: main_config,
            invoked_yzx_path: None,
            redirected_from_stale_yzx_path: None,
            shell_resolved_yzx_path: None,
        });

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
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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

    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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
            format!("[Desktop Entry]\nName=Yazelix\nTerminal=true\nExec={good_exec}\n"),
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
            r.desktop_entry_freshness.message,
            "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry"
        );
    }
}
