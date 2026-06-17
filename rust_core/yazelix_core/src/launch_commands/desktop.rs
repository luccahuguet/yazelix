use super::launch::run_launch_flow;
use super::process::find_command;
use super::terminal::current_platform_name;
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_override_from_env, home_dir_from_env, runtime_dir_from_env};
use crate::install_ownership_env::install_ownership_request_from_env_with_runtime_dir;
use crate::install_ownership_report::{
    InstallOwnershipEvaluateData, evaluate_install_ownership_report,
};
use crate::terminal_variant::{
    active_terminal_from_runtime_dir, terminal_desktop_entry_file_name,
    terminal_desktop_entry_name, terminal_startup_wm_class,
};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// Test lane: default
const DESKTOP_ICON_SIZES: &[&str] = &["48x48", "64x64", "128x128", "256x256"];
pub(super) const MACOS_PREVIEW_APP_DIR_NAME: &str = "Yazelix Preview.app";
const MACOS_PREVIEW_APP_NAME: &str = "Yazelix Preview";
pub(super) const MACOS_PREVIEW_BUNDLE_ID: &str = "com.yazelix.YazelixPreview";
pub(super) const MACOS_PREVIEW_BUNDLE_SHORT_VERSION: &str = "0.1";
pub(super) const MACOS_PREVIEW_BUNDLE_VERSION: &str = "1";
pub(super) const MACOS_PREVIEW_EXECUTABLE_NAME: &str = "yazelix_preview_launcher";
pub(super) const MACOS_PREVIEW_MARKER_FILE: &str = "yazelix_preview_launcher.marker";
const MACOS_PREVIEW_MIN_SYSTEM_VERSION: &str = "12.0";
const DESKTOP_LAUNCH_CLEARED_ENV_KEYS: &[&str] = &[
    "IN_YAZELIX_SHELL",
    "YAZELIX_DIR",
    "YAZELIX_CURSOR_COLOR",
    "YAZELIX_CURSOR_DIVIDER",
    "YAZELIX_CURSOR_FAMILY",
    "YAZELIX_CURSOR_NAME",
    "YAZELIX_CURSOR_PRIMARY_COLOR",
    "YAZELIX_CURSOR_SECONDARY_COLOR",
    "YAZELIX_NU_BIN",
    "YAZELIX_TERMINAL",
    "YAZELIX_ZELLIJ_SESSION_NAME",
    "YAZI_ID",
    "ZELLIJ",
    "ZELLIJ_DEFAULT_LAYOUT",
    "ZELLIJ_PANE_ID",
    "ZELLIJ_SESSION_NAME",
    "ZELLIJ_TAB_NAME",
    "ZELLIJ_TAB_POSITION",
];
pub(super) fn run_desktop_install(print_path: bool) -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let report = evaluate_install_ownership_report(
        &install_ownership_request_from_env_with_runtime_dir(runtime_dir.clone())?,
    );
    if report.install_owner == "home-manager" {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "home_manager_desktop_owner",
            "Home Manager owns Yazelix desktop integration for this install.",
            "Reapply your Home Manager configuration for the profile desktop entry, or run `yzx desktop uninstall` only to remove a stale user-local entry.",
            serde_json::json!({}),
        ));
    }

    let launcher_path = PathBuf::from(report.desktop_launcher_path);
    if !runtime_dir.exists() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_runtime_dir",
            format!("Missing Yazelix runtime at {}", runtime_dir.display()),
            "Reinstall Yazelix so the runtime tree is present, then retry.",
            serde_json::json!({}),
        ));
    }
    if !launcher_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_desktop_launcher",
            format!("Missing stable Yazelix CLI at {}", launcher_path.display()),
            "Restore the stable launcher path or reinstall Yazelix, then retry.",
            serde_json::json!({}),
        ));
    }

    let home_dir = home_dir_from_env()?;
    let xdg_data_home = xdg_data_home(&home_dir);
    let applications_dir = xdg_data_home.join("applications");
    let icons_root = xdg_data_home.join("icons").join("hicolor");
    let active_terminal = active_terminal_from_runtime_dir(&runtime_dir)?;
    let desktop_path = applications_dir.join(terminal_desktop_entry_file_name(&active_terminal));
    let desktop_entry = render_desktop_entry(&launcher_path, &active_terminal);

    fs::create_dir_all(&applications_dir).map_err(|source| {
        CoreError::io(
            "desktop_applications_dir",
            format!(
                "Could not create desktop applications directory {}.",
                applications_dir.display()
            ),
            "Create the directory or fix permissions, then retry.",
            applications_dir.display().to_string(),
            source,
        )
    })?;
    install_desktop_icons(&runtime_dir, &icons_root)?;
    fs::write(&desktop_path, desktop_entry).map_err(|source| {
        CoreError::io(
            "desktop_entry_write",
            format!("Could not write desktop entry {}.", desktop_path.display()),
            "Fix the directory permissions, then retry.",
            desktop_path.display().to_string(),
            source,
        )
    })?;

    maybe_validate_desktop_entry(&desktop_path)?;
    maybe_refresh_desktop_database(&applications_dir);
    maybe_refresh_icon_cache(&icons_root);

    if print_path {
        println!("{}", desktop_path.display());
    } else {
        println!(
            "Installed Yazelix desktop entry: {}",
            desktop_path.display()
        );
    }
    Ok(0)
}

