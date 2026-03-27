#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaneSnapshot<'a> {
    pub title: &'a str,
    pub is_plugin: bool,
    pub exited: bool,
    pub is_focused: bool,
    pub is_suppressed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusContextPolicy {
    Editor,
    Sidebar,
    Other,
}

pub fn select_managed_pane_index(
    panes: &[PaneSnapshot<'_>],
    expected_title: &str,
) -> Option<usize> {
    let matching_indexes: Vec<usize> = panes
        .iter()
        .enumerate()
        .filter(|(_, pane)| !pane.is_plugin)
        .filter(|(_, pane)| !pane.exited)
        .filter(|(_, pane)| pane.title.trim() == expected_title)
        .map(|(index, _)| index)
        .collect();

    matching_indexes
        .iter()
        .copied()
        .find(|index| panes[*index].is_focused)
        .or_else(|| {
            matching_indexes
                .iter()
                .copied()
                .find(|index| !panes[*index].is_suppressed)
        })
        .or_else(|| matching_indexes.first().copied())
}

pub fn resolve_focus_context(
    focused_title: Option<&str>,
    previous_focus_context: FocusContextPolicy,
) -> FocusContextPolicy {
    match focused_title.map(str::trim) {
        Some("editor") => FocusContextPolicy::Editor,
        Some("sidebar") => FocusContextPolicy::Sidebar,
        Some(title) if title.starts_with("yzx_") => previous_focus_context,
        Some(_) | None => FocusContextPolicy::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_focus_context, select_managed_pane_index, FocusContextPolicy, PaneSnapshot,
    };

    #[test]
    fn only_exact_editor_title_counts_as_managed_editor() {
        let panes = [
            PaneSnapshot {
                title: "hx",
                is_plugin: false,
                exited: false,
                is_focused: true,
                is_suppressed: false,
            },
            PaneSnapshot {
                title: "editor",
                is_plugin: false,
                exited: false,
                is_focused: false,
                is_suppressed: false,
            },
        ];

        assert_eq!(select_managed_pane_index(&panes, "editor"), Some(1));
        assert_eq!(select_managed_pane_index(&panes, "hx"), Some(0));
    }

    #[test]
    fn focused_managed_editor_wins_when_multiple_editor_titled_panes_exist() {
        let panes = [
            PaneSnapshot {
                title: "editor",
                is_plugin: false,
                exited: false,
                is_focused: false,
                is_suppressed: false,
            },
            PaneSnapshot {
                title: "editor",
                is_plugin: false,
                exited: false,
                is_focused: true,
                is_suppressed: false,
            },
        ];

        assert_eq!(select_managed_pane_index(&panes, "editor"), Some(1));
    }

    #[test]
    fn yzx_helper_panes_preserve_previous_focus_context() {
        assert_eq!(
            resolve_focus_context(Some("yzx_menu"), FocusContextPolicy::Editor),
            FocusContextPolicy::Editor
        );
        assert_eq!(
            resolve_focus_context(Some("something_else"), FocusContextPolicy::Sidebar),
            FocusContextPolicy::Other
        );
    }
}
