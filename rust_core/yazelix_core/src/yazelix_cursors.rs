// Test lane: default
//! Reusable Yazelix cursor registry and Ghostty shader generation.

use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_CURSOR_CONFIG_FILENAME: &str = "yazelix_cursors_default.toml";
pub const STANDALONE_CURSOR_CONFIG_DIR_NAME: &str = "yazelix_cursors";
pub const STANDALONE_CURSOR_SETTINGS_FILENAME: &str = "settings.jsonc";
pub const DEFAULT_GHOSTTY_TRAIL_DURATION: f64 = 1.0;
pub const GHOSTTY_TRAIL_DURATION_MIN: f64 = 0.25;
pub const GHOSTTY_TRAIL_DURATION_MAX: f64 = 4.0;

const SUPPORTED_TRAIL_EFFECTS: &[&str] = &["tail", "warp", "sweep"];
const SUPPORTED_MODE_EFFECTS: &[&str] =
    &["ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle"];
const SUPPORTED_GLOW_LEVELS: &[&str] = &["none", "low", "medium", "high"];
const SUPPORTED_CURATED_TEMPLATES: &[&str] = &["neon"];
const REMOVED_CURSOR_NAMES: &[&str] = &["party"];
const GHOSTTY_CURSOR_EFFECT_TEMPLATES: &[(&str, &str)] = &[
    ("tail", "cursor_tail.glsl"),
    ("warp", "cursor_warp.glsl"),
    ("ripple", "ripple_cursor.glsl"),
    ("rectangle_boom", "rectangle_boom_cursor.glsl"),
    ("sonic_boom", "sonic_boom_cursor.glsl"),
    ("sweep", "cursor_sweep.glsl"),
    ("ripple_rectangle", "ripple_rectangle_cursor.glsl"),
];
const GHOSTTY_CURSOR_MOVEMENT_EFFECTS: &[&str] = &["tail", "warp", "sweep"];

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CursorRegistry {
    pub schema_version: u32,
    pub enabled_cursors: Vec<String>,
    pub settings: CursorSettings,
    pub definitions: BTreeMap<String, CursorDefinition>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CursorSettings {
    pub trail: String,
    pub trail_effect: String,
    pub mode_effect: String,
    pub glow: String,
    pub duration: f64,
    pub kitty_enable_cursor: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CursorDefinition {
    pub name: String,
    pub family: CursorFamily,
    pub colors: Vec<CursorColor>,
    pub divider: Option<SplitDivider>,
    pub transition: Option<SplitTransition>,
    pub template: Option<String>,
    pub cursor_color: CursorColor,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CursorFamily {
    Mono,
    Split,
    CuratedTemplate,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SplitDivider {
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SplitTransition {
    Soft,
    Hard,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CursorColor {
    pub hex: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResolvedCursorRegistryState {
    pub selected_cursor: Option<CursorDefinition>,
    pub trail_disabled: bool,
    pub selected_trail_effect: Option<String>,
    pub selected_mode_effect: Option<String>,
    pub duration: f64,
    pub glow: String,
    pub kitty_enable_cursor: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCursorRegistry {
    schema_version: u32,
    enabled_cursors: Vec<String>,
    settings: RawCursorSettings,
    #[serde(default)]
    cursor: Vec<RawCursorDefinition>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCursorSettings {
    trail: String,
    trail_effect: String,
    mode_effect: String,
    glow: String,
    duration: f64,
    kitty_enable_cursor: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCursorDefinition {
    name: String,
    family: String,
    color: Option<String>,
    accent_color: Option<String>,
    #[serde(default)]
    colors: Vec<String>,
    divider: Option<String>,
    transition: Option<String>,
    template: Option<String>,
    cursor_color: Option<String>,
}

impl CursorRegistry {
    pub fn parse_json_value(path: &Path, cursors: Value) -> Result<Self, CoreError> {
        let parsed = serde_json::from_value::<RawCursorRegistry>(cursors).map_err(|source| {
            CoreError::classified(
                ErrorClass::Config,
                "invalid_cursor_registry_json",
                format!(
                    "Could not parse Yazelix cursor registry JSON in {}.",
                    path.display()
                ),
                "Fix the cursor registry data and retry.",
                json!({
                    "path": path.display().to_string(),
                    "error": source.to_string(),
                }),
            )
        })?;
        CursorRegistry::from_raw(path, parsed)
    }

    pub fn parse_str(path: &Path, raw: &str) -> Result<Self, CoreError> {
        let parsed = toml::from_str::<RawCursorRegistry>(&raw).map_err(|source| {
            CoreError::toml(
                "invalid_cursor_config_toml",
                "Could not parse Yazelix cursor config",
                "Fix the cursor registry data and retry.",
                path.to_string_lossy(),
                source,
            )
        })?;
        CursorRegistry::from_raw(path, parsed)
    }

    pub fn enabled_definitions(&self) -> Vec<&CursorDefinition> {
        self.enabled_cursors
            .iter()
            .filter_map(|name| self.definitions.get(name))
            .collect()
    }

    pub fn is_random_request(&self) -> bool {
        self.settings.trail == "random"
            || self.settings.trail_effect == "random"
            || self.settings.mode_effect == "random"
    }

    pub fn resolve(&self) -> ResolvedCursorRegistryState {
        let entropy = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos() as usize)
            .unwrap_or(0);
        self.resolve_with_entropy(entropy)
    }

    pub fn resolve_with_entropy(&self, entropy: usize) -> ResolvedCursorRegistryState {
        let selected_cursor = match self.settings.trail.as_str() {
            "none" => None,
            "random" => self
                .enabled_cursors
                .get(entropy % self.enabled_cursors.len())
                .and_then(|name| self.definitions.get(name))
                .cloned(),
            name => self.definitions.get(name).cloned(),
        };

        ResolvedCursorRegistryState {
            selected_cursor,
            trail_disabled: self.settings.trail == "none",
            selected_trail_effect: resolve_optional_effect(
                &self.settings.trail_effect,
                SUPPORTED_TRAIL_EFFECTS,
                entropy,
            ),
            selected_mode_effect: resolve_optional_effect(
                &self.settings.mode_effect,
                SUPPORTED_MODE_EFFECTS,
                entropy / 17,
            ),
            duration: self.settings.duration,
            glow: self.settings.glow.clone(),
            kitty_enable_cursor: self.settings.kitty_enable_cursor,
        }
    }

    fn from_raw(path: &Path, raw: RawCursorRegistry) -> Result<Self, CoreError> {
        if raw.schema_version != 1 {
            return Err(invalid_cursor_config(
                path,
                "schema_version",
                format!(
                    "Unsupported cursor config schema_version {}. Expected 1.",
                    raw.schema_version
                ),
            ));
        }

        let mut enabled_seen = BTreeSet::new();
        let mut enabled_cursors = Vec::new();
        for name in raw.enabled_cursors {
            let normalized = validate_cursor_name(path, "enabled_cursors", &name)?;
            if !enabled_seen.insert(normalized.clone()) {
                return Err(invalid_cursor_config(
                    path,
                    "enabled_cursors",
                    format!("Cursor '{normalized}' is listed more than once in enabled_cursors."),
                ));
            }
            enabled_cursors.push(normalized);
        }
        if enabled_cursors.is_empty() {
            return Err(invalid_cursor_config(
                path,
                "enabled_cursors",
                "enabled_cursors must contain at least one cursor name.".to_string(),
            ));
        }

        let settings = validate_settings(path, raw.settings, &enabled_cursors)?;
        let mut definitions = BTreeMap::new();
        for raw_definition in raw.cursor {
            let definition = validate_definition(path, raw_definition)?;
            if definitions
                .insert(definition.name.clone(), definition.clone())
                .is_some()
            {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.name",
                    format!("Cursor '{}' is defined more than once.", definition.name),
                ));
            }
        }

        for enabled in &enabled_cursors {
            if !definitions.contains_key(enabled) {
                return Err(invalid_cursor_config(
                    path,
                    "enabled_cursors",
                    format!(
                        "enabled_cursors references '{enabled}', but no matching [[cursor]] table exists."
                    ),
                ));
            }
        }

        Ok(CursorRegistry {
            schema_version: raw.schema_version,
            enabled_cursors,
            settings,
            definitions,
        })
    }
}

impl CursorDefinition {
    pub fn shader_path(&self) -> String {
        match self.family {
            CursorFamily::CuratedTemplate => format!(
                "./shaders/cursor_trail_{}.glsl",
                self.template.as_deref().unwrap_or(&self.name)
            ),
            CursorFamily::Mono | CursorFamily::Split => {
                format!("./shaders/cursor_trail_{}.glsl", self.name)
            }
        }
    }

    pub fn cursor_color_hex(&self) -> &str {
        &self.cursor_color.hex
    }

    pub fn family_name(&self) -> &'static str {
        self.family.as_str()
    }

    pub fn divider_name(&self) -> Option<&'static str> {
        self.divider.map(|divider| divider.as_str())
    }

    pub fn split_primary_color_hex(&self) -> Option<&str> {
        matches!(self.family, CursorFamily::Split)
            .then(|| self.colors.first().map(|color| color.hex.as_str()))
            .flatten()
    }

    pub fn split_secondary_color_hex(&self) -> Option<&str> {
        matches!(self.family, CursorFamily::Split)
            .then(|| self.colors.get(1).map(|color| color.hex.as_str()))
            .flatten()
    }

    pub fn cursor_color_literal(&self) -> String {
        self.cursor_color.glsl_vec4()
    }
}

impl CursorFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            CursorFamily::Mono => "mono",
            CursorFamily::Split => "split",
            CursorFamily::CuratedTemplate => "curated_template",
        }
    }
}

impl SplitDivider {
    pub fn as_str(self) -> &'static str {
        match self {
            SplitDivider::Vertical => "vertical",
            SplitDivider::Horizontal => "horizontal",
        }
    }
}

impl SplitTransition {
    pub fn as_str(self) -> &'static str {
        match self {
            SplitTransition::Soft => "soft",
            SplitTransition::Hard => "hard",
        }
    }
}

