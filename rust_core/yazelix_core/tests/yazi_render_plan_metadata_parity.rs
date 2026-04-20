// Test lane: maintainer

use std::path::PathBuf;

fn read_repo_config_metadata(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../config_metadata")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn string_list(value: &toml::Value, ctx: &str) -> Vec<String> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("{ctx}: expected array"))
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("{ctx}: expected string elements"))
                .to_string()
        })
        .collect()
}

// Defends: yazi_render_plan.toml cannot drift from main_config_contract.toml for shared yazi.sort_by / yazi.plugins defaults.
// Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
#[test]
fn yazi_render_plan_toml_matches_main_contract() {
    let render_raw = read_repo_config_metadata("yazi_render_plan.toml");
    let render: toml::Value = toml::from_str(&render_raw).expect("parse yazi_render_plan.toml");

    let contract_raw = read_repo_config_metadata("main_config_contract.toml");
    let contract: toml::Value =
        toml::from_str(&contract_raw).expect("parse main_config_contract.toml");

    let fields = contract
        .get("fields")
        .expect("main_config_contract.toml missing [fields] tables");

    let sort_allowed = string_list(
        fields
            .get("yazi.sort_by")
            .and_then(|t| t.get("allowed_values"))
            .expect("contract missing yazi.sort_by allowed_values"),
        "contract yazi.sort_by.allowed_values",
    );
    let render_sort = string_list(
        render
            .get("sort_by_allowed")
            .expect("yazi_render_plan.toml missing sort_by_allowed"),
        "yazi_render_plan.toml sort_by_allowed",
    );
    assert_eq!(
        render_sort, sort_allowed,
        "sort_by_allowed must match main_config_contract.toml [fields.\"yazi.sort_by\"].allowed_values"
    );

    let plugins_default = string_list(
        fields
            .get("yazi.plugins")
            .and_then(|t| t.get("default"))
            .expect("contract missing yazi.plugins default"),
        "contract yazi.plugins.default",
    );
    let render_default_plugins = string_list(
        render
            .get("default_plugins")
            .expect("yazi_render_plan.toml missing default_plugins"),
        "yazi_render_plan.toml default_plugins",
    );
    assert_eq!(
        render_default_plugins, plugins_default,
        "default_plugins must match main_config_contract.toml [fields.\"yazi.plugins\"].default"
    );
}
