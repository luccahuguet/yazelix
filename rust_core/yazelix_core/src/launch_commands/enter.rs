use super::config_override::{
    config_override_extra_env, prepare_session_config_override, resolve_cli_config_override,
};
use super::process::command_status_with_overrides;
use super::resolve_requested_working_dir;
use super::sidebar_bootstrap_extra_env;
use crate::atomic_fs::is_executable_file;
use crate::bridge::{CoreError, ErrorClass};
use crate::command_metadata::{YzxExternBridgeSyncRequest, sync_yzx_extern_bridge};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, default_shell_from_config, expand_user_path,
    home_dir_from_env, load_normalized_config_for_control, read_yazelix_version_from_runtime,
    runtime_dir_from_env, runtime_env_request, runtime_materialization_plan_request_from_env,
    state_dir_from_env, zellij_default_shell_from_runtime,
};
use crate::front_door_render::{GameOfLifeCellStyle, play_welcome_style_with_appearance};
use crate::initializer_commands::generate_shell_initializers_for_env;
use crate::runtime_contract::evaluate_startup_working_dir_preflight;
use crate::runtime_env::compute_runtime_env;
use crate::runtime_materialization::{RuntimeArtifact, materialize_runtime_state};
use crate::session_config_snapshot::{
    SessionConfigSnapshotCreateRequest, write_session_config_snapshot_for_launch,
};
use crate::startup_facts::{StartupFactsData, compute_startup_facts_from_config};
use crate::startup_handoff::{
    StartupHandoffArtifact, StartupHandoffCaptureRequest, capture_startup_handoff_context,
};
use crate::terminal_variant::{SESSION_TERMINAL_ENV, current_session_terminal_label_from_env};
use crate::upgrade_summary::{current_release_headline, maybe_show_first_run_upgrade_summary};
use crossterm::event::{Event, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EnterArgs {
    path: Option<String>,
    config: Option<String>,
    with_overrides: Vec<String>,
    home: bool,
    verbose: bool,
    setup_only: bool,
    help: bool,
}

pub(super) fn run_enter(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_enter_args(args)?;
    if parsed.help {
        print_enter_help();
        return Ok(0);
    }

    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let inherited_config_override = config_override_from_env();
    let config_override = prepare_session_config_override(
        parsed
            .config
            .as_deref()
            .or(inherited_config_override.as_deref()),
        &parsed.with_overrides,
    )?;
    let mut extra_env = config_override_extra_env(config_override.as_deref());
    let normalized =
        load_normalized_config_for_control(&runtime_dir, &config_dir, config_override.as_deref())?;
    let default_shell = default_shell_from_config(&normalized);
    let session_terminal_label = current_session_terminal_label_from_env();
    let mut startup_facts = compute_startup_facts_from_config(&runtime_dir, &normalized)?;
    startup_facts.terminals = vec![session_terminal_label.clone()];

    if parsed.verbose {
        println!("🔍 start_yazelix: verbose mode enabled");
        println!("🔍 Startup runtime env computed");
    }

    if parsed.setup_only {
        println!("🔧 Setting up Yazelix generated environment files...");
        run_runtime_setup(&runtime_dir, &default_shell, false)?;
        println!("✅ Setup complete.");
        return Ok(0);
    }

    let requested_working_dir = resolve_requested_working_dir(parsed.path.as_deref(), parsed.home)?;
    let working_dir = resolve_startup_working_dir(requested_working_dir)?;

    run_runtime_setup(&runtime_dir, &default_shell, true)?;
    extra_env.extend(sidebar_bootstrap_extra_env("enter", &working_dir)?);
    show_startup_presentation(&runtime_dir, &startup_facts, parsed.verbose)?;

    let startup = prepare_rust_startup(
        &runtime_dir,
        &working_dir,
        config_override.as_deref(),
        &session_terminal_label,
        parsed.verbose,
    )?;
    extra_env.extend(startup.extra_env);

    if startup.profile_exit_before_zellij {
        return Ok(0);
    }

    let status = command_status_with_overrides(
        &startup.argv,
        Some(&startup.runtime_env),
        &working_dir,
        &[],
        &extra_env,
        "enter_startup",
        "Retry from a valid Yazelix runtime or relaunch with `yzx launch`.",
    )?;
    Ok(status.code().unwrap_or(1))
}

fn parse_enter_args(args: &[String]) -> Result<EnterArgs, CoreError> {
    let mut parsed = EnterArgs::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" | "help" => parsed.help = true,
            "--home" => parsed.home = true,
            "--verbose" => parsed.verbose = true,
            "--setup-only" => parsed.setup_only = true,
            "--path" | "-p" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage("Missing value for yzx enter --path. Try `yzx enter --help`.")
                })?;
                parsed.path = Some(value.clone());
            }
            "--config" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage(
                        "Missing value for yzx enter --config. Try `yzx enter --help`.",
                    )
                })?;
                if parsed.config.is_some() {
                    return Err(CoreError::usage(
                        "yzx enter accepts at most one --config override.",
                    ));
                }
                parsed.config = Some(resolve_cli_config_override(value)?);
            }
            "--with" => {
                index += 1;
                let value = args.get(index).ok_or_else(|| {
                    CoreError::usage("Missing value for yzx enter --with. Try `yzx enter --help`.")
                })?;
                parsed.with_overrides.push(value.clone());
            }
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx enter: {other}. Try `yzx enter --help`."
                )));
            }
            other => {
                if parsed.path.is_some() {
                    return Err(CoreError::usage(
                        "yzx enter accepts at most one positional cwd override.",
                    ));
                }
                parsed.path = Some(other.to_string());
            }
        }
        index += 1;
    }
    Ok(parsed)
}

