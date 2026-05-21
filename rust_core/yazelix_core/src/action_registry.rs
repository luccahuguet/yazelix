pub const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";
pub const YZPP_PLUGIN_ALIAS: &str = "yzpp";

pub const ZELLIJ_SEMANTIC_KEYBINDING_DIAGNOSTICS: &[&str] = &[
    "unsupported_zellij_keybinding_action",
    "invalid_zellij_keybindings",
    "invalid_zellij_keybinding_keys",
    "invalid_zellij_keybinding_key",
    "duplicate_zellij_keybinding",
];

pub const YAZI_SEMANTIC_KEYBINDING_DIAGNOSTICS: &[&str] = &[
    "unsupported_yazi_keybinding_action",
    "invalid_yazi_keybindings",
    "invalid_yazi_keybinding_keys",
    "invalid_yazi_keybinding_key",
    "duplicate_yazi_keybinding",
    "disabled_required_yazi_keybinding",
];

pub const ZELLIJ_NATIVE_KEYBINDING_DIAGNOSTICS: &[&str] = &[
    "unsupported_zellij_native_keybinding_action",
    "invalid_zellij_native_keybindings",
    "invalid_zellij_native_keybinding_keys",
    "invalid_zellij_native_keybinding_key",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YazelixActionOwner {
    Zellij,
    Yazi,
    Editor,
}

impl YazelixActionOwner {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Zellij => "zellij",
            Self::Yazi => "yazi",
            Self::Editor => "editor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YazelixActionBackend {
    ZellijPaneOrchestratorMessage,
    ZellijPluginMessage,
    ZellijNativeAction,
    YaziKeymapCommand,
    EditorCommand,
}

impl YazelixActionBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ZellijPaneOrchestratorMessage => "zellij_pane_orchestrator_message",
            Self::ZellijPluginMessage => "zellij_plugin_message",
            Self::ZellijNativeAction => "zellij_native_action",
            Self::YaziKeymapCommand => "yazi_keymap_command",
            Self::EditorCommand => "editor_command",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YazelixActionDisablePolicy {
    Optional,
    Required,
}

impl YazelixActionDisablePolicy {
    pub const fn empty_binding_list_allowed(self) -> bool {
        match self {
            Self::Optional => true,
            Self::Required => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct YazelixActionMetadata {
    pub id: &'static str,
    pub local_id: &'static str,
    pub label: &'static str,
    pub owner: YazelixActionOwner,
    pub backend: YazelixActionBackend,
    pub default_keys: &'static [&'static str],
    pub generated_command: &'static str,
    pub disable_policy: YazelixActionDisablePolicy,
    pub diagnostics: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct ZellijActionSpec {
    pub action: YazelixActionMetadata,
    pub mode: &'static str,
    pub plugin_alias: &'static str,
    pub message_name: &'static str,
    pub payload: Option<&'static str>,
}

#[derive(Debug, Clone, Copy)]
pub struct ZellijNativeKeybindingBlock {
    pub mode: &'static str,
    pub action_lines: &'static [&'static str],
}

#[derive(Debug, Clone, Copy)]
pub struct ZellijNativeKeybindingSpec {
    pub action: YazelixActionMetadata,
    pub blocks: &'static [ZellijNativeKeybindingBlock],
}

#[derive(Debug, Clone, Copy)]
pub struct YaziActionSpec {
    pub action: YazelixActionMetadata,
    pub section: &'static str,
    pub keymap_list: &'static str,
    pub description: &'static str,
}

const fn zellij_action(
    local_id: &'static str,
    scoped_id: &'static str,
    label: &'static str,
    mode: &'static str,
    message_name: &'static str,
    payload: Option<&'static str>,
    default_keys: &'static [&'static str],
    generated_command: &'static str,
) -> ZellijActionSpec {
    zellij_plugin_action(
        local_id,
        scoped_id,
        label,
        mode,
        PANE_ORCHESTRATOR_PLUGIN_ALIAS,
        YazelixActionBackend::ZellijPaneOrchestratorMessage,
        message_name,
        payload,
        default_keys,
        generated_command,
    )
}

const fn zellij_plugin_action(
    local_id: &'static str,
    scoped_id: &'static str,
    label: &'static str,
    mode: &'static str,
    plugin_alias: &'static str,
    backend: YazelixActionBackend,
    message_name: &'static str,
    payload: Option<&'static str>,
    default_keys: &'static [&'static str],
    generated_command: &'static str,
) -> ZellijActionSpec {
    ZellijActionSpec {
        action: YazelixActionMetadata {
            id: scoped_id,
            local_id,
            label,
            owner: YazelixActionOwner::Zellij,
            backend,
            default_keys,
            generated_command,
            disable_policy: YazelixActionDisablePolicy::Optional,
            diagnostics: ZELLIJ_SEMANTIC_KEYBINDING_DIAGNOSTICS,
        },
        mode,
        plugin_alias,
        message_name,
        payload,
    }
}

const fn yazi_action(
    local_id: &'static str,
    scoped_id: &'static str,
    label: &'static str,
    section: &'static str,
    keymap_list: &'static str,
    default_keys: &'static [&'static str],
    generated_command: &'static str,
    description: &'static str,
) -> YaziActionSpec {
    YaziActionSpec {
        action: YazelixActionMetadata {
            id: scoped_id,
            local_id,
            label,
            owner: YazelixActionOwner::Yazi,
            backend: YazelixActionBackend::YaziKeymapCommand,
            default_keys,
            generated_command,
            disable_policy: YazelixActionDisablePolicy::Optional,
            diagnostics: YAZI_SEMANTIC_KEYBINDING_DIAGNOSTICS,
        },
        section,
        keymap_list,
        description,
    }
}

