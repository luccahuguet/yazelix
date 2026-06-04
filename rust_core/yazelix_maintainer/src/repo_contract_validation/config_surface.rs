use super::{
    HOME_MANAGER_MODULE_DECLARATION_PATH, MAIN_CONTRACT_RELATIVE_PATH, MAIN_TEMPLATE_RELATIVE_PATH,
    MODULE_RELATIVE_PATH, TOML_TOOLING_CONFIG_RELATIVE_PATH, create_unique_temp_dir,
    escape_nix_string, format_json_value, format_toml_value, get_nested_toml_value,
    json_values_equal, read_toml_file, run_nix_eval, set_nested_toml_value, sorted_keys,
    split_field_path, toml_to_json, toml_values_equal,
};
use crate::repo_validation::ValidationReport;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use toml::{Table as TomlTable, Value as TomlValue};
use yazelix_core::config_state::{
    ComputeConfigStateRequest, ConfigStateData, RecordConfigStateRequest, compute_config_state,
    record_config_state,
};
use yazelix_core::settings_surface::{
    read_config_table, read_settings_jsonc_value, render_settings_jsonc_value,
};
use yazelix_core::{
    RuntimeApplyMode, YAZI_ACTIONS, YazelixActionMetadata, ZELLIJ_ACTIONS,
    ZELLIJ_NATIVE_KEYBINDINGS, runtime_apply_mode_codes,
};

pub fn validate_config_surface_contract(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    for errors in [
        validate_main_contract_parity(repo_root)?,
        validate_zellij_keybinding_registry_defaults(repo_root)?,
        validate_zellij_native_keybinding_registry_defaults(repo_root)?,
        validate_yazi_keybinding_registry_defaults(repo_root)?,
        validate_home_manager_option_declaration_contract(repo_root)?,
        validate_home_manager_desktop_entry_contract(repo_root)?,
        validate_home_manager_activation_contract(repo_root)?,
        validate_generated_state_contract(repo_root)?,
        validate_startup_snapshot_env_contract(repo_root)?,
    ] {
        report.errors.extend(errors);
    }
    Ok(report)
}

pub fn validate_home_manager_option_declaration_contract(
    repo_root: &Path,
) -> Result<Vec<String>, String> {
    let declarations = load_home_manager_option_declarations(repo_root)?;
    let mut errors = Vec::new();
    for (option_name, option_declarations) in declarations {
        for declaration in option_declarations {
            if declaration != HOME_MANAGER_MODULE_DECLARATION_PATH {
                errors.push(format!(
                    "Home Manager option `{}` declaration path must be `{}`, got `{}`",
                    option_name, HOME_MANAGER_MODULE_DECLARATION_PATH, declaration
                ));
            }
        }
    }
    Ok(errors)
}

fn validate_main_contract_parity(repo_root: &Path) -> Result<Vec<String>, String> {
    let contract = read_toml_file(&repo_root.join(MAIN_CONTRACT_RELATIVE_PATH))?;
    let template_json = read_settings_jsonc_value(&repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH))
        .map_err(|error| error.message())?;
    let template = read_config_table(
        &repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH),
        "read_main_settings_default",
    )
    .map_err(|error| error.message())?;
    let fields = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .ok_or_else(|| "main_config_contract.toml is missing its [fields] table".to_string())?;
    let declared_fields = sorted_keys(fields);
    let hm_option_names = declared_fields
        .iter()
        .filter_map(|field_path| {
            fields
                .get(field_path)
                .and_then(TomlValue::as_table)
                .and_then(|field| field.get("home_manager_option"))
                .and_then(TomlValue::as_str)
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    let hm_defaults = load_home_manager_defaults(repo_root, &hm_option_names)?;
    let mut errors = Vec::new();

    let declared_field_count = contract
        .get("contract")
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("field_count"))
        .and_then(TomlValue::as_integer)
        .unwrap_or_default() as usize;
    if declared_field_count != declared_fields.len() {
        errors.push(format!(
            "main_config_contract.toml field_count mismatch: declared={}, actual={}",
            declared_field_count,
            declared_fields.len()
        ));
    }

    for field_path in declared_fields {
        let Some(field) = fields.get(&field_path).and_then(TomlValue::as_table) else {
            continue;
        };
        validate_main_contract_apply_mode(&field_path, field, &mut errors);
        let hm_option = field
            .get("home_manager_option")
            .and_then(TomlValue::as_str)
            .unwrap_or_default();
        if !hm_defaults.contains_key(hm_option) {
            errors.push(format!(
                "Home Manager option `{}` is missing for main-contract field `{}`",
                hm_option, field_path
            ));
            continue;
        }

        let expected_hm_default = if field
            .get("home_manager_default_is_null")
            .and_then(TomlValue::as_bool)
            .unwrap_or(false)
        {
            JsonValue::Null
        } else {
            toml_to_json(
                field
                    .get("default")
                    .unwrap_or(&TomlValue::String(String::new())),
            )
        };
        let actual_hm_default = hm_defaults
            .get(hm_option)
            .cloned()
            .unwrap_or(JsonValue::Null);
        if !json_values_equal(&actual_hm_default, &expected_hm_default) {
            errors.push(format!(
                "Home Manager default mismatch for `{}` via `{}`: expected {}, got {}",
                field_path,
                hm_option,
                format_json_value(&expected_hm_default),
                format_json_value(&actual_hm_default)
            ));
        }

        let emit_in_template = field
            .get("emit_in_default_template")
            .and_then(TomlValue::as_bool)
            .unwrap_or(true);
        let template_value = get_nested_toml_value(&template, &split_field_path(&field_path));

        if !emit_in_template {
            if let Some(value) = template_value {
                errors.push(format!(
                    "Default template should omit `{}`, but it is present with value {}",
                    field_path,
                    format_toml_value(value)
                ));
            }
            continue;
        }

        if field
            .get("home_manager_default_is_null")
            .and_then(TomlValue::as_bool)
            .unwrap_or(false)
            && field.get("kind").and_then(TomlValue::as_str) == Some("helix_external")
        {
            match get_nested_json_value(&template_json, &split_field_path(&field_path)) {
                Some(JsonValue::Null) => continue,
                Some(value) => {
                    errors.push(format!(
                        "Default template mismatch for `{}`: expected null, got {}",
                        field_path,
                        format_json_value(value)
                    ));
                    continue;
                }
                None => {
                    errors.push(format!(
                        "Default template is missing required field `{}`",
                        field_path
                    ));
                    continue;
                }
            }
        }

        let Some(template_value) = template_value else {
            errors.push(format!(
                "Default template is missing required field `{}`",
                field_path
            ));
            continue;
        };
        let expected_template_value = field
            .get("default")
            .ok_or_else(|| format!("Config contract field `{field_path}` is missing `default`"))?;
        if !toml_values_equal(template_value, expected_template_value) {
            errors.push(format!(
                "Default template mismatch for `{}`: expected {}, got {}",
                field_path,
                format_toml_value(expected_template_value),
                format_toml_value(template_value)
            ));
        }
    }

    Ok(errors)
}

