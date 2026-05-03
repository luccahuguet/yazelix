use crate::pane_contract::FocusContextPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarVisibilityAction {
    Open,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarPostLayoutFocus {
    Preserve,
    MoveLeftToSidebar,
    MoveRightToNonSidebar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarVisibilityTogglePlan {
    pub action: SidebarVisibilityAction,
    pub post_layout_focus: SidebarPostLayoutFocus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarFocusNudgeDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SidebarFocusNudge {
    pub delay_ms: u64,
    pub direction: SidebarFocusNudgeDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarFocusTogglePlan {
    FocusEditor,
    FocusSidebar,
    OpenAndFocusSidebar,
    MissingTarget,
}

pub fn resolve_sidebar_visibility_toggle(
    sidebar_is_closed: bool,
    focus_context: FocusContextPolicy,
    has_editor: bool,
    has_focus_fallback: bool,
) -> SidebarVisibilityTogglePlan {
    if sidebar_is_closed {
        SidebarVisibilityTogglePlan {
            action: SidebarVisibilityAction::Open,
            post_layout_focus: SidebarPostLayoutFocus::Preserve,
        }
    } else if focus_context == FocusContextPolicy::Sidebar && has_editor {
        SidebarVisibilityTogglePlan {
            action: SidebarVisibilityAction::Close,
            post_layout_focus: SidebarPostLayoutFocus::MoveRightToNonSidebar,
        }
    } else if focus_context == FocusContextPolicy::Sidebar && has_focus_fallback {
        SidebarVisibilityTogglePlan {
            action: SidebarVisibilityAction::Close,
            post_layout_focus: SidebarPostLayoutFocus::MoveRightToNonSidebar,
        }
    } else {
        SidebarVisibilityTogglePlan {
            action: SidebarVisibilityAction::Close,
            post_layout_focus: SidebarPostLayoutFocus::Preserve,
        }
    }
}

pub fn resolve_sidebar_focus_toggle(
    focus_context: FocusContextPolicy,
    sidebar_exists: bool,
    sidebar_is_closed: bool,
    has_editor: bool,
) -> SidebarFocusTogglePlan {
    if focus_context == FocusContextPolicy::Sidebar {
        if has_editor {
            SidebarFocusTogglePlan::FocusEditor
        } else {
            SidebarFocusTogglePlan::MissingTarget
        }
    } else if !sidebar_exists {
        SidebarFocusTogglePlan::MissingTarget
    } else if sidebar_is_closed {
        SidebarFocusTogglePlan::OpenAndFocusSidebar
    } else {
        SidebarFocusTogglePlan::FocusSidebar
    }
}

pub fn resolve_sidebar_hide(
    sidebar_is_closed: bool,
    focus_context: FocusContextPolicy,
    has_editor: bool,
    has_focus_fallback: bool,
) -> Option<SidebarPostLayoutFocus> {
    if sidebar_is_closed {
        return None;
    }

    if focus_context == FocusContextPolicy::Sidebar && (has_editor || has_focus_fallback) {
        Some(SidebarPostLayoutFocus::MoveRightToNonSidebar)
    } else {
        Some(SidebarPostLayoutFocus::Preserve)
    }
}

pub fn sidebar_close_swap_steps(active_layout_is_base: bool) -> usize {
    if active_layout_is_base {
        2
    } else {
        1
    }
}

pub fn sidebar_post_layout_focus_nudges(
    post_layout_focus: SidebarPostLayoutFocus,
) -> &'static [SidebarFocusNudge] {
    const MOVE_LEFT_TO_SIDEBAR: [SidebarFocusNudge; 3] = [
        SidebarFocusNudge {
            delay_ms: 35,
            direction: SidebarFocusNudgeDirection::Left,
        },
        SidebarFocusNudge {
            delay_ms: 70,
            direction: SidebarFocusNudgeDirection::Left,
        },
        SidebarFocusNudge {
            delay_ms: 105,
            direction: SidebarFocusNudgeDirection::Left,
        },
    ];
    const MOVE_RIGHT_TO_NON_SIDEBAR: [SidebarFocusNudge; 2] = [
        SidebarFocusNudge {
            delay_ms: 35,
            direction: SidebarFocusNudgeDirection::Right,
        },
        SidebarFocusNudge {
            delay_ms: 105,
            direction: SidebarFocusNudgeDirection::Right,
        },
    ];

    match post_layout_focus {
        SidebarPostLayoutFocus::Preserve => &[],
        SidebarPostLayoutFocus::MoveLeftToSidebar => &MOVE_LEFT_TO_SIDEBAR,
        SidebarPostLayoutFocus::MoveRightToNonSidebar => &MOVE_RIGHT_TO_NON_SIDEBAR,
    }
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::{
        resolve_sidebar_focus_toggle, resolve_sidebar_hide, resolve_sidebar_visibility_toggle,
        sidebar_close_swap_steps, sidebar_post_layout_focus_nudges, SidebarFocusNudge,
        SidebarFocusNudgeDirection, SidebarFocusTogglePlan, SidebarPostLayoutFocus,
        SidebarVisibilityAction, SidebarVisibilityTogglePlan,
    };
    use crate::pane_contract::FocusContextPolicy;

    // Defends: opening the sidebar preserves the current focus context instead of forcing a focus jump.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn opening_sidebar_preserves_current_focus() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(true, FocusContextPolicy::Editor, true, true),
            SidebarVisibilityTogglePlan {
                action: SidebarVisibilityAction::Open,
                post_layout_focus: SidebarPostLayoutFocus::Preserve
            }
        );
    }

    // Defends: closing a focused sidebar prefers the editor when that fallback exists.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn closing_focused_sidebar_prefers_editor_fallback() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(false, FocusContextPolicy::Sidebar, true, true),
            SidebarVisibilityTogglePlan {
                action: SidebarVisibilityAction::Close,
                post_layout_focus: SidebarPostLayoutFocus::MoveRightToNonSidebar
            }
        );
    }

    // Defends: closing a focused sidebar falls back to a non-sidebar target when the editor is missing.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn closing_focused_sidebar_uses_non_sidebar_fallback_when_editor_missing() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(false, FocusContextPolicy::Sidebar, false, true),
            SidebarVisibilityTogglePlan {
                action: SidebarVisibilityAction::Close,
                post_layout_focus: SidebarPostLayoutFocus::MoveRightToNonSidebar
            }
        );
    }

    // Regression: the programmatic hide path must move focus off the sidebar before a missing editor pane is opened.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn hide_focused_sidebar_uses_non_sidebar_fallback_when_editor_missing() {
        assert_eq!(
            resolve_sidebar_hide(false, FocusContextPolicy::Sidebar, false, true),
            Some(SidebarPostLayoutFocus::MoveRightToNonSidebar)
        );
    }

    // Defends: hiding an already hidden sidebar is a no-op and does not inject focus motion.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn hide_closed_sidebar_is_noop() {
        assert_eq!(
            resolve_sidebar_hide(true, FocusContextPolicy::Sidebar, false, true),
            None
        );
    }

    // Regression: closing the startup BASE layout needs two swaps because the first swap is the open single layout.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn close_from_base_layout_skips_open_single_layout() {
        assert_eq!(sidebar_close_swap_steps(true), 2);
        assert_eq!(sidebar_close_swap_steps(false), 1);
    }

    // Defends: closing a non-focused sidebar does not inject extra focus motion.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn closing_unfocused_sidebar_preserves_current_focus() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(false, FocusContextPolicy::Editor, true, true),
            SidebarVisibilityTogglePlan {
                action: SidebarVisibilityAction::Close,
                post_layout_focus: SidebarPostLayoutFocus::Preserve
            }
        );
    }

    // Defends: explicit sidebar focus toggles reopen a closed sidebar and focus it.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn explicit_focus_toggle_reopens_closed_sidebar_and_focuses_it() {
        assert_eq!(
            resolve_sidebar_focus_toggle(FocusContextPolicy::Editor, true, true, true),
            SidebarFocusTogglePlan::OpenAndFocusSidebar
        );
    }

    // Defends: explicit sidebar focus toggles return from sidebar focus back to the editor.
    // Strength: defect=1 behavior=2 resilience=2 cost=2 uniqueness=1 total=8/10
    #[test]
    fn explicit_focus_toggle_returns_from_sidebar_to_editor() {
        assert_eq!(
            resolve_sidebar_focus_toggle(FocusContextPolicy::Sidebar, true, false, true),
            SidebarFocusTogglePlan::FocusEditor
        );
    }

    // Defends: the reusable sidebar contract owns the post-layout focus nudge sequence.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn post_layout_focus_nudges_are_contract_owned() {
        assert_eq!(
            sidebar_post_layout_focus_nudges(SidebarPostLayoutFocus::MoveLeftToSidebar),
            [
                SidebarFocusNudge {
                    delay_ms: 35,
                    direction: SidebarFocusNudgeDirection::Left,
                },
                SidebarFocusNudge {
                    delay_ms: 70,
                    direction: SidebarFocusNudgeDirection::Left,
                },
                SidebarFocusNudge {
                    delay_ms: 105,
                    direction: SidebarFocusNudgeDirection::Left,
                },
            ]
        );
        assert!(sidebar_post_layout_focus_nudges(SidebarPostLayoutFocus::Preserve).is_empty());
    }
}
