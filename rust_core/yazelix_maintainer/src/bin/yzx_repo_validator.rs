use std::path::{Path, PathBuf};
use yazelix_maintainer::repo_child_release::validate_child_release_transaction;
use yazelix_maintainer::repo_contract_validation::{
    ColdProfileInstallOptions, UpgradeContractOptions, validate_config_surface_contract,
    validate_flake_interface, validate_flake_profile_install, validate_installed_runtime_contract,
    validate_nix_customization_api, validate_nixpkgs_package, validate_nixpkgs_submission,
    validate_nushell_syntax, validate_readme_version, validate_upgrade_contract,
};
use yazelix_maintainer::repo_docs_validation::validate_docs_experience;
use yazelix_maintainer::repo_rust_budget::validate_rust_ownership_budget;
use yazelix_maintainer::repo_validation::{
    ValidationReport, repo_root, validate_contracts, validate_package_rust_test_purity,
    validate_rust_test_traceability,
};
use yazelix_maintainer::workspace_session_contract::validate_workspace_session_contract;

const USAGE_COMMANDS: &str = "validate-contracts|validate-rust-test-traceability|validate-package-rust-test-purity|validate-workspace-session-contract|validate-config-surface-contract|validate-docs-experience|validate-rust-ownership-budget|validate-child-release-transaction|validate-upgrade-contract|validate-installed-runtime-contract|validate-flake-interface|validate-flake-profile-install|validate-nix-customization-api|validate-nixpkgs-package|validate-nixpkgs-submission|validate-nushell-syntax|validate-readme-version";

fn main() {
    let invocation = parse_invocation(std::env::args().skip(1)).unwrap_or_else(|exit| exit.exit());
    let ValidatorInvocation {
        repo_root,
        command,
        remaining_args,
    } = invocation;
    let outcome = run_validator_command(&repo_root, &command, remaining_args)
        .unwrap_or_else(|exit| exit.exit());
    std::process::exit(emit_report(&command, outcome));
}

type ValidatorReport = Result<ValidationReport, String>;
type CommandOutcome = (ValidatorReport, Option<String>);
type SimpleValidatorFn = fn(&Path) -> ValidatorReport;

struct SimpleValidator(
    &'static str,
    &'static str,
    Option<&'static str>,
    SimpleValidatorFn,
);

