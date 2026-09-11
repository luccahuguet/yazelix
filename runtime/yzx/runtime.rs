use std::{
    env,
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    AGENT_POPUP_KDL_CONFIG_PATH, CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH,
    CUSTOM_POPUPS_KDL_CONFIG_PATH, MANAGED_HELIX, MANAGED_KEYBINDING_SPECS, NOVA_BAR_WASM, RIO,
    YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM, YAZELIX_ZELLIJ_POPUP_WASM, YZX_CONFIG, YZX_CONFIG_KDL,
    YZX_EDITOR, YZX_HELIX, YZX_ZELLIJ_CONFIG, ZELLIJ, ZJ_RADAR_WASM,
    command::{create_dir_all_checked, run_checked, trim_output},
    error::{AppError, path_error, startup},
    paths::{config_home, home_dir, nonempty_env, parent, runtime_path, state_dir},
    yazi::YaziRuntime,
    zellij::{active_layout, active_zellij_config},
};

pub(crate) struct Runtime {
    pub(crate) config_home: PathBuf,
    pub(crate) state_dir: PathBuf,
    bridge_session_id: Option<OsString>,
    pub(crate) yzx_open_log: String,
    pub(crate) shell_program: String,
    pub(crate) editor_command: String,
    pub(crate) editor: String,
    pub(crate) agent_command: String,
    pub(crate) agent_args: String,
    pub(crate) sidebar_command: String,
    pub(crate) sidebar_args: String,
    pub(crate) welcome_enabled: String,
    pub(crate) welcome_style: String,
    pub(crate) welcome_duration_seconds: String,
    pub(crate) rio_config: PathBuf,
    pub(crate) zellij_sidecar: PathBuf,
    pub(crate) zellij_config: PathBuf,
    pub(crate) appearance_mode: String,
    zellij_config_source: &'static str,
    pub(crate) layout: PathBuf,
    layout_source: &'static str,
    pub(crate) bar_widgets: String,
    pub(crate) popup_side_margin: String,
    pub(crate) popup_vertical_margin: String,
    pub(crate) managed_keybindings: Vec<ManagedKeybinding>,
    pub(crate) zellij_status_cache: PathBuf,
    yazi: Option<YaziRuntime>,
}

const ZELLIJ_PERMISSIONS_FILE: &str = "zellij/permissions.kdl";

pub(crate) struct ManagedKeybinding {
    pub(crate) label: &'static str,
    pub(crate) path: &'static str,
    pub(crate) default: &'static str,
    pub(crate) configured: Option<String>,
}

impl ManagedKeybinding {
    pub(crate) fn is_default(&self) -> bool {
        self.configured.as_deref() == Some(self.default)
    }

    pub(crate) fn description(&self) -> String {
        match self.configured.as_deref() {
            None => "unmapped".to_string(),
            Some(chord) if self.is_default() => format!("{chord} (default)"),
            Some(chord) => format!("{chord} (remapped)"),
        }
    }
}

fn read_managed_keybindings(
    config_home: &Path,
    config_toml: &Path,
) -> Result<Vec<ManagedKeybinding>, AppError> {
    MANAGED_KEYBINDING_SPECS
        .iter()
        .filter(|&&(_, path, _)| MANAGED_HELIX == "included" || path != "keybindings.sidebar_focus")
        .map(|&(label, path, default)| {
            let configured = trim_output(config_value(config_home, config_toml, path)?);
            Ok(ManagedKeybinding {
                label,
                path,
                default,
                configured: (configured != "false").then_some(configured),
            })
        })
        .collect()
}

