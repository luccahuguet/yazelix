use std::{
    env,
    ffi::{OsStr, OsString},
    fmt::Display,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    command::{
        create_dir_all_checked, run_checked, seed_permission_checked, touch_checked, trim_output,
    },
    error::AppError,
    paths::{config_home, home_dir, nonempty_env, parent, runtime_path, state_dir},
    zellij::{active_layout, active_zellij_config},
    CUSTOM_POPUPS_KDL_CONFIG_PATH, CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH,
    POPUP_KEYBINDING_SPECS, YAZELIX_ZELLIJ_BAR_WASM, YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM,
    YAZELIX_ZELLIJ_POPUP_WASM, YZN_CONFIG, YZN_CONFIG_KDL, YZN_HELIX, YZN_MARS_CONFIG,
    YZN_MARS_LIGHT_CONFIG, YZN_YA, YZN_ZELLIJ_CONFIG, ZELLIJ,
};

pub(crate) struct Runtime {
    pub(crate) config_home: PathBuf,
    pub(crate) state_dir: PathBuf,
    bridge_session_id: Option<OsString>,
    pub(crate) yzn_open_log: String,
    pub(crate) shell_program: String,
    pub(crate) editor_command: String,
    pub(crate) editor: String,
    pub(crate) appearance_mode: String,
    pub(crate) welcome_enabled: String,
    pub(crate) welcome_style: String,
    pub(crate) welcome_duration_seconds: String,
    mars_config_source: &'static str,
    mars_config_home: PathBuf,
    pub(crate) zellij_sidecar: PathBuf,
    pub(crate) zellij_config: PathBuf,
    zellij_config_source: &'static str,
    pub(crate) layout: PathBuf,
    layout_source: &'static str,
    pub(crate) bar_widgets: String,
    pub(crate) popup_side_margin: String,
    pub(crate) popup_vertical_margin: String,
    pub(crate) popup_keybindings: Vec<PopupKeybinding>,
    pub(crate) zellij_status_cache: PathBuf,
    pub(crate) zellij_permissions: PathBuf,
}

pub(crate) struct PopupKeybinding {
    pub(crate) label: &'static str,
    pub(crate) path: &'static str,
    pub(crate) default: &'static str,
    pub(crate) configured: String,
}

fn read_popup_keybindings(
    config_home: &Path,
    config_toml: &Path,
) -> Result<Vec<PopupKeybinding>, AppError> {
    POPUP_KEYBINDING_SPECS
        .iter()
        .map(|&(label, path, default)| {
            Ok(PopupKeybinding {
                label,
                path,
                default,
                configured: trim_output(config_value(config_home, config_toml, path)?),
            })
        })
        .collect()
}

