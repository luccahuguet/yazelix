use crate::bridge::{CoreError, ErrorClass};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxCommandCategory {
    Config,
    Development,
    Help,
    Integration,
    Session,
    System,
    Workspace,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxMenuCategory {
    Config,
    Help,
    Session,
    System,
    Workspace,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum YzxParameterKind {
    Switch,
    Named,
    Positional,
    Rest,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct YzxCommandParameter {
    pub kind: YzxParameterKind,
    pub name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short: Option<&'static str>,
    pub shape: &'static str,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct YzxCommandMetadata {
    pub name: &'static str,
    pub description: &'static str,
    pub category: YzxCommandCategory,
    pub parameters: &'static [YzxCommandParameter],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub menu_category: Option<YzxMenuCategory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_description: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YzxPublicRootRoute {
    Help,
    Version,
    VersionFull,
    RustControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct YzxRustControlFamily {
    root_token: &'static str,
    commands: &'static [YzxCommandMetadata],
}

const VERSION_FLAGS: &[YzxCommandParameter] = &[
    switch("version", Some("V")),
    switch("version-short", Some("v")),
    switch("version-full", None),
];
const ENV_FLAGS: &[YzxCommandParameter] = &[switch("no-shell", Some("n"))];
const RUN_REST: &[YzxCommandParameter] = &[rest("argv")];
const LAUNCH_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    named("config", None, "path", true),
    named("with", None, "string", true),
    switch("home", None),
    named("term", Some("t"), "string", true),
    named("terminal", None, "string", true),
    switch("verbose", None),
];
const RESTART_FLAGS: &[YzxCommandParameter] = &[
    switch("skip", Some("s")),
    named("config", None, "path", true),
    named("with", None, "string", true),
];
const ENTER_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    named("config", None, "path", true),
    named("with", None, "string", true),
    switch("home", None),
    switch("verbose", None),
];
const UPDATE_NIX_FLAGS: &[YzxCommandParameter] = &[switch("yes", None), switch("verbose", None)];
const REVEAL_ARGS: &[YzxCommandParameter] = &[positional("target", "string", false)];
const INSPECT_FLAGS: &[YzxCommandParameter] = &[switch("json", None)];
const STATUS_FLAGS: &[YzxCommandParameter] = &[switch("versions", Some("V")), switch("json", None)];
const DOCTOR_FLAGS: &[YzxCommandParameter] = &[
    switch("verbose", Some("v")),
    switch("fix", Some("f")),
    switch("fix-plan", None),
    switch("json", None),
];
const ONBOARD_FLAGS: &[YzxCommandParameter] = &[switch("force", None), switch("dry-run", None)];
const CONFIG_FLAGS: &[YzxCommandParameter] = &[switch("path", None)];
const CONFIG_SET_ARGS: &[YzxCommandParameter] = &[
    positional("path", "string", false),
    positional("value", "string", false),
];
const CONFIG_UNSET_ARGS: &[YzxCommandParameter] = &[positional("path", "string", false)];
const RESET_FLAGS: &[YzxCommandParameter] = &[switch("yes", None), switch("no-backup", None)];

const POPUP_ARGS: &[YzxCommandParameter] = &[rest("program")];
const SCREEN_ARGS: &[YzxCommandParameter] = &[positional("style", "string", true)];
const DEV_INSPECT_SESSION_FLAGS: &[YzxCommandParameter] = &[switch("json", None)];
const DEV_PROFILE_FLAGS: &[YzxCommandParameter] = &[
    switch("cold", Some("c")),
    switch("desktop", None),
    switch("launch", None),
    switch("clear-cache", None),
    named("terminal", Some("t"), "string", true),
    switch("verbose", None),
];
const HM_PREPARE_FLAGS: &[YzxCommandParameter] = &[switch("apply", None), switch("yes", None)];

const ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx",
    "Show Yazelix help or version information",
    YzxCommandCategory::Help,
    VERSION_FLAGS,
    Some(YzxMenuCategory::Help),
    None,
);

const AGENT_COMMAND: YzxCommandMetadata = metadata(
    "yzx agent",
    "Open the managed right agent pane",
    YzxCommandCategory::Workspace,
    &[],
    Some(YzxMenuCategory::Workspace),
    Some(
        "Launch host-installed Codex when available, otherwise show an actionable right-sidebar shell placeholder.",
    ),
);
const AGENT_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[AGENT_COMMAND];

const ENV_COMMAND: YzxCommandMetadata = metadata(
    "yzx env",
    "Load the Yazelix environment without UI",
    YzxCommandCategory::Session,
    ENV_FLAGS,
    None,
    None,
);
const RUN_COMMAND: YzxCommandMetadata = metadata(
    "yzx run",
    "Run a command in the Yazelix environment and exit",
    YzxCommandCategory::Session,
    RUN_REST,
    None,
    None,
);
const STATUS_COMMAND: YzxCommandMetadata = metadata(
    "yzx status",
    "Canonical inspection command",
    YzxCommandCategory::System,
    STATUS_FLAGS,
    Some(YzxMenuCategory::System),
    None,
);
const UPDATE_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx update",
    "Show supported update owners",
    YzxCommandCategory::System,
    &[],
    Some(YzxMenuCategory::System),
    None,
);
const UPDATE_HOME_MANAGER_COMMAND: YzxCommandMetadata = metadata(
    "yzx update home_manager",
    "Refresh the current Home Manager flake input for Yazelix",
    YzxCommandCategory::System,
    &[],
    Some(YzxMenuCategory::System),
    Some("Refresh the current Home Manager input and print the switch step."),
);
const UPDATE_NIX_COMMAND: YzxCommandMetadata = metadata(
    "yzx update nix",
    "Upgrade Determinate Nix through determinate-nixd",
    YzxCommandCategory::System,
    UPDATE_NIX_FLAGS,
    Some(YzxMenuCategory::System),
    Some("Refresh the runtime lock and print the local install step."),
);
const UPDATE_UPSTREAM_COMMAND: YzxCommandMetadata = metadata(
    "yzx update upstream",
    "Upgrade the active Yazelix package in the default Nix profile",
    YzxCommandCategory::System,
    &[],
    Some(YzxMenuCategory::System),
    Some("Upgrade the active default-profile Yazelix package."),
);

