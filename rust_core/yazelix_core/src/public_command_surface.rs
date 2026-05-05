use crate::bridge::{CoreError, ErrorClass};
use serde::Serialize;
use serde_json::json;

const YZX_DEV_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "dev.nu"];
const YZX_MENU_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "yzx", "menu.nu"];

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
pub enum YzxPublicRootRoute<'a> {
    Help,
    Version,
    RustControl,
    InternalNu(YzxInternalNuRoutePlan<'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct YzxInternalNuRoutePlan<'a> {
    pub module_relative_path: &'static [&'static str],
    pub command_name: &'static str,
    pub tail: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum YzxUnknownSubcommandBehavior {
    RouteRoot,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct YzxCommandLeaf {
    metadata: YzxCommandMetadata,
    tokens_after_root: &'static [&'static str],
    module_relative_path: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct YzxRustControlFamily {
    root_token: &'static str,
    commands: &'static [YzxCommandMetadata],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct YzxInternalNuFamily {
    root_token: &'static str,
    commands: &'static [YzxCommandLeaf],
    root_command_index: Option<usize>,
    help_token_routes_to_root_empty_tail: bool,
    help_flags_route_to_root_with_tail: bool,
    unknown_subcommand_behavior: YzxUnknownSubcommandBehavior,
    required_subcommands: &'static [&'static str],
}

const VERSION_FLAGS: &[YzxCommandParameter] = &[
    switch("version", Some("V")),
    switch("version-short", Some("v")),
];
const ENV_FLAGS: &[YzxCommandParameter] = &[switch("no-shell", Some("n"))];
const RUN_REST: &[YzxCommandParameter] = &[rest("argv")];
const LAUNCH_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    named("config", None, "path", true),
    named("with", None, "key=value", true),
    switch("home", None),
    named("terminal", Some("t"), "string", true),
    switch("verbose", None),
];
const RESTART_FLAGS: &[YzxCommandParameter] = &[
    switch("skip", Some("s")),
    named("config", None, "path", true),
    named("with", None, "key=value", true),
];
const ENTER_FLAGS: &[YzxCommandParameter] = &[
    named("path", Some("p"), "string", true),
    named("config", None, "path", true),
    named("with", None, "key=value", true),
    switch("home", None),
    switch("verbose", None),
];
const UPDATE_NIX_FLAGS: &[YzxCommandParameter] = &[switch("yes", None), switch("verbose", None)];
const WARP_ARGS: &[YzxCommandParameter] = &[
    positional("target", "string", true),
    switch("kill", Some("k")),
];
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
const RESET_FLAGS: &[YzxCommandParameter] = &[switch("yes", None), switch("no-backup", None)];

const POPUP_ARGS: &[YzxCommandParameter] = &[rest("program")];
const SCREEN_ARGS: &[YzxCommandParameter] = &[positional("style", "string", true)];
const DEV_UPDATE_FLAGS: &[YzxCommandParameter] = &[
    switch("yes", None),
    switch("no-canary", None),
    named("activate", None, "string", true),
    named("home-manager-dir", None, "string", true),
    named("home-manager-input", None, "string", true),
    named("home-manager-attr", None, "string", true),
    switch("canary-only", None),
    named("canaries", None, "string", true),
];
const DEV_BUMP_ARGS: &[YzxCommandParameter] = &[positional("version", "string", false)];
const DEV_SYNC_FLAGS: &[YzxCommandParameter] = &[switch("dry-run", None)];
const DEV_BUILD_FLAGS: &[YzxCommandParameter] = &[switch("sync", None)];
const DEV_INSPECT_SESSION_FLAGS: &[YzxCommandParameter] = &[switch("json", None)];
const DEV_RUST_TARGET_ARG: &[YzxCommandParameter] = &[positional("target", "string", true)];
const DEV_RUST_FMT_ARGS: &[YzxCommandParameter] =
    &[positional("target", "string", true), switch("check", None)];
const DEV_RUST_TEST_ARGS: &[YzxCommandParameter] = &[rest("args")];
const DEV_TEST_FLAGS: &[YzxCommandParameter] = &[
    switch("verbose", Some("v")),
    switch("new-window", Some("n")),
    switch("lint-only", None),
    switch("profile", None),
    switch("sweep", None),
    switch("visual", None),
    switch("all", Some("a")),
    named("delay", None, "int", true),
];
const DEV_PROFILE_FLAGS: &[YzxCommandParameter] = &[
    switch("cold", Some("c")),
    switch("desktop", None),
    switch("launch", None),
    switch("clear-cache", None),
    named("terminal", Some("t"), "string", true),
    switch("verbose", None),
];
const DEV_LINT_ARGS: &[YzxCommandParameter] =
    &[named("format", Some("f"), "string", true), rest("paths")];
