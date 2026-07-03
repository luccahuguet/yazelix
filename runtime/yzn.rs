use std::{
    env,
    ffi::{OsStr, OsString},
    fmt::Display,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{self, Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const YZN_CONFIG_UI: &str = "@yznConfigUi@";
const YZN_MENU: &str = "@yznMenu@";
const YZN_TUTOR: &str = "@yznTutor@";
const YZN_SCREEN: &str = "@yznScreen@";
const YZN_WELCOME: &str = "@yznWelcome@";
const YZN_SHELL: &str = "@yznShell@";
const YZN_ENV_SUPERVISOR: &str = "@yznEnvSupervisor@";
const ZELLIJ: &str = "@zellij@";
const MARS: &str = "@mars@";
const LAYOUT: &str = "@layout@";
const LAYOUT_TEMPLATE: &str = "@layoutTemplate@";
const LAYOUT_SWAP_TEMPLATE: &str = "@layoutSwapTemplate@";
const YZN_YAZI: &str = "@yznYazi@";
const YZN_HELIX: &str = "@yznHelix@";
const YZN_CONFIG: &str = "@yznConfig@";
const YZN_MARS_CONFIG: &str = "@yznMarsConfig@";
const YZN_ZELLIJ_CONFIG: &str = "@yznZellijConfig@";
const YZN_CONFIG_KDL: &str = "@yznConfigKdl@";
const YZN_REVEAL: &str = "@yznReveal@";
const YZN_YA: &str = "@yznYa@";
const YZN_BAR_RENDER_REQUEST: &str = "@yznBarRenderRequest@";
const YZN_BAR_RENDER: &str = "@yznBarRender@";
const YAZELIX_ZELLIJ_POPUP_WASM: &str = "@yazelixZellijPopupWasm@";
const YAZELIX_ZELLIJ_BAR_WASM: &str = "@yazelixZellijBarWasm@";
const YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM: &str = "@yazelixZellijPaneOrchestratorWasm@";
const DEFAULT_BAR_WIDGETS_JSON: &str = r#"@defaultBarWidgetsJson@"#;
const DEFAULT_POPUP_SIDE_MARGIN: &str = "@defaultPopupSideMargin@";
const DEFAULT_POPUP_VERTICAL_MARGIN: &str = "@defaultPopupVerticalMargin@";
const PATH_PREFIX: &str = "@pathPrefix@";
const SPONSOR_URL: &str = "https://github.com/sponsors/luccahuguet";
const ZELLIJ_HOME_PLACEHOLDER: &str = "\"__YZN_HOME__\"";
const LAYOUT_YAZI_PLACEHOLDER: &str = concat!("@", "yazi", "@");
const LAYOUT_BAR_PLACEHOLDER: &str = concat!("@", "bar", "@");
const HELIX_REVEAL_COMMAND: &str = r#":sh yzn reveal "%{buffer_name}""#;
const POPUP_KEYBINDING_SPECS: &[(&str, &str, &str)] = &[
    ("config", "keybindings.config", "@defaultConfigKeybinding@"),
    ("agent", "keybindings.agent", "@defaultAgentKeybinding@"),
    (
        "lazygit",
        "keybindings.lazygit",
        "@defaultLazygitKeybinding@",
    ),
    ("menu", "keybindings.menu", "@defaultMenuKeybinding@"),
];

fn main() {
    process::exit(run().map(|()| 0).unwrap_or_else(AppError::report));
}

fn run() -> Result<(), AppError> {
    let mut raw_args = env::args_os().skip(1);
    let command = raw_args.next().unwrap_or_else(|| OsString::from("launch"));
    let args = raw_args.collect::<Vec<_>>();

    match command.to_string_lossy().as_ref() {
        "help" | "-h" | "--help" => {
            print!("{HELP}");
            Ok(())
        }
        "config" => {
            expect_no_args("config", &args)?;
            exec_plain(YZN_CONFIG_UI)
        }
        "menu" => {
            expect_no_args("menu", &args)?;
            exec_menu()
        }
        "tutor" => exec_tutor(args),
        "screen" => exec_screen(args),
        "doctor" => {
            expect_no_args("doctor", &args)?;
            print_doctor()
        }
        "status" => {
            expect_no_args("status", &args)?;
            print_status()
        }
        "sponsor" => {
            expect_no_args("sponsor", &args)?;
            open_sponsor();
            Ok(())
        }
        "env" => {
            expect_no_args("env", &args)?;
            exec_env()
        }
        "reveal" => exec_reveal(args),
        "enter" => exec_managed(false, args),
        "launch" => exec_managed(true, args),
        unknown => Err(AppError::Usage(format!(
            "yzn: unknown command: {unknown}\n\n{HELP}"
        ))),
    }
}

fn expect_no_args(command: &str, args: &[OsString]) -> Result<(), AppError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(AppError::Usage(format!(
            "yzn {command} does not accept arguments yet\n"
        )))
    }
}

