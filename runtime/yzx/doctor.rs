use std::{
    env,
    fmt::Display,
    fs,
    io::{self, IsTerminal},
    path::Path,
    process::Command,
};

use crate::{
    command::executable_file,
    error::{path_error, startup, AppError},
    paths::{runtime_path, zellij_session_label},
    runtime::Runtime,
    yazi::YaziRuntime,
    AGENT_AUTO_COMMAND, HELIX_REVEAL_COMMAND, LAYOUT, LAYOUT_SWAP_TEMPLATE, LAYOUT_TEMPLATE,
    MANAGED_HELIX, NOVA_BAR_WASM, PACKAGE_VARIANT, RIO, YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM,
    YAZELIX_ZELLIJ_POPUP_WASM, YAZI_SOURCE, YZX_BAR_RENDER, YZX_BAR_RENDER_REQUEST, YZX_CONFIG,
    YZX_CONFIG_KDL, YZX_CONFIG_UI, YZX_HELIX, YZX_MENU, YZX_REVEAL, YZX_SCREEN, YZX_TUTOR,
    YZX_WELCOME, YZX_YAZI, YZX_ZELLIJ_CONFIG, ZELLIJ, ZJ_RADAR_WASM,
};

pub(crate) fn print_doctor(verbose: bool) -> Result<(), AppError> {
    let runtime = Runtime::prepare()?;
    let yazi = YaziRuntime::resolve()?;
    let has_managed_helix = MANAGED_HELIX == "included";
    check_doctor_inputs()?;
    require_command("editor", &runtime.editor)?;
    if runtime.agent_command != AGENT_AUTO_COMMAND {
        require_command("agent.command", &runtime.agent_command)?;
    }
    if !runtime.radar_enabled() {
        require_command("sidebar.command", &runtime.sidebar_command)?;
    }

    doctor_header();
    doctor_section("Core");
    doctor_ok("Configuration", "config and state directories ready");
    doctor_ok(
        "Commands",
        format!(
            "shell {} · editor {} · agent {}",
            runtime.shell_program, runtime.editor_command, runtime.agent_command
        ),
    );
    if !has_managed_helix && runtime.editor == YZX_HELIX {
        doctor_warn(
            "Editor",
            format!(
                "{} is unavailable in package {}; set editor.command to an installed editor",
                runtime.editor_command, PACKAGE_VARIANT
            ),
        );
    }
    doctor_ok(
        "Interface",
        format!(
            "{} keybindings · bar widgets configured",
            runtime.managed_keybindings.len()
        ),
    );

    doctor_section("Runtime");
    doctor_ok(
        "Configs",
        if RIO.is_empty() {
            "Zellij · layout ready · Rio omitted"
        } else {
            "Rio · Zellij · layout ready"
        },
    );
    doctor_ok("Yazi", format!("{} ({YAZI_SOURCE})", yazi.version));
    if let Some(warning) = &yazi.warning {
        doctor_warn("Yazi compatibility", warning);
    }
    doctor_ok("Components", "all packaged helpers and plugins found");

    doctor_section("Integrations");
    if runtime.radar_enabled() {
        doctor_radar_codex(&runtime.agent_command, verbose);
    }
    if has_managed_helix {
        doctor_helix_config_warning(&runtime.config_home)?;
    }

    doctor_section("Cleanup");
    let residue = classic_residue_lines(&runtime.config_home, &runtime.state_dir);
    if residue.is_empty() {
        doctor_ok("Classic residue", "none found");
    } else {
        doctor_warn(
            "Classic residue",
            format!(
                "{} unused paths ignored by Nova · details: yzx doctor --verbose",
                residue.len()
            ),
        );
        if verbose {
            for line in &residue {
                doctor_detail(line);
            }
            doctor_detail("external scripts may still reference these paths");
        }
    }
    doctor_info(
        "Session",
        zellij_session_label("inside Zellij", "outside Zellij"),
    );

    Ok(())
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
        ("anima helper", Path::new(YZX_SCREEN)),
        ("welcome helper", Path::new(YZX_WELCOME)),
        ("config helper", Path::new(YZX_CONFIG)),
        ("zellij config helper", Path::new(YZX_ZELLIJ_CONFIG)),
        ("reveal helper", Path::new(YZX_REVEAL)),
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
        ("bar plugin", Path::new(NOVA_BAR_WASM)),
        ("Radar plugin", Path::new(ZJ_RADAR_WASM)),
        (
            "pane orchestrator plugin",
            Path::new(YAZELIX_ZELLIJ_PANE_ORCHESTRATOR_WASM),
        ),
    ] {
        require_file(label, path)?;
    }
    if !RIO.is_empty() {
        require_file("Rio", Path::new(RIO))?;
    }
    require_command("Radar CLI", "zj-radar")?;

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
    if command_exists(command) {
        return Ok(());
    }
    Err(startup(
        format!("{label} command not found: {command}"),
        command,
        1,
    ))
}

fn command_exists(command: &str) -> bool {
    if command.as_bytes().contains(&b'/') {
        executable_file(Path::new(command))
    } else {
        env::split_paths(&runtime_path()).any(|dir| executable_file(&dir.join(command)))
    }
}