const HM_PREPARE_FLAGS: &[YzxCommandParameter] = &[switch("apply", None), switch("yes", None)];

const ROOT_COMMAND: YzxCommandMetadata = metadata(
    "yzx",
    "Show Yazelix help or version information",
    YzxCommandCategory::Help,
    VERSION_FLAGS,
    Some(YzxMenuCategory::Help),
    None,
);

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
    Some("Print the active config TOML or its resolved path."),
);
const CONFIG_UI_COMMAND: YzxCommandMetadata = metadata(
    "yzx config ui",
    "Browse Yazelix settings in a read-only terminal UI",
    YzxCommandCategory::Config,
    &[],
    Some(YzxMenuCategory::Config),
    Some("Inspect explicit, defaulted, stale, and advanced config surfaces without writing files."),
);
const CONFIG_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[CONFIG_ROOT_COMMAND, CONFIG_UI_COMMAND];
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
    "Inspect Ghostty cursor presets and resolved colors",
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
    "Show the current Yazelix upgrade summary",
    YzxCommandCategory::Help,
    &[],
    Some(YzxMenuCategory::Help),
    Some("Show the latest release notes."),
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
    "Import native Yazi config files into Yazelix-managed override paths",
    YzxCommandCategory::Config,
    &[switch("force", None)],
    Some(YzxMenuCategory::Config),
    None,
);
const IMPORT_ZELLIJ_COMMAND: YzxCommandMetadata = metadata(
    "yzx import zellij",
    "Import the native Zellij config into Yazelix-managed overrides",
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
    rust_control_family("config", CONFIG_FAMILY_COMMANDS),
    rust_control_family("cursors", CURSORS_FAMILY_COMMANDS),
    rust_control_family("desktop", DESKTOP_FAMILY_COMMANDS),
    rust_control_family("edit", EDIT_FAMILY_COMMANDS),
    rust_control_family("enter", ENTER_FAMILY_COMMANDS),
    rust_control_family("env", ENV_FAMILY_COMMANDS),
    rust_control_family("import", IMPORT_FAMILY_COMMANDS),
    rust_control_family("inspect", INSPECT_FAMILY_COMMANDS),
    rust_control_family("launch", LAUNCH_FAMILY_COMMANDS),
    rust_control_family("onboard", ONBOARD_FAMILY_COMMANDS),
    rust_control_family("run", RUN_FAMILY_COMMANDS),
    rust_control_family("popup", POPUP_FAMILY_COMMANDS),
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
    rust_control_family("warp", WARP_FAMILY_COMMANDS),
    rust_control_family("whats_new", WHATS_NEW_FAMILY_COMMANDS),
    rust_control_family("why", WHY_FAMILY_COMMANDS),
];

const WARP_COMMAND: YzxCommandMetadata = metadata(
    "yzx warp",
    "Open a project workspace in a new Yazelix tab",
    YzxCommandCategory::Workspace,
    WARP_ARGS,
    None,
    Some("Resolve a directory or zoxide query, then open it as a fresh workspace tab."),
);
const WARP_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[WARP_COMMAND];

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