fn seed_plugin_permissions(path: &Path, radar_enabled: bool) -> Result<(), AppError> {
    create_dir_all_checked(parent(path), path)?;
    let current = match fs::read_to_string(path) {
        Ok(current) => current,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(path_error("read", path, path, error)),
    };
    let grants = [
        (
            YAZELIX_ZELLIJ_POPUP_WASM,
            concat!(
                "ReadApplicationState ChangeApplicationState OpenTerminalsOrPlugins ",
                "RunCommands ReadCliPipes",
            ),
        ),
        (
            NOVA_BAR_WASM,
            "ReadApplicationState ChangeApplicationState RunCommands",
        ),
        (
            ZJ_RADAR_WASM,
            if radar_enabled {
                concat!(
                    "ReadApplicationState ChangeApplicationState RunCommands ReadCliPipes ",
                    "MessageAndLaunchOtherPlugins",
                )
            } else {
                ""
            },
        ),
        (
            YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM,
            concat!(
                "ReadApplicationState ChangeApplicationState OpenTerminalsOrPlugins ",
                "RunCommands WriteToStdin ReadCliPipes MessageAndLaunchOtherPlugins ",
                "ReadSessionEnvironmentVariables",
            ),
        ),
    ];
    let mut additions = String::new();
    for (plugin, permissions) in grants {
        if permissions.is_empty() {
            continue;
        }
        let header = format!("\"{plugin}\" {{");
        let complete = current.rsplit_once(&header).is_some_and(|(_, tail)| {
            tail.split_once('}').is_some_and(|(body, _)| {
                permissions
                    .split_ascii_whitespace()
                    .all(|permission| body.lines().any(|line| line.trim() == permission))
            })
        });
        if !complete {
            additions.push_str(&format!(
                "{header}\n    {}\n}}\n",
                permissions.replace(' ', "\n    ")
            ));
        }
    }
    if additions.is_empty() {
        return Ok(());
    }
    if !current.is_empty() && !current.ends_with('\n') {
        additions.insert(0, '\n');
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(additions.as_bytes()))
        .map_err(|error| path_error("write", path, path, error))
}

impl Runtime {
    pub(crate) fn inspect(with_yazi: bool) -> Result<Self, AppError> {
        let yazi = with_yazi.then(YaziRuntime::resolve).transpose()?;
        if let Some(yazi) = &yazi {
            yazi.warn();
        }
        Self::resolve(yazi, false, false)
    }

    pub(crate) fn prepare() -> Result<Self, AppError> {
        Self::resolve(None, false, true)
    }

    pub(crate) fn prepare_with_yazi() -> Result<Self, AppError> {
        let yazi = YaziRuntime::resolve()?;
        yazi.warn();
        Self::resolve(Some(yazi), false, true)
    }

    pub(crate) fn prepare_new_session_with_yazi() -> Result<Self, AppError> {
        let yazi = YaziRuntime::resolve()?;
        yazi.warn();
        Self::resolve(Some(yazi), true, true)
    }

