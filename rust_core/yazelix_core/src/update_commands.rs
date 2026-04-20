//! `yzx update` family implemented in Rust for `yzx_control` (bead: yazelix-ulb2.6).

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    config_dir_from_env, expand_user_path, home_dir_from_env, runtime_dir_from_env,
};
use crate::install_ownership_report::{
    evaluate_install_ownership_report, InstallOwnershipEvaluateRequest,
};
use serde_json::Value;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn xdg_config_home(home: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("XDG_CONFIG_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home);
        }
    }
    home.join(".config")
}

fn xdg_data_home(home: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("XDG_DATA_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home);
        }
    }
    home.join(".local").join("share")
}

fn yazelix_state_dir(home: &Path) -> PathBuf {
    if let Ok(raw) = std::env::var("YAZELIX_STATE_DIR") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home);
        }
    }
    if let Ok(raw) = std::env::var("XDG_DATA_HOME") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return expand_user_path(trimmed, home).join("yazelix");
        }
    }
    home.join(".local").join("share").join("yazelix")
}

fn shell_resolved_yzx_path(home: &Path) -> Option<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("command -v yzx")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(expand_user_path(&s, home).to_string_lossy().into_owned())
    }
}

fn install_ownership_request_from_env() -> Result<InstallOwnershipEvaluateRequest, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let home_dir = home_dir_from_env()?;
    let config_root = config_dir_from_env()?;
    let main_config_path = config_root.join("user_configs").join("yazelix.toml");
    Ok(InstallOwnershipEvaluateRequest {
        runtime_dir,
        home_dir: home_dir.clone(),
        user: std::env::var("USER").ok().filter(|s| !s.trim().is_empty()),
        xdg_config_home: xdg_config_home(&home_dir),
        xdg_data_home: xdg_data_home(&home_dir),
        yazelix_state_dir: yazelix_state_dir(&home_dir),
        main_config_path,
        invoked_yzx_path: std::env::var("YAZELIX_INVOKED_YZX_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty()),
        redirected_from_stale_yzx_path: std::env::var("YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH")
            .ok()
            .filter(|s| !s.trim().is_empty()),
        shell_resolved_yzx_path: shell_resolved_yzx_path(&home_dir),
    })
}

fn normalize_path_for_compare(path: &Path, home: &Path) -> PathBuf {
    let expanded = if path.starts_with("~") {
        expand_user_path(&path.to_string_lossy(), home)
    } else {
        path.to_path_buf()
    };
    std::fs::canonicalize(&expanded).unwrap_or(expanded)
}

fn command_exists(name: &str) -> bool {
    // Use a POSIX shell so `command -v` is a builtin. After `runtime_env.sh`, PATH may omit
    // `/usr/bin`, so spawning the external `command` helper from coreutils can fail spuriously.
    Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn print_completed_output(stdout: &[u8], stderr: &[u8]) {
    if !stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(stdout));
    }
    if !stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(stderr));
    }
}

fn print_update_owner_warning() {
    println!("Choose one update owner for this Yazelix install.");
    println!();
    println!("  Use `yzx update upstream` if this install is owned by a Nix profile package.");
    println!("  Use `yzx update home_manager` if Home Manager owns this install.");
    println!();
    println!("Do not use both update paths for the same installed Yazelix runtime.");
}

fn print_update_path_confirmation(owner: &str) -> Result<(), CoreError> {
    match owner {
        "upstream" => {
            println!("Requested update path: default Nix profile.");
            println!();
            println!("  Use this only when a Nix profile package owns the active Yazelix runtime.");
        }
        "home_manager" => {
            println!("Requested update path: Home Manager flake input.");
            println!();
            println!("  Use this only when Home Manager owns the active Yazelix runtime.");
        }
        _ => {
            return Err(CoreError::classified(
                ErrorClass::Internal,
                "update_owner_bug",
                format!("Unsupported update owner confirmation: {owner}"),
                "Report this as a Yazelix bug.",
                serde_json::json!({}),
            ));
        }
    }
    println!();
    println!("Do not use both update paths for the same installed Yazelix runtime.");
    Ok(())
}