fn exec_plain(program: &str) -> Result<(), AppError> {
    let mut command = Command::new(program);
    command.env("PATH", runtime_path());
    exec(command, program)
}

fn exec_menu() -> Result<(), AppError> {
    let mut command = Command::new(YZN_MENU);
    command.env("PATH", runtime_path());
    if let Ok(current_exe) = env::current_exe() {
        command.env("YZN_MENU_YZN", current_exe);
    }
    exec(command, "yzn menu")
}

fn exec_tutor(args: Vec<OsString>) -> Result<(), AppError> {
    let mut command = Command::new(YZN_TUTOR);
    command.args(args).env("PATH", runtime_path());
    exec(command, "yzn tutor")
}

fn exec_env() -> Result<(), AppError> {
    let runtime = Runtime::prepare()?;
    let mut command = Command::new(YZN_ENV_SUPERVISOR);
    command.arg(YZN_SHELL);
    runtime.apply(&mut command);
    exec(command, "yzn env")
}

fn exec_reveal(args: Vec<OsString>) -> Result<(), AppError> {
    let mut command = Command::new(YZN_REVEAL);
    command
        .args(args)
        .env("YZN_YA", YZN_YA)
        .env("YZN_ZELLIJ", ZELLIJ)
        .env("PATH", runtime_path());
    exec(command, "yzn reveal")
}

fn exec_screen(args: Vec<OsString>) -> Result<(), AppError> {
    let mut command = Command::new(YZN_SCREEN);
    command
        .args(args)
        .env("YAZELIX_SCREEN_COMMAND_NAME", "yzn screen")
        .env("PATH", runtime_path());
    exec(command, "yzn screen")
}

fn exec_managed(through_mars: bool, zellij_args: Vec<OsString>) -> Result<(), AppError> {
    let runtime = Runtime::prepare()?;
    let program = if through_mars { MARS } else { YZN_WELCOME };
    let mut command = Command::new(program);
    if through_mars {
        command.arg("-e").arg(YZN_WELCOME).arg(ZELLIJ);
    } else {
        command.arg(ZELLIJ);
    }
    command
        .arg("--config")
        .arg(&runtime.zellij_config)
        .arg("--new-session-with-layout")
        .arg(&runtime.layout)
        .args(zellij_args);
    runtime.apply(&mut command);
    command.env(
        "YAZELIX_SESSION_TERMINAL",
        if through_mars {
            nonempty_env("YAZELIX_SESSION_TERMINAL").unwrap_or_else(|| OsString::from("mars"))
        } else {
            enter_terminal_label()
        },
    );
    exec(command, program)
}

fn exec(mut command: Command, check: &str) -> Result<(), AppError> {
    Err(startup(
        format!("failed to exec {check}: {}", command.exec()),
        check,
        1,
    ))
}

struct Runtime {
    config_home: PathBuf,
    state_dir: PathBuf,
    bridge_session_id: Option<OsString>,
    yzn_open_log: String,
    shell_program: String,
    editor_command: String,
    editor: String,
    welcome_enabled: String,
    welcome_style: String,
    welcome_duration_seconds: String,
    mars_config_source: &'static str,
    mars_config_home: PathBuf,
    zellij_sidecar: PathBuf,
    zellij_config: PathBuf,
    zellij_config_source: &'static str,
    layout: PathBuf,
    layout_source: &'static str,
    bar_widgets: String,
    popup_side_margin: String,
    popup_vertical_margin: String,
    popup_keybindings: Vec<PopupKeybinding>,
    zellij_status_cache: PathBuf,
    zellij_permissions: PathBuf,
}

