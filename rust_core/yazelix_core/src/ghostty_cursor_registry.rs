// Test lane: default
//! Data-driven Yazelix cursor registry for Ghostty shader materialization.

use crate::bridge::{CoreError, ErrorClass};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_CURSOR_CONFIG_FILENAME: &str = "yazelix_cursors_default.toml";
pub const USER_CURSOR_CONFIG_FILENAME: &str = "yazelix_cursors.toml";
pub const DEFAULT_GHOSTTY_TRAIL_DURATION: f64 = 1.0;
pub const GHOSTTY_TRAIL_DURATION_MIN: f64 = 0.25;
pub const GHOSTTY_TRAIL_DURATION_MAX: f64 = 4.0;

const SUPPORTED_TRAIL_EFFECTS: &[&str] = &["tail", "warp", "sweep"];
const SUPPORTED_MODE_EFFECTS: &[&str] =
    &["ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle"];
const SUPPORTED_GLOW_LEVELS: &[&str] = &["none", "low", "medium", "high"];
const SUPPORTED_CURATED_TEMPLATES: &[&str] = &["neon"];
const REMOVED_CURSOR_NAMES: &[&str] = &["party"];

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
    pub template: Option<String>,
    pub cursor_color: CursorColor,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CursorFamily {
    SimpleDual,
    AxisGradient,
    CuratedTemplate,
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
    #[serde(default)]
    colors: Vec<String>,
    template: Option<String>,
    cursor_color: String,
}

impl CursorRegistry {
    pub fn load(path: &Path) -> Result<Self, CoreError> {
        let raw = fs::read_to_string(path).map_err(|source| {
            CoreError::io(
                "read_cursor_config",
                "Could not read Yazelix cursor config",
                "Restore user_configs/yazelix_cursors.toml from yazelix_cursors_default.toml, then retry.",
                path.to_string_lossy(),
                source,
            )
        })?;
        CursorRegistry::parse_str(path, &raw)
    }

    pub fn parse_str(path: &Path, raw: &str) -> Result<Self, CoreError> {
        let parsed = toml::from_str::<RawCursorRegistry>(&raw).map_err(|source| {
            CoreError::toml(
                "invalid_cursor_config_toml",
                "Could not parse Yazelix cursor config",
                "Fix ~/.config/yazelix/user_configs/yazelix_cursors.toml or restore it from yazelix_cursors_default.toml.",
                path.to_string_lossy(),
                source,
            )
        })?;
        CursorRegistry::from_raw(path, parsed)
    }

    pub fn user_config_path(config_dir: &Path) -> PathBuf {
        config_dir
            .join("user_configs")
            .join(USER_CURSOR_CONFIG_FILENAME)
    }

