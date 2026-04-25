//! Versioned JSON snapshot for `get_active_tab_session_state` (bead `yazelix-0w1u.1`).

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionWorkspace {
    pub root: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionManagedPanes {
    pub editor_pane_id: Option<String>,
    pub sidebar_pane_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionLayout {
    pub active_swap_layout_name: Option<String>,
    pub sidebar_collapsed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SessionSidebarYazi {
    pub yazi_id: String,
    pub cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveTabReadState {
    pub explicit_workspace: Option<SessionWorkspace>,
    pub bootstrap_workspace: Option<SessionWorkspace>,
    pub editor_pane_id: Option<String>,
    pub sidebar_pane_id: Option<String>,
    pub focus_context: String,
    pub active_swap_layout_name: Option<String>,
    pub sidebar_collapsed: Option<bool>,
    pub sidebar_yazi: Option<SessionSidebarYazi>,
}

/// Stable v1 payload for the active tab. Serialized to JSON for the pipe response.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ActiveTabSessionStateV1 {
    pub schema_version: i32,
    pub active_tab_position: usize,
    pub workspace: Option<SessionWorkspace>,
    pub managed_panes: SessionManagedPanes,
    pub focus_context: String,
    pub layout: SessionLayout,
    pub sidebar_yazi: Option<SessionSidebarYazi>,
}

pub fn build_active_tab_session_state_v1(
    active_tab_position: usize,
    read_state: ActiveTabReadState,
) -> ActiveTabSessionStateV1 {
    ActiveTabSessionStateV1 {
        schema_version: 1,
        active_tab_position,
        workspace: read_state
            .explicit_workspace
            .or(read_state.bootstrap_workspace),
        managed_panes: SessionManagedPanes {
            editor_pane_id: read_state.editor_pane_id,
            sidebar_pane_id: read_state.sidebar_pane_id,
        },
        focus_context: read_state.focus_context,
        layout: SessionLayout {
            active_swap_layout_name: read_state.active_swap_layout_name,
            sidebar_collapsed: read_state.sidebar_collapsed,
        },
        sidebar_yazi: read_state.sidebar_yazi,
    }
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;

    // Regression: the stable active-tab snapshot must prefer explicit workspace truth over bootstrap fallback.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn session_snapshot_prefers_explicit_workspace_and_keeps_typed_session_fields() {
        let snapshot = build_active_tab_session_state_v1(
            3,
            ActiveTabReadState {
                explicit_workspace: Some(SessionWorkspace {
                    root: "/tmp/project".into(),
                    source: "explicit".into(),
                }),
                bootstrap_workspace: Some(SessionWorkspace {
                    root: "/tmp/bootstrap".into(),
                    source: "bootstrap".into(),
                }),
                editor_pane_id: Some("terminal:7".into()),
                sidebar_pane_id: Some("terminal:8".into()),
                focus_context: "sidebar".into(),
                active_swap_layout_name: Some("single_closed".into()),
                sidebar_collapsed: Some(true),
                sidebar_yazi: Some(SessionSidebarYazi {
                    yazi_id: "sidebar-123".into(),
                    cwd: "/tmp/project".into(),
                }),
            },
        );

        assert_eq!(snapshot.schema_version, 1);
        assert_eq!(snapshot.active_tab_position, 3);
        assert_eq!(
            snapshot.workspace,
            Some(SessionWorkspace {
                root: "/tmp/project".into(),
                source: "explicit".into(),
            })
        );
        assert_eq!(
            snapshot.managed_panes,
            SessionManagedPanes {
                editor_pane_id: Some("terminal:7".into()),
                sidebar_pane_id: Some("terminal:8".into()),
            }
        );
        assert_eq!(snapshot.focus_context, "sidebar");
        assert_eq!(
            snapshot.layout,
            SessionLayout {
                active_swap_layout_name: Some("single_closed".into()),
                sidebar_collapsed: Some(true),
            }
        );
        assert_eq!(
            snapshot.sidebar_yazi,
            Some(SessionSidebarYazi {
                yazi_id: "sidebar-123".into(),
                cwd: "/tmp/project".into(),
            })
        );
    }

    // Invariant: bootstrap workspace remains the fallback only when no explicit workspace state exists for the tab.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn session_snapshot_falls_back_to_bootstrap_workspace_when_explicit_is_missing() {
        let snapshot = build_active_tab_session_state_v1(
            1,
            ActiveTabReadState {
                explicit_workspace: None,
                bootstrap_workspace: Some(SessionWorkspace {
                    root: "/tmp/bootstrap".into(),
                    source: "bootstrap".into(),
                }),
                editor_pane_id: None,
                sidebar_pane_id: Some("terminal:9".into()),
                focus_context: "other".into(),
                active_swap_layout_name: None,
                sidebar_collapsed: None,
                sidebar_yazi: None,
            },
        );

        assert_eq!(
            snapshot.workspace,
            Some(SessionWorkspace {
                root: "/tmp/bootstrap".into(),
                source: "bootstrap".into(),
            })
        );
        assert_eq!(
            snapshot.managed_panes.sidebar_pane_id,
            Some("terminal:9".into())
        );
        assert_eq!(snapshot.focus_context, "other");
        assert_eq!(snapshot.sidebar_yazi, None);
    }
}
