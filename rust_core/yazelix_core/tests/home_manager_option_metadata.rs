// Test lane: default

use yazelix_core::repo_contract_validation::validate_home_manager_option_declaration_contract;

mod support;

use support::fixtures::repo_root;

// Regression: Home Manager option docs must not serialize the Yazelix flake source store path into `options.json`.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn home_manager_option_declarations_use_stable_logical_source_path() {
    let errors = validate_home_manager_option_declaration_contract(&repo_root()).unwrap();
    assert!(errors.is_empty(), "{errors:#?}");
}