fn print_enter_help() {
    println!("Start Yazelix in the current terminal");
    println!();
    println!("Usage:");
    println!(
        "  yzx enter [--path <dir> | --home] [--config <file>] [--with key=value] [--verbose]"
    );
}

fn show_startup_presentation(
    runtime_dir: &Path,
    facts: &StartupFactsData,
    verbose: bool,
) -> Result<(), CoreError> {
    let state_dir = state_dir_from_env()?;
    let log_dir = state_dir.join("logs");
    fs::create_dir_all(&log_dir).map_err(|source| {
        CoreError::io(
            "startup_welcome_log_dir",
            format!("Cannot create Yazelix log directory {}.", log_dir.display()),
            "Fix permissions or set YAZELIX_LOGS_DIR to a writable path.",
            log_dir.display().to_string(),
            source,
        )
    })?;

    let env_only = bool_env("YAZELIX_ENV_ONLY");
    let should_skip =
        facts.skip_welcome_screen || env_only || bool_env("YAZELIX_STARTUP_PROFILE_SKIP_WELCOME");
    let runtime_version = read_yazelix_version_from_runtime(runtime_dir)?;
    let welcome_message = build_welcome_message(runtime_dir, &runtime_version, facts);

    if !should_skip {
        play_welcome_art(runtime_dir, facts)?;
        print_welcome_message(&welcome_message)?;
    } else if env_only {
        println!(
            "🔧 Yazelix environment loaded! Launch the full interface in a separate terminal with 'yzx launch' or here with 'yzx enter'."
        );
    } else {
        let log_path = write_welcome_log(&log_dir, &welcome_message)?;
        println!(
            "💡 Welcome screen skipped. Welcome info logged to: {}",
            log_path.display()
        );
    }

    show_first_run_upgrade_summary(runtime_dir, &state_dir, &runtime_version, verbose);
    Ok(())
}