struct PopupKeybinding {
    label: &'static str,
    path: &'static str,
    default: &'static str,
    configured: String,
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
    fn prepare() -> Result<Self, AppError> {
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
        let (layout_source, layout) = active_layout(&state_dir, &bar_widgets)?;
        let user_mars_config_home = config_home.join("mars");
        let (mars_config_source, mars_config_home) =
            if user_mars_config_home.join("config.toml").is_file() {
                ("user", user_mars_config_home)
            } else {
                ("packaged", PathBuf::from(YZN_MARS_CONFIG))
            };
        let zellij_sidecar = config_home.join("zellij/config.kdl");
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

    fn apply(&self, command: &mut Command) {
        let yzn_menu_yzn = env::current_exe().unwrap_or_else(|_| PathBuf::from("yzn"));
        command
            .env("YAZELIX_NEXT_CONFIG_HOME", &self.config_home)
            .env("YAZELIX_STATE_DIR", &self.state_dir)
            .env("YAZELIX_NEXT_EDITOR", &self.editor)
            .env("EDITOR", &self.editor)
            .env("VISUAL", &self.editor)
            .env("YZN_EDITOR", &self.editor)
            .env("GIT_EDITOR", &self.editor)
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
            .env("PATH", runtime_path());
        if let Some(bridge_session_id) = &self.bridge_session_id {
            command.env("YAZELIX_HELIX_BRIDGE_SESSION_ID", bridge_session_id);
        }
    }

    fn mars_config(&self) -> String {
        source_path(
            self.mars_config_source,
            self.mars_config_home.join("config.toml").display(),
        )
    }

    fn zellij_config(&self) -> String {
        source_path(self.zellij_config_source, self.zellij_config.display())
    }

    fn layout(&self) -> String {
        source_path(self.layout_source, self.layout.display())
    }
}

fn source_path(source: &str, path: impl Display) -> String {
    format!("{source} ({path})")
}

fn print_status() -> Result<(), AppError> {
    let runtime = Runtime::prepare()?;
    println!("Yazelix status");
    println!("config home: {}", runtime.config_home.display());
    println!("state dir: {}", runtime.state_dir.display());
    println!("shell: {}", runtime.shell_program);
    println!("editor command: {}", runtime.editor_command);
    println!("editor: {}", runtime.editor);
    println!("open log: {}", runtime.yzn_open_log);
    println!("welcome enabled: {}", runtime.welcome_enabled);
    println!("welcome style: {}", runtime.welcome_style);
    println!("welcome duration: {}s", runtime.welcome_duration_seconds);
    println!("mars config: {}", runtime.mars_config());
    println!("zellij config: {}", runtime.zellij_config());
    println!("zellij sidecar: {}", runtime.zellij_sidecar.display());
    println!("bar widgets: {}", runtime.bar_widgets);
    println!("popup side margin: {}", runtime.popup_side_margin);
    println!("popup vertical margin: {}", runtime.popup_vertical_margin);
    for binding in &runtime.popup_keybindings {
        println!("{} keybinding: {}", binding.label, binding.configured);
    }
    println!("layout: {}", runtime.layout());
    println!("inside zellij: {}", zellij_session_label("yes", "no"));
    Ok(())
}

fn print_doctor() -> Result<(), AppError> {
    let runtime = Runtime::prepare().map_err(doctor_failure)?;
    check_doctor_inputs().map_err(doctor_failure)?;
    require_command("editor", &runtime.editor).map_err(doctor_failure)?;

    println!("Yazelix doctor");
    doctor_ok("config home", runtime.config_home.display());
    doctor_ok("state dir", runtime.state_dir.display());
    doctor_ok("shell.program", &runtime.shell_program);
    doctor_ok("editor.command", &runtime.editor_command);
    doctor_ok("editor", &runtime.editor);
    doctor_ok("open.log_level", &runtime.yzn_open_log);
    doctor_ok("welcome.enabled", &runtime.welcome_enabled);
    doctor_ok("welcome.style", &runtime.welcome_style);
    doctor_ok(
        "welcome.duration_seconds",
        &runtime.welcome_duration_seconds,
    );
    doctor_ok("mars config", runtime.mars_config());
    doctor_ok("zellij config", runtime.zellij_config());
    doctor_ok("zellij sidecar", runtime.zellij_sidecar.display());
    doctor_ok("bar.widgets", &runtime.bar_widgets);
    doctor_ok("popup.side_margin", &runtime.popup_side_margin);
    doctor_ok("popup.vertical_margin", &runtime.popup_vertical_margin);
    for binding in &runtime.popup_keybindings {
        doctor_ok(binding.path, &binding.configured);
    }
    doctor_ok("zellij status cache", runtime.zellij_status_cache.display());
    doctor_ok("zellij permissions", runtime.zellij_permissions.display());
    doctor_ok("layout", runtime.layout());
    doctor_ok("config helper", YZN_CONFIG);
    doctor_ok("tutor helper", YZN_TUTOR);
    doctor_ok("screen helper", YZN_SCREEN);
    doctor_ok("welcome helper", YZN_WELCOME);
    doctor_ok("zellij helper", YZN_ZELLIJ_CONFIG);
    doctor_ok("reveal helper", YZN_REVEAL);
    doctor_ok("yazi cli", YZN_YA);
    doctor_ok("zellij", ZELLIJ);
    doctor_ok("mars", MARS);
    doctor_ok("yazi opener", YZN_YAZI);
    doctor_ok(
        "pane orchestrator plugin",
        YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM,
    );
    doctor_helix_config_warning(&runtime.config_home).map_err(doctor_failure)?;

    println!(
        "warn session: {}",
        zellij_session_label("already inside zellij", "not inside zellij")
    );
    Ok(())
}

fn doctor_failure(error: AppError) -> AppError {
    println!("Yazelix doctor");
    if let AppError::Startup { reason, check, .. } = &error {
        let reason = reason.lines().next().unwrap_or("startup check failed");
        println!("fail runtime preflight: {reason}");
        if !check.is_empty() {
            println!("check: {check}");
        }
    }
    error
}

fn check_doctor_inputs() -> Result<(), AppError> {
    let current_exe = env::current_exe().map_err(|error| {
        startup(
            format!("failed to resolve current yzn executable: {error}"),
            "yzn",
            1,
        )
    })?;
    for (label, path) in [
        ("front door", current_exe.as_path()),
        ("config UI", Path::new(YZN_CONFIG_UI)),
        ("menu helper", Path::new(YZN_MENU)),
        ("tutor helper", Path::new(YZN_TUTOR)),
        ("screen helper", Path::new(YZN_SCREEN)),
        ("welcome helper", Path::new(YZN_WELCOME)),
        ("config helper", Path::new(YZN_CONFIG)),
        ("zellij config helper", Path::new(YZN_ZELLIJ_CONFIG)),
        ("reveal helper", Path::new(YZN_REVEAL)),
        ("yazi cli", Path::new(YZN_YA)),
        ("packaged Zellij config", Path::new(YZN_CONFIG_KDL)),
        ("Zellij", Path::new(ZELLIJ)),
        ("Mars", Path::new(MARS)),
        ("layout", Path::new(LAYOUT)),
        ("layout template", Path::new(LAYOUT_TEMPLATE)),
        ("layout swap template", Path::new(LAYOUT_SWAP_TEMPLATE)),
        ("bar render request", Path::new(YZN_BAR_RENDER_REQUEST)),
        ("bar renderer", Path::new(YZN_BAR_RENDER)),
        ("managed editor", Path::new(YZN_HELIX)),
        ("Yazi opener", Path::new(YZN_YAZI)),
        ("popup plugin", Path::new(YAZELIX_ZELLIJ_POPUP_WASM)),
        ("bar plugin", Path::new(YAZELIX_ZELLIJ_BAR_WASM)),
        (
            "pane orchestrator plugin",
            Path::new(YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM),
        ),
    ] {
        require_file(label, path)?;
    }

    Ok(())
}

fn require_file(label: &str, path: &Path) -> Result<(), AppError> {
    if path.is_file() {
        Ok(())
    } else {
        Err(startup(
            format!("{label} is missing: {}", path.display()),
            path.display(),
            1,
        ))
    }
}

fn require_command(label: &str, command: &str) -> Result<(), AppError> {
    let path = runtime_path();
    if command_exists(OsStr::new(command), Some(path.as_os_str())) {
        return Ok(());
    }
    Err(startup(
        format!("{label} command not found: {command}"),
        command,
        1,
    ))
}

fn command_exists(command: &OsStr, path: Option<&OsStr>) -> bool {
    if command.as_bytes().contains(&b'/') {
        return executable_file(Path::new(command));
    }
    path.into_iter()
        .flat_map(env::split_paths)
        .any(|dir| executable_file(&dir.join(command)))
}

fn executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

fn doctor_ok(label: &str, value: impl Display) {
    println!("ok {label}: {value}");
}

fn doctor_helix_config_warning(config_home: &Path) -> Result<(), AppError> {
    let config = config_home.join("helix/config.toml");
    if !config.is_file() {
        return Ok(());
    }

    let text =
        fs::read_to_string(&config).map_err(|error| path_error("read", &config, &config, error))?;
    if !text.contains(HELIX_REVEAL_COMMAND) {
        println!(
            "warn helix config: helix config override exists without the '{HELIX_REVEAL_COMMAND}' configuration ({})",
            config.display()
        );
    }
    Ok(())
}

fn zellij_session_label(inside: &'static str, outside: &'static str) -> &'static str {
    if nonempty_env("ZELLIJ_SESSION_NAME").is_some() {
        inside
    } else {
        outside
    }
}

