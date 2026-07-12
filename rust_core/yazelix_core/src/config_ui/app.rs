use super::*;
use crossterm::{
    cursor, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratconfig::toml_adapter::{TomlPatchError, set_toml_value_text, unset_toml_value_text};

pub fn run_config_ui(request: ConfigUiRequest) -> Result<i32, CoreError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "config_ui_requires_terminal",
            "`yzx config ui` requires an interactive terminal.",
            "Run `yzx config` for plain text output, or retry from an interactive shell.",
            json!({}),
        ));
    }

    let model = build_config_ui_model(&request)?;
    let mut ui = ConfigUiApp::new(model);
    let host = YazelixConfigUiHost { request: &request };
    run_ratconfig_config_ui_with_details(&mut ui, render_details, |ui, intent| {
        host.handle_ratconfig_intent(ui, intent);
        Ok::<(), CoreError>(())
    })
    .map_err(ratconfig_runner_err)?;
    Ok(0)
}

struct YazelixConfigUiHost<'a> {
    request: &'a ConfigUiRequest,
}

#[cfg(test)]
impl YazelixConfigUiApp {
    pub(crate) fn new(request: ConfigUiRequest, model: ConfigUiModel) -> Self {
        Self {
            request,
            ui: ConfigUiApp::new(model),
        }
    }

    pub(super) fn write_source_field_value(
        &mut self,
        source_id: &str,
        setting_path: &str,
        value: &JsonValue,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let host = YazelixConfigUiHost {
            request: &self.request,
        };
        host.write_source_field_value(&mut self.ui, source_id, setting_path, value)
    }
}

