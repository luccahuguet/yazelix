use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use yazelix_core::control_plane::state_dir_from_env;

const BUILD_TARGET: &str = "wasm32-wasip1";
const PANE_ORCHESTRATOR_SOURCE_ENV: &str = "YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_SOURCE_DIR";
const PANE_ORCHESTRATOR_SOURCE_PROJECT: &str = "yazelix-zellij-pane-orchestrator";
const PANE_ORCHESTRATOR_PUBLIC_WASM_NAME: &str = "yazelix_zellij_pane_orchestrator.wasm";
const PANE_ORCHESTRATOR_WASM_NAME: &str = "yazelix_pane_orchestrator.wasm";
const PANE_ORCHESTRATOR_SYNC_STAMP_NAME: &str = "yazelix_pane_orchestrator.sync_stamp";
const PANE_ORCHESTRATOR_BUILD_COMMAND: &str = "yzx dev build_pane_orchestrator --sync";
const YZPP_SOURCE_ENV: &str = "YAZELIX_ZELLIJ_POPUP_SOURCE_DIR";
const YZPP_SOURCE_PROJECT: &str = "yazelix-zellij-popup";
const YZPP_WASM_NAME: &str = "yzpp.wasm";
const YZPP_SYNC_STAMP_NAME: &str = "yzpp.sync_stamp";
const YZPP_PACKAGE_WASM_PATH: &str = "share/yazelix_zellij_popup/yzpp.wasm";
const YZPP_SYNC_COMMAND: &str = "yzx dev sync_yzpp_wasm";

struct PaneOrchestratorPaths {
    repo_root: PathBuf,
    crate_dir: PathBuf,
    wasm_path: PathBuf,
}

struct YzppPaths {
    source_dir: PathBuf,
}

pub fn build_pane_orchestrator(repo_root: &Path, sync: bool) -> Result<(), String> {
    let paths = pane_orchestrator_paths(repo_root);
    ensure_build_tools_available()?;
    let source_git = if sync {
        Some(clean_pane_orchestrator_source_git(&paths.crate_dir)?)
    } else {
        None
    };
    run_wasm_build(&paths, "pane orchestrator")?;

    if sync {
        sync_built_wasm(&paths, "pane orchestrator", source_git.as_ref().unwrap())?;
    }

    Ok(())
}

pub fn validate_pane_orchestrator_sync(repo_root: &Path) -> Result<Vec<String>, String> {
    let paths = pane_orchestrator_paths(repo_root);
    let repo_wasm = tracked_wasm_path(repo_root);
    let sync_stamp_path = tracked_sync_stamp_path(repo_root);
    let mut errors = Vec::new();

    if !repo_wasm.is_file() {
        errors.push(format!(
            "Missing tracked pane-orchestrator wasm: {}",
            repo_wasm.display()
        ));
    }

    match read_sync_stamp(&sync_stamp_path)? {
        None => errors.push(format!(
            "Missing pane-orchestrator sync stamp: {}. Run `yzx dev build_pane_orchestrator --sync`.",
            sync_stamp_path.display()
        )),
        Some(stamp) => {
            if stamp.source_project != PANE_ORCHESTRATOR_SOURCE_PROJECT {
                errors.push(format!(
                    "Pane-orchestrator sync stamp points at `{}` instead of `{}`.",
                    stamp.source_project, PANE_ORCHESTRATOR_SOURCE_PROJECT
                ));
            }
            if paths.crate_dir.is_dir() {
                let current_source_hash = pane_orchestrator_source_hash(&paths)?;
                if stamp.source_sha256 != current_source_hash {
                    errors.push(format!(
                        "Pane-orchestrator source changed since the tracked wasm was synced. Run `yzx dev build_pane_orchestrator --sync`.\n   current source sha256: {}\n   synced source sha256:  {}",
                        current_source_hash, stamp.source_sha256
                    ));
                }
                let source_git = pane_orchestrator_source_git(&paths.crate_dir)?;
                if source_git.dirty {
                    errors.push(format!(
                        "Pane-orchestrator source checkout is dirty: {}. Commit or discard source changes before syncing the tracked wasm.",
                        paths.crate_dir.display()
                    ));
                }
                if stamp.source_git_commit != source_git.commit {
                    errors.push(format!(
                        "Pane-orchestrator source Git commit differs from its sync stamp. Run `yzx dev build_pane_orchestrator --sync`.\n   current source commit: {}\n   synced source commit:  {}",
                        source_git.commit, stamp.source_git_commit
                    ));
                }
                if stamp.source_git_remote != source_git.remote {
                    errors.push(format!(
                        "Pane-orchestrator source Git remote differs from its sync stamp.\n   current source remote: {}\n   synced source remote:  {}",
                        source_git.remote, stamp.source_git_remote
                    ));
                }
            }
            if stamp.build_command != PANE_ORCHESTRATOR_BUILD_COMMAND {
                errors.push(format!(
                    "Pane-orchestrator sync stamp build command is `{}` instead of `{}`.",
                    stamp.build_command, PANE_ORCHESTRATOR_BUILD_COMMAND
                ));
            }
            if repo_wasm.is_file() {
                let current_wasm_hash = file_sha256_hex(&repo_wasm)?;
                if stamp.tracked_wasm_sha256 != current_wasm_hash {
                    errors.push(format!(
                        "Tracked pane-orchestrator wasm hash does not match its sync stamp. Run `yzx dev build_pane_orchestrator --sync`.\n   current wasm sha256: {}\n   stamped wasm sha256: {}",
                        current_wasm_hash, stamp.tracked_wasm_sha256
                    ));
                }
            }
        }
    }

    if paths.wasm_path.is_file() && repo_wasm.is_file() {
        let built_wasm_hash = file_sha256_hex(&paths.wasm_path)?;
        let tracked_wasm_hash = file_sha256_hex(&repo_wasm)?;
        if built_wasm_hash != tracked_wasm_hash {
            errors.push(format!(
                "Built pane-orchestrator wasm differs from the tracked wasm. Run `yzx dev build_pane_orchestrator --sync`.\n   built wasm sha256:   {}\n   tracked wasm sha256: {}",
                built_wasm_hash, tracked_wasm_hash
            ));
        }
    }

    Ok(errors)
}