fn get_nested_json_value<'a>(value: &'a JsonValue, path: &[&str]) -> Option<&'a JsonValue> {
    let mut current = value;
    for segment in path {
        current = current.as_object()?.get(*segment)?;
    }
    Some(current)
}

fn validate_main_contract_apply_mode(
    field_path: &str,
    field: &TomlTable,
    errors: &mut Vec<String>,
) {
    let Some(apply_mode) = field.get("apply_mode").and_then(TomlValue::as_str) else {
        errors.push(format!(
            "main_config_contract.toml field `{field_path}` is missing apply_mode"
        ));
        return;
    };

    if apply_mode.parse::<RuntimeApplyMode>().is_err() {
        errors.push(format!(
            "main_config_contract.toml field `{field_path}` has unsupported apply_mode `{}`; expected one of: {}",
            apply_mode,
            runtime_apply_mode_codes().join(", ")
        ));
    }
}

fn validate_zellij_keybinding_registry_defaults(repo_root: &Path) -> Result<Vec<String>, String> {
    validate_keybinding_registry_defaults(
        repo_root,
        "zellij.keybindings",
        collect_action_registry_defaults(ZELLIJ_ACTIONS.iter().map(|spec| &spec.action)),
    )
}

fn validate_zellij_native_keybinding_registry_defaults(
    repo_root: &Path,
) -> Result<Vec<String>, String> {
    validate_keybinding_registry_defaults(
        repo_root,
        "zellij.native_keybindings",
        collect_action_registry_defaults(ZELLIJ_NATIVE_KEYBINDINGS.iter().map(|spec| &spec.action)),
    )
}

fn validate_yazi_keybinding_registry_defaults(repo_root: &Path) -> Result<Vec<String>, String> {
    validate_keybinding_registry_defaults(
        repo_root,
        "yazi.keybindings",
        collect_action_registry_defaults(YAZI_ACTIONS.iter().map(|spec| &spec.action)),
    )
}

