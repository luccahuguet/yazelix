use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientPaneKind {
    Popup,
    Menu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransientPaneIdentityContract {
    pub pane_title: &'static str,
    pub command_marker: &'static str,
    pub wrapper_relative_path: &'static str,
}

impl TransientPaneKind {
    pub fn from_payload(payload: &str) -> Option<Self> {
        match payload.trim() {
            "popup" => Some(Self::Popup),
            "menu" => Some(Self::Menu),
            _ => None,
        }
    }

    pub fn identity(&self) -> TransientPaneIdentityContract {
        match self {
            Self::Popup => TransientPaneIdentityContract {
                pane_title: "yzx_popup",
                command_marker: "nushell/scripts/zellij_wrappers/yzx_popup_program.nu",
                wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_popup_program.nu",
            },
            Self::Menu => TransientPaneIdentityContract {
                pane_title: "yzx_menu",
                command_marker: "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu",
                wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu",
            },
        }
    }

    pub fn title(&self) -> &'static str {
        self.identity().pane_title
    }

    pub fn command_marker(&self) -> &'static str {
        self.identity().command_marker
    }

    pub fn wrapper_relative_path(&self) -> &'static str {
        self.identity().wrapper_relative_path
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
    Close(Id),
}

pub fn select_transient_pane<Id: Copy>(
    panes: &[TransientPaneSnapshot<'_, Id>],
    kind: TransientPaneKind,
) -> Option<TransientPaneState<Id>> {
    panes
        .iter()
        .filter(|pane| {
            !pane.is_plugin
                && !pane.exited
                && pane.is_floating
                && (pane.title.trim() == kind.title()
                    || pane
                        .terminal_command
                        .map(|command| command.contains(kind.command_marker()))
                        .unwrap_or(false))
        })
        .max_by_key(|pane| pane.is_focused)
        .map(|pane| TransientPaneState {
            pane_id: pane.pane_id,
            is_focused: pane.is_focused,
        })
}

pub fn resolve_transient_toggle_plan<Id: Copy>(
    panes: &[TransientPaneSnapshot<'_, Id>],
    kind: TransientPaneKind,
) -> TransientTogglePlan<Id> {
    match select_transient_pane(panes, kind) {
        Some(pane) if pane.is_focused => TransientTogglePlan::Close(pane.pane_id),
        Some(pane) => TransientTogglePlan::Focus(pane.pane_id),
        None => TransientTogglePlan::Open,
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::{
        resolve_transient_toggle_plan, select_transient_pane, TransientPaneIdentityContract,
        TransientPaneKind, TransientPaneSnapshot, TransientPaneState, TransientTogglePlan,
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

    // Defends: popup transient panes are discoverable by either pane title or the canonical runtime wrapper marker.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn selects_popup_by_title_or_command_marker() {
        let popup_by_title = [transient_pane(7, "yzx_popup", Some("nu"), false)];
        assert_eq!(
            select_transient_pane(&popup_by_title, TransientPaneKind::Popup),
            Some(TransientPaneState {
                pane_id: 7,
                is_focused: false,
            })
        );

        let popup_by_command = [transient_pane(
            8,
            "misc",
            Some("/tmp/runtime/nushell/scripts/zellij_wrappers/yzx_popup_program.nu lazygit"),
            false,
        )];
        assert_eq!(
            select_transient_pane(&popup_by_command, TransientPaneKind::Popup),
            Some(TransientPaneState {
                pane_id: 8,
                is_focused: false,
            })
        );
    }

    // Defends: popup and menu transient panes expose an explicit runtime wrapper identity contract for launcher code.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=1 total=7/10
    #[test]
    fn exposes_explicit_identity_contract_for_popup_and_menu() {
        assert_eq!(
            TransientPaneKind::Popup.identity(),
            TransientPaneIdentityContract {
                pane_title: "yzx_popup",
                command_marker: "nushell/scripts/zellij_wrappers/yzx_popup_program.nu",
                wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_popup_program.nu",
            }
        );

        assert_eq!(
            TransientPaneKind::Menu.identity(),
            TransientPaneIdentityContract {
                pane_title: "yzx_menu",
                command_marker: "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu",
                wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu",
            }
        );
    }

    // Defends: menu transient panes are discoverable by either pane title or the canonical runtime wrapper marker.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn selects_menu_by_title_or_command_marker() {
        let menu_by_title = [transient_pane(3, "yzx_menu", Some("nu"), false)];
        assert_eq!(
            select_transient_pane(&menu_by_title, TransientPaneKind::Menu),
            Some(TransientPaneState {
                pane_id: 3,
                is_focused: false,
            })
        );

        let menu_by_command = [transient_pane(
            4,
            "other",
            Some("/tmp/runtime/nushell/scripts/zellij_wrappers/yzx_menu_popup.nu"),
            false,
        )];
        assert_eq!(
            select_transient_pane(&menu_by_command, TransientPaneKind::Menu),
            Some(TransientPaneState {
                pane_id: 4,
                is_focused: false,
            })
        );
    }

    // Defends: focused transient panes win over unfocused duplicates during transient lookup.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn prefers_focused_transient_pane_when_duplicates_exist() {
        let panes = [
            transient_pane(1, "yzx_menu", Some("yzx_menu_popup.nu"), false),
            transient_pane(2, "yzx_menu", Some("yzx_menu_popup.nu"), true),
        ];

        assert_eq!(
            select_transient_pane(&panes, TransientPaneKind::Menu),
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
                title: "yzx_popup",
                terminal_command: Some("yzx_popup_program.nu"),
                is_plugin: false,
                exited: false,
                is_floating: false,
                is_focused: false,
            },
            transient_pane(2, "editor", Some("hx"), true),
        ];

        assert_eq!(
            select_transient_pane(&panes, TransientPaneKind::Popup),
            None
        );
    }

    // Defends: transient toggle planning distinguishes missing, present, and focused panes.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn resolves_toggle_plan_for_missing_present_and_focused_panes() {
        let missing: [TransientPaneSnapshot<'_, i32>; 0] = [];
        assert_eq!(
            resolve_transient_toggle_plan(&missing, TransientPaneKind::Popup),
            TransientTogglePlan::Open
        );

        let present = [transient_pane(
            5,
            "yzx_popup",
            Some("yzx_popup_program.nu"),
            false,
        )];
        assert_eq!(
            resolve_transient_toggle_plan(&present, TransientPaneKind::Popup),
            TransientTogglePlan::Focus(5)
        );

        let focused = [transient_pane(
            6,
            "yzx_popup",
            Some("yzx_popup_program.nu"),
            true,
        )];
        assert_eq!(
            resolve_transient_toggle_plan(&focused, TransientPaneKind::Popup),
            TransientTogglePlan::Close(6)
        );
    }
}
