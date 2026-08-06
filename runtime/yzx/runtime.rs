use std::{
    env,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    AGENT_POPUP_KDL_CONFIG_PATH, CUSTOM_POPUP_KEYBINDINGS_KDL_CONFIG_PATH,
    CUSTOM_POPUPS_KDL_CONFIG_PATH, MANAGED_KEYBINDING_SPECS, MARS, YAZELIX_ZELLIJ_BAR_WASM,
    YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM, YAZELIX_ZELLIJ_POPUP_WASM, YZX_CONFIG, YZX_CONFIG_KDL,
    YZX_EDITOR, YZX_HELIX, YZX_MARS_CONFIG, YZX_ZELLIJ_CONFIG, ZELLIJ,
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
    pub(crate) welcome_enabled: String,
    pub(crate) welcome_style: String,
    pub(crate) welcome_duration_seconds: String,
    mars_config_source: &'static str,
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

fn seed_plugin_permissions(path: &Path) -> Result<(), AppError> {
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
            YAZELIX_ZELLIJ_BAR_WASM,
            "ReadApplicationState ChangeApplicationState RunCommands",
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
    pub(crate) fn prepare() -> Result<Self, AppError> {
        Self::prepare_with(None)
    }

    pub(crate) fn prepare_with_yazi() -> Result<Self, AppError> {
        let yazi = YaziRuntime::resolve()?;
        yazi.warn();
        Self::prepare_with(Some(yazi))
    }

    fn prepare_with(yazi: Option<YaziRuntime>) -> Result<Self, AppError> {
        let state_dir = state_dir();
        create_dir_all_checked(&state_dir, &state_dir)?;
        let home_dir = home_dir()?;
        let config_home = config_home()?;
        let config_toml = config_home.join("config.toml");
        let cursor_config = config_home.join("cursors.toml");
        run_checked(
            &cursor_config,
            Command::new(YZX_CONFIG)
                .arg("--init-cursors")
                .env("YAZELIX_CONFIG_HOME", &config_home),
        )?;
        let yzx_open_log = config_value(&config_home, &config_toml, "open.log_level")?;
        let shell_program = trim_output(config_value(&config_home, &config_toml, "shell.program")?);
        let editor_command =
            trim_output(config_value(&config_home, &config_toml, "editor.command")?);
        let editor = effective_editor_command(&editor_command);
        let agent_command = trim_output(config_value(&config_home, &config_toml, "agent.command")?);
        let agent_args = trim_output(config_value(&config_home, &config_toml, "agent.args")?);
        let welcome_enabled = config_value(&config_home, &config_toml, "welcome.enabled")?;
        let welcome_style = config_value(&config_home, &config_toml, "welcome.style")?;
        let welcome_duration_seconds =
            config_value(&config_home, &config_toml, "welcome.duration_seconds")?;
        let appearance_mode =
            trim_output(config_value(&config_home, &config_toml, "appearance.mode")?);
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
        let (layout_source, layout) =
            active_layout(&state_dir, &appearance_mode, &bar_widgets, &shell_program)?;
        let mars_config_source = if config_home.join("mars/config.toml").is_file() {
            "user"
        } else {
            "packaged"
        };
        let zellij_sidecar = config_home.join("zellij/config.kdl");
        let zellij_plugins_sidecar = config_home.join("zellij/plugins.kdl");
        let zellij_config = PathBuf::from(trim_output(run_checked(
            &zellij_sidecar,
            Command::new(YZX_ZELLIJ_CONFIG)
                .arg(YZX_CONFIG_KDL)
                .arg(&zellij_sidecar)
                .arg(state_dir.join("zellij/config.kdl")),
        )?));
        let zellij_config_source = if zellij_config == PathBuf::from(YZX_CONFIG_KDL) {
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
            &managed_keybindings,
            &agent_popup_kdl,
            &custom_popups_kdl,
            &custom_popup_keybindings_kdl,
            &zellij_plugins_sidecar,
            &home_dir,
        )?;
        let zellij_status_cache = state_dir.join("zellij/session/status_bar_cache.json");
        create_dir_all_checked(parent(&zellij_status_cache), &zellij_status_cache)?;
        seed_plugin_permissions(&state_dir.join(ZELLIJ_PERMISSIONS_FILE))?;

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
            welcome_enabled: trim_output(welcome_enabled),
            welcome_style: trim_output(welcome_style),
            welcome_duration_seconds: trim_output(welcome_duration_seconds),
            mars_config_source,
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
        if !MARS.is_empty() {
            command
                .env("MARS_CONFIG_HOME", self.config_home.join("mars"))
                .env("MARS_BASE_CONFIG_HOME", YZX_MARS_CONFIG);
        }
        if let Some(bridge_session_id) = &self.bridge_session_id {
            command.env("YAZELIX_HELIX_BRIDGE_SESSION_ID", bridge_session_id);
        }
    }

    pub(crate) fn project_mars_appearance(&self) -> Result<Option<String>, AppError> {
        let mars_config = self.config_home.join("mars/config.toml");
        let output = run_checked(
            &mars_config,
            Command::new(YZX_CONFIG)
                .arg("--project-mars-appearance")
                .env("YAZELIX_CONFIG_HOME", &self.config_home),
        )?;
        parse_mars_appearance_projection(&output)
            .map_err(|reason| startup(reason, mars_config.display(), 1))
    }

    pub(crate) fn yazi(&self) -> &YaziRuntime {
        self.yazi
            .as_ref()
            .expect("Yazi runtime was not prepared for this command")
    }

    pub(crate) fn mars_config(&self) -> String {
        if MARS.is_empty() && self.mars_config_source == "packaged" {
            return "not included".to_string();
        }
        let path = if self.mars_config_source == "user" {
            self.config_home.join("mars/config.toml")
        } else {
            Path::new(YZX_MARS_CONFIG).join("config.toml")
        };
        source_path(self.mars_config_source, path.display())
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

fn parse_mars_appearance_projection(output: &str) -> Result<Option<String>, String> {
    match output.trim() {
        "config" => Ok(None),
        "environment dark" => Ok(Some("dark".to_string())),
        "environment light" => Ok(Some("light".to_string())),
        _ => Err(format!(
            "yzx-config returned an invalid Mars appearance projection: {}",
            output.trim()
        )),
    }
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
    fn mars_appearance_projection_is_strict() {
        assert_eq!(parse_mars_appearance_projection("config\n"), Ok(None));
        assert_eq!(
            parse_mars_appearance_projection("environment light\n"),
            Ok(Some("light".to_string()))
        );
        assert!(parse_mars_appearance_projection("environment auto").is_err());
        assert!(parse_mars_appearance_projection("environment  light").is_err());
        assert!(parse_mars_appearance_projection("config extra").is_err());
    }
}