    fn resolve(
        yazi: Option<YaziRuntime>,
        new_session: bool,
        materialize: bool,
    ) -> Result<Self, AppError> {
        let state_dir = state_dir();
        match fs::metadata(&state_dir) {
            Ok(metadata) if !metadata.is_dir() => {
                return Err(startup(
                    "state path is not a directory; move the conflicting file or set YAZELIX_STATE_DIR to a directory",
                    state_dir.display(),
                    1,
                ));
            }
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                return Err(path_error("inspect", &state_dir, &state_dir, error));
            }
            _ => {}
        }
        let home_dir = home_dir()?;
        let config_home = config_home()?;
        let config_toml = config_home.join("config.toml");
        let rio_config = config_home.join("rio/config.toml");
        if materialize && !RIO.is_empty() {
            run_checked(
                &rio_config,
                Command::new(YZX_CONFIG)
                    .arg("--init-rio")
                    .env("YAZELIX_CONFIG_HOME", &config_home),
            )?;
        }
        let yzx_open_log = config_value(&config_home, &config_toml, "open.log_level")?;
        let shell_program = trim_output(config_value(&config_home, &config_toml, "shell.program")?);
        let editor_command =
            trim_output(config_value(&config_home, &config_toml, "editor.command")?);
        let editor = effective_editor_command(&editor_command);
        let agent_command = trim_output(config_value(&config_home, &config_toml, "agent.command")?);
        let agent_args = trim_output(config_value(&config_home, &config_toml, "agent.args")?);
        let sidebar_command =
            trim_output(config_value(&config_home, &config_toml, "sidebar.command")?);
        let sidebar_args = trim_output(config_value(&config_home, &config_toml, "sidebar.args")?);
        let sidebar_pane_kdl = config_value(
            &config_home,
            &config_toml,
            crate::SIDEBAR_PANE_KDL_CONFIG_PATH,
        )?;
        let radar_enabled = sidebar_command == crate::SIDEBAR_RADAR_COMMAND;
        let welcome_enabled = config_value(&config_home, &config_toml, "welcome.enabled")?;
        let welcome_style = config_value(&config_home, &config_toml, "welcome.style")?;
        let welcome_duration_seconds =
            config_value(&config_home, &config_toml, "welcome.duration_seconds")?;
        let configured_appearance =
            trim_output(config_value(&config_home, &config_toml, "appearance.mode")?);
        let appearance_mode = current_appearance_mode(configured_appearance, new_session);
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
        let managed_keybindings = read_managed_keybindings(&config_home, &config_toml)?;
        let custom_popups_kdl =
            config_value(&config_home, &config_toml, CUSTOM_POPUPS_KDL_CONFIG_PATH)?;
        let custom_popup_keybindings_kdl = config_value(
            &config_home,
            &config_toml,
            CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH,
        )?;
        let agent_popup_kdl =
            config_value(&config_home, &config_toml, AGENT_POPUP_KDL_CONFIG_PATH)?;
        let (layout_source, layout) = active_layout(
            &state_dir,
            &appearance_mode,
            &bar_widgets,
            &shell_program,
            &sidebar_pane_kdl,
            radar_enabled,
            materialize,
        )?;
        let zellij_sidecar = config_home.join("zellij/config.kdl");
        let zellij_plugins_sidecar = config_home.join("zellij/plugins.kdl");
        let zellij_text = run_checked(
            &zellij_sidecar,
            Command::new(YZX_ZELLIJ_CONFIG)
                .arg(YZX_CONFIG_KDL)
                .arg(&zellij_sidecar),
        )?;
        let zellij_config_source = if zellij_sidecar.is_file() {
            "sidecar"
        } else {
            "packaged"
        };
        let (zellij_config_source, zellij_config) = active_zellij_config(
            &state_dir,
            zellij_config_source,
            PathBuf::from(YZX_CONFIG_KDL),
            zellij_text,
            &layout,
            &popup_side_margin,
            &popup_vertical_margin,
            &managed_keybindings,
            &agent_popup_kdl,
            &custom_popups_kdl,
            &custom_popup_keybindings_kdl,
            &zellij_plugins_sidecar,
            &home_dir,
            radar_enabled,
            materialize,
        )?;
        let zellij_status_cache = state_dir.join("zellij/session/status_bar_cache.json");
        if materialize {
            create_dir_all_checked(parent(&zellij_status_cache), &zellij_status_cache)?;
            seed_plugin_permissions(&state_dir.join(ZELLIJ_PERMISSIONS_FILE), radar_enabled)?;
        }

        Ok(Self {
            config_home,
            state_dir,
            bridge_session_id: uses_helix_bridge(&editor).then(bridge_session_id),
            yzx_open_log: trim_output(yzx_open_log),
            shell_program,
            editor_command,
            editor,
            agent_command,
            agent_args,
            sidebar_command,
            sidebar_args,
            welcome_enabled: trim_output(welcome_enabled),
            welcome_style: trim_output(welcome_style),
            welcome_duration_seconds: trim_output(welcome_duration_seconds),
            rio_config,
            zellij_sidecar,
            zellij_config,
            appearance_mode,
            zellij_config_source,
            layout,
            layout_source,
            bar_widgets,
            popup_side_margin,
            popup_vertical_margin,
            managed_keybindings,
            zellij_status_cache,
            yazi,
        })
    }

    pub(crate) fn apply(&self, command: &mut Command) {
        let yzx_menu_yzx = env::current_exe().unwrap_or_else(|_| PathBuf::from("yzx"));
        command
            .env("YAZELIX_CONFIG_HOME", &self.config_home)
            .env("YAZELIX_STATE_DIR", &self.state_dir)
            .env("YAZELIX_EDITOR", &self.editor)
            .env("EDITOR", YZX_EDITOR)
            .env("VISUAL", YZX_EDITOR)
            .env("YZX_EDITOR", &self.editor)
            .env("GIT_EDITOR", YZX_EDITOR)
            .env("YZX_OPEN_LOG", &self.yzx_open_log)
            .env("YZX_WELCOME_ENABLED", &self.welcome_enabled)
            .env("YZX_WELCOME_STYLE", &self.welcome_style)
            .env(
                "YZX_RADAR_ENABLED",
                if self.radar_enabled() {
                    "true"
                } else {
                    "false"
                },
            )
            .env(
                "YZX_WELCOME_DURATION_SECONDS",
                &self.welcome_duration_seconds,
            )
            .env("YAZELIX_STATUS_BAR_CACHE_PATH", &self.zellij_status_cache)
            .env(
                "ZELLIJ_PLUGIN_PERMISSIONS_CACHE",
                self.state_dir.join(ZELLIJ_PERMISSIONS_FILE),
            )
            .env("YZX_MENU_YZX", yzx_menu_yzx)
            .env("YZX_ZELLIJ", ZELLIJ)
            .env("PATH", runtime_path());
        if let Some(yazi) = &self.yazi {
            command
                .env("YZX_YAZI_BIN", &yazi.yazi)
                .env("YZX_YA", &yazi.ya);
        }
        if let Some(bridge_session_id) = &self.bridge_session_id {
            command.env("YAZELIX_HELIX_BRIDGE_SESSION_ID", bridge_session_id);
        }
    }

    pub(crate) fn yazi(&self) -> &YaziRuntime {
        self.yazi
            .as_ref()
            .expect("Yazi runtime was not prepared for this command")
    }

    pub(crate) fn radar_enabled(&self) -> bool {
        self.sidebar_command == crate::SIDEBAR_RADAR_COMMAND
    }

    pub(crate) fn rio_config(&self) -> String {
        if !RIO.is_empty() {
            source_path("user", &self.rio_config)
        } else {
            "not included".to_string()
        }
    }

    pub(crate) fn zellij_config(&self) -> String {
        source_path(self.zellij_config_source, &self.zellij_config)
    }

    pub(crate) fn layout(&self) -> String {
        source_path(self.layout_source, &self.layout)
    }
}

