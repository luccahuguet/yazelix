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

#[cfg(test)]
mod tests {
    // Test lane: default
    // Defends: stable session snapshot JSON keeps schema_version and required object keys.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    use super::*;

    #[test]
    fn v1_snapshot_serializes_expected_top_level_keys() {
        let snap = ActiveTabSessionStateV1 {
            schema_version: 1,
            active_tab_position: 2,
            workspace: Some(SessionWorkspace {
                root: "/tmp/ws".into(),
                source: "explicit".into(),
            }),
            managed_panes: SessionManagedPanes {
                editor_pane_id: Some("terminal:1".into()),
                sidebar_pane_id: Some("terminal:2".into()),
            },
            focus_context: "editor".into(),
            layout: SessionLayout {
                active_swap_layout_name: Some("default_sidebar".into()),
                sidebar_collapsed: Some(false),
            },
            sidebar_yazi: Some(SessionSidebarYazi {
                yazi_id: "y1".into(),
                cwd: "/tmp/ws".into(),
            }),
        };
        let v: serde_json::Value = serde_json::to_value(&snap).expect("serialize");
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["active_tab_position"], 2);
        assert!(v.get("workspace").is_some());
        assert!(v.get("managed_panes").is_some());
        assert_eq!(v["focus_context"], "editor");
        assert!(v.get("layout").is_some());
        assert!(v.get("sidebar_yazi").is_some());
    }
}
