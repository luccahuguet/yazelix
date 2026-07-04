use std::{
    env,
    ffi::OsStr,
    fmt::Display,
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};

use crate::{
    error::{path_error, startup, AppError},
    paths::{runtime_path, zellij_session_label},
    runtime::Runtime,
    HELIX_REVEAL_COMMAND, LAYOUT, LAYOUT_SWAP_TEMPLATE, LAYOUT_TEMPLATE, MARS,
    YAZELIX_ZELLIJ_BAR_WASM, YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM, YAZELIX_ZELLIJ_POPUP_WASM,
    YZN_BAR_RENDER, YZN_BAR_RENDER_REQUEST, YZN_CONFIG, YZN_CONFIG_KDL, YZN_CONFIG_UI, YZN_HELIX,
    YZN_MENU, YZN_REVEAL, YZN_SCREEN, YZN_TUTOR, YZN_WELCOME, YZN_YA, YZN_YAZI, YZN_ZELLIJ_CONFIG,
    ZELLIJ,
};

pub(crate) fn print_doctor() -> Result<(), AppError> {
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