pub(super) fn run_desktop_uninstall(print_path: bool) -> Result<i32, CoreError> {
    let home_dir = home_dir_from_env()?;
    let xdg_data_home = xdg_data_home(&home_dir);
    let applications_dir = xdg_data_home.join("applications");
    let icons_root = xdg_data_home.join("icons").join("hicolor");
    let runtime_dir = runtime_dir_from_env()?;
    let active_terminal = active_terminal_from_runtime_dir(&runtime_dir)?;
    let desktop_path = applications_dir.join(terminal_desktop_entry_file_name(&active_terminal));

    if desktop_path.exists() {
        fs::remove_file(&desktop_path).map_err(|source| {
            CoreError::io(
                "desktop_entry_remove",
                format!("Could not remove desktop entry {}.", desktop_path.display()),
                "Fix the directory permissions, then retry.",
                desktop_path.display().to_string(),
                source,
            )
        })?;
    }
    for size in DESKTOP_ICON_SIZES {
        let path = icons_root.join(size).join("apps").join("yazelix.png");
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }
    maybe_refresh_desktop_database(&applications_dir);
    maybe_refresh_icon_cache(&icons_root);

    if print_path {
        println!("{}", desktop_path.display());
    } else {
        println!("Removed Yazelix desktop entry: {}", desktop_path.display());
    }
    Ok(0)
}

pub(super) fn run_macos_preview_install(print_path: bool) -> Result<i32, CoreError> {
    require_macos_preview_platform()?;
    let runtime_dir = runtime_dir_from_env()?;
    if !runtime_dir.exists() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_runtime_dir",
            format!("Missing Yazelix runtime at {}", runtime_dir.display()),
            "Reinstall Yazelix so the runtime tree is present, then retry.",
            serde_json::json!({}),
        ));
    }

    let report = evaluate_install_ownership_report(
        &install_ownership_request_from_env_with_runtime_dir(runtime_dir)?,
    );
    let launcher_path = macos_preview_profile_launcher_from_report(&report)?;
    if !launcher_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_macos_preview_launcher",
            format!(
                "Missing package-owned Yazelix launcher at {}.",
                launcher_path.display()
            ),
            "Reinstall Yazelix through the default Nix profile or Home Manager, then rerun `yzx desktop macos_preview install`.",
            serde_json::json!({}),
        ));
    }

    let home_dir = home_dir_from_env()?;
    let app_path = macos_preview_app_path(&home_dir);
    install_macos_preview_app(&app_path, &launcher_path)?;

    if print_path {
        println!("{}", app_path.display());
    } else {
        println!(
            "Installed experimental Yazelix macOS launcher preview: {}",
            app_path.display()
        );
        println!(
            "This preview is package-first, unsigned, unnotarized, and maintainer-unverified on macOS hardware."
        );
    }
    Ok(0)
}

