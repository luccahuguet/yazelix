use std::collections::HashMap;

pub fn retain_tab_local_sidebar_pane_state<S>(
    state_by_tab: &mut HashMap<usize, S>,
    sidebar_pane_id_by_tab: &HashMap<usize, String>,
    state_pane_id: impl Fn(&S) -> &str,
) {
    state_by_tab.retain(|tab_position, state| {
        sidebar_pane_id_by_tab
            .get(tab_position)
            .map(|pane_id| pane_id == state_pane_id(state))
            .unwrap_or(false)
    });
}

pub fn find_tab_for_sidebar_pane_id(
    sidebar_pane_id_by_tab: &HashMap<usize, String>,
    pane_id: &str,
) -> Option<usize> {
    sidebar_pane_id_by_tab
        .iter()
        .find_map(|(tab_position, candidate_pane_id)| {
            if candidate_pane_id == pane_id {
                Some(*tab_position)
            } else {
                None
            }
        })
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{find_tab_for_sidebar_pane_id, retain_tab_local_sidebar_pane_state};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct SidebarAssociatedState {
        pane_id: String,
        value: &'static str,
    }

    // Defends: sidebar-associated state is retained only for the same tab and same live pane.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn retain_sidebar_state_requires_same_tab_and_same_live_pane() {
        let live_sidebar_pane_id_by_tab =
            HashMap::from([(1, "terminal:1".to_string()), (2, "terminal:2".to_string())]);
        let mut state_by_tab = HashMap::from([
            (
                1,
                SidebarAssociatedState {
                    pane_id: "terminal:1".to_string(),
                    value: "kept",
                },
            ),
            (
                2,
                SidebarAssociatedState {
                    pane_id: "terminal:old".to_string(),
                    value: "stale_same_tab",
                },
            ),
            (
                3,
                SidebarAssociatedState {
                    pane_id: "terminal:2".to_string(),
                    value: "stale_global_match",
                },
            ),
        ]);

        retain_tab_local_sidebar_pane_state(
            &mut state_by_tab,
            &live_sidebar_pane_id_by_tab,
            |state| state.pane_id.as_str(),
        );

        assert_eq!(
            state_by_tab,
            HashMap::from([(
                1,
                SidebarAssociatedState {
                    pane_id: "terminal:1".to_string(),
                    value: "kept",
                },
            )])
        );
    }

    // Defends: registration resolves to the tab that currently owns the sidebar pane id.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn find_sidebar_pane_owner_uses_live_pane_identity() {
        let live_sidebar_pane_id_by_tab =
            HashMap::from([(1, "terminal:1".to_string()), (2, "terminal:2".to_string())]);

        assert_eq!(
            find_tab_for_sidebar_pane_id(&live_sidebar_pane_id_by_tab, "terminal:2"),
            Some(2)
        );
        assert_eq!(
            find_tab_for_sidebar_pane_id(&live_sidebar_pane_id_by_tab, "terminal:missing"),
            None
        );
    }
}
