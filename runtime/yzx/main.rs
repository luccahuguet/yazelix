mod cli;
mod command;
mod desktop;
mod doctor;
mod error;
mod paths;
mod runtime;
mod status;
mod zellij;

use std::process;

pub(crate) const YZX_CONFIG_UI: &str = "@yzxConfigUi@";
pub(crate) const YZX_AGENT: &str = "@yzxAgent@";
pub(crate) const YZX_MENU: &str = "@yzxMenu@";
pub(crate) const YZX_TUTOR: &str = "@yzxTutor@";
pub(crate) const YZX_SCREEN: &str = "@yzxScreen@";
pub(crate) const YZX_WELCOME: &str = "@yzxWelcome@";
pub(crate) const YZX_SHELL: &str = "@yzxShell@";
pub(crate) const YZX_ENV_SUPERVISOR: &str = "@yzxEnvSupervisor@";
pub(crate) const ZELLIJ: &str = "@zellij@";
pub(crate) const MARS: &str = "@mars@";
pub(crate) const DESKTOP_ENTRY_SOURCE: &str = "@desktopEntrySource@";
pub(crate) const DESKTOP_DATABASE_UPDATER: &str = "@desktopDatabaseUpdater@";
pub(crate) const DEFAULT_STATE_DIR: &str = "@defaultStateDir@";
pub(crate) const PACKAGE_VARIANT: &str = if MARS.is_empty() { "runtime" } else { "full" };
pub(crate) const LAYOUT: &str = "@layout@";
pub(crate) const LAYOUT_TEMPLATE: &str = "@layoutTemplate@";
pub(crate) const LAYOUT_SWAP_TEMPLATE: &str = "@layoutSwapTemplate@";
pub(crate) const YZX_YAZI: &str = "@yzxYazi@";
pub(crate) const YZX_HELIX: &str = "@yzxHelix@";
pub(crate) const YZX_EDITOR: &str = "@yzxEditor@";
pub(crate) const YZX_CONFIG: &str = "@yzxConfig@";
pub(crate) const YZX_MARS_CONFIG: &str = "@yzxMarsConfig@";
pub(crate) const YZX_ZELLIJ_CONFIG: &str = "@yzxZellijConfig@";
pub(crate) const YZX_CONFIG_KDL: &str = "@yzxConfigKdl@";
pub(crate) const YZX_RUNTIME_IDENTITY: &str = "@yzxRuntimeIdentity@";
pub(crate) const YZX_REVEAL: &str = "@yzxReveal@";
pub(crate) const YZX_SIDEBAR_REFRESH: &str = "@yzxSidebarRefresh@";
pub(crate) const YZX_YA: &str = "@yzxYa@";
pub(crate) const YZX_BAR_RENDER_REQUEST: &str = "@yzxBarRenderRequest@";
pub(crate) const YZX_BAR_RENDER: &str = "@yzxBarRender@";
pub(crate) const YAZELIX_ZELLIJ_POPUP_WASM: &str = "@yazelixZellijPopupWasm@";
pub(crate) const YAZELIX_ZELLIJ_BAR_WASM: &str = "@yazelixZellijBarWasm@";
pub(crate) const YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM: &str =
    "@yazelixZellijPaneOrchestratorWasm@";
pub(crate) const DEFAULT_BAR_WIDGETS_JSON: &str = r#"@defaultBarWidgetsJson@"#;
pub(crate) const DEFAULT_SHELL_PROGRAM: &str = "@defaultShellProgram@";
pub(crate) const DEFAULT_POPUP_SIDE_MARGIN: &str = "@defaultPopupSideMargin@";
pub(crate) const DEFAULT_POPUP_VERTICAL_MARGIN: &str = "@defaultPopupVerticalMargin@";
pub(crate) const AGENT_POPUP_KDL_CONFIG_PATH: &str = "agent.popup.kdl";
pub(crate) const AGENT_AUTO_COMMAND: &str = "auto";
pub(crate) const CUSTOM_POPUPS_KDL_CONFIG_PATH: &str = "popups.kdl";
pub(crate) const CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH: &str = "popups.keybindings.kdl";
pub(crate) const PATH_PREFIX: &str = "@pathPrefix@";
pub(crate) const VERSION: &str = "@version@";
pub(crate) const ZELLIJ_HOME_PLACEHOLDER: &str = "\"__YZX_HOME__\"";
pub(crate) const LAYOUT_YAZI_PLACEHOLDER: &str = concat!("@", "yazi", "@");
pub(crate) const LAYOUT_BAR_PLACEHOLDER: &str = concat!("@", "bar", "@");
pub(crate) const HELIX_REVEAL_COMMAND: &str = r#":sh yzx reveal "%{buffer_name}""#;
pub(crate) const POPUP_KEYBINDING_SPECS: &[(&str, &str, &str)] = &[
    ("config", "keybindings.config", "@defaultConfigKeybinding@"),
    ("agent", "keybindings.agent", "@defaultAgentKeybinding@"),
    ("git", "keybindings.git", "@defaultGitKeybinding@"),
    ("menu", "keybindings.menu", "@defaultMenuKeybinding@"),
];

fn main() {
    process::exit(
        cli::run()
            .map(|()| 0)
            .unwrap_or_else(error::AppError::report),
    );
}
