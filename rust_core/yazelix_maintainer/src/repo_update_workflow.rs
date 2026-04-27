// Test lane: maintainer
use crate::repo_contract_validation::sync_readme_surface;
use serde::Deserialize;
use serde_json::Map as JsonMap;
use serde_json::Value as JsonValue;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value as TomlValue;

const DEFAULT_HOME_MANAGER_DIR: &str = "~/.config/home-manager";
const DEFAULT_HOME_MANAGER_INPUT: &str = "yazelix-hm";
const DEFAULT_MAIN_CONFIG_RELATIVE_PATH: &str = "yazelix_default.toml";
const DEFAULT_CONSTANTS_RELATIVE_PATH: &str = "nushell/scripts/utils/constants.nu";
const DEFAULT_VENDORED_YAZI_MANIFEST_RELATIVE_PATH: &str =
    "config_metadata/vendored_yazi_plugins.toml";
const UPDATE_CANARY_BASE_RELATIVE_PATH: &str = ".local/share/yazelix/update_canaries";

#[derive(Debug, Clone)]
pub struct RepoUpdateOptions {
    pub yes: bool,
    pub no_canary: bool,
    pub activate: String,
    pub home_manager_dir: String,
    pub home_manager_input: String,
    pub home_manager_attr: String,
    pub canary_only: bool,
    pub canaries: Vec<String>,
}

impl Default for RepoUpdateOptions {
    fn default() -> Self {
        Self {
            yes: false,
            no_canary: false,
            activate: String::new(),
            home_manager_dir: DEFAULT_HOME_MANAGER_DIR.to_string(),
            home_manager_input: DEFAULT_HOME_MANAGER_INPUT.to_string(),
            home_manager_attr: String::new(),
            canary_only: false,
            canaries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UpdateActivationMode {
    Profile,
    HomeManager,
    None,
}

#[derive(Debug, Clone)]
struct UpdateCanary {
    name: String,
    config_path: PathBuf,
    description: String,
}

#[derive(Debug)]
struct UpdateCanaryContext {
    _temp_dir: TempDirGuard,
    canaries: Vec<UpdateCanary>,
}

#[derive(Debug)]
struct UpdateCanaryResult {
    name: String,
    config_path: PathBuf,
    description: String,
    exit_code: i32,
    stdout_tail: String,
    stderr_tail: String,
    ok: bool,
}

#[derive(Debug)]
struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Debug, Deserialize)]
struct VendoredYaziManifest {
    plugins: Vec<VendoredYaziPlugin>,
}

#[derive(Debug, Deserialize)]
struct VendoredYaziPlugin {
    name: String,
    upstream_repo: String,
    pinned_rev: String,
    #[serde(default)]
    source_subdir: String,
    target_dir: String,
    #[serde(default)]
    managed_files: Vec<String>,
    #[serde(default)]
    patch_file: String,
}

#[derive(Debug, Clone)]
struct ZjstatusPackage {
    flake_ref: String,
    store_root: PathBuf,
}

#[derive(Debug, Clone)]
struct HomeManagerActivation {
    switch_ref: String,
}

pub fn run_repo_update_workflow(
    repo_root: &Path,
    options: &RepoUpdateOptions,
) -> Result<(), String> {
    ensure_nix_available()?;

    if options.no_canary && options.canary_only {
        return Err("--no-canary and --canary-only cannot be used together.".to_string());
    }

    let selected_canaries = resolve_update_canary_selection(&options.canaries)?;
    let activation_mode =
        resolve_requested_update_activation_mode(&options.activate, options.canary_only)?;

    if !options.yes && !options.canary_only && !confirm_real_update()? {
        println!("Aborted.");
        return Ok(());
    }

    if options.canary_only {
        println!(
            "🧪 Running update canaries only: {}",
            selected_canaries.join(", ")
        );
    }

    if !options.canary_only {
        refresh_repo_runtime_inputs(repo_root)?;
    }

    if options.no_canary {
        println!("⚠️  Canary checks were skipped.");
    } else {
        let canary_results = run_update_canaries(repo_root, &selected_canaries)?;
        print_update_canary_summary(&canary_results);
        if canary_results.iter().any(|result| !result.ok) {
            print_update_canary_failure_details(&canary_results);
            println!();
            println!("❌ One or more canaries failed.");
            if !options.canary_only {
                println!("   Keep this lockfile update local until the failures are resolved.");
            }
            return Err("Update canaries failed".to_string());
        }
        println!("✅ All selected canaries passed.");
    }

    if options.canary_only {
        println!("✅ Canary run completed. No lockfile or pin changes were made.");
        return Ok(());
    }

    println!("🔄 Syncing pinned runtime expectations...");
    sync_runtime_pins(repo_root)?;
    sync_readme_version_marker(repo_root)?;
    sync_vendored_zjstatus(repo_root)?;
    sync_vendored_yazi_plugins(repo_root)?;

    match activation_mode {
        UpdateActivationMode::None => {
            println!("⚠️  No local activation was requested.");
            println!(
                "✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, and vendored Yazi plugin runtime files are in sync in the repo checkout. Review and commit the changes if everything looks good."
            );
        }
        UpdateActivationMode::Profile => {
            activate_updated_profile_runtime(repo_root)?;
            println!(
                "✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, vendored Yazi plugin runtime files, and the local default-profile Yazelix package are in sync. Review and commit the changes if everything looks good."
            );
        }
        UpdateActivationMode::HomeManager => {
            let activation = activate_updated_home_manager_runtime(
                &expand_user_path(&options.home_manager_dir)?,
                options.home_manager_input.trim(),
                options.home_manager_attr.trim(),
            )?;
            println!(
                "✅ Inputs, canaries, runtime pins, README version marker, vendored zjstatus, vendored Yazi plugin runtime files, and the Home Manager activation at {} are in sync. Review and commit the changes if everything looks good.",
                activation.switch_ref
            );
        }
    }

    Ok(())
}

fn ensure_nix_available() -> Result<(), String> {
    if !command_exists("nix") {
        return Err("nix not found in PATH.\n   Install Nix, restart the shell, or enter an environment where `nix --version` works before running the maintainer update workflow.".to_string());
    }

    let version_result = run_command_capture("nix", ["--version"], None)?;
    if !version_result.status.success() {
        return Err(format!(
            "nix exists in PATH, but `nix --version` failed.\n{}",
            String::from_utf8_lossy(&version_result.stderr).trim()
        ));
    }

    let flake_result = run_command_capture("nix", ["flake", "--help"], None)?;
    if !flake_result.status.success() {
        return Err("nix flakes are not available in this shell.\n   Enable `nix-command flakes` or use the Yazelix maintainer shell before running this workflow.".to_string());
    }

    Ok(())
}

fn confirm_real_update() -> Result<bool, String> {
    println!("⚠️  This updates Yazelix runtime inputs to latest upstream unstable revisions.");
    println!(
        "   The hardened flow updates flake.lock locally, then runs canary refresh/build checks before finishing."
    );
    println!("   Broken updates should stay local and never be pushed.");
    print!("Continue? [y/N]: ");
    io::stdout()
        .flush()
        .map_err(|error| format!("Failed to flush confirmation prompt: {error}"))?;

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|error| format!("Failed to read confirmation input: {error}"))?;
    let normalized = line.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}