fn fail_if_home_manager_owned_upstream_update() -> Result<(), CoreError> {
    let req = install_ownership_request_from_env()?;
    let report = evaluate_install_ownership_report(&req);
    if report.install_owner != "home-manager" {
        return Ok(());
    }
    println!("❌ `yzx update upstream` is for default Nix profile installs, but this Yazelix runtime appears to be Home Manager-owned.");
    println!("   Run `yzx update home_manager` from the Home Manager flake that owns this install.");
    println!("   Then run `home-manager switch` to apply the updated input.");
    println!("   Do not use both update paths for the same installed Yazelix runtime.");
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "hm_owned_upstream",
        "Home Manager owns this install; use yzx update home_manager.",
        "Run `yzx update home_manager` from the owning flake, then `home-manager switch`.",
        serde_json::json!({}),
    ))
}

/// `Err` is an exit code (already printed for user-facing failures).
fn load_default_profile_elements_json() -> Result<Value, i32> {
    let output = match Command::new("nix")
        .args(["profile", "list", "--json"])
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            println!("❌ Failed to inspect the default Nix profile.");
            return Err(1);
        }
    };

    if !output.status.success() {
        println!("❌ Failed to inspect the default Nix profile.");
        print_completed_output(&output.stdout, &output.stderr);
        return Err(1);
    }

    let text = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&text).map_err(|err| {
        println!("❌ Failed to parse `nix profile list --json`: {err}");
        1
    })
}

fn resolve_active_yazelix_profile_entry_name(profile_json: &Value) -> Result<String, i32> {
    let home = match home_dir_from_env() {
        Ok(h) => h,
        Err(_) => return Err(1),
    };
    let runtime_dir = match runtime_dir_from_env() {
        Ok(p) => p,
        Err(_) => return Err(1),
    };
    let runtime_root = normalize_path_for_compare(runtime_dir.as_path(), &home);
    let elements = profile_json
        .get("elements")
        .and_then(|e| e.as_object())
        .cloned()
        .unwrap_or_default();

    let mut matches: Vec<String> = Vec::new();
    for (name, entry) in &elements {
        let store_paths = entry
            .get("storePaths")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for store_path in store_paths {
            let expanded = normalize_path_for_compare(Path::new(store_path), &home);
            if expanded == runtime_root {
                matches.push(name.clone());
                break;
            }
        }
    }

    if matches.len() == 1 {
        return Ok(matches[0].clone());
    }

    if matches.len() > 1 {
        let names = matches.join(", ");
        println!("❌ Multiple default-profile Yazelix entries point at the active runtime: {names}");
        println!("   Keep one clear profile owner, then rerun `yzx update upstream`.");
        return Err(1);
    }

    println!("❌ `yzx update upstream` could not find the active Yazelix runtime in the default Nix profile.");
    println!("   Current runtime: {}", runtime_root.display());
    println!("   This command now updates profile-installed Yazelix packages after the legacy flake installer was removed.");
    println!("   Recovery: Reinstall with `nix profile add github:luccahuguet/yazelix#yazelix`, or use `yzx update home_manager` if Home Manager owns this install.");
    Err(1)
}

fn print_exact_command(command: &str) {
    println!("Running:");
    println!("  {command}");
}

fn require_current_working_flake() -> Result<(), i32> {
    let flake_file = match std::env::current_dir() {
        Ok(cwd) => cwd.join("flake.nix"),
        Err(_) => return Err(1),
    };
    if flake_file.is_file() {
        return Ok(());
    }
    println!("❌ yzx update home_manager must be run from the Home Manager flake directory that owns this install.");
    println!(
        "   Missing flake.nix in the current directory: {}",
        flake_file.display()
    );
    Err(1)
}

fn parse_nix_subcommand_flags(args: &[String]) -> Result<(bool, bool), CoreError> {
    let mut yes = false;
    let mut verbose = false;
    for a in args {
        match a.as_str() {
            "--yes" => yes = true,
            "--verbose" => verbose = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx update nix: {other}"
                )));
            }
        }
    }
    Ok((yes, verbose))
}

