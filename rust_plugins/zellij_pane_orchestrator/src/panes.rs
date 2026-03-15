use std::collections::HashMap;

use serde::Serialize;
use zellij_tile::prelude::*;

use crate::{State, RESULT_INVALID_PAYLOAD, RESULT_MISSING, RESULT_OK};

pub(crate) const EDITOR_TITLE: &str = "editor";
pub(crate) const SIDEBAR_TITLE: &str = "sidebar";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ManagedTerminalPane {
    pub(crate) pane_id: PaneId,
    pub(crate) is_suppressed: bool,
    pub(crate) pane_columns: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ManagedTabPanes {
    pub(crate) editor: Option<ManagedTerminalPane>,
    pub(crate) sidebar: Option<ManagedTerminalPane>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ManagedPaneKind {
    Editor,
    Sidebar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FocusContext {
    Editor,
    Sidebar,
    Other,
}

#[derive(Serialize)]
struct DebugEditorState {
    permissions_granted: bool,
    active_tab_position: Option<usize>,
    active_swap_layout_name: Option<String>,
    workspace_root: Option<String>,
    editor_pane_id: Option<String>,
    sidebar_pane_id: Option<String>,
    sidebar_is_suppressed: Option<bool>,
    sidebar_pane_columns: Option<usize>,
    sidebar_is_collapsed: Option<bool>,
}

pub(crate) fn build_managed_panes_by_tab(
    pane_manifest: &PaneManifest,
) -> HashMap<usize, ManagedTabPanes> {
    let mut managed_panes_by_tab = HashMap::new();

    for (tab_position, panes) in &pane_manifest.panes {
        managed_panes_by_tab.insert(
            *tab_position,
            ManagedTabPanes {
                editor: select_managed_terminal_pane(panes, EDITOR_TITLE),
                sidebar: select_managed_terminal_pane(panes, SIDEBAR_TITLE),
            },
        );
    }

    managed_panes_by_tab
}

pub(crate) fn build_user_pane_count_by_tab(pane_manifest: &PaneManifest) -> HashMap<usize, usize> {
    pane_manifest
        .panes
        .iter()
        .map(|(tab_position, panes)| {
            let user_pane_count = panes
                .iter()
                .filter(|pane| !pane.is_plugin)
                .filter(|pane| !pane.exited)
                .count();
            (*tab_position, user_pane_count)
        })
        .collect()
}

pub(crate) fn build_focus_context_by_tab(
    pane_manifest: &PaneManifest,
    previous_focus_context_by_tab: &HashMap<usize, FocusContext>,
) -> HashMap<usize, FocusContext> {
    let mut focus_context_by_tab = HashMap::new();

    for (tab_position, panes) in &pane_manifest.panes {
        let focused_pane = panes.iter().find(|pane| pane.is_focused && !pane.is_plugin);
        let focus_context = match focused_pane.map(|pane| pane.title.trim()) {
            Some(EDITOR_TITLE) => FocusContext::Editor,
            Some(SIDEBAR_TITLE) => FocusContext::Sidebar,
            Some(title) if title.starts_with("yzx_") => previous_focus_context_by_tab
                .get(tab_position)
                .copied()
                .unwrap_or(FocusContext::Other),
            Some(_) | None => FocusContext::Other,
        };
        focus_context_by_tab.insert(*tab_position, focus_context);
    }

    focus_context_by_tab
}

pub(crate) fn build_focused_terminal_pane_by_tab(
    pane_manifest: &PaneManifest,
) -> HashMap<usize, PaneId> {
    pane_manifest
        .panes
        .iter()
        .filter_map(|(tab_position, panes)| {
            panes
                .iter()
                .find(|pane| pane.is_focused && !pane.is_plugin && !pane.exited)
                .map(|pane| (*tab_position, PaneId::Terminal(pane.id)))
        })
        .collect()
}

impl State {
    pub(crate) fn smart_reveal(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let focus_context = self
            .focus_context_by_tab
            .get(&active_tab_position)
            .copied()
            .unwrap_or(FocusContext::Other);

        if focus_context == FocusContext::Editor {
            let Some(editor_pane) = self.get_managed_pane(pipe_message, ManagedPaneKind::Editor)
            else {
                return;
            };

            write_to_pane_id(vec![27, b'r'], editor_pane.pane_id);
            self.respond(pipe_message, RESULT_OK);
            return;
        }

        self.toggle_editor_sidebar_focus(pipe_message);
    }

    pub(crate) fn focus_managed_pane(
        &self,
        pipe_message: &PipeMessage,
        pane_kind: ManagedPaneKind,
    ) {
        let Some(managed_pane) = self.get_managed_pane(pipe_message, pane_kind) else {
            return;
        };

        focus_pane_with_id(managed_pane.pane_id, false);
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn toggle_editor_sidebar_focus(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        let Some(managed_tab_panes) = self.managed_panes_by_tab.get(&active_tab_position) else {
            self.respond(pipe_message, RESULT_MISSING);
            return;
        };

        let target_pane = if self
            .focus_context_by_tab
            .get(&active_tab_position)
            .copied()
            .unwrap_or(FocusContext::Other)
            == FocusContext::Sidebar
        {
            managed_tab_panes.editor
        } else {
            managed_tab_panes.sidebar
        };

        let Some(target_pane) = target_pane else {
            self.respond(pipe_message, RESULT_MISSING);
            return;
        };

        focus_pane_with_id(target_pane.pane_id, false);
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn debug_editor_state(&self, pipe_message: &PipeMessage) {
        let active_tab_position = self.active_tab_position;
        let active_swap_layout_name = active_tab_position
            .and_then(|tab_position| self.active_swap_layout_name_by_tab.get(&tab_position))
            .cloned()
            .flatten();
        let layout_variant = active_tab_position
            .and_then(|tab_position| self.get_active_layout_variant(tab_position));
        let workspace_root = active_tab_position
            .and_then(|tab_position| self.workspace_state_by_tab.get(&tab_position))
            .map(|workspace_state| workspace_state.root.clone())
            .or_else(|| self.initial_workspace_state.clone().map(|state| state.root));
        let editor_pane = active_tab_position
            .and_then(|tab_position| self.managed_panes_by_tab.get(&tab_position))
            .and_then(|managed_tab_panes| managed_tab_panes.editor);
        let sidebar_pane = active_tab_position
            .and_then(|tab_position| self.managed_panes_by_tab.get(&tab_position))
            .and_then(|managed_tab_panes| managed_tab_panes.sidebar);

        let state = DebugEditorState {
            permissions_granted: self.permissions_granted,
            active_tab_position,
            active_swap_layout_name,
            workspace_root,
            editor_pane_id: pane_id_to_string(editor_pane.map(|pane| pane.pane_id)),
            sidebar_pane_id: pane_id_to_string(sidebar_pane.map(|pane| pane.pane_id)),
            sidebar_is_suppressed: sidebar_pane.map(|pane| pane.is_suppressed),
            sidebar_pane_columns: sidebar_pane.map(|pane| pane.pane_columns),
            sidebar_is_collapsed: layout_variant.map(|variant| variant.is_sidebar_closed()),
        };

        match serde_json::to_string(&state) {
            Ok(serialized_state) => self.respond(pipe_message, &serialized_state),
            Err(_) => self.respond(pipe_message, RESULT_INVALID_PAYLOAD),
        }
    }

    pub(crate) fn get_managed_pane(
        &self,
        pipe_message: &PipeMessage,
        pane_kind: ManagedPaneKind,
    ) -> Option<ManagedTerminalPane> {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return None;
        };

        let managed_pane = self
            .managed_panes_by_tab
            .get(&active_tab_position)
            .and_then(|managed_tab_panes| match pane_kind {
                ManagedPaneKind::Editor => managed_tab_panes.editor,
                ManagedPaneKind::Sidebar => managed_tab_panes.sidebar,
            });

        match managed_pane {
            Some(managed_pane) => Some(managed_pane),
            None => {
                self.respond(pipe_message, RESULT_MISSING);
                None
            }
        }
    }

    pub(crate) fn get_focused_terminal_pane(&self, pipe_message: &PipeMessage) -> Option<PaneId> {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return None;
        };

        match self
            .focused_terminal_pane_by_tab
            .get(&active_tab_position)
            .copied()
        {
            Some(pane_id) => Some(pane_id),
            None => {
                self.respond(pipe_message, RESULT_MISSING);
                None
            }
        }
    }
}

fn select_managed_terminal_pane(
    panes: &[PaneInfo],
    expected_title: &str,
) -> Option<ManagedTerminalPane> {
    let matching_panes: Vec<&PaneInfo> = panes
        .iter()
        .filter(|pane| !pane.is_plugin)
        .filter(|pane| !pane.exited)
        .filter(|pane| pane.title.trim() == expected_title)
        .collect();

    let selected_pane = matching_panes
        .iter()
        .copied()
        .find(|pane| pane.is_focused)
        .or_else(|| {
            matching_panes
                .iter()
                .copied()
                .find(|pane| !pane.is_suppressed)
        })
        .or_else(|| matching_panes.first().copied());

    selected_pane.map(|pane| ManagedTerminalPane {
        pane_id: PaneId::Terminal(pane.id),
        is_suppressed: pane.is_suppressed,
        pane_columns: pane.pane_columns,
    })
}

fn pane_id_to_string(pane_id: Option<PaneId>) -> Option<String> {
    match pane_id {
        Some(PaneId::Terminal(id)) => Some(format!("terminal:{id}")),
        Some(PaneId::Plugin(id)) => Some(format!("plugin:{id}")),
        None => None,
    }
}
