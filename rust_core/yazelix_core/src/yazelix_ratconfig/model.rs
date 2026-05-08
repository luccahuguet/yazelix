use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiModel {
    pub active_config_path: PathBuf,
    pub cursor_config_path: PathBuf,
    pub default_cursor_config_path: PathBuf,
    pub active_config_exists: bool,
    pub config_owner: ConfigUiPathOwner,
    pub config_read_only: bool,
    pub tabs: Vec<String>,
    pub fields: Vec<ConfigUiField>,
    pub sidecars: Vec<ConfigUiSidecar>,
    pub native_config_statuses: Vec<ConfigUiNativeStatus>,
    pub diagnostics: Vec<ConfigUiDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigUiPathOwner {
    Default,
    HomeManager,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiRowRef {
    Field(usize),
    Sidecar(usize),
    NativeStatus(usize),
    Diagnostic(usize),
}

pub(crate) fn visible_rows_for_tab_search(
    model: &ConfigUiModel,
    selected_tab: usize,
    search: &str,
) -> Vec<UiRowRef> {
    let tab = model
        .tabs
        .get(selected_tab)
        .map(String::as_str)
        .unwrap_or("general");
    let search = search.to_ascii_lowercase();
    if tab == "advanced" {
        return (0..model.diagnostics.len())
            .map(UiRowRef::Diagnostic)
            .chain((0..model.sidecars.len()).map(UiRowRef::Sidecar))
            .chain((0..model.native_config_statuses.len()).map(UiRowRef::NativeStatus))
            .filter(|row| row_matches_search(model, *row, &search))
            .collect();
    }

    (0..model.fields.len())
        .filter(|index| model.fields[*index].tab == tab)
        .map(UiRowRef::Field)
        .filter(|row| row_matches_search(model, *row, &search))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigUiValueState {
    Explicit,
    Defaulted,
    Unset,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiField {
    pub path: String,
    pub tab: String,
    pub kind: String,
    pub current_value: String,
    pub(crate) edit_value: String,
    pub default_value: String,
    pub state: ConfigUiValueState,
    pub description: String,
    pub allowed_values: Vec<String>,
    pub validation: String,
    pub rebuild_required: bool,
    pub apply_status: ConfigUiApplyStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiApplyStatus {
    pub summary: String,
    pub label: String,
    pub detail: String,
    pub pending: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiSidecar {
    pub name: String,
    pub path: PathBuf,
    pub present: bool,
    pub owner: ConfigUiPathOwner,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiDiagnostic {
    pub path: String,
    pub status: String,
    pub headline: String,
    pub blocking: bool,
    pub detail_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigUiNativeStatus {
    pub surface: String,
    pub tool: String,
    pub description: String,
    pub status: String,
    pub label: String,
    pub severity: String,
    pub active_path: Option<String>,
    pub managed_path: Option<String>,
    pub native_paths: Vec<String>,
    pub generated_path: Option<String>,
    pub allowed_action: String,
    pub read_only_reason: Option<String>,
}

pub(crate) fn owner_label(owner: ConfigUiPathOwner) -> &'static str {
    match owner {
        ConfigUiPathOwner::Default => "default",
        ConfigUiPathOwner::HomeManager => "home-manager",
        ConfigUiPathOwner::User => "user",
    }
}

fn row_matches_search(model: &ConfigUiModel, row: UiRowRef, search: &str) -> bool {
    match row {
        UiRowRef::Field(index) => {
            let field = &model.fields[index];
            search_matches(
                search,
                [
                    field.path.as_str(),
                    field.current_value.as_str(),
                    field.default_value.as_str(),
                    field.description.as_str(),
                ],
            )
        }
        UiRowRef::Sidecar(index) => {
            let sidecar = &model.sidecars[index];
            let path = sidecar.path.to_string_lossy();
            search_matches(
                search,
                [
                    sidecar.name.as_str(),
                    path.as_ref(),
                    owner_label(sidecar.owner),
                ],
            )
        }
        UiRowRef::Diagnostic(index) => {
            let diagnostic = &model.diagnostics[index];
            search_matches(
                search,
                [
                    diagnostic.path.as_str(),
                    diagnostic.status.as_str(),
                    diagnostic.headline.as_str(),
                ],
            )
        }
        UiRowRef::NativeStatus(index) => {
            let status = &model.native_config_statuses[index];
            search_matches(
                search,
                [
                    status.surface.as_str(),
                    status.tool.as_str(),
                    status.status.as_str(),
                    status.label.as_str(),
                    status.description.as_str(),
                ],
            )
        }
    }
}

fn search_matches<'a>(search: &str, candidates: impl IntoIterator<Item = &'a str>) -> bool {
    search.is_empty()
        || candidates
            .into_iter()
            .any(|candidate| candidate.to_ascii_lowercase().contains(search))
}
