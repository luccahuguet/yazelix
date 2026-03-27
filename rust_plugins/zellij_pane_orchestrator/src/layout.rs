use std::collections::BTreeMap;
use std::env;
use std::thread::sleep;
use std::time::Duration;

use serde::Serialize;
use zellij_tile::prelude::*;
use yazelix_pane_orchestrator::pane_contract::FocusContextPolicy;
use yazelix_pane_orchestrator::sidebar_contract::{
    SidebarVisibilityTogglePlan, resolve_sidebar_visibility_toggle,
};

use crate::panes::{ManagedTabPanes, SIDEBAR_TITLE, TerminalPaneLayout};
use crate::{State, RESULT_MISSING, RESULT_OK, RESULT_UNKNOWN_LAYOUT, SWAP_LAYOUT_STEP_DELAY_MS};

const SINGLE_OPEN_LAYOUT_NAME: &str = "single_open";
const SINGLE_CLOSED_LAYOUT_NAME: &str = "single_closed";
const VERTICAL_SPLIT_OPEN_LAYOUT_NAME: &str = "vertical_split_open";
const VERTICAL_SPLIT_CLOSED_LAYOUT_NAME: &str = "vertical_split_closed";
const BOTTOM_TERMINAL_OPEN_LAYOUT_NAME: &str = "bottom_terminal_open";
const BOTTOM_TERMINAL_CLOSED_LAYOUT_NAME: &str = "bottom_terminal_closed";
const SIDEBAR_COLLAPSED_WIDTH_COLUMNS: usize = 2;
const HOME_DIR_PLACEHOLDER: &str = "__YAZELIX_HOME_DIR__";
const RUNTIME_DIR_PLACEHOLDER: &str = "__YAZELIX_RUNTIME_DIR__";
const WIDGET_TRAY_PLACEHOLDER: &str = "__YAZELIX_WIDGET_TRAY__";
const CUSTOM_TEXT_SEGMENT_PLACEHOLDER: &str = "__YAZELIX_CUSTOM_TEXT_SEGMENT__";
const ZJSTATUS_TAB_TEMPLATE_PLACEHOLDER: &str = "__YAZELIX_ZJSTATUS_TAB_TEMPLATE__";
const KEYBINDS_COMMON_PLACEHOLDER: &str = "__YAZELIX_KEYBINDS_COMMON__";
const SWAP_SIDEBAR_OPEN_PLACEHOLDER: &str = "__YAZELIX_SWAP_SIDEBAR_OPEN__";
const SWAP_SIDEBAR_CLOSED_PLACEHOLDER: &str = "__YAZELIX_SWAP_SIDEBAR_CLOSED__";
const DEFAULT_WIDGET_TRAY: &str =
    "{swap_layout} #[fg=#00ff88,bold][editor: {command_editor}] #[fg=#00ff88,bold][shell: {command_shell}] #[fg=#00ff88,bold][term: {command_term}] {command_cpu} {command_ram}";
const DEFAULT_CUSTOM_TEXT_SEGMENT: &str = "";
const SIDE_LAYOUT_TEMPLATE: &str = include_str!("../../../configs/zellij/layouts/yzx_side.kdl");
const SIDE_SWAP_LAYOUT_TEMPLATE: &str = include_str!("../../../configs/zellij/layouts/yzx_side.swap.kdl");
const ZJSTATUS_TAB_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/zjstatus_tab_template.kdl");
const KEYBINDS_COMMON_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/keybinds_common.kdl");
const SWAP_SIDEBAR_OPEN_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/swap_sidebar_open.kdl");
const SWAP_SIDEBAR_CLOSED_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/swap_sidebar_closed.kdl");