const DEV_ROOT_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev",
        "Development and maintainer commands",
        YzxCommandCategory::Development,
        &[],
        None,
        None,
    ),
    &[],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_BUILD_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev build_pane_orchestrator",
        "Build the Zellij pane-orchestrator wasm",
        YzxCommandCategory::Development,
        DEV_BUILD_FLAGS,
        None,
        None,
    ),
    &["build_pane_orchestrator"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_BUMP_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev bump",
        "Bump the tracked Yazelix version and create release metadata",
        YzxCommandCategory::Development,
        DEV_BUMP_ARGS,
        None,
        None,
    ),
    &["bump"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_INSPECT_SESSION_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev inspect_session",
        "Inspect the current Yazelix tab session state",
        YzxCommandCategory::Development,
        DEV_INSPECT_SESSION_FLAGS,
        None,
        None,
    ),
    &["inspect_session"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_LINT_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev lint_nu",
        "Lint Nushell scripts with repo-tuned nu-lint config",
        YzxCommandCategory::Development,
        DEV_LINT_ARGS,
        None,
        None,
    ),
    &["lint_nu"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_PROFILE_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev profile",
        "Profile launch sequence and identify bottlenecks",
        YzxCommandCategory::Development,
        DEV_PROFILE_FLAGS,
        None,
        None,
    ),
    &["profile"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_RUST_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev rust",
        "Show fast Rust inner-loop commands",
        YzxCommandCategory::Development,
        &[],
        None,
        None,
    ),
    &["rust"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_RUST_FMT_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev rust fmt",
        "Format Rust code without entering nix develop",
        YzxCommandCategory::Development,
        DEV_RUST_FMT_ARGS,
        None,
        None,
    ),
    &["rust", "fmt"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_RUST_CHECK_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev rust check",
        "Run fast cargo check without entering nix develop",
        YzxCommandCategory::Development,
        DEV_RUST_TARGET_ARG,
        None,
        None,
    ),
    &["rust", "check"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_RUST_TEST_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev rust test",
        "Run fast cargo tests without entering nix develop",
        YzxCommandCategory::Development,
        DEV_RUST_TEST_ARGS,
        None,
        None,
    ),
    &["rust", "test"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_SYNC_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev sync_issues",
        "Sync GitHub issue lifecycle into Beads locally",
        YzxCommandCategory::Development,
        DEV_SYNC_FLAGS,
        None,
        None,
    ),
    &["sync_issues"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_TEST_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev test",
        "Run Yazelix test suite",
        YzxCommandCategory::Development,
        DEV_TEST_FLAGS,
        None,
        None,
    ),
    &["test"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_UPDATE_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx dev update",
        "Refresh maintainer flake inputs and run update canaries",
        YzxCommandCategory::Development,
        DEV_UPDATE_FLAGS,
        None,
        None,
    ),
    &["update"],
    YZX_DEV_RELATIVE_PATH,
);
const DEV_COMMANDS: &[YzxCommandLeaf] = &[
    DEV_ROOT_COMMAND,
    DEV_BUILD_COMMAND,
    DEV_BUMP_COMMAND,
    DEV_INSPECT_SESSION_COMMAND,
    DEV_LINT_COMMAND,
    DEV_PROFILE_COMMAND,
    DEV_RUST_COMMAND,
    DEV_RUST_FMT_COMMAND,
    DEV_RUST_CHECK_COMMAND,
    DEV_RUST_TEST_COMMAND,
    DEV_SYNC_COMMAND,
    DEV_TEST_COMMAND,
    DEV_UPDATE_COMMAND,
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

const MENU_COMMAND: YzxCommandLeaf = leaf(
    metadata(
        "yzx menu",
        "Interactive command palette for Yazelix",
        YzxCommandCategory::Help,
        &[],
        None,
        None,
    ),
    &[],
    YZX_MENU_RELATIVE_PATH,
);
const MENU_COMMANDS: &[YzxCommandLeaf] = &[MENU_COMMAND];

const POPUP_COMMAND: YzxCommandMetadata = metadata(
    "yzx popup",
    "Open or toggle the configured Yazelix popup program in Zellij",
    YzxCommandCategory::Workspace,
    POPUP_ARGS,
    Some(YzxMenuCategory::Workspace),
    Some("Open a floating terminal tool pane, for example `yzx popup lazygit`."),
);
const POPUP_FAMILY_COMMANDS: &[YzxCommandMetadata] = &[POPUP_COMMAND];

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

const INTERNAL_NU_FAMILIES: &[YzxInternalNuFamily] = &[
    internal_family(
        "dev",
        DEV_COMMANDS,
        Some(0),
        true,
        true,
        YzxUnknownSubcommandBehavior::Error,
        &[],
    ),
    internal_family(
        "menu",
        MENU_COMMANDS,
        Some(0),
        false,
        false,
        YzxUnknownSubcommandBehavior::RouteRoot,
        &[],
    ),
];

pub fn yzx_command_metadata() -> Vec<YzxCommandMetadata> {
    let mut commands = vec![ROOT_COMMAND];
    for family in RUST_CONTROL_FAMILIES {
        commands.extend(family.commands.iter().copied());
    }
    for family in INTERNAL_NU_FAMILIES {
        commands.extend(family.commands.iter().map(|command| command.metadata));
    }
    commands.sort_by(|left, right| left.name.cmp(right.name));
    commands
}

pub fn classify_yzx_root_route(argv: &[String]) -> Result<YzxPublicRootRoute<'_>, CoreError> {
    let Some(first) = argv.first().map(|value| value.as_str()) else {
        return Ok(YzxPublicRootRoute::Help);
    };

    if matches!(first, "help" | "-h" | "--help") {
        return Ok(YzxPublicRootRoute::Help);
    }

    if matches!(first, "-V" | "--version" | "-v" | "--version-short") {
        return Ok(YzxPublicRootRoute::Version);
    }

    if RUST_CONTROL_FAMILIES
        .iter()
        .any(|family| family.root_token == first)
    {
        return Ok(YzxPublicRootRoute::RustControl);
    }

    if let Some(family) = INTERNAL_NU_FAMILIES
        .iter()
        .find(|family| family.root_token == first)
    {
        return Ok(YzxPublicRootRoute::InternalNu(plan_internal_nu_route(
            family,
            &argv[1..],
        )?));
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

const fn leaf(
    metadata: YzxCommandMetadata,
    tokens_after_root: &'static [&'static str],
    module_relative_path: &'static [&'static str],
) -> YzxCommandLeaf {
    YzxCommandLeaf {
        metadata,
        tokens_after_root,
        module_relative_path,
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

const fn internal_family(
    root_token: &'static str,
    commands: &'static [YzxCommandLeaf],
    root_command_index: Option<usize>,
    help_token_routes_to_root_empty_tail: bool,
    help_flags_route_to_root_with_tail: bool,
    unknown_subcommand_behavior: YzxUnknownSubcommandBehavior,
    required_subcommands: &'static [&'static str],
) -> YzxInternalNuFamily {
    YzxInternalNuFamily {
        root_token,
        commands,
        root_command_index,
        help_token_routes_to_root_empty_tail,
        help_flags_route_to_root_with_tail,
        unknown_subcommand_behavior,
        required_subcommands,
    }
}

fn plan_internal_nu_route<'a>(
    family: &'static YzxInternalNuFamily,
    argv: &'a [String],
) -> Result<YzxInternalNuRoutePlan<'a>, CoreError> {
    if let Some(command) = match_subcommand(family, argv) {
        let tail = &argv[command.tokens_after_root.len()..];
        return Ok(plan_route(command, tail));
    }

    if argv.is_empty() {
        if let Some(command) = root_command(family) {
            return Ok(plan_route(command, argv));
        }
        return Err(required_subcommand_error(
            family.root_token,
            family.required_subcommands,
        ));
    }

    if family.help_token_routes_to_root_empty_tail && matches!(first_arg(argv), Some("help")) {
        if let Some(command) = root_command(family) {
            return Ok(plan_route(command, &[]));
        }
    }

    if family.help_flags_route_to_root_with_tail
        && matches!(first_arg(argv), Some("-h") | Some("--help"))
    {
        if let Some(command) = root_command(family) {
            return Ok(plan_route(command, argv));
        }
    }

    if !family.required_subcommands.is_empty() && root_command(family).is_none() {
        return Err(required_subcommand_error(
            family.root_token,
            family.required_subcommands,
        ));
    }

    match family.unknown_subcommand_behavior {
        YzxUnknownSubcommandBehavior::RouteRoot => {
            let command =
                root_command(family).expect("route-root families must define a root command");
            Ok(plan_route(command, argv))
        }
        YzxUnknownSubcommandBehavior::Error => Err(unknown_subcommand_error(family.root_token)),
    }
}

