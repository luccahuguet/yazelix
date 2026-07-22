use std::{env, fmt::Display, fs, path::Path};

use crate::{
    AGENT_AUTO_COMMAND, HELIX_REVEAL_COMMAND, LAYOUT, LAYOUT_SWAP_TEMPLATE, LAYOUT_TEMPLATE,
    MANAGED_HELIX, MARS, PACKAGE_VARIANT, YAZELIX_ZELLIJ_BAR_WASM,
    YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM, YAZELIX_ZELLIJ_POPUP_WASM, YAZI_SOURCE,
    YAZI_TESTED_VERSION, YZX_BAR_RENDER, YZX_BAR_RENDER_REQUEST, YZX_CONFIG, YZX_CONFIG_KDL,
    YZX_CONFIG_UI, YZX_HELIX, YZX_MENU, YZX_REVEAL, YZX_SCREEN, YZX_SIDEBAR_REFRESH, YZX_TUTOR,
    YZX_WELCOME, YZX_YAZI, YZX_ZELLIJ_CONFIG, ZELLIJ,
    command::executable_file,
    error::{AppError, path_error, startup},
    paths::{runtime_path, zellij_session_label},
    runtime::Runtime,
    yazi::YaziRuntime,
};

pub(crate) fn print_doctor() -> Result<(), AppError> {
    let runtime = Runtime::prepare().map_err(doctor_failure)?;
    let yazi = YaziRuntime::resolve().map_err(doctor_failure)?;
    let has_managed_helix = MANAGED_HELIX == "included";
    check_doctor_inputs().map_err(doctor_failure)?;
    require_command("editor", &runtime.editor).map_err(doctor_failure)?;
    if runtime.agent_command != AGENT_AUTO_COMMAND {
        require_command("agent.command", &runtime.agent_command).map_err(doctor_failure)?;
    }

    println!("Yazelix Nova doctor");
    doctor_ok("config home", runtime.config_home.display());
    doctor_ok("state dir", runtime.state_dir.display());
    doctor_ok("shell.program", &runtime.shell_program);
    if !has_managed_helix && runtime.editor == YZX_HELIX {
        println!(
            "warn editor.command: {} is unavailable in package {}; set editor.command to an installed editor",
            runtime.editor_command, PACKAGE_VARIANT
        );
    } else {
        doctor_ok("editor.command", &runtime.editor_command);
        doctor_ok("editor", &runtime.editor);
    }
    doctor_ok("agent.command", &runtime.agent_command);
    doctor_ok("agent.args", &runtime.agent_args);
    doctor_ok("open.log_level", &runtime.yzx_open_log);
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
    for binding in &runtime.managed_keybindings {
        doctor_ok(binding.path, &binding.configured);
    }
    doctor_ok("zellij status cache", runtime.zellij_status_cache.display());
    doctor_ok("zellij permissions", runtime.zellij_permissions.display());
    doctor_ok("layout", runtime.layout());
    doctor_ok("config helper", YZX_CONFIG);
    doctor_ok("tutor helper", YZX_TUTOR);
    doctor_ok("screen helper", YZX_SCREEN);
    doctor_ok("welcome helper", YZX_WELCOME);
    doctor_ok("zellij helper", YZX_ZELLIJ_CONFIG);
    doctor_ok("reveal helper", YZX_REVEAL);
    doctor_ok("sidebar refresh helper", YZX_SIDEBAR_REFRESH);
    doctor_ok("yazi source", YAZI_SOURCE);
    doctor_ok("yazi lookup PATH", yazi.lookup_path.to_string_lossy());
    doctor_ok("yazi", yazi.yazi.display());
    doctor_ok("ya", yazi.ya.display());
    doctor_ok("yazi version", &yazi.version);
    doctor_ok("yazi tested version", YAZI_TESTED_VERSION);
    if let Some(warning) = &yazi.warning {
        println!("warn yazi compatibility: {warning}");
    }
    doctor_ok("zellij", ZELLIJ);
    doctor_ok(
        "mars",
        if MARS.is_empty() {
            "not included"
        } else {
            MARS
        },
    );
    doctor_ok("yazi opener", YZX_YAZI);
    doctor_ok(
        "pane orchestrator plugin",
        YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM,
    );
    if has_managed_helix {
        doctor_helix_config_warning(&runtime.config_home).map_err(doctor_failure)?;
    }

    println!(
        "warn session: {}",
        zellij_session_label("already inside zellij", "not inside zellij")
    );
    Ok(())
}

fn doctor_failure(error: AppError) -> AppError {
    println!("Yazelix Nova doctor");
    if let AppError::Startup { reason, check, .. } = &error {
        for reason in reason.lines() {
            println!("fail runtime preflight: {reason}");
        }
        if !check.is_empty() {
            println!("check: {check}");
        }
    }
    error
}

fn check_doctor_inputs() -> Result<(), AppError> {
    let current_exe = env::current_exe().map_err(|error| {
        startup(
            format!("failed to resolve current yzx executable: {error}"),
            "yzx",
            1,
        )
    })?;
    for (label, path) in [
        ("front door", current_exe.as_path()),
        ("config UI", Path::new(YZX_CONFIG_UI)),
        ("menu helper", Path::new(YZX_MENU)),
        ("tutor helper", Path::new(YZX_TUTOR)),
        ("screen helper", Path::new(YZX_SCREEN)),
        ("welcome helper", Path::new(YZX_WELCOME)),
        ("config helper", Path::new(YZX_CONFIG)),
        ("zellij config helper", Path::new(YZX_ZELLIJ_CONFIG)),
        ("reveal helper", Path::new(YZX_REVEAL)),
        ("sidebar refresh helper", Path::new(YZX_SIDEBAR_REFRESH)),
        ("packaged Zellij config", Path::new(YZX_CONFIG_KDL)),
        ("Zellij", Path::new(ZELLIJ)),
        ("layout", Path::new(LAYOUT)),
        ("layout template", Path::new(LAYOUT_TEMPLATE)),
        ("layout swap template", Path::new(LAYOUT_SWAP_TEMPLATE)),
        ("bar render request", Path::new(YZX_BAR_RENDER_REQUEST)),
        ("bar renderer", Path::new(YZX_BAR_RENDER)),
        ("managed editor", Path::new(YZX_HELIX)),
        ("Yazi opener", Path::new(YZX_YAZI)),
        ("popup plugin", Path::new(YAZELIX_ZELLIJ_POPUP_WASM)),
        ("bar plugin", Path::new(YAZELIX_ZELLIJ_BAR_WASM)),
        (
            "pane orchestrator plugin",
            Path::new(YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM),
        ),
    ] {
        require_file(label, path)?;
    }
    if !MARS.is_empty() {
        require_file("Mars", Path::new(MARS))?;
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
    let exists = if command.as_bytes().contains(&b'/') {
        executable_file(Path::new(command))
    } else {
        env::split_paths(&path).any(|dir| executable_file(&dir.join(command)))
    };
    if exists {
        return Ok(());
    }
    Err(startup(
        format!("{label} command not found: {command}"),
        command,
        1,
    ))
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
    let escaped_command = HELIX_REVEAL_COMMAND.replace('"', "\\\"");
    if text.contains("A-r")
        && !text.contains(HELIX_REVEAL_COMMAND)
        && !text.contains(&escaped_command)
    {
        println!(
            "warn helix config: helix config override sets reserved Alt r; generated config keeps '{HELIX_REVEAL_COMMAND}' ({})",
            config.display()
        );
    }
    Ok(())
}