#[derive(Clone, Copy, Debug)]
pub(crate) enum FamilyDirection {
    Next,
    Previous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum LayoutFamily {
    Single,
    VerticalSplit,
    BottomTerminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum SidebarState {
    Open,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LayoutVariant {
    family: LayoutFamily,
    sidebar_state: SidebarState,
}

#[derive(Serialize)]
struct DebugLayoutPaneState {
    pane_id: String,
    title: String,
    is_focused: bool,
    pane_x: usize,
    pane_y: usize,
    pane_columns: usize,
    pane_rows: usize,
}

#[derive(Serialize)]
struct DebugLayoutState {
    active_tab_position: Option<usize>,
    active_swap_layout_name: Option<String>,
    resolved_layout_variant: Option<LayoutVariant>,
    focus_context: Option<String>,
    user_pane_count: Option<usize>,
    managed_editor_pane_id: Option<String>,
    managed_sidebar_pane_id: Option<String>,
    focused_terminal_pane_id: Option<String>,
    fallback_terminal_pane_id: Option<String>,
    terminal_panes: Vec<DebugLayoutPaneState>,
}

#[derive(Serialize)]
struct DebugOverrideBuildState {
    active_tab_position: Option<usize>,
    active_swap_layout_name: Option<String>,
    resolved_layout_variant: Option<LayoutVariant>,
    total_terminal_panes: Option<usize>,
    current_home_dir: String,
    resolved_runtime_dir: String,
    embedded_layout_template_available: bool,
    embedded_swap_layout_template_available: bool,
    embedded_zjstatus_template_available: bool,
    embedded_swap_sidebar_open_available: bool,
    embedded_swap_sidebar_closed_available: bool,
    ui_tab_template_extracted: bool,
    content_layout_generated: bool,
    override_layout_generated: bool,
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
        self.log_layout_action_snapshot("switch_layout_family", active_tab_position);

        if !self.can_switch_layout_family(active_tab_position) {
            self.append_layout_debug_log("switch_layout_family short-circuited: not enough panes");
            self.respond(pipe_message, RESULT_OK);
            return;
        }

        if is_no_sidebar_mode(self.managed_panes_by_tab.get(&active_tab_position)) {
            match direction {
                FamilyDirection::Next => self.run_next_swap_layout_steps(1),
                FamilyDirection::Previous => self.run_previous_swap_layout_steps(1),
            }
            self.append_layout_debug_log("switch_layout_family used legacy no-sidebar swap-layout path");
            self.respond(pipe_message, RESULT_OK);
            return;
        }

        let Some(layout_variant) = self.get_active_layout_variant(active_tab_position) else {
            self.log_layout_action_failure(
                "switch_layout_family",
                active_tab_position,
                RESULT_UNKNOWN_LAYOUT,
            );
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        };

        if self
            .apply_override_layout_for_variant(layout_variant.shift_family(direction), active_tab_position)
            .is_none()
        {
            self.log_layout_action_failure(
                "switch_layout_family",
                active_tab_position,
                RESULT_UNKNOWN_LAYOUT,
            );
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        }

        self.log_layout_action_snapshot("switch_layout_family success", active_tab_position);
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn toggle_sidebar(&self, pipe_message: &PipeMessage) {
        let Some(active_tab_position) = self.ensure_action_ready(pipe_message) else {
            return;
        };
        self.log_layout_action_snapshot("toggle_sidebar", active_tab_position);

        if is_no_sidebar_mode(self.managed_panes_by_tab.get(&active_tab_position)) {
            self.append_layout_debug_log("toggle_sidebar returned missing in no-sidebar mode");
            self.respond(pipe_message, RESULT_MISSING);
            return;
        }

        let Some(layout_variant) = self.get_active_layout_variant(active_tab_position) else {
            self.log_layout_action_failure("toggle_sidebar", active_tab_position, RESULT_UNKNOWN_LAYOUT);
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
            SidebarVisibilityTogglePlan::OpenPreservingFocus => {
                if self
                    .apply_override_layout_for_variant(
                        layout_variant.with_sidebar_state(SidebarState::Open),
                        active_tab_position,
                    )
                    .is_none()
                {
                    self.log_layout_action_failure(
                        "toggle_sidebar",
                        active_tab_position,
                        RESULT_UNKNOWN_LAYOUT,
                    );
                    self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
                    return;
                }
            }
            SidebarVisibilityTogglePlan::ClosePreservingFocus => {
                if self
                    .apply_override_layout_for_variant(
                        layout_variant.with_sidebar_state(SidebarState::Closed),
                        active_tab_position,
                    )
                    .is_none()
                {
                    self.log_layout_action_failure(
                        "toggle_sidebar",
                        active_tab_position,
                        RESULT_UNKNOWN_LAYOUT,
                    );
                    self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
                    return;
                }
            }
            SidebarVisibilityTogglePlan::CloseAndFocusEditor => {
                if self
                    .apply_override_layout_for_variant(
                        layout_variant.with_sidebar_state(SidebarState::Closed),
                        active_tab_position,
                    )
                    .is_none()
                {
                    self.log_layout_action_failure(
                        "toggle_sidebar",
                        active_tab_position,
                        RESULT_UNKNOWN_LAYOUT,
                    );
                    self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
                    return;
                }
                self.move_focus_right_after_layout_settle();
            }
            SidebarVisibilityTogglePlan::CloseAndFocusFallback => {
                if self
                    .apply_override_layout_for_variant(
                        layout_variant.with_sidebar_state(SidebarState::Closed),
                        active_tab_position,
                    )
                    .is_none()
                {
                    self.log_layout_action_failure(
                        "toggle_sidebar",
                        active_tab_position,
                        RESULT_UNKNOWN_LAYOUT,
                    );
                    self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
                    return;
                }
                self.move_focus_right_after_layout_settle();
            }
        }

        self.log_layout_action_snapshot("toggle_sidebar success", active_tab_position);
        self.respond(pipe_message, RESULT_OK);
    }

    pub(crate) fn debug_layout_state(&self, pipe_message: &PipeMessage) {
        let state = self.build_debug_layout_state(self.active_tab_position);

        match serde_json::to_string(&state) {
            Ok(serialized_state) => self.respond(pipe_message, &serialized_state),
            Err(_) => self.respond(pipe_message, crate::RESULT_INVALID_PAYLOAD),
        }
    }

    pub(crate) fn debug_override_build_state(&self, pipe_message: &PipeMessage) {
        let state = self.build_debug_override_build_state(self.active_tab_position);

        match serde_json::to_string(&state) {
            Ok(serialized_state) => self.respond(pipe_message, &serialized_state),
            Err(_) => self.respond(pipe_message, crate::RESULT_INVALID_PAYLOAD),
        }
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
            .or_else(|| {
                self.last_known_layout_variant_by_tab
                    .borrow()
                    .get(&active_tab_position)
                    .copied()
            })
            .or_else(|| {
                self.terminal_panes_by_tab
                    .get(&active_tab_position)
                    .and_then(|terminal_panes| {
                        infer_layout_variant_from_terminal_panes(
                            terminal_panes,
                            self.managed_panes_by_tab.get(&active_tab_position),
                        )
                    })
            })
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
        if let Some(active_tab_position) = self.active_tab_position {
            if let Some(layout_variant) = self.get_active_layout_variant(active_tab_position) {
                if self
                    .apply_override_layout_for_variant(
                        layout_variant.with_sidebar_state(SidebarState::Open),
                        active_tab_position,
                    )
                    .is_none()
                {
                    self.run_previous_swap_layout_steps(1);
                }
            } else {
                self.run_previous_swap_layout_steps(1);
            }
        } else {
            self.run_previous_swap_layout_steps(1);
        }
        self.move_focus_to_sidebar_after_layout_settle();
    }

    fn build_debug_layout_state(&self, active_tab_position: Option<usize>) -> DebugLayoutState {
        let active_swap_layout_name = active_tab_position
            .and_then(|tab_position| self.active_swap_layout_name_by_tab.get(&tab_position))
            .cloned()
            .flatten();
        let resolved_layout_variant =
            active_tab_position.and_then(|tab_position| self.get_active_layout_variant(tab_position));
        let focus_context = active_tab_position
            .and_then(|tab_position| self.focus_context_by_tab.get(&tab_position))
            .map(|context| match context {
                crate::panes::FocusContext::Editor => "editor",
                crate::panes::FocusContext::Sidebar => "sidebar",
                crate::panes::FocusContext::Other => "other",
            })
            .map(str::to_string);
        let user_pane_count = active_tab_position
            .and_then(|tab_position| self.user_pane_count_by_tab.get(&tab_position))
            .copied();
        let managed_tab_panes =
            active_tab_position.and_then(|tab_position| self.managed_panes_by_tab.get(&tab_position));
        let terminal_panes = active_tab_position
            .and_then(|tab_position| self.terminal_panes_by_tab.get(&tab_position))
            .map(|panes| {
                panes
                    .iter()
                    .map(|pane| DebugLayoutPaneState {
                        pane_id: format!("{:?}", pane.pane_id),
                        title: pane.title.clone(),
                        is_focused: pane.is_focused,
                        pane_x: pane.pane_x,
                        pane_y: pane.pane_y,
                        pane_columns: pane.pane_columns,
                        pane_rows: pane.pane_rows,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        DebugLayoutState {
            active_tab_position,
            active_swap_layout_name,
            resolved_layout_variant,
            focus_context,
            user_pane_count,
            managed_editor_pane_id: managed_tab_panes
                .and_then(|panes| panes.editor)
                .map(|pane| format!("{:?}", pane.pane_id)),
            managed_sidebar_pane_id: managed_tab_panes
                .and_then(|panes| panes.sidebar)
                .map(|pane| format!("{:?}", pane.pane_id)),
            focused_terminal_pane_id: active_tab_position
                .and_then(|tab_position| self.focused_terminal_pane_by_tab.get(&tab_position))
                .map(|pane_id| format!("{:?}", pane_id)),
            fallback_terminal_pane_id: active_tab_position
                .and_then(|tab_position| self.fallback_terminal_pane_by_tab.get(&tab_position))
                .map(|pane_id| format!("{:?}", pane_id)),
            terminal_panes,
        }
    }

    fn build_debug_override_build_state(
        &self,
        active_tab_position: Option<usize>,
    ) -> DebugOverrideBuildState {
        let current_home_dir = home_dir();
        let resolved_runtime_dir = runtime_dir();
        let runtime_sidebar_launcher_file =
            runtime_script_path("launch_sidebar_yazi.nu", &resolved_runtime_dir);
        let active_swap_layout_name = active_tab_position
            .and_then(|tab_position| self.active_swap_layout_name_by_tab.get(&tab_position))
            .cloned()
            .flatten();
        let resolved_layout_variant =
            active_tab_position.and_then(|tab_position| self.get_active_layout_variant(tab_position));
        let total_terminal_panes = active_tab_position
            .and_then(|tab_position| self.terminal_panes_by_tab.get(&tab_position))
            .map(|panes| panes.len());

        let rendered_side_layout = render_embedded_side_layout(&resolved_runtime_dir);
        let rendered_swap_layouts = render_embedded_swap_layouts(&resolved_runtime_dir);
        let ui_tab_template_extracted = extract_ui_tab_template(&rendered_side_layout).is_some();
        let content_layout_generated = resolved_layout_variant
            .zip(total_terminal_panes)
            .and_then(|(layout_variant, pane_count)| {
                build_content_layout_kdl(
                    layout_variant,
                    pane_count.saturating_sub(1),
                    &runtime_sidebar_launcher_file,
                )
            })
            .is_some();
        let override_layout_generated = resolved_layout_variant
            .zip(total_terminal_panes)
            .and_then(|(layout_variant, pane_count)| {
                build_override_layout_kdl_with_inputs(
                    layout_variant,
                    pane_count,
                    Some(rendered_side_layout.as_str()),
                    Some(rendered_swap_layouts.as_str()),
                    &runtime_sidebar_launcher_file,
                )
            })
            .is_some();

        DebugOverrideBuildState {
            active_tab_position,
            active_swap_layout_name,
            resolved_layout_variant,
            total_terminal_panes,
            current_home_dir,
            resolved_runtime_dir,
            embedded_layout_template_available: !SIDE_LAYOUT_TEMPLATE.trim().is_empty(),
            embedded_swap_layout_template_available: !SIDE_SWAP_LAYOUT_TEMPLATE.trim().is_empty(),
            embedded_zjstatus_template_available: !ZJSTATUS_TAB_TEMPLATE.trim().is_empty(),
            embedded_swap_sidebar_open_available: !SWAP_SIDEBAR_OPEN_TEMPLATE.trim().is_empty(),
            embedded_swap_sidebar_closed_available: !SWAP_SIDEBAR_CLOSED_TEMPLATE.trim().is_empty(),
            ui_tab_template_extracted,
            content_layout_generated,
            override_layout_generated,
        }
    }

    fn log_layout_action_failure(
        &self,
        action_name: &str,
        active_tab_position: usize,
        result: &str,
    ) {
        let state = self.build_debug_layout_state(Some(active_tab_position));
        if let Ok(serialized_state) = serde_json::to_string(&state) {
            self.append_layout_debug_log(&format!(
                "{action_name} failed with {result}: {serialized_state}"
            ));
        } else {
            self.append_layout_debug_log(&format!(
                "{action_name} failed with {result}: <state serialization failed>"
            ));
        }
    }

    fn log_layout_action_snapshot(&self, action_name: &str, active_tab_position: usize) {
        let state = self.build_debug_layout_state(Some(active_tab_position));
        if let Ok(serialized_state) = serde_json::to_string(&state) {
            self.append_layout_debug_log(&format!("{action_name}: {serialized_state}"));
        } else {
            self.append_layout_debug_log(&format!(
                "{action_name}: <state serialization failed>"
            ));
        }
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

    fn apply_override_layout_for_variant(
        &self,
        layout_variant: LayoutVariant,
        active_tab_position: usize,
    ) -> Option<()> {
        let terminal_panes = self.terminal_panes_by_tab.get(&active_tab_position)?;
        let layout_kdl = build_override_layout_kdl(layout_variant, terminal_panes.len())?;
        let layout_info = LayoutInfo::Stringified(layout_kdl);
        override_layout(layout_info, false, false, true, BTreeMap::new());
        self.last_known_layout_variant_by_tab
            .borrow_mut()
            .insert(active_tab_position, layout_variant);
        Some(())
    }
}

impl LayoutVariant {
    pub(crate) fn is_sidebar_closed(&self) -> bool {
        self.sidebar_state == SidebarState::Closed
    }

    fn with_sidebar_state(&self, sidebar_state: SidebarState) -> Self {
        Self {
            family: self.family,
            sidebar_state,
        }
    }

    fn shift_family(&self, direction: FamilyDirection) -> Self {
        let family = match (self.family, direction) {
            (LayoutFamily::Single, FamilyDirection::Next) => LayoutFamily::VerticalSplit,
            (LayoutFamily::VerticalSplit, FamilyDirection::Next) => LayoutFamily::BottomTerminal,
            (LayoutFamily::BottomTerminal, FamilyDirection::Next) => LayoutFamily::Single,
            (LayoutFamily::Single, FamilyDirection::Previous) => LayoutFamily::BottomTerminal,
            (LayoutFamily::VerticalSplit, FamilyDirection::Previous) => LayoutFamily::Single,
            (LayoutFamily::BottomTerminal, FamilyDirection::Previous) => LayoutFamily::VerticalSplit,
        };

        Self {
            family,
            sidebar_state: self.sidebar_state,
        }
    }

    pub(crate) fn from_layout_name(layout_name: &str) -> Option<Self> {
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

fn infer_layout_variant_from_terminal_panes(
    terminal_panes: &[TerminalPaneLayout],
    managed_tab_panes: Option<&ManagedTabPanes>,
) -> Option<LayoutVariant> {
    let managed_sidebar_id = managed_tab_panes.and_then(|tab| tab.sidebar).map(|pane| pane.pane_id);
    let sidebar_pane = managed_sidebar_id
        .and_then(|pane_id| terminal_panes.iter().find(|pane| pane.pane_id == pane_id))
        .or_else(|| terminal_panes.iter().find(|pane| pane.title.trim() == SIDEBAR_TITLE))?;
    let non_sidebar_panes = terminal_panes
        .iter()
        .filter(|pane| pane.pane_id != sidebar_pane.pane_id)
        .collect::<Vec<_>>();

    if non_sidebar_panes.is_empty() {
        return None;
    }

    let sidebar_state = if sidebar_pane.pane_columns <= SIDEBAR_COLLAPSED_WIDTH_COLUMNS {
        SidebarState::Closed
    } else {
        SidebarState::Open
    };

    let min_x = non_sidebar_panes.iter().map(|pane| pane.pane_x).min()?;
    let min_y = non_sidebar_panes.iter().map(|pane| pane.pane_y).min()?;
    let max_x = non_sidebar_panes.iter().map(|pane| pane.pane_x).max()?;
    let max_y = non_sidebar_panes.iter().map(|pane| pane.pane_y).max()?;

    let family = if max_x > min_x {
        LayoutFamily::VerticalSplit
    } else if max_y > min_y {
        LayoutFamily::BottomTerminal
    } else {
        LayoutFamily::Single
    };

    Some(LayoutVariant {
        family,
        sidebar_state,
    })
}

fn build_override_layout_kdl(
    layout_variant: LayoutVariant,
    total_terminal_panes: usize,
) -> Option<String> {
    let resolved_runtime_dir = runtime_dir();
    let sidebar_launcher = runtime_script_path("launch_sidebar_yazi.nu", &resolved_runtime_dir);
    let runtime_layout = render_embedded_side_layout(&resolved_runtime_dir);
    let swap_layouts = render_embedded_swap_layouts(&resolved_runtime_dir);
    build_override_layout_kdl_with_inputs(
        layout_variant,
        total_terminal_panes,
        Some(runtime_layout.as_str()),
        Some(swap_layouts.as_str()),
        &sidebar_launcher,
    )
}

fn build_override_layout_kdl_with_inputs(
    layout_variant: LayoutVariant,
    total_terminal_panes: usize,
    runtime_layout: Option<&str>,
    swap_layouts: Option<&str>,
    sidebar_launcher: &str,
) -> Option<String> {
    if total_terminal_panes < 2 {
        return None;
    }

    let ui_tab_template = extract_ui_tab_template(runtime_layout?)?;
    let content_layout = build_content_layout_kdl(
        layout_variant,
        total_terminal_panes.saturating_sub(1),
        sidebar_launcher,
    )?;

    Some(format!(
        "layout {{\n{ui_tab_template}\n\n{}\n\nui {{\n{content_layout}\n}}\n}}\n",
        swap_layouts?
    ))
}

fn build_content_layout_kdl(
    layout_variant: LayoutVariant,
    non_sidebar_terminal_panes: usize,
    sidebar_launcher: &str,
) -> Option<String> {
    if non_sidebar_terminal_panes < 1 {
        return None;
    }

    let sidebar_pane = match layout_variant.sidebar_state {
        SidebarState::Open => format!(
            "        pane name=\"sidebar\" {{\n            command \"nu\"\n            args \"{sidebar_launcher}\"\n        }}"
        ),
        SidebarState::Closed => format!(
            "        pane name=\"sidebar\" {{\n            command \"nu\"\n            args \"{sidebar_launcher}\"\n            size \"1\"\n        }}"
        ),
    };

    let content_region = match layout_variant.family {
        LayoutFamily::Single => build_single_family_kdl(non_sidebar_terminal_panes),
        LayoutFamily::VerticalSplit => build_vertical_split_family_kdl(non_sidebar_terminal_panes)?,
        LayoutFamily::BottomTerminal => {
            build_bottom_terminal_family_kdl(non_sidebar_terminal_panes)?
        }
    };

    Some(format!(
        "    pane split_direction=\"vertical\" {{\n{sidebar_pane}\n{content_region}\n    }}"
    ))
}

fn build_single_family_kdl(non_sidebar_terminal_panes: usize) -> String {
    format!(
        "        pane stacked=true {{\n            size \"80%\"\n{}\n        }}",
        build_generic_terminal_panes(non_sidebar_terminal_panes, 3)
    )
}

fn build_vertical_split_family_kdl(non_sidebar_terminal_panes: usize) -> Option<String> {
    let stacked_panes = non_sidebar_terminal_panes.checked_sub(1)?;
    Some(format!(
        "        pane stacked=true {{\n            size \"48%\"\n{}\n        }}\n        pane {{\n            size \"32%\"\n        }}",
        build_generic_terminal_panes(stacked_panes, 3)
    ))
}

fn build_bottom_terminal_family_kdl(non_sidebar_terminal_panes: usize) -> Option<String> {
    let stacked_panes = non_sidebar_terminal_panes.checked_sub(1)?;
    Some(format!(
        "        pane split_direction=\"horizontal\" {{\n            size \"80%\"\n            pane stacked=true {{\n                size \"70%\"\n{}\n            }}\n            pane {{\n                size \"30%\"\n            }}\n        }}",
        build_generic_terminal_panes(stacked_panes, 4)
    ))
}

fn build_generic_terminal_panes(count: usize, indent_level: usize) -> String {
    let indent = "    ".repeat(indent_level);
    (0..count)
        .map(|_| format!("{indent}pane"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_embedded_side_layout(runtime_dir: &str) -> String {
    let with_fragments = apply_static_fragment(
        apply_static_fragment(
            SIDE_LAYOUT_TEMPLATE.to_string(),
            ZJSTATUS_TAB_TEMPLATE_PLACEHOLDER,
            ZJSTATUS_TAB_TEMPLATE,
        ),
        KEYBINDS_COMMON_PLACEHOLDER,
        KEYBINDS_COMMON_TEMPLATE,
    );
    replace_layout_placeholders(with_fragments, runtime_dir)
}

fn render_embedded_swap_layouts(runtime_dir: &str) -> String {
    let with_fragments = apply_static_fragment(
        apply_static_fragment(
            SIDE_SWAP_LAYOUT_TEMPLATE.to_string(),
            SWAP_SIDEBAR_OPEN_PLACEHOLDER,
            SWAP_SIDEBAR_OPEN_TEMPLATE,
        ),
        SWAP_SIDEBAR_CLOSED_PLACEHOLDER,
        SWAP_SIDEBAR_CLOSED_TEMPLATE,
    );
    replace_layout_placeholders(with_fragments, runtime_dir)
}

fn apply_static_fragment(content: String, placeholder: &str, fragment: &str) -> String {
    if !content.contains(placeholder) {
        return content;
    }

    let fragment_lines = fragment.lines().collect::<Vec<_>>();
    content
        .lines()
        .map(|line| {
            if line.contains(placeholder) {
                let indent = line
                    .chars()
                    .take_while(|character| character.is_whitespace())
                    .collect::<String>();
                fragment_lines
                    .iter()
                    .map(|fragment_line| format!("{indent}{fragment_line}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn replace_layout_placeholders(content: String, runtime_dir: &str) -> String {
    content
        .replace(HOME_DIR_PLACEHOLDER, &home_dir())
        .replace(RUNTIME_DIR_PLACEHOLDER, runtime_dir)
        .replace(WIDGET_TRAY_PLACEHOLDER, DEFAULT_WIDGET_TRAY)
        .replace(CUSTOM_TEXT_SEGMENT_PLACEHOLDER, DEFAULT_CUSTOM_TEXT_SEGMENT)
}

fn runtime_dir() -> String {
    env::var("YAZELIX_RUNTIME_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("{}/.config/yazelix", home_dir()))
}

fn runtime_script_path(file_name: &str, runtime_dir: &str) -> String {
    format!("{runtime_dir}/configs/zellij/scripts/{file_name}")
}

fn home_dir() -> String {
    env::var("HOME").unwrap_or_else(|_| String::from("/"))
}

fn extract_ui_tab_template(layout_text: &str) -> Option<String> {
    let start = layout_text.find("tab_template name=\"ui\" {")?;
    let end = layout_text.find("default_tab_template")?;
    Some(layout_text[start..end].trim_end().to_string())
}

fn is_no_sidebar_mode(managed_tab_panes: Option<&ManagedTabPanes>) -> bool {
    managed_tab_panes.and_then(|tab| tab.sidebar).is_none()
}

#[cfg(test)]
mod tests {
    use super::{
        FamilyDirection, LayoutFamily, LayoutVariant, SidebarState, build_bottom_terminal_family_kdl,
        build_content_layout_kdl, build_generic_terminal_panes, build_override_layout_kdl,
        build_single_family_kdl, build_vertical_split_family_kdl, extract_ui_tab_template,
        infer_layout_variant_from_terminal_panes,
    };
    use crate::panes::{ManagedTabPanes, ManagedTerminalPane, TerminalPaneLayout};
    use zellij_tile::prelude::PaneId;

    fn terminal_pane(
        pane_id: PaneId,
        title: &str,
        x: usize,
        y: usize,
        columns: usize,
        rows: usize,
    ) -> TerminalPaneLayout {
        TerminalPaneLayout {
            pane_id,
            title: title.to_string(),
            is_focused: false,
            pane_x: x,
            pane_y: y,
            pane_columns: columns,
            pane_rows: rows,
        }
    }

    #[test]
    fn infers_single_open_from_stacked_editor_region() {
        let panes = vec![
            terminal_pane(PaneId::Terminal(1), "sidebar", 0, 0, 16, 24),
            terminal_pane(PaneId::Terminal(2), "editor", 16, 0, 64, 24),
            terminal_pane(PaneId::Terminal(3), "shell", 16, 0, 64, 24),
        ];

        assert_eq!(
            infer_layout_variant_from_terminal_panes(&panes, None),
            Some(LayoutVariant {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            })
        );
    }

    #[test]
    fn infers_vertical_split_closed_from_multiple_x_positions() {
        let panes = vec![
            terminal_pane(PaneId::Terminal(1), "sidebar", 0, 0, 1, 24),
            terminal_pane(PaneId::Terminal(2), "editor", 1, 0, 39, 24),
            terminal_pane(PaneId::Terminal(3), "shell", 40, 0, 40, 24),
        ];

        assert_eq!(
            infer_layout_variant_from_terminal_panes(&panes, None),
            Some(LayoutVariant {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Closed,
            })
        );
    }

    #[test]
    fn infers_bottom_terminal_from_multiple_y_positions() {
        let panes = vec![
            terminal_pane(PaneId::Terminal(1), "sidebar", 0, 0, 16, 24),
            terminal_pane(PaneId::Terminal(2), "editor", 16, 0, 64, 16),
            terminal_pane(PaneId::Terminal(3), "shell", 16, 16, 64, 8),
        ];

        assert_eq!(
            infer_layout_variant_from_terminal_panes(&panes, None),
            Some(LayoutVariant {
                family: LayoutFamily::BottomTerminal,
                sidebar_state: SidebarState::Open,
            })
        );
    }

    #[test]
    fn infers_layout_from_managed_sidebar_pane_id_when_title_drifted() {
        let panes = vec![
            terminal_pane(PaneId::Terminal(41), "yazi", 0, 0, 16, 24),
            terminal_pane(PaneId::Terminal(42), "editor", 16, 0, 39, 24),
            terminal_pane(PaneId::Terminal(43), "shell", 55, 0, 25, 24),
        ];
        let managed_tab_panes = ManagedTabPanes {
            editor: Some(ManagedTerminalPane {
                pane_id: PaneId::Terminal(42),
            }),
            sidebar: Some(ManagedTerminalPane {
                pane_id: PaneId::Terminal(41),
            }),
        };

        assert_eq!(
            infer_layout_variant_from_terminal_panes(&panes, Some(&managed_tab_panes)),
            Some(LayoutVariant {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Open,
            })
        );
    }

    #[test]
    fn cycles_family_without_changing_sidebar_state() {
        let variant = LayoutVariant {
            family: LayoutFamily::Single,
            sidebar_state: SidebarState::Closed,
        };

        assert_eq!(
            variant.shift_family(FamilyDirection::Next),
            LayoutVariant {
                family: LayoutFamily::VerticalSplit,
                sidebar_state: SidebarState::Closed,
            }
        );
        assert_eq!(
            variant.shift_family(FamilyDirection::Previous),
            LayoutVariant {
                family: LayoutFamily::BottomTerminal,
                sidebar_state: SidebarState::Closed,
            }
        );
    }

    #[test]
    fn generates_explicit_terminal_slots_without_children_placeholders() {
        let single = build_single_family_kdl(3);
        let vertical = build_vertical_split_family_kdl(3).unwrap();
        let bottom = build_bottom_terminal_family_kdl(3).unwrap();

        assert!(!single.contains("children"));
        assert!(!vertical.contains("children"));
        assert!(!bottom.contains("children"));
        assert_eq!(single.matches("\n            pane").count(), 4);
        assert_eq!(vertical.matches("\n            pane").count(), 4);
        assert_eq!(bottom.matches("\n                pane").count(), 4);
    }

    #[test]
    fn builds_single_sidebar_layout_with_one_content_pane() {
        let content_layout = build_content_layout_kdl(
            LayoutVariant {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            },
            1,
            "/tmp/launch_sidebar_yazi.nu",
        )
        .unwrap();

        assert!(content_layout.contains("name=\"sidebar\""));
        assert!(content_layout.contains("pane stacked=true"));
        assert_eq!(content_layout.matches("\n            pane").count(), 1);
    }

    #[test]
    fn builds_override_layout_from_embedded_templates_without_runtime_file_reads() {
        unsafe { std::env::set_var("YAZELIX_RUNTIME_DIR", "/tmp/yazelix-runtime"); }

        let layout_kdl = build_override_layout_kdl(
            LayoutVariant {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            },
            3,
        )
        .unwrap();

        assert!(layout_kdl.contains("tab_template name=\"ui\""));
        assert!(layout_kdl.contains("swap_tiled_layout name=\"single_open\""));
        assert!(layout_kdl.contains("file:/tmp/yazelix-runtime/configs/zellij/plugins/zjstatus.wasm"));
        assert!(layout_kdl.contains("/tmp/yazelix-runtime/configs/zellij/scripts/launch_sidebar_yazi.nu"));

        unsafe { std::env::remove_var("YAZELIX_RUNTIME_DIR"); }
    }

    #[test]
    fn extracts_only_the_ui_tab_template_from_runtime_layout_text() {
        let runtime_layout = r#"
layout {
    tab_template name="ui" {
        pane
        children
        pane
    }

    default_tab_template {
        pane
    }
}
"#;

        let ui_template = extract_ui_tab_template(runtime_layout).unwrap();
        assert!(ui_template.contains("tab_template name=\"ui\""));
        assert!(!ui_template.contains("default_tab_template"));
    }

    #[test]
    fn generic_terminal_panes_respect_requested_count() {
        let panes = build_generic_terminal_panes(4, 2);
        assert_eq!(panes.matches("\n        pane").count(), 3);
        assert!(panes.starts_with("        pane"));
    }
}