const fn zellij_native_action(
    local_id: &'static str,
    label: &'static str,
    default_keys: &'static [&'static str],
    generated_command: &'static str,
    blocks: &'static [ZellijNativeKeybindingBlock],
) -> ZellijNativeKeybindingSpec {
    ZellijNativeKeybindingSpec {
        action: YazelixActionMetadata {
            id: local_id,
            local_id,
            label,
            owner: YazelixActionOwner::Zellij,
            backend: YazelixActionBackend::ZellijNativeAction,
            default_keys,
            generated_command,
            disable_policy: YazelixActionDisablePolicy::Optional,
            diagnostics: ZELLIJ_NATIVE_KEYBINDING_DIAGNOSTICS,
        },
        blocks,
    }
}

pub const ZELLIJ_ACTIONS: &[ZellijActionSpec] = &[
    zellij_action(
        "open_workspace_terminal",
        "zellij.open_workspace_terminal",
        "Open a terminal in the current workspace root",
        "shared",
        "open_workspace_terminal",
        None,
        &["Alt m"],
        "MessagePlugin yazelix_pane_orchestrator { name \"open_workspace_terminal\" }",
    ),
    zellij_plugin_action(
        "popup",
        "zellij.popup",
        "Toggle the managed popup program",
        "shared",
        YZPP_PLUGIN_ALIAS,
        YazelixActionBackend::ZellijPluginMessage,
        "toggle",
        Some("popup"),
        &[],
        "MessagePlugin yzpp { name \"toggle\" payload \"popup\" }",
    ),
    zellij_plugin_action(
        "bottom_popup",
        "zellij.bottom_popup",
        "Toggle the bottom popup slot",
        "shared",
        YZPP_PLUGIN_ALIAS,
        YazelixActionBackend::ZellijPluginMessage,
        "toggle",
        Some("bottom_popup"),
        &["Alt Shift J"],
        "MessagePlugin yzpp { name \"toggle\" payload \"bottom_popup\" }",
    ),
    zellij_plugin_action(
        "top_popup",
        "zellij.top_popup",
        "Toggle the top popup slot",
        "shared",
        YZPP_PLUGIN_ALIAS,
        YazelixActionBackend::ZellijPluginMessage,
        "toggle",
        Some("top_popup"),
        &["Alt Shift K"],
        "MessagePlugin yzpp { name \"toggle\" payload \"top_popup\" }",
    ),
    zellij_plugin_action(
        "menu",
        "zellij.menu",
        "Open the Yazelix command palette popup",
        "shared",
        YZPP_PLUGIN_ALIAS,
        YazelixActionBackend::ZellijPluginMessage,
        "toggle",
        Some("menu"),
        &["Alt Shift M"],
        "MessagePlugin yzpp { name \"toggle\" payload \"menu\" }",
    ),
    zellij_plugin_action(
        "config",
        "zellij.config",
        "Open the Yazelix config UI popup",
        "shared",
        YZPP_PLUGIN_ALIAS,
        YazelixActionBackend::ZellijPluginMessage,
        "toggle",
        Some("config"),
        &["Alt Shift C"],
        "MessagePlugin yzpp { name \"toggle\" payload \"config\" }",
    ),
    zellij_action(
        "move_focus_left_or_tab",
        "zellij.move_focus_left_or_tab",
        "Move focus left, falling back to the previous tab",
        "shared_except \"locked\"",
        "move_focus_left_or_tab",
        None,
        &["Alt h", "Alt Left"],
        "MessagePlugin yazelix_pane_orchestrator { name \"move_focus_left_or_tab\" }",
    ),
    zellij_action(
        "move_focus_right_or_tab",
        "zellij.move_focus_right_or_tab",
        "Move focus right, falling back to the next tab",
        "shared_except \"locked\"",
        "move_focus_right_or_tab",
        None,
        &["Alt l", "Alt Right"],
        "MessagePlugin yazelix_pane_orchestrator { name \"move_focus_right_or_tab\" }",
    ),
    zellij_action(
        "toggle_editor_sidebar_focus",
        "zellij.toggle_editor_sidebar_focus",
        "Toggle focus between the managed editor and left sidebar",
        "shared_except \"locked\"",
        "toggle_editor_sidebar_focus",
        None,
        &["Ctrl y"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_editor_sidebar_focus\" }",
    ),
    zellij_action(
        "toggle_editor_right_sidebar_focus",
        "zellij.toggle_editor_right_sidebar_focus",
        "Toggle focus between the managed editor and right agent sidebar",
        "shared_except \"locked\"",
        "toggle_editor_right_sidebar_focus",
        None,
        &["Ctrl Shift Y"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_editor_right_sidebar_focus\" }",
    ),
    zellij_action(
        "toggle_left_sidebar",
        "zellij.toggle_left_sidebar",
        "Show or hide the managed left sidebar",
        "shared_except \"locked\"",
        "toggle_sidebar",
        None,
        &["Alt Shift H"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_sidebar\" }",
    ),
    zellij_action(
        "open_codex_agent_right",
        "zellij.open_codex_agent_right",
        "Toggle the managed Codex agent sidebar",
        "shared_except \"locked\"",
        "toggle_agent_sidebar",
        None,
        &["Alt Shift L"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_agent_sidebar\" }",
    ),
    zellij_action(
        "smart_reveal",
        "zellij.smart_reveal",
        "Reveal the editor path in the managed sidebar",
        "shared_except \"locked\"",
        "smart_reveal",
        None,
        &["Alt r"],
        "MessagePlugin yazelix_pane_orchestrator { name \"smart_reveal\" }",
    ),
    zellij_action(
        "previous_family",
        "zellij.previous_family",
        "Switch to the previous Yazelix layout family",
        "shared_except \"locked\"",
        "previous_family",
        None,
        &["Alt ["],
        "MessagePlugin yazelix_pane_orchestrator { name \"previous_family\" }",
    ),
    zellij_action(
        "next_family",
        "zellij.next_family",
        "Switch to the next Yazelix layout family",
        "shared_except \"locked\"",
        "next_family",
        None,
        &["Alt ]"],
        "MessagePlugin yazelix_pane_orchestrator { name \"next_family\" }",
    ),
];

pub const ZELLIJ_NATIVE_KEYBINDINGS: &[ZellijNativeKeybindingSpec] = &[
    zellij_native_action(
        "move_tab_left_unbind",
        "Unbind the default move-tab-left key",
        &["Alt i"],
        "unbind Alt i",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &[],
        }],
    ),
    zellij_native_action(
        "move_tab_left",
        "Move tab left",
        &["Ctrl Shift H"],
        "MoveTab \"Left\"",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["MoveTab \"Left\""],
        }],
    ),
    zellij_native_action(
        "move_tab_right_unbind",
        "Unbind the default move-tab-right key",
        &["Alt o"],
        "unbind Alt o",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &[],
        }],
    ),
    zellij_native_action(
        "move_tab_right",
        "Move tab right",
        &["Ctrl Shift L"],
        "MoveTab \"Right\"",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["MoveTab \"Right\""],
        }],
    ),
    zellij_native_action(
        "new_pane_unbind",
        "Unbind the default new-pane key",
        &["Alt n"],
        "unbind Alt n",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &[],
        }],
    ),
    zellij_native_action(
        "go_to_tab_1",
        "Go to tab 1",
        &["Alt 1"],
        "GoToTab 1",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 1"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_2",
        "Go to tab 2",
        &["Alt 2"],
        "GoToTab 2",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 2"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_3",
        "Go to tab 3",
        &["Alt 3"],
        "GoToTab 3",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 3"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_4",
        "Go to tab 4",
        &["Alt 4"],
        "GoToTab 4",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 4"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_5",
        "Go to tab 5",
        &["Alt 5"],
        "GoToTab 5",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 5"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_6",
        "Go to tab 6",
        &["Alt 6"],
        "GoToTab 6",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 6"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_7",
        "Go to tab 7",
        &["Alt 7"],
        "GoToTab 7",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 7"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_8",
        "Go to tab 8",
        &["Alt 8"],
        "GoToTab 8",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 8"],
        }],
    ),
    zellij_native_action(
        "go_to_tab_9",
        "Go to tab 9",
        &["Alt 9"],
        "GoToTab 9",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToTab 9"],
        }],
    ),
    zellij_native_action(
        "toggle_focus_fullscreen",
        "Toggle focused pane fullscreen",
        &["Alt Shift F"],
        "ToggleFocusFullscreen; SwitchToMode \"Normal\"",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["ToggleFocusFullscreen", "SwitchToMode \"Normal\""],
        }],
    ),
    zellij_native_action(
        "previous_tab",
        "Go to previous tab",
        &["Alt q"],
        "GoToPreviousTab",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToPreviousTab"],
        }],
    ),
    zellij_native_action(
        "next_tab",
        "Go to next tab",
        &["Alt w"],
        "GoToNextTab",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["GoToNextTab"],
        }],
    ),
    zellij_native_action(
        "move_pane_down",
        "Move pane down",
        &["Ctrl Shift J"],
        "MovePane \"Down\"",
        &[ZellijNativeKeybindingBlock {
            mode: "shared_except \"locked\"",
            action_lines: &["MovePane \"Down\""],
        }],
    ),
    zellij_native_action(
        "move_pane_up",
        "Move pane up",
        &["Ctrl Shift K"],
        "MovePane \"Up\"",
        &[ZellijNativeKeybindingBlock {
            mode: "shared_except \"locked\"",
            action_lines: &["MovePane \"Up\""],
        }],
    ),
    zellij_native_action(
        "selection_cycle_unbind",
        "Unbind terminal-specific selection-cycle conflicts",
        &["Alt (", "Alt )"],
        "unbind Alt ( / Alt )",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &[],
        }],
    ),
    zellij_native_action(
        "toggle_pane_in_group_unbind",
        "Unbind default pane grouping key",
        &["Alt p"],
        "unbind Alt p",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &[],
        }],
    ),
    zellij_native_action(
        "toggle_pane_in_group",
        "Toggle pane grouping",
        &["Ctrl Alt p"],
        "TogglePaneInGroup",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["TogglePaneInGroup"],
        }],
    ),
    zellij_native_action(
        "toggle_group_marking",
        "Toggle group marking",
        &["Ctrl Alt Shift P"],
        "ToggleGroupMarking",
        &[ZellijNativeKeybindingBlock {
            mode: "shared",
            action_lines: &["ToggleGroupMarking"],
        }],
    ),
    zellij_native_action(
        "locked_mode_unbind",
        "Unbind default locked-mode key outside locked mode",
        &["Ctrl g"],
        "unbind Ctrl g",
        &[ZellijNativeKeybindingBlock {
            mode: "shared_except \"locked\"",
            action_lines: &[],
        }],
    ),
    zellij_native_action(
        "locked_mode",
        "Toggle locked mode",
        &["Ctrl Alt g"],
        "SwitchToMode \"Locked\" / SwitchToMode \"Normal\"",
        &[
            ZellijNativeKeybindingBlock {
                mode: "shared_except \"locked\"",
                action_lines: &["SwitchToMode \"Locked\""],
            },
            ZellijNativeKeybindingBlock {
                mode: "locked",
                action_lines: &["SwitchToMode \"Normal\""],
            },
        ],
    ),
    zellij_native_action(
        "scroll_mode_unbind",
        "Unbind default scroll-mode key",
        &["Ctrl s"],
        "unbind Ctrl s",
        &[
            ZellijNativeKeybindingBlock {
                mode: "shared_except \"scroll\" \"locked\"",
                action_lines: &[],
            },
            ZellijNativeKeybindingBlock {
                mode: "scroll",
                action_lines: &[],
            },
        ],
    ),
    zellij_native_action(
        "scroll_mode",
        "Toggle scroll mode",
        &["Ctrl Alt s"],
        "SwitchToMode \"Scroll\" / SwitchToMode \"Normal\"",
        &[
            ZellijNativeKeybindingBlock {
                mode: "shared_except \"scroll\" \"locked\"",
                action_lines: &["SwitchToMode \"Scroll\""],
            },
            ZellijNativeKeybindingBlock {
                mode: "scroll",
                action_lines: &["SwitchToMode \"Normal\""],
            },
        ],
    ),
    zellij_native_action(
        "session_mode_unbind",
        "Unbind default session-mode key",
        &["Ctrl o"],
        "unbind Ctrl o",
        &[
            ZellijNativeKeybindingBlock {
                mode: "shared_except \"session\" \"locked\"",
                action_lines: &[],
            },
            ZellijNativeKeybindingBlock {
                mode: "session",
                action_lines: &[],
            },
        ],
    ),
    zellij_native_action(
        "session_mode",
        "Toggle session mode",
        &["Ctrl Alt o"],
        "SwitchToMode \"Session\" / SwitchToMode \"Normal\"",
        &[
            ZellijNativeKeybindingBlock {
                mode: "shared_except \"session\" \"locked\"",
                action_lines: &["SwitchToMode \"Session\""],
            },
            ZellijNativeKeybindingBlock {
                mode: "session",
                action_lines: &["SwitchToMode \"Normal\""],
            },
        ],
    ),
    zellij_native_action(
        "tmux_mode_unbind",
        "Unbind default tmux-mode key",
        &["Ctrl b"],
        "unbind Ctrl b",
        &[ZellijNativeKeybindingBlock {
            mode: "shared_except \"locked\"",
            action_lines: &[],
        }],
    ),
];

