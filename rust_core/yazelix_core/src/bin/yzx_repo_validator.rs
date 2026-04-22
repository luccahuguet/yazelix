use yazelix_core::repo_validation::{
    repo_root, validate_default_test_traceability, validate_rust_test_traceability, validate_specs,
};

fn main() {
    let Some(command) = std::env::args().nth(1) else {
        eprintln!(
            "Usage: yzx_repo_validator <validate-specs|validate-default-test-traceability|validate-rust-test-traceability>"
        );
        std::process::exit(2);
    };

    let repo_root = repo_root();
    let report = match command.as_str() {
        "validate-specs" => validate_specs(&repo_root),
        "validate-default-test-traceability" => validate_default_test_traceability(&repo_root),
        "validate-rust-test-traceability" => validate_rust_test_traceability(&repo_root),
        _ => {
            eprintln!(
                "Unknown validator command `{}`. Expected validate-specs, validate-default-test-traceability, or validate-rust-test-traceability.",
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
                    _ => unreachable!(),
                };
                eprintln!("{}", failure_label);
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(1);
        }
    }
}