fn play_welcome_art(_runtime_dir: &Path, facts: &StartupFactsData) -> Result<(), CoreError> {
    let duration = Duration::from_millis((facts.welcome_duration_seconds.max(0.0) * 1000.0) as u64);
    let cell_style = GameOfLifeCellStyle::parse(&facts.game_of_life_cell_style).map_err(|err| {
        CoreError::classified(
            ErrorClass::Usage,
            "invalid_game_of_life_cell_style",
            format!("Invalid Game of Life cell style `{}`.", err.normalized()),
            "Use `full_block` or `dotted`.",
            serde_json::json!({ "style": err.normalized() }),
        )
    })?;
    play_welcome_style_with_appearance(
        &facts.welcome_style,
        duration,
        cell_style,
        &facts.appearance_mode,
    )?;
    if facts.show_macchina_on_welcome {
        let status = Command::new("macchina")
            .args([
                "-o",
                "machine",
                "-o",
                "distribution",
                "-o",
                "desktop-environment",
                "-o",
                "processor",
                "-o",
                "gpu",
                "-o",
                "terminal",
            ])
            .status()
            .map_err(|source| {
                CoreError::io(
                    "startup_macchina",
                    "Could not run macchina for the Yazelix welcome screen.",
                    "Set welcome.enabled = false or reinstall Yazelix with macchina available.",
                    "macchina",
                    source,
                )
            })?;
        if !status.success() {
            return Err(CoreError::classified(
                ErrorClass::Runtime,
                "startup_macchina_failed",
                format!("macchina failed during the Yazelix welcome screen with status {status}."),
                "Set welcome.enabled = false or fix macchina, then retry.",
                serde_json::json!({}),
            ));
        }
    }
    Ok(())
}

fn print_welcome_message(lines: &[String]) -> Result<(), CoreError> {
    for line in lines {
        println!("{line}");
    }
    if io::stdin().is_terminal() {
        print!("Press any key to launch Zellij and start your session... ");
        io::stdout().flush().map_err(|source| {
            CoreError::io(
                "startup_welcome_flush",
                "Could not flush the Yazelix welcome prompt.",
                "Retry the launch.",
                "<stdout>",
                source,
            )
        })?;
        wait_for_welcome_keypress()?;
        println!();
    }
    println!("Launching Zellij...");
    Ok(())
}

struct WelcomeRawModeGuard {
    active: bool,
}

impl WelcomeRawModeGuard {
    fn enable() -> Result<Self, CoreError> {
        enable_raw_mode().map_err(|source| {
            CoreError::io(
                "startup_welcome_raw_mode",
                "Could not enter terminal raw mode for the Yazelix welcome prompt.",
                "Run Yazelix in an interactive terminal that supports raw key input, or set welcome.enabled = false.",
                ".",
                source,
            )
        })?;
        Ok(Self { active: true })
    }

    fn disable(&mut self) -> Result<(), CoreError> {
        if !self.active {
            return Ok(());
        }
        disable_raw_mode().map_err(|source| {
            CoreError::io(
                "startup_welcome_raw_mode_restore",
                "Could not restore terminal mode after the Yazelix welcome prompt.",
                "Reset the terminal, then retry the launch.",
                ".",
                source,
            )
        })?;
        self.active = false;
        Ok(())
    }
}

impl Drop for WelcomeRawModeGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = disable_raw_mode();
        }
    }
}

fn wait_for_welcome_keypress() -> Result<(), CoreError> {
    let mut raw_mode = WelcomeRawModeGuard::enable()?;
    wait_for_welcome_keypress_from_events(|| {
        crossterm::event::read().map_err(|source| {
            CoreError::io(
                "startup_welcome_key_read",
                "Could not read the Yazelix welcome prompt key.",
                "Retry in an interactive terminal that supports raw key input, or set welcome.enabled = false.",
                ".",
                source,
            )
        })
    })?;
    raw_mode.disable()
}

fn wait_for_welcome_keypress_from_events(
    mut read_event: impl FnMut() -> Result<Event, CoreError>,
) -> Result<(), CoreError> {
    loop {
        let event = read_event()?;
        if is_welcome_keypress(&event) {
            return Ok(());
        }
    }
}