impl CursorColor {
    pub fn glsl_vec4(&self) -> String {
        let bytes = self.rgb_bytes();
        format!(
            "vec4({:.3}, {:.3}, {:.3}, 1.0)",
            bytes[0] as f64 / 255.0,
            bytes[1] as f64 / 255.0,
            bytes[2] as f64 / 255.0
        )
    }

    fn rgb_bytes(&self) -> [u8; 3] {
        [
            u8::from_str_radix(&self.hex[1..3], 16).unwrap_or(0),
            u8::from_str_radix(&self.hex[3..5], 16).unwrap_or(0),
            u8::from_str_radix(&self.hex[5..7], 16).unwrap_or(0),
        ]
    }
}

pub fn render_cursor_settings_jsonc(registry: &CursorRegistry) -> String {
    let mut out = String::new();
    out.push_str("// Yazelix Cursors settings\n");
    out.push_str("// Edit this file through `yzx config ui`, `yzc init`, or your editor.\n");
    out.push_str("// In Ghostty standalone setups, add: config-file = ~/.config/yazelix_cursors/ghostty.conf\n");
    out.push_str("{\n");
    out.push_str(&format!(
        "  \"schema_version\": {},\n",
        registry.schema_version
    ));
    out.push_str("  \"enabled_cursors\": [\n");
    for (index, name) in registry.enabled_cursors.iter().enumerate() {
        let comma = if index + 1 == registry.enabled_cursors.len() {
            ""
        } else {
            ","
        };
        out.push_str(&format!("    \"{name}\"{comma}\n"));
    }
    out.push_str("  ],\n");
    out.push_str("  \"settings\": {\n");
    out.push_str(&format!(
        "    \"trail\": \"{}\",\n",
        registry.settings.trail
    ));
    out.push_str(&format!(
        "    \"trail_effect\": \"{}\",\n",
        registry.settings.trail_effect
    ));
    out.push_str(&format!(
        "    \"mode_effect\": \"{}\",\n",
        registry.settings.mode_effect
    ));
    out.push_str(&format!("    \"glow\": \"{}\",\n", registry.settings.glow));
    out.push_str(&format!(
        "    \"duration\": {},\n",
        format_ghostty_trail_duration(registry.settings.duration)
    ));
    out.push_str(&format!(
        "    \"kitty_enable_cursor\": {}\n",
        registry.settings.kitty_enable_cursor
    ));
    out.push_str("  },\n");
    out.push_str("  \"cursor\": [\n");
    let definitions = registry.enabled_definitions();
    for (index, definition) in definitions.iter().enumerate() {
        let comma = if index + 1 == definitions.len() {
            ""
        } else {
            ","
        };
        out.push_str(&render_cursor_definition_jsonc(definition, comma));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn render_cursor_definition_jsonc(definition: &CursorDefinition, comma: &str) -> String {
    let mut out = String::new();
    out.push_str("    {\n");
    out.push_str(&format!("      \"name\": \"{}\",\n", definition.name));
    out.push_str(&format!(
        "      \"family\": \"{}\",\n",
        definition.family.as_str()
    ));
    match definition.family {
        CursorFamily::Mono => {
            out.push_str(&format!(
                "      \"color\": \"{}\",\n",
                definition.colors[0].hex
            ));
            out.push_str(&format!(
                "      \"accent_color\": \"{}\",\n",
                definition.colors[1].hex
            ));
        }
        CursorFamily::Split => {
            let divider = definition
                .divider
                .expect("validated split cursor definitions always have a divider");
            let transition = definition
                .transition
                .expect("validated split cursor definitions always have a transition");
            out.push_str(&format!("      \"divider\": \"{}\",\n", divider.as_str()));
            out.push_str(&format!(
                "      \"transition\": \"{}\",\n",
                transition.as_str()
            ));
            out.push_str("      \"colors\": [\n");
            out.push_str(&format!("        \"{}\",\n", definition.colors[0].hex));
            out.push_str(&format!("        \"{}\"\n", definition.colors[1].hex));
            out.push_str("      ],\n");
        }
        CursorFamily::CuratedTemplate => {
            out.push_str(&format!(
                "      \"template\": \"{}\",\n",
                definition.template.as_deref().unwrap_or("neon")
            ));
        }
    }
    out.push_str(&format!(
        "      \"cursor_color\": \"{}\"\n",
        definition.cursor_color.hex
    ));
    out.push_str(&format!("    }}{comma}\n"));
    out
}

fn validate_settings(
    path: &Path,
    raw: RawCursorSettings,
    enabled_cursors: &[String],
) -> Result<CursorSettings, CoreError> {
    let trail = raw.trail.trim().to_ascii_lowercase();
    if trail != "none" && trail != "random" && !enabled_cursors.contains(&trail) {
        return Err(invalid_cursor_config(
            path,
            "settings.trail",
            format!(
                "settings.trail is '{trail}', but it must be \"none\", \"random\", or a name from enabled_cursors."
            ),
        ));
    }

    let trail_effect = validate_optional_setting(
        path,
        "settings.trail_effect",
        &raw.trail_effect,
        SUPPORTED_TRAIL_EFFECTS,
    )?;
    let mode_effect = validate_optional_setting(
        path,
        "settings.mode_effect",
        &raw.mode_effect,
        SUPPORTED_MODE_EFFECTS,
    )?;
    let glow = validate_required_setting(path, "settings.glow", &raw.glow, SUPPORTED_GLOW_LEVELS)?;
    if !raw.duration.is_finite()
        || !(GHOSTTY_TRAIL_DURATION_MIN..=GHOSTTY_TRAIL_DURATION_MAX).contains(&raw.duration)
    {
        return Err(invalid_cursor_config(
            path,
            "settings.duration",
            format!(
                "settings.duration is {}. Expected a number from {} to {}.",
                raw.duration, GHOSTTY_TRAIL_DURATION_MIN, GHOSTTY_TRAIL_DURATION_MAX
            ),
        ));
    }

    Ok(CursorSettings {
        trail,
        trail_effect,
        mode_effect,
        glow,
        duration: raw.duration,
        kitty_enable_cursor: raw.kitty_enable_cursor,
    })
}

fn validate_definition(
    path: &Path,
    raw: RawCursorDefinition,
) -> Result<CursorDefinition, CoreError> {
    let name = validate_cursor_name(path, "cursor.name", &raw.name)?;
    if REMOVED_CURSOR_NAMES.contains(&name.as_str()) {
        return Err(invalid_cursor_config(
            path,
            "cursor.name",
            format!("Cursor '{name}' is not supported. Remove it from the cursor registry."),
        ));
    }

    let family = match raw.family.trim() {
        "mono" => CursorFamily::Mono,
        "split" => CursorFamily::Split,
        "curated_template" => CursorFamily::CuratedTemplate,
        other => {
            return Err(invalid_cursor_config(
                path,
                "cursor.family",
                format!(
                    "Cursor '{name}' uses unsupported family '{other}'. Expected mono, split, or curated_template."
                ),
            ));
        }
    };

    match family {
        CursorFamily::Mono => {
            if !raw.colors.is_empty() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.colors",
                    format!("Cursor '{name}' uses mono and must not define colors."),
                ));
            }
            if raw.divider.is_some() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.divider",
                    format!("Cursor '{name}' uses mono and must not set divider."),
                ));
            }
            if raw.transition.is_some() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.transition",
                    format!("Cursor '{name}' uses mono and must not set transition."),
                ));
            }
            if raw.template.is_some() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.template",
                    format!("Cursor '{name}' is data-driven and must not set template."),
                ));
            }
            let color = raw.color.as_deref().ok_or_else(|| {
                invalid_cursor_config(
                    path,
                    "cursor.color",
                    format!("Cursor '{name}' uses mono and must set color."),
                )
            })?;
            let base_color = validate_color(path, &format!("cursor.{name}.color"), color)?;
            let accent_color = match raw.accent_color.as_deref() {
                Some(accent) => {
                    validate_color(path, &format!("cursor.{name}.accent_color"), accent)?
                }
                None => derive_accent_color(&base_color),
            };
            let cursor_color = match raw.cursor_color.as_deref() {
                Some(cursor_color) => {
                    validate_color(path, &format!("cursor.{name}.cursor_color"), cursor_color)?
                }
                None => base_color.clone(),
            };

            Ok(CursorDefinition {
                name,
                family,
                colors: vec![base_color, accent_color],
                divider: None,
                transition: None,
                template: None,
                cursor_color,
            })
        }
        CursorFamily::Split => {
            if raw.color.is_some() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.color",
                    format!("Cursor '{name}' uses split and must not set color."),
                ));
            }
            if raw.accent_color.is_some() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.accent_color",
                    format!("Cursor '{name}' uses split and must not set accent_color."),
                ));
            }
            if raw.colors.len() != 2 {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.colors",
                    format!("Cursor '{name}' uses split and must define exactly 2 colors."),
                ));
            }
            if raw.template.is_some() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.template",
                    format!("Cursor '{name}' is data-driven and must not set template."),
                ));
            }
            let divider = validate_split_divider(path, &name, raw.divider.as_deref())?;
            let transition = validate_split_transition(path, &name, raw.transition.as_deref())?;
            let colors = raw
                .colors
                .iter()
                .enumerate()
                .map(|(index, color)| {
                    validate_color(path, &format!("cursor.{name}.colors[{index}]"), color)
                })
                .collect::<Result<Vec<_>, _>>()?;
            let cursor_color = match raw.cursor_color.as_deref() {
                Some(cursor_color) => {
                    validate_color(path, &format!("cursor.{name}.cursor_color"), cursor_color)?
                }
                None => colors[0].clone(),
            };

            Ok(CursorDefinition {
                name,
                family,
                colors,
                divider: Some(divider),
                transition: Some(transition),
                template: None,
                cursor_color,
            })
        }
        CursorFamily::CuratedTemplate => {
            validate_curated_template_definition(path, name, family, raw)
        }
    }
}