impl Runtime {
    pub(crate) fn prepare() -> Result<Self, AppError> {
        let state_dir = state_dir();
        create_dir_all_checked(&state_dir, &state_dir)?;
        let home_dir = home_dir()?;
        let config_home = config_home()?;
        let config_toml = config_home.join("config.toml");
        let yzn_open_log = config_value(&config_home, &config_toml, "open.log_level")?;
        let shell_program = config_value(&config_home, &config_toml, "shell.program")?;
        let editor_command =
            trim_output(config_value(&config_home, &config_toml, "editor.command")?);
        let editor = effective_editor_command(&editor_command);
        let appearance_mode =
            trim_output(config_value(&config_home, &config_toml, "appearance.mode")?);
        let welcome_enabled = config_value(&config_home, &config_toml, "welcome.enabled")?;
        let welcome_style = config_value(&config_home, &config_toml, "welcome.style")?;
        let welcome_duration_seconds =
            config_value(&config_home, &config_toml, "welcome.duration_seconds")?;
        let bar_widgets = trim_output(config_value(&config_home, &config_toml, "bar.widgets")?);
        let popup_side_margin = trim_output(config_value(
            &config_home,
            &config_toml,
            "popup.side_margin",
        )?);
        let popup_vertical_margin = trim_output(config_value(
            &config_home,
            &config_toml,
            "popup.vertical_margin",
        )?);
        let popup_keybindings = read_popup_keybindings(&config_home, &config_toml)?;
        let custom_popups_kdl =
            config_value(&config_home, &config_toml, CUSTOM_POPUPS_KDL_CONFIG_PATH)?;
        let custom_popup_keybindings_kdl = config_value(
            &config_home,
            &config_toml,
            CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH,
        )?;
        let (layout_source, layout) = active_layout(&state_dir, &bar_widgets, &appearance_mode)?;
        let user_mars_config_home = config_home.join("mars");
        let (mars_config_source, mars_config_home) =
            if user_mars_config_home.join("config.toml").is_file() {
                ("user", user_mars_config_home)
            } else if appearance_mode == "light" {
                ("packaged-light", PathBuf::from(YZN_MARS_LIGHT_CONFIG))
            } else {
                ("packaged", PathBuf::from(YZN_MARS_CONFIG))
            };
        let zellij_sidecar = config_home.join("zellij/config.kdl");
        let zellij_plugins_sidecar = config_home.join("zellij/plugins.kdl");
        let zellij_config = PathBuf::from(trim_output(run_checked(
            &zellij_sidecar,
            Command::new(YZN_ZELLIJ_CONFIG)
                .arg(YZN_CONFIG_KDL)
                .arg(&zellij_sidecar)
                .arg(state_dir.join("zellij/config.kdl")),
        )?));
        let zellij_config_source = if zellij_config == PathBuf::from(YZN_CONFIG_KDL) {
            "packaged"
        } else {
            "sidecar"
        };
        let (zellij_config_source, zellij_config) = active_zellij_config(
            &state_dir,
            zellij_config_source,
            zellij_config,
            &layout,
            &popup_side_margin,
            &popup_vertical_margin,
            &popup_keybindings,
            &custom_popups_kdl,
            &custom_popup_keybindings_kdl,
            &zellij_plugins_sidecar,
            &home_dir,
        )?;
        let zellij_status_cache = state_dir.join("zellij/session/status_bar_cache.json");
        create_dir_all_checked(parent(&zellij_status_cache), &zellij_status_cache)?;
        let zellij_permissions = state_dir.join("zellij/permissions.kdl");
        create_dir_all_checked(parent(&zellij_permissions), &zellij_permissions)?;
        touch_checked(&zellij_permissions)?;
        seed_permission_checked(
            &zellij_permissions,
            YAZELIX_ZELLIJ_POPUP_WASM,
            &[
                "ReadApplicationState",
                "ChangeApplicationState",
                "OpenTerminalsOrPlugins",
                "RunCommands",
                "ReadCliPipes",
            ],
        )?;
        seed_permission_checked(
            &zellij_permissions,
            YAZELIX_ZELLIJ_BAR_WASM,
            &[
                "ReadApplicationState",
                "ChangeApplicationState",
                "RunCommands",
            ],
        )?;
        seed_permission_checked(
            &zellij_permissions,
            YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM,
            &[
                "ReadApplicationState",
                "ChangeApplicationState",
                "OpenTerminalsOrPlugins",
                "RunCommands",
                "WriteToStdin",
                "ReadCliPipes",
                "MessageAndLaunchOtherPlugins",
                "ReadSessionEnvironmentVariables",
            ],
        )?;

        Ok(Self {
            config_home,
            state_dir,
            bridge_session_id: uses_helix_bridge(&editor).then(bridge_session_id),
            yzn_open_log: trim_output(yzn_open_log),
            shell_program: trim_output(shell_program),
            editor_command,
            editor,
            appearance_mode,
            welcome_enabled: trim_output(welcome_enabled),
            welcome_style: trim_output(welcome_style),
            welcome_duration_seconds: trim_output(welcome_duration_seconds),
            mars_config_source,
            mars_config_home,
            zellij_sidecar,
            zellij_config,
            zellij_config_source,
            layout,
            layout_source,
            bar_widgets,
            popup_side_margin,
            popup_vertical_margin,
            popup_keybindings,
            zellij_status_cache,
            zellij_permissions,
        })
    }