fn is_welcome_keypress(event: &Event) -> bool {
    matches!(event, Event::Key(key) if key.kind == KeyEventKind::Press)
}

fn show_first_run_upgrade_summary(
    runtime_dir: &Path,
    state_dir: &Path,
    runtime_version: &str,
    verbose: bool,
) {
    match maybe_show_first_run_upgrade_summary(runtime_dir, state_dir, runtime_version) {
        Ok(result) if result.shown && !result.report.output.trim().is_empty() => {
            println!("{}", result.report.output);
            println!();
        }
        Err(error) if verbose => {
            eprintln!("⚠️ Failed to render upgrade summary: {}", error.message())
        }
        _ => {}
    }
}

fn build_welcome_message(
    runtime_dir: &Path,
    runtime_version: &str,
    facts: &StartupFactsData,
) -> Vec<String> {
    let plain = use_plain_welcome_message(facts);
    let mut lines = vec![
        String::new(),
        if plain {
            format!("Welcome: Yazelix {runtime_version}")
        } else {
            format!("🎉 Welcome to Yazelix {runtime_version}!")
        },
    ];
    let headline = current_release_headline(runtime_dir, runtime_version).unwrap_or_default();
    if !headline.trim().is_empty() {
        lines.push(headline);
    }
    lines.extend([
        flake_last_updated_line(runtime_dir, plain),
        if plain {
            "Now: Nix auto-setup, lazygit, Starship, and markdown-oxide".to_string()
        } else {
            "✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide".to_string()
        },
        if plain {
            "Session: creating new Zellij session".to_string()
        } else {
            "🆕 Creating new Zellij session".to_string()
        },
        format!(
            "{}{}",
            if plain {
                "Terminal: preferred host terminal: "
            } else {
                "🖥️  Preferred host terminal: "
            },
            facts
                .terminals
                .first()
                .map(String::as_str)
                .unwrap_or("unknown")
        ),
        if plain {
            "First run: Yazelix pre-seeds bundled Zellij plugin permissions before launch. If Zellij still prompts, answer yes; troubleshooting covers cache-reset recovery.".to_string()
        } else {
            "⚠️  First run: Yazelix pre-seeds bundled Zellij plugin permissions before launch. If Zellij still prompts, answer yes; troubleshooting covers cache-reset recovery.".to_string()
        },
        if plain {
            "Quick tips: Use Alt+Shift+H/J/K/L for the left sidebar, bottom popup, top popup, and right sidebar; use Ctrl+Y and Ctrl+Shift+Y for sidebar/editor focus".to_string()
        } else {
            "💡 Quick tips: Use Alt+Shift+H/J/K/L for the left sidebar, bottom popup, top popup, and right sidebar; use Ctrl+Y and Ctrl+Shift+Y for sidebar/editor focus".to_string()
        },
    ]);
    lines
}

fn use_plain_welcome_message(facts: &StartupFactsData) -> bool {
    facts
        .terminals
        .first()
        .is_some_and(|terminal| terminal == "rio")
}

fn flake_last_updated_line(runtime_dir: &Path, plain: bool) -> String {
    let days = runtime_dir
        .join("flake.nix")
        .metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .map(|duration| duration.as_secs() / 86_400);
    match (plain, days) {
        (true, Some(days)) => format!("Flake: last updated {days} day(s) ago"),
        (true, None) => "Flake: last updated unknown".to_string(),
        (false, Some(days)) => format!("🕒 Flake last updated: {days} day(s) ago"),
        (false, None) => "🕒 Flake last updated: unknown".to_string(),
    }
}