fn collect_action_registry_defaults<'a>(
    actions: impl IntoIterator<Item = &'a YazelixActionMetadata>,
) -> BTreeMap<String, Vec<String>> {
    actions
        .into_iter()
        .map(|action| {
            (
                action.local_id.to_string(),
                action
                    .default_keys
                    .iter()
                    .map(|key| (*key).to_string())
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn validate_keybinding_registry_defaults(
    repo_root: &Path,
    field_path: &str,
    registry_defaults: BTreeMap<String, Vec<String>>,
) -> Result<Vec<String>, String> {
    let contract = read_toml_file(&repo_root.join(MAIN_CONTRACT_RELATIVE_PATH))?;
    let contract_defaults = load_contract_keybinding_defaults(&contract, field_path)?;
    let contract_ids = contract_defaults.keys().cloned().collect::<BTreeSet<_>>();
    let registry_ids = registry_defaults.keys().cloned().collect::<BTreeSet<_>>();
    let mut errors = Vec::new();

    for missing in registry_ids.difference(&contract_ids) {
        errors.push(format!(
            "main_config_contract.toml {field_path} defaults are missing action `{missing}` from the Rust action registry"
        ));
    }
    for extra in contract_ids.difference(&registry_ids) {
        errors.push(format!(
            "main_config_contract.toml {field_path} default `{extra}` is not present in the Rust action registry"
        ));
    }
    for action_id in contract_ids.intersection(&registry_ids) {
        let contract_keys = contract_defaults
            .get(action_id)
            .expect("intersection key exists in contract defaults");
        let registry_keys = registry_defaults
            .get(action_id)
            .expect("intersection key exists in registry defaults");
        if contract_keys != registry_keys {
            errors.push(format!(
                "main_config_contract.toml {field_path} default mismatch for `{}`: contract=[{}], registry=[{}]",
                action_id,
                contract_keys.join(", "),
                registry_keys.join(", ")
            ));
        }
    }

    Ok(errors)
}

fn load_contract_keybinding_defaults(
    contract: &TomlTable,
    field_path: &str,
) -> Result<BTreeMap<String, Vec<String>>, String> {
    let defaults = contract
        .get("fields")
        .and_then(TomlValue::as_table)
        .and_then(|fields| fields.get(field_path))
        .and_then(TomlValue::as_table)
        .and_then(|field| field.get("default"))
        .and_then(TomlValue::as_table)
        .ok_or_else(|| {
            format!("main_config_contract.toml is missing [fields.\"{field_path}\".default]")
        })?;

    let mut parsed = BTreeMap::new();
    for (action_id, raw_keys) in defaults {
        let Some(keys) = raw_keys.as_array() else {
            return Err(format!(
                "main_config_contract.toml {field_path} default `{action_id}` must be an array of strings"
            ));
        };
        let mut parsed_keys = Vec::new();
        for key in keys {
            let Some(key) = key.as_str() else {
                return Err(format!(
                    "main_config_contract.toml {field_path} default `{action_id}` must contain only strings"
                ));
            };
            parsed_keys.push(key.to_string());
        }
        parsed.insert(action_id.clone(), parsed_keys);
    }
    Ok(parsed)
}

fn validate_home_manager_desktop_entry_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let entry = load_home_manager_desktop_entry_contract(repo_root)?;
    let shader_entry = load_home_manager_desktop_entry_contract_with_profile(repo_root, "shaders")?;
    let extra_entry = load_home_manager_extra_terminal_launchers_contract(repo_root)?;
    let is_present = entry
        .get("present")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false);
    let actual_exec = entry
        .get("exec")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let actual_name = entry
        .get("name")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let mut errors = Vec::new();

    if !is_present {
        errors.push("Home Manager Linux Ghostty desktop entry must be generated".to_string());
    }

    if actual_name != "New Yazelix - Ghostty" {
        errors.push(format!(
            "Home Manager Ghostty desktop entry name mismatch: expected New Yazelix - Ghostty, got {}",
            format_json_value(&JsonValue::String(actual_name.to_string()))
        ));
    }

    if !entry
        .get("terminal")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        errors.push(
            "Home Manager desktop entry must set terminal = true so pre-terminal config failures stay visible"
                .to_string(),
        );
    }

    if actual_exec != "/tmp/profile/bin/yzx desktop launch" {
        errors.push(format!(
            "Home Manager desktop entry Exec mismatch: expected /tmp/profile/bin/yzx desktop launch, got {}",
            format_json_value(&JsonValue::String(actual_exec.to_string()))
        ));
    }
    let shader_exec = shader_entry
        .get("exec")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let shader_name = shader_entry
        .get("name")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let shader_session_profile = shader_entry
        .get("sessionProfile")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    if !shader_entry
        .get("terminal")
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
    {
        errors.push(
            "Home Manager yzxterm desktop entry must set terminal = true so pre-terminal config failures stay visible"
                .to_string(),
        );
    }
    if shader_exec != "env YAZELIX_TERMINAL_PROFILE=shaders /tmp/profile/bin/yzx desktop launch" {
        errors.push(format!(
            "Home Manager shader yzxterm profile desktop entry Exec mismatch: expected env YAZELIX_TERMINAL_PROFILE=shaders /tmp/profile/bin/yzx desktop launch, got {}",
            format_json_value(&JsonValue::String(shader_exec.to_string()))
        ));
    }
    if shader_name != "New Yazelix - Yzxterm" {
        errors.push(format!(
            "Home Manager yzxterm desktop entry name mismatch: expected New Yazelix - Yzxterm, got {}",
            format_json_value(&JsonValue::String(shader_name.to_string()))
        ));
    }
    if shader_session_profile != "shaders" {
        errors.push(format!(
            "Home Manager shader yzxterm profile session variable mismatch: expected shaders, got {}",
            format_json_value(&JsonValue::String(shader_session_profile.to_string()))
        ));
    }
    let package_count = extra_entry
        .get("packageCount")
        .and_then(JsonValue::as_i64)
        .unwrap_or_default();
    if package_count != 1 {
        errors.push(format!(
            "Home Manager extra terminal launchers must not add duplicate profile packages: expected 1 active package, got {package_count}"
        ));
    }
    for (terminal, expected_name) in [
        ("ghostty", "New Yazelix - Ghostty"),
        ("foot", "New Yazelix - Foot"),
        ("yzxterm", "New Yazelix - Yzxterm"),
        ("rio", "New Yazelix - Rio"),
        ("wezterm", "New Yazelix - WezTerm"),
    ] {
        let entry = extra_entry
            .get(terminal)
            .and_then(JsonValue::as_object)
            .cloned()
            .unwrap_or_default();
        if !entry
            .get("present")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
        {
            errors.push(format!(
                "Home Manager extra {terminal} desktop entry must be generated"
            ));
            continue;
        }
        let name = entry
            .get("name")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        if name != expected_name {
            errors.push(format!(
                "Home Manager extra {terminal} desktop entry name mismatch: expected {expected_name}, got {}",
                format_json_value(&JsonValue::String(name.to_string()))
            ));
        }
        let exec = entry
            .get("exec")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        if !exec.ends_with("/bin/yzx desktop launch") || exec.contains("/tmp/profile/bin/yzx") {
            errors.push(format!(
                "Home Manager extra {terminal} desktop entry Exec must point at the terminal package store yzx, got {}",
                format_json_value(&JsonValue::String(exec.to_string()))
            ));
        }
        if !exec.starts_with("env YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1 ") {
            errors.push(format!(
                "Home Manager extra {terminal} desktop entry Exec must disable stable-profile redirects for intentional variant package launches, got {}",
                format_json_value(&JsonValue::String(exec.to_string()))
            ));
        }
        if terminal == "yzxterm" && !exec.contains("YAZELIX_TERMINAL_PROFILE=shaders") {
            errors.push(format!(
                "Home Manager extra yzxterm desktop entry Exec must pass the configured yzxterm profile, got {}",
                format_json_value(&JsonValue::String(exec.to_string()))
            ));
        }
        if !entry
            .get("terminal")
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
        {
            errors.push(format!(
                "Home Manager extra {terminal} desktop entry must set terminal = true so pre-terminal config failures stay visible"
            ));
        }
    }

    validate_home_manager_darwin_without_desktop_entry_option(repo_root)?;

    Ok(errors)
}

fn validate_home_manager_activation_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let script = load_home_manager_activation_contract(repo_root)?;
    let materialization_lines = script
        .lines()
        .filter(|line| line.contains("terminal-materialization.generate --from-env"))
        .collect::<Vec<_>>();
    let mut errors = Vec::new();

    if materialization_lines.len() != 2 {
        errors.push(format!(
            "Home Manager activation with one extra terminal launcher must generate active and extra terminal configs, got {} materialization command(s)",
            materialization_lines.len()
        ));
    }
    let yzxterm_line = materialization_lines
        .iter()
        .find(|line| line.contains("-yazelix-yzxterm/"));
    if yzxterm_line.is_none() {
        errors.push(
            "Home Manager activation must materialize the extra yzxterm launcher runtime"
                .to_string(),
        );
    }
    if !yzxterm_line
        .map(|line| line.contains("YAZELIX_TERMINAL_PROFILE=shaders"))
        .unwrap_or(false)
    {
        errors.push(
            "Home Manager activation must pass yzxterm_profile to extra yzxterm launcher materialization"
                .to_string(),
        );
    }
    if !materialization_lines
        .iter()
        .all(|line| line.contains("/libexec/yzx_core"))
    {
        errors.push(
            "Home Manager activation terminal materialization must run each terminal package's own yzx_core"
                .to_string(),
        );
    }

    Ok(errors)
}

