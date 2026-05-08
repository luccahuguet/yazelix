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
pub(crate) use model::{UiRowRef, owner_label, visible_rows_for_tab_search};
pub(crate) use render::*;
