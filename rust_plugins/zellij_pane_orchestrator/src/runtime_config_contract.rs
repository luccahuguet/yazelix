// Test lane: default

use serde::{Deserialize, Serialize};

use crate::screen_saver_contract::ScreenSaverConfig;

pub const RUNTIME_CONFIG_RELOAD_SCHEMA_VERSION: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaneOrchestratorRuntimeConfig {
    pub screen_saver_enabled: bool,
    pub screen_saver_idle_seconds: u64,
    pub screen_saver_style: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeConfigReloadRequest {
    schema_version: u64,
    generation: String,
    runtime_config: PaneOrchestratorRuntimeConfig,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeConfigReloadError {
    InvalidPayload,
    UnsupportedVersion,
    StaleGeneration,
}

impl PaneOrchestratorRuntimeConfig {
    pub fn screen_saver_config(&self) -> ScreenSaverConfig {
        ScreenSaverConfig {
            enabled: self.screen_saver_enabled,
            idle_seconds: self.screen_saver_idle_seconds,
            style: self.screen_saver_style.clone(),
        }
    }
}

pub fn decode_runtime_config_reload(
    payload: Option<&str>,
    active_generation: &str,
) -> Result<PaneOrchestratorRuntimeConfig, RuntimeConfigReloadError> {
    let payload = payload.ok_or(RuntimeConfigReloadError::InvalidPayload)?;
    let request = serde_json::from_str::<RuntimeConfigReloadRequest>(payload)
        .map_err(|_| RuntimeConfigReloadError::InvalidPayload)?;
    if request.schema_version != RUNTIME_CONFIG_RELOAD_SCHEMA_VERSION {
        return Err(RuntimeConfigReloadError::UnsupportedVersion);
    }
    if request.generation.trim() != active_generation.trim() {
        return Err(RuntimeConfigReloadError::StaleGeneration);
    }
    Ok(request.runtime_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: pane-orchestrator live reload accepts only the current versioned payload for the generated config generation active in the plugin.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn decodes_runtime_config_reload_for_active_generation() {
        let payload = r#"{"schema_version":1,"generation":"gen-a","runtime_config":{"screen_saver_enabled":true,"screen_saver_idle_seconds":120,"screen_saver_style":"mandelbrot"}}"#;

        let config = decode_runtime_config_reload(Some(payload), "gen-a").unwrap();

        assert_eq!(
            config.screen_saver_config(),
            ScreenSaverConfig {
                enabled: true,
                idle_seconds: 120,
                style: "mandelbrot".to_string()
            }
        );
    }

    // Defends: old or future control helpers cannot silently mutate pane-orchestrator runtime state with an unsupported reload schema.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn rejects_unsupported_runtime_config_reload_version() {
        let payload = r#"{"schema_version":99,"generation":"gen-a","runtime_config":{"screen_saver_enabled":true,"screen_saver_idle_seconds":120,"screen_saver_style":"mandelbrot"}}"#;

        assert_eq!(
            decode_runtime_config_reload(Some(payload), "gen-a"),
            Err(RuntimeConfigReloadError::UnsupportedVersion)
        );
    }

    // Regression: reload requests generated against a different config generation remain pending instead of being applied to the wrong running plugin.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn rejects_stale_runtime_config_generation() {
        let payload = r#"{"schema_version":1,"generation":"gen-new","runtime_config":{"screen_saver_enabled":true,"screen_saver_idle_seconds":120,"screen_saver_style":"mandelbrot"}}"#;

        assert_eq!(
            decode_runtime_config_reload(Some(payload), "gen-old"),
            Err(RuntimeConfigReloadError::StaleGeneration)
        );
    }
}