fn root_command(family: &'static YzxInternalNuFamily) -> Option<&'static YzxCommandLeaf> {
    family
        .root_command_index
        .map(|index| &family.commands[index])
}

fn match_subcommand(
    family: &'static YzxInternalNuFamily,
    argv: &[String],
) -> Option<&'static YzxCommandLeaf> {
    family
        .commands
        .iter()
        .filter(|command| {
            !command.tokens_after_root.is_empty()
                && tokens_match_prefix(command.tokens_after_root, argv)
        })
        .max_by_key(|command| command.tokens_after_root.len())
}

fn tokens_match_prefix(expected: &[&str], argv: &[String]) -> bool {
    argv.len() >= expected.len()
        && expected
            .iter()
            .zip(argv.iter())
            .all(|(expected, actual)| *expected == actual.as_str())
}

fn plan_route<'a>(
    command: &'static YzxCommandLeaf,
    tail: &'a [String],
) -> YzxInternalNuRoutePlan<'a> {
    YzxInternalNuRoutePlan {
        module_relative_path: command.module_relative_path,
        command_name: command.metadata.name,
        tail,
    }
}

fn first_arg(argv: &[String]) -> Option<&str> {
    argv.first().map(String::as_str)
}

fn unknown_subcommand_error(route: &str) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "unknown_subcommand",
        format!("Unknown yzx {route} subcommand"),
        format!("Run `yzx {route} --help` or `yzx --help` to see supported commands."),
        json!({ "route": route }),
    )
}

