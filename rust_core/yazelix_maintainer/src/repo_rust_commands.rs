// Test lane: maintainer

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
struct RustTargetSpec {
    name: &'static str,
    manifest_path: PathBuf,
    check_args: Vec<String>,
    test_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRustTarget {
    target: String,
    tail: Vec<String>,
}

pub fn run_repo_rust_command(repo_root: &Path, args: &[String]) -> Result<(), String> {
    if args.is_empty() || matches!(args[0].as_str(), "-h" | "--help" | "help") {
        print_repo_rust_help();
        return Ok(());
    }

    let sub = args[0].as_str();
    let tail = &args[1..];
    match sub {
        "fmt" => run_repo_rust_fmt(repo_root, tail),
        "check" => run_repo_rust_check(repo_root, tail),
        "test" => run_repo_rust_test(repo_root, tail),
        other => Err(format!("Unknown yzx dev rust subcommand: {other}")),
    }
}

fn print_repo_rust_help() {
    println!("Fast Rust inner-loop commands:");
    println!("  yzx dev rust fmt [TARGET] [--check]");
    println!("  yzx dev rust check [TARGET]");
    println!("  yzx dev rust test [TARGET] [cargo test args...]");
    println!();
    println!("TARGET: core, maintainer, pane_orchestrator, or all");
    println!("For tests, TARGET can be omitted; unmatched args are passed to core cargo test.");
    println!(
        "These commands run cargo directly from the current environment. Use Nix/Home Manager/package validation as explicit final gates."
    );
}

fn run_repo_rust_fmt(repo_root: &Path, args: &[String]) -> Result<(), String> {
    let mut target = "all".to_string();
    let mut check = false;
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            value if !value.starts_with('-') => target = value.to_string(),
            other => return Err(format!("Unknown rust fmt option {other}")),
        }
    }
    for spec in rust_target_specs(repo_root, &target)? {
        let mut cargo_args = vec![
            "fmt".to_string(),
            "--manifest-path".to_string(),
            spec.manifest_path.display().to_string(),
            "--all".to_string(),
        ];
        if check {
            cargo_args.extend(["--".to_string(), "--check".to_string()]);
        }
        run_fast_cargo_checked(repo_root, &format!("rust fmt ({})", spec.name), &cargo_args)?;
    }
    Ok(())
}

fn run_repo_rust_check(repo_root: &Path, args: &[String]) -> Result<(), String> {
    let target = args.first().map(String::as_str).unwrap_or("core");
    if args.len() > 1 {
        return Err("yzx dev rust check accepts at most one target".to_string());
    }
    for spec in rust_target_specs(repo_root, target)? {
        let mut cargo_args = vec![
            "check".to_string(),
            "--manifest-path".to_string(),
            spec.manifest_path.display().to_string(),
        ];
        cargo_args.extend(spec.check_args);
        run_fast_cargo_checked(
            repo_root,
            &format!("rust check ({})", spec.name),
            &cargo_args,
        )?;
    }
    Ok(())
}

fn run_repo_rust_test(repo_root: &Path, args: &[String]) -> Result<(), String> {
    let parsed = parse_rust_target_and_tail(args, "core");
    for spec in rust_target_specs(repo_root, &parsed.target)? {
        let mut cargo_args = vec![
            "test".to_string(),
            "--manifest-path".to_string(),
            spec.manifest_path.display().to_string(),
        ];
        cargo_args.extend(spec.test_args);
        cargo_args.extend(parsed.tail.clone());
        run_fast_cargo_checked(
            repo_root,
            &format!("rust test ({})", spec.name),
            &cargo_args,
        )?;
    }
    Ok(())
}

fn rust_target_specs(repo_root: &Path, target: &str) -> Result<Vec<RustTargetSpec>, String> {
    let specs = vec![
        RustTargetSpec {
            name: "core",
            manifest_path: repo_root.join("rust_core").join("Cargo.toml"),
            check_args: vec!["-p".into(), "yazelix_core".into()],
            test_args: vec!["-p".into(), "yazelix_core".into()],
        },
        RustTargetSpec {
            name: "maintainer",
            manifest_path: repo_root.join("rust_core").join("Cargo.toml"),
            check_args: vec!["-p".into(), "yazelix_maintainer".into()],
            test_args: vec!["-p".into(), "yazelix_maintainer".into()],
        },
        RustTargetSpec {
            name: "pane_orchestrator",
            manifest_path: repo_root
                .join("rust_plugins")
                .join("zellij_pane_orchestrator")
                .join("Cargo.toml"),
            check_args: vec!["--lib".into()],
            test_args: vec!["--lib".into()],
        },
    ];

    match target {
        "all" => Ok(specs),
        "core" | "maintainer" | "pane_orchestrator" => Ok(specs
            .into_iter()
            .filter(|spec| spec.name == target)
            .collect()),
        _ => Err(format!(
            "Unknown Rust target '{target}'. Expected one of: core, maintainer, pane_orchestrator, all."
        )),
    }
}

fn parse_rust_target_and_tail(args: &[String], default_target: &str) -> ParsedRustTarget {
    let known_targets = ["core", "maintainer", "pane_orchestrator", "all"];
    if let Some(first) = args.first()
        && known_targets.contains(&first.as_str())
    {
        return ParsedRustTarget {
            target: first.clone(),
            tail: args[1..].to_vec(),
        };
    }
    ParsedRustTarget {
        target: default_target.to_string(),
        tail: args.to_vec(),
    }
}

fn run_fast_cargo_checked(
    repo_root: &Path,
    label: &str,
    cargo_args: &[String],
) -> Result<(), String> {
    require_fast_cargo()?;
    println!("Running: cargo {}", cargo_args.join(" "));
    let status = Command::new("cargo")
        .current_dir(repo_root)
        .args(cargo_args)
        .status()
        .map_err(|source| format!("Could not run fast Rust {label}: {source}"))?;
    if !status.success() {
        return Err(format!(
            "Fast Rust {label} failed.\nReview the cargo output above, fix the Rust error, then retry."
        ));
    }
    Ok(())
}

fn require_fast_cargo() -> Result<(), String> {
    if command_exists("cargo") && command_exists("rustc") {
        return Ok(());
    }
    Err(
        "Fast Rust maintainer commands require cargo and rustc on PATH.\nAdd the maintainer Rust toolchain to the loaded Yazelix/Home Manager profile, then rerun."
            .to_string(),
    )
}

fn command_exists(command: &str) -> bool {
    Command::new(command).arg("--version").output().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: maintainer-owned `yzx dev rust test` keeps the same default target and passthrough behavior after leaving the user runtime helper.
    #[test]
    fn rust_test_defaults_to_core_and_preserves_cargo_tail() {
        let parsed = parse_rust_target_and_tail(&["config_ui".to_string()], "core");
        assert_eq!(
            parsed,
            ParsedRustTarget {
                target: "core".to_string(),
                tail: vec!["config_ui".to_string()],
            }
        );
    }

    // Defends: target parsing still recognizes explicit maintainer and all-target runs in the maintainer command surface.
    #[test]
    fn rust_target_specs_accept_known_targets_only() {
        let repo_root = Path::new("/repo");
        assert_eq!(rust_target_specs(repo_root, "all").unwrap().len(), 3);
        assert_eq!(rust_target_specs(repo_root, "maintainer").unwrap().len(), 1);
        assert!(rust_target_specs(repo_root, "website").is_err());
    }
}
