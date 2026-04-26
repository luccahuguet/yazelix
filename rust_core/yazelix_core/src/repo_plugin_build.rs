use crate::active_config_surface::resolve_active_config_paths;
use crate::control_plane::{config_dir_from_env, state_dir_from_env};
use crate::zellij_materialization::{
    ZellijMaterializationRequest, generate_zellij_materialization,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUILD_TARGET: &str = "wasm32-wasip1";
const PANE_ORCHESTRATOR_WASM_NAME: &str = "yazelix_pane_orchestrator.wasm";
const PANE_ORCHESTRATOR_SYNC_STAMP_NAME: &str = "yazelix_pane_orchestrator.sync_stamp";

struct PaneOrchestratorPaths {
    repo_root: PathBuf,
    crate_dir: PathBuf,
    wasm_path: PathBuf,
}

pub fn build_pane_orchestrator(repo_root: &Path, sync: bool) -> Result<(), String> {
    let paths = pane_orchestrator_paths(repo_root);
    ensure_build_tools_available()?;
    run_wasm_build(&paths, "pane orchestrator")?;

    if sync {
        sync_built_wasm(&paths, "pane orchestrator")?;
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

    let current_source_hash = pane_orchestrator_source_hash(&paths)?;
    match read_sync_stamp(&sync_stamp_path)? {
        None => errors.push(format!(
            "Missing pane-orchestrator sync stamp: {}. Run `yzx dev build_pane_orchestrator --sync`.",
            sync_stamp_path.display()
        )),
        Some(stamp) if stamp.source_sha256 != current_source_hash => errors.push(format!(
            "Pane-orchestrator source changed since the tracked wasm was synced. Run `yzx dev build_pane_orchestrator --sync`.\n   current source sha256: {}\n   synced source sha256:  {}",
            current_source_hash, stamp.source_sha256
        )),
        Some(stamp) => {
            if repo_wasm.is_file() {
                let current_wasm_hash = file_sha256_hex(&repo_wasm)?;
                if stamp.wasm_sha256 != current_wasm_hash {
                    errors.push(format!(
                        "Tracked pane-orchestrator wasm hash does not match its sync stamp. Run `yzx dev build_pane_orchestrator --sync`.\n   current wasm sha256: {}\n   stamped wasm sha256: {}",
                        current_wasm_hash, stamp.wasm_sha256
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

fn pane_orchestrator_paths(repo_root: &Path) -> PaneOrchestratorPaths {
    let crate_dir = repo_root
        .join("rust_plugins")
        .join("zellij_pane_orchestrator");
    let wasm_path = crate_dir
        .join("target")
        .join(BUILD_TARGET)
        .join("release")
        .join(PANE_ORCHESTRATOR_WASM_NAME);

    PaneOrchestratorPaths {
        repo_root: repo_root.to_path_buf(),
        crate_dir,
        wasm_path,
    }
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
            "❌ {label} crate not found: {}",
            paths.crate_dir.display()
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
    repo_root
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join(PANE_ORCHESTRATOR_WASM_NAME)
}

fn tracked_sync_stamp_path(repo_root: &Path) -> PathBuf {
    repo_root
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join(PANE_ORCHESTRATOR_SYNC_STAMP_NAME)
}

fn runtime_wasm_path() -> Result<PathBuf, String> {
    Ok(state_dir_from_env()
        .map_err(|error| error.message().to_string())?
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join(PANE_ORCHESTRATOR_WASM_NAME))
}

fn generate_merged_zellij_config(repo_root: &Path) -> Result<String, String> {
    let config_dir = config_dir_from_env().map_err(|error| error.message().to_string())?;
    let config_surface = resolve_active_config_paths(repo_root, &config_dir, None)
        .map_err(|error| error.message().to_string())?;
    let zellij_config_dir = state_dir_from_env()
        .map_err(|error| error.message().to_string())?
        .join("configs")
        .join("zellij");

    let data = generate_zellij_materialization(&ZellijMaterializationRequest {
        config_path: config_surface.config_file,
        default_config_path: config_surface.default_config_path,
        contract_path: config_surface.contract_path,
        runtime_dir: repo_root.to_path_buf(),
        zellij_config_dir,
        seed_plugin_permissions: false,
    })
    .map_err(|error| error.message().to_string())?;

    Ok(data.merged_config_path)
}

fn sync_built_wasm(paths: &PaneOrchestratorPaths, label: &str) -> Result<(), String> {
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

    let sync_stamp = render_sync_stamp(paths, &repo_target)?;
    let sync_stamp_path = tracked_sync_stamp_path(&paths.repo_root);
    fs::write(&sync_stamp_path, sync_stamp).map_err(|error| {
        format!(
            "Failed to write pane-orchestrator sync stamp {}: {error}",
            sync_stamp_path.display()
        )
    })?;

    let merged_config_path = generate_merged_zellij_config(&paths.repo_root)?;
    let byte_len = fs::metadata(&paths.wasm_path)
        .map_err(|error| format!("Failed to inspect built wasm size: {error}"))?
        .len();

    println!(
        "Updated pane orchestrator repo wasm: {}",
        repo_target.display()
    );
    println!(
        "Updated pane orchestrator sync stamp: {}",
        sync_stamp_path.display()
    );
    println!(
        "Updated pane orchestrator runtime wasm: {}",
        runtime_target.display()
    );
    println!("Updated merged Zellij config: {merged_config_path}");
    println!("Size: {byte_len} bytes");
    println!();
    println!("Safest next step:");
    println!(
        "Restart Yazelix or open a fresh Yazelix window so Zellij loads the updated plugin cleanly."
    );
    println!("In-place plugin reloads can leave the current session in a broken permission state.");
    println!();
    println!("If you are already stuck in a blank/permission-limbo session, recover with:");
    println!("zellij delete-all-sessions -f -y");

    Ok(())
}

struct PaneOrchestratorSyncStamp {
    source_sha256: String,
    wasm_sha256: String,
}

fn render_sync_stamp(paths: &PaneOrchestratorPaths, repo_wasm: &Path) -> Result<String, String> {
    Ok(format!(
        "source_sha256 = \"{}\"\nwasm_sha256 = \"{}\"\n",
        pane_orchestrator_source_hash(paths)?,
        file_sha256_hex(repo_wasm)?,
    ))
}

fn read_sync_stamp(path: &Path) -> Result<Option<PaneOrchestratorSyncStamp>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    let mut source_sha256 = None;
    let mut wasm_sha256 = None;
    for line in raw.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "Invalid sync stamp line in {}: {line}",
                path.display()
            ));
        };
        let value = value.trim().trim_matches('"').to_string();
        match key.trim() {
            "source_sha256" => source_sha256 = Some(value),
            "wasm_sha256" => wasm_sha256 = Some(value),
            other => {
                return Err(format!(
                    "Unknown sync stamp field `{other}` in {}",
                    path.display()
                ));
            }
        }
    }
    Ok(Some(PaneOrchestratorSyncStamp {
        source_sha256: source_sha256
            .ok_or_else(|| format!("Missing source_sha256 in {}", path.display()))?,
        wasm_sha256: wasm_sha256
            .ok_or_else(|| format!("Missing wasm_sha256 in {}", path.display()))?,
    }))
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
        let repo = tmp.path().to_path_buf();
        let crate_dir = repo.join("rust_plugins").join("zellij_pane_orchestrator");
        fs::create_dir_all(crate_dir.join("src")).unwrap();
        fs::create_dir_all(repo.join("configs").join("zellij").join("plugins")).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"fixture\"\n",
        )
        .unwrap();
        fs::write(crate_dir.join("Cargo.lock"), "# lock\n").unwrap();
        fs::write(crate_dir.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(tracked_wasm_path(&repo), b"wasm").unwrap();
        (tmp, repo)
    }

    // Defends: the maintainer sync guard accepts a repo whose tracked wasm and source stamp agree.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn pane_orchestrator_sync_validator_accepts_current_stamp() {
        let (_tmp, repo) = write_fixture_repo();
        let paths = pane_orchestrator_paths(&repo);
        let stamp = render_sync_stamp(&paths, &tracked_wasm_path(&repo)).unwrap();
        fs::write(tracked_sync_stamp_path(&repo), stamp).unwrap();

        assert!(validate_pane_orchestrator_sync(&repo).unwrap().is_empty());
    }

    // Regression: pane-orchestrator source edits must not pass maintainer checks until the wasm is rebuilt and synced.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn pane_orchestrator_sync_validator_rejects_stale_source_stamp() {
        let (_tmp, repo) = write_fixture_repo();
        let paths = pane_orchestrator_paths(&repo);
        let stamp = render_sync_stamp(&paths, &tracked_wasm_path(&repo)).unwrap();
        fs::write(tracked_sync_stamp_path(&repo), stamp).unwrap();
        fs::write(
            paths.crate_dir.join("src").join("main.rs"),
            "fn main() { println!(\"changed\"); }\n",
        )
        .unwrap();

        let errors = validate_pane_orchestrator_sync(&repo).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("source changed since the tracked wasm was synced"));
    }
}
