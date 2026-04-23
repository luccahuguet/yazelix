use std::path::PathBuf;
use yazelix_core::repo_contract_validation::{
    ColdProfileInstallOptions, UpgradeContractOptions, validate_config_surface_contract,
    validate_flake_interface, validate_flake_profile_install, validate_installed_runtime_contract,
    validate_nixpkgs_package, validate_nixpkgs_submission, validate_nushell_budget,
    validate_nushell_syntax, validate_readme_version, validate_upgrade_contract,
};
use yazelix_core::repo_validation::{
    repo_root, validate_default_test_traceability, validate_rust_test_traceability, validate_specs,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let mut resolved_repo_root = repo_root();
    let Some(first_arg) = args.next() else {
        eprintln!(
            "Usage: yzx_repo_validator [--repo-root PATH] <validate-specs|validate-default-test-traceability|validate-rust-test-traceability|validate-config-surface-contract|validate-nushell-budget|validate-upgrade-contract|validate-installed-runtime-contract|validate-flake-interface|validate-flake-profile-install|validate-nixpkgs-package|validate-nixpkgs-submission|validate-nushell-syntax|validate-readme-version>"
        );
        std::process::exit(2);
    };

    let (command, remaining_args) = if first_arg == "--repo-root" {
        let Some(path) = args.next() else {
            eprintln!("Missing PATH after --repo-root");
            std::process::exit(2);
        };
        resolved_repo_root = PathBuf::from(path);
        let Some(command) = args.next() else {
            eprintln!(
                "Usage: yzx_repo_validator [--repo-root PATH] <validate-specs|validate-default-test-traceability|validate-rust-test-traceability|validate-config-surface-contract|validate-nushell-budget|validate-upgrade-contract|validate-installed-runtime-contract|validate-flake-interface|validate-flake-profile-install|validate-nixpkgs-package|validate-nixpkgs-submission|validate-nushell-syntax|validate-readme-version>"
            );
            std::process::exit(2);
        };
        (command, args.collect::<Vec<_>>())
    } else {
        (first_arg, args.collect::<Vec<_>>())
    };

    let (report, success_label) = match command.as_str() {
        "validate-specs" => (validate_specs(&resolved_repo_root), None),
        "validate-default-test-traceability" => {
            (validate_default_test_traceability(&resolved_repo_root), None)
        }
        "validate-rust-test-traceability" => {
            (validate_rust_test_traceability(&resolved_repo_root), None)
        }
        "validate-config-surface-contract" => (
            validate_config_surface_contract(&resolved_repo_root),
            Some(
                "✅ Main config surface, Home Manager desktop entry, and generated-state contract is valid"
                    .to_string(),
            ),
        ),
        "validate-nushell-budget" => (
            validate_nushell_budget(&resolved_repo_root),
            Some("✅ Nushell budget allowlist and no-growth ceilings are valid".to_string()),
        ),
        "validate-installed-runtime-contract" => (
            validate_installed_runtime_contract(&resolved_repo_root),
            Some("✅ Installed-runtime contract smoke passed".to_string()),
        ),
        "validate-flake-interface" => (
            validate_flake_interface(&resolved_repo_root),
            Some("✅ Top-level flake interface is valid".to_string()),
        ),
        "validate-flake-profile-install" => {
            let options = parse_cold_profile_install_options(command.as_str(), remaining_args);
            (
                validate_flake_profile_install(&resolved_repo_root, &options),
                Some(match options.phase.as_str() {
                    "install" => "✅ Cold profile-install build phase passed".to_string(),
                    "verify" => "✅ Cold profile-install verification phase passed".to_string(),
                    _ => "✅ Cold profile-install check passed".to_string(),
                }),
            )
        }
        "validate-nixpkgs-package" => (
            validate_nixpkgs_package(&resolved_repo_root),
            Some("✅ Nixpkgs-style Yazelix package smoke check passed".to_string()),
        ),
        "validate-nixpkgs-submission" => (
            validate_nixpkgs_submission(&resolved_repo_root),
            Some("✅ Nixpkgs submission draft smoke check passed".to_string()),
        ),
        "validate-nushell-syntax" => {
            let verbose = parse_nushell_syntax_options(command.as_str(), remaining_args);
            (
                validate_nushell_syntax(&resolved_repo_root, verbose),
                Some("✅ Nushell syntax validation passed".to_string()),
            )
        }
        "validate-readme-version" => (
            validate_readme_version(&resolved_repo_root),
            Some("✅ README version and latest-series block are valid".to_string()),
        ),
        "validate-upgrade-contract" => {
            let mut options = UpgradeContractOptions::default();
            let mut args = remaining_args.into_iter();
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--ci" => options.ci = true,
                    "--diff-base" => {
                        let Some(diff_base) = args.next() else {
                            eprintln!("Missing value after --diff-base");
                            std::process::exit(2);
                        };
                        options.diff_base = Some(diff_base);
                    }
                    _ => {
                        eprintln!("Unknown validator command `{}` option `{}`.", command, arg);
                        std::process::exit(2);
                    }
                }
            }
            let success_label = if options.ci {
                "✅ Upgrade contract is valid in CI mode"
            } else {
                "✅ Upgrade contract is valid"
            };
            (
                validate_upgrade_contract(&resolved_repo_root, &options),
                Some(success_label.to_string()),
            )
        }
        _ => {
            eprintln!(
                "Unknown validator command `{}`. Expected validate-specs, validate-default-test-traceability, validate-rust-test-traceability, validate-config-surface-contract, validate-nushell-budget, validate-upgrade-contract, validate-installed-runtime-contract, validate-flake-interface, validate-flake-profile-install, validate-nixpkgs-package, validate-nixpkgs-submission, validate-nushell-syntax, or validate-readme-version.",
                command
            );
            std::process::exit(2);
        }
    };

    match report {
        Ok(report) => {
            for warning in report.warnings {
                println!("⚠️ {}", warning);
            }
            if !report.errors.is_empty() {
                for error in report.errors {
                    println!("❌ {}", error);
                }
                let failure_label = match command.as_str() {
                    "validate-specs" => "Spec traceability validation failed",
                    "validate-default-test-traceability" => {
                        "Governed test traceability validation failed"
                    }
                    "validate-rust-test-traceability" => "Rust test traceability validation failed",
                    "validate-config-surface-contract" => {
                        "Main config surface, Home Manager desktop entry, and generated-state contract validation failed"
                    }
                    "validate-nushell-budget" => "Nushell budget validation failed",
                    "validate-upgrade-contract" => "Upgrade contract validation failed",
                    "validate-installed-runtime-contract" => {
                        "Installed-runtime contract validation failed"
                    }
                    "validate-flake-interface" => "Top-level flake interface validation failed",
                    "validate-flake-profile-install" => "Cold profile-install validation failed",
                    "validate-nixpkgs-package" => "Nixpkgs package validation failed",
                    "validate-nixpkgs-submission" => "Nixpkgs submission validation failed",
                    "validate-nushell-syntax" => "Nushell syntax validation failed",
                    "validate-readme-version" => "README version validation failed",
                    _ => unreachable!(),
                };
                eprintln!("{}", failure_label);
                std::process::exit(1);
            }
            if let Some(success_label) = success_label {
                println!("{}", success_label);
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}

fn parse_cold_profile_install_options(
    command: &str,
    remaining_args: Vec<String>,
) -> ColdProfileInstallOptions {
    let mut options = ColdProfileInstallOptions::default();
    let mut args = remaining_args.into_iter();
    if let Some(phase) = args.next() {
        options.phase = phase;
    }
    if let Some(temp_home) = args.next() {
        options.temp_home = Some(PathBuf::from(temp_home));
    }
    if let Some(extra) = args.next() {
        eprintln!(
            "Unknown validator command `{}` extra argument `{}`.",
            command, extra
        );
        std::process::exit(2);
    }
    options
}

fn parse_nushell_syntax_options(command: &str, remaining_args: Vec<String>) -> bool {
    let mut verbose = false;
    for arg in remaining_args {
        match arg.as_str() {
            "--verbose" | "-v" => verbose = true,
            "--quiet" | "-q" => {}
            _ => {
                eprintln!("Unknown validator command `{}` option `{}`.", command, arg);
                std::process::exit(2);
            }
        }
    }
    verbose
}