fn open_sponsor() {
    for opener in ["xdg-open", "open"] {
        if Command::new(opener)
            .arg(SPONSOR_URL)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
        {
            return;
        }
    }
    println!("{SPONSOR_URL}");
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

fn active_layout(state_dir: &Path, bar_widgets: &str) -> Result<(&'static str, PathBuf), AppError> {
    if bar_widgets == DEFAULT_BAR_WIDGETS_JSON {
        return Ok(("packaged", PathBuf::from(LAYOUT)));
    }

    let layout = state_dir.join("zellij/layout.kdl");
    let plugin_block = render_bar_plugin_block(bar_widgets)?;
    materialize_layout(&layout, &plugin_block)?;
    Ok(("runtime", layout))
}

fn active_zellij_config(
    state_dir: &Path,
    source: &'static str,
    config: PathBuf,
    layout: &Path,
    popup_side_margin: &str,
    popup_vertical_margin: &str,
    popup_keybindings: &[PopupKeybinding],
    home_dir: &Path,
) -> Result<(&'static str, PathBuf), AppError> {
    let runtime_config = state_dir.join("zellij/config.kdl");
    let text =
        fs::read_to_string(&config).map_err(|error| path_error("read", &config, &config, error))?;
    let mut patched = text;
    let replaced = patched.replace(ZELLIJ_HOME_PLACEHOLDER, &kdl_string(home_dir.display()));
    if replaced == patched {
        return Err(startup(
            "Zellij config is missing the managed home cwd placeholder",
            config.display(),
            1,
        ));
    }
    patched = replaced;
    if layout != Path::new(LAYOUT) {
        let replaced = patched.replace(LAYOUT, &layout.display().to_string());
        if replaced == patched {
            return Err(startup(
                "Zellij config is missing the packaged layout path",
                config.display(),
                1,
            ));
        }
        patched = replaced;
    }
    if popup_side_margin != DEFAULT_POPUP_SIDE_MARGIN
        || popup_vertical_margin != DEFAULT_POPUP_VERTICAL_MARGIN
    {
        let side_marker = format!("side_margin {DEFAULT_POPUP_SIDE_MARGIN}");
        let vertical_marker = format!("vertical_margin {DEFAULT_POPUP_VERTICAL_MARGIN}");
        if !patched.contains(&side_marker) || !patched.contains(&vertical_marker) {
            return Err(startup(
                "Zellij config is missing packaged popup geometry",
                config.display(),
                1,
            ));
        }
        let replaced = patched
            .replace(&side_marker, &format!("side_margin {popup_side_margin}"))
            .replace(
                &vertical_marker,
                &format!("vertical_margin {popup_vertical_margin}"),
            );
        patched = replaced;
    }
    patched = patch_popup_keybindings(patched, &config, popup_keybindings)?;
    create_dir_all_checked(parent(&runtime_config), &runtime_config)?;
    fs::write(&runtime_config, patched)
        .map_err(|error| path_error("write", &runtime_config, &runtime_config, error))?;
    Ok((
        if source == "sidecar" {
            "sidecar+runtime"
        } else {
            "runtime"
        },
        runtime_config,
    ))
}

fn patch_popup_keybindings(
    text: String,
    config: &Path,
    popup_keybindings: &[PopupKeybinding],
) -> Result<String, AppError> {
    let mut patched = text;
    for (index, binding) in popup_keybindings.iter().enumerate() {
        if binding.configured == binding.default {
            continue;
        }
        let marker = format!("bind {}", kdl_string(binding.default));
        if !patched.contains(&marker) {
            return Err(startup(
                format!(
                    "Zellij config is missing the packaged {} key binding",
                    binding.label
                ),
                config.display(),
                1,
            ));
        }
        patched = patched.replace(&marker, &format!("bind __YZN_POPUP_KEY_{index}__"));
    }
    for (index, binding) in popup_keybindings.iter().enumerate() {
        if binding.configured == binding.default {
            continue;
        }
        patched = patched.replace(
            &format!("__YZN_POPUP_KEY_{index}__"),
            &kdl_string(&binding.configured),
        );
    }
    Ok(patched)
}

fn render_bar_plugin_block(bar_widgets: &str) -> Result<String, AppError> {
    let template_path = Path::new(YZN_BAR_RENDER_REQUEST);
    let template = fs::read_to_string(template_path)
        .map_err(|error| path_error("read", template_path, template_path, error))?;
    let request = template.replace(r#""__YZN_BAR_WIDGET_TRAY__""#, bar_widgets);
    Ok(trim_output(run_checked(
        Path::new(YZN_BAR_RENDER),
        Command::new(YZN_BAR_RENDER).arg(request),
    )?))
}

fn materialize_layout(path: &Path, plugin_block: &str) -> Result<(), AppError> {
    let template_path = Path::new(LAYOUT_TEMPLATE);
    let swap_template_path = Path::new(LAYOUT_SWAP_TEMPLATE);
    let template = fs::read_to_string(template_path)
        .map_err(|error| path_error("read", template_path, template_path, error))?;
    let swap_template = fs::read_to_string(swap_template_path)
        .map_err(|error| path_error("read", swap_template_path, swap_template_path, error))?;
    let layout = template
        .replace(LAYOUT_YAZI_PLACEHOLDER, YZN_YAZI)
        .replace(LAYOUT_BAR_PLACEHOLDER, plugin_block);
    let swap_layout = swap_template.replace(LAYOUT_YAZI_PLACEHOLDER, YZN_YAZI);
    let swap_path = path.with_file_name("layout.swap.kdl");
    create_dir_all_checked(parent(path), path)?;
    fs::write(path, layout).map_err(|error| path_error("write", path, path, error))?;
    fs::write(&swap_path, swap_layout)
        .map_err(|error| path_error("write", &swap_path, &swap_path, error))
}

fn run_checked(check: &Path, command: &mut Command) -> Result<String, AppError> {
    match command.output() {
        Ok(output) if output.status.success() => Ok(String::from_utf8_lossy(&output.stdout).into()),
        Ok(output) => Err(startup(
            output_reason(&output).unwrap_or_else(|| {
                format!(
                    "{} failed with status {}",
                    command.get_program().to_string_lossy(),
                    output.status.code().unwrap_or(1)
                )
            }),
            check.display(),
            output.status.code().unwrap_or(1),
        )),
        Err(error) => Err(startup(
            format!(
                "failed to run {}: {error}",
                command.get_program().to_string_lossy()
            ),
            check.display(),
            1,
        )),
    }
}

fn output_reason(output: &Output) -> Option<String> {
    let trimmed = trim_output(format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ));
    (!trimmed.is_empty()).then_some(trimmed)
}

fn create_dir_all_checked(path: &Path, check: &Path) -> Result<(), AppError> {
    fs::create_dir_all(path).map_err(|error| path_error("create", path, check, error))
}

fn touch_checked(path: &Path) -> Result<(), AppError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(|_| ())
        .map_err(|error| path_error("create", path, path, error))
}

