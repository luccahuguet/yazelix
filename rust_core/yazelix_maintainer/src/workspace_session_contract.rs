//! Maintainer validator for workspace/session integration seams.

use std::fs;
use std::path::Path;
use yazelix_core::ZELLIJ_ACTIONS;
use yazelix_core::workspace_asset_contract::validate_workspace_assets_for_repo;
use yazelix_core::zellij_commands::INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS;

const SEMANTIC_KEYBINDING_BOUND_PIPE_COMMANDS: &[&str] = &[
    "open_workspace_terminal",
    "move_focus_left_or_tab",
    "move_focus_right_or_tab",
    "toggle_editor_sidebar_focus",
    "toggle_editor_right_sidebar_focus",
    "toggle_sidebar",
    "toggle_agent_sidebar",
    "smart_reveal",
    "previous_family",
    "next_family",
];

const REQUIRED_ZELLIJ_SEMANTIC_ACTION_IDS: &[&str] = &[
    "open_workspace_terminal",
    "popup",
    "menu",
    "config",
    "move_focus_left_or_tab",
    "move_focus_right_or_tab",
    "toggle_editor_sidebar_focus",
    "toggle_editor_right_sidebar_focus",
    "toggle_left_sidebar",
    "open_codex_agent_right",
    "smart_reveal",
    "previous_family",
    "next_family",
];

pub fn validate_workspace_session_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    errors.extend(validate_workspace_assets_for_repo(repo_root)?);
    errors.extend(validate_internal_zellij_control_surface(repo_root)?);
    errors.extend(validate_pane_orchestrator_pipe_surface(repo_root)?);
    errors.extend(validate_yazi_workspace_entrypoints(repo_root)?);
    Ok(errors)
}

fn validate_internal_zellij_control_surface(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let control_source = read_repo_file(
        repo_root,
        &["rust_core", "yazelix_core", "src", "bin", "yzx_control.rs"],
    )?;
    for subcommand in INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS {
        if !control_source.contains(&format!("\"{subcommand}\" =>")) {
            errors.push(format!(
                "yzx_control zellij subcommand `{subcommand}` is listed in the central command surface but has no run_zellij match arm"
            ));
        }
    }
    Ok(errors)
}

fn validate_pane_orchestrator_pipe_surface(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    for command in SEMANTIC_KEYBINDING_BOUND_PIPE_COMMANDS {
        if !ZELLIJ_ACTIONS
            .iter()
            .any(|action| action.message_name == *command)
        {
            errors.push(format!(
                "Yazelix semantic Zellij keybindings no longer generate required pane-orchestrator command `{command}`"
            ));
        }
    }
    let default_config = read_repo_file(repo_root, &["settings_default.jsonc"])?;
    for action in REQUIRED_ZELLIJ_SEMANTIC_ACTION_IDS {
        if !default_config.contains(&format!("\"{action}\": [")) {
            errors.push(format!(
                "Default config no longer declares semantic Zellij keybinding action `{action}`"
            ));
        }
    }
    Ok(errors)
}

fn validate_yazi_workspace_entrypoints(repo_root: &Path) -> Result<Vec<String>, String> {
    let yazi_config = read_repo_file(repo_root, &["configs", "yazi", "yazelix_yazi.toml"])?;
    let zoxide_editor_plugin = read_repo_file(
        repo_root,
        &[
            "configs",
            "yazi",
            "plugins",
            "zoxide-editor.yazi",
            "main.lua",
        ],
    )?;
    let mut errors = Vec::new();
    if !yazi_config.contains("yzx_control zellij open-editor ") {
        errors.push(
            "Bundled Yazi config no longer references required workspace entrypoint `yzx_control zellij open-editor`"
                .to_string(),
        );
    }
    if !zoxide_editor_plugin.contains("\"zellij\"")
        || !zoxide_editor_plugin.contains("\"open-editor-cwd\"")
    {
        errors.push(
            "Yazi zoxide-editor plugin no longer invokes required workspace entrypoint `yzx_control zellij open-editor-cwd`"
                .to_string(),
        );
    }
    if !zoxide_editor_plugin.contains(r#"os.getenv("YAZELIX_RUNTIME_DIR")"#)
        || zoxide_editor_plugin.contains("__YAZELIX_RUNTIME_DIR__/libexec/yzx_control")
    {
        errors.push(
            "Yazi zoxide-editor plugin must resolve yzx_control from YAZELIX_RUNTIME_DIR at runtime instead of baking a generated store path"
                .to_string(),
        );
    }
    Ok(errors)
}

fn read_repo_file(repo_root: &Path, relative: &[&str]) -> Result<String, String> {
    let path = relative
        .iter()
        .fold(repo_root.to_path_buf(), |path, segment| path.join(segment));
    fs::read_to_string(&path).map_err(|error| format!("Failed to read {}: {error}", path.display()))
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Defends: the workspace validator checks the central yzx_control zellij command list against the actual dispatcher.
    #[test]
    fn internal_zellij_control_surface_contains_session_inspector() {
        assert!(INTERNAL_ZELLIJ_CONTROL_SUBCOMMANDS.contains(&"inspect-session"));
    }

    // Regression: semantic keybinding validation reads the shared action registry after message metadata moved out of zellij materialization literals.
    #[test]
    fn semantic_keybinding_bound_pipe_commands_are_declared_in_action_registry() {
        for command in SEMANTIC_KEYBINDING_BOUND_PIPE_COMMANDS {
            assert!(
                ZELLIJ_ACTIONS
                    .iter()
                    .any(|action| action.message_name == *command),
                "missing semantic Zellij action registry entry for {command}"
            );
        }
    }

    // Regression: the workspace validator must check bundled Yazi entrypoint files, not incidental Rust test fixture strings.
    #[test]
    fn yazi_workspace_entrypoint_validation_checks_bundled_yazi_config() {
        let temp = tempdir().unwrap();
        let yazi_dir = temp.path().join("configs").join("yazi");
        let plugin_dir = yazi_dir.join("plugins").join("zoxide-editor.yazi");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            yazi_dir.join("yazelix_yazi.toml"),
            r#"[opener]
edit = [{ run = "hx \"$1\"" }]
"#,
        )
        .unwrap();
        fs::write(
            plugin_dir.join("main.lua"),
            r#"local runtime_dir = os.getenv("YAZELIX_RUNTIME_DIR")
Command(runtime_dir .. "/libexec/yzx_control"):arg({ "zellij", "open-editor-cwd", target_dir })"#,
        )
        .unwrap();

        let errors = validate_yazi_workspace_entrypoints(temp.path()).unwrap();

        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Bundled Yazi config"));
    }
}