const SIMPLE_VALIDATORS: &[SimpleValidator] = &[
    SimpleValidator(
        "validate-contracts",
        "Contract validation failed",
        None,
        validate_contracts,
    ),
    SimpleValidator(
        "validate-rust-test-traceability",
        "Rust test traceability validation failed",
        None,
        validate_rust_test_traceability,
    ),
    SimpleValidator(
        "validate-package-rust-test-purity",
        "Package-time Rust test purity validation failed",
        Some("✅ Package-time Rust tests are host-tool clean"),
        validate_package_rust_test_purity,
    ),
    SimpleValidator(
        "validate-workspace-session-contract",
        "Workspace/session contract validation failed",
        Some("✅ Workspace/session asset contract is valid"),
        validate_workspace_session_report,
    ),
    SimpleValidator(
        "validate-config-surface-contract",
        "Main config surface, Home Manager desktop entry, and generated-state contract validation failed",
        Some(
            "✅ Main config surface, Home Manager metadata, desktop entry, and generated-state contract is valid",
        ),
        validate_config_surface_contract,
    ),
    SimpleValidator(
        "validate-docs-experience",
        "Docs experience validation failed",
        Some("✅ Docs links, keybindings, and config examples are valid"),
        validate_docs_experience,
    ),
    SimpleValidator(
        "validate-rust-ownership-budget",
        "Rust ownership budget validation failed",
        Some("✅ Rust ownership budget and no-growth ceilings are valid"),
        validate_rust_ownership_budget,
    ),
    SimpleValidator(
        "validate-child-release-transaction",
        "Child release transaction validation failed",
        Some(
            "✅ First-party child input revisions are published, Cargo source hashes match, and child wasm package toolchains satisfy the release contract",
        ),
        validate_child_release_transaction,
    ),
    SimpleValidator(
        "validate-installed-runtime-contract",
        "Installed-runtime contract validation failed",
        Some("✅ Installed-runtime contract smoke passed"),
        validate_installed_runtime_contract,
    ),
    SimpleValidator(
        "validate-flake-interface",
        "Top-level flake interface validation failed",
        Some("✅ Top-level flake interface is valid"),
        validate_flake_interface,
    ),
    SimpleValidator(
        "validate-nix-customization-api",
        "Nix customization API validation failed",
        Some("✅ Nix customization API is valid"),
        validate_nix_customization_api,
    ),
    SimpleValidator(
        "validate-nixpkgs-package",
        "Nixpkgs package validation failed",
        Some("✅ Nixpkgs-style Yazelix package smoke check passed"),
        validate_nixpkgs_package,
    ),
    SimpleValidator(
        "validate-nixpkgs-submission",
        "Nixpkgs submission validation failed",
        Some("✅ Nixpkgs submission draft smoke check passed"),
        validate_nixpkgs_submission,
    ),
    SimpleValidator(
        "validate-readme-version",
        "README version validation failed",
        Some("✅ README version and latest-series block are valid"),
        validate_readme_version,
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatorInvocation {
    repo_root: PathBuf,
    command: String,
    remaining_args: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct CliExit {
    message: String,
    code: i32,
}

impl CliExit {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: 2,
        }
    }

    fn exit(self) -> ! {
        eprintln!("{}", self.message);
        std::process::exit(self.code);
    }
}

fn parse_invocation(
    args: impl IntoIterator<Item = String>,
) -> Result<ValidatorInvocation, CliExit> {
    let mut args = args.into_iter();
    let mut resolved_repo_root = repo_root();
    let Some(first_arg) = args.next() else {
        return Err(CliExit::usage(usage_message()));
    };

    let (command, remaining_args) = if first_arg == "--repo-root" {
        let Some(path) = args.next() else {
            return Err(CliExit::usage("Missing PATH after --repo-root"));
        };
        resolved_repo_root = PathBuf::from(path);
        let Some(command) = args.next() else {
            return Err(CliExit::usage(usage_message()));
        };
        (command, args.collect::<Vec<_>>())
    } else {
        (first_arg, args.collect::<Vec<_>>())
    };

    Ok(ValidatorInvocation {
        repo_root: resolved_repo_root,
        command,
        remaining_args,
    })
}

fn usage_message() -> String {
    format!("Usage: yzx_repo_validator [--repo-root PATH] <{USAGE_COMMANDS}>")
}

fn run_validator_command(
    repo_root: &Path,
    command: &str,
    remaining_args: Vec<String>,
) -> Result<CommandOutcome, CliExit> {
    if let Some(validator) = find_simple_validator(command) {
        return ok((validator.3)(repo_root), validator.2);
    }

    match command {
        "validate-upgrade-contract" => run_upgrade_contract_command(repo_root, remaining_args),
        "validate-flake-profile-install" => {
            run_flake_profile_install_command(repo_root, command, remaining_args)
        }
        "validate-nushell-syntax" => run_nushell_syntax_command(repo_root, command, remaining_args),
        _ => Err(CliExit::usage(format!(
            "Unknown validator command `{command}`. Expected one of: {USAGE_COMMANDS}."
        ))),
    }
}

fn ok(report: ValidatorReport, success_label: Option<&str>) -> Result<CommandOutcome, CliExit> {
    Ok((report, success_label.map(str::to_string)))
}

fn find_simple_validator(command: &str) -> Option<&'static SimpleValidator> {
    SIMPLE_VALIDATORS
        .iter()
        .find(|validator| validator.0 == command)
}

fn validate_workspace_session_report(repo_root: &Path) -> ValidatorReport {
    validate_workspace_session_contract(repo_root).map(|errors| ValidationReport {
        warnings: Vec::new(),
        errors,
    })
}

fn run_flake_profile_install_command(
    repo_root: &Path,
    command: &str,
    remaining_args: Vec<String>,
) -> Result<CommandOutcome, CliExit> {
    let options = parse_cold_profile_install_options(command, remaining_args)?;
    ok(
        validate_flake_profile_install(repo_root, &options),
        Some(match options.phase.as_str() {
            "install" => "✅ Cold profile-install build phase passed",
            "verify" => "✅ Cold profile-install verification phase passed",
            _ => "✅ Cold profile-install check passed",
        }),
    )
}

fn run_nushell_syntax_command(
    repo_root: &Path,
    command: &str,
    remaining_args: Vec<String>,
) -> Result<CommandOutcome, CliExit> {
    let verbose = parse_nushell_syntax_options(command, remaining_args)?;
    ok(
        validate_nushell_syntax(repo_root, verbose),
        Some("✅ Nushell syntax validation passed"),
    )
}