fn validate_curated_template_definition(
    path: &Path,
    name: String,
    family: CursorFamily,
    raw: RawCursorDefinition,
) -> Result<CursorDefinition, CoreError> {
    if raw.color.is_some() {
        return Err(invalid_cursor_config(
            path,
            "cursor.color",
            format!("Cursor '{name}' uses curated_template and must not set color."),
        ));
    }
    if raw.accent_color.is_some() {
        return Err(invalid_cursor_config(
            path,
            "cursor.accent_color",
            format!("Cursor '{name}' uses curated_template and must not set accent_color."),
        ));
    }
    if !raw.colors.is_empty() {
        return Err(invalid_cursor_config(
            path,
            "cursor.colors",
            format!("Cursor '{name}' uses curated_template and must not define colors."),
        ));
    }
    if raw.divider.is_some() {
        return Err(invalid_cursor_config(
            path,
            "cursor.divider",
            format!("Cursor '{name}' uses curated_template and must not set divider."),
        ));
    }
    if raw.transition.is_some() {
        return Err(invalid_cursor_config(
            path,
            "cursor.transition",
            format!("Cursor '{name}' uses curated_template and must not set transition."),
        ));
    }

    let template = raw
        .template
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            invalid_cursor_config(
                path,
                "cursor.template",
                format!("Cursor '{name}' uses curated_template and must set template."),
            )
        })?;
    if !SUPPORTED_CURATED_TEMPLATES.contains(&template) {
        return Err(invalid_cursor_config(
            path,
            "cursor.template",
            format!(
                "Cursor '{name}' uses unsupported curated template '{template}'. Expected neon."
            ),
        ));
    }

    let cursor_color = raw.cursor_color.as_deref().ok_or_else(|| {
        invalid_cursor_config(
            path,
            "cursor.cursor_color",
            format!("Cursor '{name}' uses curated_template and must set cursor_color."),
        )
    })?;
    let cursor_color = validate_color(path, &format!("cursor.{name}.cursor_color"), cursor_color)?;

    Ok(CursorDefinition {
        name,
        family,
        colors: Vec::new(),
        divider: None,
        transition: None,
        template: Some(template.to_ascii_lowercase()),
        cursor_color,
    })
}

