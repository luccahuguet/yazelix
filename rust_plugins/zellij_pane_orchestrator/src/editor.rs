use std::thread::sleep;
use std::time::Duration;

use serde::Deserialize;
use zellij_tile::prelude::*;

use crate::panes::ManagedPaneKind;
use crate::{
    State, COMMAND_STEP_DELAY_MS, RESULT_INVALID_PAYLOAD, RESULT_OK, RESULT_UNSUPPORTED_EDITOR,
};

#[derive(Deserialize)]
struct OpenFileRequest {
    editor: String,
    file_path: String,
    working_dir: String,
}

#[derive(Deserialize)]
struct EditorCwdRequest {
    editor: String,
    working_dir: String,
}

struct EditorCommandSequence {
    change_directory_command: String,
    open_file_command: String,
}

impl State {
    pub(crate) fn set_managed_editor_cwd(&self, pipe_message: &PipeMessage) {
        let Some(editor_pane) = self.get_managed_pane(pipe_message, ManagedPaneKind::Editor) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        let editor_cwd_request: EditorCwdRequest = match serde_json::from_str(payload) {
            Ok(request) => request,
            Err(_) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            }
        };

        let Some(change_directory_command) = build_editor_change_directory_command(
            &editor_cwd_request.editor,
            &editor_cwd_request.working_dir,
        ) else {
            self.respond(pipe_message, RESULT_UNSUPPORTED_EDITOR);
            return;
        };

        write_to_pane_id(vec![27], editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_chars_to_pane_id(&change_directory_command, editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![13], editor_pane.pane_id);

        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn open_file_in_managed_editor(&self, pipe_message: &PipeMessage) {
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

        focus_pane_with_id(editor_pane.pane_id, false, false);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![27], editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_chars_to_pane_id(
            &command_sequence.change_directory_command,
            editor_pane.pane_id,
        );
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![13], editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_chars_to_pane_id(&command_sequence.open_file_command, editor_pane.pane_id);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![13], editor_pane.pane_id);

        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn debug_write_literal(&self, pipe_message: &PipeMessage) {
        let Some(editor_pane) = self.get_managed_pane(pipe_message, ManagedPaneKind::Editor) else {
            return;
        };

        let Some(payload) = pipe_message.payload.as_deref() else {
            self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
            return;
        };

        focus_pane_with_id(editor_pane.pane_id, false, false);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_chars_to_pane_id(payload, editor_pane.pane_id);
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn debug_send_escape(&self, pipe_message: &PipeMessage) {
        let Some(editor_pane) = self.get_managed_pane(pipe_message, ManagedPaneKind::Editor) else {
            return;
        };

        focus_pane_with_id(editor_pane.pane_id, false, false);
        sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        write_to_pane_id(vec![27], editor_pane.pane_id);
        self.respond(pipe_message, RESULT_OK);
    }
}

impl EditorCommandSequence {
    fn new(open_file_request: &OpenFileRequest) -> Option<Self> {
        let change_directory_command = build_editor_change_directory_command(
            &open_file_request.editor,
            &open_file_request.working_dir,
        )?;

        match open_file_request.editor.as_str() {
            "helix" => Some(Self {
                change_directory_command,
                open_file_command: format!(
                    ":open \"{}\"",
                    escape_helix_path(&open_file_request.file_path)
                ),
            }),
            "neovim" => Some(Self {
                change_directory_command,
                open_file_command: format!(
                    ":execute 'edit ' . fnameescape('{}')",
                    escape_vim_single_quoted_string(&open_file_request.file_path)
                ),
            }),
            _ => None,
        }
    }
}

fn build_editor_change_directory_command(editor: &str, working_dir: &str) -> Option<String> {
    match editor {
        "helix" => Some(format!(":cd \"{}\"", escape_helix_path(working_dir))),
        "neovim" => Some(format!(
            ":execute 'cd ' . fnameescape('{}')",
            escape_vim_single_quoted_string(working_dir)
        )),
        _ => None,
    }
}

fn escape_helix_path(path: &str) -> String {
    path.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_vim_single_quoted_string(path: &str) -> String {
    path.replace('\'', "''")
}
