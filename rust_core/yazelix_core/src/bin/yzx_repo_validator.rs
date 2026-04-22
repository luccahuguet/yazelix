use std::path::PathBuf;
use yazelix_core::repo_contract_validation::{
    UpgradeContractOptions, validate_config_surface_contract, validate_upgrade_contract,
};
use yazelix_core::repo_validation::{
    repo_root, validate_default_test_traceability, validate_rust_test_traceability, validate_specs,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let mut resolved_repo_root = repo_root();
    let Some(first_arg) = args.next() else {
        eprintln!(
            "Usage: yzx_repo_validator [--repo-root PATH] <validate-specs|validate-default-test-traceability|validate-rust-test-traceability|validate-config-surface-contract|validate-upgrade-contract>"
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
                "Usage: yzx_repo_validator [--repo-root PATH] <validate-specs|validate-default-test-traceability|validate-rust-test-traceability|validate-config-surface-contract|validate-upgrade-contract>"
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
                "Unknown validator command `{}`. Expected validate-specs, validate-default-test-traceability, validate-rust-test-traceability, validate-config-surface-contract, or validate-upgrade-contract.",
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
                    "validate-upgrade-contract" => "Upgrade contract validation failed",
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