const ENV_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[ENV_COMMAND];
const RUN_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[RUN_COMMAND];
const INSPECT_COMMAND: YzxCommandMetadata = metadata(
    "yzx inspect",
    "Inspect active Yazelix runtime truth",
    YzxCommandCategory::System,
    INSPECT_FLAGS,
    Some(YzxMenuCategory::System),
    Some(
        "Emit a stable runtime/config/install/session report for humans, agents, and diagnostics.",
    ),
);
const INSPECT_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[INSPECT_COMMAND];
const STATUS_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[STATUS_COMMAND];
const DOCTOR_COMMAND: YzxCommandMetadata = metadata(
    "yzx doctor",
    "Run health checks and diagnostics",
    YzxCommandCategory::System,
    DOCTOR_FLAGS,
    Some(YzxMenuCategory::System),
    None,
);
const DOCTOR_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[DOCTOR_COMMAND];
const UPDATE_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[
    UPDATE_ROOT_COMMAND,
    UPDATE_HOME_MANAGER_COMMAND,
    UPDATE_NIX_COMMAND,
    UPDATE_UPSTREAM_COMMAND,
];
const CONFIG_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx config",
    "Show the active Yazelix configuration",
    YzxCommandCategory::Config,
    CONFIG_FLAGS,
    Some(YzxMenuCategory::Config),
    Some("Print the active settings surface or its resolved path."),
);
const CONFIG_UI_COMMAND: YzxCommandMetadata = metadata(
    "yzx config ui",
    "Browse and edit Yazelix settings in a terminal UI",
    YzxCommandCategory::Config,
    &[],
    Some(YzxMenuCategory::Config),
    Some("Inspect and edit explicit, defaulted, stale, and advanced config surfaces."),
);
const CONFIG_SET_COMMAND: YzxCommandMetadata = metadata(
    "yzx config set",
    "Set a supported config value",
    YzxCommandCategory::Config,
    CONFIG_SET_ARGS,
    Some(YzxMenuCategory::Config),
    Some("Patch a supported config path with a JSON literal while preserving comments."),
);
const CONFIG_UNSET_COMMAND: YzxCommandMetadata = metadata(
    "yzx config unset",
    "Remove an explicit config value",
    YzxCommandCategory::Config,
    CONFIG_UNSET_ARGS,
    Some(YzxMenuCategory::Config),
    Some("Remove an explicit value so Yazelix falls back to the shipped default."),
);
const CONFIG_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[
    CONFIG_ROOT_COMMAND,
    CONFIG_UI_COMMAND,
    CONFIG_SET_COMMAND,
    CONFIG_UNSET_COMMAND,
];
const RESET_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx reset",
    "Show Yazelix reset targets",
    YzxCommandCategory::Config,
    &[],
    Some(YzxMenuCategory::Config),
    Some("Show reset targets for managed Yazelix config surfaces."),
);
const RESET_CONFIG_COMMAND: YzxCommandMetadata = metadata(
    "yzx reset config",
    "Replace the main Yazelix config with a fresh shipped template",
    YzxCommandCategory::Config,
    RESET_FLAGS,
    Some(YzxMenuCategory::Config),
    Some("Reset settings.jsonc back to the shipped default."),
);
const RESET_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[RESET_ROOT_COMMAND, RESET_CONFIG_COMMAND];
const CURSORS_COMMAND: YzxCommandMetadata = metadata(
    "yzx cursors",
    "Inspect Yazelix cursor presets and resolved colors",
    YzxCommandCategory::Config,
    &[],
    Some(YzxMenuCategory::Config),
    Some("Show the active settings.jsonc cursor registry, effects, and resolved preset colors."),
);
const CURSORS_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[CURSORS_COMMAND];
const HOME_MANAGER_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx home_manager",
    "Show Yazelix Home Manager takeover helpers",
    YzxCommandCategory::Integration,
    &[],
    Some(YzxMenuCategory::System),
    Some("Home Manager takeover helpers for Yazelix-owned paths."),
);
const HOME_MANAGER_PREPARE_COMMAND: YzxCommandMetadata = metadata(
    "yzx home_manager prepare",
    "Preview or archive manual-install artifacts before Home Manager takeover",
    YzxCommandCategory::Integration,
    HM_PREPARE_FLAGS,
    Some(YzxMenuCategory::System),
    Some("Preview or archive manual-install artifacts before Home Manager takeover."),
);
const HOME_MANAGER_FAMILY_COMMANDS: &[YzxCommandMetadata] =
    &[HOME_MANAGER_ROOT_COMMAND, HOME_MANAGER_PREPARE_COMMAND];
