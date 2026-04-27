use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenSaverConfig {
    pub enabled: bool,
    pub idle_seconds: u64,
    pub style: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScreenSaverTimerPlan {
    Disabled,
    Wait(Duration),
    Open { style: String },
}

impl Default for ScreenSaverConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            idle_seconds: 300,
            style: "random".to_string(),
        }
    }
}

impl ScreenSaverConfig {
    pub fn from_plugin_configuration(configuration: &BTreeMap<String, String>) -> Self {
        let default = Self::default();
        let enabled = bool_config(configuration, "screen_saver_enabled", default.enabled);
        let idle_seconds = configuration
            .get("screen_saver_idle_seconds")
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(default.idle_seconds);
        let style = configuration
            .get("screen_saver_style")
            .map(|raw| raw.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty())
            .unwrap_or(default.style);

        Self {
            enabled,
            idle_seconds,
            style,
        }
    }
}

fn bool_config(configuration: &BTreeMap<String, String>, key: &str, default: bool) -> bool {
    match configuration.get(key).map(|raw| raw.trim()) {
        Some("true") => true,
        Some("false") => false,
        _ => default,
    }
}

pub fn resolve_screen_saver_timer_plan(
    config: &ScreenSaverConfig,
    idle_elapsed: Duration,
    screen_is_open: bool,
) -> ScreenSaverTimerPlan {
    if !config.enabled || screen_is_open {
        return ScreenSaverTimerPlan::Disabled;
    }

    let threshold = Duration::from_secs(config.idle_seconds);
    if idle_elapsed >= threshold {
        ScreenSaverTimerPlan::Open {
            style: config.style.clone(),
        }
    } else {
        ScreenSaverTimerPlan::Wait(threshold - idle_elapsed)
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::{resolve_screen_saver_timer_plan, ScreenSaverConfig, ScreenSaverTimerPlan};
    use std::collections::BTreeMap;
    use std::time::Duration;

    // Defends: idle screen saver config is opt-in, so loading the pane orchestrator cannot open `yzx screen` by default.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn screen_saver_is_disabled_by_default() {
        let config = ScreenSaverConfig::from_plugin_configuration(&BTreeMap::new());

        assert_eq!(config, ScreenSaverConfig::default());
        assert_eq!(
            resolve_screen_saver_timer_plan(&config, Duration::from_secs(600), false),
            ScreenSaverTimerPlan::Disabled
        );
    }

    // Defends: opt-in idle policy opens the configured `yzx screen` style only after the configured threshold.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn screen_saver_waits_until_idle_threshold_then_opens_configured_style() {
        let mut raw = BTreeMap::new();
        raw.insert("screen_saver_enabled".to_string(), "true".to_string());
        raw.insert("screen_saver_idle_seconds".to_string(), "120".to_string());
        raw.insert("screen_saver_style".to_string(), "mandelbrot".to_string());
        let config = ScreenSaverConfig::from_plugin_configuration(&raw);

        assert_eq!(
            resolve_screen_saver_timer_plan(&config, Duration::from_secs(90), false),
            ScreenSaverTimerPlan::Wait(Duration::from_secs(30))
        );
        assert_eq!(
            resolve_screen_saver_timer_plan(&config, Duration::from_secs(120), false),
            ScreenSaverTimerPlan::Open {
                style: "mandelbrot".to_string()
            }
        );
    }
}
