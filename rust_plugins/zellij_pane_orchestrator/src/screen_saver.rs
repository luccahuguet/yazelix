use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use yazelix_pane_orchestrator::screen_saver_contract::{
    resolve_screen_saver_timer_plan, ScreenSaverTimerPlan,
};
use zellij_tile::prelude::*;

use crate::State;

const SCREEN_SAVER_PANE_TITLE: &str = "yzx_screen";
const MINIMUM_RESCHEDULE_SECONDS: f64 = 0.5;

fn full_tab_coordinates() -> Option<FloatingPaneCoordinates> {
    FloatingPaneCoordinates::new(
        Some("0%".to_string()),
        Some("0%".to_string()),
        Some("100%".to_string()),
        Some("100%".to_string()),
        None,
        None,
    )
}

impl State {
    pub(crate) fn schedule_initial_screen_saver_timeout(&self) {
        if self.screen_saver_config.enabled {
            self.schedule_screen_saver_timeout(Duration::from_secs(
                self.screen_saver_config.idle_seconds,
            ));
        }
    }

    pub(crate) fn record_screen_saver_input(&mut self) {
        if !self.screen_saver_config.enabled {
            return;
        }

        self.screen_saver_last_input = Some(Instant::now());
        if let Some(pane_id) = self.screen_saver_pane_id.take() {
            close_pane_with_id(pane_id);
            self.schedule_initial_screen_saver_timeout();
        }
    }

    pub(crate) fn handle_screen_saver_timer(&mut self) {
        if !self.screen_saver_config.enabled {
            return;
        }

        let now = Instant::now();
        let last_input = self.screen_saver_last_input.unwrap_or(now);
        let idle_elapsed = now.saturating_duration_since(last_input);
        match resolve_screen_saver_timer_plan(
            &self.screen_saver_config,
            idle_elapsed,
            self.screen_saver_pane_id.is_some(),
        ) {
            ScreenSaverTimerPlan::Disabled => {}
            ScreenSaverTimerPlan::Wait(delay) => self.schedule_screen_saver_timeout(delay),
            ScreenSaverTimerPlan::Open { style } => self.open_screen_saver_pane(&style),
        }
    }

    pub(crate) fn handle_screen_saver_pane_closed(&mut self, pane_id: PaneId) {
        if self.screen_saver_pane_id == Some(pane_id) {
            self.screen_saver_pane_id = None;
            self.screen_saver_last_input = Some(Instant::now());
            self.schedule_initial_screen_saver_timeout();
        }
    }

    pub(crate) fn handle_screen_saver_command_exit(&mut self, terminal_id: u32) {
        self.handle_screen_saver_pane_closed(PaneId::Terminal(terminal_id));
    }

    fn schedule_screen_saver_timeout(&self, delay: Duration) {
        if !self.screen_saver_config.enabled {
            return;
        }

        set_timeout(delay.as_secs_f64().max(MINIMUM_RESCHEDULE_SECONDS));
    }

    fn open_screen_saver_pane(&mut self, style: &str) {
        if !self.permissions_granted || self.screen_saver_pane_id.is_some() {
            self.schedule_initial_screen_saver_timeout();
            return;
        }

        let Some(launcher_path) = self.transient_pane_config.yzx_cli_path() else {
            self.schedule_initial_screen_saver_timeout();
            return;
        };

        let workspace_root = self
            .active_tab_position
            .and_then(|tab_position| self.workspace_state_by_tab.get(&tab_position))
            .map(|state| state.root.as_str());
        let command_to_run = CommandToRun {
            path: launcher_path,
            args: vec!["screen".to_string(), style.to_string()],
            cwd: Some(PathBuf::from(
                self.transient_pane_config.default_cwd(workspace_root),
            )),
        };

        let pane_id =
            open_command_pane_floating(command_to_run, full_tab_coordinates(), BTreeMap::new());
        if let Some(pane_id) = pane_id {
            rename_pane_with_id(pane_id, SCREEN_SAVER_PANE_TITLE);
            focus_pane_with_id(pane_id, true, false);
            self.screen_saver_pane_id = Some(pane_id);
        } else {
            self.schedule_initial_screen_saver_timeout();
        }
    }
}
