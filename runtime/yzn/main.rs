mod cli;
mod command;
mod doctor;
mod error;
mod paths;
mod runtime;
mod status;
mod zellij;

use std::process;

pub(crate) const YZN_CONFIG_UI: &str = "@yznConfigUi@";
pub(crate) const YZN_MENU: &str = "@yznMenu@";
pub(crate) const YZN_TUTOR: &str = "@yznTutor@";
pub(crate) const YZN_SCREEN: &str = "@yznScreen@";
pub(crate) const YZN_WELCOME: &str = "@yznWelcome@";
pub(crate) const YZN_SHELL: &str = "@yznShell@";
pub(crate) const YZN_ENV_SUPERVISOR: &str = "@yznEnvSupervisor@";
pub(crate) const ZELLIJ: &str = "@zellij@";
pub(crate) const MARS: &str = "@mars@";
pub(crate) const LAYOUT: &str = "@layout@";
pub(crate) const LAYOUT_TEMPLATE: &str = "@layoutTemplate@";
pub(crate) const LAYOUT_SWAP_TEMPLATE: &str = "@layoutSwapTemplate@";
pub(crate) const YZN_YAZI: &str = "@yznYazi@";
pub(crate) const YZN_HELIX: &str = "@yznHelix@";
pub(crate) const YZN_CONFIG: &str = "@yznConfig@";
pub(crate) const YZN_MARS_CONFIG: &str = "@yznMarsConfig@";
pub(crate) const YZN_ZELLIJ_CONFIG: &str = "@yznZellijConfig@";
pub(crate) const YZN_CONFIG_KDL: &str = "@yznConfigKdl@";
pub(crate) const YZN_REVEAL: &str = "@yznReveal@";
pub(crate) const YZN_SIDEBAR_REFRESH: &str = "@yznSidebarRefresh@";
pub(crate) const YZN_YA: &str = "@yznYa@";
pub(crate) const YZN_BAR_RENDER_REQUEST: &str = "@yznBarRenderRequest@";
pub(crate) const YZN_BAR_RENDER: &str = "@yznBarRender@";
pub(crate) const YAZELIX_ZELLIJ_POPUP_WASM: &str = "@yazelixZellijPopupWasm@";
pub(crate) const YAZELIX_ZELLIJ_BAR_WASM: &str = "@yazelixZellijBarWasm@";
pub(crate) const YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM: &str =
    "@yazelixZellijPaneOrchestratorWasm@";
pub(crate) const DEFAULT_BAR_WIDGETS_JSON: &str = r#"@defaultBarWidgetsJson@"#;
pub(crate) const DEFAULT_SHELL_PROGRAM: &str = "@defaultShellProgram@";
pub(crate) const DEFAULT_POPUP_SIDE_MARGIN: &str = "@defaultPopupSideMargin@";
pub(crate) const DEFAULT_POPUP_VERTICAL_MARGIN: &str = "@defaultPopupVerticalMargin@";
pub(crate) const CUSTOM_POPUPS_KDL_CONFIG_PATH: &str = "popups.kdl";
pub(crate) const CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH: &str = "popups.keybindings.kdl";
pub(crate) const PATH_PREFIX: &str = "@pathPrefix@";
pub(crate) const SPONSOR_URL: &str = "https://github.com/sponsors/luccahuguet";
pub(crate) const ZELLIJ_HOME_PLACEHOLDER: &str = "\"__YZN_HOME__\"";
pub(crate) const LAYOUT_YAZI_PLACEHOLDER: &str = concat!("@", "yazi", "@");
pub(crate) const LAYOUT_BAR_PLACEHOLDER: &str = concat!("@", "bar", "@");
pub(crate) const HELIX_REVEAL_COMMAND: &str = r#":sh yzn reveal "%{buffer_name}""#;
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
