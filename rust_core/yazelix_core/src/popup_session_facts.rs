// Test lane: default
//! Rust-owned popup/session facts for command popup overrides.

use crate::bridge::CoreError;
use crate::session_facts::compute_session_facts_from_env;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PopupSessionFactsData {
    pub popup_width_percent: i64,
    pub popup_height_percent: i64,
}

pub fn compute_popup_session_facts_from_env() -> Result<PopupSessionFactsData, CoreError> {
    let facts = compute_session_facts_from_env()?;
    Ok(PopupSessionFactsData {
        popup_width_percent: facts.popup_width_percent,
        popup_height_percent: facts.popup_height_percent,
    })
}