fn seed_permission_checked(
    path: &Path,
    plugin: &str,
    permissions: &[&str],
) -> Result<(), AppError> {
    let current =
        fs::read_to_string(path).map_err(|error| path_error("read", path, path, error))?;
    if current.contains(&format!("\"{plugin}\" {{")) {
        return Ok(());
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|error| path_error("open", path, path, error))?;
    writeln!(
        file,
        "\"{plugin}\" {{\n    {}\n}}",
        permissions.join("\n    ")
    )
    .map_err(|error| path_error("write", path, path, error))
}

fn config_home() -> Result<PathBuf, AppError> {
    if let Some(path) = nonempty_env("YAZELIX_NEXT_CONFIG_HOME") {
        return Ok(path.into());
    }
    if let Some(path) = nonempty_env("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(path).join("yazelix-next"));
    }
    nonempty_env("HOME")
        .map(|path| PathBuf::from(path).join(".config/yazelix-next"))
        .ok_or_else(|| {
            startup(
                "HOME is required when YAZELIX_NEXT_CONFIG_HOME and XDG_CONFIG_HOME are unset.",
                "",
                1,
            )
        })
}

fn home_dir() -> Result<PathBuf, AppError> {
    nonempty_env("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| startup("HOME is required to scope home-marker new tabs.", "", 1))
}

fn state_dir() -> PathBuf {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix-next"))
        })
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix-next"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/yazelix-next"))
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

