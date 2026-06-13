//! Typed Yazi render-plan data for generated Yazi TOML/Lua config.
//!
//! **Rust dependency gate:** in-house only (std + existing workspace deps). No new crates.
//!
//! Machine lists and validation enums are loaded from `config_metadata/yazi_render_plan.toml`
//! (embedded at compile time). Shared `sort_by` / default plugin defaults are parity-checked against
//! `config_metadata/main_config_contract.toml` in integration tests.

use crate::appearance_mode::{APPEARANCE_MODE_DARK, YAZI_THEME_LIGHT, appearance_default_theme};
use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct YaziRenderPlanMetadata {
    sort_by_allowed: Vec<String>,
    default_plugins: Vec<String>,
    themes_dark: Vec<String>,
    themes_light: Vec<String>,
    core_plugins: Vec<String>,
}

static YAZI_RENDER_PLAN_METADATA: OnceLock<YaziRenderPlanMetadata> = OnceLock::new();

fn yazi_render_plan_metadata() -> &'static YaziRenderPlanMetadata {
    YAZI_RENDER_PLAN_METADATA.get_or_init(|| {
        const RAW: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../config_metadata/yazi_render_plan.toml"
        ));
        toml::from_str(RAW).expect("embedded config_metadata/yazi_render_plan.toml must parse")
    })
}

fn default_yazi_theme() -> String {
    "default".into()
}

fn default_yazi_sort_by() -> String {
    "alphabetical".into()
}

fn default_yazi_plugins() -> Vec<String> {
    yazi_render_plan_metadata().default_plugins.clone()
}

fn default_appearance_mode() -> String {
    APPEARANCE_MODE_DARK.into()
}

fn pick_index(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| (d.as_nanos() as usize) % len)
        .unwrap_or(0)
}

fn resolve_yazi_theme(theme_config: &str) -> String {
    let meta = yazi_render_plan_metadata();
    match theme_config {
        "random-dark" => meta
            .themes_dark
            .get(pick_index(meta.themes_dark.len()))
            .cloned()
            .unwrap_or_else(|| "default".into()),
        "random-light" => meta
            .themes_light
            .get(pick_index(meta.themes_light.len()))
            .cloned()
            .unwrap_or_else(|| "default".into()),
        _ => theme_config.to_string(),
    }
}

fn validate_sort_by(sort_by: &str) -> Result<(), CoreError> {
    let allowed = &yazi_render_plan_metadata().sort_by_allowed;
    if allowed.iter().any(|v| v == sort_by) {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_yazi_sort_by",
            format!("yazi_sort_by must be one of {allowed:?} (got {sort_by:?})"),
            "Set [yazi].sort_by to a documented value.",
            serde_json::json!({ "field": "yazi.sort_by" }),
        ))
    }
}

fn merged_plugin_load_order(user_plugins: &[String]) -> Vec<String> {
    let meta = yazi_render_plan_metadata();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in meta
        .core_plugins
        .iter()
        .cloned()
        .chain(user_plugins.iter().cloned())
    {
        if seen.insert(p.clone()) {
            out.push(p);
        }
    }
    out
}