fn load_home_manager_activation_contract(repo_root: &Path) -> Result<String, String> {
    let expr = build_home_manager_activation_expr(repo_root);
    let result = run_nix_eval(repo_root, &expr)?;
    result.as_str().map(str::to_string).ok_or_else(|| {
        "Home Manager activation evaluation did not return a JSON string".to_string()
    })
}

fn build_home_manager_activation_expr(repo_root: &Path) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"x86_64-linux\"; };".to_string(),
        "  lib = pkgs.lib.extend (_: super: { hm = { dag = { entryAfter = after: data: { inherit after data; }; }; }; });".to_string(),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; nixgl = null; yazelixCursorsPackage = null; yazelixTerminalPackage = null; mkYazelixPackage = args: pkgs.runCommand (args.name or \"yazelix\") {} ''mkdir -p $out/bin $out/libexec $out/toolbin; touch $out/bin/yzx $out/libexec/yzx_core $out/libexec/yzx_control''; };".to_string(),
        "    modules = [".to_string(),
        format!("      (builtins.toPath \"{}\")", module_path),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, true));
    lines.extend([
        "      { config.programs.yazelix.extra_terminal_launchers = [ \"yzxterm\" ]; }".to_string(),
        "      { config.programs.yazelix.yzxterm_profile = \"shaders\"; }".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        "in eval.config.home.activation.yazelixGeneratedRuntimeConfigs.data".to_string(),
    ]);
    lines.join("\n")
}

fn validate_generated_state_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let fixture = setup_config_state_fixture(repo_root)?;
    let mut errors = Vec::new();

    let validation = (|| -> Result<(), String> {
        let baseline = compute_fixture_state(&fixture, &fixture.runtime_root)?;
        record_fixture_state(&fixture, &baseline)?;

        mutate_fixture_config(
            &fixture.main_config_path,
            "core.skip_welcome_screen",
            TomlValue::Boolean(true),
        )?;
        let after_runtime_only = compute_fixture_state(&fixture, &fixture.runtime_root)?;
        if baseline.config_hash != after_runtime_only.config_hash {
            errors.push(
                "Non-rebuild runtime config change unexpectedly altered config_hash".to_string(),
            );
        }
        if baseline.combined_hash != after_runtime_only.combined_hash {
            errors.push(
                "Non-rebuild runtime config change unexpectedly altered combined_hash".to_string(),
            );
        }
        if after_runtime_only.needs_refresh {
            errors.push(
                "Non-rebuild runtime config change unexpectedly marked generated state as stale"
                    .to_string(),
            );
        }

        mutate_fixture_config(
            &fixture.main_config_path,
            "editor.command",
            TomlValue::String("nvim".to_string()),
        )?;
        let after_rebuild_config = compute_fixture_state(&fixture, &fixture.runtime_root)?;
        if after_runtime_only.config_hash == after_rebuild_config.config_hash {
            errors.push("Rebuild-relevant config change did not alter config_hash".to_string());
        }
        if after_runtime_only.combined_hash == after_rebuild_config.combined_hash {
            errors.push("Rebuild-relevant config change did not alter combined_hash".to_string());
        }
        if !after_rebuild_config.needs_refresh {
            errors.push(
                "Rebuild-relevant config change did not mark generated state as stale".to_string(),
            );
        }

        record_fixture_state(&fixture, &after_rebuild_config)?;
        let after_runtime_root_change = compute_fixture_state(&fixture, &fixture.runtime_root_alt)?;
        if after_rebuild_config.config_hash != after_runtime_root_change.config_hash {
            errors.push(
                "Changing only the runtime root unexpectedly altered config_hash".to_string(),
            );
        }
        if after_rebuild_config.runtime_hash == after_runtime_root_change.runtime_hash {
            errors.push("Changing the runtime root did not alter runtime_hash".to_string());
        }
        if after_rebuild_config.combined_hash == after_runtime_root_change.combined_hash {
            errors.push("Changing the runtime root did not alter combined_hash".to_string());
        }
        if !after_runtime_root_change.needs_refresh {
            errors.push(
                "Changing the runtime root did not mark generated state as stale".to_string(),
            );
        }

        Ok(())
    })();

    if let Err(error) = validation {
        errors.push(format!(
            "Generated-state contract validation failed unexpectedly: {}",
            error
        ));
    }

    let _ = fs::remove_dir_all(&fixture.fixture_root);
    Ok(errors)
}

