use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use yazelix_pane_orchestrator::transient_pane_contract::{
    resolve_transient_toggle_plan, TransientPaneKind, TransientPaneSnapshot, TransientTogglePlan,
};
use zellij_tile::prelude::*;

use crate::panes::TerminalPaneLayout;
use crate::{State, RESULT_INVALID_PAYLOAD, RESULT_MISSING, RESULT_OK};

pub(crate) const RESULT_CLOSED: &str = "closed";
pub(crate) const RESULT_FOCUSED: &str = "focused";
pub(crate) const RESULT_OPENED: &str = "opened";
pub(crate) const RESULT_RUNTIME_NOT_CONFIGURED: &str = "runtime_not_configured";

const DEFAULT_TRANSIENT_WIDTH_PERCENT: usize = 90;
const DEFAULT_TRANSIENT_HEIGHT_PERCENT: usize = 90;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TransientPaneConfig {
    pub(crate) runtime_dir: String,
    pub(crate) width_percent: usize,
    pub(crate) height_percent: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenTransientPaneRequest {
    kind: TransientPaneKind,
    #[serde(default)]
    args: Vec<String>,
    cwd: Option<String>,
}

impl Default for TransientPaneConfig {
    fn default() -> Self {
        Self {
            runtime_dir: String::new(),
            width_percent: DEFAULT_TRANSIENT_WIDTH_PERCENT,
            height_percent: DEFAULT_TRANSIENT_HEIGHT_PERCENT,
        }
    }
}

impl TransientPaneConfig {
    pub(crate) fn from_plugin_configuration(
        configuration: &BTreeMap<String, String>,
        initial_cwd: &Path,
    ) -> Self {
        let runtime_dir = configuration
            .get("runtime_dir")
            .cloned()
            .unwrap_or_else(|| initial_cwd.display().to_string());
        let width_percent = configuration
            .get("popup_width_percent")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| (1..=100).contains(value))
            .unwrap_or(DEFAULT_TRANSIENT_WIDTH_PERCENT);
        let height_percent = configuration
            .get("popup_height_percent")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| (1..=100).contains(value))
            .unwrap_or(DEFAULT_TRANSIENT_HEIGHT_PERCENT);

        Self {
            runtime_dir,
            width_percent,
            height_percent,
        }
    }

    fn floating_coordinates(&self) -> Option<FloatingPaneCoordinates> {
        let width_arg = format!("{}%", self.width_percent);
        let height_arg = format!("{}%", self.height_percent);
        let x_offset = ((100 - self.width_percent) / 2).to_string() + "%";
        let y_offset = ((100 - self.height_percent) / 2).to_string() + "%";

        FloatingPaneCoordinates::new(
            Some(x_offset),
            Some(y_offset),
            Some(width_arg),
            Some(height_arg),
            None,
            None,
        )
    }

    fn launcher_path(&self) -> Option<PathBuf> {
        let path = PathBuf::from(&self.runtime_dir).join("shells/posix/yazelix_nu.sh");
        path.exists().then_some(path)
    }

    fn wrapper_path(&self, kind: TransientPaneKind) -> Option<PathBuf> {
        let path = PathBuf::from(&self.runtime_dir).join(kind.wrapper_relative_path());
        path.exists().then_some(path)
    }

    fn default_cwd(&self, workspace_root: Option<&str>) -> String {
        let trimmed_root = workspace_root.unwrap_or("").trim();
        if trimmed_root.is_empty() {
            self.runtime_dir.clone()
        } else {
            trimmed_root.to_string()
        }
    }
}

impl State {
    pub(crate) fn open_transient_pane(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let request: OpenTransientPaneRequest = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(_) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            }
        };

        self.open_transient_pane_with_request(active_tab_position, request, pipe_message);
    }

    pub(crate) fn toggle_transient_pane(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let Some(kind) = TransientPaneKind::from_payload(payload) else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let terminal_panes = self
            .terminal_panes_by_tab
            .get(&active_tab_position)
            .cloned()
            .unwrap_or_default();
        let snapshots: Vec<TransientPaneSnapshot<'_, PaneId>> = terminal_panes
            .iter()
            .map(|pane| pane.transient_snapshot())
            .collect();

        match resolve_transient_toggle_plan(&snapshots, kind) {
            TransientTogglePlan::Open => {
                let request = OpenTransientPaneRequest {
                    kind,
                    args: vec![],
                    cwd: None,
                };
                self.open_transient_pane_with_request(active_tab_position, request, pipe_message);
            }
            TransientTogglePlan::Focus(pane_id) => {
                focus_pane_with_id(pane_id, true, false);
                self.respond(pipe_message, RESULT_FOCUSED);
            }
            TransientTogglePlan::Close(pane_id) => {
                close_pane_with_id(pane_id);
                self.respond(pipe_message, RESULT_CLOSED);
            }
        }
    }

    fn open_transient_pane_with_request(
        &self,
        active_tab_position: usize,
        request: OpenTransientPaneRequest,
        pipe_message: &PipeMessage,
    ) {
        let Some(launcher_path) = self.transient_pane_config.launcher_path() else {
            self.respond(pipe_message, RESULT_RUNTIME_NOT_CONFIGURED);
            return;
        };
        let Some(wrapper_path) = self.transient_pane_config.wrapper_path(request.kind) else {
            self.respond(pipe_message, RESULT_RUNTIME_NOT_CONFIGURED);
            return;
        };

        let workspace_root = self
            .workspace_state_by_tab
            .get(&active_tab_position)
            .map(|state| state.root.as_str());
        let requested_cwd = request
            .cwd
            .as_deref()
            .map(str::trim)
            .filter(|cwd| !cwd.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| self.transient_pane_config.default_cwd(workspace_root));

        let mut args = vec![wrapper_path.display().to_string()];
        args.extend(request.args);

        let command_to_run = CommandToRun {
            path: launcher_path,
            args,
            cwd: Some(PathBuf::from(requested_cwd)),
        };

        let pane_id = open_command_pane_floating_near_plugin(
            command_to_run,
            self.transient_pane_config.floating_coordinates(),
            BTreeMap::new(),
        );

        if pane_id.is_some() {
            self.respond(pipe_message, RESULT_OPENED);
        } else {
            self.respond(pipe_message, RESULT_MISSING);
        }
    }
}
