use std::collections::{BTreeMap, HashMap};
use std::thread::sleep;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;
use zellij_tile::prelude::*;

const EDITOR_TITLE: &str = "editor";
const SIDEBAR_TITLE: &str = "sidebar";
const RESULT_OK: &str = "ok";
const RESULT_MISSING: &str = "missing";
const RESULT_NOT_READY: &str = "not_ready";
const RESULT_DENIED: &str = "permissions_denied";
const RESULT_INVALID_PAYLOAD: &str = "invalid_payload";
const RESULT_UNKNOWN_LAYOUT: &str = "unknown_layout";
const RESULT_UNSUPPORTED_EDITOR: &str = "unsupported_editor";
const COMMAND_STEP_DELAY_MS: u64 = 35;
const SWAP_LAYOUT_STEP_DELAY_MS: u64 = 1;
const SINGLE_OPEN_LAYOUT_NAME: &str = "single_open";
const SINGLE_CLOSED_LAYOUT_NAME: &str = "single_closed";
const VERTICAL_SPLIT_OPEN_LAYOUT_NAME: &str = "vertical_split_open";
const VERTICAL_SPLIT_CLOSED_LAYOUT_NAME: &str = "vertical_split_closed";
const BOTTOM_TERMINAL_OPEN_LAYOUT_NAME: &str = "bottom_terminal_open";
const BOTTOM_TERMINAL_CLOSED_LAYOUT_NAME: &str = "bottom_terminal_closed";
const LEGACY_BASIC_LAYOUT_NAME: &str = "basic";
const LEGACY_STACKED_LAYOUT_NAME: &str = "stacked";
const LEGACY_THREE_COLUMN_LAYOUT_NAME: &str = "three_column";
const LEGACY_SIDEBAR_CLOSED_LAYOUT_NAME: &str = "sidebar_closed";
const LEGACY_SINGLE_LAYOUT_NAME: &str = "single";
const LEGACY_VERTICAL_SPLIT_LAYOUT_NAME: &str = "vertical_split";
const LEGACY_BOTTOM_TERMINAL_LAYOUT_NAME: &str = "bottom_terminal";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ManagedTerminalPane {
    pane_id: PaneId,
    is_suppressed: bool,
    pane_columns: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ManagedTabPanes {
    editor: Option<ManagedTerminalPane>,
    sidebar: Option<ManagedTerminalPane>,
}

#[derive(Default)]
struct State {
    active_tab_position: Option<usize>,
    active_swap_layout_name_by_tab: HashMap<usize, Option<String>>,
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
            PermissionType::WriteToStdin,
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
                self.active_tab_position =
                    tabs.iter().find(|tab| tab.active).map(|tab| tab.position);
                self.active_swap_layout_name_by_tab = tabs
                    .into_iter()
                    .map(|tab| (tab.position, tab.active_swap_layout_name))
                    .collect();
            }
            Event::PaneUpdate(pane_manifest) => {
                self.managed_panes_by_tab = build_managed_panes_by_tab(&pane_manifest);
            }
            Event::PermissionRequestResult(status) => {
                self.permissions_granted = status == PermissionStatus::Granted;
            }
            _ => {}
        }
        false
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "focus_editor" => {
                self.focus_managed_pane(&pipe_message, ManagedPaneKind::Editor);
                false
            }
            "focus_sidebar" => {
                self.focus_managed_pane(&pipe_message, ManagedPaneKind::Sidebar);
                false
            }
            "open_file" => {
                self.open_file_in_managed_editor(&pipe_message);
                false
            }
            "next_family" => {
                self.switch_layout_family(&pipe_message, FamilyDirection::Next);
                false
            }
            "previous_family" => {
                self.switch_layout_family(&pipe_message, FamilyDirection::Previous);
                false
            }
            "toggle_sidebar" => {
                self.toggle_sidebar(&pipe_message);
                false
            }
            "debug_editor_state" => {
                self.debug_editor_state(&pipe_message);
                false
            }
            "debug_write_literal" => {
                self.debug_write_literal(&pipe_message);
                false
            }
            "debug_send_escape" => {
                self.debug_send_escape(&pipe_message);
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    fn focus_managed_pane(&self, pipe_message: &PipeMessage, pane_kind: ManagedPaneKind) {
        let Some(managed_pane) = self.get_managed_pane(pipe_message, pane_kind) else {
            return;
        };

        focus_pane_with_id(managed_pane.pane_id, false);
        self.respond(pipe_message, RESULT_OK);
    }

    fn open_file_in_managed_editor(&self, pipe_message: &PipeMessage) {
        let Some(editor_pane) = self.get_managed_pane(pipe_message, ManagedPaneKind::Editor) else {
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
            }
        };

        let command_sequence = match EditorCommandSequence::new(&open_file_request) {
            Some(command_sequence) => command_sequence,
            None => {
                self.respond(pipe_message, RESULT_UNSUPPORTED_EDITOR);
                return;
            }
        };

        focus_pane_with_id(editor_pane.pane_id, false);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![27], editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_chars_to_pane_id(&command_sequence.change_directory_command, editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![13], editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_chars_to_pane_id(&command_sequence.open_file_command, editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![13], editor_pane.pane_id);

        self.respond(pipe_message, RESULT_OK);
    }

    fn switch_layout_family(&self, pipe_message: &PipeMessage, direction: FamilyDirection) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        if is_no_sidebar_mode(self.managed_panes_by_tab.get(&active_tab_position)) {
            match direction {
                FamilyDirection::Next => self.run_next_swap_layout_steps(1),
                FamilyDirection::Previous => self.run_previous_swap_layout_steps(1),
            }
            self.respond(pipe_message, RESULT_OK);
            return;
        }

        let Some(_layout_variant) = self.get_active_layout_variant(active_tab_position) else {
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        };

        match direction {
            FamilyDirection::Next => self.run_next_swap_layout_steps(2),
            FamilyDirection::Previous => self.run_previous_swap_layout_steps(2),
        }

        self.respond(pipe_message, RESULT_OK);
    }

    fn toggle_sidebar(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        if is_no_sidebar_mode(self.managed_panes_by_tab.get(&active_tab_position)) {
            self.respond(pipe_message, RESULT_MISSING);
            return;
        }

        let Some(layout_variant) = self.get_active_layout_variant(active_tab_position) else {
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        };

        match layout_variant.sidebar_state {
            SidebarState::Open => self.run_next_swap_layout_steps(1),
            SidebarState::Closed => self.run_previous_swap_layout_steps(1),
        }

        self.respond(pipe_message, RESULT_OK);
    }

    fn ensure_action_ready(&self, pipe_message: &PipeMessage) -> Option<usize> {
        if !self.permissions_granted {
            self.respond(pipe_message, RESULT_DENIED);
            return None;
        }

        let Some(active_tab_position) = self.active_tab_position else {
            self.respond(pipe_message, RESULT_NOT_READY);
            return None;
        };

        Some(active_tab_position)
    }

    fn get_managed_pane(
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

    fn respond(&self, pipe_message: &PipeMessage, result: &str) {
        if let PipeSource::Cli(pipe_id) = &pipe_message.source {
            cli_pipe_output(pipe_id, result);
        }
    }

    fn get_active_layout_variant(&self, active_tab_position: usize) -> Option<LayoutVariant> {
        let active_swap_layout_name = self
            .active_swap_layout_name_by_tab
            .get(&active_tab_position)
            .cloned()
            .flatten();

        active_swap_layout_name
            .as_deref()
            .and_then(LayoutVariant::from_layout_name)
    }

    fn run_next_swap_layout_steps(&self, steps: usize) {
        for _ in 0..steps {
            next_swap_layout();
            sleep(Duration::from_millis(SWAP_LAYOUT_STEP_DELAY_MS));
        }
    }

    fn run_previous_swap_layout_steps(&self, steps: usize) {
        for _ in 0..steps {
            previous_swap_layout();
            sleep(Duration::from_millis(SWAP_LAYOUT_STEP_DELAY_MS));
        }
    }

    fn debug_editor_state(&self, pipe_message: &PipeMessage) {
        let active_tab_position = self.active_tab_position;
        let active_swap_layout_name = active_tab_position
            .and_then(|tab_position| self.active_swap_layout_name_by_tab.get(&tab_position))
            .cloned()
            .flatten();
        let layout_variant = active_tab_position
            .and_then(|tab_position| self.get_active_layout_variant(tab_position));
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
            editor_pane_id: pane_id_to_string(editor_pane.map(|pane| pane.pane_id)),
            sidebar_pane_id: pane_id_to_string(sidebar_pane.map(|pane| pane.pane_id)),
            sidebar_is_suppressed: sidebar_pane.map(|pane| pane.is_suppressed),
            sidebar_pane_columns: sidebar_pane.map(|pane| pane.pane_columns),
            sidebar_is_collapsed: layout_variant.map(|variant| variant.sidebar_state == SidebarState::Closed),
        };

        match serde_json::to_string(&state) {
            Ok(serialized_state) => self.respond(pipe_message, &serialized_state),
            Err(_) => self.respond(pipe_message, RESULT_INVALID_PAYLOAD),
        }
    }

    fn debug_write_literal(&self, pipe_message: &PipeMessage) {
        let Some(editor_pane) = self.get_managed_pane(pipe_message, ManagedPaneKind::Editor) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        focus_pane_with_id(editor_pane.pane_id, false);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_chars_to_pane_id(payload, editor_pane.pane_id);
        self.respond(pipe_message, RESULT_OK);
    }

    fn debug_send_escape(&self, pipe_message: &PipeMessage) {
        let Some(editor_pane) = self.get_managed_pane(pipe_message, ManagedPaneKind::Editor) else {
            return;
        };

        focus_pane_with_id(editor_pane.pane_id, false);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![27], editor_pane.pane_id);
        self.respond(pipe_message, RESULT_OK);
    }
}

