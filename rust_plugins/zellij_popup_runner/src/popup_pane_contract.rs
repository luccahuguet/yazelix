use zellij_tile::prelude::{PaneId, PaneInfo};

pub const POPUP_WRAPPER_MARKER: &str = "yzx_popup_program.nu";
pub const POPUP_PANE_TITLE: &str = "yzx_popup";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PopupPaneState {
    pub pane_id: PaneId,
    pub is_focused: bool,
}

pub fn select_popup_pane(panes: &[PaneInfo]) -> Option<PopupPaneState> {
    panes
        .iter()
        .filter(|pane| {
            !pane.is_plugin
                && !pane.exited
                && pane.is_floating
                && (pane.title == POPUP_PANE_TITLE
                    || pane
                        .terminal_command
                        .as_deref()
                        .map(|command| command.contains(POPUP_WRAPPER_MARKER))
                        .unwrap_or(false))
        })
        .max_by_key(|pane| pane.is_focused)
        .map(|pane| PopupPaneState {
            pane_id: PaneId::Terminal(pane.id),
            is_focused: pane.is_focused,
        })
}

#[cfg(test)]
mod tests {
    use super::{select_popup_pane, PopupPaneState, POPUP_PANE_TITLE, POPUP_WRAPPER_MARKER};
    use zellij_tile::prelude::{PaneId, PaneInfo};

    fn popup_pane(command: Option<&str>, is_focused: bool) -> PaneInfo {
        PaneInfo {
            id: if is_focused { 2 } else { 1 },
            terminal_command: command.map(str::to_string),
            title: POPUP_PANE_TITLE.to_string(),
            is_floating: true,
            is_focused,
            ..Default::default()
        }
    }

    #[test]
    fn selects_popup_by_title_when_terminal_command_is_generic() {
        let panes = vec![PaneInfo {
            id: 7,
            title: POPUP_PANE_TITLE.to_string(),
            terminal_command: Some("nu".to_string()),
            is_floating: true,
            ..Default::default()
        }];

        assert_eq!(
            select_popup_pane(&panes),
            Some(PopupPaneState {
                pane_id: PaneId::Terminal(7),
                is_focused: false
            })
        );
    }

    #[test]
    fn selects_popup_by_wrapper_command() {
        let panes = vec![popup_pane(
            Some(&format!(
                "nu /tmp/runtime/configs/zellij/scripts/{POPUP_WRAPPER_MARKER}"
            )),
            false,
        )];

        assert_eq!(
            select_popup_pane(&panes),
            Some(PopupPaneState {
                pane_id: PaneId::Terminal(1),
                is_focused: false
            })
        );
    }

    #[test]
    fn prefers_focused_popup_when_duplicates_exist() {
        let panes = vec![
            popup_pane(Some(POPUP_WRAPPER_MARKER), false),
            popup_pane(Some(POPUP_WRAPPER_MARKER), true),
        ];

        assert_eq!(
            select_popup_pane(&panes),
            Some(PopupPaneState {
                pane_id: PaneId::Terminal(2),
                is_focused: true
            })
        );
    }

    #[test]
    fn ignores_non_popup_floating_panes() {
        let panes = vec![PaneInfo {
            id: 9,
            title: "other".to_string(),
            terminal_command: Some("nu /tmp/other_script.nu".to_string()),
            is_floating: true,
            is_focused: true,
            ..Default::default()
        }];

        assert_eq!(select_popup_pane(&panes), None);
    }
}