fn run_upgrade_contract_command(
    repo_root: &Path,
    remaining_args: Vec<String>,
) -> Result<CommandOutcome, CliExit> {
    let options = parse_upgrade_contract_options("validate-upgrade-contract", remaining_args)?;
    let success_label = if options.ci {
        "✅ Upgrade contract is valid in CI mode"
    } else {
        "✅ Upgrade contract is valid"
    };
    ok(
        validate_upgrade_contract(repo_root, &options),
        Some(success_label),
    )
}

fn emit_report(command: &str, outcome: CommandOutcome) -> i32 {
    match outcome.0 {
        Ok(report) => {
            for warning in report.warnings {
                println!("⚠️ {}", warning);
            }
            if !report.errors.is_empty() {
                for error in report.errors {
                    println!("❌ {}", error);
                }
                eprintln!("{}", failure_label(command));
                return 1;
            }
            if let Some(success_label) = outcome.1 {
                println!("{}", success_label);
            }
            0
        }
        Err(error) => {
            eprintln!("{}", error);
            1
        }
    }
}

fn failure_label(command: &str) -> &'static str {
    if let Some(validator) = find_simple_validator(command) {
        return validator.1;
    }

    match command {
        "validate-upgrade-contract" => "Upgrade contract validation failed",
        "validate-flake-profile-install" => "Cold profile-install validation failed",
        "validate-nushell-syntax" => "Nushell syntax validation failed",
        _ => unreachable!("unknown validator command reached report emission"),
    }
}

fn parse_cold_profile_install_options(
    command: &str,
    remaining_args: Vec<String>,
) -> Result<ColdProfileInstallOptions, CliExit> {
    let mut options = ColdProfileInstallOptions::default();
    let mut args = remaining_args.into_iter();
    if let Some(phase) = args.next() {
        options.phase = phase;
    }
    if let Some(temp_home) = args.next() {
        options.temp_home = Some(PathBuf::from(temp_home));
    }
    if let Some(extra) = args.next() {
        return Err(CliExit::usage(format!(
            "Unknown validator command `{}` extra argument `{}`.",
            command, extra
        )));
    }
    Ok(options)
}

fn parse_nushell_syntax_options(
    command: &str,
    remaining_args: Vec<String>,
) -> Result<bool, CliExit> {
    let mut verbose = false;
    for arg in remaining_args {
        match arg.as_str() {
            "--verbose" | "-v" => verbose = true,
            "--quiet" | "-q" => {}
            _ => {
                return Err(CliExit::usage(format!(
                    "Unknown validator command `{}` option `{}`.",
                    command, arg
                )));
            }
        }
    }
    Ok(verbose)
}

fn parse_upgrade_contract_options(
    command: &str,
    remaining_args: Vec<String>,
) -> Result<UpgradeContractOptions, CliExit> {
    let mut options = UpgradeContractOptions::default();
    let mut args = remaining_args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ci" => options.ci = true,
            "--diff-base" => {
                let Some(diff_base) = args.next() else {
                    return Err(CliExit::usage("Missing value after --diff-base"));
                };
                options.diff_base = Some(diff_base);
            }
            _ => {
                return Err(CliExit::usage(format!(
                    "Unknown validator command `{}` option `{}`.",
                    command, arg
                )));
            }
        }
    }
    Ok(options)
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the validator binary keeps supporting the repo-root override before command parsing.
    #[test]
    fn parses_repo_root_override_before_command() {
        let invocation = parse_invocation([
            "--repo-root".to_string(),
            "/tmp/yazelix".to_string(),
            "validate-nushell-syntax".to_string(),
            "--verbose".to_string(),
        ])
        .unwrap();

        assert_eq!(invocation.repo_root, PathBuf::from("/tmp/yazelix"));
        assert_eq!(invocation.command, "validate-nushell-syntax");
        assert_eq!(invocation.remaining_args, vec!["--verbose"]);
    }

    // Defends: command-specific option parsing keeps the same accepted flags after dispatch extraction.
    #[test]
    fn parses_upgrade_contract_options() {
        let options = parse_upgrade_contract_options(
            "validate-upgrade-contract",
            vec!["--ci".into(), "--diff-base".into(), "origin/main".into()],
        )
        .unwrap();

        assert!(options.ci);
        assert_eq!(options.diff_base.as_deref(), Some("origin/main"));
    }
}