pub(super) fn run_macos_preview_uninstall(print_path: bool) -> Result<i32, CoreError> {
    require_macos_preview_platform()?;
    let home_dir = home_dir_from_env()?;
    let app_path = macos_preview_app_path(&home_dir);

    if app_path.exists() {
        ensure_macos_preview_bundle_is_managed(&app_path)?;
        fs::remove_dir_all(&app_path).map_err(|source| {
            CoreError::io(
                "macos_preview_app_remove",
                format!(
                    "Could not remove macOS preview launcher app {}.",
                    app_path.display()
                ),
                "Fix the directory permissions, then retry.",
                app_path.display().to_string(),
                source,
            )
        })?;
    }

    if print_path {
        println!("{}", app_path.display());
    } else {
        println!(
            "Removed experimental Yazelix macOS launcher preview: {}",
            app_path.display()
        );
    }
    Ok(0)
}

pub(super) fn run_desktop_launch() -> Result<i32, CoreError> {
    print_desktop_progress("Preparing session...");
    let home_dir = home_dir_from_env()?;
    let home_dir_string = home_dir.to_string_lossy().to_string();
    match run_launch_flow(
        Some(&home_dir_string),
        config_override_from_env().as_deref(),
        false,
        false,
        true,
        DESKTOP_LAUNCH_CLEARED_ENV_KEYS,
    ) {
        Ok(code) => Ok(code),
        Err(err) => {
            acknowledge_desktop_failure(&err.message());
            Err(err)
        }
    }
}

fn xdg_data_home(home_dir: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("XDG_DATA_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_home_path(trimmed, home_dir);
        }
    }
    home_dir.join(".local").join("share")
}

fn expand_home_path(raw: &str, home_dir: &Path) -> PathBuf {
    if raw == "~" {
        return home_dir.to_path_buf();
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        return home_dir.join(rest);
    }
    PathBuf::from(raw)
}

fn quote_desktop_exec_arg(value: &Path) -> String {
    let escaped = value
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`");
    format!("\"{escaped}\"")
}

pub(super) fn render_desktop_entry(launcher_path: &Path, active_terminal: &str) -> String {
    [
        "[Desktop Entry]".to_string(),
        "Version=1.4".to_string(),
        "Type=Application".to_string(),
        format!("Name={}", terminal_desktop_entry_name(active_terminal)),
        "Comment=Yazi + Zellij + Helix integrated terminal environment".to_string(),
        "Icon=yazelix".to_string(),
        format!(
            "StartupWMClass={}",
            terminal_startup_wm_class(active_terminal)
        ),
        "Terminal=true".to_string(),
        "X-Yazelix-Managed=true".to_string(),
        format!(
            "Exec={} desktop launch",
            quote_desktop_exec_arg(launcher_path)
        ),
        "Categories=Development;".to_string(),
    ]
    .join("\n")
}

fn require_macos_preview_platform() -> Result<(), CoreError> {
    if current_platform_name() == "macos" {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "macos_preview_requires_macos",
        "The macOS launcher preview can only be installed on macOS.",
        "Use `yzx launch` on this platform, or retry `yzx desktop macos_preview install` from macOS.",
        serde_json::json!({}),
    ))
}

fn macos_preview_profile_launcher_from_report(
    report: &InstallOwnershipEvaluateData,
) -> Result<PathBuf, CoreError> {
    let Some(raw) = report.existing_home_manager_profile_yzx.as_deref() else {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_macos_preview_profile_launcher",
            "Could not find a package-owned Yazelix launcher in the default Nix or Home Manager profile.",
            "Install Yazelix with `nix profile add github:luccahuguet/yazelix#yazelix` or through Home Manager, then rerun `yzx desktop macos_preview install`.",
            serde_json::json!({
                "install_owner": &report.install_owner,
                "profile_candidates": &report.home_manager_profile_yzx_candidates,
            }),
        ));
    };
    Ok(PathBuf::from(raw))
}

fn macos_preview_app_path(home_dir: &Path) -> PathBuf {
    home_dir
        .join("Applications")
        .join(MACOS_PREVIEW_APP_DIR_NAME)
}

