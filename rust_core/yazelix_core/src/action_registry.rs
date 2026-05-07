pub const PANE_ORCHESTRATOR_PLUGIN_ALIAS: &str = "yazelix_pane_orchestrator";

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
    YaziKeymapCommand,
    EditorCommand,
}

impl YazelixActionBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ZellijPaneOrchestratorMessage => "zellij_pane_orchestrator_message",
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
    pub message_name: &'static str,
    pub payload: Option<&'static str>,
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
    ZellijActionSpec {
        action: YazelixActionMetadata {
            id: scoped_id,
            local_id,
            label,
            owner: YazelixActionOwner::Zellij,
            backend: YazelixActionBackend::ZellijPaneOrchestratorMessage,
            default_keys,
            generated_command,
            disable_policy: YazelixActionDisablePolicy::Optional,
            diagnostics: ZELLIJ_SEMANTIC_KEYBINDING_DIAGNOSTICS,
        },
        mode,
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
    zellij_action(
        "popup",
        "zellij.popup",
        "Toggle the managed popup program",
        "shared",
        "toggle_transient_pane",
        Some("popup"),
        &["Alt t"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_transient_pane\" payload \"popup\" }",
    ),
    zellij_action(
        "menu",
        "zellij.menu",
        "Open the Yazelix command palette popup",
        "shared",
        "toggle_transient_pane",
        Some("menu"),
        &["Alt Shift M"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_transient_pane\" payload \"menu\" }",
    ),
    zellij_action(
        "config",
        "zellij.config",
        "Open the Yazelix config UI popup",
        "shared",
        "toggle_transient_pane",
        Some("config"),
        &["Alt Shift C"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_transient_pane\" payload \"config\" }",
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
        "Toggle focus between the managed editor and sidebar",
        "shared_except \"locked\"",
        "toggle_editor_sidebar_focus",
        None,
        &["Ctrl y"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_editor_sidebar_focus\" }",
    ),
    zellij_action(
        "toggle_sidebar",
        "zellij.toggle_sidebar",
        "Show or hide the managed sidebar",
        "shared_except \"locked\"",
        "toggle_sidebar",
        None,
        &["Alt y"],
        "MessagePlugin yazelix_pane_orchestrator { name \"toggle_sidebar\" }",
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
        .chain(YAZI_ACTIONS.iter().map(|spec| &spec.action))
}

pub fn zellij_action_by_local_id(local_id: &str) -> Option<&'static ZellijActionSpec> {
    ZELLIJ_ACTIONS
        .iter()
        .find(|spec| spec.action.local_id == local_id)
}

pub fn yazi_action_by_local_id(local_id: &str) -> Option<&'static YaziActionSpec> {
    YAZI_ACTIONS
        .iter()
        .find(|spec| spec.action.local_id == local_id)
}
