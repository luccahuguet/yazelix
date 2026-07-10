use super::*;

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

    pub(super) fn handle_key(&mut self, key: ConfigUiKey) {
        let intent = self.ui.handle_key(key);
        let host = YazelixConfigUiHost {
            request: &self.request,
        };
        host.handle_ratconfig_intent(&mut self.ui, intent);
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
                action_id, path, ..
            } => ui.notice_error(format!(
                "The Ratconfig file action {action_id} for {} has no Yazelix host handler.",
                path.display()
            )),
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
                    if outcome.mutation == SettingsJsoncPatchMutation::Unchanged {
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
                if outcome.mutation == SettingsJsoncPatchMutation::Unchanged {
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
        if custom_popup_path(setting_path).is_some() {
            self.ensure_known_field_source(ui, source_id, setting_path)?;
            return self.write_custom_popup_list(ui, source_id, setting_path, |root, default| {
                custom_popup_list_value_after_write(root, default, setting_path, value)
            });
        }
        let target = self.editable_config_target(ui, source_id, setting_path)?;
        let raw = self.read_edit_target_or_default(ui, &target)?;
        let outcome =
            set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, value)?;
        self.finish_field_write(ui, setting_path, &target, outcome)
    }

    fn unset_field_value(
        &self,
        ui: &mut ConfigUiApp,
        source_id: &str,
        setting_path: &str,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        if custom_popup_path(setting_path).is_some() {
            self.ensure_known_field_source(ui, source_id, setting_path)?;
            return self.write_custom_popup_list(ui, source_id, setting_path, |root, default| {
                custom_popup_list_value_after_unset(root, default, setting_path)
            });
        }
        let target = self.editable_config_target(ui, source_id, setting_path)?;
        let raw = self.read_edit_target_or_default(ui, &target)?;
        let outcome = match target.kind {
            ConfigUiEditTargetKind::Main => {
                let value = default_main_setting_value_for_ui(self.request, &target.path_in_file)?;
                set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, &value)?
            }
            ConfigUiEditTargetKind::Cursors => {
                unset_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file)?
            }
        };
        self.finish_field_write(ui, setting_path, &target, outcome)
    }

    fn write_custom_popup_list(
        &self,
        ui: &mut ConfigUiApp,
        source_id: &str,
        setting_path: &str,
        next_list: impl FnOnce(&JsonValue, &JsonValue) -> Result<Option<JsonValue>, CoreError>,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let target = self.editable_config_target(ui, source_id, CUSTOM_POPUPS_FIELD_PATH)?;
        let raw = self.read_edit_target_or_default(ui, &target)?;
        let root = parse_jsonc_value(&target.path, &raw)?;
        let default_value =
            default_main_setting_value_for_ui(self.request, CUSTOM_POPUPS_FIELD_PATH)?;
        let Some(next_value) = next_list(&root, &default_value)? else {
            return Err(unsupported_custom_popup_edit_path(setting_path));
        };
        let outcome = set_settings_jsonc_value_text(
            &target.path,
            &raw,
            CUSTOM_POPUPS_FIELD_PATH,
            &next_value,
        )?;
        self.finish_field_write(ui, setting_path, &target, outcome)
    }

    fn finish_field_write(
        &self,
        ui: &mut ConfigUiApp,
        setting_path: &str,
        target: &ConfigUiEditTarget,
        outcome: SettingsJsoncPatchOutcome,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let (text, should_write) = self.reconcile_patched_edit_target(target, &outcome)?;
        if should_write {
            self.validate_patched_edit_target(target, &text)?;
            write_settings_edit(&target.path, &text)?;
        }
        let apply_notice = if outcome.changed() {
            match apply_after_field_write(self.request, &ui.model, setting_path) {
                Ok(status) => apply_status_notice(&status),
                Err(error) => Some(apply_error_notice(&error)),
            }
        } else {
            None
        };
        self.reload_model_preserving_selection(ui, setting_path)?;
        Ok(ConfigUiWriteOutcome {
            mutation: outcome.mutation,
            apply_notice,
        })
    }

    fn reconcile_patched_edit_target(
        &self,
        target: &ConfigUiEditTarget,
        outcome: &SettingsJsoncPatchOutcome,
    ) -> Result<(String, bool), CoreError> {
        let mut text = outcome.text.clone();
        let mut should_write = outcome.changed();
        if target.kind == ConfigUiEditTargetKind::Main {
            let paths = primary_config_paths(&self.request.runtime_dir, &self.request.config_dir);
            let reconciled =
                reconcile_settings_contract_text(&target.path, &text, &paths.default_config_path)?;
            should_write = should_write || reconciled.changed();
            text = reconciled.text;
        }
        Ok((text, should_write))
    }

    fn reload_model_preserving_selection(
        &self,
        ui: &mut ConfigUiApp,
        selected_path: &str,
    ) -> Result<(), CoreError> {
        let selected_tab = ui
            .model
            .fields
            .iter()
            .find(|field| field.path == selected_path)
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
                    UiRowRef::Field(index) if ui.model.fields[*index].path == selected_path
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
        let Some(field) = ui
            .model
            .fields
            .iter()
            .find(|field| field.path == setting_path)
        else {
            return Err(unsupported_config_source(source_id, setting_path));
        };
        if field.source_id == source_id {
            Ok(())
        } else {
            Err(config_source_mismatch(
                source_id,
                setting_path,
                &field.source_id,
            ))
        }
    }

    fn edit_target(
        &self,
        ui: &ConfigUiApp,
        source_id: &str,
        setting_path: &str,
    ) -> Result<ConfigUiEditTarget, CoreError> {
        match source_id {
            SETTINGS_SOURCE_ID => Ok(ConfigUiEditTarget {
                path: ui.model.active_config_path.clone(),
                path_in_file: setting_path.to_string(),
                kind: ConfigUiEditTargetKind::Main,
            }),
            CURSORS_SOURCE_ID => Ok(ConfigUiEditTarget {
                path: ui.model.cursor_config_path.clone(),
                path_in_file: cursor_path_in_file(setting_path)?,
                kind: ConfigUiEditTargetKind::Cursors,
            }),
            _ => Err(unsupported_config_source(source_id, setting_path)),
        }
    }

    fn read_edit_target_or_default(
        &self,
        ui: &ConfigUiApp,
        target: &ConfigUiEditTarget,
    ) -> Result<String, CoreError> {
        if target.path.exists() {
            return read_settings_for_edit(&target.path);
        }
        match target.kind {
            ConfigUiEditTargetKind::Main => default_main_settings_text_for_ui(self.request),
            ConfigUiEditTargetKind::Cursors => {
                let raw =
                    fs::read_to_string(&ui.model.default_cursor_config_path).map_err(|source| {
                        CoreError::io(
                            "read_default_cursor_config_for_ui_edit",
                            "Could not read the default Yazelix cursor settings",
                            "Reinstall Yazelix so the runtime includes yazelix_cursors_default.toml.",
                            ui.model.default_cursor_config_path.display().to_string(),
                            source,
                        )
                    })?;
                let registry =
                    CursorRegistry::parse_str(&ui.model.default_cursor_config_path, &raw)?;
                Ok(render_cursor_settings_jsonc(&registry))
            }
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
                let value = parse_jsonc_value(&target.path, text)?;
                CursorRegistry::parse_json_value(&target.path, value)?;
                Ok(())
            }
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
        if !is_settings_config_path(&target.path) {
            return Err(CoreError::classified(
                ErrorClass::Config,
                "unsupported_config_edit_surface",
                format!(
                    "The config UI can only edit settings.jsonc, but the active config is {}.",
                    target.path.display()
                ),
                "Move this setting to settings.jsonc, or clear YAZELIX_CONFIG_OVERRIDE.",
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
    let mut parts = Vec::new();
    if let Some(refresh) = &status.generated_refresh {
        parts.push(refresh.message.as_str());
        parts.push(refresh.remediation.as_str());
    }
    if let Some(refresh) = &status.pane_orchestrator_refresh {
        parts.push(refresh.message.as_str());
        parts.push(refresh.remediation.as_str());
    }
    (!parts.is_empty()).then(|| parts.join(" "))
}

fn apply_error_notice(error: &CoreError) -> String {
    let remediation = error.remediation();
    if remediation.trim().is_empty() {
        format!("Apply pending: {}", error.message())
    } else {
        format!("Apply pending: {} {}", error.message(), remediation)
    }
}

fn unsupported_custom_popup_edit_path(setting_path: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "unsupported_custom_popup_edit_path",
        format!("{setting_path} is not a supported custom popup editor row."),
        "Select a custom popup add row, overview row, or child field row.",
        json!({ "path": setting_path }),
    )
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