pub(super) fn install_macos_preview_app(
    app_path: &Path,
    launcher_path: &Path,
) -> Result<(), CoreError> {
    if app_path.exists() {
        ensure_macos_preview_bundle_is_managed(app_path)?;
        fs::remove_dir_all(app_path).map_err(|source| {
            CoreError::io(
                "macos_preview_app_refresh",
                format!(
                    "Could not refresh existing macOS preview launcher app {}.",
                    app_path.display()
                ),
                "Fix the directory permissions, then retry.",
                app_path.display().to_string(),
                source,
            )
        })?;
    }

    let contents_dir = app_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");
    fs::create_dir_all(&macos_dir).map_err(|source| {
        CoreError::io(
            "macos_preview_app_dir",
            format!(
                "Could not create macOS preview launcher directory {}.",
                macos_dir.display()
            ),
            "Create the directory or fix permissions, then retry.",
            macos_dir.display().to_string(),
            source,
        )
    })?;
    fs::create_dir_all(&resources_dir).map_err(|source| {
        CoreError::io(
            "macos_preview_resources_dir",
            format!(
                "Could not create macOS preview resources directory {}.",
                resources_dir.display()
            ),
            "Create the directory or fix permissions, then retry.",
            resources_dir.display().to_string(),
            source,
        )
    })?;

    fs::write(
        contents_dir.join("Info.plist"),
        render_macos_preview_info_plist(),
    )
    .map_err(|source| {
        CoreError::io(
            "macos_preview_info_plist_write",
            format!(
                "Could not write macOS preview Info.plist under {}.",
                contents_dir.display()
            ),
            "Fix the directory permissions, then retry.",
            contents_dir.display().to_string(),
            source,
        )
    })?;
    fs::write(
        resources_dir.join(MACOS_PREVIEW_MARKER_FILE),
        "Managed by `yzx desktop macos_preview install`.\n",
    )
    .map_err(|source| {
        CoreError::io(
            "macos_preview_marker_write",
            format!(
                "Could not write macOS preview marker under {}.",
                resources_dir.display()
            ),
            "Fix the directory permissions, then retry.",
            resources_dir.display().to_string(),
            source,
        )
    })?;

    let executable_path = macos_dir.join(MACOS_PREVIEW_EXECUTABLE_NAME);
    fs::write(
        &executable_path,
        render_macos_preview_launcher_script(launcher_path),
    )
    .map_err(|source| {
        CoreError::io(
            "macos_preview_launcher_write",
            format!(
                "Could not write macOS preview launcher script {}.",
                executable_path.display()
            ),
            "Fix the directory permissions, then retry.",
            executable_path.display().to_string(),
            source,
        )
    })?;
    make_file_executable(&executable_path)?;

    Ok(())
}

pub(super) fn ensure_macos_preview_bundle_is_managed(app_path: &Path) -> Result<(), CoreError> {
    if macos_preview_bundle_is_managed(app_path) {
        return Ok(());
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "macos_preview_app_conflict",
        format!(
            "Refusing to modify existing non-Yazelix preview app bundle at {}.",
            app_path.display()
        ),
        "Move that app bundle aside, or choose a clean ~/Applications path before retrying.",
        serde_json::json!({ "path": app_path.display().to_string() }),
    ))
}

pub(super) fn macos_preview_bundle_is_managed(app_path: &Path) -> bool {
    let marker = app_path
        .join("Contents")
        .join("Resources")
        .join(MACOS_PREVIEW_MARKER_FILE);
    let info = app_path.join("Contents").join("Info.plist");
    marker.is_file()
        && fs::read_to_string(info)
            .map(|raw| raw.contains(MACOS_PREVIEW_BUNDLE_ID))
            .unwrap_or(false)
}