fn write_welcome_log(log_dir: &Path, lines: &[String]) -> Result<PathBuf, CoreError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|source| {
            CoreError::classified(
                ErrorClass::Internal,
                "startup_welcome_log_clock",
                format!("System clock error while preparing welcome log path: {source}"),
                "Fix the system clock, then retry.",
                serde_json::json!({}),
            )
        })?
        .as_secs();
    let path = log_dir.join(format!("welcome_{timestamp}.log"));
    fs::write(&path, lines.join("\n")).map_err(|source| {
        CoreError::io(
            "startup_welcome_log_write",
            "Could not write Yazelix welcome log.",
            "Fix permissions for the Yazelix log directory and retry.",
            path.display().to_string(),
            source,
        )
    })?;
    Ok(path)
}

struct RustStartupPlan {
    argv: Vec<String>,
    runtime_env: JsonMap<String, JsonValue>,
    extra_env: Vec<(String, Option<String>)>,
    profile_exit_before_zellij: bool,
}

fn resolve_startup_working_dir(requested_working_dir: PathBuf) -> Result<PathBuf, CoreError> {
    evaluate_startup_working_dir_preflight(requested_working_dir)
}

fn prepare_rust_startup(
    runtime_dir: &Path,
    working_dir: &Path,
    config_override: Option<&str>,
    session_terminal_label: &str,
    verbose: bool,
) -> Result<RustStartupPlan, CoreError> {
    let state_dir = state_dir_from_env()?;
    let mut materialization_request =
        runtime_materialization_plan_request_from_env(config_override)?;
    materialization_request.session_terminal_label = Some(session_terminal_label.to_string());
    let materialization = materialize_runtime_state(&materialization_request)?;
    if verbose && materialization.plan.status != "noop" {
        println!("✅ Generated runtime state materialized.");
    }

    let runtime_version = read_yazelix_version_from_runtime(runtime_dir)?;
    let snapshot = write_session_config_snapshot_for_launch(
        &SessionConfigSnapshotCreateRequest {
            state_dir: state_dir.clone(),
            snapshot_id: launch_snapshot_id()?,
            source_config_file: materialization.plan.config_state.config_file.clone(),
            source_config_hash: materialization.plan.config_state.config_hash.clone(),
            runtime_dir: runtime_dir.to_path_buf(),
            runtime_hash: materialization.plan.config_state.runtime_hash.clone(),
            normalized_config: materialization.plan.config_state.config.clone(),
        },
        &runtime_version,
    )?;
    let status_bar_cache_path = status_bar_cache_path_for_snapshot(&snapshot.snapshot_path)?;

    let runtime_env = compute_runtime_env(&runtime_env_request(
        runtime_dir.to_path_buf(),
        &materialization.plan.config_state.config,
    )?)?
    .runtime_env;
    let default_shell = default_shell_from_config(&materialization.plan.config_state.config);
    let zellij_default_shell = zellij_default_shell_from_runtime(runtime_dir, &default_shell);
    let zellij_config_dir =
        require_existing_directory(&materialization.plan.zellij_config_dir, "Zellij config dir")?;
    let layout_path =
        require_existing_file(&materialization.plan.zellij_layout_path, "Zellij layout")?;

    capture_startup_handoff(
        &state_dir,
        working_dir,
        &zellij_config_dir,
        &layout_path,
        &zellij_default_shell,
        &materialization.plan.status,
        &materialization.plan.reason,
        materialization.plan.should_regenerate,
        materialization.plan.should_sync_static_assets,
        &materialization.plan.missing_artifacts,
        verbose,
    );

    let mut argv = vec!["zellij".to_string()];
    if let Some(session_name) = zellij_session_name_from_env() {
        argv.extend(["--session".to_string(), session_name]);
    }
    argv.extend([
        "--config-dir".to_string(),
        zellij_config_dir.clone(),
        "options".to_string(),
        "--default-cwd".to_string(),
        working_dir.to_string_lossy().into_owned(),
        "--default-layout".to_string(),
        layout_path,
        "--default-shell".to_string(),
        zellij_default_shell,
    ]);

    Ok(RustStartupPlan {
        argv,
        runtime_env,
        extra_env: vec![
            (
                "YAZELIX_SESSION_CONFIG_PATH".to_string(),
                Some(snapshot.snapshot_path),
            ),
            (
                "YAZELIX_STATUS_BAR_CACHE_PATH".to_string(),
                Some(status_bar_cache_path),
            ),
            (
                SESSION_TERMINAL_ENV.to_string(),
                Some(session_terminal_label.to_string()),
            ),
        ],
        profile_exit_before_zellij: bool_env("YAZELIX_STARTUP_PROFILE_EXIT_BEFORE_ZELLIJ"),
    })
}