const SPONSOR_COMMAND: YzxCommandMetadata = metadata(
    "yzx sponsor",
    "Open the Yazelix sponsor page or print its URL",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    Some("Show the sponsorship links and support message."),
);
const SPONSOR_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[SPONSOR_COMMAND];
const WHY_COMMAND: YzxCommandMetadata = metadata(
    "yzx why",
    "Elevator pitch: Why Yazelix",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const WHY_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[WHY_COMMAND];
const SCREEN_COMMAND: YzxCommandMetadata = metadata(
    "yzx screen",
    "Show an animated Yazelix full-terminal screen",
    YzxCommandCategory::Workspace,
    SCREEN_ARGS,
    Some(YzxMenuCategory::Workspace),
    Some("Preview the animated welcome screen directly in the current terminal."),
);
const SCREEN_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[SCREEN_COMMAND];
const TUTOR_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor",
    "Show the Yazelix guided overview",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_BEGIN_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor begin",
    "Start the first Yazelix tutor lesson",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_LIST_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor list",
    "List Yazelix tutor lessons",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_WORKSPACE_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor workspace",
    "Practice workspace roots and managed panes",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_DISCOVERY_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor discovery",
    "Practice command and key discovery surfaces",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_TOOL_TUTORS_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor tool_tutors",
    "Find the upstream Helix and Nushell tutors",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_HELIX_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor helix",
    "Alias for yzx tutor hx",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_HX_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor hx",
    "Launch Helix's built-in tutorial",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_NU_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor nu",
    "Launch Nushell's built-in tutorial in a fresh Nushell process",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_NUSHELL_COMMAND: YzxCommandMetadata = metadata(
    "yzx tutor nushell",
    "Alias for yzx tutor nu",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const TUTOR_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[
    TUTOR_ROOT_COMMAND,
    TUTOR_BEGIN_COMMAND,
    TUTOR_LIST_COMMAND,
    TUTOR_WORKSPACE_COMMAND,
    TUTOR_DISCOVERY_COMMAND,
    TUTOR_TOOL_TUTORS_COMMAND,
    TUTOR_HELIX_COMMAND,
    TUTOR_HX_COMMAND,
    TUTOR_NU_COMMAND,
    TUTOR_NUSHELL_COMMAND,
];
const WHATS_NEW_COMMAND: YzxCommandMetadata = metadata(
    "yzx whats_new",
    "Show Yazelix changes since the installed runtime",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    Some("Show bundled release notes newer than the installed runtime."),
);
const WHATS_NEW_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[WHATS_NEW_COMMAND];
const IMPORT_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx import",
    "Import native config files into Yazelix-managed override paths",
    YzxCommandCategory::Config,
    &[],
    Some(YzxMenuCategory::Config),
    None,
);
const IMPORT_HELIX_COMMAND: YzxCommandMetadata = metadata(
    "yzx import helix",
    "Import the native Helix config into Yazelix-managed overrides",
    YzxCommandCategory::Config,
    &[switch("force", None)],
    Some(YzxMenuCategory::Config),
    None,
);
const IMPORT_YAZI_COMMAND: YzxCommandMetadata = metadata(
    "yzx import yazi",
    "Import native Yazi config files and plugins into Yazelix-managed override paths",
    YzxCommandCategory::Config,
    &[switch("force", None)],
    Some(YzxMenuCategory::Config),
    None,
);
const IMPORT_ZELLIJ_COMMAND: YzxCommandMetadata = metadata(
    "yzx import zellij",
    "Import native Zellij config without keybinds blocks into zellij.kdl",
    YzxCommandCategory::Config,
    &[switch("force", None)],
    Some(YzxMenuCategory::Config),
    None,
);
const IMPORT_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[
    IMPORT_ROOT_COMMAND,
    IMPORT_HELIX_COMMAND,
    IMPORT_YAZI_COMMAND,
    IMPORT_ZELLIJ_COMMAND,
];
const EDIT_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx edit",
    "Open a Yazelix-managed config surface in the configured editor",
    YzxCommandCategory::Config,
    &[rest("query"), switch("print", None)],
    Some(YzxMenuCategory::Config),
    Some("Open the managed Yazelix config directory."),
);
const EDIT_CONFIG_COMMAND: YzxCommandMetadata = metadata(
    "yzx edit config",
    "Open the main Yazelix config in the configured editor",
    YzxCommandCategory::Config,
    &[switch("print", None)],
    Some(YzxMenuCategory::Config),
    Some("Open the active Yazelix config file."),
);
const EDIT_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[EDIT_ROOT_COMMAND, EDIT_CONFIG_COMMAND];

