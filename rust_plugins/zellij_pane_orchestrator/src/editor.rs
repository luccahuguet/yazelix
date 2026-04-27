use std::thread::sleep;
use std::time::Duration;

use serde::Deserialize;
use zellij_tile::prelude::*;

use crate::panes::ManagedPaneKind;
use crate::{
    State, COMMAND_STEP_DELAY_MS, RESULT_INVALID_PAYLOAD, RESULT_OK, RESULT_UNSUPPORTED_EDITOR,
};
use yazelix_pane_orchestrator::editor_open_contract::{
    build_editor_change_directory_command, build_editor_command_sequence,
    normalize_open_file_targets, EditorCommandSequenceError,
};

#[derive(Deserialize)]
struct OpenFileRequest {
    editor: String,
    #[serde(default)]
    file_path: Option<String>,
    #[serde(default)]
    file_paths: Vec<String>,
    working_dir: String,
}

#[derive(Deserialize)]
struct EditorCwdRequest {
    editor: String,
    working_dir: String,
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

        let file_paths = open_file_request.target_file_paths();
        let command_sequence = match build_editor_command_sequence(
            &open_file_request.editor,
            &open_file_request.working_dir,
            &file_paths,
        ) {
            Ok(command_sequence) => command_sequence,
            Err(EditorCommandSequenceError::EmptyTargets) => {
                self.respond(pipe_message, RESULT_INVALID_PAYLOAD);
                return;
            }
            Err(EditorCommandSequenceError::UnsupportedEditor) => {
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
        for open_file_command in command_sequence.open_file_commands {
            write_chars_to_pane_id(&open_file_command, editor_pane.pane_id);
            sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
            write_to_pane_id(vec![13], editor_pane.pane_id);
            sleep(Duration::from_millis(COMMAND_STEP_DELAY_MS));
        }

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

impl OpenFileRequest {
    fn target_file_paths(&self) -> Vec<&str> {
        normalize_open_file_targets(self.file_path.as_deref(), &self.file_paths)
    }
}
