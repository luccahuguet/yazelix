//! `yzx cursors` commands for the shared Yazelix cursor registry.
// Test lane: default

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{config_dir_from_env, config_override_from_env, runtime_dir_from_env};
use crate::ghostty_cursor_registry::{
    CursorDefinition, CursorFamily, CursorRegistry, SplitDivider, SplitTransition,
    load_cursor_config,
};
use crate::require_runtime_component_enabled;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;

const GHOSTTY_INCLUDE_FILE_NAME: &str = "ghostty.conf";

pub fn run_yzx_cursors(args: &[String]) -> Result<i32, CoreError> {
    match args {
        [] => run_cursors_report(),
        [single] if matches!(single.as_str(), "-h" | "--help" | "help") => {
            print_cursors_help();
            Ok(0)
        }
        [target] if target == "ghostty" => {
            print_cursors_ghostty_help();
            Ok(0)
        }
        [target, single]
            if target == "ghostty" && matches!(single.as_str(), "-h" | "--help" | "help") =>
        {
            print_cursors_ghostty_help();
            Ok(0)
        }
        [target, action] if target == "ghostty" && action == "setup" => setup_ghostty_cursors(),
        [target, action, single]
            if target == "ghostty"
                && action == "setup"
                && matches!(single.as_str(), "-h" | "--help" | "help") =>
        {
            print_cursors_ghostty_help();
            Ok(0)
        }
        [unknown, ..] if unknown.starts_with('-') => Err(CoreError::usage(format!(
            "Unknown argument for yzx cursors: {unknown}. Try `yzx cursors --help`."
        ))),
        _ => Err(CoreError::usage(format!(
            "Unknown yzx cursors command: {}. Try `yzx cursors --help`.",
            args.join(" ")
        ))),
    }
}

fn run_cursors_report() -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    require_runtime_component_enabled(&runtime_dir, "cursors", "yzx cursors")?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let active_paths =
        resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;
    let registry = load_cursor_config(&active_paths.user_cursor_config)?;

    print_cursor_report(
        &active_paths.user_cursor_config.display().to_string(),
        &registry,
    );
    Ok(0)
}

fn print_cursors_help() {
    println!("Inspect Yazelix cursor presets and resolved colors");
    println!();
    println!("Usage:");
    println!("  yzx cursors");
    println!("  yzx cursors ghostty setup");
}

fn print_cursors_ghostty_help() {
    println!("Generate the Ghostty include for Yazelix cursors");
    println!();
    println!("Usage:");
    println!("  yzx cursors ghostty setup");
    println!();
    println!("This writes ~/.config/yazelix/ghostty.conf from the active Yazelix cursor settings.");
}

fn print_cursor_report(config_path: &str, registry: &CursorRegistry) {
    println!("Yazelix cursors");
    println!("Config: {config_path}");
    println!("Trail: {}", trail_summary(registry));
    println!("Trail effect: {}", registry.settings.trail_effect);
    println!("Mode effect: {}", registry.settings.mode_effect);
    println!("Glow: {}", registry.settings.glow);
    println!("Duration: {:.2}", registry.settings.duration);
    println!();
    println!("Enabled cursors");
    for definition in registry.enabled_definitions() {
        println!("- {}", cursor_definition_summary(definition));
    }
}

fn trail_summary(registry: &CursorRegistry) -> String {
    match registry.settings.trail.as_str() {
        "none" => "none (disabled)".to_string(),
        "random" => format!(
            "random from {} enabled cursors",
            registry.enabled_cursors.len()
        ),
        selected => selected.to_string(),
    }
}

fn cursor_definition_summary(definition: &CursorDefinition) -> String {
    if definition.family == CursorFamily::Mono {
        format!(
            "{}: mono base={} accent={} cursor={}",
            definition.name,
            definition.colors[0].hex,
            definition.colors[1].hex,
            definition.cursor_color.hex
        )
    } else if definition.family == CursorFamily::Split {
        let divider = definition
            .divider
            .expect("validated split cursor definitions always have a divider");
        let transition = definition
            .transition
            .expect("validated split cursor definitions always have a transition");
        let (first_label, second_label) = split_color_labels(divider);
        format!(
            "{}: split divider={} transition={} {}={} {}={} cursor={}",
            definition.name,
            split_divider_label(divider),
            split_transition_label(transition),
            first_label,
            definition.colors[0].hex,
            second_label,
            definition.colors[1].hex,
            definition.cursor_color.hex
        )
    } else {
        format!(
            "{}: unsupported family={} cursor={}",
            definition.name,
            definition.family_name(),
            definition.cursor_color.hex
        )
    }
}

fn split_color_labels(divider: SplitDivider) -> (&'static str, &'static str) {
    match divider {
        SplitDivider::Vertical => ("left", "right"),
        SplitDivider::Horizontal => ("top", "bottom"),
    }
}

fn split_divider_label(divider: SplitDivider) -> &'static str {
    match divider {
        SplitDivider::Vertical => "vertical",
        SplitDivider::Horizontal => "horizontal",
    }
}