const ONBOARD_COMMAND: YzxCommandMetadata = metadata(
    "yzx onboard",
    "Generate a focused first-run Yazelix config",
    YzxCommandCategory::Config,
    ONBOARD_FLAGS,
    Some(YzxMenuCategory::Config),
    Some(
        "Interactive setup for core editor, shell, terminal, sidebar, session, and status-bar choices.",
    ),
);
const ONBOARD_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[ONBOARD_COMMAND];

const RUST_CONTROL_FAMILIES: &[YzxRustControlFamily] = &[
    rust_control_family("agent", AGENT_FAMILY_COMMANDS),
    rust_control_family("config", CONFIG_FAMILY_COMMANDS),
    rust_control_family("cursors", CURSORS_FAMILY_COMMANDS),
    rust_control_family("desktop", DESKTOP_FAMILY_COMMANDS),
    rust_control_family("dev", DEV_RUST_CONTROL_COMMANDS),
    rust_control_family("edit", EDIT_FAMILY_COMMANDS),
    rust_control_family("enter", ENTER_FAMILY_COMMANDS),
    rust_control_family("env", ENV_FAMILY_COMMANDS),
    rust_control_family("import", IMPORT_FAMILY_COMMANDS),
    rust_control_family("inspect", INSPECT_FAMILY_COMMANDS),
    rust_control_family("launch", LAUNCH_FAMILY_COMMANDS),
    rust_control_family("menu", MENU_FAMILY_COMMANDS),
    rust_control_family("onboard", ONBOARD_FAMILY_COMMANDS),
    rust_control_family("run", RUN_FAMILY_COMMANDS),
    rust_control_family("popup", POPUP_FAMILY_COMMANDS),
    rust_control_family("popup_run", &[]),
    rust_control_family("sidebar", SIDEBAR_FAMILY_COMMANDS),
    rust_control_family("reveal", REVEAL_FAMILY_COMMANDS),
    rust_control_family("reset", RESET_FAMILY_COMMANDS),
    rust_control_family("restart", RESTART_FAMILY_COMMANDS),
    rust_control_family("screen", SCREEN_FAMILY_COMMANDS),
    rust_control_family("status", STATUS_FAMILY_COMMANDS),
    rust_control_family("tutor", TUTOR_FAMILY_COMMANDS),
    rust_control_family("doctor", DOCTOR_FAMILY_COMMANDS),
    rust_control_family("home_manager", HOME_MANAGER_FAMILY_COMMANDS),
    rust_control_family("keys", KEYS_FAMILY_COMMANDS),
    rust_control_family("sponsor", SPONSOR_FAMILY_COMMANDS),
    rust_control_family("update", UPDATE_FAMILY_COMMANDS),
    rust_control_family("whats_new", WHATS_NEW_FAMILY_COMMANDS),
    rust_control_family("why", WHY_FAMILY_COMMANDS),
];

