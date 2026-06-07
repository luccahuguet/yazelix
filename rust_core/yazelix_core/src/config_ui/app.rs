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
    enable_raw_mode().map_err(terminal_err)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(terminal_err)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(terminal_err)?;
    let result = run_ui_loop(&mut terminal, request, model);
    let cleanup = restore_terminal(&mut terminal);

    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(0),
    }
}

fn run_ui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    request: ConfigUiRequest,
    model: ConfigUiModel,
) -> Result<(), CoreError> {
    let mut app = YazelixConfigUiApp::new(request, model);

    loop {
        app.clamp_selection();
        terminal
            .draw(|frame| draw_config_ui_with_details(frame, &mut app.ui, render_details))
            .map_err(terminal_err)?;
        if event::poll(Duration::from_millis(200)).map_err(terminal_err)?
            && let Event::Key(key) = event::read().map_err(terminal_err)?
            && key.kind != KeyEventKind::Release
            && app.handle_key(key)
        {
            break;
        }
    }

    Ok(())
}

fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), CoreError> {
    let mut first_error = disable_raw_mode().map_err(terminal_err).err();
    if let Err(error) = execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(terminal_err)
        && first_error.is_none()
    {
        first_error = Some(error);
    }
    if let Err(error) = terminal.show_cursor().map_err(terminal_err)
        && first_error.is_none()
    {
        first_error = Some(error);
    }
    first_error.map_or(Ok(()), Err)
}

impl YazelixConfigUiApp {
    pub(crate) fn new(request: ConfigUiRequest, model: ConfigUiModel) -> Self {
        Self {
            request,
            ui: ConfigUiApp::new(model),
        }
    }

    pub(super) fn handle_key(&mut self, key: KeyEvent) -> bool {
        let Some(key) = key_event_to_ratconfig_key(key) else {
            return false;
        };
        let intent = self.ui.handle_key(key);
        self.handle_ratconfig_intent(intent)
    }

    fn handle_ratconfig_intent(&mut self, intent: ConfigUiIntent) -> bool {
        match intent {
            ConfigUiIntent::None => false,
            ConfigUiIntent::Exit => true,
            ConfigUiIntent::BeginEdit { field_index, path } => {
                match self.ensure_editable_config(&path) {
                    Ok(()) => self.ui.begin_edit_field(field_index),
                    Err(error) => self.notice_error(error.message()),
                }
                false
            }
            ConfigUiIntent::SetField {
                field_index, value, ..
            } => {
                self.set_field_value(field_index, value);
                self.ui.finish_successful_write();
                false
            }
            ConfigUiIntent::UnsetField { path, .. } => {
                match self.unset_field_value(&path) {
                    Ok(outcome) => {
                        if outcome.mutation == SettingsJsoncPatchMutation::Unchanged {
                            self.notice_info(format!("{path} was already unset."));
                        } else {
                            self.notice_info(write_notice_text("Unset", &path, &outcome));
                        }
                    }
                    Err(error) => self.notice_error(error.message()),
                }
                false
            }
        }
    }

    fn set_field_value(&mut self, field_index: usize, value: JsonValue) {
        let path = self.model.fields[field_index].path.clone();
        match self.write_field_value(&path, &value) {
            Ok(outcome) => {
                if outcome.mutation == SettingsJsoncPatchMutation::Unchanged {
                    self.notice_info(format!("{path} was already set."));
                } else {
                    self.notice_info(write_notice_text("Saved", &path, &outcome));
                }
            }
            Err(error) => self.notice_error(error.message()),
        }
    }

