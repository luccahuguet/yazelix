#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HorizontalPaneSnapshot<'a> {
    pub title: &'a str,
    pub is_plugin: bool,
    pub exited: bool,
    pub is_focused: bool,
    pub pane_x: usize,
    pub pane_y: usize,
    pub pane_columns: usize,
    pub pane_rows: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalFocusPlan {
    FocusPane(usize),
    PreviousTab,
    NextTab,
    MissingFocusedPane,
}

pub fn resolve_horizontal_focus(
    panes: &[HorizontalPaneSnapshot<'_>],
    direction: HorizontalDirection,
    sidebar_is_closed: bool,
) -> HorizontalFocusPlan {
    let Some((focused_index, focused_pane)) = panes
        .iter()
        .enumerate()
        .find(|(_, pane)| !pane.is_plugin && !pane.exited && pane.is_focused)
    else {
        return HorizontalFocusPlan::MissingFocusedPane;
    };

    let current_left = focused_pane.pane_x;
    let current_right = focused_pane.pane_x + focused_pane.pane_columns;
    let current_top = focused_pane.pane_y;
    let current_bottom = focused_pane.pane_y + focused_pane.pane_rows;

    let candidates = panes
        .iter()
        .enumerate()
        .filter(|(index, _pane)| *index != focused_index)
        .filter(|(_, pane)| !pane.is_plugin && !pane.exited)
        .filter(|(_, pane)| !(sidebar_is_closed && pane.title.trim() == "sidebar"))
        .filter_map(|(index, pane)| {
            let candidate_left = pane.pane_x;
            let candidate_right = pane.pane_x + pane.pane_columns;
            let overlap_top = current_top.max(pane.pane_y);
            let overlap_bottom = current_bottom.min(pane.pane_y + pane.pane_rows);
            let vertical_overlap = overlap_bottom.saturating_sub(overlap_top);

            if vertical_overlap == 0 {
                return None;
            }

            let edge_distance = match direction {
                HorizontalDirection::Left if candidate_right <= current_left => {
                    Some(current_left - candidate_right)
                }
                HorizontalDirection::Right if candidate_left >= current_right => {
                    Some(candidate_left - current_right)
                }
                _ => None,
            }?;

            Some((index, edge_distance, vertical_overlap))
        });

    let best = match direction {
        HorizontalDirection::Left => {
            candidates.min_by_key(|(_, edge_distance, vertical_overlap)| {
                (*edge_distance, usize::MAX - *vertical_overlap)
            })
        }
        HorizontalDirection::Right => {
            candidates.min_by_key(|(_, edge_distance, vertical_overlap)| {
                (*edge_distance, usize::MAX - *vertical_overlap)
            })
        }
    };

    match (direction, best.map(|(index, _, _)| index)) {
        (_, Some(index)) => HorizontalFocusPlan::FocusPane(index),
        (HorizontalDirection::Left, None) => HorizontalFocusPlan::PreviousTab,
        (HorizontalDirection::Right, None) => HorizontalFocusPlan::NextTab,
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::{
        resolve_horizontal_focus, HorizontalDirection, HorizontalFocusPlan, HorizontalPaneSnapshot,
    };

    // Defends: leftward focus skips a closed sidebar instead of treating it as a real target.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn closed_sidebar_is_skipped_when_walking_left() {
        let panes = [
            HorizontalPaneSnapshot {
                title: "sidebar",
                is_plugin: false,
                exited: false,
                is_focused: false,
                pane_x: 0,
                pane_y: 0,
                pane_columns: 1,
                pane_rows: 40,
            },
            HorizontalPaneSnapshot {
                title: "shell",
                is_plugin: false,
                exited: false,
                is_focused: true,
                pane_x: 1,
                pane_y: 0,
                pane_columns: 80,
                pane_rows: 40,
            },
        ];

        assert_eq!(
            resolve_horizontal_focus(&panes, HorizontalDirection::Left, true),
            HorizontalFocusPlan::PreviousTab
        );
    }

    // Defends: an open sidebar remains a valid leftward focus target.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=1 total=7/10
    #[test]
    fn open_sidebar_is_still_a_valid_left_target() {
        let panes = [
            HorizontalPaneSnapshot {
                title: "sidebar",
                is_plugin: false,
                exited: false,
                is_focused: false,
                pane_x: 0,
                pane_y: 0,
                pane_columns: 24,
                pane_rows: 40,
            },
            HorizontalPaneSnapshot {
                title: "shell",
                is_plugin: false,
                exited: false,
                is_focused: true,
                pane_x: 24,
                pane_y: 0,
                pane_columns: 80,
                pane_rows: 40,
            },
        ];

        assert_eq!(
            resolve_horizontal_focus(&panes, HorizontalDirection::Left, false),
            HorizontalFocusPlan::FocusPane(0)
        );
    }

    // Defends: the nearest visible left pane wins even when a hidden sidebar exists farther left.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn nearest_visible_left_pane_wins_over_hidden_sidebar() {
        let panes = [
            HorizontalPaneSnapshot {
                title: "sidebar",
                is_plugin: false,
                exited: false,
                is_focused: false,
                pane_x: 0,
                pane_y: 0,
                pane_columns: 1,
                pane_rows: 40,
            },
            HorizontalPaneSnapshot {
                title: "stack",
                is_plugin: false,
                exited: false,
                is_focused: false,
                pane_x: 1,
                pane_y: 0,
                pane_columns: 60,
                pane_rows: 40,
            },
            HorizontalPaneSnapshot {
                title: "shell",
                is_plugin: false,
                exited: false,
                is_focused: true,
                pane_x: 61,
                pane_y: 0,
                pane_columns: 40,
                pane_rows: 40,
            },
        ];

        assert_eq!(
            resolve_horizontal_focus(&panes, HorizontalDirection::Left, true),
            HorizontalFocusPlan::FocusPane(1)
        );
    }

    // Defends: panes without horizontal overlap do not count as left or right focus targets.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn panes_without_horizontal_overlap_do_not_count_as_left_or_right_targets() {
        let panes = [
            HorizontalPaneSnapshot {
                title: "stack",
                is_plugin: false,
                exited: false,
                is_focused: true,
                pane_x: 1,
                pane_y: 0,
                pane_columns: 80,
                pane_rows: 20,
            },
            HorizontalPaneSnapshot {
                title: "terminal",
                is_plugin: false,
                exited: false,
                is_focused: false,
                pane_x: 1,
                pane_y: 20,
                pane_columns: 80,
                pane_rows: 20,
            },
        ];

        assert_eq!(
            resolve_horizontal_focus(&panes, HorizontalDirection::Right, true),
            HorizontalFocusPlan::NextTab
        );
    }
}
