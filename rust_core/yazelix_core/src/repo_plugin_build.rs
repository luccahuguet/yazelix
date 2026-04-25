use crate::active_config_surface::resolve_active_config_paths;
use crate::control_plane::{config_dir_from_env, state_dir_from_env};
use crate::zellij_materialization::{
    ZellijMaterializationRequest, generate_zellij_materialization,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUILD_TARGET: &str = "wasm32-wasip1";
const PANE_ORCHESTRATOR_WASM_NAME: &str = "yazelix_pane_orchestrator.wasm";

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

    let merged_config_path = generate_merged_zellij_config(&paths.repo_root)?;
    let byte_len = fs::metadata(&paths.wasm_path)
        .map_err(|error| format!("Failed to inspect built wasm size: {error}"))?
        .len();

    println!(
        "Updated pane orchestrator repo wasm: {}",
        repo_target.display()
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