pub(super) fn render_macos_preview_info_plist() -> String {
    [
        r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string(),
        r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">"#.to_string(),
        r#"<plist version="1.0">"#.to_string(),
        r#"<dict>"#.to_string(),
        r#"  <key>CFBundleDevelopmentRegion</key>"#.to_string(),
        r#"  <string>en</string>"#.to_string(),
        r#"  <key>CFBundleDisplayName</key>"#.to_string(),
        format!("  <string>{MACOS_PREVIEW_APP_NAME}</string>"),
        r#"  <key>CFBundleExecutable</key>"#.to_string(),
        format!("  <string>{MACOS_PREVIEW_EXECUTABLE_NAME}</string>"),
        r#"  <key>CFBundleIdentifier</key>"#.to_string(),
        format!("  <string>{MACOS_PREVIEW_BUNDLE_ID}</string>"),
        r#"  <key>CFBundleName</key>"#.to_string(),
        format!("  <string>{MACOS_PREVIEW_APP_NAME}</string>"),
        r#"  <key>CFBundlePackageType</key>"#.to_string(),
        r#"  <string>APPL</string>"#.to_string(),
        r#"  <key>CFBundleShortVersionString</key>"#.to_string(),
        format!("  <string>{MACOS_PREVIEW_BUNDLE_SHORT_VERSION}</string>"),
        r#"  <key>CFBundleVersion</key>"#.to_string(),
        format!("  <string>{MACOS_PREVIEW_BUNDLE_VERSION}</string>"),
        r#"  <key>LSApplicationCategoryType</key>"#.to_string(),
        r#"  <string>public.app-category.developer-tools</string>"#.to_string(),
        r#"  <key>LSMinimumSystemVersion</key>"#.to_string(),
        format!("  <string>{MACOS_PREVIEW_MIN_SYSTEM_VERSION}</string>"),
        r#"  <key>NSHighResolutionCapable</key>"#.to_string(),
        r#"  <true/>"#.to_string(),
        r#"</dict>"#.to_string(),
        r#"</plist>"#.to_string(),
    ]
    .join("\n")
}

fn shell_single_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', "'\"'\"'"))
}

pub(super) fn render_macos_preview_launcher_script(launcher_path: &Path) -> String {
    let quoted_launcher = shell_single_quote(&launcher_path.to_string_lossy());
    format!(
        r#"#!/bin/sh
set -u

YAZELIX_STABLE_YZX={quoted_launcher}

show_failure() {{
  message=$1
  if command -v osascript >/dev/null 2>&1; then
    osascript <<'YAZELIX_APPLESCRIPT' >/dev/null 2>&1
display dialog "Yazelix Preview could not start. Run yzx doctor --verbose from Terminal, then reinstall the preview launcher with yzx desktop macos_preview install." buttons {{"OK"}} default button "OK" with title "Yazelix Preview"
YAZELIX_APPLESCRIPT
  fi
  printf '%s\n' "$message" >&2
}}

if [ ! -x "$YAZELIX_STABLE_YZX" ]; then
  show_failure "The package-owned yzx launcher for Yazelix Preview is missing or not executable. Reinstall Yazelix, then run: yzx desktop macos_preview install"
  exit 1
fi

"$YAZELIX_STABLE_YZX" desktop launch
status=$?
if [ "$status" -ne 0 ]; then
  show_failure "Yazelix Preview could not start. Run yzx doctor --verbose from Terminal, then reinstall the preview launcher with: yzx desktop macos_preview install"
fi
exit "$status"
"#
    )
}

fn make_file_executable(path: &Path) -> Result<(), CoreError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)
            .map_err(|source| {
                CoreError::io(
                    "macos_preview_launcher_permissions",
                    format!(
                        "Could not read permissions for macOS preview launcher {}.",
                        path.display()
                    ),
                    "Fix the directory permissions, then retry.",
                    path.display().to_string(),
                    source,
                )
            })?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|source| {
            CoreError::io(
                "macos_preview_launcher_permissions",
                format!(
                    "Could not mark macOS preview launcher executable at {}.",
                    path.display()
                ),
                "Fix the directory permissions, then retry.",
                path.display().to_string(),
                source,
            )
        })?;
    }
    let _ = path;
    Ok(())
}

fn install_desktop_icons(runtime_dir: &Path, icons_root: &Path) -> Result<(), CoreError> {
    for size in DESKTOP_ICON_SIZES {
        let source = runtime_dir
            .join("assets")
            .join("icons")
            .join(size)
            .join("yazelix.png");
        if !source.is_file() {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "missing_desktop_icon",
                format!("Missing Yazelix desktop icon asset: {}", source.display()),
                "Restore the runtime icon assets or reinstall Yazelix, then retry.",
                serde_json::json!({}),
            ));
        }
        let destination = icons_root.join(size).join("apps").join("yazelix.png");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).map_err(|source| {
                CoreError::io(
                    "desktop_icon_dir",
                    format!("Could not create icon directory {}.", parent.display()),
                    "Fix the directory permissions, then retry.",
                    parent.display().to_string(),
                    source,
                )
            })?;
        }
        match fs::remove_file(&destination) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(CoreError::io(
                    "desktop_icon_replace",
                    format!(
                        "Could not replace existing desktop icon {}.",
                        destination.display()
                    ),
                    "Fix the file or directory permissions, then retry.",
                    destination.display().to_string(),
                    error,
                ));
            }
        }
        fs::copy(&source, &destination).map_err(|error| {
            CoreError::io(
                "desktop_icon_copy",
                format!(
                    "Could not copy desktop icon {} to {}.",
                    source.display(),
                    destination.display()
                ),
                "Fix the directory permissions, then retry.",
                destination.display().to_string(),
                error,
            )
        })?;
    }
    Ok(())
}

