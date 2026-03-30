use std::collections::BTreeMap;
use std::env;
use std::thread::sleep;
use std::time::Duration;

use shlex::split as shell_split;
use yazelix_pane_orchestrator::pane_contract::FocusContextPolicy;
use yazelix_pane_orchestrator::sidebar_contract::{
    resolve_sidebar_visibility_toggle, SidebarVisibilityTogglePlan,
};
use zellij_tile::prelude::*;

use crate::panes::{ManagedTabPanes, TerminalPaneLayout, SIDEBAR_TITLE};
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
const SIDEBAR_WIDTH_PERCENT_PLACEHOLDER: &str = "__YAZELIX_SIDEBAR_WIDTH_PERCENT__";
const OPEN_CONTENT_WIDTH_PERCENT_PLACEHOLDER: &str = "__YAZELIX_OPEN_CONTENT_WIDTH_PERCENT__";
const OPEN_PRIMARY_WIDTH_PERCENT_PLACEHOLDER: &str = "__YAZELIX_OPEN_PRIMARY_WIDTH_PERCENT__";
const OPEN_SECONDARY_WIDTH_PERCENT_PLACEHOLDER: &str = "__YAZELIX_OPEN_SECONDARY_WIDTH_PERCENT__";
const CLOSED_CONTENT_WIDTH_PERCENT_PLACEHOLDER: &str = "__YAZELIX_CLOSED_CONTENT_WIDTH_PERCENT__";
const CLOSED_PRIMARY_WIDTH_PERCENT_PLACEHOLDER: &str = "__YAZELIX_CLOSED_PRIMARY_WIDTH_PERCENT__";
const CLOSED_SECONDARY_WIDTH_PERCENT_PLACEHOLDER: &str =
    "__YAZELIX_CLOSED_SECONDARY_WIDTH_PERCENT__";
const SIDE_LAYOUT_TEMPLATE: &str = include_str!("../../../configs/zellij/layouts/yzx_side.kdl");
const SIDE_SWAP_LAYOUT_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/yzx_side.swap.kdl");
const ZJSTATUS_TAB_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/zjstatus_tab_template.kdl");
const KEYBINDS_COMMON_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/keybinds_common.kdl");
const SWAP_SIDEBAR_OPEN_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/swap_sidebar_open.kdl");
const SWAP_SIDEBAR_CLOSED_TEMPLATE: &str =
    include_str!("../../../configs/zellij/layouts/fragments/swap_sidebar_closed.kdl");
pub(crate) const DEFAULT_SIDEBAR_WIDTH_PERCENT: usize = 20;
const MIN_SIDEBAR_WIDTH_PERCENT: usize = 10;
const MAX_SIDEBAR_WIDTH_PERCENT: usize = 40;

#[derive(Clone, Copy, Debug)]
pub(crate) enum FamilyDirection {
    Next,
    Previous,
}

#[derive(Default)]
pub(crate) struct ZjstatusSegments {
    pub(crate) widget_tray: String,
    pub(crate) custom_text: String,
}

pub(crate) struct OverrideLayoutConfig {
    pub(crate) zjstatus_segments: ZjstatusSegments,
    pub(crate) sidebar_width_percent: usize,
}

struct OverrideLayoutPlan {
    layout_kdl: String,
    retain_existing_terminal_panes: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PreservedTerminalPaneSpec {
    title: String,
    command: String,
    args: Vec<String>,
    cwd: Option<String>,
    is_focused: bool,
}

impl Default for OverrideLayoutConfig {
    fn default() -> Self {
        Self {
            zjstatus_segments: ZjstatusSegments::default(),
            sidebar_width_percent: DEFAULT_SIDEBAR_WIDTH_PERCENT,
        }
    }
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

        let Some(layout_variant) = self.get_active_layout_variant(active_tab_position) else {
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        };

        if self
            .apply_override_layout_for_variant(
                layout_variant.shift_family(direction),
                active_tab_position,
            )
            .is_none()
        {
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
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

        let (target_variant, focus_right_after) = match resolve_sidebar_visibility_toggle(
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
                (layout_variant.with_sidebar_state(SidebarState::Open), false)
            }
            SidebarVisibilityTogglePlan::ClosePreservingFocus => (
                layout_variant.with_sidebar_state(SidebarState::Closed),
                false,
            ),
            SidebarVisibilityTogglePlan::CloseAndFocusEditor
            | SidebarVisibilityTogglePlan::CloseAndFocusFallback => (
                layout_variant.with_sidebar_state(SidebarState::Closed),
                true,
            ),
        };