const DESKTOP_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx desktop",
    "Desktop integration commands",
    YzxCommandCategory::Integration,
    &[],
    Some(YzxMenuCategory::System),
    None,
);
const DESKTOP_INSTALL_COMMAND: YzxCommandMetadata = metadata(
    "yzx desktop install",
    "Install the user-local Yazelix desktop entry and icons",
    YzxCommandCategory::Integration,
    &[],
    Some(YzxMenuCategory::System),
    Some("Install or refresh the Yazelix desktop entry and icon assets."),
);
const DESKTOP_LAUNCH_COMMAND: YzxCommandMetadata = metadata(
    "yzx desktop launch",
    "Launch Yazelix from the desktop entry fast path",
    YzxCommandCategory::Integration,
    &[],
    Some(YzxMenuCategory::System),
    Some("Launch Yazelix through the desktop-entry path."),
);
const DESKTOP_UNINSTALL_COMMAND: YzxCommandMetadata = metadata(
    "yzx desktop uninstall",
    "Remove the user-local Yazelix desktop entry and icons",
    YzxCommandCategory::Integration,
    &[],
    Some(YzxMenuCategory::System),
    Some("Remove Yazelix-managed desktop entry and icon assets."),
);
const DESKTOP_MACOS_PREVIEW_INSTALL_COMMAND: YzxCommandMetadata = metadata(
    "yzx desktop macos_preview install",
    "Install the experimental macOS launcher preview app bundle",
    YzxCommandCategory::Integration,
    &[],
    Some(YzxMenuCategory::System),
    Some("Install the package-first experimental macOS launcher preview."),
);
const DESKTOP_MACOS_PREVIEW_UNINSTALL_COMMAND: YzxCommandMetadata = metadata(
    "yzx desktop macos_preview uninstall",
    "Remove the experimental macOS launcher preview app bundle",
    YzxCommandCategory::Integration,
    &[],
    Some(YzxMenuCategory::System),
    Some("Remove the Yazelix-managed experimental macOS launcher preview."),
);
const DESKTOP_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[
    DESKTOP_ROOT_COMMAND,
    DESKTOP_INSTALL_COMMAND,
    DESKTOP_LAUNCH_COMMAND,
    DESKTOP_UNINSTALL_COMMAND,
    DESKTOP_MACOS_PREVIEW_INSTALL_COMMAND,
    DESKTOP_MACOS_PREVIEW_UNINSTALL_COMMAND,
];