fn enter_terminal_label() -> OsString {
    nonempty_env("YAZELIX_SESSION_TERMINAL")
        .or_else(|| nonempty_env("TERM_PROGRAM"))
        .or_else(|| nonempty_env("TERM"))
        .unwrap_or_else(|| OsString::from("unknown"))
}

fn runtime_path() -> OsString {
    match nonempty_env("PATH") {
        Some(path) => {
            let mut merged = OsString::from(PATH_PREFIX);
            merged.push(":");
            merged.push(path);
            merged
        }
        None => PATH_PREFIX.into(),
    }
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}

fn parent(path: &Path) -> &Path {
    path.parent().unwrap_or_else(|| Path::new("."))
}

fn trim_output(text: String) -> String {
    text.trim_end_matches(['\n', '\r']).to_owned()
}

fn kdl_string(value: impl Display) -> String {
    format!("{:?}", value.to_string())
}

enum AppError {
    Usage(String),
    Startup {
        reason: String,
        check: String,
        status: i32,
    },
}

fn startup(reason: impl Into<String>, check: impl Display, status: i32) -> AppError {
    AppError::Startup {
        reason: reason.into(),
        check: check.to_string(),
        status,
    }
}

fn path_error(action: &str, path: &Path, check: &Path, error: impl Display) -> AppError {
    startup(
        format!("failed to {action} {}: {error}", path.display()),
        check.display(),
        1,
    )
}

