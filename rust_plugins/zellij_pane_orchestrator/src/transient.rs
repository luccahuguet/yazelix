use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use yazelix_pane_orchestrator::transient_pane_contract::{
    resolve_transient_toggle_plan, TransientPaneKind, TransientPaneSnapshot, TransientTogglePlan,
};
use zellij_tile::prelude::*;

use crate::{State, RESULT_INVALID_PAYLOAD, RESULT_MISSING};

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
    runtime_dir: Option<String>,
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
    fn runtime_root(&self) -> Option<PathBuf> {
        let trimmed_runtime_dir = self.runtime_dir.trim();
        if trimmed_runtime_dir.is_empty() {
            return None;
        }
        Some(PathBuf::from(trimmed_runtime_dir))
    }

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

    fn nu_launcher_path(&self) -> Option<PathBuf> {
        self.runtime_root()
            .map(|root| root.join("shells/posix/yazelix_nu.sh"))
    }

    fn yzx_cli_path(&self) -> Option<PathBuf> {
        self.runtime_root()
            .map(|root| root.join("shells/posix/yzx_cli.sh"))
    }

    fn wrapper_path(&self, kind: TransientPaneKind) -> Option<PathBuf> {
        self.runtime_root()
            .map(|root| root.join(kind.wrapper_relative_path()))
    }

    fn with_runtime_dir(&self, runtime_dir: Option<&str>) -> Self {
        let trimmed_override = runtime_dir.unwrap_or("").trim();
        if trimmed_override.is_empty() {
            return self.clone();
        }

        Self {
            runtime_dir: trimmed_override.to_owned(),
            width_percent: self.width_percent,
            height_percent: self.height_percent,
        }
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

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::TransientPaneConfig;

    // Defends: transient popup launch paths derive strictly from the configured runtime root instead of plugin-local wrapper probes.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn transient_runtime_launcher_and_wrapper_paths_are_runtime_relative() {
        let config = TransientPaneConfig {
            runtime_dir: "/runtime/root".to_owned(),
            width_percent: 90,
            height_percent: 90,
        };

        assert_eq!(
            config.nu_launcher_path().unwrap(),
            std::path::PathBuf::from("/runtime/root/shells/posix/yazelix_nu.sh")
        );
        assert_eq!(
            config.yzx_cli_path().unwrap(),
            std::path::PathBuf::from("/runtime/root/shells/posix/yzx_cli.sh")
        );
        assert_eq!(
            config
                .wrapper_path(super::TransientPaneKind::Popup)
                .unwrap(),
            std::path::PathBuf::from(
                "/runtime/root/nushell/scripts/zellij_wrappers/yzx_popup_program.nu"
            )
        );
    }
}

impl State {
    pub(crate) fn open_transient_pane(&mut self, pipe_message: &PipeMessage) {
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

    pub(crate) fn toggle_transient_pane(&mut self, pipe_message: &PipeMessage) {
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
                    runtime_dir: None,
                };
                self.open_transient_pane_with_request(active_tab_position, request, pipe_message);
            }
            TransientTogglePlan::Focus(pane_id) => {
                focus_pane_with_id(pane_id, true, false);
                self.respond(pipe_message, RESULT_FOCUSED);
            }
            TransientTogglePlan::Close(pane_id) => {
                if kind == TransientPaneKind::Popup {
                    self.refresh_sidebar_yazi_for_transient_close();
                }
                close_pane_with_id(pane_id);
                self.respond(pipe_message, RESULT_CLOSED);
            }
        }
    }

    fn open_transient_pane_with_request(
        &mut self,
        active_tab_position: usize,
        request: OpenTransientPaneRequest,
        pipe_message: &PipeMessage,
    ) {
        let transient_pane_config = self
            .transient_pane_config
            .with_runtime_dir(request.runtime_dir.as_deref());

        let Some(launcher_path) = transient_pane_config.nu_launcher_path() else {
            self.respond(pipe_message, RESULT_RUNTIME_NOT_CONFIGURED);
            return;
        };
        let Some(wrapper_path) = transient_pane_config.wrapper_path(request.kind) else {
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
            .unwrap_or_else(|| transient_pane_config.default_cwd(workspace_root));

        let mut args = vec![wrapper_path.display().to_string()];
        args.extend(request.args);

        let command_to_run = CommandToRun {
            path: launcher_path,
            args,
            cwd: Some(PathBuf::from(requested_cwd)),
        };

        // The pane orchestrator runs as a background plugin, so the "near plugin"
        // variant can hang waiting for a pane-local placement anchor that does not exist.
        let pane_id = open_command_pane_floating(
            command_to_run,
            transient_pane_config.floating_coordinates(),
            BTreeMap::new(),
        );

        if pane_id.is_some() {
            self.respond(pipe_message, RESULT_OPENED);
        } else {
            self.respond(pipe_message, RESULT_MISSING);
        }
    }

    // Trigger the same sidebar refresh contract the popup wrapper uses, but in
    // the background so toggle-close does not depend on a visible helper pane.
    fn refresh_sidebar_yazi_for_transient_close(&self) {
        let Some(launcher_path) = self.transient_pane_config.yzx_cli_path() else {
            return;
        };

        let runtime_dir = self.transient_pane_config.default_cwd(None);
        let launcher = launcher_path.display().to_string();
        let command = [launcher.as_str(), "popup", "--refresh-sidebar-only"];

        run_command_with_env_variables_and_cwd(
            &command,
            BTreeMap::new(),
            PathBuf::from(runtime_dir),
            BTreeMap::new(),
        );
    }
}
