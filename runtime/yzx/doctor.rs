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
    for line in classic_residue_lines(&runtime.config_home, &runtime.state_dir) {
        println!("{line}");
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
        let certain =
            exact_generated_file && !metadata.file_type().is_symlink() && metadata.is_file();
        residue.push(classic_residue_line(
            &state_dir.join(relative),
            if certain { "certain" } else { "ambiguous" },
        ));
    }

    if fs::symlink_metadata(config_home)
        .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
    {
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

    if residue.is_empty() {
        return vec!["ok classic residue: none recognized in active roots".into()];
    }
    residue.push(
        "warn classic residue: external scripts may still reference these paths; Nova did not load or modify them"
            .into(),
    );
    residue
}

fn metadata_without_symlink_parents(root: &Path, relative: &str) -> Option<fs::Metadata> {
    let root_metadata = fs::symlink_metadata(root).ok()?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return None;
    }

    let mut path = root.to_path_buf();
    let mut components = Path::new(relative).components().peekable();
    while let Some(component) = components.next() {
        path.push(component);
        let metadata = fs::symlink_metadata(&path).ok()?;
        if components.peek().is_none() {
            return Some(metadata);
        }
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return None;
        }
    }
    None
}

fn classic_residue_line(path: &Path, ownership: &str) -> String {
    format!(
        "warn classic residue: ownership={ownership} nova=unused path={}",
        path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        os::unix::fs::symlink,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn classifies_classic_residue_without_following_symlinks() {
        let root = env::temp_dir().join(format!(
            "yzx-doctor-residue-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let config_home = root.join("config");
        let state_dir = root.join("state");
        fs::create_dir_all(&config_home).unwrap();
        fs::create_dir_all(&state_dir).unwrap();

        assert_eq!(
            classic_residue_lines(&config_home, &state_dir),
            ["ok classic residue: none recognized in active roots"]
        );

        for current in ["yazi", "zellij", "helix", "helix-steel", "logs"] {
            fs::create_dir(state_dir.join(current)).unwrap();
        }
        let configs = state_dir.join("configs");
        let sessions = state_dir.join("sessions");
        fs::create_dir(&configs).unwrap();
        let outside_sessions = root.join("outside-sessions");
        fs::create_dir(&outside_sessions).unwrap();
        fs::write(outside_sessions.join("config_snapshot.json"), "untouched").unwrap();
        symlink(&outside_sessions, &sessions).unwrap();
        let nushell = state_dir.join("initializers/nushell");
        fs::create_dir_all(&nushell).unwrap();
        let extern_file = nushell.join("yazelix_extern.nu");
        let fingerprint = nushell.join("yazelix_extern.fingerprint.json");
        fs::write(&extern_file, "classic").unwrap();
        symlink(root.join("missing-fingerprint"), &fingerprint).unwrap();
        let config_backup = config_home.join("config.toml.backup-20260712");
        let settings_backup = config_home.join("settings.jsonc.backup-20260711");
        fs::write(&config_backup, "classic").unwrap();
        symlink(root.join("missing-backup"), &settings_backup).unwrap();
        fs::write(config_home.join("keep.txt"), "current").unwrap();

        assert_eq!(
            classic_residue_lines(&config_home, &state_dir),
            [
                classic_residue_line(&configs, "ambiguous"),
                classic_residue_line(&sessions, "ambiguous"),
                classic_residue_line(&extern_file, "certain"),
                classic_residue_line(&fingerprint, "ambiguous"),
                classic_residue_line(&config_backup, "ambiguous"),
                classic_residue_line(&settings_backup, "ambiguous"),
                "warn classic residue: external scripts may still reference these paths; Nova did not load or modify them".to_string(),
            ]
        );
        assert_eq!(
            fs::read_to_string(outside_sessions.join("config_snapshot.json")).unwrap(),
            "untouched"
        );

        let linked_state = root.join("linked-state");
        let linked_config = root.join("linked-config");
        fs::create_dir(&linked_state).unwrap();
        fs::create_dir(&linked_config).unwrap();
        symlink(
            state_dir.join("initializers"),
            linked_state.join("initializers"),
        )
        .unwrap();
        assert_eq!(
            classic_residue_lines(&linked_config, &linked_state),
            ["ok classic residue: none recognized in active roots"]
        );

        fs::remove_dir_all(root).unwrap();
    }
}