pub fn sync_yzpp_wasm(repo_root: &Path) -> Result<(), String> {
    let paths = yzpp_paths(repo_root);
    let source_git = clean_source_git(&paths.source_dir, YZPP_SOURCE_PROJECT, YZPP_SYNC_COMMAND)?;
    let package_wasm = build_yzpp_package(&paths.source_dir)?;
    let tracked_wasm = tracked_zellij_plugin_path(repo_root, YZPP_WASM_NAME);

    make_writable_if_exists(&tracked_wasm)?;
    fs::copy(&package_wasm, &tracked_wasm).map_err(|error| {
        format!(
            "Failed to copy yzpp wasm into tracked repo path {}: {error}",
            tracked_wasm.display()
        )
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tracked_wasm, fs::Permissions::from_mode(0o755)).map_err(|error| {
            format!(
                "Failed to mark tracked yzpp wasm executable at {}: {error}",
                tracked_wasm.display()
            )
        })?;
    }

    let sync_stamp = render_yzpp_sync_stamp(&paths, &tracked_wasm, &package_wasm, &source_git)?;
    let sync_stamp_path = tracked_zellij_plugin_path(repo_root, YZPP_SYNC_STAMP_NAME);
    fs::write(&sync_stamp_path, sync_stamp).map_err(|error| {
        format!(
            "Failed to write yzpp sync stamp {}: {error}",
            sync_stamp_path.display()
        )
    })?;

    println!(
        "Updated yzpp tracked wasm: {}\nUpdated yzpp sync stamp: {}\nSource commit: {}",
        tracked_wasm.display(),
        sync_stamp_path.display(),
        source_git.commit
    );
    Ok(())
}