fn source_path(source: &str, path: &Path) -> String {
    let state = if path.is_file() {
        ""
    } else {
        "; not initialized"
    };
    format!("{source} ({}{state})", path.display())
}

fn config_value(config_home: &Path, config_toml: &Path, key: &str) -> Result<String, AppError> {
    run_checked(
        config_toml,
        Command::new(YZX_CONFIG)
            .arg("--get")
            .arg(key)
            .env("YAZELIX_CONFIG_HOME", config_home),
    )
}

fn effective_editor_command(command: &str) -> String {
    if matches!(command, "yzx-hx" | "hx") {
        YZX_HELIX.to_string()
    } else {
        command.to_string()
    }
}

fn select_appearance_mode(
    configured: String,
    session_mode: Option<&OsStr>,
    live: bool,
    new_session: bool,
) -> String {
    if !new_session && !live {
        if let Some(mode @ ("dark" | "light")) = session_mode.and_then(OsStr::to_str) {
            return mode.to_string();
        }
    }
    configured
}

pub(crate) fn current_appearance_mode(configured: String, new_session: bool) -> String {
    let session_mode = nonempty_env("YZX_APPEARANCE_MODE");
    select_appearance_mode(
        configured,
        session_mode.as_deref(),
        nonempty_env("YZX_APPEARANCE_LIVE").as_deref() == Some(OsStr::new("1")),
        new_session,
    )
}

fn bridge_session_id() -> OsString {
    nonempty_env("YAZELIX_HELIX_BRIDGE_SESSION_ID").unwrap_or_else(|| {
        OsString::from(format!(
            "yzx-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
            process::id()
        ))
    })
}

fn uses_helix_bridge(command: &str) -> bool {
    command == YZX_HELIX || Path::new(command).file_name() == Some(OsStr::new("yzx-hx"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::YZX_HELIX;

    #[test]
    fn short_hx_maps_to_packaged_helix_bridge() {
        assert_eq!(effective_editor_command("hx"), YZX_HELIX);
        assert!(uses_helix_bridge(YZX_HELIX));
        assert!(uses_helix_bridge("/nix/store/example/bin/yzx-hx"));
        assert!(uses_helix_bridge("yzx-hx"));
        assert!(!uses_helix_bridge("hx"));
        assert!(!uses_helix_bridge("nvim"));
    }

    #[test]
    fn read_only_sessions_keep_their_captured_appearance() {
        assert_eq!(
            select_appearance_mode("light".into(), Some(OsStr::new("dark")), false, false),
            "dark"
        );
        assert_eq!(
            select_appearance_mode("light".into(), Some(OsStr::new("dark")), true, false),
            "light"
        );
        assert_eq!(
            select_appearance_mode("light".into(), Some(OsStr::new("dark")), false, true),
            "light"
        );
    }
}
