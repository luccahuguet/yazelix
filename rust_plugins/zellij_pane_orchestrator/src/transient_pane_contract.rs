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

impl TransientPaneIdentityContract {
    pub fn as_view(&self) -> TransientPaneIdentityView<'_> {
        TransientPaneIdentityView {
            pane_title: self.pane_title,
            command_marker: self.command_marker,
        }
    }
}

pub fn transient_pane_identity(kind: TransientPaneKind) -> TransientPaneIdentityContract {
    match kind {
        TransientPaneKind::Popup => TransientPaneIdentityContract {
            pane_title: "yzx_popup",
            command_marker: None,
        },
        TransientPaneKind::Menu => TransientPaneIdentityContract {
            pane_title: "yzx_menu",
            command_marker: None,
        },
        TransientPaneKind::Config => TransientPaneIdentityContract {
            pane_title: "yzx_config",
            command_marker: None,
        },
    }
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

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::{
        select_transient_pane, transient_pane_identity, TransientPaneIdentityContract,
        TransientPaneKind, TransientPaneSnapshot, TransientPaneState,
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

    // Defends: built-in Yazelix transient identity follows yzpp-managed pane titles.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn built_in_identity_uses_yzpp_pane_titles() {
        assert_eq!(
            transient_pane_identity(TransientPaneKind::Popup),
            TransientPaneIdentityContract {
                pane_title: "yzx_popup",
                command_marker: None,
            }
        );
        assert_eq!(
            transient_pane_identity(TransientPaneKind::Menu),
            TransientPaneIdentityContract {
                pane_title: "yzx_menu",
                command_marker: None,
            }
        );
        assert_eq!(
            transient_pane_identity(TransientPaneKind::Config),
            TransientPaneIdentityContract {
                pane_title: "yzx_config",
                command_marker: None,
            }
        );
    }
}