pub fn validate_yzpp_sync(repo_root: &Path) -> Result<Vec<String>, String> {
    let paths = yzpp_paths(repo_root);
    let tracked_wasm = tracked_zellij_plugin_path(repo_root, YZPP_WASM_NAME);
    let sync_stamp_path = tracked_zellij_plugin_path(repo_root, YZPP_SYNC_STAMP_NAME);
    let mut errors = Vec::new();

    if !tracked_wasm.is_file() {
        errors.push(format!(
            "Missing tracked yzpp wasm: {}",
            tracked_wasm.display()
        ));
    }

    match read_yzpp_sync_stamp(&sync_stamp_path)? {
        None => errors.push(format!(
            "Missing yzpp sync stamp: {}. Run `{}`.",
            sync_stamp_path.display(),
            YZPP_SYNC_COMMAND
        )),
        Some(stamp) => {
            if stamp.source_project != YZPP_SOURCE_PROJECT {
                errors.push(format!(
                    "yzpp sync stamp points at `{}` instead of `{}`.",
                    stamp.source_project, YZPP_SOURCE_PROJECT
                ));
            }
            if stamp.build_command != YZPP_SYNC_COMMAND {
                errors.push(format!(
                    "yzpp sync stamp build command is `{}` instead of `{}`.",
                    stamp.build_command, YZPP_SYNC_COMMAND
                ));
            }
            if paths.source_dir.is_dir() {
                let current_source_hash = zellij_popup_source_hash(&paths.source_dir)?;
                if stamp.source_sha256 != current_source_hash {
                    errors.push(format!(
                        "yazelix-zellij-popup source changed since yzpp.wasm was synced. Run `{}`.\n   current source sha256: {}\n   synced source sha256:  {}",
                        YZPP_SYNC_COMMAND, current_source_hash, stamp.source_sha256
                    ));
                }
                let source_git = source_git(YZPP_SOURCE_PROJECT, &paths.source_dir)?;
                if source_git.dirty {
                    errors.push(format!(
                        "yazelix-zellij-popup source checkout is dirty: {}. Commit or discard source changes before syncing yzpp.wasm.",
                        paths.source_dir.display()
                    ));
                }
                if stamp.source_git_commit != source_git.commit {
                    errors.push(format!(
                        "yazelix-zellij-popup source Git commit differs from its sync stamp. Run `{}`.\n   current source commit: {}\n   synced source commit:  {}",
                        YZPP_SYNC_COMMAND, source_git.commit, stamp.source_git_commit
                    ));
                }
                if stamp.source_git_remote != source_git.remote {
                    errors.push(format!(
                        "yazelix-zellij-popup source Git remote differs from its sync stamp.\n   current source remote: {}\n   synced source remote:  {}",
                        source_git.remote, stamp.source_git_remote
                    ));
                }
            }
            if tracked_wasm.is_file() {
                let current_wasm_hash = file_sha256_hex(&tracked_wasm)?;
                if stamp.tracked_wasm_sha256 != current_wasm_hash {
                    errors.push(format!(
                        "Tracked yzpp wasm hash does not match its sync stamp. Run `{}`.\n   current wasm sha256: {}\n   stamped wasm sha256: {}",
                        YZPP_SYNC_COMMAND, current_wasm_hash, stamp.tracked_wasm_sha256
                    ));
                }
            }
        }
    }

    Ok(errors)
}

fn pane_orchestrator_paths(repo_root: &Path) -> PaneOrchestratorPaths {
    let crate_dir = pane_orchestrator_source_dir(repo_root);
    let wasm_path = crate_dir
        .join("target")
        .join(BUILD_TARGET)
        .join("release")
        .join(PANE_ORCHESTRATOR_PUBLIC_WASM_NAME);

    PaneOrchestratorPaths {
        repo_root: repo_root.to_path_buf(),
        crate_dir,
        wasm_path,
    }
}

fn yzpp_paths(repo_root: &Path) -> YzppPaths {
    YzppPaths {
        source_dir: zellij_popup_source_dir(repo_root),
    }
}

pub(crate) fn pane_orchestrator_source_dir(repo_root: &Path) -> PathBuf {
    env::var_os(PANE_ORCHESTRATOR_SOURCE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo_root
                .parent()
                .unwrap_or(repo_root)
                .join(PANE_ORCHESTRATOR_SOURCE_PROJECT)
        })
}

fn zellij_popup_source_dir(repo_root: &Path) -> PathBuf {
    env::var_os(YZPP_SOURCE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo_root
                .parent()
                .unwrap_or(repo_root)
                .join(YZPP_SOURCE_PROJECT)
        })
}

fn print_rust_wasi_enable_hint() {
    println!("   Install a WASI-capable Rust toolchain in your maintainer environment.");
    println!(
        "   Example: run the build inside the repo's maintainer shell, or use `rustup target add wasm32-wasip1`."
    );
}

fn ensure_build_tools_available() -> Result<(), String> {
    let mut missing = Vec::new();
    if !command_exists("cargo") {
        missing.push("cargo");
    }
    if !command_exists("rustc") {
        missing.push("rustc");
    }

    if missing.is_empty() {
        return Ok(());
    }

    println!("❌ Missing Rust tool(s): {}", missing.join(", "));
    print_rust_wasi_enable_hint();
    Err("Missing Rust toolchain for pane-orchestrator build".to_string())
}

