use std::time::Instant;

use yazelix_pane_orchestrator::runtime_config_contract::{
    decode_runtime_config_reload, PaneOrchestratorRuntimeConfig, RuntimeConfigReloadError,
};
use zellij_tile::prelude::*;

use crate::{
    State, RESULT_DENIED, RESULT_INVALID_PAYLOAD, RESULT_NOT_READY, RESULT_OK,
    RESULT_STALE_GENERATION, RESULT_VERSION_MISMATCH,
};

impl State {
    pub(crate) fn reload_runtime_config(&mut self, pipe_message: &PipeMessage) {
        if !self.permissions_granted {
            self.respond(pipe_message, RESULT_DENIED);
            return;
        }
        if self.active_tab_position.is_none() {
            self.respond(pipe_message, RESULT_NOT_READY);
            return;
        }

        match decode_runtime_config_reload(
            pipe_message.payload.as_deref(),
            &self.runtime_config_generation,
        ) {
            Ok(runtime_config) => {
                self.apply_runtime_config(runtime_config);
                self.respond(pipe_message, RESULT_OK);
            }
            Err(RuntimeConfigReloadError::InvalidPayload) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            }
            Err(RuntimeConfigReloadError::UnsupportedVersion) => {
                self.respond(pipe_message, RESULT_VERSION_MISMATCH);
            }
            Err(RuntimeConfigReloadError::StaleGeneration) => {
                self.respond(pipe_message, RESULT_STALE_GENERATION);
            }
        }
    }

    fn apply_runtime_config(&mut self, runtime_config: PaneOrchestratorRuntimeConfig) {
        self.apply_screen_saver_runtime_config(runtime_config);
        self.arm_next_timer();
    }

    fn apply_screen_saver_runtime_config(&mut self, runtime_config: PaneOrchestratorRuntimeConfig) {
        let was_enabled = self.screen_saver_config.enabled;
        self.screen_saver_config = runtime_config.screen_saver_config();

        if self.screen_saver_config.enabled {
            if !was_enabled {
                subscribe(&[
                    EventType::InputReceived,
                    EventType::PaneClosed,
                    EventType::CommandPaneExited,
                ]);
            }
            self.screen_saver_last_input = Some(Instant::now());
            self.schedule_initial_screen_saver_timeout();
            return;
        }

        self.screen_saver_last_input = None;
        self.screen_saver_next_timeout = None;
        if let Some(pane_id) = self.screen_saver_pane_id.take() {
            close_pane_with_id(pane_id);
        }
        self.restore_screen_saver_floating_layer();
    }
}