#[derive(Clone, Copy, Debug)]
enum ManagedPaneKind {
    Editor,
    Sidebar,
}

#[derive(Clone, Copy, Debug)]
enum FamilyDirection {
    Next,
    Previous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayoutFamily {
    Single,
    VerticalSplit,
    BottomTerminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SidebarState {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LayoutVariant {
    family: LayoutFamily,
    sidebar_state: SidebarState,
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

#[derive(Serialize)]
struct DebugEditorState {
    permissions_granted: bool,
    active_tab_position: Option<usize>,
    active_swap_layout_name: Option<String>,
    editor_pane_id: Option<String>,
    sidebar_pane_id: Option<String>,
    sidebar_is_suppressed: Option<bool>,
    sidebar_pane_columns: Option<usize>,
    sidebar_is_collapsed: Option<bool>,
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

impl LayoutVariant {
    fn from_layout_name(layout_name: &str) -> Option<Self> {
        match layout_name {
            SINGLE_OPEN_LAYOUT_NAME
            | LEGACY_BASIC_LAYOUT_NAME
            | LEGACY_STACKED_LAYOUT_NAME
            | LEGACY_SINGLE_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            }),
            SINGLE_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Closed,
            }),
            VERTICAL_SPLIT_OPEN_LAYOUT_NAME
            | LEGACY_THREE_COLUMN_LAYOUT_NAME
            | LEGACY_VERTICAL_SPLIT_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Open,
            }),
            VERTICAL_SPLIT_CLOSED_LAYOUT_NAME
            | LEGACY_SIDEBAR_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Closed,
            }),
            BOTTOM_TERMINAL_OPEN_LAYOUT_NAME | LEGACY_BOTTOM_TERMINAL_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::BottomTerminal,
                sidebar_state: SidebarState::Open,
            }),
            BOTTOM_TERMINAL_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::BottomTerminal,
                sidebar_state: SidebarState::Closed,
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
        .or_else(|| matching_panes.iter().copied().find(|pane| !pane.is_suppressed))
        .or_else(|| matching_panes.first().copied());

    selected_pane.map(|pane| ManagedTerminalPane {
        pane_id: PaneId::Terminal(pane.id),
        is_suppressed: pane.is_suppressed,
        pane_columns: pane.pane_columns,
    })
}

fn escape_helix_path(path: &str) -> String {
    path.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_vim_single_quoted_string(path: &str) -> String {
    path.replace('\'', "''")
}

fn pane_id_to_string(pane_id: Option<PaneId>) -> Option<String> {
    match pane_id {
        Some(PaneId::Terminal(id)) => Some(format!("terminal:{id}")),
        Some(PaneId::Plugin(id)) => Some(format!("plugin:{id}")),
        None => None,
    }
}

fn is_no_sidebar_mode(managed_tab_panes: Option<&ManagedTabPanes>) -> bool {
    managed_tab_panes.and_then(|tab| tab.sidebar).is_none()
}