impl YazelixConfigUiHost<'_> {
    fn handle_ratconfig_intent(&self, ui: &mut ConfigUiApp, intent: ConfigUiIntent) {
        match intent {
            ConfigUiIntent::None | ConfigUiIntent::Exit => {}
            ConfigUiIntent::BeginEdit {
                field_index,
                source_id,
                path,
            } => match self.editable_config_target(ui, &source_id, &path) {
                Ok(_) => ui.begin_edit_field(field_index),
                Err(error) => ui.notice_error(error.message()),
            },
            ConfigUiIntent::OpenFile {
                source_id,
                action_id,
                path,
                ..
            } => match self.open_config_file(ui, &source_id, &action_id, &path) {
                Ok(()) => ui.notice_info(format!("Opened {}.", path.display())),
                Err(error) => ui.notice_error(error.message()),
            },
            ConfigUiIntent::EditTextExternally { path, .. } => ui.notice_error(format!(
                "The Ratconfig external editor action for {path} has no Yazelix host handler."
            )),
            ConfigUiIntent::SetField {
                source_id,
                path,
                value,
                ..
            } => {
                self.set_field_value(ui, &source_id, &path, value);
                ui.finish_successful_write();
            }
            ConfigUiIntent::UnsetField {
                source_id, path, ..
            } => match self.unset_field_value(ui, &source_id, &path) {
                Ok(outcome) => {
                    if outcome.mutation == PatchMutation::Unchanged {
                        ui.notice_info(format!("{path} was already unset."));
                    } else {
                        ui.notice_info(write_notice_text("Unset", &path, &outcome));
                    }
                }
                Err(error) => ui.notice_error(error.message()),
            },
        }
    }

    fn set_field_value(&self, ui: &mut ConfigUiApp, source_id: &str, path: &str, value: JsonValue) {
        match self.write_source_field_value(ui, source_id, path, &value) {
            Ok(outcome) => {
                if outcome.mutation == PatchMutation::Unchanged {
                    ui.notice_info(format!("{path} was already set."));
                } else {
                    ui.notice_info(write_notice_text("Saved", path, &outcome));
                }
            }
            Err(error) => ui.notice_error(error.message()),
        }
    }

    fn write_source_field_value(
        &self,
        ui: &mut ConfigUiApp,
        source_id: &str,
        setting_path: &str,
        value: &JsonValue,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let target = self.editable_config_target(ui, source_id, setting_path)?;
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome = set_toml_value_text(&raw, &target.path_in_file, value)
            .map_err(|error| config_toml_patch_error(&target.path, error))?;
        self.finish_field_write(ui, source_id, setting_path, &target, outcome)
    }

    fn unset_field_value(
        &self,
        ui: &mut ConfigUiApp,
        source_id: &str,
        setting_path: &str,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let target = self.editable_config_target(ui, source_id, setting_path)?;
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome = unset_toml_value_text(&raw, &target.path_in_file)
            .map_err(|error| config_toml_patch_error(&target.path, error))?;
        self.finish_field_write(ui, source_id, setting_path, &target, outcome)
    }

    fn finish_field_write(
        &self,
        ui: &mut ConfigUiApp,
        source_id: &str,
        setting_path: &str,
        target: &ConfigUiEditTarget,
        outcome: PatchOutcome,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let should_write = outcome.changed();
        let mutation = outcome.mutation;
        let text = outcome.text;
        let remove_empty_main = target.kind == ConfigUiEditTargetKind::Main
            && sparse_config_is_semantically_empty(&target.path, &text)?;
        if remove_empty_main {
            match fs::remove_file(&target.path) {
                Ok(()) => {}
                Err(source) if source.kind() == io::ErrorKind::NotFound => {}
                Err(source) => {
                    return Err(CoreError::io(
                        "remove_empty_settings_config",
                        "Could not remove the empty Yazelix config",
                        "Fix permissions for config.toml, then retry.",
                        target.path.display().to_string(),
                        source,
                    ));
                }
            }
        } else if should_write {
            self.validate_patched_edit_target(target, &text)?;
            if matches!(
                target.kind,
                ConfigUiEditTargetKind::Main | ConfigUiEditTargetKind::Mars
            ) {
                crate::atomic_fs::write_text_atomic(&target.path, &text)?;
            } else {
                write_settings_edit(&target.path, &text)?;
            }
        }
        let apply_notice = if should_write && target.kind == ConfigUiEditTargetKind::Mars {
            Some("Mars reads this native config when the next window opens.".to_string())
        } else if should_write {
            match apply_after_field_write(self.request, setting_path) {
                Ok(status) => apply_status_notice(&status),
                Err(error) => Some(apply_error_notice(&error)),
            }
        } else {
            None
        };
        self.reload_model_preserving_selection(ui, source_id, setting_path)?;
        Ok(ConfigUiWriteOutcome {
            mutation,
            apply_notice,
        })
    }

    fn reload_model_preserving_selection(
        &self,
        ui: &mut ConfigUiApp,
        source_id: &str,
        selected_path: &str,
    ) -> Result<(), CoreError> {
        let selected_tab = ui
            .model
            .fields
            .iter()
            .find(|field| field.source_id == source_id && field.path == selected_path)
            .map(|field| field.tab.clone());
        ui.model = build_config_ui_model(self.request)?;
        if let Some(tab) = selected_tab
            && let Some(tab_index) = ui.model.tabs.iter().position(|candidate| candidate == &tab)
        {
            ui.selected_tab = tab_index;
        }
        ui.selected_row = ui
            .visible_rows()
            .iter()
            .position(|row| {
                matches!(
                    row,
                    UiRowRef::Field(index)
                        if ui.model.fields[*index].source_id == source_id
                            && ui.model.fields[*index].path == selected_path
                )
            })
            .unwrap_or(0);
        Ok(())
    }

    fn ensure_known_field_source(
        &self,
        ui: &ConfigUiApp,
        source_id: &str,
        setting_path: &str,
    ) -> Result<(), CoreError> {
        if ui
            .model
            .fields
            .iter()
            .any(|field| field.source_id == source_id && field.path == setting_path)
        {
            return Ok(());
        }
        if let Some(field) = ui
            .model
            .fields
            .iter()
            .find(|field| field.path == setting_path)
        {
            return Err(config_source_mismatch(
                source_id,
                setting_path,
                &field.source_id,
            ));
        }
        Err(unsupported_config_source(source_id, setting_path))
    }

    fn edit_target(
        &self,
        ui: &ConfigUiApp,
        source_id: &str,
        setting_path: &str,
    ) -> Result<ConfigUiEditTarget, CoreError> {
        let path = ui
            .model
            .sources
            .iter()
            .find(|source| source.id == source_id)
            .map(|source| source.path.clone())
            .ok_or_else(|| unsupported_config_source(source_id, setting_path))?;
        let (path_in_file, kind) = match source_id {
            SETTINGS_SOURCE_ID => (setting_path.to_string(), ConfigUiEditTargetKind::Main),
            CURSORS_SOURCE_ID => (
                cursor_path_in_file(setting_path)?,
                ConfigUiEditTargetKind::Cursors,
            ),
            MARS_SOURCE_ID => (setting_path.to_string(), ConfigUiEditTargetKind::Mars),
            _ => return Err(unsupported_config_source(source_id, setting_path)),
        };
        Ok(ConfigUiEditTarget {
            path,
            path_in_file,
            kind,
        })
    }

    fn read_edit_target_or_default(
        &self,
        target: &ConfigUiEditTarget,
    ) -> Result<String, CoreError> {
        if target.path.exists() {
            return read_settings_for_edit(&target.path);
        }
        match target.kind {
            ConfigUiEditTargetKind::Main => Ok(String::new()),
            ConfigUiEditTargetKind::Cursors => {
                let default_cursor_config_path =
                    primary_config_paths(&self.request.runtime_dir, &self.request.config_dir)
                        .default_cursor_config_path;
                let raw = fs::read_to_string(&default_cursor_config_path).map_err(|source| {
                    CoreError::io(
                        "read_default_cursor_config_for_ui_edit",
                        "Could not read the default Yazelix cursor settings",
                        "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml.",
                        default_cursor_config_path.display().to_string(),
                        source,
                    )
                })?;
                CursorRegistry::parse_str(&default_cursor_config_path, &raw)?;
                Ok(raw)
            }
            ConfigUiEditTargetKind::Mars => Ok(String::new()),
        }
    }

    fn validate_patched_edit_target(
        &self,
        target: &ConfigUiEditTarget,
        text: &str,
    ) -> Result<(), CoreError> {
        match target.kind {
            ConfigUiEditTargetKind::Main => validate_patched_settings_for_ui(self.request, text),
            ConfigUiEditTargetKind::Cursors => {
                CursorRegistry::parse_str(&target.path, text)?;
                Ok(())
            }
            ConfigUiEditTargetKind::Mars => toml::from_str::<toml::Table>(text)
                .map(|_| ())
                .map_err(|source| {
                    CoreError::classified(
                        ErrorClass::Config,
                        "invalid_mars_config",
                        format!("Could not parse {}: {source}.", target.path.display()),
                        "Fix the TOML syntax in ~/.config/yazelix/mars/config.toml, then retry.",
                        json!({ "path": target.path.display().to_string() }),
                    )
                }),
        }
    }

    fn editable_config_target(
        &self,
        ui: &ConfigUiApp,
        source_id: &str,
        setting_path: &str,
    ) -> Result<ConfigUiEditTarget, CoreError> {
        self.ensure_known_field_source(ui, source_id, setting_path)?;
        let target = self.edit_target(ui, source_id, setting_path)?;
        if target.kind == ConfigUiEditTargetKind::Main && !is_settings_config_path(&target.path) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_config_edit_surface",
                format!(
                    "The config UI can only edit config.toml, but the active config is {}.",
                    target.path.display()
                ),
                "Move this setting to config.toml, or clear YAZELIX_CONFIG_OVERRIDE.",
                json!({ "path": target.path.display().to_string() }),
            ));
        }
        let target_exists = path_present(&target.path);
        if classify_path_owner(&target.path, target_exists) == ConfigUiPathOwner::HomeManager {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "home_manager_owned_config",
                "This settings file is owned by Home Manager.",
                "Edit your Home Manager module options instead, then run home-manager switch.",
                json!({ "path": target.path.display().to_string() }),
            ));
        }
        if path_is_read_only(&target.path) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "read_only_settings_config",
                format!(
                    "The active settings file is read-only: {}.",
                    target.path.display()
                ),
                "Fix file permissions or edit the owning configuration source.",
                json!({ "path": target.path.display().to_string() }),
            ));
        }
        Ok(target)
    }

    fn open_config_file(
        &self,
        ui: &mut ConfigUiApp,
        source_id: &str,
        action_id: &str,
        path: &Path,
    ) -> Result<(), CoreError> {
        let mars_path = user_config_paths::mars_config(&self.request.config_dir);
        let cursor_path = user_config_paths::cursor_config(&self.request.config_dir);
        let tab = if source_id == MARS_SOURCE_ID
            && action_id == MARS_CONFIG_ACTION_ID
            && path == mars_path
        {
            prepare_mars_config_file(self.request)?;
            MARS_TAB
        } else if source_id == CURSORS_SOURCE_ID
            && action_id == CURSORS_CONFIG_ACTION_ID
            && path == cursor_path
        {
            yazelix_cursors::initialize_cursor_config(path)?;
            "cursors"
        } else {
            return Err(CoreError::usage(format!(
                "Unsupported config file action {source_id}/{action_id} for {}.",
                path.display()
            )));
        };

        suspend_config_ui_terminal(|| {
            crate::edit_commands::run_editor_child(&self.request.runtime_dir, path)
        })?;
        let raw = fs::read_to_string(path).map_err(|source| {
            CoreError::io(
                "read_edited_native_config",
                "Could not read the edited config",
                "Fix permissions for the file, then retry.",
                path.display().to_string(),
                source,
            )
        })?;
        if source_id == CURSORS_SOURCE_ID {
            CursorRegistry::parse_str(path, &raw)?;
        } else {
            toml::from_str::<toml::Table>(&raw).map_err(|source| {
                CoreError::classified(
                    ErrorClass::Config,
                    "invalid_mars_config",
                    format!("Could not parse {}: {source}.", path.display()),
                    "Fix the TOML syntax before opening a new Mars window.",
                    json!({ "path": path.display().to_string() }),
                )
            })?;
        }
        ui.model = build_config_ui_model(self.request)?;
        ui.selected_tab = ui
            .model
            .tabs
            .iter()
            .position(|candidate| candidate == tab)
            .unwrap_or(0);
        ui.selected_row = 0;
        Ok(())
    }
}

