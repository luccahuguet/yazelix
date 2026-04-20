//! Typed Yazi render-plan data for Nushell TOML/Lua renderers.
//!
//! **Rust dependency gate:** in-house only (std + existing workspace deps). No new crates.

use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Dark flavor names bundled/expected by Yazelix (must stay aligned with `yazi_config_merger.nu`).
const YAZI_THEMES_DARK: &[&str] = &[
    "catppuccin-mocha",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "dracula",
    "gruvbox-dark",
    "tokyo-night",
    "kanagawa",
    "kanagawa-dragon",
    "rose-pine",
    "rose-pine-moon",
    "flexoki-dark",
    "bluloco-dark",
    "ayu-dark",
    "everforest-medium",
    "ashen",
    "neon",
    "nord",
    "synthwave84",
    "monokai",
];

/// Light flavor names bundled/expected by Yazelix (must stay aligned with `yazi_config_merger.nu`).
const YAZI_THEMES_LIGHT: &[&str] = &[
    "catppuccin-latte",
    "kanagawa-lotus",
    "rose-pine-dawn",
    "flexoki-light",
    "bluloco-light",
];

const CORE_PLUGINS: &[&str] = &["sidebar-status", "auto-layout", "sidebar-state"];

const SORT_BY_ALLOWED: &[&str] = &[
    "alphabetical",
    "natural",
    "modified",
    "created",
    "size",
];

fn default_yazi_theme() -> String {
    "default".into()
}

fn default_yazi_sort_by() -> String {
    "alphabetical".into()
}

fn default_yazi_plugins() -> Vec<String> {
    vec!["git".into(), "starship".into()]
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
    match theme_config {
        "random-dark" => YAZI_THEMES_DARK
            .get(pick_index(YAZI_THEMES_DARK.len()))
            .map(|s| (*s).to_string())
            .unwrap_or_else(|| "default".into()),
        "random-light" => YAZI_THEMES_LIGHT
            .get(pick_index(YAZI_THEMES_LIGHT.len()))
            .map(|s| (*s).to_string())
            .unwrap_or_else(|| "default".into()),
        _ => theme_config.to_string(),
    }
}

fn validate_sort_by(sort_by: &str) -> Result<(), CoreError> {
    if SORT_BY_ALLOWED.contains(&sort_by) {
        Ok(())
    } else {
        Err(CoreError::classified(
            ErrorClass::Config,
            "invalid_yazi_sort_by",
            format!("yazi_sort_by must be one of {:?} (got {sort_by:?})", SORT_BY_ALLOWED),
            "Set [yazi].sort_by to a documented value.",
            serde_json::json!({ "field": "yazi.sort_by" }),
        ))
    }
}

fn merged_plugin_load_order(user_plugins: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in CORE_PLUGINS
        .iter()
        .map(|s| (*s).to_string())
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
pub struct YaziRenderPlanRequest {
    #[serde(default = "default_yazi_theme")]
    pub yazi_theme: String,
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
    let resolved_theme = resolve_yazi_theme(&request.yazi_theme);
    let theme_flavor = theme_flavor_plan(&resolved_theme);
    let load_order = merged_plugin_load_order(&yazi_plugins);
    let core_plugins: Vec<String> = CORE_PLUGINS.iter().map(|s| (*s).to_string()).collect();

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
            yazi_sort_by: "alphabetical".into(),
            yazi_plugins: None,
        }
    }

    // Defends: invalid yazi.sort_by values fail as structured config errors instead of slipping into generated TOML.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
    #[test]
    fn rejects_invalid_sort_by() {
        let mut req = sample_request();
        req.yazi_sort_by = "not-a-sort".into();
        assert!(compute_yazi_render_plan(&req).is_err());
    }

    // Defends: git fetcher stripping tracks whether the git plugin is enabled in the normalized plugin list.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
    #[test]
    fn default_theme_uses_no_flavor_block() {
        let plan = compute_yazi_render_plan(&sample_request()).unwrap();
        assert_eq!(plan.resolved_theme, "default");
        assert_eq!(plan.theme_flavor, ThemeFlavorPlan::None);
    }

    // Defends: concrete non-default themes map to a uniform dark/light flavor block for theme.toml.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
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

    // Defends: random-dark resolves to one of the maintained dark palette entries (same list as Nushell).
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=2 total=8/10
    #[test]
    fn random_dark_resolves_into_dark_palette() {
        let mut req = sample_request();
        req.yazi_theme = "random-dark".into();
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert!(YAZI_THEMES_DARK.contains(&plan.resolved_theme.as_str()));
    }

    // Defends: random-light resolves to one of the maintained light palette entries (same list as Nushell).
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=2 total=8/10
    #[test]
    fn random_light_resolves_into_light_palette() {
        let mut req = sample_request();
        req.yazi_theme = "random-light".into();
        let plan = compute_yazi_render_plan(&req).unwrap();
        assert!(YAZI_THEMES_LIGHT.contains(&plan.resolved_theme.as_str()));
    }

    // Defends: init.lua load order prepends core plugins and dedupes user entries in first-wins order.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
    #[test]
    fn init_load_order_merges_core_then_user_deduped() {
        let mut req = sample_request();
        req.yazi_plugins = Some(vec!["git".into(), "sidebar-status".into(), "starship".into()]);
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