pub const YAZI_ACTIONS: &[YaziActionSpec] = &[
    yazi_action(
        "open_directory_as_workspace_pane",
        "yazi.open_directory_as_workspace_pane",
        "Open the selected directory as a workspace pane",
        "mgr",
        "append_keymap",
        &["<A-p>"],
        "shell '__YAZELIX_RUNTIME_DIR__/libexec/yzx_control zellij open-terminal \"$0\"'",
        "Open directory in new pane",
    ),
    yazi_action(
        "open_zoxide_in_editor",
        "yazi.open_zoxide_in_editor",
        "Retarget the managed editor through the Yazi zoxide picker",
        "mgr",
        "append_keymap",
        &["<A-z>"],
        "plugin zoxide-editor",
        "Zoxide jump -> open in editor",
    ),
];

pub fn all_yazelix_actions() -> impl Iterator<Item = &'static YazelixActionMetadata> {
    ZELLIJ_ACTIONS
        .iter()
        .map(|spec| &spec.action)
        .chain(ZELLIJ_NATIVE_KEYBINDINGS.iter().map(|spec| &spec.action))
        .chain(YAZI_ACTIONS.iter().map(|spec| &spec.action))
}

pub fn zellij_action_by_local_id(local_id: &str) -> Option<&'static ZellijActionSpec> {
    ZELLIJ_ACTIONS
        .iter()
        .find(|spec| spec.action.local_id == local_id)
}

pub fn zellij_native_keybinding_by_local_id(
    local_id: &str,
) -> Option<&'static ZellijNativeKeybindingSpec> {
    ZELLIJ_NATIVE_KEYBINDINGS
        .iter()
        .find(|spec| spec.action.local_id == local_id)
}

pub fn yazi_action_by_local_id(local_id: &str) -> Option<&'static YaziActionSpec> {
    YAZI_ACTIONS
        .iter()
        .find(|spec| spec.action.local_id == local_id)
}