fn resolve_update_canary_selection(requested: &[String]) -> Result<Vec<String>, String> {
    let available = ["default", "shell_layout"];
    if requested.is_empty() {
        return Ok(available.iter().map(|value| value.to_string()).collect());
    }

    let normalized = requested
        .iter()
        .map(|name| name.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let invalid = normalized
        .iter()
        .filter(|name| !available.contains(&name.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !invalid.is_empty() {
        return Err(format!(
            "Unknown canary name(s): {}. Expected one of: {}",
            invalid.join(", "),
            available.join(", ")
        ));
    }

    let mut deduped = Vec::new();
    for name in normalized {
        if !deduped.contains(&name) {
            deduped.push(name);
        }
    }
    Ok(deduped)
}

fn resolve_update_activation_mode(requested: &str) -> Result<UpdateActivationMode, String> {
    match requested.trim().to_ascii_lowercase().as_str() {
        "profile" => Ok(UpdateActivationMode::Profile),
        "home_manager" => Ok(UpdateActivationMode::HomeManager),
        "none" => Ok(UpdateActivationMode::None),
        other => Err(format!(
            "Unknown activation mode: {}. Expected one of: profile, home_manager, none",
            other
        )),
    }
}

fn resolve_requested_update_activation_mode(
    requested: &str,
    canary_only: bool,
) -> Result<UpdateActivationMode, String> {
    let normalized = requested.trim();
    if canary_only {
        if !normalized.is_empty() {
            resolve_update_activation_mode(normalized)?;
        }
        return Ok(UpdateActivationMode::None);
    }
    if normalized.is_empty() {
        return Err(
            "yzx dev update now requires --activate profile|home_manager|none unless you are using --canary-only.".to_string(),
        );
    }
    resolve_update_activation_mode(normalized)
}

fn refresh_repo_runtime_inputs(repo_root: &Path) -> Result<(), String> {
    println!(
        "⚙️ Running: nix flake update nixpkgs (cwd: {})",
        repo_root.display()
    );
    run_command_streaming(
        "nix",
        ["flake", "update", "nixpkgs", "--flake"]
            .into_iter()
            .chain([repo_root.to_string_lossy().as_ref()]),
        Some(repo_root),
    )?;
    println!("✅ flake.lock nixpkgs input updated.");
    Ok(())
}

fn sync_runtime_pins(repo_root: &Path) -> Result<(), String> {
    let constants_path = repo_root.join(DEFAULT_CONSTANTS_RELATIVE_PATH);
    if !constants_path.is_file() {
        return Err(format!(
            "Constants file not found: {}",
            constants_path.display()
        ));
    }

    println!("   Resolving runtime pins from the locked nixpkgs input...");
    let nix_version =
        eval_locked_nixpkgs_version(repo_root, "pkgs.nixVersions.latest.version", "Nix")?;
    let nushell_version =
        eval_locked_nixpkgs_version(repo_root, "pkgs.nushell.version", "Nushell")?;
    let contents = fs::read_to_string(&constants_path)
        .map_err(|error| format!("Failed to read {}: {}", constants_path.display(), error))?;
    let updated = update_constant_value(
        &update_constant_value(&contents, "PINNED_NIX_VERSION", &nix_version),
        "PINNED_NUSHELL_VERSION",
        &nushell_version,
    );
    if updated == contents {
        println!(
            "✅ Runtime pins unchanged: nix {}, nushell {}",
            nix_version, nushell_version
        );
        return Ok(());
    }

    fs::write(&constants_path, updated)
        .map_err(|error| format!("Failed to write {}: {}", constants_path.display(), error))?;
    println!(
        "✅ Updated runtime pins: nix {}, nushell {}",
        nix_version, nushell_version
    );
    Ok(())
}

fn update_constant_value(contents: &str, key: &str, new_value: &str) -> String {
    let marker = format!("export const {key} = \"");
    contents
        .lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix(&marker) {
                if rest.ends_with('"') {
                    return format!("{marker}{new_value}\"");
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if contents.ends_with('\n') { "\n" } else { "" }
}

fn eval_locked_nixpkgs_version(
    repo_root: &Path,
    attr_expr: &str,
    label: &str,
) -> Result<String, String> {
    let expr = format!(
        "let\n  flake = builtins.getFlake \"path:{}\";\n  system = builtins.currentSystem;\n  pkgs = flake.inputs.nixpkgs.legacyPackages.${{system}};\nin {}",
        repo_root.display(),
        attr_expr
    );
    let output = run_command_capture(
        "nix",
        [
            "eval",
            "--raw",
            "--impure",
            "--extra-experimental-features",
            "nix-command flakes",
            "--expr",
            expr.as_str(),
        ],
        None,
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed to resolve {label} version from the locked nixpkgs input: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    extract_version(&raw).ok_or_else(|| format!("Failed to parse {label} version from: {raw}"))
}

fn extract_version(value: &str) -> Option<String> {
    for token in value.split(|ch: char| !ch.is_ascii_digit() && ch != '.') {
        let parts = token.split('.').collect::<Vec<_>>();
        if parts.len() == 3
            && parts
                .iter()
                .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
        {
            return Some(token.to_string());
        }
    }
    None
}

fn sync_readme_version_marker(repo_root: &Path) -> Result<(), String> {
    let sync = sync_readme_surface(repo_root, Some(&repo_root.join("README.md")), None)?;
    if !sync.title_changed && !sync.series_changed {
        println!("✅ README version marker and generated latest-series block already up to date");
    } else {
        println!("✅ Synced README title/version marker and generated latest-series block");
    }
    Ok(())
}

fn sync_vendored_zjstatus(repo_root: &Path) -> Result<(), String> {
    let target_dir = repo_root.join("configs").join("zellij").join("plugins");
    fs::create_dir_all(&target_dir)
        .map_err(|error| format!("Failed to create {}: {}", target_dir.display(), error))?;

    println!("🔄 Refreshing vendored zjstatus.wasm...");
    let package = resolve_locked_zjstatus_store_root(repo_root)?;
    let store_path = package.store_root.join("bin").join("zjstatus.wasm");
    if !store_path.is_file() {
        return Err(format!(
            "zjstatus wasm not found at: {}",
            store_path.display()
        ));
    }

    let bytes = fs::read(&store_path)
        .map_err(|error| format!("Failed to read {}: {}", store_path.display(), error))?;
    if bytes.len() < 1024 {
        return Err(format!(
            "Nix-provided zjstatus wasm is too small to be valid (size={} bytes)",
            bytes.len()
        ));
    }

    let target_path = target_dir.join("zjstatus.wasm");
    let tmp_path = target_dir.join("zjstatus.wasm.tmp");
    fs::write(&tmp_path, bytes)
        .map_err(|error| format!("Failed to write temporary zjstatus file: {error}"))?;
    fs::rename(&tmp_path, &target_path).map_err(|error| {
        format!(
            "Failed to move {} into place: {}",
            target_path.display(),
            error
        )
    })?;
    println!(
        "✅ Updated vendored zjstatus at: {} (size={} bytes, source={})",
        target_path.display(),
        fs::metadata(&target_path)
            .map_err(|error| format!("Failed to inspect {}: {}", target_path.display(), error))?
            .len(),
        package.flake_ref
    );
    Ok(())
}

fn resolve_locked_zjstatus_store_root(repo_root: &Path) -> Result<ZjstatusPackage, String> {
    let lock_path = repo_root.join("flake.lock");
    let lock_raw = fs::read_to_string(&lock_path)
        .map_err(|error| format!("Failed to read {}: {}", lock_path.display(), error))?;
    let lock: JsonValue = serde_json::from_str(&lock_raw)
        .map_err(|error| format!("Invalid JSON in {}: {}", lock_path.display(), error))?;
    let locked = &lock["nodes"]["zjstatus"]["locked"];
    let owner = locked["owner"]
        .as_str()
        .ok_or_else(|| "flake.lock is missing nodes.zjstatus.locked.owner".to_string())?;
    let repo = locked["repo"]
        .as_str()
        .ok_or_else(|| "flake.lock is missing nodes.zjstatus.locked.repo".to_string())?;
    let rev = locked["rev"]
        .as_str()
        .ok_or_else(|| "flake.lock is missing nodes.zjstatus.locked.rev".to_string())?;
    let system = resolve_current_system()?;
    let flake_ref = format!("github:{owner}/{repo}/{rev}#packages.{system}.default");
    let output = run_command_capture(
        "nix",
        ["build", "--no-link", "--print-out-paths", &flake_ref],
        None,
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed to build zjstatus flake ref: {}\n{}",
            flake_ref,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(ZjstatusPackage {
        flake_ref,
        store_root: PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()),
    })
}

fn resolve_current_system() -> Result<String, String> {
    let output = run_command_capture(
        "nix",
        [
            "eval",
            "--impure",
            "--raw",
            "--expr",
            "builtins.currentSystem",
        ],
        None,
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed to resolve current Nix system\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let system = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if system.is_empty() {
        return Err("Failed to resolve current Nix system".to_string());
    }
    Ok(system)
}

fn sync_vendored_yazi_plugins(repo_root: &Path) -> Result<(), String> {
    ensure_command_available("git", "git is not available in PATH")?;
    println!("🔄 Refreshing vendored Yazi plugin runtime files...");
    let manifest_path = repo_root.join(DEFAULT_VENDORED_YAZI_MANIFEST_RELATIVE_PATH);
    let manifest = load_vendored_yazi_manifest(&manifest_path)?;

    for plugin in &manifest.plugins {
        ensure_clean_managed_targets(repo_root, plugin)?;
        if plugin.pinned_rev.trim().is_empty() {
            return Err(format!(
                "Vendored Yazi plugin entry is missing a pinned revision: {}",
                plugin.name
            ));
        }
        refresh_vendored_yazi_plugin(repo_root, plugin)?;
    }

    let names = manifest
        .plugins
        .iter()
        .map(|plugin| plugin.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    println!("Vendored Yazi plugin runtime files are in sync: {}", names);
    Ok(())
}

fn load_vendored_yazi_manifest(path: &Path) -> Result<VendoredYaziManifest, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    let manifest: VendoredYaziManifest = toml::from_str(&raw)
        .map_err(|error| format!("Invalid TOML in {}: {}", path.display(), error))?;
    if manifest.plugins.is_empty() {
        return Err(format!(
            "Vendored Yazi plugin manifest has no plugin entries: {}",
            path.display()
        ));
    }
    Ok(manifest)
}

fn ensure_clean_managed_targets(
    repo_root: &Path,
    plugin: &VendoredYaziPlugin,
) -> Result<(), String> {
    if plugin.managed_files.is_empty() {
        return Err(format!(
            "Vendored Yazi plugin entry has no managed files: {}",
            plugin.name
        ));
    }
    let target_dir = repo_root.join(&plugin.target_dir);
    let target_paths = plugin
        .managed_files
        .iter()
        .map(|path| target_dir.join(path))
        .collect::<Vec<_>>();
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo_root)
        .arg("status")
        .arg("--porcelain")
        .arg("--");
    for path in &target_paths {
        command.arg(path);
    }
    let output = command.output().map_err(|error| {
        format!(
            "Failed to inspect git status for managed Yazi plugin targets under {}: {}",
            target_dir.display(),
            error
        )
    })?;
    if !output.status.success() {
        return Err(format!(
            "Failed to inspect git status for managed Yazi plugin targets under {}: {}",
            target_dir.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let dirty = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if !dirty.is_empty() {
        return Err(format!(
            "Local changes detected in managed vendored plugin files for {}: {}. Commit, stash, or revert them before refreshing vendored Yazi plugins.",
            plugin.name,
            dirty.join("; ")
        ));
    }
    Ok(())
}

fn refresh_vendored_yazi_plugin(
    repo_root: &Path,
    plugin: &VendoredYaziPlugin,
) -> Result<(), String> {
    let checkout_dir = create_unique_temp_dir("yazelix_vendored_yazi_checkout")?;
    let stage_dir = create_unique_temp_dir("yazelix_vendored_yazi_stage")?;

    clone_repo_at_rev(
        &plugin.upstream_repo,
        &plugin.pinned_rev,
        checkout_dir.path(),
    )?;

    let source_root = if plugin.source_subdir.trim().is_empty() || plugin.source_subdir == "." {
        checkout_dir.path().to_path_buf()
    } else {
        checkout_dir.path().join(&plugin.source_subdir)
    };
    if !source_root.exists() {
        return Err(format!(
            "Source subdir missing for vendored Yazi plugin {}: {}",
            plugin.name,
            source_root.display()
        ));
    }

    copy_managed_files(&source_root, stage_dir.path(), &plugin.managed_files)?;
    apply_patch_overlay(repo_root, stage_dir.path(), &plugin.patch_file)?;
    install_staged_files(
        stage_dir.path(),
        &repo_root.join(&plugin.target_dir),
        &plugin.managed_files,
    )?;

    println!(
        "Updated vendored Yazi plugin runtime files for {} from {}",
        plugin.name, plugin.pinned_rev
    );
    Ok(())
}

fn clone_repo_at_rev(repo_url: &str, rev: &str, checkout_dir: &Path) -> Result<(), String> {
    let clone = run_command_capture(
        "git",
        [
            "clone",
            "--quiet",
            repo_url,
            checkout_dir.to_string_lossy().as_ref(),
        ],
        None,
    )?;
    if !clone.status.success() {
        return Err(format!(
            "Failed to clone {}: {}",
            repo_url,
            String::from_utf8_lossy(&clone.stderr).trim()
        ));
    }

    let checkout = run_command_capture(
        "git",
        [
            "-C",
            checkout_dir.to_string_lossy().as_ref(),
            "checkout",
            "--quiet",
            rev,
        ],
        None,
    )?;
    if !checkout.status.success() {
        return Err(format!(
            "Failed to checkout revision {} for {}: {}",
            rev,
            repo_url,
            String::from_utf8_lossy(&checkout.stderr).trim()
        ));
    }
    Ok(())
}

fn copy_managed_files(
    source_root: &Path,
    stage_root: &Path,
    managed_files: &[String],
) -> Result<(), String> {
    for relative in managed_files {
        let source_path = source_root.join(relative);
        if !source_path.is_file() {
            return Err(format!(
                "Managed vendored file not found in upstream source: {}",
                source_path.display()
            ));
        }
        let target_path = stage_root.join(relative);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
        }
        fs::copy(&source_path, &target_path).map_err(|error| {
            format!(
                "Failed to stage {} into {}: {}",
                source_path.display(),
                target_path.display(),
                error
            )
        })?;
    }
    Ok(())
}

fn apply_patch_overlay(
    repo_root: &Path,
    stage_root: &Path,
    patch_file: &str,
) -> Result<(), String> {
    let trimmed = patch_file.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let patch_path = repo_root.join(trimmed);
    if !patch_path.is_file() {
        return Err(format!(
            "Vendored Yazi plugin patch not found: {}",
            patch_path.display()
        ));
    }
    let output = run_command_capture(
        "git",
        ["apply", patch_path.to_string_lossy().as_ref()],
        Some(stage_root),
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed to apply vendored Yazi plugin patch {}: {}",
            patch_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(())
}

fn install_staged_files(
    stage_root: &Path,
    target_root: &Path,
    managed_files: &[String],
) -> Result<(), String> {
    fs::create_dir_all(target_root)
        .map_err(|error| format!("Failed to create {}: {}", target_root.display(), error))?;
    for relative in managed_files {
        let source_path = stage_root.join(relative);
        let target_path = target_root.join(relative);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
        }
        fs::copy(&source_path, &target_path).map_err(|error| {
            format!(
                "Failed to install {} into {}: {}",
                source_path.display(),
                target_path.display(),
                error
            )
        })?;
    }
    Ok(())
}

fn activate_updated_profile_runtime(repo_root: &Path) -> Result<(), String> {
    println!("🔄 Activating updated local Yazelix package in the default Nix profile...");
    println!(
        "   Streaming local profile activation logs (this may take a while when Nix rebuilds)..."
    );
    let existing_entries = find_default_profile_yazelix_entries()?;
    if !existing_entries.is_empty() {
        println!(
            "   Removing existing Yazelix profile entries before installing the local checkout: {}",
            existing_entries.join(", ")
        );
        let mut command = Command::new("nix");
        command.arg("profile").arg("remove");
        for entry in &existing_entries {
            command.arg(entry);
        }
        let output = command
            .output()
            .map_err(|error| format!("Failed to run `nix profile remove`: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "Failed to remove existing Yazelix profile entries with `nix profile remove {}`: {}",
                existing_entries.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
    }

    run_command_streaming(
        "nix",
        ["profile", "add", "--refresh", "-L", ".#yazelix"],
        Some(repo_root),
    )?;
    println!("✅ Default-profile Yazelix package updated from the local checkout.");
    Ok(())
}

fn find_default_profile_yazelix_entries() -> Result<Vec<String>, String> {
    let output = run_command_capture("nix", ["profile", "list", "--json"], None)?;
    if !output.status.success() {
        return Err(format!(
            "Failed to inspect the default Nix profile: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let parsed: JsonValue = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Failed to parse `nix profile list --json`: {error}"))?;
    let elements = parsed["elements"]
        .as_object()
        .cloned()
        .unwrap_or_else(JsonMap::new);
    let mut names = Vec::new();
    for (name, entry) in elements {
        if is_yazelix_profile_entry(&name, &entry) {
            names.push(name);
        }
    }
    Ok(names)
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

fn activate_updated_home_manager_runtime(
    flake_dir: &Path,
    input_name: &str,
    attr: &str,
) -> Result<HomeManagerActivation, String> {
    ensure_command_available(
        "home-manager",
        "home-manager not found in PATH.\n   Recovery: Install Home Manager first, or use `yzx dev update --activate profile` or `--activate none`.",
    )?;
    let resolved_dir = resolve_home_manager_flake_dir(flake_dir)?;
    let switch_ref = build_home_manager_switch_ref(&resolved_dir, attr);
    refresh_home_manager_input_lock(&resolved_dir, input_name)?;

    println!("🔄 Applying updated Home Manager Yazelix configuration...");
    run_command_streaming(
        "home-manager",
        ["switch", "--flake", switch_ref.as_str()],
        None,
    )?;
    println!("✅ Home Manager configuration applied.");
    Ok(HomeManagerActivation { switch_ref })
}

fn resolve_home_manager_flake_dir(candidate: &Path) -> Result<PathBuf, String> {
    let expanded = candidate.canonicalize().map_err(|error| {
        format!(
            "Home Manager flake directory not found: {} ({error})",
            candidate.display()
        )
    })?;
    let flake_file = expanded.join("flake.nix");
    if !flake_file.is_file() {
        return Err(format!(
            "Home Manager flake is missing flake.nix: {}",
            flake_file.display()
        ));
    }
    Ok(expanded)
}

fn build_home_manager_switch_ref(flake_dir: &Path, attr: &str) -> String {
    let trimmed = attr.trim();
    if trimmed.is_empty() {
        flake_dir.display().to_string()
    } else {
        format!("{}#{}", flake_dir.display(), trimmed)
    }
}

fn refresh_home_manager_input_lock(flake_dir: &Path, input_name: &str) -> Result<(), String> {
    let trimmed = input_name.trim();
    if trimmed.is_empty() {
        return Err("Home Manager activation requires a non-empty input name.".to_string());
    }

    println!("🔄 Refreshing Home Manager Yazelix input...");
    run_command_streaming(
        "nix",
        [
            "flake",
            "update",
            trimmed,
            "--flake",
            flake_dir.to_string_lossy().as_ref(),
        ],
        None,
    )?;
    println!("✅ Home Manager flake input updated.");
    Ok(())
}

fn run_update_canaries(
    repo_root: &Path,
    selected: &[String],
) -> Result<Vec<UpdateCanaryResult>, String> {
    let context = materialize_update_canaries(repo_root, selected)?;
    let mut results = Vec::new();
    for canary in &context.canaries {
        println!("🧪 Canary: {} — {}", canary.name, canary.description);
        results.push(run_update_canary(repo_root, canary)?);
    }
    Ok(results)
}

fn materialize_update_canaries(
    repo_root: &Path,
    selected: &[String],
) -> Result<UpdateCanaryContext, String> {
    let default_config_path = repo_root.join(DEFAULT_MAIN_CONFIG_RELATIVE_PATH);
    if !default_config_path.is_file() {
        return Err(format!(
            "Default config not found: {}",
            default_config_path.display()
        ));
    }
    let temp_base = home_dir()?.join(UPDATE_CANARY_BASE_RELATIVE_PATH);
    fs::create_dir_all(&temp_base)
        .map_err(|error| format!("Failed to create {}: {}", temp_base.display(), error))?;
    let temp_dir = create_unique_temp_dir_in(&temp_base, "update")?;
    let template_raw = fs::read_to_string(&default_config_path).map_err(|error| {
        format!(
            "Failed to read {}: {}",
            default_config_path.display(),
            error
        )
    })?;
    let template_value: TomlValue = template_raw.parse::<TomlValue>().map_err(|error| {
        format!(
            "Invalid TOML in {}: {}",
            default_config_path.display(),
            error
        )
    })?;

    let mut canaries = Vec::new();
    for name in selected {
        match name.as_str() {
            "default" => canaries.push(UpdateCanary {
                name: "default".to_string(),
                config_path: default_config_path.clone(),
                description: "default v15 runtime config".to_string(),
            }),
            "shell_layout" => {
                let config_dir = temp_dir.path().join("shell_layout");
                fs::create_dir_all(&config_dir).map_err(|error| {
                    format!("Failed to create {}: {}", config_dir.display(), error)
                })?;
                let config_path = config_dir.join("yazelix.toml");
                let mut config = template_value.clone();
                apply_shell_layout_canary_overrides(&mut config)?;
                fs::write(
                    &config_path,
                    toml::to_string(&config).map_err(|error| {
                        format!("Failed to render shell_layout canary TOML: {error}")
                    })?,
                )
                .map_err(|error| format!("Failed to write {}: {}", config_path.display(), error))?;
                canaries.push(UpdateCanary {
                    name: "shell_layout".to_string(),
                    config_path,
                    description: "zsh entry, neovim editor, collapsed-sidebar layout".to_string(),
                });
            }
            other => {
                return Err(format!("Unsupported update canary: {other}"));
            }
        }
    }

    Ok(UpdateCanaryContext {
        _temp_dir: temp_dir,
        canaries,
    })
}

fn apply_shell_layout_canary_overrides(config: &mut TomlValue) -> Result<(), String> {
    let root = config
        .as_table_mut()
        .ok_or_else(|| "Default config must be a TOML table".to_string())?;
    set_nested_toml_string(root, &["shell", "default_shell"], "zsh");
    set_nested_toml_string(root, &["editor", "command"], "nvim");
    set_nested_toml_string(root, &["editor", "initial_sidebar_state"], "closed");
    Ok(())
}

fn set_nested_toml_string(table: &mut toml::Table, path: &[&str], value: &str) {
    set_nested_toml_value(table, path, TomlValue::String(value.to_string()));
}

fn set_nested_toml_value(table: &mut toml::Table, path: &[&str], value: TomlValue) {
    if path.len() == 1 {
        table.insert(path[0].to_string(), value);
        return;
    }
    let child = table
        .entry(path[0].to_string())
        .or_insert_with(|| TomlValue::Table(toml::Table::new()));
    if !child.is_table() {
        *child = TomlValue::Table(toml::Table::new());
    }
    let child_table = child.as_table_mut().expect("table");
    set_nested_toml_value(child_table, &path[1..], value);
}

fn run_update_canary(
    repo_root: &Path,
    canary: &UpdateCanary,
) -> Result<UpdateCanaryResult, String> {
    let config_parent = canary.config_path.parent().ok_or_else(|| {
        format!(
            "Config path has no parent: {}",
            canary.config_path.display()
        )
    })?;
    let config_dir = if config_parent
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        == "user_configs"
    {
        config_parent
            .parent()
            .ok_or_else(|| {
                format!(
                    "Config parent has no grandparent: {}",
                    config_parent.display()
                )
            })?
            .to_path_buf()
    } else {
        config_parent.to_path_buf()
    };
    let yzx_core_bin = resolve_update_yzx_core_bin(repo_root)?;
    let output = Command::new(&yzx_core_bin)
        .args(["runtime-materialization.repair", "--from-env", "--force"])
        .env("YAZELIX_CONFIG_OVERRIDE", &canary.config_path)
        .env("YAZELIX_CONFIG_DIR", &config_dir)
        .env("YAZELIX_RUNTIME_DIR", repo_root)
        .output()
        .map_err(|error| {
            format!(
                "Failed to launch update canary helper {}: {}",
                yzx_core_bin.display(),
                error
            )
        })?;
    let stdout_tail = trim_output_tail(&String::from_utf8_lossy(&output.stdout), 25);
    let stderr_tail = trim_output_tail(&String::from_utf8_lossy(&output.stderr), 25);
    Ok(UpdateCanaryResult {
        name: canary.name.clone(),
        config_path: canary.config_path.clone(),
        description: canary.description.clone(),
        exit_code: output.status.code().unwrap_or(1),
        stdout_tail,
        stderr_tail,
        ok: output.status.success(),
    })
}

fn resolve_update_yzx_core_bin(repo_root: &Path) -> Result<PathBuf, String> {
    if let Ok(explicit) = std::env::var("YAZELIX_YZX_CORE_BIN") {
        let candidate = PathBuf::from(expand_tilde_if_needed(&explicit)?);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!(
            "YAZELIX_YZX_CORE_BIN points to a missing helper: {}",
            candidate.display()
        ));
    }
    let candidate = repo_root.join("libexec").join("yzx_core");
    if candidate.is_file() {
        return Ok(candidate);
    }
    Err(format!(
        "Yazelix Rust helper not found for update canary repair: {}",
        candidate.display()
    ))
}

fn print_update_canary_summary(results: &[UpdateCanaryResult]) {
    println!();
    println!("Canary summary:");
    for result in results {
        let icon = if result.ok { "✅" } else { "❌" };
        println!("  {} {} — {}", icon, result.name, result.description);
    }
}

fn print_update_canary_failure_details(results: &[UpdateCanaryResult]) {
    let failures = results
        .iter()
        .filter(|result| !result.ok)
        .collect::<Vec<_>>();
    if failures.is_empty() {
        return;
    }
    println!();
    println!("Failed canary details:");
    for failure in failures {
        println!("  ❌ {}", failure.name);
        println!("     Config: {}", failure.config_path.display());
        println!("     Exit code: {}", failure.exit_code);
        if !failure.stderr_tail.is_empty() {
            println!("     stderr tail:");
            for line in failure.stderr_tail.lines() {
                println!("       {}", line);
            }
        } else if !failure.stdout_tail.is_empty() {
            println!("     stdout tail:");
            for line in failure.stdout_tail.lines() {
                println!("       {}", line);
            }
        }
    }
}

fn trim_output_tail(text: &str, max_lines: usize) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lines = trimmed.lines().collect::<Vec<_>>();
    if lines.len() <= max_lines {
        trimmed.to_string()
    } else {
        lines[lines.len() - max_lines..].join("\n")
    }
}

fn command_exists(name: &str) -> bool {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn ensure_command_available(name: &str, message: &str) -> Result<(), String> {
    if command_exists(name) {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

fn run_command_capture<I, S>(
    program: &str,
    args: I,
    current_dir: Option<&Path>,
) -> Result<Output, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    let mut command = Command::new(program);
    command.args(&args_vec);
    if let Some(dir) = current_dir {
        command.current_dir(dir);
    }
    command.output().map_err(|error| {
        format!(
            "Failed to launch `{program} {}`: {error}",
            args_vec.join(" ")
        )
    })
}

fn run_command_streaming<I, S>(
    program: &str,
    args: I,
    current_dir: Option<&Path>,
) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args_vec = args
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect::<Vec<_>>();
    let mut command = Command::new(program);
    command
        .args(&args_vec)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(dir) = current_dir {
        command.current_dir(dir);
    }
    let status = command.status().map_err(|error| {
        format!(
            "Failed to launch `{program} {}`: {error}",
            args_vec.join(" ")
        )
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "`{program} {}` failed with exit code {}",
            args_vec.join(" "),
            status.code().unwrap_or(1)
        ))
    }
}

fn home_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    Ok(PathBuf::from(home))
}

fn expand_user_path(path: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(expand_tilde_if_needed(path)?))
}

fn expand_tilde_if_needed(path: &str) -> Result<String, String> {
    if path == "~" {
        return Ok(home_dir()?.display().to_string());
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return Ok(home_dir()?.join(rest).display().to_string());
    }
    if let Some(rest) = path.strip_prefix("$HOME/") {
        return Ok(home_dir()?.join(rest).display().to_string());
    }
    if path == "$HOME" {
        return Ok(home_dir()?.display().to_string());
    }
    Ok(path.to_string())
}

fn create_unique_temp_dir(prefix: &str) -> Result<TempDirGuard, String> {
    create_unique_temp_dir_in(&std::env::temp_dir(), prefix)
}

fn create_unique_temp_dir_in(base: &Path, prefix: &str) -> Result<TempDirGuard, String> {
    fs::create_dir_all(base)
        .map_err(|error| format!("Failed to create {}: {}", base.display(), error))?;
    let pid = std::process::id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock error: {error}"))?
        .as_nanos();
    for attempt in 0..32 {
        let candidate = base.join(format!("{prefix}_{pid}_{now}_{attempt}"));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(TempDirGuard::new(candidate)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Failed to create {}: {}",
                    candidate.display(),
                    error
                ));
            }
        }
    }
    Err(format!(
        "Failed to allocate a unique temp dir under {}",
        base.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        UpdateActivationMode, apply_shell_layout_canary_overrides,
        resolve_requested_update_activation_mode, resolve_update_canary_selection,
        update_constant_value,
    };
    use toml::Value as TomlValue;

    // Defends: maintainer updates require an explicit activation mode on real updates but allow omitted activation for canary-only runs.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn real_updates_require_explicit_activation_mode() {
        let err = resolve_requested_update_activation_mode("", false).unwrap_err();
        assert!(err.contains("--activate profile|home_manager|none"));
        assert_eq!(
            resolve_requested_update_activation_mode("profile", false).unwrap(),
            UpdateActivationMode::Profile
        );
        assert_eq!(
            resolve_requested_update_activation_mode("", true).unwrap(),
            UpdateActivationMode::None
        );
    }

    // Defends: the shell-layout update canary forces the maintained zsh+nvim+collapsed-sidebar override set instead of mutating unrelated config fields.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn shell_layout_canary_overrides_expected_fields() {
        let mut value: TomlValue = toml::from_str(
            r#"
[shell]
default_shell = "nu"

[editor]
command = "hx"
initial_sidebar_state = "open"
"#,
        )
        .unwrap();
        apply_shell_layout_canary_overrides(&mut value).unwrap();
        let table = value.as_table().unwrap();
        assert_eq!(
            table["shell"].as_table().unwrap()["default_shell"]
                .as_str()
                .unwrap(),
            "zsh"
        );
        assert_eq!(
            table["editor"].as_table().unwrap()["command"]
                .as_str()
                .unwrap(),
            "nvim"
        );
        assert_eq!(
            table["editor"].as_table().unwrap()["initial_sidebar_state"]
                .as_str()
                .unwrap(),
            "closed"
        );
    }

    // Defends: runtime pin updates replace only the targeted constants and preserve surrounding file content.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn constant_updates_only_touch_targeted_exports() {
        let input = "export const YAZELIX_VERSION = \"v15.4\"\nexport const PINNED_NIX_VERSION = \"1.0.0\"\nexport const PINNED_NUSHELL_VERSION = \"2.0.0\"\n";
        let output = update_constant_value(
            &update_constant_value(input, "PINNED_NIX_VERSION", "3.4.5"),
            "PINNED_NUSHELL_VERSION",
            "0.111.0",
        );
        assert!(output.contains("export const YAZELIX_VERSION = \"v15.4\""));
        assert!(output.contains("export const PINNED_NIX_VERSION = \"3.4.5\""));
        assert!(output.contains("export const PINNED_NUSHELL_VERSION = \"0.111.0\""));
    }

    // Defends: update canary selection accepts only the maintained allowlist and deduplicates repeated requests.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn canary_selection_rejects_unknown_names_and_deduplicates() {
        assert_eq!(
            resolve_update_canary_selection(&["default".into(), "default".into()]).unwrap(),
            vec!["default".to_string()]
        );
        let err = resolve_update_canary_selection(&["unknown".into()]).unwrap_err();
        assert!(err.contains("Unknown canary name"));
    }
}
