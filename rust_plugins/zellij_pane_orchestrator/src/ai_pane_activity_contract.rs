use serde::Deserialize;

use crate::active_tab_session_state::{SessionAiPaneActivity, SessionAiPaneActivityState};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AiPaneActivityRegistration {
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub pane_id: String,
    #[serde(default)]
    pub activity: String,
    #[serde(default)]
    pub state: Option<SessionAiPaneActivityState>,
}

pub fn normalized_ai_activity_state(
    registration: &AiPaneActivityRegistration,
) -> Option<SessionAiPaneActivityState> {
    if let Some(state) = registration.state {
        return Some(state);
    }
    let activity = registration.activity.trim();
    if activity.is_empty() {
        return Some(SessionAiPaneActivityState::Unknown);
    }
    SessionAiPaneActivityState::from_activity(activity)
}

pub fn upsert_ai_pane_activity_fact(
    facts: &mut Vec<SessionAiPaneActivity>,
    fact: SessionAiPaneActivity,
) {
    if fact.pane_id.trim().is_empty() {
        facts.retain(|existing| !existing.pane_id.trim().is_empty());
        facts.push(fact);
        return;
    }

    if let Some(existing) = facts
        .iter_mut()
        .find(|existing| existing.pane_id == fact.pane_id && existing.provider == fact.provider)
    {
        *existing = fact;
    } else {
        facts.push(fact);
    }
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;

    fn registration(activity: &str) -> AiPaneActivityRegistration {
        AiPaneActivityRegistration {
            provider: "codex".into(),
            pane_id: "terminal:5".into(),
            activity: activity.into(),
            state: None,
        }
    }

    // Defends: legacy activity tokens map into the normalized status-bus state taxonomy.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn normalizes_legacy_ai_activity_tokens_to_status_states() {
        assert_eq!(
            normalized_ai_activity_state(&registration("streaming")),
            Some(SessionAiPaneActivityState::Active)
        );
        assert_eq!(
            normalized_ai_activity_state(&registration("thinking")),
            Some(SessionAiPaneActivityState::Thinking)
        );
        assert_eq!(normalized_ai_activity_state(&registration("busy")), None);
    }

    // Defends: repeated activity observations update a tab-local provider/pane fact instead of duplicating it.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn upserts_ai_activity_by_provider_and_pane_identity() {
        let mut facts = vec![SessionAiPaneActivity::tab_local(
            1,
            "codex".into(),
            "terminal:5".into(),
            SessionAiPaneActivityState::Active,
        )];

        upsert_ai_pane_activity_fact(
            &mut facts,
            SessionAiPaneActivity::tab_local(
                1,
                "codex".into(),
                "terminal:5".into(),
                SessionAiPaneActivityState::Thinking,
            ),
        );

        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].state, SessionAiPaneActivityState::Thinking);
    }
}
