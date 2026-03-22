use std::thread::sleep;
use std::time::Duration;

use zellij_tile::prelude::*;
use yazelix_pane_orchestrator::pane_contract::FocusContextPolicy;
use yazelix_pane_orchestrator::sidebar_contract::{
    SidebarVisibilityTogglePlan, resolve_sidebar_visibility_toggle,
};

use crate::panes::ManagedTabPanes;
use crate::{State, RESULT_MISSING, RESULT_OK, RESULT_UNKNOWN_LAYOUT, SWAP_LAYOUT_STEP_DELAY_MS};

const SINGLE_OPEN_LAYOUT_NAME: &str = "single_open";
const SINGLE_CLOSED_LAYOUT_NAME: &str = "single_closed";
const VERTICAL_SPLIT_OPEN_LAYOUT_NAME: &str = "vertical_split_open";
const VERTICAL_SPLIT_CLOSED_LAYOUT_NAME: &str = "vertical_split_closed";
const BOTTOM_TERMINAL_OPEN_LAYOUT_NAME: &str = "bottom_terminal_open";
const BOTTOM_TERMINAL_CLOSED_LAYOUT_NAME: &str = "bottom_terminal_closed";

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

        let focus_context = self
            .focus_context_by_tab
            .get(&active_tab_position)
            .copied()
            .unwrap_or(crate::panes::FocusContext::Other);
        let managed_tab_panes = self.managed_panes_by_tab.get(&active_tab_position);
        let has_editor = managed_tab_panes.and_then(|tab| tab.editor).is_some();
        let has_focus_fallback = self
            .fallback_terminal_pane_by_tab
            .get(&active_tab_position)
            .is_some();

        match resolve_sidebar_visibility_toggle(
            layout_variant.is_sidebar_closed(),
            match focus_context {
                crate::panes::FocusContext::Editor => FocusContextPolicy::Editor,
                crate::panes::FocusContext::Sidebar => FocusContextPolicy::Sidebar,
                crate::panes::FocusContext::Other => FocusContextPolicy::Other,
            },
            has_editor,
            has_focus_fallback,
        ) {
            SidebarVisibilityTogglePlan::OpenPreservingFocus => self.run_previous_swap_layout_steps(1),
            SidebarVisibilityTogglePlan::ClosePreservingFocus => self.run_next_swap_layout_steps(1),
            SidebarVisibilityTogglePlan::CloseAndFocusEditor => {
                self.run_next_swap_layout_steps(1);
                self.move_focus_right_after_layout_settle();
            }
            SidebarVisibilityTogglePlan::CloseAndFocusFallback => {
                self.run_next_swap_layout_steps(1);
                self.move_focus_right_after_layout_settle();
            }
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

    pub(crate) fn run_next_swap_layout_steps(&self, steps: usize) {
        for _ in 0..steps {
            next_swap_layout();
            sleep(Duration::from_millis(SWAP_LAYOUT_STEP_DELAY_MS));
        }
    }

    pub(crate) fn run_previous_swap_layout_steps(&self, steps: usize) {
        for _ in 0..steps {
            previous_swap_layout();
            sleep(Duration::from_millis(SWAP_LAYOUT_STEP_DELAY_MS));
        }
    }

    pub(crate) fn open_sidebar_and_focus_after_layout_settle(&self) {
        self.run_previous_swap_layout_steps(1);
        self.move_focus_to_sidebar_after_layout_settle();
    }

    fn move_focus_right_after_layout_settle(&self) {
        for delay in [35, 105] {
            sleep(Duration::from_millis(delay));
            move_focus(Direction::Right);
        }
    }

    pub(crate) fn move_focus_to_sidebar_after_layout_settle(&self) {
        // Sidebar is always the leftmost managed pane, but the currently focused pane
        // may be one or two panes to the right depending on the active layout family.
        for delay in [35, 70, 105] {
            sleep(Duration::from_millis(delay));
            move_focus(Direction::Left);
        }
    }
}

impl LayoutVariant {
    pub(crate) fn is_sidebar_closed(&self) -> bool {
        self.sidebar_state == SidebarState::Closed
    }

    fn from_layout_name(layout_name: &str) -> Option<Self> {
        match layout_name {
            SINGLE_OPEN_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            }),
            SINGLE_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Closed,
            }),
            VERTICAL_SPLIT_OPEN_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Open,
            }),
            VERTICAL_SPLIT_CLOSED_LAYOUT_NAME => Some(Self {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Closed,
            }),
            BOTTOM_TERMINAL_OPEN_LAYOUT_NAME => Some(Self {
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