    pub(super) fn write_field_value(
        &mut self,
        setting_path: &str,
        value: &JsonValue,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        if custom_popup_path(setting_path).is_some() {
            return self.write_custom_popup_field_value(setting_path, value);
        }
        self.ensure_editable_config(setting_path)?;
        let target = self.edit_target(setting_path);
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome =
            set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, value)?;
        self.finish_field_write(setting_path, &target, outcome)
    }

    fn unset_field_value(&mut self, setting_path: &str) -> Result<ConfigUiWriteOutcome, CoreError> {
        if custom_popup_path(setting_path).is_some() {
            return self.unset_custom_popup_field_value(setting_path);
        }
        self.ensure_editable_config(setting_path)?;
        let target = self.edit_target(setting_path);
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome = match target.kind {
            ConfigUiEditTargetKind::Main => {
                let value = default_main_setting_value_for_ui(&self.request, &target.path_in_file)?;
                set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, &value)?
            }
            ConfigUiEditTargetKind::Cursors => {
                unset_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file)?
            }
        };
        self.finish_field_write(setting_path, &target, outcome)
    }

    fn write_custom_popup_field_value(
        &mut self,
        setting_path: &str,
        value: &JsonValue,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        self.ensure_editable_config(CUSTOM_POPUPS_FIELD_PATH)?;
        let target = self.edit_target(CUSTOM_POPUPS_FIELD_PATH);
        let raw = self.read_edit_target_or_default(&target)?;
        let root = parse_jsonc_value(&target.path, &raw)?;
        let default_value =
            default_main_setting_value_for_ui(&self.request, CUSTOM_POPUPS_FIELD_PATH)?;
        let Some(next_value) =
            custom_popup_list_value_after_write(&root, &default_value, setting_path, value)?
        else {
            return Err(unsupported_custom_popup_edit_path(setting_path));
        };
        let outcome = set_settings_jsonc_value_text(
            &target.path,
            &raw,
            CUSTOM_POPUPS_FIELD_PATH,
            &next_value,
        )?;
        self.finish_field_write(setting_path, &target, outcome)
    }

    fn unset_custom_popup_field_value(
        &mut self,
        setting_path: &str,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        self.ensure_editable_config(CUSTOM_POPUPS_FIELD_PATH)?;
        let target = self.edit_target(CUSTOM_POPUPS_FIELD_PATH);
        let raw = self.read_edit_target_or_default(&target)?;
        let root = parse_jsonc_value(&target.path, &raw)?;
        let default_value =
            default_main_setting_value_for_ui(&self.request, CUSTOM_POPUPS_FIELD_PATH)?;
        let Some(next_value) =
            custom_popup_list_value_after_unset(&root, &default_value, setting_path)?
        else {
            return Err(unsupported_custom_popup_edit_path(setting_path));
        };
        let outcome = set_settings_jsonc_value_text(
            &target.path,
            &raw,
            CUSTOM_POPUPS_FIELD_PATH,
            &next_value,
        )?;
        self.finish_field_write(setting_path, &target, outcome)
    }

    fn finish_field_write(
        &mut self,
        setting_path: &str,
        target: &ConfigUiEditTarget,
        outcome: SettingsJsoncPatchOutcome,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let mut apply_status = None;
        let mut apply_error = None;
        let (text, should_write) = self.reconcile_patched_edit_target(target, &outcome)?;
        if should_write {
            self.validate_patched_edit_target(&target, &text)?;
            write_settings_edit(&target.path, &text)?;
        }
        if outcome.changed() {
            match apply_after_field_write(&self.request, &self.model, setting_path) {
                Ok(status) => apply_status = Some(status),
                Err(error) => apply_error = Some(apply_error_notice(&error)),
            }
        }
        self.reload_model_preserving_selection(setting_path)?;
        Ok(ConfigUiWriteOutcome {
            mutation: outcome.mutation,
            apply_status,
            apply_error,
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

    fn reload_model_preserving_selection(&mut self, selected_path: &str) -> Result<(), CoreError> {
        let selected_tab = self
            .model
            .fields
            .iter()
            .find(|field| field.path == selected_path)
            .map(|field| field.tab.clone());
        self.model = build_config_ui_model(&self.request)?;
        if let Some(tab) = selected_tab
            && let Some(tab_index) = self
                .model
                .tabs
                .iter()
                .position(|candidate| candidate == &tab)
        {
            self.selected_tab = tab_index;
        }
        self.selected_row = self
            .visible_rows()
            .iter()
            .position(|row| {
                matches!(
                    row,
                    UiRowRef::Field(index) if self.model.fields[*index].path == selected_path
                )
            })
            .unwrap_or(0);
        Ok(())
    }

    fn edit_target(&self, setting_path: &str) -> ConfigUiEditTarget {
        if let Some(cursor_path) = setting_path.strip_prefix("cursors.") {
            ConfigUiEditTarget {
                path: self.model.cursor_config_path.clone(),
                path_in_file: cursor_path.to_string(),
                kind: ConfigUiEditTargetKind::Cursors,
            }
        } else {
            ConfigUiEditTarget {
                path: self.model.active_config_path.clone(),
                path_in_file: setting_path.to_string(),
                kind: ConfigUiEditTargetKind::Main,
            }
        }
    }

    fn read_edit_target_or_default(
        &self,
        target: &ConfigUiEditTarget,
    ) -> Result<String, CoreError> {
        if target.path.exists() {
            return read_settings_for_edit(&target.path);
        }
        match target.kind {
            ConfigUiEditTargetKind::Main => default_main_settings_text_for_ui(&self.request),
            ConfigUiEditTargetKind::Cursors => {
                let raw =
                    fs::read_to_string(&self.model.default_cursor_config_path).map_err(|source| {
                        CoreError::io(
                            "read_default_cursor_config_for_ui_edit",
                            "Could not read the default Yazelix cursor settings",
                            "Reinstall Yazelix so the runtime includes yazelix_ghostty_cursors_default.toml.",
                            self.model.default_cursor_config_path.display().to_string(),
                            source,
                        )
                    })?;
                let registry =
                    CursorRegistry::parse_str(&self.model.default_cursor_config_path, &raw)?;
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
            ConfigUiEditTargetKind::Main => validate_patched_settings_for_ui(&self.request, text),
            ConfigUiEditTargetKind::Cursors => {
                let value = parse_jsonc_value(&target.path, text)?;
                CursorRegistry::parse_json_value(&target.path, value)?;
                Ok(())
            }
        }
    }

    fn ensure_editable_config(&self, setting_path: &str) -> Result<(), CoreError> {
        let target = self.edit_target(setting_path);
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
        Ok(())
    }
}

fn key_event_to_ratconfig_key(key: KeyEvent) -> Option<ConfigUiKey> {
    match key.code {
        KeyCode::Esc => Some(ConfigUiKey::Esc),
        KeyCode::Enter => Some(ConfigUiKey::Enter),
        KeyCode::Backspace => Some(ConfigUiKey::Backspace),
        KeyCode::Tab => Some(ConfigUiKey::Tab),
        KeyCode::BackTab => Some(ConfigUiKey::BackTab),
        KeyCode::Up => Some(ConfigUiKey::Up),
        KeyCode::Down => Some(ConfigUiKey::Down),
        KeyCode::Left => Some(ConfigUiKey::Left),
        KeyCode::Right => Some(ConfigUiKey::Right),
        KeyCode::Char(ch) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(ConfigUiKey::Ctrl(ch))
        }
        KeyCode::Char(ch) => Some(ConfigUiKey::Char(ch)),
        _ => None,
    }
}

pub(super) fn write_notice_text(verb: &str, path: &str, outcome: &ConfigUiWriteOutcome) -> String {
    let mut text = format!("{verb} {path}.");
    if let Some(error) = &outcome.apply_error {
        text.push(' ');
        text.push_str(error);
        return text;
    }
    if let Some(refresh) = outcome
        .apply_status
        .as_ref()
        .and_then(|status| status.generated_refresh.as_ref())
    {
        text.push(' ');
        text.push_str(&refresh.message);
        text.push(' ');
        text.push_str(&refresh.remediation);
    }
    if let Some(refresh) = outcome
        .apply_status
        .as_ref()
        .and_then(|status| status.pane_orchestrator_refresh.as_ref())
    {
        text.push(' ');
        text.push_str(&refresh.message);
        text.push(' ');
        text.push_str(&refresh.remediation);
    }
    text
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

fn terminal_err(source: io::Error) -> CoreError {
    CoreError::io(
        "config_ui_terminal",
        "Could not run the Yazelix config UI terminal session",
        "Retry from a healthy interactive terminal, or run `yzx config` for plain text output.",
        ".",
        source,
    )
}