pub(super) fn prepare_mars_config_file(request: &ConfigUiRequest) -> Result<(), CoreError> {
    let path = user_config_paths::mars_config(&request.config_dir);
    if path_present(&path) {
        if path_owned_by_home_manager(&path) || path_is_read_only(&path) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "read_only_mars_config",
                "The Mars override is read-only.",
                "Edit its owning configuration source or fix its permissions before retrying.",
                json!({ "path": path.display().to_string() }),
            ));
        }
        if !path.is_file() {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "invalid_mars_config_path",
                format!("The Mars config path is not a file: {}.", path.display()),
                "Remove the conflicting filesystem entry or replace it with a config.toml file.",
                json!({ "path": path.display().to_string() }),
            ));
        }
        return Ok(());
    }
    crate::atomic_fs::write_text_atomic(&path, "")
}

fn suspend_config_ui_terminal(
    action: impl FnOnce() -> Result<(), CoreError>,
) -> Result<(), CoreError> {
    disable_raw_mode().map_err(terminal_err)?;
    if let Err(source) = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen) {
        let _ = enable_raw_mode();
        return Err(terminal_err(source));
    }
    let result = action();
    let resume =
        enable_raw_mode().and_then(|_| execute!(io::stdout(), EnterAlternateScreen, cursor::Hide));
    if let Err(source) = resume {
        return Err(terminal_err(source));
    }
    result
}

