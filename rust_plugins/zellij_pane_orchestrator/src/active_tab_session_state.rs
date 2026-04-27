//! Versioned JSON snapshot for `get_active_tab_session_state`.
//!
//! Compatibility policy:
//! - `schema_version == 1` may add optional fields, but must not rename or remove existing fields.
//! - Breaking field shape changes require a new schema version and a compatibility producer.
//! - The schema carries session facts only. Presentation strings, colors, and bar/widget formatting
//!   belong to consumers such as `yazelix_bar`.

use serde::{Deserialize, Serialize};

pub const ACTIVE_TAB_SESSION_SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionWorkspace {
    /// Owned live state: workspace root selected by the orchestrator for this tab.
    pub root: String,
    /// Adapter state: where the root came from, currently `explicit` or `bootstrap`.
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionManagedPanes {
    /// Owned live state: managed editor pane identity, when present.
    pub editor_pane_id: Option<String>,
    /// Owned live state: managed sidebar pane identity, when present.
    pub sidebar_pane_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionLayout {
    /// Derived state: active Zellij swap layout name reported for the active tab.
    pub active_swap_layout_name: Option<String>,
    /// Derived state: sidebar visibility resolved from the active Yazelix layout family.
    pub sidebar_collapsed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionSidebarYazi {
    /// Owned live state: Yazi instance registered by the sidebar wrapper.
    pub yazi_id: String,
    /// Owned live state: latest cwd reported by that Yazi instance.
    pub cwd: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionTransientPane {
    /// Derived state: transient pane identity discovered from the live pane manifest.
    pub pane_id: String,
    /// Derived state: whether the transient pane currently owns terminal focus.
    pub is_focused: bool,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionTransientPanes {
    /// Derived state: currently visible Yazelix popup pane, if any.
    pub popup: Option<SessionTransientPane>,
    /// Derived state: currently visible Yazelix menu pane, if any.
    pub menu: Option<SessionTransientPane>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionAiPaneActivityState {
    Unknown,
    Inactive,
    Active,
    Thinking,
    Stale,
}

impl SessionAiPaneActivityState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Inactive => "inactive",
            Self::Active => "active",
            Self::Thinking => "thinking",
            Self::Stale => "stale",
        }
    }

    pub fn from_activity(activity: &str) -> Option<Self> {
        match activity.trim() {
            "unknown" => Some(Self::Unknown),
            "inactive" | "idle" => Some(Self::Inactive),
            "active" | "streaming" => Some(Self::Active),
            "thinking" => Some(Self::Thinking),
            "stale" => Some(Self::Stale),
            _ => None,
        }
    }
}

impl Default for SessionAiPaneActivityState {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionAiPaneActivity {
    /// Adapter state: tab position this activity fact belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_position: Option<usize>,
    /// Adapter state: provider label supplied by a future AI pane activity detector.
    #[serde(default)]
    pub provider: String,
    /// Adapter state: pane identity associated with the activity signal.
    #[serde(default)]
    pub pane_id: String,
    /// Adapter state: legacy stable activity token, retained for schema-v1 compatibility.
    #[serde(default)]
    pub activity: String,
    /// Adapter state: normalized activity state for status-bus consumers.
    #[serde(default)]
    pub state: SessionAiPaneActivityState,
}

impl SessionAiPaneActivity {
    pub fn tab_local(
        tab_position: usize,
        provider: String,
        pane_id: String,
        state: SessionAiPaneActivityState,
    ) -> Self {
        Self {
            tab_position: Some(tab_position),
            provider,
            pane_id,
            activity: state.as_str().to_string(),
            state,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionAiTokenBudget {
    /// Adapter state: tab position this token-budget fact belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_position: Option<usize>,
    /// Adapter state: provider label supplied by a future token-budget detector.
    #[serde(default)]
    pub provider: String,
    /// Adapter state: known remaining context tokens, when a provider adapter can report it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remaining_tokens: Option<u64>,
    /// Adapter state: known total context tokens, when a provider adapter can report it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionStatusExtensions {
    /// Extension slot for future AI pane indicators. Empty means unknown, not idle.
    #[serde(default)]
    pub ai_pane_activity: Vec<SessionAiPaneActivity>,
    /// Extension slot for future provider token-budget adapters. Empty means unknown.
    #[serde(default)]
    pub ai_token_budget: Vec<SessionAiTokenBudget>,
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
    pub transient_panes: SessionTransientPanes,
    pub extensions: SessionStatusExtensions,
}

/// Stable v1 payload for the active tab. Serialized to JSON for the pipe response.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct ActiveTabSessionStateV1 {
    pub schema_version: i32,
    pub active_tab_position: usize,
    pub workspace: Option<SessionWorkspace>,
    pub managed_panes: SessionManagedPanes,
    pub focus_context: String,
    pub layout: SessionLayout,
    pub sidebar_yazi: Option<SessionSidebarYazi>,
    #[serde(default)]
    pub transient_panes: SessionTransientPanes,
    #[serde(default)]
    pub extensions: SessionStatusExtensions,
}

pub fn build_active_tab_session_state_v1(
    active_tab_position: usize,
    read_state: ActiveTabReadState,
) -> ActiveTabSessionStateV1 {
    ActiveTabSessionStateV1 {
        schema_version: ACTIVE_TAB_SESSION_SCHEMA_VERSION,
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
        transient_panes: read_state.transient_panes,
        extensions: read_state.extensions,
    }
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use serde_json::json;

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
                transient_panes: SessionTransientPanes {
                    popup: Some(SessionTransientPane {
                        pane_id: "terminal:11".into(),
                        is_focused: false,
                    }),
                    menu: None,
                },
                extensions: SessionStatusExtensions::default(),
            },
        );

        assert_eq!(snapshot.schema_version, ACTIVE_TAB_SESSION_SCHEMA_VERSION);
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
        assert_eq!(
            snapshot.transient_panes.popup,
            Some(SessionTransientPane {
                pane_id: "terminal:11".into(),
                is_focused: false,
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
                transient_panes: SessionTransientPanes::default(),
                extensions: SessionStatusExtensions::default(),
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
        assert_eq!(snapshot.transient_panes, SessionTransientPanes::default());
    }

    // Defends: additive v1 fields remain readable by consumers replaying older active-tab payload fixtures.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn deserializes_older_v1_payloads_with_default_extension_fields() {
        let decoded: ActiveTabSessionStateV1 = serde_json::from_value(json!({
            "schema_version": ACTIVE_TAB_SESSION_SCHEMA_VERSION,
            "active_tab_position": 1,
            "workspace": null,
            "managed_panes": {
                "editor_pane_id": null,
                "sidebar_pane_id": null
            },
            "focus_context": "other",
            "layout": {
                "active_swap_layout_name": null,
                "sidebar_collapsed": null
            },
            "sidebar_yazi": null
        }))
        .unwrap();

        assert_eq!(decoded.transient_panes, SessionTransientPanes::default());
        assert_eq!(decoded.extensions, SessionStatusExtensions::default());
    }

    // Defends: the status bus exposes stable session facts without embedding bar/zjstatus formatting.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn serializes_representative_payload_without_presentation_formatting() {
        let snapshot = build_active_tab_session_state_v1(
            2,
            ActiveTabReadState {
                explicit_workspace: Some(SessionWorkspace {
                    root: "/repo".into(),
                    source: "explicit".into(),
                }),
                bootstrap_workspace: None,
                editor_pane_id: Some("terminal:1".into()),
                sidebar_pane_id: Some("terminal:2".into()),
                focus_context: "editor".into(),
                active_swap_layout_name: Some("single_open".into()),
                sidebar_collapsed: Some(false),
                sidebar_yazi: None,
                transient_panes: SessionTransientPanes {
                    popup: None,
                    menu: Some(SessionTransientPane {
                        pane_id: "terminal:9".into(),
                        is_focused: true,
                    }),
                },
                extensions: SessionStatusExtensions {
                    ai_pane_activity: vec![SessionAiPaneActivity::tab_local(
                        2,
                        "codex".into(),
                        "terminal:4".into(),
                        SessionAiPaneActivityState::Thinking,
                    )],
                    ai_token_budget: vec![SessionAiTokenBudget {
                        tab_position: Some(2),
                        provider: "codex".into(),
                        remaining_tokens: Some(120_000),
                        total_tokens: Some(200_000),
                    }],
                },
            },
        );

        let serialized = serde_json::to_string(&snapshot).unwrap();
        let decoded: ActiveTabSessionStateV1 = serde_json::from_str(&serialized).unwrap();
        let value = serde_json::to_value(&snapshot).unwrap();

        assert_eq!(decoded, snapshot);
        assert_eq!(
            value,
            json!({
                "schema_version": ACTIVE_TAB_SESSION_SCHEMA_VERSION,
                "active_tab_position": 2,
                "workspace": {
                    "root": "/repo",
                    "source": "explicit"
                },
                "managed_panes": {
                    "editor_pane_id": "terminal:1",
                    "sidebar_pane_id": "terminal:2"
                },
                "focus_context": "editor",
                "layout": {
                    "active_swap_layout_name": "single_open",
                    "sidebar_collapsed": false
                },
                "sidebar_yazi": null,
                "transient_panes": {
                    "popup": null,
                    "menu": {
                        "pane_id": "terminal:9",
                        "is_focused": true
                    }
                },
                "extensions": {
                    "ai_pane_activity": [
                        {
                            "tab_position": 2,
                            "provider": "codex",
                            "pane_id": "terminal:4",
                            "activity": "thinking",
                            "state": "thinking"
                        }
                    ],
                    "ai_token_budget": [
                        {
                            "tab_position": 2,
                            "provider": "codex",
                            "remaining_tokens": 120000,
                            "total_tokens": 200000
                        }
                    ]
                }
            })
        );
        assert!(!serialized.contains("#["));
        assert!(!serialized.contains("command_cpu"));
        assert!(!serialized.contains("zjstatus"));
    }

    // Defends: the AI activity extension has an explicit tab-local state taxonomy without provider UI formatting.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn ai_activity_extension_represents_tab_local_state_taxonomy() {
        let states = [
            SessionAiPaneActivityState::Inactive,
            SessionAiPaneActivityState::Active,
            SessionAiPaneActivityState::Thinking,
            SessionAiPaneActivityState::Stale,
            SessionAiPaneActivityState::Unknown,
        ];

        let facts = states
            .iter()
            .map(|state| {
                SessionAiPaneActivity::tab_local(4, "codex".into(), "terminal:12".into(), *state)
            })
            .collect::<Vec<_>>();
        let value = serde_json::to_value(SessionStatusExtensions {
            ai_pane_activity: facts,
            ai_token_budget: Vec::new(),
        })
        .unwrap();

        assert_eq!(
            value
                .get("ai_pane_activity")
                .and_then(|value| value.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.get("state").and_then(|state| state.as_str()))
                        .collect::<Vec<_>>()
                })
                .unwrap(),
            vec!["inactive", "active", "thinking", "stale", "unknown"]
        );
        assert!(!value.to_string().contains("#["));
    }
}