fn command_exists(name: &str) -> bool {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn run_wasm_build(paths: &PaneOrchestratorPaths, label: &str) -> Result<(), String> {
    if !paths.crate_dir.exists() {
        return Err(format!(
            "❌ {label} source checkout not found: {}. Clone `{}` next to the Yazelix repo or set {}.",
            paths.crate_dir.display(),
            PANE_ORCHESTRATOR_SOURCE_PROJECT,
            PANE_ORCHESTRATOR_SOURCE_ENV
        ));
    }

    println!("🦀 Building {label} for target {BUILD_TARGET}...");
    let output = Command::new("cargo")
        .args(["build", "--target", BUILD_TARGET, "--profile", "release"])
        .current_dir(&paths.crate_dir)
        .output()
        .map_err(|error| format!("Failed to launch cargo build: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        println!("{stdout}");
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !stderr.is_empty() {
            eprintln!("{stderr}");
        }
        if stderr.contains("can't find crate for `core`")
            || stderr.contains("can't find crate for `std`")
            || stderr.contains("target may not be installed")
        {
            println!();
            println!("❌ The wasm target stdlib is not available in the current Rust toolchain.");
            print_rust_wasi_enable_hint();
        } else {
            println!();
            println!("❌ {label} build failed.");
        }
        return Err(format!(
            "{label} build failed with exit code {}",
            output.status.code().unwrap_or(1)
        ));
    }

    if !paths.wasm_path.is_file() {
        return Err(format!(
            "❌ Build reported success, but wasm output was not found at: {}",
            paths.wasm_path.display()
        ));
    }

    println!("✅ Built {label} wasm: {}", paths.wasm_path.display());
    Ok(())
}

fn tracked_wasm_path(repo_root: &Path) -> PathBuf {
    tracked_zellij_plugin_path(repo_root, PANE_ORCHESTRATOR_WASM_NAME)
}

fn tracked_zellij_plugin_path(repo_root: &Path, file_name: &str) -> PathBuf {
    repo_root
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join(file_name)
}

fn tracked_sync_stamp_path(repo_root: &Path) -> PathBuf {
    tracked_zellij_plugin_path(repo_root, PANE_ORCHESTRATOR_SYNC_STAMP_NAME)
}

fn runtime_wasm_path() -> Result<PathBuf, String> {
    Ok(state_dir_from_env()
        .map_err(|error| error.message().to_string())?
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join(PANE_ORCHESTRATOR_WASM_NAME))
}

fn sync_built_wasm(
    paths: &PaneOrchestratorPaths,
    label: &str,
    source_git: &SourceGitMetadata,
) -> Result<(), String> {
    println!("🔄 Syncing {label} wasm into Yazelix...");
    let repo_target = tracked_wasm_path(&paths.repo_root);
    let runtime_target = runtime_wasm_path()?;
    let runtime_target_dir = runtime_target
        .parent()
        .ok_or_else(|| "Runtime target path has no parent directory".to_string())?;

    fs::copy(&paths.wasm_path, &repo_target).map_err(|error| {
        format!(
            "Failed to copy pane-orchestrator wasm into tracked repo path {}: {error}",
            repo_target.display()
        )
    })?;
    fs::create_dir_all(runtime_target_dir).map_err(|error| {
        format!(
            "Failed to create runtime plugin directory {}: {error}",
            runtime_target_dir.display()
        )
    })?;
    fs::copy(&paths.wasm_path, &runtime_target).map_err(|error| {
        format!(
            "Failed to copy pane-orchestrator wasm into runtime path {}: {error}",
            runtime_target.display()
        )
    })?;

    let sync_stamp = render_sync_stamp(paths, &repo_target, source_git)?;
    let sync_stamp_path = tracked_sync_stamp_path(&paths.repo_root);
    fs::write(&sync_stamp_path, sync_stamp).map_err(|error| {
        format!(
            "Failed to write pane-orchestrator sync stamp {}: {error}",
            sync_stamp_path.display()
        )
    })?;

    let byte_len = fs::metadata(&paths.wasm_path)
        .map_err(|error| format!("Failed to inspect built wasm size: {error}"))?
        .len();

    println!(
        "Updated pane orchestrator repo wasm: {}\nUpdated pane orchestrator sync stamp: {}\nUpdated pane orchestrator runtime wasm: {}\nSource commit: {}\nSize: {byte_len} bytes\n\nSafest next step:\nRestart Yazelix or open a fresh Yazelix window so Zellij loads the updated plugin cleanly.",
        repo_target.display(),
        sync_stamp_path.display(),
        runtime_target.display(),
        source_git.commit
    );
    println!("In-place plugin reloads can leave the current session in a broken permission state.");
    println!(
        "\nIf you are already stuck in a blank/permission-limbo session, recover with:\nzellij delete-all-sessions -f -y"
    );

    Ok(())
}

struct PaneOrchestratorSyncStamp {
    source_project: String,
    source_git_commit: String,
    source_git_remote: String,
    source_sha256: String,
    build_command: String,
    tracked_wasm_sha256: String,
}

struct YzppSyncStamp {
    source_project: String,
    source_git_commit: String,
    source_git_remote: String,
    source_sha256: String,
    build_command: String,
    tracked_wasm_sha256: String,
}

#[derive(Debug)]
struct SourceGitMetadata {
    commit: String,
    remote: String,
    dirty: bool,
}

fn render_sync_stamp(
    paths: &PaneOrchestratorPaths,
    repo_wasm: &Path,
    source_git: &SourceGitMetadata,
) -> Result<String, String> {
    Ok(format!(
        "source_project = \"{}\"\nsource_git_commit = \"{}\"\nsource_git_remote = \"{}\"\nsource_sha256 = \"{}\"\nbuild_command = \"{}\"\npublic_wasm_sha256 = \"{}\"\ntracked_wasm_sha256 = \"{}\"\n",
        PANE_ORCHESTRATOR_SOURCE_PROJECT,
        source_git.commit,
        source_git.remote,
        pane_orchestrator_source_hash(paths)?,
        PANE_ORCHESTRATOR_BUILD_COMMAND,
        file_sha256_hex(&paths.wasm_path)?,
        file_sha256_hex(repo_wasm)?,
    ))
}

fn read_sync_stamp(path: &Path) -> Result<Option<PaneOrchestratorSyncStamp>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    let mut source_project = None;
    let mut source_git_commit = None;
    let mut source_git_remote = None;
    let mut source_sha256 = None;
    let mut build_command = None;
    let mut tracked_wasm_sha256 = None;
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "Invalid sync stamp line in {}: {line}",
                path.display()
            ));
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.trim() {
            "source_project" => source_project = Some(value),
            "source_git_commit" => source_git_commit = Some(value),
            "source_git_remote" => source_git_remote = Some(value),
            "source_sha256" => source_sha256 = Some(value),
            "build_command" => build_command = Some(value),
            "public_wasm_sha256" => {}
            "tracked_wasm_sha256" => tracked_wasm_sha256 = Some(value),
            other => {
                return Err(format!(
                    "Unknown sync stamp field `{other}` in {}",
                    path.display()
                ));
            }
        }
    }
    Ok(Some(PaneOrchestratorSyncStamp {
        source_project: source_project
            .ok_or_else(|| format!("Missing source_project in {}", path.display()))?,
        source_git_commit: source_git_commit
            .ok_or_else(|| format!("Missing source_git_commit in {}", path.display()))?,
        source_git_remote: source_git_remote
            .ok_or_else(|| format!("Missing source_git_remote in {}", path.display()))?,
        source_sha256: source_sha256
            .ok_or_else(|| format!("Missing source_sha256 in {}", path.display()))?,
        build_command: build_command
            .ok_or_else(|| format!("Missing build_command in {}", path.display()))?,
        tracked_wasm_sha256: tracked_wasm_sha256
            .ok_or_else(|| format!("Missing tracked_wasm_sha256 in {}", path.display()))?,
    }))
}