    pub(crate) fn apply(&self, command: &mut Command) {
        let yzn_menu_yzn = env::current_exe().unwrap_or_else(|_| PathBuf::from("yzn"));
        command
            .env("YAZELIX_NEXT_CONFIG_HOME", &self.config_home)
            .env("YAZELIX_STATE_DIR", &self.state_dir)
            .env("YAZELIX_NEXT_EDITOR", &self.editor)
            .env("EDITOR", &self.editor)
            .env("VISUAL", &self.editor)
            .env("YZN_EDITOR", &self.editor)
            .env("GIT_EDITOR", &self.editor)
            .env("YAZELIX_APPEARANCE_MODE", &self.appearance_mode)
            .env("YZN_OPEN_LOG", &self.yzn_open_log)
            .env("YZN_WELCOME_ENABLED", &self.welcome_enabled)
            .env("YZN_WELCOME_STYLE", &self.welcome_style)
            .env(
                "YZN_WELCOME_DURATION_SECONDS",
                &self.welcome_duration_seconds,
            )
            .env("MARS_CONFIG_HOME", &self.mars_config_home)
            .env("YAZELIX_STATUS_BAR_CACHE_PATH", &self.zellij_status_cache)
            .env("ZELLIJ_PLUGIN_PERMISSIONS_CACHE", &self.zellij_permissions)
            .env("YZN_MENU_YZN", yzn_menu_yzn)
            .env("YZN_YA", YZN_YA)
            .env("YZN_ZELLIJ", ZELLIJ)
            .env("PATH", runtime_path());
        if let Some(bridge_session_id) = &self.bridge_session_id {
            command.env("YAZELIX_HELIX_BRIDGE_SESSION_ID", bridge_session_id);
        }
    }

    pub(crate) fn mars_config(&self) -> String {
        source_path(
            self.mars_config_source,
            self.mars_config_home.join("config.toml").display(),
        )
    }

    pub(crate) fn zellij_config(&self) -> String {
        source_path(self.zellij_config_source, self.zellij_config.display())
    }

    pub(crate) fn layout(&self) -> String {
        source_path(self.layout_source, self.layout.display())
    }
}

fn source_path(source: &str, path: impl Display) -> String {
    format!("{source} ({path})")
}

fn config_value(config_home: &Path, config_toml: &Path, key: &str) -> Result<String, AppError> {
    run_checked(
        config_toml,
        Command::new(YZN_CONFIG)
            .arg("--get")
            .arg(key)
            .env("YAZELIX_NEXT_CONFIG_HOME", config_home),
    )
}

fn effective_editor_command(command: &str) -> String {
    if command == "yzn-hx" {
        YZN_HELIX.to_string()
    } else {
        command.to_string()
    }
}

fn bridge_session_id() -> OsString {
    nonempty_env("YAZELIX_HELIX_BRIDGE_SESSION_ID").unwrap_or_else(|| {
        OsString::from(format!(
            "yzn-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
            process::id()
        ))
    })
}

fn uses_helix_bridge(command: &str) -> bool {
    command == YZN_HELIX || Path::new(command).file_name() == Some(OsStr::new("yzn-hx"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::YZN_HELIX;

    #[test]
    fn helix_bridge_only_matches_packaged_yzn_hx() {
        assert!(uses_helix_bridge(YZN_HELIX));
        assert!(uses_helix_bridge("/nix/store/example/bin/yzn-hx"));
        assert!(uses_helix_bridge("yzn-hx"));
        assert!(!uses_helix_bridge("hx"));
        assert!(!uses_helix_bridge("nvim"));
    }
}