fn launch_snapshot_id() -> Result<String, CoreError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .map_err(|source| {
            CoreError::classified(
                ErrorClass::Internal,
                "startup_snapshot_clock",
                format!("System clock error while preparing a Yazelix session snapshot: {source}"),
                "Fix the system clock, then retry.",
                serde_json::json!({}),
            )
        })
}

fn status_bar_cache_path_for_snapshot(snapshot_path: &str) -> Result<String, CoreError> {
    let snapshot_path = Path::new(snapshot_path);
    let session_dir = snapshot_path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "startup_snapshot_path_parent",
            format!(
                "Yazelix session snapshot path has no parent directory: {}.",
                snapshot_path.display()
            ),
            "Report this as a Yazelix internal error.",
            serde_json::json!({ "snapshot_path": snapshot_path.to_string_lossy() }),
        )
    })?;
    Ok(session_dir
        .join("status_bar_cache.json")
        .to_string_lossy()
        .to_string())
}

fn require_existing_directory(path: &str, label: &str) -> Result<String, CoreError> {
    let path = PathBuf::from(path);
    if path.is_dir() {
        return Ok(path.to_string_lossy().to_string());
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "startup_missing_generated_directory",
        format!(
            "{label} is missing after Yazelix startup materialization: {}.",
            path.display()
        ),
        "Run `yzx doctor` to inspect the generated-state contract, then retry.",
        serde_json::json!({ "path": path.to_string_lossy() }),
    ))
}

fn require_existing_file(path: &str, label: &str) -> Result<String, CoreError> {
    let path = PathBuf::from(path);
    if path.is_file() {
        return Ok(path.to_string_lossy().to_string());
    }
    Err(CoreError::classified(
        ErrorClass::Runtime,
        "startup_missing_generated_file",
        format!(
            "{label} is missing after Yazelix startup materialization: {}.",
            path.display()
        ),
        "Run `yzx doctor` to inspect the generated-state contract, then retry.",
        serde_json::json!({ "path": path.to_string_lossy() }),
    ))
}

#[cfg(test)]
mod tests {
    // Test lane: default

    use super::{
        build_welcome_message, is_welcome_keypress, wait_for_welcome_keypress_from_events,
    };
    use crate::startup_facts::StartupFactsData;
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
    use std::collections::VecDeque;

    fn startup_facts_for_terminal(terminal: &str) -> StartupFactsData {
        StartupFactsData {
            default_shell: "nu".to_string(),
            debug_mode: false,
            skip_welcome_screen: false,
            welcome_style: "static".to_string(),
            game_of_life_cell_style: "full_block".to_string(),
            appearance_mode: "dark".to_string(),
            welcome_duration_seconds: 0.0,
            show_macchina_on_welcome: false,
            terminals: vec![terminal.to_string()],
        }
    }

    // Regression: vanilla Rio keeps plain startup copy until its emoji fallback renders reliably.
    #[test]
    fn rio_welcome_message_uses_plain_terminal_stable_copy() {
        let runtime_dir = tempfile::tempdir().unwrap();
        let message = build_welcome_message(
            runtime_dir.path(),
            "v-test",
            &startup_facts_for_terminal("rio"),
        )
        .join("\n");

        assert!(message.is_ascii(), "{message}");
        assert!(message.contains("Welcome: Yazelix v-test"));
        assert!(message.contains("Flake: last updated unknown"));
        assert!(message.contains("Terminal: preferred host terminal: rio"));
        assert!(!message.contains("🎉 Welcome to Yazelix v-test!"));
        assert!(!message.contains("🖥️  Preferred host terminal: rio"));
    }