fn config_toml_patch_error(path: &Path, error: TomlPatchError) -> CoreError {
    CoreError::classified(
        ErrorClass::Config,
        "config_toml_patch_failed",
        format!("Could not update {}: {error:?}.", path.display()),
        "Fix the TOML structure in the reported config file, then retry.",
        json!({ "path": path.display().to_string() }),
    )
}

pub(super) fn write_notice_text(verb: &str, path: &str, outcome: &ConfigUiWriteOutcome) -> String {
    let mut text = format!("{verb} {path}.");
    if let Some(apply_notice) = &outcome.apply_notice {
        text.push(' ');
        text.push_str(apply_notice);
    }
    text
}

fn apply_status_notice(status: &crate::config_apply::ConfigEditApplyStatus) -> Option<String> {
    Some(format!("Apply: {}.", status.apply_mode.label()))
}

fn apply_error_notice(error: &CoreError) -> String {
    let remediation = error.remediation();
    if remediation.trim().is_empty() {
        format!("Apply pending: {}", error.message())
    } else {
        format!("Apply pending: {} {}", error.message(), remediation)
    }
}

fn unsupported_config_source(source_id: &str, setting_path: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "unsupported_config_source",
        format!("Config source {source_id} cannot edit {setting_path}."),
        "Choose a field from a supported Yazelix config source.",
        json!({
            "source_id": source_id,
            "path": setting_path,
        }),
    )
}

fn config_source_mismatch(
    source_id: &str,
    setting_path: &str,
    expected_source_id: &str,
) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "config_source_mismatch",
        format!("{setting_path} belongs to config source {expected_source_id}, not {source_id}."),
        "Retry the edit through the field's owning config source.",
        json!({
            "source_id": source_id,
            "expected_source_id": expected_source_id,
            "path": setting_path,
        }),
    )
}

fn cursor_path_in_file(setting_path: &str) -> Result<String, CoreError> {
    setting_path
        .strip_prefix(CURSORS_FIELD_PREFIX)
        .map(ToOwned::to_owned)
        .ok_or_else(|| unsupported_config_source(CURSORS_SOURCE_ID, setting_path))
}

fn ratconfig_runner_err(error: CrosstermRunnerError<CoreError>) -> CoreError {
    match error {
        CrosstermRunnerError::Terminal(source) => terminal_err(source),
        CrosstermRunnerError::Host(error) => error,
    }
}

fn terminal_err(source: io::Error) -> CoreError {
    CoreError::io(
        "config_ui_terminal",
        "Could not run the Yazelix config UI terminal session",
        "Retry from a healthy interactive terminal, or run `yzx config` for plain text output.",
        ".",
        source,
    )
}
