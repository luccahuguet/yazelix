use std::thread::sleep;
use std::time::Duration;

use zellij_tile::prelude::*;

use crate::panes::ManagedTabPanes;
use crate::{State, RESULT_MISSING, RESULT_OK, RESULT_UNKNOWN_LAYOUT, SWAP_LAYOUT_STEP_DELAY_MS};

const SINGLE_OPEN_LAYOUT_NAME: &str = "single_open";
const SINGLE_CLOSED_LAYOUT_NAME: &str = "single_closed";
const VERTICAL_SPLIT_OPEN_LAYOUT_NAME: &str = "vertical_split_open";
const VERTICAL_SPLIT_CLOSED_LAYOUT_NAME: &str = "vertical_split_closed";
const BOTTOM_TERMINAL_OPEN_LAYOUT_NAME: &str = "bottom_terminal_open";
const BOTTOM_TERMINAL_CLOSED_LAYOUT_NAME: &str = "bottom_terminal_closed";
const LEGACY_BASIC_LAYOUT_NAME: &str = "basic";
const LEGACY_STACKED_LAYOUT_NAME: &str = "stacked";
const LEGACY_THREE_COLUMN_LAYOUT_NAME: &str = "three_column";
const LEGACY_SIDEBAR_CLOSED_LAYOUT_NAME: &str = "sidebar_closed";
const LEGACY_SINGLE_LAYOUT_NAME: &str = "single";
const LEGACY_VERTICAL_SPLIT_LAYOUT_NAME: &str = "vertical_split";
const LEGACY_BOTTOM_TERMINAL_LAYOUT_NAME: &str = "bottom_terminal";

#[derive(Clone, Copy, Debug)]
pub(crate) enum FamilyDirection {
    Next,
    Previous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LayoutFamily {
    Single,
    VerticalSplit,
    BottomTerminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SidebarState {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LayoutVariant {
    family: LayoutFamily,
    sidebar_state: SidebarState,
}

impl State {
    pub(crate) fn switch_layout_family(
        &self,
        pipe_message: &PipeMessage,
        direction: FamilyDirection,
    ) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        if !self.can_switch_layout_family(active_tab_position) {
            self.respond(pipe_message, RESULT_OK);
            return;
        }

        if is_no_sidebar_mode(self.managed_panes_by_tab.get(&active_tab_position)) {
            match direction {
                FamilyDirection::Next => self.run_next_swap_layout_steps(1),
                FamilyDirection::Previous => self.run_previous_swap_layout_steps(1),
            }
            self.respond(pipe_message, RESULT_OK);
            return;
        }

        let Some(_layout_variant) = self.get_active_layout_variant(active_tab_position) else {
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        };

        match direction {
            FamilyDirection::Next => self.run_next_swap_layout_steps(2),
            FamilyDirection::Previous => self.run_previous_swap_layout_steps(2),
        }

        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn toggle_sidebar(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };

        if is_no_sidebar_mode(self.managed_panes_by_tab.get(&active_tab_position)) {
            self.respond(pipe_message, RESULT_MISSING);
            return;
        }

        let Some(layout_variant) = self.get_active_layout_variant(active_tab_position) else {
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        };

        match layout_variant.sidebar_state {
            SidebarState::Open => self.run_next_swap_layout_steps(1),
            SidebarState::Closed => self.run_previous_swap_layout_steps(1),
        }

        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn get_active_layout_variant(
        &self,
        active_tab_position: usize,
    ) -> Option<LayoutVariant> {
        let active_swap_layout_name = self
            .active_swap_layout_name_by_tab
            .get(&active_tab_position)
            .cloned()
            .flatten();

        active_swap_layout_name
            .as_deref()
            .and_then(LayoutVariant::from_layout_name)
    }

    fn can_switch_layout_family(&self, active_tab_position: usize) -> bool {
        let user_pane_count = self
            .user_pane_count_by_tab
            .get(&active_tab_position)
            .copied()
            .unwrap_or(0);

        if is_no_sidebar_mode(self.managed_panes_by_tab.get(&active_tab_position)) {
            user_pane_count >= 2
        } else {
            user_pane_count >= 3
        }
    }

    fn run_next_swap_layout_steps(&self, steps: usize) {
        for _ in 0..steps {
            next_swap_layout();
            sleep(Duration::from_millis(SWAP_LAYOUT_STEP_DELAY_MS));
        }
    }

    fn run_previous_swap_layout_steps(&self, steps: usize) {
        for _ in 0..steps {
            previous_swap_layout();
            sleep(Duration::from_millis(SWAP_LAYOUT_STEP_DELAY_MS));
        }
    }
}

impl LayoutVariant {
    pub(crate) fn is_sidebar_closed(&self) -> bool {
        self.sidebar_state == SidebarState::Closed
    }

    fn from_layout_name(layout_name: &str) -> Option<Self> {
        match layout_name {
            SINGLE_OPEN_LAYOUT_NAME
            | LEGACY_BASIC_LAYOUT_NAME
            | LEGACY_STACKED_LAYOUT_NAME
            | LEGACY_SINGLE_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            }),
            SINGLE_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Closed,
            }),
            VERTICAL_SPLIT_OPEN_LAYOUT_NAME
            | LEGACY_THREE_COLUMN_LAYOUT_NAME
            | LEGACY_VERTICAL_SPLIT_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Open,
            }),
            VERTICAL_SPLIT_CLOSED_LAYOUT_NAME | LEGACY_SIDEBAR_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Closed,
            }),
            BOTTOM_TERMINAL_OPEN_LAYOUT_NAME | LEGACY_BOTTOM_TERMINAL_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::BottomTerminal,
                sidebar_state: SidebarState::Open,
            }),
            BOTTOM_TERMINAL_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::BottomTerminal,
                sidebar_state: SidebarState::Closed,
            }),
            _ => None,
        }
    }
}

fn is_no_sidebar_mode(managed_tab_panes: Option<&ManagedTabPanes>) -> bool {
    managed_tab_panes.and_then(|tab| tab.sidebar).is_none()
}