fn validate_startup_snapshot_env_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let relative_path = "rust_core/yazelix_core/src/launch_commands/enter.rs";
    let path = repo_root.join(relative_path);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    Ok(validate_startup_snapshot_env_contract_content(
        relative_path,
        &content,
    ))
}

fn validate_startup_snapshot_env_contract_content(label: &str, content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for required in [
        "write_session_config_snapshot_for_launch",
        "\"YAZELIX_SESSION_CONFIG_PATH\"",
        "\"YAZELIX_STATUS_BAR_CACHE_PATH\"",
        "command_status_with_overrides",
    ] {
        if !content.contains(required) {
            errors.push(format!(
                "{} must keep Rust-owned startup snapshot env contract token `{}`",
                label, required
            ));
        }
    }

    errors
}

fn load_home_manager_defaults(
    repo_root: &Path,
    option_names: &[String],
) -> Result<JsonMap<String, JsonValue>, String> {
    let expr = build_home_manager_defaults_expr(repo_root, option_names);
    let result = run_nix_eval(repo_root, &expr)?;
    result
        .as_object()
        .cloned()
        .ok_or_else(|| "Home Manager defaults evaluation did not return a JSON object".to_string())
}

fn build_home_manager_defaults_expr(repo_root: &Path, option_names: &[String]) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut names = option_names.to_vec();
    names.sort();
    names.dedup();
    let bindings = names
        .into_iter()
        .map(|name| {
            format!(
                "  {} = module.options.programs.yazelix.{}.default;",
                name, name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    [
        "let".to_string(),
        "  pkgs = import <nixpkgs> {};".to_string(),
        "  lib = pkgs.lib;".to_string(),
        format!(
            "  module = import (builtins.toPath \"{}\") {{ inherit lib pkgs; options = {{}}; config = {{ programs.yazelix = {{}}; xdg.configHome = \"/tmp\"; }}; }};",
            module_path
        ),
        "in {".to_string(),
        bindings,
        "}".to_string(),
    ]
    .join("\n")
}

fn load_home_manager_option_declarations(
    repo_root: &Path,
) -> Result<HashMap<String, Vec<String>>, String> {
    let expr = build_home_manager_option_declarations_expr(repo_root);
    let result = run_nix_eval(repo_root, &expr)?;
    serde_json::from_value(result).map_err(|error| {
        format!("Home Manager option declaration evaluation returned invalid JSON: {error}")
    })
}

fn build_home_manager_option_declarations_expr(repo_root: &Path) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> {};".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; nixgl = null; };".to_string(),
        "    modules = [".to_string(),
        format!("      (builtins.toPath \"{}\")", module_path),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, false));
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        "in builtins.mapAttrs (_: option: map builtins.toString option.declarations) eval.options.programs.yazelix".to_string(),
    ]);
    lines.join("\n")
}