fn validate_split_divider(
    path: &Path,
    name: &str,
    raw_divider: Option<&str>,
) -> Result<SplitDivider, CoreError> {
    let Some(divider) = raw_divider.map(str::trim) else {
        return Err(invalid_cursor_config(
            path,
            "cursor.divider",
            format!("Cursor '{name}' uses split and must set divider to vertical or horizontal."),
        ));
    };

    match divider {
        "vertical" => Ok(SplitDivider::Vertical),
        "horizontal" => Ok(SplitDivider::Horizontal),
        other => Err(invalid_cursor_config(
            path,
            "cursor.divider",
            format!(
                "Cursor '{name}' uses unsupported split divider '{other}'. Expected vertical or horizontal."
            ),
        )),
    }
}

fn validate_split_transition(
    path: &Path,
    name: &str,
    raw_transition: Option<&str>,
) -> Result<SplitTransition, CoreError> {
    let Some(transition) = raw_transition.map(str::trim) else {
        return Err(invalid_cursor_config(
            path,
            "cursor.transition",
            format!("Cursor '{name}' uses split and must set transition to soft or hard."),
        ));
    };

    match transition {
        "soft" => Ok(SplitTransition::Soft),
        "hard" => Ok(SplitTransition::Hard),
        other => Err(invalid_cursor_config(
            path,
            "cursor.transition",
            format!(
                "Cursor '{name}' uses unsupported split transition '{other}'. Expected soft or hard."
            ),
        )),
    }
}

fn validate_optional_setting(
    path: &Path,
    field: &str,
    value: &str,
    allowed: &[&str],
) -> Result<String, CoreError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized == "none" || normalized == "random" || allowed.contains(&normalized.as_str()) {
        return Ok(normalized);
    }
    Err(invalid_cursor_config(
        path,
        field,
        format!(
            "{field} is '{normalized}'. Expected none, random, or one of: {}.",
            allowed.join(", ")
        ),
    ))
}

fn validate_required_setting(
    path: &Path,
    field: &str,
    value: &str,
    allowed: &[&str],
) -> Result<String, CoreError> {
    let normalized = value.trim().to_ascii_lowercase();
    if allowed.contains(&normalized.as_str()) {
        return Ok(normalized);
    }
    Err(invalid_cursor_config(
        path,
        field,
        format!(
            "{field} is '{normalized}'. Expected one of: {}.",
            allowed.join(", ")
        ),
    ))
}

fn validate_cursor_name(path: &Path, field: &str, value: &str) -> Result<String, CoreError> {
    let normalized = value.trim().to_ascii_lowercase();
    let valid = !normalized.is_empty()
        && normalized
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if valid {
        return Ok(normalized);
    }
    Err(invalid_cursor_config(
        path,
        field,
        format!(
            "{field} value '{value}' is invalid. Use lowercase letters, digits, and underscores only."
        ),
    ))
}

fn validate_color(path: &Path, field: &str, value: &str) -> Result<CursorColor, CoreError> {
    let normalized = value.trim().to_ascii_lowercase();
    let valid = normalized.len() == 7
        && normalized.starts_with('#')
        && normalized[1..].bytes().all(|byte| byte.is_ascii_hexdigit());
    if valid {
        return Ok(CursorColor { hex: normalized });
    }
    Err(invalid_cursor_config(
        path,
        field,
        format!("{field} value '{value}' is invalid. Use a #rrggbb hex color."),
    ))
}