/// `args` are tokens after `yzx_control update` (e.g. `["upstream"]`, `["nix", "--yes"]`).
pub fn run_yzx_update(args: &[String]) -> Result<i32, CoreError> {
    if args.is_empty() || matches!(args[0].as_str(), "--help" | "-h" | "help") {
        print_update_owner_warning();
        println!();
        println!("Available update commands:");
        println!("  yzx update upstream      Upgrade the active Yazelix package in the default Nix profile");
        println!("  yzx update home_manager  Refresh the current Home Manager flake input, then print `home-manager switch`");
        println!("  yzx update nix           Upgrade Determinate Nix (if installed)");
        return Ok(0);
    }

    match args[0].as_str() {
        "nix" => {
            let (yes, verbose) = parse_nix_subcommand_flags(&args[1..])?;
            if !command_exists("determinate-nixd") {
                println!("❌ determinate-nixd not found in PATH.");
                println!("   Install Determinate Nix or check your PATH, then try again.");
                return Ok(1);
            }

            if !yes {
                println!("⚠️  This upgrades Determinate Nix using determinate-nixd.");
                println!("   If your Nix install is not based on Determinate Nix, this will not work.");
                println!("   It requires sudo and may prompt for your password.");
                print!("Continue? [y/N]: ");
                let _ = std::io::stdout().flush();
                let mut line = String::new();
                let _ = std::io::stdin().lock().read_line(&mut line);
                let confirm = line.trim().to_lowercase();
                if confirm != "y" && confirm != "yes" {
                    println!("Aborted.");
                    return Ok(0);
                }
            }

            if verbose {
                println!("⚙️ Running: sudo determinate-nixd upgrade");
            } else {
                println!("🔄 Upgrading Determinate Nix...");
            }

            let status = Command::new("sudo")
                .args(["determinate-nixd", "upgrade"])
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("✅ Determinate Nix upgraded.");
                    Ok(0)
                }
                Ok(s) => {
                    println!("❌ Determinate Nix upgrade failed.");
                    Ok(s.code().unwrap_or(1))
                }
                Err(err) => {
                    println!("❌ Determinate Nix upgrade failed: {err}");
                    Ok(1)
                }
            }
        }
        "upstream" => {
            if args.len() != 1 {
                return Err(CoreError::usage(
                    "yzx update upstream does not take additional arguments.",
                ));
            }
            if !command_exists("nix") {
                println!("❌ nix not found in PATH.");
                println!("   Install Nix first, then try again.");
                return Ok(1);
            }

            if let Err(e) = fail_if_home_manager_owned_upstream_update() {
                if matches!(e.class(), ErrorClass::Runtime) && e.code() == "hm_owned_upstream" {
                    return Ok(1);
                }
                return Err(e);
            }

            print_update_path_confirmation("upstream")?;
            println!();
            let profile_json = match load_default_profile_elements_json() {
                Ok(v) => v,
                Err(code) => return Ok(code),
            };
            let profile_name = match resolve_active_yazelix_profile_entry_name(&profile_json) {
                Ok(n) => n,
                Err(code) => return Ok(code),
            };
            let cmd_line = format!("nix profile upgrade --refresh {profile_name}");
            print_exact_command(&cmd_line);

            let output = match Command::new("nix")
                .args(["profile", "upgrade", "--refresh", &profile_name])
                .output()
            {
                Ok(o) => o,
                Err(_) => {
                    println!("❌ Upstream Yazelix update failed.");
                    return Ok(1);
                }
            };

            print_completed_output(&output.stdout, &output.stderr);
            if output.status.success() {
                Ok(0)
            } else {
                println!("❌ Upstream Yazelix update failed.");
                Ok(output.status.code().unwrap_or(1))
            }
        }
        "home_manager" => {
            if args.len() != 1 {
                return Err(CoreError::usage(
                    "yzx update home_manager does not take additional arguments.",
                ));
            }
            if !command_exists("nix") {
                println!("❌ nix not found in PATH.");
                println!("   Install Nix first, then try again.");
                return Ok(1);
            }

            if let Err(code) = require_current_working_flake() {
                return Ok(code);
            }
            print_update_path_confirmation("home_manager")?;
            println!();
            println!("⚠️  `yzx update home_manager` updates the `yazelix` input in the current flake directory.");
            println!("   Run it only from the Home Manager flake that owns this install.");
            println!("   If your Yazelix input uses a different name, run `nix flake update <your-input-name>` yourself.");
            println!();
            print_exact_command("nix flake update yazelix");

            let output = match Command::new("nix")
                .args(["flake", "update", "yazelix"])
                .output()
            {
                Ok(o) => o,
                Err(_) => {
                    println!("❌ Home Manager flake input update failed.");
                    return Ok(1);
                }
            };

            print_completed_output(&output.stdout, &output.stderr);
            if !output.status.success() {
                println!("❌ Home Manager flake input update failed.");
                return Ok(output.status.code().unwrap_or(1));
            }

            println!();
            println!("Next step:");
            println!("  home-manager switch");
            Ok(0)
        }
        other => Err(CoreError::usage(format!(
            "Unknown yzx update subcommand: {other}. Try `yzx update` for a list."
        ))),
    }
}
