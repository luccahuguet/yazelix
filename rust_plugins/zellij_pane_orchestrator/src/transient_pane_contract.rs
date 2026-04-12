use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransientPaneKind {
    Popup,
    Menu,
}

impl TransientPaneKind {
    pub fn from_payload(payload: &str) -> Option<Self> {
        match payload.trim() {
            "popup" => Some(Self::Popup),
            "menu" => Some(Self::Menu),
            _ => None,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Popup => "yzx_popup",
            Self::Menu => "yzx_menu",
        }
    }

    pub fn wrapper_marker(&self) -> &'static str {
        match self {
            Self::Popup => "yzx_popup_program.nu",
            Self::Menu => "yzx_menu_popup.nu",
        }
    }

    pub fn wrapper_relative_path(&self) -> &'static str {
        match self {
            Self::Popup => "nushell/scripts/zellij_wrappers/yzx_popup_program.nu",
            Self::Menu => "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu",
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
                        .map(|command| command.contains(kind.wrapper_marker()))
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

#[cfg(test)]
mod tests {
    use super::{
        resolve_transient_toggle_plan, select_transient_pane, TransientPaneKind,
        TransientPaneSnapshot, TransientPaneState, TransientTogglePlan,
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

    #[test]
    fn selects_popup_by_title_or_wrapper_marker() {
        let popup_by_title = [transient_pane(7, "yzx_popup", Some("nu"), false)];
        assert_eq!(
            select_transient_pane(&popup_by_title, TransientPaneKind::Popup),
            Some(TransientPaneState {
                pane_id: 7,
                is_focused: false,
            })
        );

        let popup_by_wrapper = [transient_pane(
            8,
            "misc",
            Some("yazelix_nu.sh /tmp/runtime/nushell/scripts/zellij_wrappers/yzx_popup_program.nu"),
            false,
        )];
        assert_eq!(
            select_transient_pane(&popup_by_wrapper, TransientPaneKind::Popup),
            Some(TransientPaneState {
                pane_id: 8,
                is_focused: false,
            })
        );
    }

    #[test]
    fn selects_menu_by_title_or_wrapper_marker() {
        let menu_by_title = [transient_pane(3, "yzx_menu", Some("nu"), false)];
        assert_eq!(
            select_transient_pane(&menu_by_title, TransientPaneKind::Menu),
            Some(TransientPaneState {
                pane_id: 3,
                is_focused: false,
            })
        );

        let menu_by_wrapper = [transient_pane(
            4,
            "other",
            Some("yazelix_nu.sh /tmp/runtime/nushell/scripts/zellij_wrappers/yzx_menu_popup.nu"),
            false,
        )];
        assert_eq!(
            select_transient_pane(&menu_by_wrapper, TransientPaneKind::Menu),
            Some(TransientPaneState {
                pane_id: 4,
                is_focused: false,
            })
        );
    }

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