        if self
            .apply_override_layout_for_variant(target_variant, active_tab_position)
            .is_none()
        {
            self.respond(pipe_message, RESULT_UNKNOWN_LAYOUT);
            return;
        }

        if focus_right_after {
            self.move_focus_right_after_layout_settle();
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
        let opened_with_override = self
            .active_tab_position
            .and_then(|active_tab_position| {
                self.get_active_layout_variant(active_tab_position)
                    .and_then(|layout_variant| {
                        self.apply_override_layout_for_variant(
                            layout_variant.with_sidebar_state(SidebarState::Open),
                            active_tab_position,
                        )
                    })
            })
            .is_some();

        if !opened_with_override {
            self.run_previous_swap_layout_steps(1);
        }
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

    fn apply_override_layout_for_variant(
        &self,
        layout_variant: LayoutVariant,
        active_tab_position: usize,
    ) -> Option<()> {
        let terminal_panes = self.terminal_panes_by_tab.get(&active_tab_position)?;
        let override_layout_plan = build_override_layout_kdl(
            layout_variant,
            terminal_panes,
            self.managed_panes_by_tab.get(&active_tab_position),
            &self.override_layout_config,
        )?;
        let layout_info = LayoutInfo::Stringified(override_layout_plan.layout_kdl);
        override_layout(
            layout_info,
            override_layout_plan.retain_existing_terminal_panes,
            false,
            true,
            BTreeMap::new(),
        );
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
            (LayoutFamily::BottomTerminal, FamilyDirection::Previous) => {
                LayoutFamily::VerticalSplit
            }
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
    let managed_sidebar_id = managed_tab_panes
        .and_then(|tab| tab.sidebar)
        .map(|pane| pane.pane_id);
    let sidebar_pane = managed_sidebar_id
        .and_then(|pane_id| terminal_panes.iter().find(|pane| pane.pane_id == pane_id))
        .or_else(|| {
            terminal_panes
                .iter()
                .find(|pane| pane.title.trim() == SIDEBAR_TITLE)
        })?;
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
    terminal_panes: &[TerminalPaneLayout],
    managed_tab_panes: Option<&ManagedTabPanes>,
    override_layout_config: &OverrideLayoutConfig,
) -> Option<OverrideLayoutPlan> {
    if let Some(layout_kdl) =
        build_preserving_override_layout_kdl(layout_variant, terminal_panes, managed_tab_panes, override_layout_config)
    {
        return Some(OverrideLayoutPlan {
            layout_kdl,
            retain_existing_terminal_panes: true,
        });
    }

    let layout_kdl = build_generic_override_layout_kdl(
        layout_variant,
        terminal_panes.len(),
        override_layout_config,
    )?;
    Some(OverrideLayoutPlan {
        layout_kdl,
        retain_existing_terminal_panes: false,
    })
}

fn build_preserving_override_layout_kdl(
    layout_variant: LayoutVariant,
    terminal_panes: &[TerminalPaneLayout],
    managed_tab_panes: Option<&ManagedTabPanes>,
    override_layout_config: &OverrideLayoutConfig,
) -> Option<String> {
    let sidebar_pane = select_sidebar_pane(terminal_panes, managed_tab_panes)?;
    let sidebar_spec = build_preserved_terminal_pane_spec(sidebar_pane)?;
    let content_pane_specs = ordered_content_pane_specs(terminal_panes, managed_tab_panes)?;
    build_override_layout_with_content_specs(
        layout_variant,
        &sidebar_spec,
        &content_pane_specs,
        override_layout_config,
    )
}

fn build_generic_override_layout_kdl(
    layout_variant: LayoutVariant,
    total_terminal_panes: usize,
    override_layout_config: &OverrideLayoutConfig,
) -> Option<String> {
    if total_terminal_panes < 2 {
        return None;
    }

    let resolved_runtime_dir = runtime_dir();
    let sidebar_launcher = runtime_script_path("launch_sidebar_yazi.nu", &resolved_runtime_dir);
    let runtime_layout = render_embedded_side_layout(&resolved_runtime_dir, override_layout_config);
    let swap_layouts = render_embedded_swap_layouts(&resolved_runtime_dir, override_layout_config);
    let ui_tab_template = extract_ui_tab_template(&runtime_layout)?;
    let content_layout = build_generic_content_layout_kdl(
        layout_variant,
        total_terminal_panes.saturating_sub(1),
        &sidebar_launcher,
        override_layout_config.sidebar_width_percent,
    )?;

    Some(format!(
        "layout {{\n{ui_tab_template}\n\n{}\n\nui {{\n{content_layout}\n}}\n}}\n",
        swap_layouts
    ))
}

fn build_override_layout_with_content_specs(
    layout_variant: LayoutVariant,
    sidebar_spec: &PreservedTerminalPaneSpec,
    content_pane_specs: &[PreservedTerminalPaneSpec],
    override_layout_config: &OverrideLayoutConfig,
) -> Option<String> {
    if content_pane_specs.is_empty() {
        return None;
    }

    let resolved_runtime_dir = runtime_dir();
    let runtime_layout = render_embedded_side_layout(&resolved_runtime_dir, override_layout_config);
    let swap_layouts = render_embedded_swap_layouts(&resolved_runtime_dir, override_layout_config);
    let ui_tab_template = extract_ui_tab_template(&runtime_layout)?;
    let content_layout = build_content_layout_kdl(
        layout_variant,
        sidebar_spec,
        content_pane_specs,
        override_layout_config.sidebar_width_percent,
    )?;

    Some(format!(
        "layout {{\n{ui_tab_template}\n\n{}\n\nui {{\n{content_layout}\n}}\n}}\n",
        swap_layouts
    ))
}

fn build_content_layout_kdl(
    layout_variant: LayoutVariant,
    sidebar_spec: &PreservedTerminalPaneSpec,
    content_pane_specs: &[PreservedTerminalPaneSpec],
    sidebar_width_percent: usize,
) -> Option<String> {
    if content_pane_specs.is_empty() {
        return None;
    }

    let layout_widths = LayoutWidths::new(sidebar_width_percent, layout_variant.sidebar_state);

    let sidebar_pane = match layout_variant.sidebar_state {
        SidebarState::Open => build_terminal_pane_node(
            sidebar_spec,
            2,
            Some(format!("{}%", layout_widths.sidebar_width_percent)),
        ),
        SidebarState::Closed => build_terminal_pane_node(sidebar_spec, 2, Some(String::from("1"))),
    };

    let content_region = match layout_variant.family {
        LayoutFamily::Single => build_single_family_kdl(content_pane_specs, &layout_widths),
        LayoutFamily::VerticalSplit => {
            build_vertical_split_family_kdl(content_pane_specs, &layout_widths)?
        }
        LayoutFamily::BottomTerminal => {
            build_bottom_terminal_family_kdl(content_pane_specs, &layout_widths)?
        }
    };

    Some(format!(
        "    pane split_direction=\"vertical\" {{\n{sidebar_pane}\n{content_region}\n    }}"
    ))
}

fn build_generic_content_layout_kdl(
    layout_variant: LayoutVariant,
    non_sidebar_terminal_panes: usize,
    sidebar_launcher: &str,
    sidebar_width_percent: usize,
) -> Option<String> {
    if non_sidebar_terminal_panes < 1 {
        return None;
    }

    let layout_widths = LayoutWidths::new(sidebar_width_percent, layout_variant.sidebar_state);

    let sidebar_pane = match layout_variant.sidebar_state {
        SidebarState::Open => format!(
            "        pane name=\"sidebar\" {{\n            command \"nu\"\n            args \"{sidebar_launcher}\"\n            size \"{}%\"\n        }}",
            layout_widths.sidebar_width_percent
        ),
        SidebarState::Closed => format!(
            "        pane name=\"sidebar\" {{\n            command \"nu\"\n            args \"{sidebar_launcher}\"\n            size \"1\"\n        }}"
        ),
    };

    let content_region = match layout_variant.family {
        LayoutFamily::Single => build_generic_single_family_kdl(non_sidebar_terminal_panes, &layout_widths),
        LayoutFamily::VerticalSplit => {
            build_generic_vertical_split_family_kdl(non_sidebar_terminal_panes, &layout_widths)?
        }
        LayoutFamily::BottomTerminal => {
            build_generic_bottom_terminal_family_kdl(non_sidebar_terminal_panes, &layout_widths)?
        }
    };

    Some(format!(
        "    pane split_direction=\"vertical\" {{\n{sidebar_pane}\n{content_region}\n    }}"
    ))
}

fn build_single_family_kdl(content_pane_specs: &[PreservedTerminalPaneSpec], layout_widths: &LayoutWidths) -> String {
    format!(
        "        pane stacked=true {{\n            size \"{}%\"\n{}\n        }}",
        layout_widths.content_width_percent,
        build_explicit_terminal_panes(content_pane_specs, 3)
    )
}

fn build_vertical_split_family_kdl(
    content_pane_specs: &[PreservedTerminalPaneSpec],
    layout_widths: &LayoutWidths,
) -> Option<String> {
    if content_pane_specs.len() < 2 {
        return Some(build_single_family_kdl(content_pane_specs, layout_widths));
    }
    let stacked_panes = content_pane_specs.len().checked_sub(1)?;
    let primary_panes = &content_pane_specs[..stacked_panes];
    let secondary_pane = content_pane_specs.last()?;
    Some(format!(
        "        pane stacked=true {{\n            size \"{}%\"\n{}\n        }}\n{}\n",
        layout_widths.primary_width_percent,
        build_explicit_terminal_panes(primary_panes, 3),
        build_terminal_pane_node(
            secondary_pane,
            2,
            Some(format!("{}%", layout_widths.secondary_width_percent)),
        ),
    ))
}

fn build_bottom_terminal_family_kdl(
    content_pane_specs: &[PreservedTerminalPaneSpec],
    layout_widths: &LayoutWidths,
) -> Option<String> {
    if content_pane_specs.len() < 2 {
        return Some(build_single_family_kdl(content_pane_specs, layout_widths));
    }
    let stacked_panes = content_pane_specs.len().checked_sub(1)?;
    let top_panes = &content_pane_specs[..stacked_panes];
    let bottom_pane = content_pane_specs.last()?;
    Some(format!(
        "        pane split_direction=\"horizontal\" {{\n            size \"{}%\"\n            pane stacked=true {{\n                size \"70%\"\n{}\n            }}\n{}\n        }}",
        layout_widths.content_width_percent,
        build_explicit_terminal_panes(top_panes, 4),
        build_terminal_pane_node(bottom_pane, 3, Some(String::from("30%"))),
    ))
}

fn build_generic_single_family_kdl(
    non_sidebar_terminal_panes: usize,
    layout_widths: &LayoutWidths,
) -> String {
    format!(
        "        pane stacked=true {{\n            size \"{}%\"\n{}\n        }}",
        layout_widths.content_width_percent,
        build_generic_terminal_panes(non_sidebar_terminal_panes, 3)
    )
}

fn build_generic_vertical_split_family_kdl(
    non_sidebar_terminal_panes: usize,
    layout_widths: &LayoutWidths,
) -> Option<String> {
    let stacked_panes = non_sidebar_terminal_panes.checked_sub(1)?;
    Some(format!(
        "        pane stacked=true {{\n            size \"{}%\"\n{}\n        }}\n        pane {{\n            size \"{}%\"\n        }}",
        layout_widths.primary_width_percent,
        build_generic_terminal_panes(stacked_panes, 3),
        layout_widths.secondary_width_percent
    ))
}

fn build_generic_bottom_terminal_family_kdl(
    non_sidebar_terminal_panes: usize,
    layout_widths: &LayoutWidths,
) -> Option<String> {
    let stacked_panes = non_sidebar_terminal_panes.checked_sub(1)?;
    Some(format!(
        "        pane split_direction=\"horizontal\" {{\n            size \"{}%\"\n            pane stacked=true {{\n                size \"70%\"\n{}\n            }}\n            pane {{\n                size \"30%\"\n            }}\n        }}",
        layout_widths.content_width_percent,
        build_generic_terminal_panes(stacked_panes, 4)
    ))
}

struct LayoutWidths {
    sidebar_width_percent: usize,
    content_width_percent: usize,
    primary_width_percent: usize,
    secondary_width_percent: usize,
}

impl LayoutWidths {
    fn new(sidebar_width_percent: usize, sidebar_state: SidebarState) -> Self {
        let normalized_sidebar_width_percent =
            sidebar_width_percent.clamp(MIN_SIDEBAR_WIDTH_PERCENT, MAX_SIDEBAR_WIDTH_PERCENT);
        let content_width_percent = match sidebar_state {
            SidebarState::Open => 100usize.saturating_sub(normalized_sidebar_width_percent),
            SidebarState::Closed => 99,
        };
        let primary_width_percent = (content_width_percent * 3) / 5;
        let secondary_width_percent = content_width_percent - primary_width_percent;

        Self {
            sidebar_width_percent: normalized_sidebar_width_percent,
            content_width_percent,
            primary_width_percent,
            secondary_width_percent,
        }
    }
}

fn build_generic_terminal_panes(count: usize, indent_level: usize) -> String {
    let indent = "    ".repeat(indent_level);
    (0..count)
        .map(|_| format!("{indent}pane"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_explicit_terminal_panes(
    pane_specs: &[PreservedTerminalPaneSpec],
    indent_level: usize,
) -> String {
    pane_specs
        .iter()
        .map(|pane_spec| build_terminal_pane_node(pane_spec, indent_level, None))
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_terminal_pane_node(
    pane_spec: &PreservedTerminalPaneSpec,
    indent_level: usize,
    size_override: Option<String>,
) -> String {
    let indent = "    ".repeat(indent_level);
    let body_indent = "    ".repeat(indent_level + 1);
    let mut header_parts = vec![String::from("pane")];

    if !pane_spec.title.trim().is_empty() {
        header_parts.push(format!("name=\"{}\"", escape_kdl_string(&pane_spec.title)));
    }
    if pane_spec.is_focused {
        header_parts.push(String::from("focus=true"));
    }

    let mut lines = vec![format!("{indent}{} {{", header_parts.join(" "))];
    lines.push(format!(
        "{body_indent}command \"{}\"",
        escape_kdl_string(&pane_spec.command)
    ));
    if !pane_spec.args.is_empty() {
        let rendered_args = pane_spec
            .args
            .iter()
            .map(|arg| format!("\"{}\"", escape_kdl_string(arg)))
            .collect::<Vec<_>>()
            .join(" ");
        lines.push(format!("{body_indent}args {rendered_args}"));
    }
    if let Some(cwd) = &pane_spec.cwd {
        lines.push(format!(
            "{body_indent}cwd \"{}\"",
            escape_kdl_string(cwd)
        ));
    }
    if let Some(size) = size_override {
        lines.push(format!("{body_indent}size \"{size}\""));
    }
    lines.push(format!("{indent}}}"));
    lines.join("\n")
}

fn ordered_content_pane_specs(
    terminal_panes: &[TerminalPaneLayout],
    managed_tab_panes: Option<&ManagedTabPanes>,
) -> Option<Vec<PreservedTerminalPaneSpec>> {
    let sidebar_pane_id = select_sidebar_pane(terminal_panes, managed_tab_panes)?.pane_id;
    let editor_pane_id = managed_tab_panes.and_then(|tab| tab.editor).map(|pane| pane.pane_id);

    let mut content_panes = terminal_panes
        .iter()
        .filter(|pane| pane.pane_id != sidebar_pane_id)
        .collect::<Vec<_>>();
    content_panes.sort_by_key(|pane| {
        let role_rank = if Some(pane.pane_id) == editor_pane_id {
            0usize
        } else {
            1usize
        };
        (role_rank, pane.pane_y, pane.pane_x, pane.title.clone())
    });

    let content_pane_specs = content_panes
        .into_iter()
        .map(build_preserved_terminal_pane_spec)
        .collect::<Option<Vec<_>>>()?;

    if content_pane_specs.is_empty() {
        return None;
    }
    Some(content_pane_specs)
}

fn select_sidebar_pane<'a>(
    terminal_panes: &'a [TerminalPaneLayout],
    managed_tab_panes: Option<&ManagedTabPanes>,
) -> Option<&'a TerminalPaneLayout> {
    let managed_sidebar_id = managed_tab_panes
        .and_then(|tab| tab.sidebar)
        .map(|pane| pane.pane_id);
    terminal_panes
        .iter()
        .find(|pane| Some(pane.pane_id) == managed_sidebar_id)
        .or_else(|| {
            terminal_panes
                .iter()
                .find(|pane| pane.title.trim() == SIDEBAR_TITLE)
        })
}

fn build_preserved_terminal_pane_spec(
    pane: &TerminalPaneLayout,
) -> Option<PreservedTerminalPaneSpec> {
    let terminal_command = pane.terminal_command.as_ref()?;
    let command_parts = shell_split(terminal_command)?;
    let (command, args) = command_parts.split_first()?;

    Some(PreservedTerminalPaneSpec {
        title: pane.title.clone(),
        command: command.clone(),
        args: args.to_vec(),
        cwd: resolve_preserved_pane_cwd(pane),
        is_focused: pane.is_focused,
    })
}

#[cfg(test)]
fn resolve_preserved_pane_cwd(_pane: &TerminalPaneLayout) -> Option<String> {
    None
}

#[cfg(not(test))]
fn resolve_preserved_pane_cwd(pane: &TerminalPaneLayout) -> Option<String> {
    if pane.title.trim() != crate::panes::EDITOR_TITLE && pane.title.trim() != SIDEBAR_TITLE {
        return None;
    }

    get_pane_cwd(pane.pane_id)
        .ok()
        .map(|cwd| cwd.display().to_string())
}

fn escape_kdl_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn render_embedded_side_layout(
    runtime_dir: &str,
    override_layout_config: &OverrideLayoutConfig,
) -> String {
    render_embedded_layout(
        SIDE_LAYOUT_TEMPLATE,
        runtime_dir,
        override_layout_config,
        &[
            (ZJSTATUS_TAB_TEMPLATE_PLACEHOLDER, ZJSTATUS_TAB_TEMPLATE),
            (KEYBINDS_COMMON_PLACEHOLDER, KEYBINDS_COMMON_TEMPLATE),
        ],
    )
}

fn render_embedded_swap_layouts(
    runtime_dir: &str,
    override_layout_config: &OverrideLayoutConfig,
) -> String {
    render_embedded_layout(
        SIDE_SWAP_LAYOUT_TEMPLATE,
        runtime_dir,
        override_layout_config,
        &[
            (SWAP_SIDEBAR_OPEN_PLACEHOLDER, SWAP_SIDEBAR_OPEN_TEMPLATE),
            (
                SWAP_SIDEBAR_CLOSED_PLACEHOLDER,
                SWAP_SIDEBAR_CLOSED_TEMPLATE,
            ),
        ],
    )
}

fn render_embedded_layout(
    template: &str,
    runtime_dir: &str,
    override_layout_config: &OverrideLayoutConfig,
    static_fragments: &[(&str, &str)],
) -> String {
    let with_fragments = static_fragments
        .iter()
        .fold(template.to_string(), |content, (placeholder, fragment)| {
            apply_static_fragment(content, placeholder, fragment)
        });
    replace_layout_placeholders(with_fragments, runtime_dir, override_layout_config)
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

fn replace_layout_placeholders(
    content: String,
    runtime_dir: &str,
    override_layout_config: &OverrideLayoutConfig,
) -> String {
    let open_layout_widths = LayoutWidths::new(
        override_layout_config.sidebar_width_percent,
        SidebarState::Open,
    );
    let closed_layout_widths = LayoutWidths::new(
        override_layout_config.sidebar_width_percent,
        SidebarState::Closed,
    );
    content
        .replace(HOME_DIR_PLACEHOLDER, &home_dir())
        .replace(RUNTIME_DIR_PLACEHOLDER, runtime_dir)
        .replace(
            WIDGET_TRAY_PLACEHOLDER,
            &override_layout_config.zjstatus_segments.widget_tray,
        )
        .replace(
            CUSTOM_TEXT_SEGMENT_PLACEHOLDER,
            &override_layout_config.zjstatus_segments.custom_text,
        )
        .replace(
            SIDEBAR_WIDTH_PERCENT_PLACEHOLDER,
            &format!("{}%", open_layout_widths.sidebar_width_percent),
        )
        .replace(
            OPEN_CONTENT_WIDTH_PERCENT_PLACEHOLDER,
            &format!("{}%", open_layout_widths.content_width_percent),
        )
        .replace(
            OPEN_PRIMARY_WIDTH_PERCENT_PLACEHOLDER,
            &format!("{}%", open_layout_widths.primary_width_percent),
        )
        .replace(
            OPEN_SECONDARY_WIDTH_PERCENT_PLACEHOLDER,
            &format!("{}%", open_layout_widths.secondary_width_percent),
        )
        .replace(
            CLOSED_CONTENT_WIDTH_PERCENT_PLACEHOLDER,
            &format!("{}%", closed_layout_widths.content_width_percent),
        )
        .replace(
            CLOSED_PRIMARY_WIDTH_PERCENT_PLACEHOLDER,
            &format!("{}%", closed_layout_widths.primary_width_percent),
        )
        .replace(
            CLOSED_SECONDARY_WIDTH_PERCENT_PLACEHOLDER,
            &format!("{}%", closed_layout_widths.secondary_width_percent),
        )
}

fn runtime_dir() -> String {
    env::var("YAZELIX_RUNTIME_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            env::var("YAZELIX_DIR")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| String::from("/"))
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
        build_bottom_terminal_family_kdl, build_content_layout_kdl, build_generic_terminal_panes,
        build_override_layout_kdl, build_preserved_terminal_pane_spec, build_single_family_kdl,
        build_terminal_pane_node, build_vertical_split_family_kdl, extract_ui_tab_template,
        infer_layout_variant_from_terminal_panes, FamilyDirection, LayoutFamily, LayoutVariant,
        OverrideLayoutConfig, SidebarState, ZjstatusSegments,
    };
    use crate::panes::{ManagedTabPanes, ManagedTerminalPane, TerminalPaneLayout};
    use zellij_tile::prelude::PaneId;

    fn terminal_pane(
        pane_id: PaneId,
        title: &str,
        terminal_command: Option<&str>,
        x: usize,
        y: usize,
        columns: usize,
        rows: usize,
    ) -> TerminalPaneLayout {
        TerminalPaneLayout {
            pane_id,
            title: title.to_string(),
            terminal_command: terminal_command.map(|value| value.to_string()),
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
            terminal_pane(PaneId::Terminal(1), "sidebar", Some("nu sidebar.nu"), 0, 0, 16, 24),
            terminal_pane(PaneId::Terminal(2), "editor", Some("hx"), 16, 0, 64, 24),
            terminal_pane(PaneId::Terminal(3), "shell", Some("bash"), 16, 0, 64, 24),
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
            terminal_pane(PaneId::Terminal(1), "sidebar", Some("nu sidebar.nu"), 0, 0, 1, 24),
            terminal_pane(PaneId::Terminal(2), "editor", Some("hx"), 1, 0, 39, 24),
            terminal_pane(PaneId::Terminal(3), "shell", Some("bash"), 40, 0, 40, 24),
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
            terminal_pane(PaneId::Terminal(1), "sidebar", Some("nu sidebar.nu"), 0, 0, 16, 24),
            terminal_pane(PaneId::Terminal(2), "editor", Some("hx"), 16, 0, 64, 16),
            terminal_pane(PaneId::Terminal(3), "shell", Some("bash"), 16, 16, 64, 8),
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
            terminal_pane(PaneId::Terminal(41), "yazi", Some("nu sidebar.nu"), 0, 0, 16, 24),
            terminal_pane(PaneId::Terminal(42), "editor", Some("hx"), 16, 0, 39, 24),
            terminal_pane(PaneId::Terminal(43), "shell", Some("bash"), 55, 0, 25, 24),
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
        let open_layout_widths = super::LayoutWidths::new(20, SidebarState::Open);
        let panes = vec![
            build_preserved_terminal_pane_spec(&terminal_pane(
                PaneId::Terminal(2),
                "editor",
                Some("hx"),
                16,
                0,
                64,
                24,
            ))
            .unwrap(),
            build_preserved_terminal_pane_spec(&terminal_pane(
                PaneId::Terminal(3),
                "shell",
                Some("bash"),
                16,
                0,
                64,
                24,
            ))
            .unwrap(),
            build_preserved_terminal_pane_spec(&terminal_pane(
                PaneId::Terminal(4),
                "term",
                Some("zsh"),
                16,
                0,
                64,
                24,
            ))
            .unwrap(),
        ];
        let single = build_single_family_kdl(&panes, &open_layout_widths);
        let vertical = build_vertical_split_family_kdl(&panes, &open_layout_widths).unwrap();
        let bottom = build_bottom_terminal_family_kdl(&panes, &open_layout_widths).unwrap();

        assert!(!single.contains("children"));
        assert!(!vertical.contains("children"));
        assert!(!bottom.contains("children"));
        assert!(single.contains("command \"hx\""));
        assert!(vertical.contains("command \"bash\""));
        assert!(bottom.contains("command \"zsh\""));
    }

    #[test]
    fn builds_single_sidebar_layout_with_one_content_pane() {
        let sidebar_spec = build_preserved_terminal_pane_spec(&terminal_pane(
            PaneId::Terminal(1),
            "sidebar",
            Some("nu /tmp/launch_sidebar_yazi.nu"),
            0,
            0,
            16,
            24,
        ))
        .unwrap();
        let content_spec = build_preserved_terminal_pane_spec(&terminal_pane(
            PaneId::Terminal(2),
            "editor",
            Some("hx"),
            16,
            0,
            64,
            24,
        ))
        .unwrap();
        let content_layout = build_content_layout_kdl(
            LayoutVariant {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            },
            &sidebar_spec,
            &[content_spec],
            20,
        )
        .unwrap();

        assert!(content_layout.contains("name=\"sidebar\""));
        assert!(content_layout.contains("pane stacked=true"));
        assert!(content_layout.contains("size \"20%\""));
        assert!(content_layout.contains("size \"80%\""));
        assert!(content_layout.contains("command \"hx\""));
    }

    #[test]
    fn builds_override_layout_from_embedded_templates_with_configured_segments() {
        unsafe {
            std::env::set_var("YAZELIX_RUNTIME_DIR", "/tmp/yazelix-runtime");
        }

        let layout_plan = build_override_layout_kdl(
            LayoutVariant {
                family: LayoutFamily::Single,
                sidebar_state: SidebarState::Open,
            },
            &[
                terminal_pane(
                    PaneId::Terminal(1),
                    "sidebar",
                    Some("nu /tmp/yazelix-runtime/configs/zellij/scripts/launch_sidebar_yazi.nu"),
                    0,
                    0,
                    16,
                    24,
                ),
                terminal_pane(PaneId::Terminal(2), "editor", Some("hx"), 16, 0, 64, 24),
                terminal_pane(PaneId::Terminal(3), "shell", Some("bash"), 16, 0, 64, 24),
            ],
            None,
            &OverrideLayoutConfig {
                zjstatus_segments: ZjstatusSegments {
                    widget_tray: "#[fg=#00ff88,bold][editor: {command_editor}] {command_cpu}"
                        .into(),
                    custom_text: "#[fg=#ffff00,bold][TEST] ".into(),
                },
                sidebar_width_percent: 25,
            },
        )
        .unwrap();
        let layout_kdl = &layout_plan.layout_kdl;

        assert!(layout_kdl.contains("tab_template name=\"ui\""));
        assert!(layout_kdl.contains("swap_tiled_layout name=\"single_open\""));
        assert!(
            layout_kdl.contains("file:/tmp/yazelix-runtime/configs/zellij/plugins/zjstatus.wasm")
        );
        assert!(layout_kdl.contains("#[fg=#00ff88,bold][editor: {command_editor}] {command_cpu}"));
        assert!(layout_kdl.contains("#[fg=#ffff00,bold][TEST]"));
        assert!(!layout_kdl.contains("{swap_layout}"));
        assert!(layout_kdl.contains("size \"25%\""));
        assert!(layout_kdl.contains("size \"75%\""));
        assert!(layout_kdl.contains("command \"hx\""));
        assert!(layout_kdl.contains("command \"bash\""));
        assert!(layout_plan.retain_existing_terminal_panes);

        unsafe {
            std::env::remove_var("YAZELIX_RUNTIME_DIR");
        }
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

    #[test]
    fn terminal_pane_node_preserves_args_and_escaping() {
        let pane_spec = build_preserved_terminal_pane_spec(&terminal_pane(
            PaneId::Terminal(7),
            "editor",
            Some("env YAZI_ID=demo hx /tmp/file.txt"),
            0,
            0,
            10,
            10,
        ))
        .unwrap();
        let pane_node = build_terminal_pane_node(&pane_spec, 2, Some(String::from("80%")));
        assert!(pane_node.contains("command \"env\""));
        assert!(pane_node.contains("args \"YAZI_ID=demo\" \"hx\" \"/tmp/file.txt\""));
        assert!(pane_node.contains("name=\"editor\""));
        assert!(pane_node.contains("size \"80%\""));
    }
}