fn split_transition_label(transition: SplitTransition) -> &'static str {
    match transition {
        SplitTransition::Soft => "soft",
        SplitTransition::Hard => "hard",
    }
}

fn setup_ghostty_cursors() -> Result<i32, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    require_runtime_component_enabled(&runtime_dir, "cursors", "yzx cursors ghostty setup")?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let active_paths =
        resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;
    let cursor_config_dir = active_paths
        .user_cursor_config
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_cursor_config_path",
                format!(
                    "Yazelix cursor settings path has no parent directory: {}.",
                    active_paths.user_cursor_config.display()
                ),
                "Use a normal Yazelix config directory, then retry.",
                json!({ "path": active_paths.user_cursor_config.display().to_string() }),
            )
        })?;
    let yzc = bundled_yzc_path(&runtime_dir)?;
    let share_dir = runtime_ghostty_cursor_share_dir(&runtime_dir)?;

    if !active_paths.user_cursor_config.exists() {
        run_yzc_command(
            &yzc,
            &cursor_config_dir,
            &share_dir,
            &["init"],
            "initializing cursor settings",
        )?;
    }
    run_yzc_command(
        &yzc,
        &cursor_config_dir,
        &share_dir,
        &["generate", "ghostty"],
        "generating the Ghostty include",
    )?;

    let include_path = cursor_config_dir.join(GHOSTTY_INCLUDE_FILE_NAME);
    if !include_path.exists() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_generated_ghostty_cursor_include",
            format!(
                "The bundled cursor generator did not write {}.",
                include_path.display()
            ),
            "Run `yzx cursors ghostty setup` again. If it still fails, report the active cursor settings.",
            json!({ "path": include_path.display().to_string() }),
        ));
    }

    println!("Ghostty cursor include generated:");
    println!("  {}", include_path.display());
    println!();
    println!("Add this line to your Ghostty config:");
    println!("  config-file = {}", include_path.display());
    Ok(0)
}

fn bundled_yzc_path(runtime_dir: &Path) -> Result<PathBuf, CoreError> {
    let path = runtime_dir.join("libexec").join("yzc");
    if path.is_file() {
        return Ok(path);
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_bundled_yzc",
        format!(
            "The Yazelix runtime does not include the bundled cursor helper at {}.",
            path.display()
        ),
        "Reinstall Yazelix with the cursors component enabled, then retry.",
        json!({ "path": path.display().to_string() }),
    ))
}

fn runtime_ghostty_cursor_share_dir(runtime_dir: &Path) -> Result<PathBuf, CoreError> {
    let share_dir = runtime_dir
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty");
    let shader_root = share_dir.join("shaders");
    if shader_root.is_dir() {
        return Ok(share_dir);
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "missing_runtime_cursor_shaders",
        format!(
            "The Yazelix runtime does not include Ghostty-compatible cursor shaders at {}.",
            shader_root.display()
        ),
        "Reinstall Yazelix with the cursors component enabled, then retry.",
        json!({ "path": shader_root.display().to_string() }),
    ))
}

fn run_yzc_command(
    yzc: &Path,
    cursor_config_dir: &Path,
    share_dir: &Path,
    args: &[&str],
    action: &str,
) -> Result<(), CoreError> {
    let output = Command::new(yzc)
        .arg("--config-dir")
        .arg(cursor_config_dir)
        .arg("--share-dir")
        .arg(share_dir)
        .args(args)
        .output()
        .map_err(|source| {
            CoreError::io(
                "run_bundled_yzc",
                "Could not run the bundled Yazelix cursor helper",
                "Reinstall Yazelix with the cursors component enabled, then retry.",
                yzc.to_string_lossy(),
                source,
            )
        })?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "bundled_yzc_failed",
        format!("Bundled Yazelix cursor helper failed while {action}."),
        "Fix ~/.config/yazelix/cursors.toml or move it aside and retry `yzx cursors ghostty setup`.",
        json!({
            "command": args,
            "status": output.status.code(),
            "stdout": stdout,
            "stderr": stderr,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Defends: the Ghostty setup wrapper fails fast when the packaged cursor helper is absent instead of searching PATH.
    #[test]
    fn bundled_yzc_path_requires_runtime_private_helper() {
        let temp = tempdir().unwrap();
        let error = bundled_yzc_path(temp.path()).unwrap_err();
        assert_eq!(error.code(), "missing_bundled_yzc");
    }

    // Defends: the setup wrapper passes Ghostty-compatible shader assets as a share directory, not the shader directory itself.
    #[test]
    fn runtime_ghostty_cursor_share_dir_points_at_parent_of_shaders() {
        let temp = tempdir().unwrap();
        let share_dir = temp.path().join("configs/terminal_emulators/ghostty");
        fs::create_dir_all(share_dir.join("shaders")).unwrap();
        assert_eq!(
            runtime_ghostty_cursor_share_dir(temp.path()).unwrap(),
            share_dir
        );
    }
}
