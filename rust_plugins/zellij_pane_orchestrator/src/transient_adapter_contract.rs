use crate::transient_pane_contract::{TransientPaneIdentityContract, TransientPaneKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransientPostCloseHook {
    None,
    RefreshSidebarYazi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct YazelixTransientPaneAdapter {
    pub identity: TransientPaneIdentityContract,
    pub wrapper_relative_path: &'static str,
    pub post_close_hook: TransientPostCloseHook,
}

pub fn yazelix_transient_adapter(kind: TransientPaneKind) -> YazelixTransientPaneAdapter {
    match kind {
        TransientPaneKind::Popup => YazelixTransientPaneAdapter {
            identity: TransientPaneIdentityContract {
                pane_title: "yzx_popup",
                command_marker: Some("nushell/scripts/zellij_wrappers/yzx_popup_program.nu"),
            },
            wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_popup_program.nu",
            post_close_hook: TransientPostCloseHook::RefreshSidebarYazi,
        },
        TransientPaneKind::Menu => YazelixTransientPaneAdapter {
            identity: TransientPaneIdentityContract {
                pane_title: "yzx_menu",
                command_marker: Some("nushell/scripts/zellij_wrappers/yzx_menu_popup.nu"),
            },
            wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu",
            post_close_hook: TransientPostCloseHook::None,
        },
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::{yazelix_transient_adapter, TransientPostCloseHook, YazelixTransientPaneAdapter};
    use crate::transient_pane_contract::{TransientPaneIdentityContract, TransientPaneKind};

    // Defends: Yazelix popup and menu adapters keep wrapper identity separate from generic transient policy.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn exposes_yazelix_popup_and_menu_adapters() {
        assert_eq!(
            yazelix_transient_adapter(TransientPaneKind::Popup),
            YazelixTransientPaneAdapter {
                identity: TransientPaneIdentityContract {
                    pane_title: "yzx_popup",
                    command_marker: Some("nushell/scripts/zellij_wrappers/yzx_popup_program.nu"),
                },
                wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_popup_program.nu",
                post_close_hook: TransientPostCloseHook::RefreshSidebarYazi,
            }
        );
        assert_eq!(
            yazelix_transient_adapter(TransientPaneKind::Menu),
            YazelixTransientPaneAdapter {
                identity: TransientPaneIdentityContract {
                    pane_title: "yzx_menu",
                    command_marker: Some("nushell/scripts/zellij_wrappers/yzx_menu_popup.nu"),
                },
                wrapper_relative_path: "nushell/scripts/zellij_wrappers/yzx_menu_popup.nu",
                post_close_hook: TransientPostCloseHook::None,
            }
        );
    }
}
