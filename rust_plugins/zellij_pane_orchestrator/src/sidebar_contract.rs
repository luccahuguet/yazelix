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

#[cfg(test)]
mod tests {
    use super::{
        SidebarFocusTogglePlan, SidebarVisibilityTogglePlan, resolve_sidebar_focus_toggle,
        resolve_sidebar_visibility_toggle,
    };
    use crate::pane_contract::FocusContextPolicy;

    #[test]
    fn opening_sidebar_preserves_current_focus() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(true, FocusContextPolicy::Editor, true, true),
            SidebarVisibilityTogglePlan::OpenPreservingFocus
        );
    }

    #[test]
    fn closing_focused_sidebar_prefers_editor_fallback() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(false, FocusContextPolicy::Sidebar, true, true),
            SidebarVisibilityTogglePlan::CloseAndFocusEditor
        );
    }

    #[test]
    fn closing_focused_sidebar_uses_non_sidebar_fallback_when_editor_missing() {
        assert_eq!(
            resolve_sidebar_visibility_toggle(false, FocusContextPolicy::Sidebar, false, true),
            SidebarVisibilityTogglePlan::CloseAndFocusFallback
        );
    }

    #[test]
    fn explicit_focus_toggle_reopens_closed_sidebar_and_focuses_it() {
        assert_eq!(
            resolve_sidebar_focus_toggle(FocusContextPolicy::Editor, true, true, true),
            SidebarFocusTogglePlan::OpenAndFocusSidebar
        );
    }

    #[test]
    fn explicit_focus_toggle_returns_from_sidebar_to_editor() {
        assert_eq!(
            resolve_sidebar_focus_toggle(FocusContextPolicy::Sidebar, true, false, true),
            SidebarFocusTogglePlan::FocusEditor
        );
    }
}