const DEV_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx dev",
    "Runtime diagnostics",
    YzxCommandCategory::Development,
    &[],
    None,
    None,
);
const DEV_INSPECT_SESSION_COMMAND: YzxCommandMetadata = metadata(
    "yzx dev inspect_session",
    "Inspect the current Yazelix tab session state",
    YzxCommandCategory::Development,
    DEV_INSPECT_SESSION_FLAGS,
    None,
    None,
);
const DEV_PROFILE_COMMAND: YzxCommandMetadata = metadata(
    "yzx dev profile",
    "Profile launch sequence and identify bottlenecks",
    YzxCommandCategory::Development,
    DEV_PROFILE_FLAGS,
    None,
    None,
);
const DEV_RUST_CONTROL_COMMANDS: &[YzxCommandMetadata] = &[
    DEV_ROOT_COMMAND,
    DEV_INSPECT_SESSION_COMMAND,
    DEV_PROFILE_COMMAND,
];

const ENTER_COMMAND: YzxCommandMetadata = metadata(
    "yzx enter",
    "Start Yazelix in the current terminal",
    YzxCommandCategory::Session,
    ENTER_FLAGS,
    Some(YzxMenuCategory::Session),
    None,
);
const ENTER_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[ENTER_COMMAND];

const KEYS_ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx keys",
    "Show Yazelix-owned keybindings and remaps",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const KEYS_HELIX_COMMAND: YzxCommandMetadata = metadata(
    "yzx keys helix",
    "Alias for yzx keys hx",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const KEYS_HX_COMMAND: YzxCommandMetadata = metadata(
    "yzx keys hx",
    "Explain how to discover Helix keybindings and commands",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const KEYS_NU_COMMAND: YzxCommandMetadata = metadata(
    "yzx keys nu",
    "Show a small curated subset of useful Nushell keybindings",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const KEYS_NUSHELL_COMMAND: YzxCommandMetadata = metadata(
    "yzx keys nushell",
    "Alias for yzx keys nu",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const KEYS_YAZI_COMMAND: YzxCommandMetadata = metadata(
    "yzx keys yazi",
    "Explain how to view Yazi's built-in keybindings",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const KEYS_YZX_COMMAND: YzxCommandMetadata = metadata(
    "yzx keys yzx",
    "Alias for the default Yazelix keybinding view",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    None,
);
const KEYS_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[
    KEYS_ROOT_COMMAND,
    KEYS_HELIX_COMMAND,
    KEYS_HX_COMMAND,
    KEYS_NU_COMMAND,
    KEYS_NUSHELL_COMMAND,
    KEYS_YAZI_COMMAND,
    KEYS_YZX_COMMAND,
];

const LAUNCH_COMMAND: YzxCommandMetadata = metadata(
    "yzx launch",
    "Launch Yazelix",
    YzxCommandCategory::Session,
    LAUNCH_FLAGS,
    Some(YzxMenuCategory::Session),
    None,
);
const LAUNCH_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[LAUNCH_COMMAND];

const MENU_COMMAND: YzxCommandMetadata = metadata(
    "yzx menu",
    "Interactive command palette for Yazelix",
    YzxCommandCategory::Help,
    &[],
    None,
    None,
);
const MENU_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[MENU_COMMAND];

const POPUP_COMMAND: YzxCommandMetadata = metadata(
    "yzx popup",
    "Open an explicit command in a transient Yazelix popup pane",
    YzxCommandCategory::Workspace,
    POPUP_ARGS,
    Some(YzxMenuCategory::Workspace),
    Some("Open a floating terminal tool pane, for example `yzx popup lazygit`."),
);
const POPUP_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[POPUP_COMMAND];

const SIDEBAR_REFRESH_COMMAND: YzxCommandMetadata = metadata(
    "yzx sidebar refresh",
    "Refresh the managed Yazi sidebar",
    YzxCommandCategory::Workspace,
    &[],
    Some(YzxMenuCategory::Workspace),
    Some("Refresh the managed Yazi sidebar file tree and status widgets."),
);
const SIDEBAR_YAZI_COMMAND: YzxCommandMetadata = metadata(
    "yzx sidebar yazi",
    "Launch the managed Yazi sidebar",
    YzxCommandCategory::Workspace,
    &[],
    Some(YzxMenuCategory::Workspace),
    Some("Launch the managed Yazi file-tree sidebar."),
);
const SIDEBAR_FAMILY_COMMANDS: &[YzxCommandMetadata] =
    &[SIDEBAR_YAZI_COMMAND, SIDEBAR_REFRESH_COMMAND];

const RESTART_COMMAND: YzxCommandMetadata = metadata(
    "yzx restart",
    "Restart Yazelix",
    YzxCommandCategory::Session,
    RESTART_FLAGS,
    Some(YzxMenuCategory::Session),
    Some("Restart Yazelix. Use `--skip` or `-s` to skip the welcome screen once."),
);
const RESTART_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[RESTART_COMMAND];

const REVEAL_COMMAND: YzxCommandMetadata = metadata(
    "yzx reveal",
    "Reveal a file or directory in the managed Yazi sidebar",
    YzxCommandCategory::Workspace,
    REVEAL_ARGS,
    Some(YzxMenuCategory::Workspace),
    Some("Reveal a path in the managed Yazi sidebar."),
);
const REVEAL_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[REVEAL_COMMAND];

pub fn yzx_command_metadata() -> Vec<YzxCommandMetadata> {
    let mut commands = vec![ROOT_COMMAND];
    for family in RUST_CONTROL_FAMILIES {
        commands.extend(family.commands.iter().copied());
    }
    commands.sort_by(|left, right| left.name.cmp(right.name));
    commands
}

pub fn classify_yzx_root_route(argv: &[String]) -> Result<YzxPublicRootRoute, CoreError> {
    let Some(first) = argv.first().map(|value| value.as_str()) else {
        return Ok(YzxPublicRootRoute::Help);
    };

    if matches!(first, "help" | "-h" | "--help") {
        return Ok(YzxPublicRootRoute::Help);
    }

    if matches!(first, "-V" | "--version" | "-v" | "--version-short") {
        return Ok(YzxPublicRootRoute::Version);
    }

    if first == "--version-full" {
        return Ok(YzxPublicRootRoute::VersionFull);
    }

    if RUST_CONTROL_FAMILIES
        .iter()
        .any(|family| family.root_token == first)
    {
        return Ok(YzxPublicRootRoute::RustControl);
    }

    Err(CoreError::classified(
        ErrorClass::Usage,
        "unknown_command",
        format!("Unknown yzx command: {first}"),
        "Run `yzx --help` to see available commands.",
        json!({ "command": first }),
    ))
}

const fn metadata(
    name: &'static str,
    description: &'static str,
    category: YzxCommandCategory,
    parameters: &'static [YzxCommandParameter],
    menu_category: Option<YzxMenuCategory>,
    extra_description: Option<&'static str>,
) -> YzxCommandMetadata {
    YzxCommandMetadata {
        name,
        description,
        category,
        parameters,
        menu_category,
        extra_description,
    }
}

const fn switch(name: &'static str, short: Option<&'static str>) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Switch,
        name,
        short,
        shape: "string",
        optional: true,
    }
}

const fn named(
    name: &'static str,
    short: Option<&'static str>,
    shape: &'static str,
    optional: bool,
) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Named,
        name,
        short,
        shape,
        optional,
    }
}

const fn positional(
    name: &'static str,
    shape: &'static str,
    optional: bool,
) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Positional,
        name,
        short: None,
        shape,
        optional,
    }
}

