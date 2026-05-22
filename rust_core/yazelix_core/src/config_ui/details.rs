use super::*;

pub(super) fn render_details(app: &ConfigUiApp, row: UiRowRef) -> Vec<Line<'static>> {
    match row {
        UiRowRef::Field(index) => {
            let field = &app.model.fields[index];
            if let Some(edit) = &app.edit
                && edit.field_index == index
                && edit.mode == ConfigUiEditMode::Choice
                && is_scalar_enum_field(field)
            {
                return single_choice_detail_lines(field, edit);
            }
            if let Some(edit) = &app.edit
                && edit.field_index == index
                && edit.mode == ConfigUiEditMode::MultiChoice
            {
                return multi_choice_detail_lines(field, edit);
            }
            if is_scalar_enum_field(field) {
                return single_choice_field_detail_lines(field);
            }
            field_detail_lines(field)
        }
        UiRowRef::Sidecar(index) => sidecar_detail_lines(&app.model.sidecars[index]),
        UiRowRef::Diagnostic(index) => diagnostic_detail_lines(&app.model.diagnostics[index]),
        UiRowRef::NativeStatus(index) => {
            native_status_detail_lines(&app.model.native_config_statuses[index])
        }
    }
}

pub(super) fn field_detail_lines(field: &ConfigUiField) -> Vec<Line<'static>> {
    if is_keybinding_map_field_path(&field.path) {
        keybinding_map_detail_lines(field)
    } else if let Some(action) = keybinding_action_metadata_for_field_path(&field.path) {
        keybinding_action_detail_lines(field, action)
    } else {
        default_field_detail_lines(field)
    }
}
