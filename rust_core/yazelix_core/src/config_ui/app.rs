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
        if self.edit.is_some() {
            self.handle_edit_key(key);
            return false;
        }

        if self.search_active {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => self.search_active = false,
                KeyCode::Backspace => {
                    self.search.pop();
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.search.clear();
                }
                KeyCode::Char(ch) => {
                    self.search.push(ch);
                    self.selected_row = 0;
                }
                _ => {}
            }
            return false;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char('/') => self.search_active = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Enter => self.activate_selected_field(),
            KeyCode::Char('e') => self.begin_edit_selected_field(),
            KeyCode::Char(' ') => self.quick_edit_selected_field(),
            KeyCode::Char('u') => self.unset_selected_field(),
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => self.next_tab(),
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => self.previous_tab(),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,
            _ => {}
        }

        false
    }

    pub(super) fn handle_edit_key(&mut self, key: KeyEvent) {
        if let Some(mode) = self.edit.as_ref().map(|edit| edit.mode) {
            match mode {
                ConfigUiEditMode::Choice | ConfigUiEditMode::MultiChoice => {
                    self.handle_choice_edit_key(key, mode);
                    return;
                }
                ConfigUiEditMode::Text => {}
            }
        }

        match key.code {
            KeyCode::Esc => {
                self.edit = None;
                self.notice_info("Edit canceled.");
            }
            KeyCode::Enter => self.save_edit(),
            KeyCode::Backspace => {
                self.notice = None;
                if let Some(edit) = &mut self.edit {
                    edit.input.pop();
                }
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.notice = None;
                if let Some(edit) = &mut self.edit {
                    edit.input.clear();
                }
            }
            KeyCode::Char(ch) => {
                self.notice = None;
                if let Some(edit) = &mut self.edit {
                    edit.input.push(ch);
                }
            }
            _ => {}
        }
    }

    fn handle_choice_edit_key(&mut self, key: KeyEvent, mode: ConfigUiEditMode) {
        let scalar_enum = self
            .edit
            .as_ref()
            .and_then(|edit| self.model.fields.get(edit.field_index))
            .is_some_and(is_scalar_enum_field);
        let multi_choice = mode == ConfigUiEditMode::MultiChoice;
        match key.code {
            KeyCode::Esc => {
                self.edit = None;
                self.notice_info("Edit canceled.");
            }
            KeyCode::Enter if scalar_enum => {
                self.select_single_choice_edit();
                self.save_edit();
            }
            KeyCode::Enter => self.save_edit(),
            KeyCode::Up | KeyCode::Left | KeyCode::Char('k') | KeyCode::Char('h')
                if scalar_enum || multi_choice =>
            {
                self.notice = None;
                self.move_choice_edit(-1);
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('j') | KeyCode::Char('l')
                if scalar_enum || multi_choice =>
            {
                self.notice = None;
                self.move_choice_edit(1);
            }
            KeyCode::Char(' ') if multi_choice => {
                self.notice = None;
                self.toggle_multi_choice_edit();
            }
            KeyCode::Char(' ') if scalar_enum => {
                self.notice = None;
                self.select_single_choice_edit();
            }
            KeyCode::Up | KeyCode::Right | KeyCode::Down | KeyCode::Left | KeyCode::Char(' ')
                if !multi_choice =>
            {
                self.notice = None;
                self.cycle_choice_edit();
            }
            _ => {}
        }
    }

    fn cycle_choice_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let next = if edit.input.trim() == "true" {
            "false".to_string()
        } else {
            "true".to_string()
        };
        if let Some(edit) = &mut self.edit {
            edit.input = next;
        }
    }

    fn move_choice_edit(&mut self, delta: isize) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let len = field.allowed_values.len();
        if len == 0 {
            return;
        }
        let index = edit.choice_index.min(len - 1);
        let next = if delta < 0 {
            index.checked_sub(1).unwrap_or(len - 1)
        } else {
            (index + 1) % len
        };
        if let Some(edit) = &mut self.edit {
            edit.choice_index = next;
        }
    }

    fn select_single_choice_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let Some(value) = field.allowed_values.get(edit.choice_index) else {
            return;
        };
        let value = value.clone();
        if let Some(edit) = &mut self.edit {
            edit.input = value;
        }
    }

    fn toggle_multi_choice_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = &self.model.fields[edit.field_index];
        let next = match toggled_string_list_input(field, &edit.input, edit.choice_index) {
            Ok(next) => next,
            Err(message) => {
                self.notice_error(message);
                return;
            }
        };
        if let Some(edit) = &mut self.edit {
            edit.input = next;
        }
    }

    fn activate_selected_field(&mut self) {
        if self.selected_field().is_some_and(is_bool_field) {
            self.quick_edit_selected_field();
        } else {
            self.begin_edit_selected_field();
        }
    }

    fn begin_edit_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be edited.");
            return;
        };
        let field = &self.model.fields[field_index];
        if let Err(error) = self.ensure_editable_config(&field.path) {
            self.notice_error(error.message());
            return;
        }
        if let Some(message) = structured_only_edit_notice(field).map(str::to_string) {
            self.notice_info(message);
            return;
        }
        let input = edit_input_for_field(field);
        self.edit = Some(ConfigUiEditState {
            field_index,
            choice_index: initial_edit_choice_index(field, &input),
            input,
            mode: edit_mode_for_field(field),
        });
    }

    fn quick_edit_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be edited.");
            return;
        };
        let field = &self.model.fields[field_index];
        if is_bool_field(field) {
            if let Err(error) = self.ensure_editable_config(&field.path) {
                self.notice_error(error.message());
                return;
            }
            let value = JsonValue::Bool(!field_bool_value(field).unwrap_or(false));
            self.set_field_value(field_index, value);
        } else {
            self.begin_edit_selected_field();
        }
    }

    fn unset_selected_field(&mut self) {
        self.notice = None;
        let Some(field_index) = self.selected_field_index() else {
            self.notice_error("Only settings rows can be unset.");
            return;
        };
        let path = self.model.fields[field_index].path.clone();
        if let Err(error) = self.ensure_editable_config(&path) {
            self.notice_error(error.message());
            return;
        }
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
    }

    fn save_edit(&mut self) {
        let Some(edit) = self.edit.clone() else {
            return;
        };
        let field = self.model.fields[edit.field_index].clone();
        let value = match parse_edit_input(&field, &edit.input) {
            Ok(value) => value,
            Err(message) => {
                self.notice_error(message);
                return;
            }
        };
        self.set_field_value(edit.field_index, value);
        if self
            .notice
            .as_ref()
            .map(|notice| !notice.is_error)
            .unwrap_or(false)
        {
            self.edit = None;
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
        self.ensure_editable_config(setting_path)?;
        let target = self.edit_target(setting_path);
        let raw = self.read_edit_target_or_default(&target)?;
        let outcome =
            set_settings_jsonc_value_text(&target.path, &raw, &target.path_in_file, value)?;
        self.finish_field_write(setting_path, &target, outcome)
    }

    fn unset_field_value(&mut self, setting_path: &str) -> Result<ConfigUiWriteOutcome, CoreError> {
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

    fn finish_field_write(
        &mut self,
        setting_path: &str,
        target: &ConfigUiEditTarget,
        outcome: SettingsJsoncPatchOutcome,
    ) -> Result<ConfigUiWriteOutcome, CoreError> {
        let mut apply_status = None;
        let mut apply_error = None;
        if outcome.changed() {
            self.validate_patched_edit_target(&target, &outcome.text)?;
            write_settings_edit(&target.path, &outcome.text)?;
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

fn terminal_err(source: io::Error) -> CoreError {
    CoreError::io(
        "config_ui_terminal",
        "Could not run the Yazelix config UI terminal session",
        "Retry from a healthy interactive terminal, or run `yzx config` for plain text output.",
        ".",
        source,
    )
}