fn standalone_home_manager_eval_fixture_module(
    include_desktop_entries_option: bool,
    enable_yazelix: bool,
) -> Vec<String> {
    let mut lines = vec![
        "      ({ lib, ... }: {".to_string(),
        "        options.assertions = lib.mkOption { type = lib.types.listOf lib.types.anything; default = []; };".to_string(),
        "        options.xdg.configHome = lib.mkOption { type = lib.types.str; default = \"/tmp/config\"; };".to_string(),
        "        options.xdg.dataHome = lib.mkOption { type = lib.types.str; default = \"/tmp/data\"; };".to_string(),
        "        options.xdg.dataFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.xdg.configFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
    ];
    if include_desktop_entries_option {
        lines.push("        options.xdg.desktopEntries = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string());
    }
    lines.extend([
        "        options.home.packages = lib.mkOption { type = lib.types.listOf lib.types.package; default = []; };".to_string(),
        "        options.home.activation = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };".to_string(),
        "        options.home.profileDirectory = lib.mkOption { type = lib.types.str; default = \"/tmp/profile\"; };".to_string(),
        "        options.home.sessionVariables = lib.mkOption { type = lib.types.attrsOf lib.types.str; default = {}; };".to_string(),
    ]);
    if enable_yazelix {
        lines.push("        config.programs.yazelix.enable = true;".to_string());
    }
    lines.push("      })".to_string());
    lines
}

fn load_home_manager_desktop_entry_contract(
    repo_root: &Path,
) -> Result<JsonMap<String, JsonValue>, String> {
    let expr = build_home_manager_desktop_entry_expr(repo_root, None, &[], None);
    let result = run_nix_eval(repo_root, &expr)?;
    result.as_object().cloned().ok_or_else(|| {
        "Home Manager desktop-entry evaluation did not return a JSON object".to_string()
    })
}

fn load_home_manager_desktop_entry_contract_with_profile(
    repo_root: &Path,
    yzxterm_profile: &str,
) -> Result<JsonMap<String, JsonValue>, String> {
    let expr = build_home_manager_desktop_entry_expr(
        repo_root,
        Some(yzxterm_profile),
        &[],
        Some("yzxterm"),
    );
    let result = run_nix_eval(repo_root, &expr)?;
    result.as_object().cloned().ok_or_else(|| {
        "Home Manager desktop-entry evaluation did not return a JSON object".to_string()
    })
}

fn load_home_manager_extra_terminal_launchers_contract(
    repo_root: &Path,
) -> Result<JsonMap<String, JsonValue>, String> {
    let expr = build_home_manager_desktop_entry_expr(
        repo_root,
        Some("shaders"),
        &["ghostty", "yzxterm", "foot", "rio", "wezterm"],
        Some("kitty"),
    );
    let result = run_nix_eval(repo_root, &expr)?;
    result.as_object().cloned().ok_or_else(|| {
        "Home Manager desktop-entry evaluation did not return a JSON object".to_string()
    })
}

fn build_home_manager_desktop_entry_expr(
    repo_root: &Path,
    yzxterm_profile: Option<&str>,
    extra_launchers: &[&str],
    active_terminal: Option<&str>,
) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"x86_64-linux\"; };".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; nixgl = null; yazelixCursorsPackage = null; yazelixTerminalPackage = null; mkYazelixPackage = args: pkgs.runCommand (args.name or \"yazelix\") {} \"mkdir -p $out/bin; touch $out/bin/yzx\"; };".to_string(),
        "    modules = [".to_string(),
        format!("      (builtins.toPath \"{}\")", module_path),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(true, true));
    if let Some(profile) = yzxterm_profile {
        lines.push(format!(
            "      {{ config.programs.yazelix.yzxterm_profile = \"{}\"; }}",
            escape_nix_string(profile)
        ));
    }
    if let Some(terminal) = active_terminal {
        lines.push(format!(
            "      {{ config.programs.yazelix.terminal = \"{}\"; }}",
            escape_nix_string(terminal)
        ));
    }
    if !extra_launchers.is_empty() {
        let launchers = extra_launchers
            .iter()
            .map(|terminal| format!("\"{}\"", escape_nix_string(terminal)))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(format!(
            "      {{ config.programs.yazelix.extra_terminal_launchers = [ {launchers} ]; }}"
        ));
    }
    let entry_key = match active_terminal {
        Some("foot") => "com.yazelix.Yazelix.Foot",
        Some("kitty") => "com.yazelix.Yazelix.Kitty",
        Some("rio") => "com.yazelix.Yazelix.Rio",
        Some("wezterm") => "com.yazelix.Yazelix.WezTerm",
        Some("yzxterm") => "com.yazelix.Yazelix.Yzxterm",
        _ => "com.yazelix.Yazelix.Ghostty",
    };
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        format!("  entryKey = \"{}\";", entry_key),
        "  ghosttyKey = \"com.yazelix.Yazelix.Ghostty\";".to_string(),
        "  footKey = \"com.yazelix.Yazelix.Foot\";".to_string(),
        "  yzxtermKey = \"com.yazelix.Yazelix.Yzxterm\";".to_string(),
        "  rioKey = \"com.yazelix.Yazelix.Rio\";".to_string(),
        "  weztermKey = \"com.yazelix.Yazelix.WezTerm\";".to_string(),
        "  entries = eval.config.xdg.desktopEntries;".to_string(),
        "  entry = if builtins.hasAttr entryKey entries then builtins.getAttr entryKey entries else {};".to_string(),
        "  ghosttyEntry = if builtins.hasAttr ghosttyKey entries then builtins.getAttr ghosttyKey entries else {};".to_string(),
        "  footEntry = if builtins.hasAttr footKey entries then builtins.getAttr footKey entries else {};".to_string(),
        "  yzxtermEntry = if builtins.hasAttr yzxtermKey entries then builtins.getAttr yzxtermKey entries else {};".to_string(),
        "  rioEntry = if builtins.hasAttr rioKey entries then builtins.getAttr rioKey entries else {};".to_string(),
        "  weztermEntry = if builtins.hasAttr weztermKey entries then builtins.getAttr weztermKey entries else {};".to_string(),
        "in {".to_string(),
        "  present = builtins.hasAttr entryKey entries;".to_string(),
        "  name = entry.name or \"\";".to_string(),
        "  exec = entry.exec or \"\";".to_string(),
        "  terminal = entry.terminal or false;".to_string(),
        "  sessionProfile = eval.config.home.sessionVariables.YAZELIX_TERMINAL_PROFILE or \"\";"
            .to_string(),
        "  packageCount = builtins.length eval.config.home.packages;".to_string(),
        "  ghostty = { present = builtins.hasAttr ghosttyKey entries; name = ghosttyEntry.name or \"\"; exec = ghosttyEntry.exec or \"\"; terminal = ghosttyEntry.terminal or false; };".to_string(),
        "  foot = { present = builtins.hasAttr footKey entries; name = footEntry.name or \"\"; exec = footEntry.exec or \"\"; terminal = footEntry.terminal or false; };".to_string(),
        "  yzxterm = { present = builtins.hasAttr yzxtermKey entries; name = yzxtermEntry.name or \"\"; exec = yzxtermEntry.exec or \"\"; terminal = yzxtermEntry.terminal or false; };".to_string(),
        "  rio = { present = builtins.hasAttr rioKey entries; name = rioEntry.name or \"\"; exec = rioEntry.exec or \"\"; terminal = rioEntry.terminal or false; };".to_string(),
        "  wezterm = { present = builtins.hasAttr weztermKey entries; name = weztermEntry.name or \"\"; exec = weztermEntry.exec or \"\"; terminal = weztermEntry.terminal or false; };".to_string(),
        "}".to_string(),
    ]);
    lines.join("\n")
}

