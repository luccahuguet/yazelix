// Test lane: maintainer

use std::fs;

use tempfile::tempdir;
use yazelix_maintainer::repo_yazelix_file_inventory::{
    InventoryOptions, collect_yazelix_file_inventory,
};

fn write(path: &std::path::Path, body: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

// Defends: the CodeDB/envctl inventory producer classifies every required target root before import.
#[test]
fn inventory_rows_cover_source_envctl_nix_store_and_local_targets() {
    let tmp = tempdir().unwrap();
    let yazelix_root = tmp.path().join("src/yazelix");
    let envctl_root = tmp.path().join("src/envctl");
    let meta_root = tmp.path().join("meta");
    let real_home = tmp.path().join("home");
    let nix_store = tmp.path().join("nix/store");
    let systemd_root = tmp.path().join("etc/systemd/user");

    write(&yazelix_root.join("settings_default.jsonc"), "{}\n");
    write(&envctl_root.join("manifest/base.toml"), "[components]\n");
    write(&real_home.join(".config/yazelix/settings.jsonc"), "{}\n");
    write(
        &meta_root.join(".local/share/yazelix/generated/nushell/config.nu"),
        "$env.PROMPT_COMMAND = {|| '' }\n",
    );
    write(
        &real_home.join(".local/share/yazelix/logs/welcome_1.log"),
        "hello\n",
    );
    write(
        &real_home.join(".local/share/applications/com.yazelix.Yazelix.Mars.desktop"),
        "[Desktop Entry]\nName=Yazelix\n",
    );
    write(
        &nix_store.join("abcd-yazelix-runtime/bin/yzx"),
        "#!/bin/sh\n",
    );
    write(&nix_store.join("wxyz-unrelated/bin/tool"), "#!/bin/sh\n");
    write(
        &systemd_root.join("yazelix-agent.service"),
        "[Service]\nExecStart=yzx\n",
    );

    let rows = collect_yazelix_file_inventory(&InventoryOptions {
        yazelix_root: yazelix_root.clone(),
        envctl_root: Some(envctl_root.clone()),
        meta_root: Some(meta_root.clone()),
        real_home: Some(real_home.clone()),
        nix_store: Some(nix_store.clone()),
        system_service_roots: vec![systemd_root.clone()],
    })
    .unwrap();

    assert!(rows.iter().any(|row| {
        row.absolute_path == yazelix_root.join("settings_default.jsonc")
            && row.owner == "yazelix"
            && row.source_of_truth_class == "repo_source"
            && row.parser_hint == "jsonc"
            && row.import_mode == "content_blob"
    }));
    assert!(rows.iter().any(|row| {
        row.absolute_path == envctl_root.join("manifest/base.toml")
            && row.owner == "envctl"
            && row.source_of_truth_class == "envctl_control_surface"
            && row.parser_hint == "toml"
    }));
    assert!(rows.iter().any(|row| {
        row.absolute_path == real_home.join(".config/yazelix/settings.jsonc")
            && row.source_of_truth_class == "real_home_user_config"
            && row.safety_policy == "real_home_metadata_first"
            && row.import_mode == "metadata_only"
    }));
    assert!(rows.iter().any(|row| {
        row.absolute_path == meta_root.join(".local/share/yazelix/generated/nushell/config.nu")
            && row.owner == "yazelix"
            && row.source_of_truth_class == "meta_local_generated"
            && row.safety_policy == "generated_content_import_allowed"
    }));
    assert!(rows.iter().any(|row| {
        row.absolute_path == real_home.join(".local/share/yazelix/logs/welcome_1.log")
            && row.source_of_truth_class == "real_home_runtime_state"
            && row.import_mode == "metadata_only"
            && row.safety_policy == "runtime_state_no_content_import"
    }));
    assert!(rows.iter().any(|row| {
        row.absolute_path
            == real_home.join(".local/share/applications/com.yazelix.Yazelix.Mars.desktop")
            && row.source_of_truth_class == "real_home_desktop_entry"
            && row.parser_hint == "desktop_entry"
    }));
    assert!(rows.iter().any(|row| {
        row.absolute_path == nix_store.join("abcd-yazelix-runtime")
            && row.source_of_truth_class == "nix_store_package_output"
            && row.mutability == "immutable"
            && row.import_mode == "metadata_only"
    }));
    assert!(rows.iter().any(|row| {
        row.absolute_path == systemd_root.join("yazelix-agent.service")
            && row.source_of_truth_class == "system_service_target"
            && row.parser_hint == "systemd_unit"
            && row.import_mode == "metadata_only"
    }));
    assert!(
        !rows
            .iter()
            .any(|row| row.absolute_path == nix_store.join("wxyz-unrelated"))
    );
}