fn render_yzpp_sync_stamp(
    paths: &YzppPaths,
    tracked_wasm: &Path,
    package_wasm: &Path,
    source_git: &SourceGitMetadata,
) -> Result<String, String> {
    Ok(format!(
        "source_project = \"{}\"\nsource_git_commit = \"{}\"\nsource_git_remote = \"{}\"\nsource_sha256 = \"{}\"\nbuild_command = \"{}\"\npackage_wasm_sha256 = \"{}\"\ntracked_wasm_sha256 = \"{}\"\n",
        YZPP_SOURCE_PROJECT,
        source_git.commit,
        source_git.remote,
        zellij_popup_source_hash(&paths.source_dir)?,
        YZPP_SYNC_COMMAND,
        file_sha256_hex(package_wasm)?,
        file_sha256_hex(tracked_wasm)?,
    ))
}

fn read_yzpp_sync_stamp(path: &Path) -> Result<Option<YzppSyncStamp>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    let mut source_project = None;
    let mut source_git_commit = None;
    let mut source_git_remote = None;
    let mut source_sha256 = None;
    let mut build_command = None;
    let mut tracked_wasm_sha256 = None;
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "Invalid yzpp sync stamp line in {}: {line}",
                path.display()
            ));
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.trim() {
            "source_project" => source_project = Some(value),
            "source_git_commit" => source_git_commit = Some(value),
            "source_git_remote" => source_git_remote = Some(value),
            "source_sha256" => source_sha256 = Some(value),
            "build_command" => build_command = Some(value),
            "package_wasm_sha256" => {}
            "tracked_wasm_sha256" => tracked_wasm_sha256 = Some(value),
            other => {
                return Err(format!(
                    "Unknown yzpp sync stamp field `{other}` in {}",
                    path.display()
                ));
            }
        }
    }
    Ok(Some(YzppSyncStamp {
        source_project: source_project
            .ok_or_else(|| format!("Missing source_project in {}", path.display()))?,
        source_git_commit: source_git_commit
            .ok_or_else(|| format!("Missing source_git_commit in {}", path.display()))?,
        source_git_remote: source_git_remote
            .ok_or_else(|| format!("Missing source_git_remote in {}", path.display()))?,
        source_sha256: source_sha256
            .ok_or_else(|| format!("Missing source_sha256 in {}", path.display()))?,
        build_command: build_command
            .ok_or_else(|| format!("Missing build_command in {}", path.display()))?,
        tracked_wasm_sha256: tracked_wasm_sha256
            .ok_or_else(|| format!("Missing tracked_wasm_sha256 in {}", path.display()))?,
    }))
}

