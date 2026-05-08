//! Private reusable Ratatui config editor boundary used by Yazelix.
//!
//! Yazelix-specific loading, patching, Home Manager ownership, native-config
//! classification, generated refresh, and pane-orchestrator apply behavior stay
//! in `config_ui`.

pub(crate) mod editor;
pub mod model;
pub(crate) mod render;

pub(crate) use editor::*;
pub use model::{
    ConfigUiApplyStatus, ConfigUiDiagnostic, ConfigUiField, ConfigUiModel, ConfigUiNativeStatus,
    ConfigUiPathOwner, ConfigUiSidecar, ConfigUiValueState,
};
pub(crate) use model::{
    UiRowRef, effective_string_config, effective_string_list_config, get_json_path, owner_label,
    render_json_edit_value, render_json_value, tab_index, visible_rows_for_tab_search,
};
pub(crate) use render::*;