    // Defends: Mars and capable terminals keep the richer emoji welcome copy instead of inheriting Rio's fallback.
    #[test]
    fn non_rio_welcome_messages_keep_rich_copy() {
        let runtime_dir = tempfile::tempdir().unwrap();
        for terminal in ["mars", "ghostty", "wezterm"] {
            let message = build_welcome_message(
                runtime_dir.path(),
                "v-test",
                &startup_facts_for_terminal(terminal),
            )
            .join("\n");

            assert!(message.contains("🎉 Welcome to Yazelix v-test!"));
            assert!(message.contains("🕒 Flake last updated: unknown"));
            assert!(
                message
                    .contains("✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide")
            );
            assert!(message.contains("🆕 Creating new Zellij session"));
            assert!(message.contains(&format!("🖥️  Preferred host terminal: {terminal}")));
            assert!(message.contains("⚠️  First run:"));
            assert!(message.contains("💡 Quick tips:"));
            assert!(!message.contains("Welcome: Yazelix v-test"));
            assert!(!message.contains("Flake: last updated unknown"));
            assert!(!message.contains("Terminal: preferred host terminal:"));
        }
    }

    // Defends: the welcome prompt's "any key" contract accepts a non-Enter key without waiting for a newline.
    #[test]
    fn welcome_prompt_continues_on_non_enter_key_press() {
        let mut events = VecDeque::from([
            Event::Resize(80, 24),
            Event::Key(KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Repeat,
                state: KeyEventState::NONE,
            }),
            Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE)),
        ]);

        wait_for_welcome_keypress_from_events(|| {
            Ok(events
                .pop_front()
                .expect("welcome prompt should stop after the key press"))
        })
        .unwrap();

        assert!(events.is_empty());
    }

    // Defends: non-key and non-press terminal events do not accidentally pass the welcome prompt.
    #[test]
    fn welcome_prompt_ignores_non_press_terminal_events() {
        let repeat = Event::Key(KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Repeat,
            state: KeyEventState::NONE,
        });

        assert!(!is_welcome_keypress(&repeat));
        assert!(!is_welcome_keypress(&Event::Resize(80, 24)));
    }
}

fn capture_startup_handoff(
    state_dir: &Path,
    working_dir: &Path,
    zellij_config_dir: &str,
    layout_path: &str,
    default_shell: &str,
    materialization_status: &str,
    materialization_reason: &str,
    materialization_should_regenerate: bool,
    materialization_should_sync_static_assets: bool,
    missing_artifacts: &[RuntimeArtifact],
    verbose: bool,
) {
    let result = capture_startup_handoff_context(&StartupHandoffCaptureRequest {
        state_dir: state_dir.to_path_buf(),
        working_dir: working_dir.to_string_lossy().to_string(),
        session_default_cwd: working_dir.to_string_lossy().to_string(),
        launch_process_cwd: working_dir.to_string_lossy().to_string(),
        zellij_config_dir: zellij_config_dir.to_string(),
        layout_path: layout_path.to_string(),
        default_shell: default_shell.to_string(),
        materialization_status: materialization_status.to_string(),
        materialization_reason: materialization_reason.to_string(),
        materialization_should_regenerate,
        materialization_should_sync_static_assets,
        missing_artifacts: missing_artifacts
            .iter()
            .map(|artifact| StartupHandoffArtifact {
                label: artifact.label.clone(),
                path: artifact.path.clone(),
            })
            .collect(),
    });

    match result {
        Ok(capture) if verbose && capture.recorded => {
            if let Some(path) = capture.context_path.or(capture.latest_path) {
                println!("📝 Startup handoff context: {path}");
            }
        }
        Err(error) => eprintln!(
            "⚠️ Failed to write startup handoff context: {}",
            error.message()
        ),
        _ => {}
    }
}

