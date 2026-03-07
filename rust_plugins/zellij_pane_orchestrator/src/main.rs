use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use zellij_tile::prelude::*;

const EDITOR_TITLE: &str = "editor";
const SIDEBAR_TITLE: &str = "sidebar";
const RESULT_OK: &str = "ok";
const RESULT_MISSING: &str = "missing";
const RESULT_NOT_READY: &str = "not_ready";
const RESULT_DENIED: &str = "permissions_denied";
const RESULT_INVALID_PAYLOAD: &str = "invalid_payload";
const RESULT_UNSUPPORTED_EDITOR: &str = "unsupported_editor";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ManagedTabPanes {
    editor: Option<PaneId>,
    sidebar: Option<PaneId>,
}

#[derive(Default)]
struct State {
    active_tab_position: Option<usize>,
    managed_panes_by_tab: HashMap<usize, ManagedTabPanes>,
    permissions_granted: bool,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        set_selectable(false);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::ReadCliPipes,
        ]);
        subscribe(&[
            EventType::TabUpdate,
            EventType::PaneUpdate,
            EventType::PermissionRequestResult,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::TabUpdate(tabs) => {
                self.active_tab_position = tabs.iter().find(|tab| tab.active).map(|tab| tab.position);
            },
            Event::PaneUpdate(pane_manifest) => {
                self.managed_panes_by_tab = build_managed_panes_by_tab(&pane_manifest);
            },
            Event::PermissionRequestResult(status) => {
                self.permissions_granted = status == PermissionStatus::Granted;
            },
            _ => {},
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "focus_editor" => {
                self.focus_managed_pane(&pipe_message, ManagedPaneKind::Editor);
                false
            },
            "focus_sidebar" => {
                self.focus_managed_pane(&pipe_message, ManagedPaneKind::Sidebar);
                false
            },
            "open_file" => {
                self.open_file_in_managed_editor(&pipe_message);
                false
            },
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    fn focus_managed_pane(&self, pipe_message: &PipeMessage, pane_kind: ManagedPaneKind) {
        let Some(pane_id) = self.get_managed_pane_id(pipe_message, pane_kind) else {
            return;
        };

        focus_pane_with_id(pane_id, false);
        self.respond(pipe_message, RESULT_OK);
    }

    fn open_file_in_managed_editor(&self, pipe_message: &PipeMessage) {
        let Some(pane_id) = self.get_managed_pane_id(pipe_message, ManagedPaneKind::Editor) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let open_file_request: OpenFileRequest = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(_) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            },
        };

        let command_sequence = match EditorCommandSequence::new(&open_file_request) {
            Some(command_sequence) => command_sequence,
            None => {
                self.respond(pipe_message, RESULT_UNSUPPORTED_EDITOR);
                return;
            },
        };

        focus_pane_with_id(pane_id, false);
        write_to_pane_id(vec![27], pane_id);
        write_chars_to_pane_id(&command_sequence.change_directory_command, pane_id);
        write_to_pane_id(vec![13], pane_id);
        write_chars_to_pane_id(&command_sequence.open_file_command, pane_id);
        write_to_pane_id(vec![13], pane_id);

        self.respond(pipe_message, RESULT_OK);
    }

    fn get_managed_pane_id(
        &self,
        pipe_message: &PipeMessage,
        pane_kind: ManagedPaneKind,
    ) -> Option<PaneId> {
        if !self.permissions_granted {
            self.respond(pipe_message, RESULT_DENIED);
            return None;
        }

        let Some(active_tab_position) = self.active_tab_position else {
            self.respond(pipe_message, RESULT_NOT_READY);
            return None;
        };

        let pane_id = self
            .managed_panes_by_tab
            .get(&active_tab_position)
            .and_then(|managed_tab_panes| match pane_kind {
                ManagedPaneKind::Editor => managed_tab_panes.editor,
                ManagedPaneKind::Sidebar => managed_tab_panes.sidebar,
            });

        match pane_id {
            Some(pane_id) => Some(pane_id),
            None => {
                self.respond(pipe_message, RESULT_MISSING);
                None
            },
        }
    }

    fn respond(&self, pipe_message: &PipeMessage, result: &str) {
        if let PipeSource::Cli(pipe_id) = &pipe_message.source {
            cli_pipe_output(pipe_id, result);
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ManagedPaneKind {
    Editor,
    Sidebar,
}

#[derive(Deserialize)]
struct OpenFileRequest {
    editor: String,
    file_path: String,
    working_dir: String,
}

struct EditorCommandSequence {
    change_directory_command: String,
    open_file_command: String,
}

impl EditorCommandSequence {
    fn new(open_file_request: &OpenFileRequest) -> Option<Self> {
        match open_file_request.editor.as_str() {
            "helix" => Some(Self {
                change_directory_command: format!(
                    ":cd \"{}\"",
                    escape_helix_path(&open_file_request.working_dir)
                ),
                open_file_command: format!(
                    ":open \"{}\"",
                    escape_helix_path(&open_file_request.file_path)
                ),
            }),
            "neovim" => Some(Self {
                change_directory_command: format!(
                    ":execute 'cd ' . fnameescape('{}')",
                    escape_vim_single_quoted_string(&open_file_request.working_dir)
                ),
                open_file_command: format!(
                    ":execute 'edit ' . fnameescape('{}')",
                    escape_vim_single_quoted_string(&open_file_request.file_path)
                ),
            }),
            _ => None,
        }
    }
}

fn build_managed_panes_by_tab(pane_manifest: &PaneManifest) -> HashMap<usize, ManagedTabPanes> {
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

fn select_managed_terminal_pane(panes: &[PaneInfo], expected_title: &str) -> Option<PaneId> {
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
        .or_else(|| matching_panes.iter().copied().find(|pane| !pane.is_suppressed))
        .or_else(|| matching_panes.first().copied());

    selected_pane.map(|pane| PaneId::Terminal(pane.id))
}

fn escape_helix_path(path: &str) -> String {
    path.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_vim_single_quoted_string(path: &str) -> String {
    path.replace('\'', "''")
}