fn clean_pane_orchestrator_source_git(crate_dir: &Path) -> Result<SourceGitMetadata, String> {
    clean_source_git(
        crate_dir,
        PANE_ORCHESTRATOR_SOURCE_PROJECT,
        PANE_ORCHESTRATOR_BUILD_COMMAND,
    )
}

fn pane_orchestrator_source_git(crate_dir: &Path) -> Result<SourceGitMetadata, String> {
    source_git(PANE_ORCHESTRATOR_SOURCE_PROJECT, crate_dir)
}

fn clean_source_git(
    source_dir: &Path,
    project: &str,
    sync_command: &str,
) -> Result<SourceGitMetadata, String> {
    let source_git = source_git(project, source_dir)?;
    if source_git.dirty {
        return Err(format!(
            "{project} source checkout is dirty: {}. Commit or discard source changes before running `{sync_command}`.",
            source_dir.display()
        ));
    }
    Ok(source_git)
}

fn source_git(project: &str, source_dir: &Path) -> Result<SourceGitMetadata, String> {
    let inside_work_tree = git_output(source_dir, &["rev-parse", "--is-inside-work-tree"])?;
    if inside_work_tree.trim() != "true" {
        return Err(format!(
            "{project} source checkout is not a Git worktree: {}",
            source_dir.display()
        ));
    }
    Ok(SourceGitMetadata {
        commit: git_output(source_dir, &["rev-parse", "HEAD"])?,
        remote: git_output(source_dir, &["remote", "get-url", "origin"])?,
        dirty: !git_output(source_dir, &["status", "--porcelain"])?.is_empty(),
    })
}

fn git_output(crate_dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(crate_dir)
        .args(args)
        .output()
        .map_err(|error| format!("Failed to launch git in {}: {error}", crate_dir.display()))?;
    if !output.status.success() {
        return Err(format!(
            "Git command failed in {}: git -C {} {}\n{}",
            crate_dir.display(),
            crate_dir.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn pane_orchestrator_source_hash(paths: &PaneOrchestratorPaths) -> Result<String, String> {
    let mut files = Vec::new();
    collect_pane_orchestrator_source_files(&paths.crate_dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(&paths.crate_dir).map_err(|error| {
            format!(
                "Failed to make {} relative to {}: {}",
                path.display(),
                paths.crate_dir.display(),
                error
            )
        })?;
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(
            fs::read(&path)
                .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?,
        );
        hasher.update([0]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn zellij_popup_source_hash(source_dir: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_zellij_popup_source_files(source_dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(source_dir).map_err(|error| {
            format!(
                "Failed to make {} relative to {}: {}",
                path.display(),
                source_dir.display(),
                error
            )
        })?;
        hasher.update(relative.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(
            fs::read(&path)
                .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?,
        );
        hasher.update([0]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_pane_orchestrator_source_files(
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!(
            "Pane-orchestrator crate directory not found: {}",
            dir.display()
        ));
    }

    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {}", dir.display(), error))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path
            .components()
            .any(|component| component.as_os_str().to_string_lossy() == "target")
        {
            continue;
        }
        if path.is_dir() {
            collect_pane_orchestrator_source_files(&path, files)?;
        } else if is_pane_orchestrator_source_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_pane_orchestrator_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, "Cargo.toml" | "Cargo.lock"))
}

fn collect_zellij_popup_source_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.exists() {
        return Err(format!(
            "yazelix-zellij-popup source directory not found: {}",
            dir.display()
        ));
    }

    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {}", dir.display(), error))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.components().any(|component| {
            matches!(
                component.as_os_str().to_string_lossy().as_ref(),
                ".git" | "target" | "result"
            )
        }) {
            continue;
        }
        if path.is_dir() {
            collect_zellij_popup_source_files(&path, files)?;
        } else if is_zellij_popup_source_file(&path) {
            files.push(path);
        }
    }
    Ok(())
}

fn is_zellij_popup_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "rs" | "kdl"))
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                matches!(
                    name,
                    "Cargo.toml" | "Cargo.lock" | "flake.nix" | "flake.lock"
                )
            })
}

