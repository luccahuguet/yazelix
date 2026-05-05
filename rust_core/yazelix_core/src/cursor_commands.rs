//! `yzx cursors` inspection command for the Ghostty cursor registry in settings.jsonc.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::CoreError;
use crate::control_plane::{config_dir_from_env, config_override_from_env, runtime_dir_from_env};
use crate::ghostty_cursor_registry::{
    CursorDefinition, CursorFamily, CursorRegistry, SplitDivider, SplitTransition,
};

pub fn run_yzx_cursors(args: &[String]) -> Result<i32, CoreError> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help" | "help"))
    {
        print_cursors_help();
        return Ok(0);
    }
    if let Some(unknown) = args.iter().find(|arg| arg.starts_with('-')) {
        return Err(CoreError::usage(format!(
            "Unknown argument for yzx cursors: {unknown}. Try `yzx cursors --help`."
        )));
    }
    if !args.is_empty() {
        return Err(CoreError::usage(
            "yzx cursors does not accept subcommands. Try `yzx cursors --help`.".to_string(),
        ));
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let config_override = config_override_from_env();
    let active_paths =
        resolve_active_config_paths(&runtime_dir, &config_dir, config_override.as_deref())?;
    let registry = CursorRegistry::load(&active_paths.user_cursor_config)?;

    print_cursor_report(
        &active_paths.user_cursor_config.display().to_string(),
        &registry,
    );
    Ok(0)
}

fn print_cursors_help() {
    println!("Inspect Ghostty cursor presets and resolved colors");
    println!();
    println!("Usage:");
    println!("  yzx cursors");
}

fn print_cursor_report(config_path: &str, registry: &CursorRegistry) {
    println!("Ghostty cursors");
    println!("Config: {config_path}");
    println!("Trail: {}", trail_summary(registry));
    println!("Trail effect: {}", registry.settings.trail_effect);
    println!("Mode effect: {}", registry.settings.mode_effect);
    println!("Glow: {}", registry.settings.glow);
    println!("Duration: {:.2}", registry.settings.duration);
    println!(
        "Kitty cursor fallback: {}",
        if registry.settings.kitty_enable_cursor {
            "enabled"
        } else {
            "disabled"
        }
    );
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
    match definition.family {
        CursorFamily::Mono => format!(
            "{}: mono base={} accent={} cursor={}",
            definition.name,
            definition.colors[0].hex,
            definition.colors[1].hex,
            definition.cursor_color.hex
        ),
        CursorFamily::Split => {
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
        }
        CursorFamily::CuratedTemplate => format!(
            "{}: curated_template template={} cursor={}",
            definition.name,
            definition.template.as_deref().unwrap_or("unknown"),
            definition.cursor_color.hex
        ),
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