fn maybe_validate_desktop_entry(desktop_path: &Path) -> Result<(), CoreError> {
    let Some(command) = find_command("desktop-file-validate") else {
        return Ok(());
    };
    let output = Command::new(command)
        .arg(desktop_path)
        .output()
        .map_err(|source| {
            CoreError::io(
                "desktop_file_validate",
                format!(
                    "Failed to run desktop-file-validate for {}.",
                    desktop_path.display()
                ),
                "Install desktop-file-validate or fix the host PATH, then retry.",
                desktop_path.display().to_string(),
                source,
            )
        })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Runtime,
            "desktop_entry_invalid",
            format!(
                "Generated desktop entry failed validation: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            "Fix the generated desktop entry contract, then retry.",
            serde_json::json!({}),
        ))
    }
}

fn maybe_refresh_desktop_database(applications_dir: &Path) {
    if let Some(command) = find_command("update-desktop-database") {
        let _ = Command::new(command).arg(applications_dir).status();
    }
}

fn maybe_refresh_icon_cache(icons_root: &Path) {
    if let Some(command) = find_command("gtk-update-icon-cache") {
        let _ = Command::new(command)
            .args(["--force", "--ignore-theme-index"])
            .arg(icons_root)
            .status();
    }
}

fn print_desktop_progress(message: &str) {
    println!("Yazelix: {message}");
}

fn acknowledge_desktop_failure(error_text: &str) {
    println!();
    println!("Yazelix: Launch failed.");
    println!();
    println!("{error_text}");
    println!();
    print!("Press Enter to close this window.");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression: Mars desktop entries need terminal-specific startup class names so desktop switchers do not merge them with Ghostty windows.
    #[test]
    fn render_desktop_entry_uses_terminal_specific_startup_class_for_yzxterm() {
        let ghostty_entry = render_desktop_entry(Path::new("/tmp/yzx"), "ghostty");
        assert!(ghostty_entry.contains("Name=New Yazelix - Ghostty"));
        assert!(ghostty_entry.contains("StartupWMClass=com.yazelix.Yazelix"));

        let yzxterm_entry = render_desktop_entry(Path::new("/tmp/yzx"), "yzxterm");
        assert!(yzxterm_entry.contains("Name=New Yazelix - Mars"));
        assert!(yzxterm_entry.contains("StartupWMClass=com.yazelix.Yazelix.Mars"));
    }

    #[cfg(unix)]
    fn set_read_only(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o444);
        fs::set_permissions(path, permissions).unwrap();
    }

    // Regression: desktop reinstall must replace icons copied from the read-only Nix store instead of failing on the next install.
    #[test]
    fn install_desktop_icons_replaces_read_only_existing_icons() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path().join("runtime");
        let icons_root = temp.path().join("icons").join("hicolor");

        for size in DESKTOP_ICON_SIZES {
            let source = runtime_dir
                .join("assets")
                .join("icons")
                .join(size)
                .join("yazelix.png");
            fs::create_dir_all(source.parent().unwrap()).unwrap();
            fs::write(&source, format!("new {size}")).unwrap();

            let destination = icons_root.join(size).join("apps").join("yazelix.png");
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::write(&destination, format!("old {size}")).unwrap();
            #[cfg(unix)]
            set_read_only(&destination);
        }

        install_desktop_icons(&runtime_dir, &icons_root).unwrap();

        for size in DESKTOP_ICON_SIZES {
            let destination = icons_root.join(size).join("apps").join("yazelix.png");
            assert_eq!(
                fs::read_to_string(destination).unwrap(),
                format!("new {size}")
            );
        }
    }
}
