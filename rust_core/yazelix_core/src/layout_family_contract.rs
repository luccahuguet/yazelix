//! Machine-readable contract for built-in Zellij layout families.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const CONTRACT_RELATIVE_PATH: &[&str] = &["config_metadata", "zellij_layout_families.toml"];
const LAYOUTS_RELATIVE_PATH: &[&str] = &["configs", "zellij", "layouts"];

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ZellijLayoutFamilyContract {
    pub schema_version: u32,
    #[serde(default)]
    pub layout_families: Vec<ZellijLayoutFamily>,
    #[serde(default)]
    pub auxiliary_layouts: Vec<ZellijAuxiliaryLayout>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ZellijLayoutFamily {
    pub id: String,
    pub layout_file: String,
    pub swap_layout_file: String,
    pub sidebar_enabled: bool,
    #[serde(default)]
    pub required_pane_names: Vec<String>,
    #[serde(default)]
    pub required_runtime_scripts: Vec<String>,
    #[serde(default)]
    pub swap_layouts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ZellijAuxiliaryLayout {
    pub id: String,
    pub layout_file: String,
    pub purpose: String,
}

pub fn zellij_layout_family_contract_path(root: &Path) -> PathBuf {
    CONTRACT_RELATIVE_PATH
        .iter()
        .fold(root.to_path_buf(), |path, segment| path.join(segment))
}

pub fn zellij_layouts_dir(root: &Path) -> PathBuf {
    LAYOUTS_RELATIVE_PATH
        .iter()
        .fold(root.to_path_buf(), |path, segment| path.join(segment))
}

pub fn load_zellij_layout_family_contract(
    root: &Path,
) -> Result<ZellijLayoutFamilyContract, String> {
    let path = zellij_layout_family_contract_path(root);
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
    toml::from_str(&raw).map_err(|error| format!("Invalid {}: {error}", path.display()))
}

pub fn expected_zellij_layout_template_files(
    contract: &ZellijLayoutFamilyContract,
) -> BTreeSet<String> {
    let mut expected = BTreeSet::new();
    for family in &contract.layout_families {
        expected.insert(family.layout_file.clone());
        expected.insert(family.swap_layout_file.clone());
    }
    for layout in &contract.auxiliary_layouts {
        expected.insert(layout.layout_file.clone());
    }
    expected
}

pub fn expected_zellij_generated_layout_files(root: &Path) -> Result<BTreeSet<String>, String> {
    let contract = load_zellij_layout_family_contract(root)?;
    Ok(expected_zellij_layout_template_files(&contract))
}

pub fn validate_zellij_layout_family_contract(root: &Path) -> Result<Vec<String>, String> {
    let contract = load_zellij_layout_family_contract(root)?;
    let layouts_dir = zellij_layouts_dir(root);
    let mut errors = Vec::new();

    if contract.schema_version != 1 {
        errors.push(format!(
            "Zellij layout family contract schema_version must be 1, got {}",
            contract.schema_version
        ));
    }

    let mut ids = BTreeSet::new();
    let mut layout_owners = BTreeMap::new();
    for family in &contract.layout_families {
        if !ids.insert(format!("family:{}", family.id)) {
            errors.push(format!("Duplicate Zellij layout family id `{}`", family.id));
        }
        record_layout_owner(
            &mut errors,
            &mut layout_owners,
            &family.layout_file,
            &family.id,
        );
        record_layout_owner(
            &mut errors,
            &mut layout_owners,
            &family.swap_layout_file,
            &family.id,
        );
        validate_family_files(&mut errors, &layouts_dir, family);
    }

    for layout in &contract.auxiliary_layouts {
        if !ids.insert(format!("auxiliary:{}", layout.id)) {
            errors.push(format!("Duplicate auxiliary layout id `{}`", layout.id));
        }
        record_layout_owner(
            &mut errors,
            &mut layout_owners,
            &layout.layout_file,
            &layout.id,
        );
        let path = layouts_dir.join(&layout.layout_file);
        if !path.is_file() {
            errors.push(format!(
                "Auxiliary layout `{}` points at missing file {}",
                layout.id,
                path.display()
            ));
        }
    }

    let actual_top_level_layouts = list_top_level_kdl_files(&layouts_dir)?;
    let expected = expected_zellij_layout_template_files(&contract);
    for missing in expected.difference(&actual_top_level_layouts) {
        errors.push(format!(
            "Zellij layout metadata expects `{missing}`, but the file is missing from {}",
            layouts_dir.display()
        ));
    }
    for untracked in actual_top_level_layouts.difference(&expected) {
        errors.push(format!(
            "Zellij layout file `{untracked}` is not represented in {}",
            zellij_layout_family_contract_path(root).display()
        ));
    }

    Ok(errors)
}

fn record_layout_owner(
    errors: &mut Vec<String>,
    owners: &mut BTreeMap<String, String>,
    layout_file: &str,
    owner: &str,
) {
    if let Some(previous) = owners.insert(layout_file.to_string(), owner.to_string()) {
        errors.push(format!(
            "Zellij layout file `{layout_file}` is owned by both `{previous}` and `{owner}`"
        ));
    }
}

fn validate_family_files(
    errors: &mut Vec<String>,
    layouts_dir: &Path,
    family: &ZellijLayoutFamily,
) {
    let layout_path = layouts_dir.join(&family.layout_file);
    let layout = match fs::read_to_string(&layout_path) {
        Ok(layout) => layout,
        Err(error) => {
            errors.push(format!(
                "Zellij layout family `{}` points at unreadable file {}: {error}",
                family.id,
                layout_path.display()
            ));
            return;
        }
    };

    if !layout.contains("__YAZELIX_KEYBINDS_COMMON__") {
        errors.push(format!(
            "Zellij layout `{}` must include the shared keybind placeholder",
            family.layout_file
        ));
    }

    for pane_name in &family.required_pane_names {
        if !layout.contains(&format!("pane name=\"{pane_name}\"")) {
            errors.push(format!(
                "Zellij layout `{}` for family `{}` is missing required pane name `{pane_name}`",
                family.layout_file, family.id
            ));
        }
    }
    for script in &family.required_runtime_scripts {
        if !layout.contains(script) {
            errors.push(format!(
                "Zellij layout `{}` for family `{}` is missing required runtime script `{script}`",
                family.layout_file, family.id
            ));
        }
    }

    let swap_path = layouts_dir.join(&family.swap_layout_file);
    let swap = match fs::read_to_string(&swap_path) {
        Ok(swap) => swap,
        Err(error) => {
            errors.push(format!(
                "Zellij layout family `{}` points at unreadable swap file {}: {error}",
                family.id,
                swap_path.display()
            ));
            return;
        }
    };

    if family.swap_layouts.is_empty() {
        errors.push(format!(
            "Zellij layout family `{}` must declare at least one swap layout",
            family.id
        ));
    }
    for swap_layout in &family.swap_layouts {
        if !swap.contains(&format!("swap_tiled_layout name=\"{swap_layout}\"")) {
            errors.push(format!(
                "Zellij swap file `{}` for family `{}` is missing swap layout `{swap_layout}`",
                family.swap_layout_file, family.id
            ));
        }
    }
}

fn list_top_level_kdl_files(dir: &Path) -> Result<BTreeSet<String>, String> {
    let mut files = BTreeSet::new();
    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {error}", dir.display()))?
    {
        let path = entry
            .map_err(|error| format!("Failed to read layout directory entry: {error}"))?
            .path();
        if path.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension == "kdl")
        {
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            files.insert(file_name.to_string());
        }
    }
    Ok(files)
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_fixture_repo() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let repo = tmp.path().to_path_buf();
        let metadata_dir = repo.join("config_metadata");
        let layouts_dir = zellij_layouts_dir(&repo);
        fs::create_dir_all(&metadata_dir).unwrap();
        fs::create_dir_all(&layouts_dir).unwrap();
        fs::write(
            metadata_dir.join("zellij_layout_families.toml"),
            r#"
schema_version = 1

[[layout_families]]
id = "sidebar"
layout_file = "yzx_side.kdl"
swap_layout_file = "yzx_side.swap.kdl"
sidebar_enabled = true
required_pane_names = ["sidebar"]
required_runtime_scripts = ["configs/zellij/scripts/launch_sidebar_yazi.nu"]
swap_layouts = ["single_open", "single_closed"]

[[auxiliary_layouts]]
id = "sweep_test"
layout_file = "yzx_sweep_test.kdl"
purpose = "test"
"#,
        )
        .unwrap();
        fs::write(
            layouts_dir.join("yzx_side.kdl"),
            r#"layout { pane name="sidebar" { args "__YAZELIX_RUNTIME_DIR__/configs/zellij/scripts/launch_sidebar_yazi.nu" } } __YAZELIX_KEYBINDS_COMMON__"#,
        )
        .unwrap();
        fs::write(
            layouts_dir.join("yzx_side.swap.kdl"),
            r#"swap_tiled_layout name="single_open" {} swap_tiled_layout name="single_closed" {}"#,
        )
        .unwrap();
        fs::write(layouts_dir.join("yzx_sweep_test.kdl"), "layout {}\n").unwrap();
        (tmp, repo)
    }

    // Defends: built-in Zellij layout metadata must describe the concrete tracked KDL files and their swap layouts.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn layout_family_contract_accepts_matching_files() {
        let (_tmp, repo) = write_fixture_repo();
        let errors = validate_zellij_layout_family_contract(&repo).unwrap();
        assert!(errors.is_empty(), "{errors:?}");
    }

    // Regression: adding or renaming a top-level built-in KDL layout must update machine-readable metadata too.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn layout_family_contract_rejects_untracked_layout_file() {
        let (_tmp, repo) = write_fixture_repo();
        fs::write(
            zellij_layouts_dir(&repo).join("surprise.kdl"),
            "layout {}\n",
        )
        .unwrap();

        let errors = validate_zellij_layout_family_contract(&repo).unwrap();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not represented"));
    }
}