fn build_yzpp_package(source_dir: &Path) -> Result<PathBuf, String> {
    if !source_dir.exists() {
        return Err(format!(
            "❌ yzpp source checkout not found: {}. Clone `{}` next to the Yazelix repo or set {}.",
            source_dir.display(),
            YZPP_SOURCE_PROJECT,
            YZPP_SOURCE_ENV
        ));
    }

    println!("📦 Building yzpp package from {}...", source_dir.display());
    let flake_ref = format!("{}#yzpp", source_dir.to_string_lossy());
    let output = Command::new("nix")
        .args(["build", "--no-link", "--print-out-paths", &flake_ref])
        .output()
        .map_err(|error| format!("Failed to launch nix build for yzpp: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() {
        return Err(format!(
            "yzpp package build failed for {flake_ref}:\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let store_root = stdout
        .lines()
        .last()
        .map(PathBuf::from)
        .ok_or_else(|| format!("nix build produced no output path for {flake_ref}"))?;
    let package_wasm = store_root.join(YZPP_PACKAGE_WASM_PATH);
    if !package_wasm.is_file() {
        return Err(format!(
            "yzpp package did not contain expected wasm at {}",
            package_wasm.display()
        ));
    }
    Ok(package_wasm)
}

fn make_writable_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let mut permissions = fs::metadata(path)
        .map_err(|error| format!("Failed to inspect {}: {error}", path.display()))?
        .permissions();
    if permissions.readonly() {
        permissions.set_readonly(false);
        fs::set_permissions(path, permissions).map_err(|error| {
            format!(
                "Failed to make existing file writable at {}: {error}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn file_sha256_hex(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_fixture_repo() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let repo = tmp.path().join("yazelix");
        let crate_dir = pane_orchestrator_source_dir(&repo);
        fs::create_dir_all(crate_dir.join("src")).unwrap();
        fs::create_dir_all(repo.join("configs").join("zellij").join("plugins")).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\n",
        )
        .unwrap();
        fs::write(crate_dir.join("Cargo.lock"), "# lock\n").unwrap();
        fs::write(crate_dir.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(crate_dir.join(".gitignore"), "target/\n").unwrap();
        init_fixture_source_git(&crate_dir);
        let public_wasm = pane_orchestrator_paths(&repo).wasm_path;
        fs::create_dir_all(public_wasm.parent().unwrap()).unwrap();
        fs::write(public_wasm, b"wasm").unwrap();
        fs::write(tracked_wasm_path(&repo), b"wasm").unwrap();
        (tmp, repo)
    }

    fn write_yzpp_fixture_repo() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let tmp = tempdir().unwrap();
        let repo = tmp.path().join("yazelix");
        let source_dir = zellij_popup_source_dir(&repo);
        fs::create_dir_all(source_dir.join("src")).unwrap();
        fs::create_dir_all(source_dir.join("examples")).unwrap();
        fs::create_dir_all(repo.join("configs").join("zellij").join("plugins")).unwrap();
        fs::write(
            source_dir.join("Cargo.toml"),
            "[package]\nname = \"yazelix_zellij_popup\"\n",
        )
        .unwrap();
        fs::write(source_dir.join("Cargo.lock"), "# lock\n").unwrap();
        fs::write(source_dir.join("flake.nix"), "{ outputs = _: {}; }\n").unwrap();
        fs::write(source_dir.join("flake.lock"), "{}\n").unwrap();
        fs::write(source_dir.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(source_dir.join("examples").join("gitui.kdl"), "layout {}\n").unwrap();
        fs::write(source_dir.join(".gitignore"), "target/\nresult\n").unwrap();
        init_fixture_source_git(&source_dir);
        fs::write(tracked_zellij_plugin_path(&repo, YZPP_WASM_NAME), b"yzpp").unwrap();
        let package_wasm = repo.join("package_yzpp.wasm");
        fs::write(&package_wasm, b"yzpp").unwrap();
        (tmp, repo, package_wasm)
    }

    // Defends: the sync guard accepts source-present and artifact-only checkouts whose tracked wasm and stamp agree.
    #[test]
    fn pane_orchestrator_sync_validator_accepts_current_stamp() {
        let (_tmp, repo) = write_fixture_repo();
        let paths = write_fixture_sync_stamp(&repo);

        assert!(validate_pane_orchestrator_sync(&repo).unwrap().is_empty());
        fs::remove_dir_all(paths.crate_dir).unwrap();
        assert!(validate_pane_orchestrator_sync(&repo).unwrap().is_empty());
    }

    // Regression: pane-orchestrator source edits must not pass maintainer checks until the wasm is rebuilt and synced.
    #[test]
    fn pane_orchestrator_sync_validator_rejects_stale_source_stamp() {
        let (_tmp, repo) = write_fixture_repo();
        let paths = write_fixture_sync_stamp(&repo);
        fs::write(
            paths.crate_dir.join("src").join("main.rs"),
            "fn main() { println!(\"changed\"); }\n",
        )
        .unwrap();

        let errors = validate_pane_orchestrator_sync(&repo).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("source changed since the tracked wasm was synced"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("source checkout is dirty"))
        );
    }

    // Defends: sync stamps must not be produced from uncommitted pane-orchestrator source edits.
    #[test]
    fn pane_orchestrator_sync_refuses_dirty_source_checkout() {
        let (_tmp, repo) = write_fixture_repo();
        let paths = write_fixture_sync_stamp(&repo);
        fs::write(
            paths.crate_dir.join("src").join("main.rs"),
            "fn main() { println!(\"dirty\"); }\n",
        )
        .unwrap();

        let error = clean_pane_orchestrator_source_git(&paths.crate_dir).unwrap_err();
        assert!(error.contains("source checkout is dirty"));
    }

    // Defends: yzpp copied-artifact guard accepts source-present checkouts whose tracked wasm and stamp agree.
    #[test]
    fn yzpp_sync_validator_accepts_current_stamp() {
        let (_tmp, repo, package_wasm) = write_yzpp_fixture_repo();
        write_fixture_yzpp_sync_stamp(&repo, &package_wasm);

        assert!(validate_yzpp_sync(&repo).unwrap().is_empty());
    }

    // Regression: yzpp source edits must not pass the copied-artifact validator until the wasm is rebuilt and stamped.
    #[test]
    fn yzpp_sync_validator_rejects_stale_source_stamp() {
        let (_tmp, repo, package_wasm) = write_yzpp_fixture_repo();
        let paths = write_fixture_yzpp_sync_stamp(&repo, &package_wasm);
        fs::write(
            paths.source_dir.join("src").join("main.rs"),
            "fn main() { println!(\"changed\"); }\n",
        )
        .unwrap();

        let errors = validate_yzpp_sync(&repo).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("source changed since yzpp.wasm was synced"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("source checkout is dirty"))
        );
    }

    fn write_fixture_sync_stamp(repo: &Path) -> PaneOrchestratorPaths {
        let paths = pane_orchestrator_paths(repo);
        let source_git = clean_pane_orchestrator_source_git(&paths.crate_dir).unwrap();
        fs::write(
            tracked_sync_stamp_path(repo),
            render_sync_stamp(&paths, &tracked_wasm_path(repo), &source_git).unwrap(),
        )
        .unwrap();
        paths
    }

    fn write_fixture_yzpp_sync_stamp(repo: &Path, package_wasm: &Path) -> YzppPaths {
        let paths = yzpp_paths(repo);
        let source_git =
            clean_source_git(&paths.source_dir, YZPP_SOURCE_PROJECT, YZPP_SYNC_COMMAND).unwrap();
        fs::write(
            tracked_zellij_plugin_path(repo, YZPP_SYNC_STAMP_NAME),
            render_yzpp_sync_stamp(
                &paths,
                &tracked_zellij_plugin_path(repo, YZPP_WASM_NAME),
                package_wasm,
                &source_git,
            )
            .unwrap(),
        )
        .unwrap();
        paths
    }

    fn init_fixture_source_git(crate_dir: &Path) {
        run_fixture_git(crate_dir, &["init"]);
        run_fixture_git(
            crate_dir,
            &["config", "user.email", "fixture@example.invalid"],
        );
        run_fixture_git(crate_dir, &["config", "user.name", "Fixture"]);
        run_fixture_git(
            crate_dir,
            &[
                "remote",
                "add",
                "origin",
                "https://example.invalid/yazelix-zellij-pane-orchestrator.git",
            ],
        );
        run_fixture_git(crate_dir, &["add", "."]);
        run_fixture_git(crate_dir, &["commit", "-m", "initial"]);
    }

    fn run_fixture_git(crate_dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(crate_dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git -C {} {} failed:\n{}",
            crate_dir.display(),
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