fn theme_flavor_plan(resolved_theme: &str) -> ThemeFlavorPlan {
    if resolved_theme == "default" || resolved_theme == "random" {
        ThemeFlavorPlan::None
    } else {
        ThemeFlavorPlan::Uniform {
            flavor: resolved_theme.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YaziRenderPlanRequest {
    #[serde(default = "default_yazi_theme")]
    pub yazi_theme: String,
    #[serde(default = "default_appearance_mode")]
    pub appearance_mode: String,
    #[serde(default = "default_yazi_sort_by")]
    pub yazi_sort_by: String,
    #[serde(default)]
    pub yazi_plugins: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ThemeFlavorPlan {
    None,
    Uniform { flavor: String },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InitLuaPlan {
    pub core_plugins: Vec<String>,
    pub load_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct YaziRenderPlanData {
    pub resolved_theme: String,
    pub sort_by: String,
    pub yazi_plugins: Vec<String>,
    pub git_plugin_enabled: bool,
    pub theme_flavor: ThemeFlavorPlan,
    pub init_lua: InitLuaPlan,
}

pub fn compute_yazi_render_plan(
    request: &YaziRenderPlanRequest,
) -> Result<YaziRenderPlanData, CoreError> {
    validate_sort_by(&request.yazi_sort_by)?;

    let yazi_plugins = request
        .yazi_plugins
        .clone()
        .unwrap_or_else(default_yazi_plugins);
    let git_plugin_enabled = yazi_plugins.iter().any(|p| p == "git");
    let theme_config = appearance_default_theme(
        &request.yazi_theme,
        YAZI_THEME_LIGHT,
        &request.appearance_mode,
    );
    let resolved_theme = resolve_yazi_theme(&theme_config);
    let theme_flavor = theme_flavor_plan(&resolved_theme);
    let load_order = merged_plugin_load_order(&yazi_plugins);
    let core_plugins = yazi_render_plan_metadata().core_plugins.clone();

    Ok(YaziRenderPlanData {
        resolved_theme,
        sort_by: request.yazi_sort_by.clone(),
        yazi_plugins,
        git_plugin_enabled,
        theme_flavor,
        init_lua: InitLuaPlan {
            core_plugins,
            load_order,
        },
    })
}

// Test lane: maintainer

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> YaziRenderPlanRequest {
        YaziRenderPlanRequest {
            yazi_theme: "default".into(),
            appearance_mode: APPEARANCE_MODE_DARK.into(),
            yazi_sort_by: "alphabetical".into(),
            yazi_plugins: None,
        }
    }

    // Defends: the embedded yazi_render_plan.toml metadata schema rejects dead fields instead of letting metadata drift.
    #[test]
    fn metadata_rejects_unknown_fields() {
        let err = toml::from_str::<YaziRenderPlanMetadata>(
            r#"
sort_by_allowed = ["alphabetical"]
default_plugins = ["git"]
themes_dark = ["dracula"]
themes_light = ["catppuccin-latte"]
core_plugins = ["sidebar-status"]
stale_metadata_field = true
"#,
        )
        .unwrap_err();

        assert!(err.to_string().contains("unknown field"));
        assert!(err.to_string().contains("stale_metadata_field"));
    }

    // Defends: invalid yazi.sort_by values fail as structured config errors instead of slipping into generated TOML.
    #[test]
    fn rejects_invalid_sort_by() {
        let mut req = sample_request();
        req.yazi_sort_by = "not-a-sort".into();
        assert!(compute_yazi_render_plan(&req).is_err());
    }

    // Defends: git fetcher stripping tracks whether the git plugin is enabled in the normalized plugin list.
    #[test]
    fn git_plugin_enabled_follows_plugin_list() {
        let mut req = sample_request();
        req.yazi_plugins = Some(vec!["starship".into()]);
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert!(!plan.git_plugin_enabled);

        req.yazi_plugins = Some(vec!["git".into()]);
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert!(plan.git_plugin_enabled);
    }

    // Defends: default theme keeps theme.toml flavor empty like the historical Nushell merger.
    #[test]
    fn default_theme_uses_no_flavor_block() {
        let plan = compute_yazi_render_plan(&sample_request()).unwrap();
        assert_eq!(plan.resolved_theme, "default");
        assert_eq!(plan.theme_flavor, ThemeFlavorPlan::None);
    }

    // Defends: light appearance changes only the implicit default Yazi theme, preserving explicit user choices.
    #[test]
    fn light_appearance_changes_default_theme_only() {
        let mut req = sample_request();
        req.appearance_mode = "light".into();
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert_eq!(plan.resolved_theme, "catppuccin-latte");
        assert_eq!(
            plan.theme_flavor,
            ThemeFlavorPlan::Uniform {
                flavor: "catppuccin-latte".into()
            }
        );

        req.yazi_theme = "dracula".into();
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert_eq!(plan.resolved_theme, "dracula");
    }

    // Defends: concrete non-default themes map to a uniform dark/light flavor block for theme.toml.
    #[test]
    fn dracula_maps_to_uniform_flavor() {
        let mut req = sample_request();
        req.yazi_theme = "dracula".into();
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert_eq!(
            plan.theme_flavor,
            ThemeFlavorPlan::Uniform {
                flavor: "dracula".into()
            }
        );
    }

    // Defends: random-dark resolves to one of the maintained dark palette entries in yazi_render_plan.toml.
    #[test]
    fn random_dark_resolves_into_dark_palette() {
        let mut req = sample_request();
        req.yazi_theme = "random-dark".into();
        let plan = compute_yazi_render_plan(&req).unwrap();
        let meta = yazi_render_plan_metadata();
        assert!(meta.themes_dark.contains(&plan.resolved_theme));
    }

    // Defends: random-light resolves to one of the maintained light palette entries in yazi_render_plan.toml.
    #[test]
    fn random_light_resolves_into_light_palette() {
        let mut req = sample_request();
        req.yazi_theme = "random-light".into();
        let plan = compute_yazi_render_plan(&req).unwrap();
        let meta = yazi_render_plan_metadata();
        assert!(meta.themes_light.contains(&plan.resolved_theme));
    }

    // Defends: init.lua load order prepends core plugins and dedupes user entries in first-wins order.
    #[test]
    fn init_load_order_merges_core_then_user_deduped() {
        let mut req = sample_request();
        req.yazi_plugins = Some(vec![
            "git".into(),
            "sidebar-status".into(),
            "starship".into(),
        ]);
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert_eq!(
            plan.init_lua.load_order,
            vec![
                "sidebar-status".to_string(),
                "auto-layout".to_string(),
                "sidebar-state".to_string(),
                "git".to_string(),
                "starship".to_string(),
            ]
        );
    }
}