const fn rest(name: &'static str) -> YzxCommandParameter {
    YzxCommandParameter {
        kind: YzxParameterKind::Rest,
        name,
        short: None,
        shape: "string",
        optional: true,
    }
}

const fn rust_control_family(
    root_token: &'static str,
    commands: &'static [YzxCommandMetadata],
) -> YzxRustControlFamily {
    YzxRustControlFamily {
        root_token,
        commands,
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the public Rust root keeps the already migrated control-plane family on the Rust-owned path.
    #[test]
    fn classifies_rust_owned_control_family_at_root() {
        assert_eq!(
            classify_yzx_root_route(&["env".into(), "--no-shell".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["config".into(), "--path".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["reset".into(), "config".into(), "--yes".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["cursors".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["run".into(), "rg".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["agent".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["reveal".into(), "/tmp/file".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["doctor".into(), "--json".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["update".into(), "nix".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["home_manager".into(), "prepare".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["keys".into(), "helix".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["keys".into(), "yazi".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["sponsor".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["why".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
    }

    // Defends: the shared root classifier preserves no-arg help, help flags, and all supported version flags.
    #[test]
    fn classifies_root_help_and_version_behaviors() {
        assert_eq!(
            classify_yzx_root_route(&[]).unwrap(),
            YzxPublicRootRoute::Help
        );
        assert_eq!(
            classify_yzx_root_route(&[String::from("help")]).unwrap(),
            YzxPublicRootRoute::Help
        );
        assert_eq!(
            classify_yzx_root_route(&[String::from("-h")]).unwrap(),
            YzxPublicRootRoute::Help
        );
        assert_eq!(
            classify_yzx_root_route(&[String::from("--help")]).unwrap(),
            YzxPublicRootRoute::Help
        );
        for flag in ["-V", "--version", "-v", "--version-short"] {
            assert_eq!(
                classify_yzx_root_route(&[flag.to_string()]).unwrap(),
                YzxPublicRootRoute::Version
            );
        }
        assert_eq!(
            classify_yzx_root_route(&[String::from("--version-full")]).unwrap(),
            YzxPublicRootRoute::VersionFull
        );
    }

    // Defends: the Rust root rejects unknown top-level commands instead of reviving the old generic Nu root fallback.
    #[test]
    fn rejects_unknown_top_level_command() {
        let err = classify_yzx_root_route(&["not-a-command".into()]).unwrap_err();
        assert!(matches!(err.class(), ErrorClass::Usage));
        assert_eq!(err.code(), "unknown_command");
    }

    // Defends: grouped Rust-owned families route through yzx_control instead of reviving direct Nu module ownership.
    #[test]
    fn routes_grouped_rust_family_to_control_plane() {
        let argv = [String::from("desktop"), String::from("launch")];
        let route = classify_yzx_root_route(&argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));

        let menu_argv = [String::from("menu")];
        let route = classify_yzx_root_route(&menu_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));

        let popup_run_argv = [String::from("popup_run"), String::from("--help")];
        let route = classify_yzx_root_route(&popup_run_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));
    }

    // Regression: grouped Rust-owned families route through the Rust control plane even for help aliases.
    #[test]
    fn routes_grouped_help_aliases_to_control_plane() {
        let dev_argv = [
            String::from("dev"),
            String::from("help"),
            String::from("ignored"),
        ];
        let route = classify_yzx_root_route(&dev_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));

        let import_argv = [String::from("import"), String::from("--help")];
        let route = classify_yzx_root_route(&import_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));
    }

    // Regression: the direct route planner must preserve alias leaves and the family-specific missing-subcommand contract.
    #[test]
    fn preserves_alias_and_missing_subcommand_contracts() {
        let edit_argv = [String::from("edit"), String::from("config")];
        let route = classify_yzx_root_route(&edit_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));

        let tutor_argv = [String::from("tutor"), String::from("nushell")];
        let route = classify_yzx_root_route(&tutor_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));

        let screen_argv = [String::from("screen"), String::from("logo")];
        let route = classify_yzx_root_route(&screen_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));

        let desktop_argv = [String::from("desktop")];
        let desktop_route = classify_yzx_root_route(&desktop_argv).unwrap();
        assert!(matches!(desktop_route, YzxPublicRootRoute::RustControl));

        let dev_argv = [String::from("dev"), String::from("not-a-subcommand")];
        let route = classify_yzx_root_route(&dev_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));
    }

    // Regression: menu visibility and menu categories come from the shared Rust command surface instead of a second Nushell-owned map.
    #[test]
    fn shared_surface_carries_menu_visibility_and_category() {
        let commands = yzx_command_metadata();
        let menu = commands
            .iter()
            .find(|command| command.name == "yzx menu")
            .unwrap();
        let update = commands
            .iter()
            .find(|command| command.name == "yzx update")
            .unwrap();
        let env = commands
            .iter()
            .find(|command| command.name == "yzx env")
            .unwrap();

        assert_eq!(menu.menu_category, None);
        assert_eq!(update.menu_category, Some(YzxMenuCategory::System));
        assert_eq!(env.menu_category, None);
        assert_eq!(
            update.extra_description, None,
            "root update keeps the base description while subcommands can override palette copy"
        );
    }
}