impl AppError {
    fn report(self) -> i32 {
        match self {
            Self::Usage(message) => {
                eprint!("{message}");
                64
            }
            Self::Startup {
                reason,
                check,
                status,
            } => {
                eprintln!("Yazelix could not start.\n");
                eprintln!("Reason:");
                for line in reason.lines() {
                    eprintln!("  {line}");
                }
                if !check.is_empty() {
                    eprintln!("\nCheck:\n  {check}");
                }
                status
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helix_bridge_only_matches_packaged_yzn_hx() {
        assert!(uses_helix_bridge(YZN_HELIX));
        assert!(uses_helix_bridge("/nix/store/example/bin/yzn-hx"));
        assert!(uses_helix_bridge("yzn-hx"));
        assert!(!uses_helix_bridge("hx"));
        assert!(!uses_helix_bridge("nvim"));
    }
}

const HELP: &str = "Yazelix

Usage:
  yzn
  yzn help
  yzn config
  yzn doctor
  yzn env
  yzn enter [zellij-args...]
  yzn launch [zellij-args...]
  yzn menu
  yzn tutor [lesson]
  yzn reveal <target>
  yzn screen [style]
  yzn sponsor
  yzn status

Commands:
  config  Open Yazelix Next config
  doctor  Check Yazelix runtime setup
  env     Open the managed shell without launching the UI
  enter   Start Yazelix in the current terminal
  launch  Open Mars and start Yazelix
  menu    Open the Yazelix command palette
  tutor   Show the guided Yazelix tutor
  reveal  Reveal a file or directory in the managed Yazi sidebar
  screen  Show a Yazelix terminal screen
  sponsor Open the Yazelix sponsor page or print its URL
  status  Show Yazelix runtime status
  help    Show this help
";
