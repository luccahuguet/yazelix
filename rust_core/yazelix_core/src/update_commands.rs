//! `yzx update` family implemented in Rust for `yzx_control` (bead: yazelix-ulb2.6).

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{expand_user_path, home_dir_from_env, runtime_dir_from_env};
use crate::install_ownership_env::install_ownership_request_from_env;
use crate::install_ownership_report::evaluate_install_ownership_report;
use serde_json::Value;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
    println!(
        "❌ `yzx update upstream` is for default Nix profile installs, but this Yazelix runtime appears to be Home Manager-owned."
    );
    println!(
        "   Run `yzx update home_manager` from the Home Manager flake that owns this install."
    );
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
            .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
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
        println!(
            "❌ Multiple default-profile Yazelix entries point at the active runtime: {names}"
        );
        println!("   Keep one clear profile owner, then rerun `yzx update upstream`.");
        return Err(1);
    }

    println!(
        "❌ `yzx update upstream` could not find the active Yazelix runtime in the default Nix profile."
    );
    println!("   Current runtime: {}", runtime_root.display());
    println!(
        "   This command now updates profile-installed Yazelix packages after the legacy flake installer was removed."
    );
    println!(
        "   Recovery: Reinstall with `nix profile add github:luccahuguet/yazelix#yazelix`, or use `yzx update home_manager` if Home Manager owns this install."
    );
    Err(1)
}

fn print_exact_command(command: &str) {
    println!("Running:");
    println!("  {command}");
}

fn current_working_flake_dir() -> Result<PathBuf, i32> {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(_) => return Err(1),
    };
    let flake_file = cwd.join("flake.nix");
    if flake_file.is_file() {
        return Ok(cwd);
    }
    println!(
        "❌ yzx update home_manager must be run from the Home Manager flake directory that owns this install."
    );
    println!(
        "   Missing flake.nix in the current directory: {}",
        flake_file.display()
    );
    Err(1)
}

fn flake_input_lock_node(flake_dir: &Path, input_name: &str) -> Option<Value> {
    let lock_path = flake_dir.join("flake.lock");
    let raw = std::fs::read_to_string(lock_path).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    parsed.get("nodes")?.get(input_name).cloned()
}

fn local_git_checkout_path(path: &Path) -> Option<PathBuf> {
    let canonical = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let git_marker = canonical.join(".git");
    if git_marker.is_dir() || git_marker.is_file() {
        Some(canonical)
    } else {
        None
    }
}

fn encode_file_url_path(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .flat_map(|ch| match ch {
            '%' => "%25".chars().collect::<Vec<_>>(),
            ' ' => "%20".chars().collect::<Vec<_>>(),
            '#' => "%23".chars().collect::<Vec<_>>(),
            '?' => "%3F".chars().collect::<Vec<_>>(),
            _ => vec![ch],
        })
        .collect()
}

fn local_path_input_git_migration_url(flake_dir: &Path, input_name: &str) -> Option<String> {
    let node = flake_input_lock_node(flake_dir, input_name)?;
    let source = node
        .get("original")
        .and_then(|value| value.get("type"))
        .and_then(|value| value.as_str())
        .filter(|kind| *kind == "path")
        .and_then(|_| node.get("original"))
        .or_else(|| {
            node.get("locked")
                .and_then(|value| value.get("type"))
                .and_then(|value| value.as_str())
                .filter(|kind| *kind == "path")
                .and_then(|_| node.get("locked"))
        })?;
    let path = PathBuf::from(source.get("path")?.as_str()?);
    let checkout = local_git_checkout_path(&path)?;
    Some(format!("git+file://{}", encode_file_url_path(&checkout)))
}

fn print_home_manager_local_path_input_guidance(flake_dir: &Path, input_name: &str) {
    let Some(git_file_url) = local_path_input_git_migration_url(flake_dir, input_name) else {
        return;
    };

    let node = match flake_input_lock_node(flake_dir, input_name) {
        Some(node) => node,
        None => return,
    };
    let source_path = node
        .get("original")
        .and_then(|value| value.get("path"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            node.get("locked")
                .and_then(|value| value.get("path"))
                .and_then(|value| value.as_str())
        });
    let Some(source_path) = source_path else {
        return;
    };

    println!();
    println!(
        "⚠️  The current `{input_name}` input is pinned as a local `path:` source: {source_path}"
    );
    println!(
        "   `path:` snapshots the whole directory, including local build artifacts and untracked files."
    );
    println!(
        "   This checkout is a git repository, so prefer `git+file:` for faster Home Manager updates."
    );
    println!("   Replace the flake input with:");
    println!("     url = \"{git_file_url}\";");
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
        println!(
            "  yzx update upstream      Upgrade the active Yazelix package in the default Nix profile"
        );
        println!(
            "  yzx update home_manager  Refresh the current Home Manager flake input, then print `home-manager switch`"
        );
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
                println!(
                    "   If your Nix install is not based on Determinate Nix, this will not work."
                );
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

            let flake_dir = match current_working_flake_dir() {
                Ok(dir) => dir,
                Err(code) => return Ok(code),
            };
            print_update_path_confirmation("home_manager")?;
            println!();
            println!(
                "⚠️  `yzx update home_manager` updates the `yazelix` input in the current flake directory."
            );
            println!("   Run it only from the Home Manager flake that owns this install.");
            println!(
                "   If your Yazelix input uses a different name, run `nix flake update <your-input-name>` yourself."
            );
            println!(
                "   This still matters for `path:` inputs because `flake.lock` pins a snapshot of that local path until you refresh it."
            );
            println!();
            print_exact_command("nix flake update yazelix");

            let output = match Command::new("nix")
                .args(["flake", "update", "yazelix"])
                .current_dir(&flake_dir)
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
            print_home_manager_local_path_input_guidance(&flake_dir, "yazelix");
            if local_path_input_git_migration_url(&flake_dir, "yazelix").is_some() {
                println!();
            }
            println!("Next step:");
            println!("  home-manager switch");
            Ok(0)
        }
        other => Err(CoreError::usage(format!(
            "Unknown yzx update subcommand: {other}. Try `yzx update` for a list."
        ))),
    }
}
