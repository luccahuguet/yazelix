use crate::pane_contract::FocusContextPolicy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SidebarVisibilityTogglePlan {
    OpenPreservingFocus,
    ClosePreservingFocus,
    CloseAndFocusEditor,
    CloseAndFocusFallback,
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
        SidebarVisibilityTogglePlan::OpenPreservingFocus
    } else if focus_context == FocusContextPolicy::Sidebar && has_editor {
        SidebarVisibilityTogglePlan::CloseAndFocusEditor
    } else if focus_context == FocusContextPolicy::Sidebar && has_focus_fallback {
        SidebarVisibilityTogglePlan::CloseAndFocusFallback
    } else {
        SidebarVisibilityTogglePlan::ClosePreservingFocus
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

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::{
        resolve_sidebar_focus_toggle, resolve_sidebar_visibility_toggle, SidebarFocusTogglePlan,
        SidebarVisibilityTogglePlan,
    };
    use crate::pane_contract::FocusContextPolicy;

    // Defends: opening the sidebar preserves the current focus context instead of forcing a focus jump.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=1 total=7/10
    #[test]
    fn opening_sidebar_preserves_current_focus() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(true, FocusContextPolicy::Editor, true, true),
            SidebarVisibilityTogglePlan::OpenPreservingFocus
        );
    }

    // Defends: closing a focused sidebar prefers the editor when that fallback exists.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn closing_focused_sidebar_prefers_editor_fallback() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(false, FocusContextPolicy::Sidebar, true, true),
            SidebarVisibilityTogglePlan::CloseAndFocusEditor
        );
    }

    // Defends: closing a focused sidebar falls back to a non-sidebar target when the editor is missing.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn closing_focused_sidebar_uses_non_sidebar_fallback_when_editor_missing() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(false, FocusContextPolicy::Sidebar, false, true),
            SidebarVisibilityTogglePlan::CloseAndFocusFallback
        );
    }

    // Defends: explicit sidebar focus toggles reopen a closed sidebar and focus it.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=1 total=7/10
    #[test]
    fn explicit_focus_toggle_reopens_closed_sidebar_and_focuses_it() {
        assert_eq!(
            resolve_sidebar_focus_toggle(FocusContextPolicy::Editor, true, true, true),
            SidebarFocusTogglePlan::OpenAndFocusSidebar
        );
    }

    // Defends: explicit sidebar focus toggles return from sidebar focus back to the editor.
    // Strength: defect=1 behavior=2 resilience=2 cost=1 uniqueness=1 total=7/10
    #[test]
    fn explicit_focus_toggle_returns_from_sidebar_to_editor() {
        assert_eq!(
            resolve_sidebar_focus_toggle(FocusContextPolicy::Sidebar, true, false, true),
            SidebarFocusTogglePlan::FocusEditor
        );
    }
}