fn derive_accent_color(base: &CursorColor) -> CursorColor {
    let [red, green, blue] = base.rgb_bytes();
    let (hue, saturation, lightness) = rgb_to_hsl(red, green, blue);
    let (accent_hue, accent_saturation, accent_lightness) = if saturation < 0.08 || lightness > 0.92
    {
        (hue, saturation, lightness * 0.80)
    } else if !(45.0..330.0).contains(&hue) {
        (hue - 22.0, (saturation + 0.05).min(1.0), lightness - 0.06)
    } else if hue < 80.0 {
        (hue - 45.0, saturation, lightness - 0.08)
    } else if hue < 180.0 {
        (hue + 4.0, (saturation + 0.08).min(1.0), lightness - 0.16)
    } else if hue < 250.0 {
        (hue + 8.0, (saturation - 0.08).max(0.0), lightness - 0.15)
    } else {
        (hue - 20.0, (saturation - 0.15).max(0.0), lightness - 0.12)
    };

    let [red, green, blue] = hsl_to_rgb(
        accent_hue,
        accent_saturation,
        accent_lightness.clamp(0.0, 1.0),
    );
    CursorColor {
        hex: format!("#{red:02x}{green:02x}{blue:02x}"),
    }
}

fn rgb_to_hsl(red: u8, green: u8, blue: u8) -> (f64, f64, f64) {
    let red = f64::from(red) / 255.0;
    let green = f64::from(green) / 255.0;
    let blue = f64::from(blue) / 255.0;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let lightness = (max + min) / 2.0;
    let delta = max - min;

    if delta == 0.0 {
        return (0.0, 0.0, lightness);
    }

    let saturation = if lightness > 0.5 {
        delta / (2.0 - max - min)
    } else {
        delta / (max + min)
    };
    let hue = if max == red {
        60.0 * ((green - blue) / delta).rem_euclid(6.0)
    } else if max == green {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    };

    (hue, saturation, lightness)
}

fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> [u8; 3] {
    let hue = hue.rem_euclid(360.0) / 360.0;
    let saturation = saturation.clamp(0.0, 1.0);
    let lightness = lightness.clamp(0.0, 1.0);

    if saturation == 0.0 {
        let value = float_to_byte(lightness);
        return [value, value, value];
    }

    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - (lightness * saturation)
    };
    let p = (2.0 * lightness) - q;
    [
        float_to_byte(hue_to_rgb(p, q, hue + (1.0 / 3.0))),
        float_to_byte(hue_to_rgb(p, q, hue)),
        float_to_byte(hue_to_rgb(p, q, hue - (1.0 / 3.0))),
    ]
}

fn hue_to_rgb(p: f64, q: f64, hue: f64) -> f64 {
    let hue = hue.rem_euclid(1.0);
    if hue < 1.0 / 6.0 {
        p + (q - p) * 6.0 * hue
    } else if hue < 1.0 / 2.0 {
        q
    } else if hue < 2.0 / 3.0 {
        p + (q - p) * ((2.0 / 3.0) - hue) * 6.0
    } else {
        p
    }
}