fn doctor_radar_codex(agent_command: &str, verbose: bool) {
    let configured_codex = (Path::new(agent_command)
        .file_name()
        .and_then(|name| name.to_str())
        == Some("codex")
        && command_exists(agent_command))
    .then_some(agent_command);
    let codex = configured_codex.or_else(|| command_exists("codex").then_some("codex"));
    let Some(codex) = codex else {
        doctor_info("Radar", "Codex not found; hook check skipped");
        return;
    };
    let mut path = runtime_path();
    if let Some(parent) = Path::new(codex)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        if !path.is_empty() {
            path.push(":");
        }
        path.push(parent);
    }
    let output = match Command::new("zj-radar")
        .args(["setup", "codex", "--check"])
        .env("PATH", path)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            doctor_warn("Radar", format!("zj-radar could not run: {error}"));
            return;
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines = stdout.lines().chain(stderr.lines()).collect::<Vec<_>>();
    let attention = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| line.starts_with("warn ") || line.starts_with("missing "))
        .collect::<Vec<_>>();
    if attention.is_empty() && output.status.success() {
        doctor_ok("Radar", "Codex hooks installed");
    } else {
        doctor_warn("Radar", "Codex hooks need attention");
        for line in &attention {
            doctor_detail(line);
        }
    }
    if !attention.is_empty() {
        doctor_detail("action: resolve the warning, then run yzx radar-setup");
    } else if !output.status.success() {
        doctor_detail("action: rerun with yzx doctor --verbose for zj-radar output");
    }
    if output.status.success()
        && lines
            .iter()
            .any(|line| line.trim().starts_with("note hook trust:"))
    {
        doctor_info("Codex trust", "review with /hooks");
    }
    if verbose {
        for line in lines {
            doctor_detail(&format!("radar: {line}"));
        }
    }
}

fn doctor_ok(label: &str, value: impl Display) {
    doctor_status("ok  ", "32", label, value);
}

fn doctor_warn(label: &str, value: impl Display) {
    doctor_status("warn", "33", label, value);
}

fn doctor_info(label: &str, value: impl Display) {
    doctor_status("info", "36", label, value);
}

fn doctor_status(status: &str, color: &str, label: &str, value: impl Display) {
    println!("  {}  {:<16} {value}", paint(status, color), label);
}

fn doctor_header() {
    println!("{}", paint("Yazelix Nova doctor", "1;35"));
}

fn doctor_section(name: &str) {
    println!("\n{}", paint(name, "1;36"));
}

fn doctor_detail(detail: &str) {
    println!("      {detail}");
}

fn paint(text: &str, color: &str) -> String {
    if io::stdout().is_terminal()
        && env::var_os("NO_COLOR").is_none()
        && env::var("TERM").is_ok_and(|term| term != "dumb")
    {
        format!("\x1b[{color}m{text}\x1b[0m")
    } else {
        text.into()
    }
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
        doctor_warn(
            "Helix config",
            format!(
                "override sets reserved Alt r; generated config keeps '{HELIX_REVEAL_COMMAND}' ({})",
                config.display()
            ),
        );
    }
    Ok(())
}

fn classic_residue_lines(config_home: &Path, state_dir: &Path) -> Vec<String> {
    let mut residue = Vec::new();
    for (relative, exact_generated_file) in [
        ("configs", false),
        ("sessions", false),
        ("initializers/nushell/yazelix_extern.nu", true),
        ("initializers/nushell/yazelix_extern.fingerprint.json", true),
    ] {
        let Some(metadata) = metadata_without_symlink_parents(state_dir, relative) else {
            continue;
        };
        let certain = exact_generated_file && metadata.is_file();
        residue.push(classic_residue_line(
            &state_dir.join(relative),
            if certain { "certain" } else { "ambiguous" },
        ));
    }

    if fs::symlink_metadata(config_home).is_ok_and(|metadata| metadata.is_dir()) {
        let mut backups = fs::read_dir(config_home)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_name().to_str().is_some_and(|name| {
                    [
                        "config.toml.backup-",
                        "settings.jsonc.backup-",
                        "zellij.kdl.backup-",
                        "config.toml.home-manager-prepare-backup-",
                    ]
                    .iter()
                    .any(|prefix| name.starts_with(prefix))
                })
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        backups.sort();
        residue.extend(
            backups
                .iter()
                .map(|path| classic_residue_line(path, "ambiguous")),
        );
    }

    residue
}

fn metadata_without_symlink_parents(root: &Path, relative: &str) -> Option<fs::Metadata> {
    if !fs::symlink_metadata(root).ok()?.is_dir() {
        return None;
    }

    let relative = Path::new(relative);
    let mut parent = root.to_path_buf();
    for component in relative.parent()?.components() {
        parent.push(component);
        if !fs::symlink_metadata(&parent).ok()?.is_dir() {
            return None;
        }
    }
    fs::symlink_metadata(root.join(relative)).ok()
}

fn classic_residue_line(path: &Path, ownership: &str) -> String {
    format!(
        "warn classic residue: ownership={ownership} nova=unused path={}",
        path.display()
    )
}