fn required_subcommand_error(route: &str, expected: &[&str]) -> CoreError {
    CoreError::classified(
        ErrorClass::Usage,
        "missing_subcommand",
        format!("yzx {route} requires one of: {}", expected.join(", ")),
        format!("Run `yzx {route} --help` or `yzx --help` to see supported subcommands."),
        json!({ "route": route, "expected": expected }),
    )
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the public Rust root keeps the already migrated control-plane family on the Rust-owned path.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
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
            classify_yzx_root_route(&["warp".into(), "/tmp/project".into()]).unwrap(),
            YzxPublicRootRoute::RustControl
        );
        assert_eq!(
            classify_yzx_root_route(&["run".into(), "rg".into()]).unwrap(),
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
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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
    }

    // Defends: the Rust root rejects unknown top-level commands instead of reviving the old generic Nu root fallback.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn rejects_unknown_top_level_command() {
        let err = classify_yzx_root_route(&["not-a-command".into()]).unwrap_err();
        assert!(matches!(err.class(), ErrorClass::Usage));
        assert_eq!(err.code(), "unknown_command");
    }

    // Defends: grouped Rust-owned families route through yzx_control instead of reviving direct Nu module ownership.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn routes_grouped_rust_family_to_control_plane() {
        let argv = [String::from("desktop"), String::from("launch")];
        let route = classify_yzx_root_route(&argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));
    }

    // Regression: dev and import keep their explicit help shims instead of treating `help` as an unknown subcommand.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn keeps_help_alias_behavior_for_grouped_internal_families() {
        let dev_argv = [
            String::from("dev"),
            String::from("help"),
            String::from("ignored"),
        ];
        let route = classify_yzx_root_route(&dev_argv).unwrap();
        let YzxPublicRootRoute::InternalNu(plan) = route else {
            panic!("expected internal Nu route");
        };
        assert_eq!(plan.command_name, "yzx dev");
        assert!(plan.tail.is_empty());

        let import_argv = [String::from("import"), String::from("--help")];
        let route = classify_yzx_root_route(&import_argv).unwrap();
        assert!(matches!(route, YzxPublicRootRoute::RustControl));
    }

    // Regression: the direct route planner must preserve alias leaves and the family-specific missing-subcommand contract.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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

        let err = classify_yzx_root_route(&[String::from("dev"), String::from("not-a-subcommand")])
            .unwrap_err();
        assert_eq!(err.code(), "unknown_subcommand");
    }

    // Defends: nested maintainer Nu leaves keep their longest-prefix route instead of falling back to the dev root.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn routes_nested_dev_rust_commands_to_internal_nu_leaf() {
        let argv = [
            String::from("dev"),
            String::from("rust"),
            String::from("test"),
            String::from("core"),
            String::from("front_door_render"),
        ];
        let route = classify_yzx_root_route(&argv).unwrap();
        let YzxPublicRootRoute::InternalNu(plan) = route else {
            panic!("expected internal Nu route");
        };
        assert_eq!(plan.command_name, "yzx dev rust test");
        assert_eq!(
            plan.tail,
            [String::from("core"), String::from("front_door_render")]
        );
    }

    // Defends: maintainer session inspection stays reachable through the public `yzx dev` route.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=1 total=8/10
    #[test]
    fn routes_dev_inspect_session_to_internal_nu_leaf() {
        let argv = [
            String::from("dev"),
            String::from("inspect_session"),
            String::from("--json"),
        ];
        let route = classify_yzx_root_route(&argv).unwrap();
        let YzxPublicRootRoute::InternalNu(plan) = route else {
            panic!("expected internal Nu route");
        };
        assert_eq!(plan.command_name, "yzx dev inspect_session");
        assert_eq!(plan.tail, [String::from("--json")]);
    }

    // Regression: menu visibility and menu categories come from the shared Rust command surface instead of a second Nushell-owned map.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
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
