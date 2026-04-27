// Test lane: default
//! Rust-owned transient pane facts for popup and menu callers.

use crate::bridge::CoreError;
use crate::session_facts::compute_session_facts_from_env;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TransientPaneFactsData {
    pub popup_program: Vec<String>,
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
}

pub fn compute_transient_pane_facts_from_env() -> Result<TransientPaneFactsData, CoreError> {
    let facts = compute_session_facts_from_env()?;
    Ok(TransientPaneFactsData {
        popup_program: facts.popup_program,
        popup_width_percent: facts.popup_width_percent,
        popup_height_percent: facts.popup_height_percent,
    })
}