fn float_to_byte(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn resolve_optional_effect(value: &str, allowed: &[&str], entropy: usize) -> Option<String> {
    match value {
        "none" => None,
        "random" => allowed
            .get(entropy % allowed.len())
            .map(|value| value.to_string()),
        other => Some(other.to_string()),
    }
}

pub fn format_ghostty_trail_duration(duration: f64) -> String {
    let mut rendered = format!("{duration:.3}");
    while rendered.contains('.') && rendered.ends_with('0') {
        rendered.pop();
    }
    if rendered.ends_with('.') {
        rendered.push('0');
    }
    rendered
}

pub fn write_ghostty_cursor_palette_shaders(
    shaders_dest: &Path,
    registry: &CursorRegistry,
    glow_level: &str,
    trail_duration: f64,
) -> Result<(), CoreError> {
    let data_driven = registry
        .enabled_definitions()
        .into_iter()
        .filter(|definition| matches!(definition.family, CursorFamily::Mono | CursorFamily::Split))
        .collect::<Vec<_>>();
    if data_driven.is_empty() {
        return Ok(());
    }

    let common_path = shaders_dest.join("cursor_trail_common.glsl");
    let common = fs::read_to_string(&common_path).map_err(|source| {
        CoreError::io(
            "read_ghostty_shader_common",
            "Could not read the copied Ghostty cursor shader common library",
            "Reinstall Yazelix so the runtime includes configs/terminal_emulators/ghostty/shaders/cursor_trail_common.glsl.",
            common_path.to_string_lossy(),
            source,
        )
    })?;
    let glow_header = render_trail_glow_header(glow_level);

    for definition in data_driven {
        let output_path = shaders_dest.join(format!("cursor_trail_{}.glsl", definition.name));
        let rendered = format!(
            "{}{}\n{}",
            glow_header,
            common,
            render_data_driven_cursor_variant(definition, trail_duration)
        );
        fs::write(&output_path, rendered).map_err(|source| {
            CoreError::io(
                "write_data_driven_cursor_shader",
                "Could not write generated Ghostty cursor shader",
                "Check permissions for the Yazelix state directory and retry.",
                output_path.to_string_lossy(),
                source,
            )
        })?;
    }

    Ok(())
}

pub fn write_ghostty_cursor_effect_shaders(
    shaders_dest: &Path,
    glow_level: &str,
    effect_color_literal: &str,
    trail_duration: f64,
) -> Result<(), CoreError> {
    let templates_dir = shaders_dest.join("upstream_effects");
    if !templates_dir.exists() {
        return Err(CoreError::classified(
            ErrorClass::Io,
            "missing_ghostty_effect_templates",
            "Could not find bundled Ghostty cursor effect templates.",
            "Reinstall the yazelix_cursors package so share/yazelix/yazelix_cursors/shaders/upstream_effects exists.",
            json!({ "path": templates_dir.display().to_string() }),
        ));
    }

    let generated_dir = shaders_dest.join("generated_effects");
    if generated_dir.exists() {
        fs::remove_dir_all(&generated_dir).map_err(|source| {
            CoreError::io(
                "remove_generated_ghostty_effect_shaders",
                "Could not remove previous generated Ghostty cursor effect shaders",
                "Check permissions for the generated Yazelix cursor shader directory and retry.",
                generated_dir.to_string_lossy(),
                source,
            )
        })?;
    }
    fs::create_dir_all(&generated_dir).map_err(|source| {
        CoreError::io(
            "create_generated_ghostty_effect_shaders",
            "Could not create generated Ghostty cursor effect shader directory",
            "Check permissions for the generated Yazelix cursor shader directory and retry.",
            generated_dir.to_string_lossy(),
            source,
        )
    })?;

    for (effect, template_name) in GHOSTTY_CURSOR_EFFECT_TEMPLATES {
        let template_path = templates_dir.join(template_name);
        let template = fs::read_to_string(&template_path).map_err(|source| {
            CoreError::io(
                "read_ghostty_effect_template",
                "Could not read bundled Ghostty cursor effect template",
                "Reinstall the yazelix_cursors package and retry.",
                template_path.to_string_lossy(),
                source,
            )
        })?;
        let duration = if GHOSTTY_CURSOR_MOVEMENT_EFFECTS.contains(effect) {
            trail_duration
        } else {
            1.0
        };
        let rendered = render_ghostty_cursor_effect_shader(
            &template,
            glow_level,
            effect_color_literal,
            duration,
        );
        let header = format!(
            "// Generated by Yazelix from a vendored Ghostty cursor effect template\n\
             // Source repository: https://github.com/sahaj-b/ghostty-cursor-shaders\n\
             // Effect: {effect}\n\
             // Color source: {effect_color_literal}\n\
             // cursor settings.glow = {glow_level}\n\
             // cursor settings.duration = {}\n\n",
            format_ghostty_trail_duration(duration)
        );
        let output_path = generated_dir.join(format!("{effect}.glsl"));
        fs::write(&output_path, format!("{header}{rendered}")).map_err(|source| {
            CoreError::io(
                "write_ghostty_effect_shader",
                "Could not write generated Ghostty cursor effect shader",
                "Check permissions for the generated Yazelix cursor shader directory and retry.",
                output_path.to_string_lossy(),
                source,
            )
        })?;
    }

    Ok(())
}

fn render_data_driven_cursor_variant(definition: &CursorDefinition, duration_scale: f64) -> String {
    let color_0 = definition.colors[0].glsl_vec4();
    let color_1 = definition.colors[1].glsl_vec4();
    match definition.family {
        CursorFamily::Mono => {
            let duration = format_ghostty_trail_duration(0.25 * duration_scale);
            format!(
                r#"// Generated Yazelix mono cursor variant

const vec4 YAZELIX_CURSOR_COLOR_0 = {color_0};
const vec4 YAZELIX_CURSOR_COLOR_1 = {color_1};
const float DURATION = {duration};

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{{
    renderMonoColorTrail(fragColor, fragCoord, YAZELIX_CURSOR_COLOR_0, YAZELIX_CURSOR_COLOR_1, DURATION, .007, 1.5);
}}
"#
            )
        }
        CursorFamily::Split => {
            let duration = format_ghostty_trail_duration(0.24 * duration_scale);
            let horizontal = match definition
                .divider
                .expect("validated split cursor definitions always have a divider")
            {
                SplitDivider::Vertical => "0.0",
                SplitDivider::Horizontal => "1.0",
            };
            let transition = match definition
                .transition
                .expect("validated split cursor definitions always have a transition")
            {
                SplitTransition::Soft => "1.0",
                SplitTransition::Hard => "0.0",
            };
            format!(
                r#"// Generated Yazelix split cursor variant

const vec4 YAZELIX_CURSOR_COLOR_0 = {color_0};
const vec4 YAZELIX_CURSOR_COLOR_1 = {color_1};
const float DURATION = {duration};
const float YAZELIX_SPLIT_HORIZONTAL = {horizontal};
const float YAZELIX_SPLIT_BLEND = {transition};

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{{
    renderSplitColorTrail(fragColor, fragCoord, YAZELIX_CURSOR_COLOR_0, YAZELIX_CURSOR_COLOR_1, DURATION, YAZELIX_SPLIT_HORIZONTAL, YAZELIX_SPLIT_BLEND);
}}
"#
            )
        }
        CursorFamily::CuratedTemplate => String::new(),
    }
}

fn render_trail_glow_header(glow_level: &str) -> String {
    let profile = glow_profile(glow_level);
    format!(
        r#"// Generated by Yazelix with cursor glow = {glow_level}
const float YAZELIX_TRAIL_GLOW_STRENGTH = {};
const float YAZELIX_TRAIL_GLOW_WIDTH_SCALE = {};
const float YAZELIX_CURSOR_GLOW_STRENGTH = {};
const float YAZELIX_CURSOR_GLOW_WIDTH_SCALE = {};
const float YAZELIX_TRAIL_EDGE_WIDTH_SCALE = {};
const float YAZELIX_CURSOR_EDGE_WIDTH_SCALE = {};
const float YAZELIX_TRAIL_CORE_OFFSET_SCALE = {};

"#,
        profile.trail_glow_strength,
        profile.trail_glow_width_scale,
        profile.cursor_glow_strength,
        profile.cursor_glow_width_scale,
        profile.trail_edge_width_scale,
        profile.cursor_edge_width_scale,
        profile.trail_core_offset_scale,
    )
}

fn render_ghostty_cursor_effect_shader(
    template: &str,
    glow_level: &str,
    effect_color_literal: &str,
    duration_scale: f64,
) -> String {
    let profile = glow_profile(glow_level);
    let color_source = normalize_effect_color_literal(effect_color_literal);
    let mut rendered = replace_vec4_assignment(template, "COLOR", &color_source);
    rendered = replace_vec4_assignment(&rendered, "TRAIL_COLOR", &color_source);
    rendered = scale_glsl_float_constant(&rendered, "BLUR", profile.effect_blur_factor);
    rendered = scale_glsl_float_constant(&rendered, "MAX_RADIUS", profile.effect_spread_factor);
    rendered = scale_glsl_float_constant(&rendered, "MAX_SIZE", profile.effect_spread_factor);
    rendered =
        scale_glsl_float_constant(&rendered, "MAX_TRAIL_LENGTH", profile.effect_spread_factor);
    rendered = scale_glsl_float_constant(&rendered, "TRAIL_LENGTH", profile.effect_spread_factor);
    rendered = scale_glsl_float_constant(&rendered, "TRAIL_SIZE", profile.effect_spread_factor);
    rendered = scale_glsl_float_constant(
        &rendered,
        "RING_THICKNESS",
        profile.effect_ring_thickness_factor,
    );
    scale_glsl_float_constant(&rendered, "DURATION", duration_scale)
}

fn normalize_effect_color_literal(effect_color_literal: &str) -> String {
    let trimmed = effect_color_literal.trim();
    if trimmed.is_empty() {
        "iCurrentCursorColor".to_string()
    } else {
        trimmed.to_string()
    }
}

fn replace_vec4_assignment(source: &str, constant_name: &str, value: &str) -> String {
    rewrite_assignment_line(source, &format!("vec4 {constant_name} = "), value)
}

fn scale_glsl_float_constant(source: &str, constant_name: &str, factor: f64) -> String {
    let prefix = format!("const float {constant_name} = ");
    rewrite_assignment_line_with_value(source, &prefix, |value| {
        parse_leading_float(value)
            .map(|parsed| format_ghostty_trail_duration(parsed * factor))
            .unwrap_or_else(|| value.trim().to_string())
    })
}

fn rewrite_assignment_line(source: &str, prefix: &str, value: &str) -> String {
    rewrite_assignment_line_with_value(source, prefix, |_| value.to_string())
}

fn rewrite_assignment_line_with_value(
    source: &str,
    prefix: &str,
    rewrite: impl Fn(&str) -> String,
) -> String {
    source
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let indent_len = line.len() - trimmed.len();
            let Some(rest) = trimmed.strip_prefix(prefix) else {
                return line.to_string();
            };
            let Some(semicolon_index) = rest.find(';') else {
                return line.to_string();
            };
            let (value, suffix) = rest.split_at(semicolon_index);
            format!(
                "{}{}{}{}",
                &line[..indent_len],
                prefix,
                rewrite(value),
                suffix
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_leading_float(value: &str) -> Option<f64> {
    value.trim().parse::<f64>().ok()
}

struct GlowProfile {
    trail_glow_strength: &'static str,
    cursor_glow_strength: &'static str,
    trail_edge_width_scale: &'static str,
    cursor_edge_width_scale: &'static str,
    trail_core_offset_scale: &'static str,
    trail_glow_width_scale: &'static str,
    cursor_glow_width_scale: &'static str,
    effect_blur_factor: f64,
    effect_spread_factor: f64,
    effect_ring_thickness_factor: f64,
}

fn glow_profile(glow_level: &str) -> GlowProfile {
    match glow_level {
        "none" => GlowProfile {
            trail_glow_strength: "0.0",
            cursor_glow_strength: "0.0",
            trail_edge_width_scale: "0.0",
            cursor_edge_width_scale: "0.0",
            trail_core_offset_scale: "0.0",
            trail_glow_width_scale: "1.0",
            cursor_glow_width_scale: "1.0",
            effect_blur_factor: 0.1,
            effect_spread_factor: 0.0,
            effect_ring_thickness_factor: 0.0,
        },
        "low" => GlowProfile {
            trail_glow_strength: "1.0",
            cursor_glow_strength: "1.0",
            trail_edge_width_scale: "1.0",
            cursor_edge_width_scale: "1.0",
            trail_core_offset_scale: "1.0",
            trail_glow_width_scale: "0.55",
            cursor_glow_width_scale: "0.6",
            effect_blur_factor: 0.7,
            effect_spread_factor: 1.0,
            effect_ring_thickness_factor: 1.0,
        },
        "high" => GlowProfile {
            trail_glow_strength: "1.0",
            cursor_glow_strength: "1.0",
            trail_edge_width_scale: "1.0",
            cursor_edge_width_scale: "1.0",
            trail_core_offset_scale: "1.0",
            trail_glow_width_scale: "1.7",
            cursor_glow_width_scale: "1.6",
            effect_blur_factor: 1.45,
            effect_spread_factor: 1.0,
            effect_ring_thickness_factor: 1.0,
        },
        _ => GlowProfile {
            trail_glow_strength: "1.0",
            cursor_glow_strength: "1.0",
            trail_edge_width_scale: "1.0",
            cursor_edge_width_scale: "1.0",
            trail_core_offset_scale: "1.0",
            trail_glow_width_scale: "1.0",
            cursor_glow_width_scale: "1.0",
            effect_blur_factor: 1.0,
            effect_spread_factor: 1.0,
            effect_ring_thickness_factor: 1.0,
        },
    }
}

fn invalid_cursor_config(path: &Path, field: &str, detail: String) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        "invalid_cursor_config",
        format!("Invalid Yazelix cursor config at {field}."),
        "Update the cursor registry data, then retry.",
        json!({
            "path": path.display().to_string(),
            "field": field,
            "detail": detail,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};

    fn write_registry(raw: &str) -> (TempDir, PathBuf) {
        let temp = tempdir().unwrap();
        let path = temp.path().join("cursors.toml");
        fs::write(&path, raw).unwrap();
        (temp, path)
    }

    fn load_registry(path: &Path) -> Result<CursorRegistry, CoreError> {
        let raw = fs::read_to_string(path).unwrap();
        CursorRegistry::parse_str(path, &raw)
    }

    fn base_registry(extra: &str) -> String {
        format!(
            r##"
schema_version = 1
enabled_cursors = ["blaze"]

[settings]
trail = "random"
trail_effect = "random"
mode_effect = "random"
glow = "medium"
duration = 1.0
kitty_enable_cursor = true

[[cursor]]
name = "blaze"
family = "mono"
color = "#ffb929"
{extra}
"##
        )
    }

    // Defends: the shipped cursor registry can resolve a one-item enabled list and random only draws from that list.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_resolves_random_from_enabled_cursors() {
        let (_temp, path) = write_registry(&base_registry(""));
        let registry = load_registry(&path).unwrap();

        let resolved = registry.resolve_with_entropy(51);

        assert_eq!(resolved.selected_cursor.unwrap().name, "blaze");
        assert_eq!(resolved.selected_trail_effect, Some("tail".to_string()));
        assert_eq!(
            resolved.selected_mode_effect,
            Some("ripple_rectangle".to_string())
        );
        assert!(resolved.kitty_enable_cursor);
    }

    // Defends: mono cursors accept one base color and derive the shader accent without requiring palette duplication.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_derives_mono_accent_and_cursor_color() {
        let (_temp, path) = write_registry(&base_registry(""));

        let registry = load_registry(&path).unwrap();
        let blaze = registry.definitions.get("blaze").unwrap();

        assert_eq!(blaze.family, CursorFamily::Mono);
        assert_eq!(blaze.colors[0].hex, "#ffb929");
        assert_eq!(blaze.colors.len(), 2);
        assert_ne!(blaze.colors[1].hex, "#ffb929");
        assert_eq!(blaze.cursor_color.hex, "#ffb929");
    }

    // Defends: mono cursors still allow explicit accent and cursor overrides when the heuristic is not enough.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_accepts_mono_accent_and_cursor_overrides() {
        let (_temp, path) = write_registry(&base_registry(
            r##"
accent_color = "#ff0000"
cursor_color = "#00ff66"
"##,
        ));

        let registry = load_registry(&path).unwrap();
        let blaze = registry.definitions.get("blaze").unwrap();

        assert_eq!(blaze.colors[1].hex, "#ff0000");
        assert_eq!(blaze.cursor_color.hex, "#00ff66");
    }

    // Defends: split cursors carry the explicit divider and transition contract used by generated shaders.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_parses_split_divider_and_transition() {
        let raw = base_registry("").replace(
            r##"name = "blaze"
family = "mono"
color = "#ffb929""##,
            r##"name = "blaze"
family = "split"
divider = "horizontal"
transition = "hard"
colors = ["#ff1600", "#2a3340"]"##,
        );
        let (_temp, path) = write_registry(&raw);

        let registry = load_registry(&path).unwrap();
        let blaze = registry.definitions.get("blaze").unwrap();

        assert_eq!(blaze.family, CursorFamily::Split);
        assert_eq!(blaze.divider, Some(SplitDivider::Horizontal));
        assert_eq!(blaze.transition, Some(SplitTransition::Hard));
        assert_eq!(blaze.cursor_color.hex, "#ff1600");
    }

    // Defends: disabled Kitty cursor fallback remains a first-class sidecar setting independent of Ghostty shader selection.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_parses_kitty_enable_cursor_as_binary_setting() {
        let mut raw = base_registry("");
        raw = raw.replace("kitty_enable_cursor = true", "kitty_enable_cursor = false");
        let (_temp, path) = write_registry(&raw);

        let registry = load_registry(&path).unwrap();

        assert!(!registry.settings.kitty_enable_cursor);
        assert!(!registry.resolve_with_entropy(0).kitty_enable_cursor);
    }

    // Defends: enabled_cursors must resolve exactly once to a cursor definition.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_rejects_missing_enabled_cursor_definition() {
        let raw = base_registry("").replace(
            "enabled_cursors = [\"blaze\"]",
            "enabled_cursors = [\"reef\"]",
        );
        let (_temp, path) = write_registry(&raw);

        let error = load_registry(&path).unwrap_err();

        assert_eq!(error.code(), "invalid_cursor_config");
        assert!(format!("{error:?}").contains("enabled_cursors"));
    }

    // Defends: duplicate cursor definitions fail fast before shader paths can become ambiguous.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_rejects_duplicate_cursor_definitions() {
        let raw = base_registry(
            r##"
[[cursor]]
name = "blaze"
family = "mono"
color = "#ffffff"
"##,
        );
        let (_temp, path) = write_registry(&raw);

        let error = load_registry(&path).unwrap_err();

        assert_eq!(error.code(), "invalid_cursor_config");
        assert!(format!("{error:?}").contains("defined more than once"));
    }

    // Defends: color and family validation rejects invalid user-authored shader inputs before runtime files are written.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_rejects_invalid_color_for_data_driven_cursor() {
        let raw = base_registry("").replace("#ffb929", "red");
        let (_temp, path) = write_registry(&raw);

        let error = load_registry(&path).unwrap_err();

        assert_eq!(error.code(), "invalid_cursor_config");
        assert!(format!("{error:?}").contains("#rrggbb"));
    }

    // Regression: retired data-driven family names must fail fast instead of silently taking compatibility paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_rejects_retired_data_driven_family_names() {
        let raw = base_registry("").replace("family = \"mono\"", "family = \"simple_dual\"");
        let (_temp, path) = write_registry(&raw);

        let error = load_registry(&path).unwrap_err();

        assert_eq!(error.code(), "invalid_cursor_config");
        assert!(format!("{error:?}").contains("Expected mono, split, or curated_template"));
    }

    // Regression: retired split field names must fail fast instead of silently taking compatibility paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_rejects_retired_split_field_names() {
        let raw = base_registry("").replace(
            r##"name = "blaze"
family = "mono"
color = "#ffb929""##,
            r##"name = "blaze"
family = "split"
direction = "horizontal"
blend = false
colors = ["#ff1600", "#2a3340"]"##,
        );
        let (_temp, path) = write_registry(&raw);

        let error = load_registry(&path).unwrap_err();

        assert_eq!(error.code(), "invalid_cursor_config_toml");
    }

    // Defends: the standalone cursor package can generate Ghostty palette shaders from the registry without settings.jsonc or runtime materialization.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn palette_shader_generation_uses_reusable_cursor_registry_boundary() {
        let (_registry_temp, path) = write_registry(&base_registry(""));
        let registry = load_registry(&path).unwrap();
        let shader_dir = tempdir().unwrap();
        fs::write(
            shader_dir.path().join("cursor_trail_common.glsl"),
            "void renderMonoColorTrail(out vec4 fragColor, in vec2 fragCoord, vec4 color0, vec4 color1, float duration, float width, float scale) {}\n",
        )
        .unwrap();

        write_ghostty_cursor_palette_shaders(shader_dir.path(), &registry, "medium", 1.0).unwrap();

        let generated =
            fs::read_to_string(shader_dir.path().join("cursor_trail_blaze.glsl")).unwrap();
        assert!(generated.contains("Generated Yazelix mono cursor variant"));
        assert!(generated.contains("YAZELIX_TRAIL_GLOW_STRENGTH"));
        assert!(generated.contains("const float DURATION = 0.25;"));
        assert!(generated.contains("const vec4 YAZELIX_CURSOR_COLOR_0"));
    }

    // Defends: the shipped default registry parses as the active product cursor surface.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn shipped_default_registry_parses_active_cursor_surface() {
        let (_temp, path) = write_registry(include_str!("../../../yazelix_cursors_default.toml"));

        let registry = load_registry(&path).unwrap();

        assert!(registry.enabled_cursors.contains(&"blaze".to_string()));
        assert!(registry.enabled_cursors.contains(&"neon".to_string()));
        assert!(registry.enabled_cursors.contains(&"magma".to_string()));
        assert!(!registry.enabled_cursors.contains(&"inferno".to_string()));
        assert!(
            registry
                .enabled_cursors
                .iter()
                .all(|name| registry.definitions.contains_key(name))
        );
        assert_eq!(
            registry.definitions.get("magma").unwrap().divider,
            Some(SplitDivider::Horizontal)
        );
        assert_eq!(
            registry.definitions.get("orchid").unwrap().transition,
            Some(SplitTransition::Hard)
        );
        assert_eq!(
            registry.definitions.get("reef").unwrap().colors[1].hex,
            "#00ff66"
        );
        assert_eq!(registry.settings.trail, "random");
    }
}