fn zellij_session_name_from_env() -> Option<String> {
    std::env::var("YAZELIX_ZELLIJ_SESSION_NAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn bool_env(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|value| value.trim() == "true")
}

fn run_runtime_setup(
    runtime_dir: &Path,
    default_shell: &str,
    quiet: bool,
) -> Result<(), CoreError> {
    ensure_default_shell_available(default_shell)?;

    let state_dir = state_dir_from_env()?;
    let log_dir = setup_log_dir(&state_dir)?;
    fs::create_dir_all(&state_dir).map_err(|source| {
        CoreError::io(
            "runtime_setup_state_dir",
            format!(
                "Cannot create Yazelix state directory {}.",
                state_dir.display()
            ),
            "Fix permissions or set YAZELIX_STATE_DIR to a writable path.",
            state_dir.display().to_string(),
            source,
        )
    })?;
    fs::create_dir_all(&log_dir).map_err(|source| {
        CoreError::io(
            "runtime_setup_log_dir",
            format!("Cannot create Yazelix log directory {}.", log_dir.display()),
            "Fix permissions or set YAZELIX_LOGS_DIR to a writable path.",
            log_dir.display().to_string(),
            source,
        )
    })?;

    let shells_to_configure = setup_shells(default_shell);
    let initializer_run = generate_shell_initializers_for_env(&shells_to_configure, quiet)?;
    for message in initializer_run.messages {
        println!("{message}");
    }

    if let Err(error) = sync_yzx_extern_bridge(&YzxExternBridgeSyncRequest {
        runtime_dir: runtime_dir.to_path_buf(),
        state_dir: state_dir.clone(),
    }) {
        eprintln!(
            "⚠️  Could not sync generated yzx extern bridge: {}",
            error.message()
        );
    }

    let zjstatus_target = runtime_dir
        .join("configs")
        .join("zellij")
        .join("plugins")
        .join("zjstatus.wasm");
    if !zjstatus_target.is_file() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_zjstatus_plugin",
            format!(
                "Expected packaged zjstatus plugin at {}, but it was not found.",
                zjstatus_target.display()
            ),
            "Reinstall Yazelix or run from a packaged runtime that includes zjstatus.wasm.",
            serde_json::json!({"path": zjstatus_target.display().to_string()}),
        ));
    }

    Ok(())
}

fn ensure_default_shell_available(default_shell: &str) -> Result<(), CoreError> {
    let shell = default_shell.trim().to_lowercase();
    if shell != "xonsh" || command_available_on_path("xonsh") {
        return Ok(());
    }

    Err(CoreError::classified(
        ErrorClass::Runtime,
        "host_default_shell_missing",
        "The configured shell.program is unavailable on PATH.",
        "Choose nu, bash, fish, or zsh through shell.program.",
        serde_json::json!({
            "shell": "xonsh",
            "command": "xonsh",
        }),
    ))
}

fn command_available_on_path(command: &str) -> bool {
    std::env::var_os("PATH")
        .into_iter()
        .flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>())
        .any(|dir| is_executable_file(&dir.join(command)))
}

fn setup_shells(default_shell: &str) -> Vec<String> {
    let mut out = Vec::new();
    for shell in ["nu", "bash", default_shell] {
        let trimmed = shell.trim().to_lowercase();
        if !trimmed.is_empty() && !out.contains(&trimmed) {
            out.push(trimmed);
        }
    }
    out
}

fn setup_log_dir(state_dir: &Path) -> Result<PathBuf, CoreError> {
    let Some(raw) = std::env::var("YAZELIX_LOGS_DIR")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
    else {
        return Ok(state_dir.join("logs"));
    };
    Ok(expand_user_path(&raw, &home_dir_from_env()?))
}