fn validate_home_manager_darwin_without_desktop_entry_option(
    repo_root: &Path,
) -> Result<(), String> {
    let expr = build_home_manager_darwin_without_desktop_entry_option_expr(repo_root);
    let _ = run_nix_eval(repo_root, &expr)?;
    Ok(())
}

fn build_home_manager_darwin_without_desktop_entry_option_expr(repo_root: &Path) -> String {
    let module_path =
        escape_nix_string(&repo_root.join(MODULE_RELATIVE_PATH).display().to_string());
    let mut lines = vec![
        "let".to_string(),
        "  pkgs = import <nixpkgs> { system = \"aarch64-darwin\"; };".to_string(),
        "  lib = pkgs.lib;".to_string(),
        "  eval = lib.evalModules {".to_string(),
        "    specialArgs = { inherit pkgs; nixgl = null; };".to_string(),
        "    modules = [".to_string(),
        format!("      (builtins.toPath \"{}\")", module_path),
    ];
    lines.extend(standalone_home_manager_eval_fixture_module(false, true));
    lines.extend([
        "    ];".to_string(),
        "  };".to_string(),
        "in { enabled = eval.config.programs.yazelix.enable; }".to_string(),
    ]);
    lines.join("\n")
}

#[derive(Debug)]
struct ConfigStateFixture {
    fixture_root: PathBuf,
    runtime_root: PathBuf,
    runtime_root_alt: PathBuf,
    main_config_path: PathBuf,
    managed_config_path: PathBuf,
    state_path: PathBuf,
}

fn setup_config_state_fixture(repo_root: &Path) -> Result<ConfigStateFixture, String> {
    let fixture_root = create_unique_temp_dir("yazelix_config_contract")?;
    let runtime_root = fixture_root.join("runtime");
    let runtime_root_alt = fixture_root.join("runtime_alt");
    let config_root = fixture_root.join("config");
    let home_root = fixture_root.join("home");
    fs::create_dir_all(&runtime_root)
        .map_err(|error| format!("Failed to create {}: {}", runtime_root.display(), error))?;
    fs::create_dir_all(&runtime_root_alt)
        .map_err(|error| format!("Failed to create {}: {}", runtime_root_alt.display(), error))?;
    fs::create_dir_all(&config_root)
        .map_err(|error| format!("Failed to create {}: {}", config_root.display(), error))?;
    fs::create_dir_all(&home_root)
        .map_err(|error| format!("Failed to create {}: {}", home_root.display(), error))?;

    for relative_path in [
        TOML_TOOLING_CONFIG_RELATIVE_PATH,
        MAIN_TEMPLATE_RELATIVE_PATH,
        MAIN_CONTRACT_RELATIVE_PATH,
    ] {
        copy_fixture_file(repo_root, &runtime_root, relative_path)?;
        copy_fixture_file(repo_root, &runtime_root_alt, relative_path)?;
    }

    let main_config_path = config_root.join("settings.jsonc");
    fs::copy(
        repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH),
        &main_config_path,
    )
    .map_err(|error| {
        format!(
            "Failed to copy {} into fixture config: {}",
            repo_root.join(MAIN_TEMPLATE_RELATIVE_PATH).display(),
            error
        )
    })?;

    Ok(ConfigStateFixture {
        fixture_root: fixture_root.clone(),
        runtime_root,
        runtime_root_alt,
        main_config_path: main_config_path.clone(),
        managed_config_path: main_config_path,
        state_path: home_root
            .join(".local")
            .join("share")
            .join("yazelix")
            .join("state")
            .join("rebuild_hash"),
    })
}

fn copy_fixture_file(
    source_root: &Path,
    target_root: &Path,
    relative_path: &str,
) -> Result<(), String> {
    let source = source_root.join(relative_path);
    let target = target_root.join(relative_path);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
    }
    fs::copy(&source, &target).map_err(|error| {
        format!(
            "Failed to copy {} to {}: {}",
            source.display(),
            target.display(),
            error
        )
    })?;
    Ok(())
}

fn compute_fixture_state(
    fixture: &ConfigStateFixture,
    runtime_root: &Path,
) -> Result<ConfigStateData, String> {
    compute_config_state(&ComputeConfigStateRequest {
        config_path: fixture.main_config_path.clone(),
        default_config_path: runtime_root.join(MAIN_TEMPLATE_RELATIVE_PATH),
        contract_path: runtime_root.join(MAIN_CONTRACT_RELATIVE_PATH),
        runtime_dir: runtime_root.to_path_buf(),
        state_path: fixture.state_path.clone(),
    })
    .map_err(|error| error.message())
}

fn record_fixture_state(
    fixture: &ConfigStateFixture,
    state: &ConfigStateData,
) -> Result<(), String> {
    record_config_state(&RecordConfigStateRequest {
        config_file: state.config_file.clone(),
        managed_config_path: fixture.managed_config_path.clone(),
        state_path: fixture.state_path.clone(),
        config_hash: state.config_hash.clone(),
        runtime_hash: state.runtime_hash.clone(),
    })
    .map_err(|error| error.message())?;
    Ok(())
}

