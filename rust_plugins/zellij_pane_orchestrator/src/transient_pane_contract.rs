use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientPaneKind {
    Popup,
    Menu,
    Config,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientPaneIdentityContract {
    pub pane_title: &'static str,
    pub command_marker: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientPaneIdentityView<'a> {
    pub pane_title: &'a str,
    pub command_marker: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientPopupAction {
    Toggle,
    Open,
    Focus,
    Close,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransientPopupSpec {
    pub id: String,
    pub pane_title: String,
    #[serde(default)]
    pub command_marker: Option<String>,
    pub command: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    pub width_percent: usize,
    pub height_percent: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TransientPopupPipeRequest {
    pub action: TransientPopupAction,
    pub spec: TransientPopupSpec,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

impl TransientPaneKind {
    pub fn from_payload(payload: &str) -> Option<Self> {
        match payload.trim() {
            "popup" => Some(Self::Popup),
            "menu" => Some(Self::Menu),
            "config" => Some(Self::Config),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientPaneSnapshot<'a, Id> {
    pub pane_id: Id,
    pub title: &'a str,
    pub terminal_command: Option<&'a str>,
    pub is_plugin: bool,
    pub exited: bool,
    pub is_floating: bool,
    pub is_focused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientPaneState<Id> {
    pub pane_id: Id,
    pub is_focused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransientTogglePlan<Id> {
    Open,
    Focus(Id),
    CloseAndHideFloatingLayer(Id),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientPaneGeometry {
    pub width_percent: usize,
    pub height_percent: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransientPaneLaunchRequest {
    pub command_path: String,
    pub args: Vec<String>,
    pub requested_cwd: Option<String>,
    pub fallback_cwd: String,
    pub geometry: TransientPaneGeometry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransientPaneLaunchPlan {
    pub command_path: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub geometry: TransientPaneGeometry,
}

impl TransientPaneIdentityContract {
    pub fn as_view(&self) -> TransientPaneIdentityView<'_> {
        TransientPaneIdentityView {
            pane_title: self.pane_title,
            command_marker: self.command_marker,
        }
    }
}

impl TransientPopupSpec {
    pub fn identity(&self) -> TransientPaneIdentityView<'_> {
        TransientPaneIdentityView {
            pane_title: self.pane_title.as_str(),
            command_marker: self.command_marker.as_deref(),
        }
    }

    pub fn geometry(&self) -> Option<TransientPaneGeometry> {
        if !(1..=100).contains(&self.width_percent) || !(1..=100).contains(&self.height_percent) {
            return None;
        }

        Some(TransientPaneGeometry {
            width_percent: self.width_percent,
            height_percent: self.height_percent,
        })
    }
}

impl TransientPopupPipeRequest {
    pub fn launch_plan(&self, fallback_cwd: &str) -> Option<TransientPaneLaunchPlan> {
        if self.spec.id.trim().is_empty() || self.spec.pane_title.trim().is_empty() {
            return None;
        }

        let command_path = self.spec.command.first()?.clone();
        let mut args = self
            .spec
            .command
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<_>>();
        args.extend(self.args.iter().cloned());

        resolve_transient_launch_plan(TransientPaneLaunchRequest {
            command_path,
            args,
            requested_cwd: self.cwd.clone().or_else(|| self.spec.cwd.clone()),
            fallback_cwd: fallback_cwd.to_string(),
            geometry: self.spec.geometry()?,
        })
    }
}

pub fn resolve_transient_launch_plan(
    request: TransientPaneLaunchRequest,
) -> Option<TransientPaneLaunchPlan> {
    let command_path = request.command_path.trim();
    if command_path.is_empty() {
        return None;
    }
    let cwd = request
        .requested_cwd
        .as_deref()
        .map(str::trim)
        .filter(|cwd| !cwd.is_empty())
        .unwrap_or_else(|| request.fallback_cwd.trim());
    if cwd.is_empty() {
        return None;
    }

    Some(TransientPaneLaunchPlan {
        command_path: command_path.to_string(),
        args: request.args,
        cwd: cwd.to_string(),
        geometry: request.geometry,
    })
}

pub fn select_transient_pane<Id: Copy>(
    panes: &[TransientPaneSnapshot<'_, Id>],
    identity: TransientPaneIdentityContract,
) -> Option<TransientPaneState<Id>> {
    select_transient_pane_by_identity(panes, identity.as_view())
}

pub fn select_transient_pane_by_identity<Id: Copy>(
    panes: &[TransientPaneSnapshot<'_, Id>],
    identity: TransientPaneIdentityView<'_>,
) -> Option<TransientPaneState<Id>> {
    panes
        .iter()
        .filter(|pane| {
            !pane.is_plugin
                && !pane.exited
                && pane.is_floating
                && (pane.title.trim() == identity.pane_title
                    || identity.command_marker.is_some_and(|command_marker| {
                        pane.terminal_command
                            .map(|command| command.contains(command_marker))
                            .unwrap_or(false)
                    }))
        })
        .max_by_key(|pane| pane.is_focused)
        .map(|pane| TransientPaneState {
            pane_id: pane.pane_id,
            is_focused: pane.is_focused,
        })
}

pub fn resolve_transient_toggle_plan<Id: Copy>(
    panes: &[TransientPaneSnapshot<'_, Id>],
    identity: TransientPaneIdentityContract,
) -> TransientTogglePlan<Id> {
    resolve_transient_toggle_plan_by_identity(panes, identity.as_view())
}

pub fn resolve_transient_toggle_plan_by_identity<Id: Copy>(
    panes: &[TransientPaneSnapshot<'_, Id>],
    identity: TransientPaneIdentityView<'_>,
) -> TransientTogglePlan<Id> {
    match select_transient_pane_by_identity(panes, identity) {
        Some(pane) if pane.is_focused => {
            TransientTogglePlan::CloseAndHideFloatingLayer(pane.pane_id)
        }
        Some(pane) => TransientTogglePlan::Focus(pane.pane_id),
        None => TransientTogglePlan::Open,
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::{
        resolve_transient_launch_plan, resolve_transient_toggle_plan,
        resolve_transient_toggle_plan_by_identity, select_transient_pane,
        select_transient_pane_by_identity, TransientPaneGeometry, TransientPaneIdentityContract,
        TransientPaneLaunchPlan, TransientPaneLaunchRequest, TransientPaneSnapshot,
        TransientPaneState, TransientPopupAction, TransientPopupPipeRequest, TransientPopupSpec,
        TransientTogglePlan,
    };

    const POPUP_IDENTITY: TransientPaneIdentityContract = TransientPaneIdentityContract {
        pane_title: "floating_picker",
        command_marker: Some("picker_wrapper"),
    };

    const MENU_IDENTITY: TransientPaneIdentityContract = TransientPaneIdentityContract {
        pane_title: "floating_menu",
        command_marker: Some("menu_wrapper"),
    };

    fn transient_pane<'a>(
        pane_id: i32,
        title: &'a str,
        terminal_command: Option<&'a str>,
        is_focused: bool,
    ) -> TransientPaneSnapshot<'a, i32> {
        TransientPaneSnapshot {
            pane_id,
            title,
            terminal_command,
            is_plugin: false,
            exited: false,
            is_floating: true,
            is_focused,
        }
    }

    // Defends: generic transient panes are discoverable by either pane title or adapter-supplied command marker.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn selects_transient_pane_by_title_or_command_marker() {
        let popup_by_title = [transient_pane(7, "floating_picker", Some("nu"), false)];
        assert_eq!(
            select_transient_pane(&popup_by_title, POPUP_IDENTITY),
            Some(TransientPaneState {
                pane_id: 7,
                is_focused: false,
            })
        );

        let popup_by_command = [transient_pane(
            8,
            "misc",
            Some("/tmp/runtime/bin/picker_wrapper lazygit"),
            false,
        )];
        assert_eq!(
            select_transient_pane(&popup_by_command, POPUP_IDENTITY),
            Some(TransientPaneState {
                pane_id: 8,
                is_focused: false,
            })
        );
    }

    // Defends: adapters may omit command matching and still get exact title-based single-instance behavior.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn supports_title_only_identity_without_command_marker() {
        let identity = TransientPaneIdentityContract {
            pane_title: "floating_menu",
            command_marker: None,
        };
        let menu_by_title = [transient_pane(3, "floating_menu", Some("nu"), false)];
        assert_eq!(
            select_transient_pane(&menu_by_title, identity),
            Some(TransientPaneState {
                pane_id: 3,
                is_focused: false,
            })
        );

        let menu_by_command = [transient_pane(
            4,
            "other",
            Some("/tmp/runtime/bin/menu_wrapper"),
            false,
        )];
        assert_eq!(select_transient_pane(&menu_by_command, identity), None);
    }

    // Defends: focused transient panes win over unfocused duplicates during transient lookup.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn prefers_focused_transient_pane_when_duplicates_exist() {
        let panes = [
            transient_pane(1, "floating_menu", Some("menu_wrapper"), false),
            transient_pane(2, "floating_menu", Some("menu_wrapper"), true),
        ];

        assert_eq!(
            select_transient_pane(&panes, MENU_IDENTITY),
            Some(TransientPaneState {
                pane_id: 2,
                is_focused: true,
            })
        );
    }

    // Defends: transient lookup ignores non-floating or unrelated panes instead of matching by stale titles alone.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn ignores_non_floating_or_irrelevant_panes() {
        let panes = [
            TransientPaneSnapshot {
                pane_id: 1,
                title: "floating_picker",
                terminal_command: Some("picker_wrapper"),
                is_plugin: false,
                exited: false,
                is_floating: false,
                is_focused: false,
            },
            transient_pane(2, "editor", Some("hx"), true),
        ];

        assert_eq!(select_transient_pane(&panes, POPUP_IDENTITY), None);
    }

    // Defends: transient toggle planning distinguishes missing, present, and focused panes.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn resolves_toggle_plan_for_missing_present_and_focused_panes() {
        let missing: [TransientPaneSnapshot<'_, i32>; 0] = [];
        assert_eq!(
            resolve_transient_toggle_plan(&missing, POPUP_IDENTITY),
            TransientTogglePlan::Open
        );

        let present = [transient_pane(
            5,
            "floating_picker",
            Some("picker_wrapper"),
            false,
        )];
        assert_eq!(
            resolve_transient_toggle_plan(&present, POPUP_IDENTITY),
            TransientTogglePlan::Focus(5)
        );

        let focused = [transient_pane(
            6,
            "floating_picker",
            Some("picker_wrapper"),
            true,
        )];
        assert_eq!(
            resolve_transient_toggle_plan(&focused, POPUP_IDENTITY),
            TransientTogglePlan::CloseAndHideFloatingLayer(6)
        );
    }

    // Regression: closing a focused popup/menu must also hide the floating layer so unrelated floating panes do not stay visible.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn close_plan_hides_floating_layer_after_managed_transient_closes() {
        let panes = [
            transient_pane(6, "floating_picker", Some("picker_wrapper"), true),
            transient_pane(7, "unrelated_floating_tool", Some("htop"), false),
            TransientPaneSnapshot {
                pane_id: 8,
                title: "editor",
                terminal_command: Some("hx"),
                is_plugin: false,
                exited: false,
                is_floating: false,
                is_focused: false,
            },
        ];

        assert_eq!(
            resolve_transient_toggle_plan(&panes, POPUP_IDENTITY),
            TransientTogglePlan::CloseAndHideFloatingLayer(6)
        );
    }

    // Defends: generic launch policy trims command/cwd inputs while preserving adapter-provided argv and geometry.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn resolves_launch_plan_from_command_cwd_and_geometry() {
        let plan = resolve_transient_launch_plan(TransientPaneLaunchRequest {
            command_path: " /runtime/wrapper ".into(),
            args: vec!["lazygit".into(), "--help".into()],
            requested_cwd: Some(" /repo ".into()),
            fallback_cwd: "/runtime".into(),
            geometry: TransientPaneGeometry {
                width_percent: 80,
                height_percent: 70,
            },
        });

        assert_eq!(
            plan,
            Some(TransientPaneLaunchPlan {
                command_path: "/runtime/wrapper".into(),
                args: vec!["lazygit".into(), "--help".into()],
                cwd: "/repo".into(),
                geometry: TransientPaneGeometry {
                    width_percent: 80,
                    height_percent: 70,
                },
            })
        );
    }

    // Defends: a transient launch cannot silently open with missing adapter command or cwd data.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn rejects_launch_plan_without_command_or_cwd() {
        let geometry = TransientPaneGeometry {
            width_percent: 90,
            height_percent: 90,
        };

        assert_eq!(
            resolve_transient_launch_plan(TransientPaneLaunchRequest {
                command_path: " ".into(),
                args: vec![],
                requested_cwd: Some("/repo".into()),
                fallback_cwd: "/runtime".into(),
                geometry,
            }),
            None
        );
        assert_eq!(
            resolve_transient_launch_plan(TransientPaneLaunchRequest {
                command_path: "/runtime/wrapper".into(),
                args: vec![],
                requested_cwd: None,
                fallback_cwd: " ".into(),
                geometry,
            }),
            None
        );
    }

    fn generic_popup_spec() -> TransientPopupSpec {
        TransientPopupSpec {
            id: "gitui".into(),
            pane_title: "gitui_popup".into(),
            command_marker: Some("gitui".into()),
            command: vec!["gitui".into(), "--watch".into()],
            cwd: Some("/repo".into()),
            width_percent: 82,
            height_percent: 76,
        }
    }

    // Defends: standalone popup specs carry pane identity without depending on Yazelix popup/menu/config kind adapters.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn generic_popup_spec_drives_identity_without_yazelix_adapter() {
        let spec = generic_popup_spec();
        let panes = [transient_pane(
            11,
            "gitui_popup",
            Some("gitui --watch"),
            false,
        )];

        assert_eq!(
            select_transient_pane_by_identity(&panes, spec.identity()),
            Some(TransientPaneState {
                pane_id: 11,
                is_focused: false,
            })
        );
        assert_eq!(
            resolve_transient_toggle_plan_by_identity(&panes, spec.identity()),
            TransientTogglePlan::Focus(11)
        );
    }

    // Defends: the future plain-Zellij popup pipe schema resolves argv/cwd/geometry without Yazelix runtime wrapper paths.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn standalone_popup_pipe_request_resolves_launch_plan() {
        let payload = r#"{
            "action": "toggle",
            "spec": {
                "id": "gitui",
                "pane_title": "gitui_popup",
                "command_marker": "gitui",
                "command": ["gitui", "--watch"],
                "cwd": "/repo",
                "width_percent": 82,
                "height_percent": 76
            },
            "args": ["--log-level", "debug"],
            "cwd": "/override"
        }"#;
        let request = serde_json::from_str::<TransientPopupPipeRequest>(payload).unwrap();

        assert_eq!(request.action, TransientPopupAction::Toggle);
        assert_eq!(
            request.launch_plan("/fallback"),
            Some(TransientPaneLaunchPlan {
                command_path: "gitui".into(),
                args: vec!["--watch".into(), "--log-level".into(), "debug".into()],
                cwd: "/override".into(),
                geometry: TransientPaneGeometry {
                    width_percent: 82,
                    height_percent: 76,
                },
            })
        );
    }

    // Defends: the generic popup pipe schema is strict enough to reject accidental Yazelix-only adapter fields.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn standalone_popup_pipe_schema_rejects_unknown_fields() {
        let payload = r#"{
            "action": "toggle",
            "runtime_dir": "/tmp/yazelix",
            "spec": {
                "id": "gitui",
                "pane_title": "gitui_popup",
                "command_marker": "gitui",
                "command": ["gitui"],
                "cwd": "/repo",
                "width_percent": 90,
                "height_percent": 85
            }
        }"#;

        assert!(serde_json::from_str::<TransientPopupPipeRequest>(payload).is_err());
    }

    // Defends: generic popup specs fail closed when pane identity, command identity, or dimensions are not launchable.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn generic_popup_launch_plan_rejects_empty_identity_command_or_invalid_geometry() {
        let empty_identity = TransientPopupPipeRequest {
            action: TransientPopupAction::Open,
            spec: TransientPopupSpec {
                id: "".into(),
                ..generic_popup_spec()
            },
            args: vec![],
            cwd: None,
        };
        assert_eq!(empty_identity.launch_plan("/fallback"), None);

        let empty_command = TransientPopupPipeRequest {
            action: TransientPopupAction::Open,
            spec: TransientPopupSpec {
                command: vec![],
                ..generic_popup_spec()
            },
            args: vec![],
            cwd: None,
        };
        assert_eq!(empty_command.launch_plan("/fallback"), None);

        let invalid_geometry = TransientPopupPipeRequest {
            action: TransientPopupAction::Open,
            spec: TransientPopupSpec {
                width_percent: 101,
                ..generic_popup_spec()
            },
            args: vec![],
            cwd: None,
        };
        assert_eq!(invalid_geometry.launch_plan("/fallback"), None);
    }
}
