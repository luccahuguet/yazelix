#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupTogglePlan {
    OpenPopup,
    FocusPopup,
    ClosePopup,
}

pub fn resolve_popup_toggle(has_popup: bool, popup_is_focused: bool) -> PopupTogglePlan {
    if !has_popup {
        PopupTogglePlan::OpenPopup
    } else if popup_is_focused {
        PopupTogglePlan::ClosePopup
    } else {
        PopupTogglePlan::FocusPopup
    }
}

#[cfg(test)]
mod tests {
    use super::{PopupTogglePlan, resolve_popup_toggle};

    #[test]
    fn opens_popup_when_missing() {
        assert_eq!(resolve_popup_toggle(false, false), PopupTogglePlan::OpenPopup);
    }

    #[test]
    fn focuses_existing_popup_when_unfocused() {
        assert_eq!(resolve_popup_toggle(true, false), PopupTogglePlan::FocusPopup);
    }

    #[test]
    fn closes_existing_popup_when_already_focused() {
        assert_eq!(resolve_popup_toggle(true, true), PopupTogglePlan::ClosePopup);
    }
}