fn mutate_fixture_config(
    config_path: &Path,
    field_path: &str,
    value: TomlValue,
) -> Result<(), String> {
    let mut table = read_config_table(config_path, "read_generated_state_fixture_config")
        .map_err(|error| error.message().to_string())?;
    set_nested_toml_value(&mut table, &split_field_path(field_path), value);
    fs::write(
        config_path,
        render_settings_jsonc_value(&toml_to_json(&TomlValue::Table(table)))
            .map_err(|error| error.message().to_string())?,
    )
    .map_err(|error| format!("Failed to write {}: {}", config_path.display(), error))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test lane: maintainer
    // Regression: startup lost `YAZELIX_SESSION_CONFIG_PATH` and `YAZELIX_STATUS_BAR_CACHE_PATH` before the Zellij handoff.
    #[test]
    fn startup_snapshot_env_contract_requires_rust_handoff_tokens() {
        let missing_status_cache = r#"
fn prepare_rust_startup() {
    write_session_config_snapshot_for_launch();
    "YAZELIX_SESSION_CONFIG_PATH";
    command_status_with_overrides();
}
"#;

        let errors = validate_startup_snapshot_env_contract_content(
            "rust_core/yazelix_core/src/launch_commands/enter.rs",
            missing_status_cache,
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("YAZELIX_STATUS_BAR_CACHE_PATH"));

        let complete = format!("{missing_status_cache}\n\"YAZELIX_STATUS_BAR_CACHE_PATH\";\n");
        assert!(
            validate_startup_snapshot_env_contract_content(
                "rust_core/yazelix_core/src/launch_commands/enter.rs",
                &complete
            )
            .is_empty()
        );
    }

    // Test lane: maintainer
    // Defends: every main config contract field declares a closed runtime apply mode before config UI and doctor consume it.
    #[test]
    fn main_contract_apply_mode_validator_rejects_missing_or_unknown_modes() {
        let mut field = TomlTable::new();
        let mut errors = Vec::new();
        validate_main_contract_apply_mode("core.debug_mode", &field, &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("missing apply_mode"));

        field.insert(
            "apply_mode".to_string(),
            TomlValue::String("restart_later".to_string()),
        );
        errors.clear();
        validate_main_contract_apply_mode("core.debug_mode", &field, &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("unsupported apply_mode"));

        field.insert(
            "apply_mode".to_string(),
            TomlValue::String("tab_session_restart".to_string()),
        );
        errors.clear();
        validate_main_contract_apply_mode("core.debug_mode", &field, &mut errors);
        assert!(errors.is_empty());
    }

    // Test lane: maintainer
    // Defends: semantic Zellij keybinding defaults stay in one-to-one parity between the config contract and action registry.
    #[test]
    fn zellij_keybinding_registry_validator_reports_extra_missing_and_mismatched_defaults() {
        let mut contract = TomlTable::new();
        let mut fields = TomlTable::new();
        let mut keybindings = TomlTable::new();
        let mut defaults = TomlTable::new();
        defaults.insert(
            "popup".to_string(),
            TomlValue::Array(vec![TomlValue::String("Alt x".to_string())]),
        );
        defaults.insert(
            "not_in_registry".to_string(),
            TomlValue::Array(vec![TomlValue::String("Alt z".to_string())]),
        );
        keybindings.insert("default".to_string(), TomlValue::Table(defaults));
        fields.insert(
            "zellij.keybindings".to_string(),
            TomlValue::Table(keybindings),
        );
        contract.insert("fields".to_string(), TomlValue::Table(fields));

        let contract_defaults =
            load_contract_keybinding_defaults(&contract, "zellij.keybindings").unwrap();
        assert_eq!(
            contract_defaults.get("popup"),
            Some(&vec!["Alt x".to_string()])
        );

        let repo = tempfile::tempdir().unwrap();
        let metadata_dir = repo.path().join("config_metadata");
        fs::create_dir_all(&metadata_dir).unwrap();
        fs::write(
            metadata_dir.join("main_config_contract.toml"),
            toml::to_string(&contract).unwrap(),
        )
        .unwrap();

        let errors = validate_zellij_keybinding_registry_defaults(repo.path()).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing action `open_workspace_terminal`"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("default `not_in_registry` is not present"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("default mismatch for `popup`"))
        );
    }

    // Test lane: maintainer
    // Defends: semantic Yazi keybinding defaults stay in one-to-one parity between the config contract and action registry.
    #[test]
    fn yazi_keybinding_registry_validator_reports_extra_missing_and_mismatched_defaults() {
        let mut contract = TomlTable::new();
        let mut fields = TomlTable::new();
        let mut keybindings = TomlTable::new();
        let mut defaults = TomlTable::new();
        defaults.insert(
            "open_zoxide_in_editor".to_string(),
            TomlValue::Array(vec![TomlValue::String("<A-x>".to_string())]),
        );
        defaults.insert(
            "not_in_registry".to_string(),
            TomlValue::Array(vec![TomlValue::String("<A-z>".to_string())]),
        );
        keybindings.insert("default".to_string(), TomlValue::Table(defaults));
        fields.insert(
            "yazi.keybindings".to_string(),
            TomlValue::Table(keybindings),
        );
        contract.insert("fields".to_string(), TomlValue::Table(fields));

        let contract_defaults =
            load_contract_keybinding_defaults(&contract, "yazi.keybindings").unwrap();
        assert_eq!(
            contract_defaults.get("open_zoxide_in_editor"),
            Some(&vec!["<A-x>".to_string()])
        );

        let repo = tempfile::tempdir().unwrap();
        let metadata_dir = repo.path().join("config_metadata");
        fs::create_dir_all(&metadata_dir).unwrap();
        fs::write(
            metadata_dir.join("main_config_contract.toml"),
            toml::to_string(&contract).unwrap(),
        )
        .unwrap();

        let errors = validate_yazi_keybinding_registry_defaults(repo.path()).unwrap();
        assert!(
            errors
                .iter()
                .any(|error| error.contains("missing action `open_directory_as_workspace_pane`"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("default `not_in_registry` is not present"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("default mismatch for `open_zoxide_in_editor`"))
        );
    }
}