    pub fn default_config_path(runtime_dir: &Path) -> PathBuf {
        runtime_dir.join(DEFAULT_CURSOR_CONFIG_FILENAME)
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
            CursorFamily::SimpleDual | CursorFamily::AxisGradient => {
                format!("./shaders/cursor_trail_{}.glsl", self.name)
            }
        }
    }

    pub fn cursor_color_hex(&self) -> &str {
        &self.cursor_color.hex
    }

    pub fn cursor_color_literal(&self) -> String {
        self.cursor_color.glsl_vec4()
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
            format!("Cursor '{name}' is not supported. Remove it from yazelix_cursors.toml."),
        ));
    }

    let family = match raw.family.trim() {
        "simple_dual" => CursorFamily::SimpleDual,
        "axis_gradient" => CursorFamily::AxisGradient,
        "curated_template" => CursorFamily::CuratedTemplate,
        other => {
            return Err(invalid_cursor_config(
                path,
                "cursor.family",
                format!(
                    "Cursor '{name}' uses unsupported family '{other}'. Expected simple_dual, axis_gradient, or curated_template."
                ),
            ));
        }
    };
    let cursor_color = validate_color(
        path,
        &format!("cursor.{name}.cursor_color"),
        &raw.cursor_color,
    )?;

    match family {
        CursorFamily::SimpleDual | CursorFamily::AxisGradient => {
            if raw.colors.len() != 2 {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.colors",
                    format!(
                        "Cursor '{name}' uses family '{}', which requires exactly 2 colors.",
                        raw.family
                    ),
                ));
            }
            if raw.template.is_some() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.template",
                    format!("Cursor '{name}' is data-driven and must not set template."),
                ));
            }
        }
        CursorFamily::CuratedTemplate => {
            if !raw.colors.is_empty() {
                return Err(invalid_cursor_config(
                    path,
                    "cursor.colors",
                    format!("Cursor '{name}' uses curated_template and must not define colors."),
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
        }
    }

    let colors = raw
        .colors
        .iter()
        .enumerate()
        .map(|(index, color)| {
            validate_color(path, &format!("cursor.{name}.colors[{index}]"), color)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CursorDefinition {
        name,
        family,
        colors,
        template: raw.template.map(|value| value.trim().to_ascii_lowercase()),
        cursor_color,
    })
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

fn resolve_optional_effect(value: &str, allowed: &[&str], entropy: usize) -> Option<String> {
    match value {
        "none" => None,
        "random" => allowed
            .get(entropy % allowed.len())
            .map(|value| value.to_string()),
        other => Some(other.to_string()),
    }
}

fn invalid_cursor_config(path: &Path, field: &str, detail: String) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        "invalid_cursor_config",
        format!("Invalid Yazelix cursor config at {field}."),
        "Update ~/.config/yazelix/user_configs/yazelix_cursors.toml, then retry.",
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
    use tempfile::{TempDir, tempdir};

    fn write_registry(raw: &str) -> (TempDir, PathBuf) {
        let temp = tempdir().unwrap();
        let path = temp.path().join(USER_CURSOR_CONFIG_FILENAME);
        fs::write(&path, raw).unwrap();
        (temp, path)
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
family = "simple_dual"
colors = ["#ffb929", "#ff0000"]
cursor_color = "#ffb929"
{extra}
"##
        )
    }

    // Defends: the shipped cursor sidecar can resolve a one-item enabled list and random only draws from that list.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_resolves_random_from_enabled_cursors() {
        let (_temp, path) = write_registry(&base_registry(""));
        let registry = CursorRegistry::load(&path).unwrap();

        let resolved = registry.resolve_with_entropy(51);

        assert_eq!(resolved.selected_cursor.unwrap().name, "blaze");
        assert_eq!(resolved.selected_trail_effect, Some("tail".to_string()));
        assert_eq!(
            resolved.selected_mode_effect,
            Some("ripple_rectangle".to_string())
        );
        assert!(resolved.kitty_enable_cursor);
    }

    // Defends: disabled Kitty cursor fallback remains a first-class sidecar setting independent of Ghostty shader selection.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_parses_kitty_enable_cursor_as_binary_setting() {
        let mut raw = base_registry("");
        raw = raw.replace("kitty_enable_cursor = true", "kitty_enable_cursor = false");
        let (_temp, path) = write_registry(&raw);

        let registry = CursorRegistry::load(&path).unwrap();

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

        let error = CursorRegistry::load(&path).unwrap_err();

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
family = "simple_dual"
colors = ["#ffffff", "#cccccc"]
cursor_color = "#ffffff"
"##,
        );
        let (_temp, path) = write_registry(&raw);

        let error = CursorRegistry::load(&path).unwrap_err();

        assert_eq!(error.code(), "invalid_cursor_config");
        assert!(format!("{error:?}").contains("defined more than once"));
    }

    // Defends: color and family validation rejects invalid user-authored shader inputs before runtime files are written.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn registry_rejects_invalid_color_for_data_driven_cursor() {
        let raw = base_registry("").replace("#ff0000", "red");
        let (_temp, path) = write_registry(&raw);

        let error = CursorRegistry::load(&path).unwrap_err();

        assert_eq!(error.code(), "invalid_cursor_config");
        assert!(format!("{error:?}").contains("#rrggbb"));
    }

    // Defends: the shipped default registry parses as the active product cursor surface.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn shipped_default_registry_parses_active_cursor_surface() {
        let (_temp, path) = write_registry(include_str!("../../../yazelix_cursors_default.toml"));

        let registry = CursorRegistry::load(&path).unwrap();

        assert!(registry.enabled_cursors.contains(&"blaze".to_string()));
        assert!(registry.enabled_cursors.contains(&"neon".to_string()));
        assert!(
            registry
                .enabled_cursors
                .iter()
                .all(|name| registry.definitions.contains_key(name))
        );
        assert_eq!(registry.settings.trail, "random");
    }
}
