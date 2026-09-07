use std::{env, ffi::OsString, path::Path, process::Command};

use crate::{
    command::{exec, run_checked, trim_output},
    doctor::print_doctor,
    error::{startup, AppError},
    paths::{enter_terminal_label, nonempty_env, runtime_path},
    runtime::{current_appearance_mode, Runtime},
    status::{print_status, print_status_json},
    RIO, VERSION, YZX_CONFIG, YZX_CONFIG_UI, YZX_ENV_SUPERVISOR, YZX_MENU, YZX_REVEAL, YZX_SCREEN,
    YZX_SHELL, YZX_TUTOR, YZX_WELCOME, YZX_YAZI, YZX_YAZI_CONFIG, YZX_YAZI_MATERIALIZER, ZELLIJ,
};

pub(crate) fn run() -> Result<(), AppError> {
    let mut raw_args = env::args_os().skip(1);
    let Some(command) = raw_args.next() else {
        print!("{HELP}");
        return Ok(());
    };
    let args = raw_args.collect::<Vec<_>>();

    match command.to_string_lossy().as_ref() {
        "help" | "-h" | "--help" => {
            print!("{HELP}");
            Ok(())
        }
        "--version" => {
            expect_no_args("--version", &args)?;
            println!("Yazelix Nova ({VERSION})");
            Ok(())
        }
        "config" => {
            expect_no_args("config", &args)?;
            exec_plain(YZX_CONFIG_UI)
        }
        "yazi-config" => exec_yazi_config(args),
        "radar-setup" => {
            expect_no_args("radar-setup", &args)?;
            let mut command = Command::new("zj-radar");
            command
                .args(["setup", "codex", "claude", "opencode"])
                .env("PATH", runtime_path());
            exec(command, "yzx radar-setup")
        }
        "menu" => {
            expect_no_args("menu", &args)?;
            exec_menu()
        }
        "tutor" => exec_tutor(args),
        "anima" => exec_anima(args),
        "doctor" => match args.as_slice() {
            [] => print_doctor(false),
            [flag] if flag == "--verbose" => print_doctor(true),
            _ => Err(AppError::Usage(
                "yzx doctor accepts only --verbose\n".to_string(),
            )),
        },
        "status" => match args.as_slice() {
            [] => print_status(),
            [flag] if flag == "--json" => print_status_json(),
            _ => Err(AppError::Usage(
                "yzx status accepts only --json\n".to_string(),
            )),
        },
        "env" => {
            expect_no_args("env", &args)?;
            exec_env()
        }
        "reveal" => exec_reveal(args),
        "run" => exec_run(args),
        "enter" => exec_managed(false, args),
        "launch" => exec_managed(true, args),
        unknown => Err(AppError::Usage(format!(
            "yzx: unknown command: {unknown}\n\n{HELP}"
        ))),
    }
}

fn expect_no_args(command: &str, args: &[OsString]) -> Result<(), AppError> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(AppError::Usage(format!(
            "yzx {command} does not accept arguments yet\n"
        )))
    }
}

fn exec_plain(program: &str) -> Result<(), AppError> {
    let mut command = Command::new(program);
    command.env("PATH", runtime_path());
    exec(command, program)
}

fn exec_yazi_config(args: Vec<OsString>) -> Result<(), AppError> {
    if args.is_empty() || matches!(args.as_slice(), [arg] if arg == "--help") {
        print!("{YAZI_CONFIG_HELP}");
        return Ok(());
    }
    match args.as_slice() {
        [command, arg] if command == "materialize" && arg == "--help" => {
            print!("{YAZI_CONFIG_MATERIALIZE_HELP}");
            Ok(())
        }
        [command, args @ ..] if command == "materialize" => {
            let Some((user_config_dir, state_dir)) = materialize_paths(args) else {
                return Err(AppError::Usage(YAZI_CONFIG_MATERIALIZE_HELP.to_string()));
            };
            let appearance_mode = current_appearance_mode(
                trim_output(run_checked(
                    Path::new(YZX_CONFIG),
                    Command::new(YZX_CONFIG).arg("--get").arg("appearance.mode"),
                )?),
                false,
            );
            let mut command = Command::new(YZX_YAZI_MATERIALIZER);
            command
                .arg(YZX_YAZI_CONFIG)
                .arg(user_config_dir)
                .arg(state_dir)
                .arg(appearance_mode);
            exec(command, "yzx yazi-config materialize")
        }
        _ => Err(AppError::Usage(YAZI_CONFIG_HELP.to_string())),
    }
}

fn materialize_paths(args: &[OsString]) -> Option<(&OsString, &OsString)> {
    let [first_flag, first_path, second_flag, second_path] = args else {
        return None;
    };
    if first_path.is_empty() || second_path.is_empty() {
        return None;
    }
    match (first_flag.to_str(), second_flag.to_str()) {
        (Some("--user-config-dir"), Some("--state-dir")) => Some((first_path, second_path)),
        (Some("--state-dir"), Some("--user-config-dir")) => Some((second_path, first_path)),
        _ => None,
    }
}

fn exec_menu() -> Result<(), AppError> {
    let mut command = Command::new(YZX_MENU);
    command.env("PATH", runtime_path());
    if let Ok(current_exe) = env::current_exe() {
        command.env("YZX_MENU_YZX", current_exe);
    }
    exec(command, "yzx menu")
}

fn exec_tutor(args: Vec<OsString>) -> Result<(), AppError> {
    let mut command = Command::new(YZX_TUTOR);
    command.args(args).env("PATH", runtime_path());
    exec(command, "yzx tutor")
}

fn exec_env() -> Result<(), AppError> {
    let runtime = Runtime::prepare_with_yazi()?;
    let mut command = Command::new(YZX_ENV_SUPERVISOR);
    command.arg(YZX_SHELL);
    runtime.apply(&mut command);
    exec(command, "yzx env")
}

fn exec_run(args: Vec<OsString>) -> Result<(), AppError> {
    let Some((program, args)) = args.split_first() else {
        return Err(AppError::Usage(
            "Usage: yzx run <program> [args...]\n".to_string(),
        ));
    };
    let needs_yazi = program == "ya" || program == "yazi";
    let runtime = if needs_yazi {
        Runtime::prepare_with_yazi()?
    } else {
        Runtime::prepare()?
    };
    let mut command = if program == "ya" {
        Command::new(&runtime.yazi().ya)
    } else if program == "yazi" {
        Command::new(YZX_YAZI)
    } else {
        Command::new(program)
    };
    command.args(args);
    runtime.apply(&mut command);
    exec(command, "yzx run")
}

fn exec_reveal(args: Vec<OsString>) -> Result<(), AppError> {
    let mut command = Command::new(YZX_REVEAL);
    command
        .args(args)
        .env("YZX_ZELLIJ", ZELLIJ)
        .env("PATH", runtime_path());
    exec(command, "yzx reveal")
}

fn exec_anima(args: Vec<OsString>) -> Result<(), AppError> {
    let mut command = Command::new(YZX_SCREEN);
    command
        .args(args)
        .env("YAZELIX_SCREEN_COMMAND_NAME", "yzx anima")
        .env("PATH", runtime_path());
    exec(command, "yzx anima")
}

fn exec_managed(graphical: bool, zellij_args: Vec<OsString>) -> Result<(), AppError> {
    let program = managed_program(graphical)?;
    let runtime = Runtime::prepare_new_session_with_yazi()?;
    let live_appearance = !RIO.is_empty() && project_rio_appearance(&runtime)?;
    let mut command = Command::new(program);
    if graphical {
        apply_rio_launch_appearance(&mut command, &runtime.appearance_mode, live_appearance);
        command.arg(YZX_WELCOME).arg(ZELLIJ);
    } else {
        command.arg(ZELLIJ);
    }
    apply_zellij_launch_theme_mode(&mut command, &runtime.appearance_mode);
    command
        .arg("--config")
        .arg(&runtime.zellij_config)
        .arg("--new-session-with-layout")
        .arg(&runtime.layout)
        .args(zellij_args);
    runtime.apply(&mut command);
    command
        .env("YZX_APPEARANCE_MODE", &runtime.appearance_mode)
        .env(
            "YZX_APPEARANCE_LIVE",
            if live_appearance { "1" } else { "0" },
        );
    if graphical {
        command.env(
            "RIO_CONFIG_HOME",
            runtime.rio_config.parent().expect("Rio config directory"),
        );
    }
    command.env(
        "YAZELIX_SESSION_TERMINAL",
        if graphical {
            nonempty_env("YAZELIX_SESSION_TERMINAL").unwrap_or_else(|| OsString::from("rio"))
        } else {
            enter_terminal_label()
        },
    );
    exec(command, program)
}

fn managed_program(graphical: bool) -> Result<&'static str, AppError> {
    match (graphical, RIO.is_empty()) {
        (true, true) => Err(AppError::Usage(
            "yzx launch is unavailable because this package omits Rio; use yzx enter or select a package that includes Rio\n".to_string(),
        )),
        (true, false) => Ok(RIO),
        (false, _) => Ok(YZX_WELCOME),
    }
}

fn project_rio_appearance(runtime: &Runtime) -> Result<bool, AppError> {
    let result = trim_output(run_checked(
        &runtime.rio_config,
        Command::new(YZX_CONFIG)
            .arg("--project-rio-appearance")
            .arg(&runtime.appearance_mode)
            .env("YAZELIX_CONFIG_HOME", &runtime.config_home),
    )?);
    match result.as_str() {
        "live" => Ok(true),
        "next-launch" => Ok(false),
        _ => Err(startup(
            format!("yzx-config returned an unknown Rio appearance capability: {result}"),
            runtime.rio_config.display(),
            1,
        )),
    }
}

fn apply_rio_launch_appearance(command: &mut Command, mode: &str, live: bool) {
    command.args(["--app-id", "yzx"]);
    if !live {
        command.args(["--theme-mode", mode]);
    }
    command.arg("-e");
}

fn apply_zellij_launch_theme_mode(command: &mut Command, mode: &str) {
    command.arg("--theme-mode").arg(mode);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_launch_theme_modes_are_explicit() {
        let mut zellij = Command::new(ZELLIJ);
        apply_zellij_launch_theme_mode(&mut zellij, "light");
        assert_eq!(
            zellij
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            ["--theme-mode", "light"]
        );

        let mut rio = Command::new(RIO);
        apply_rio_launch_appearance(&mut rio, "light", false);
        assert_eq!(
            rio.get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            ["--app-id", "yzx", "--theme-mode", "light", "-e"]
        );

        let mut live_rio = Command::new(RIO);
        apply_rio_launch_appearance(&mut live_rio, "light", true);
        assert_eq!(
            live_rio
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            ["--app-id", "yzx", "-e"]
        );
    }

    #[test]
    fn yazi_config_materialize_requires_each_named_path_once() {
        let user = OsString::from("/user/yazi");
        let state = OsString::from("/state/yazelix");
        let args = [
            OsString::from("--state-dir"),
            state.clone(),
            OsString::from("--user-config-dir"),
            user.clone(),
        ];
        assert_eq!(materialize_paths(&args), Some((&user, &state)));

        for args in [
            vec![OsString::from("--user-config-dir"), user.clone()],
            vec![
                OsString::from("--user-config-dir"),
                user.clone(),
                OsString::from("--user-config-dir"),
                user.clone(),
            ],
            vec![
                OsString::from("--unknown"),
                user.clone(),
                OsString::from("--state-dir"),
                state.clone(),
            ],
        ] {
            assert_eq!(materialize_paths(&args), None);
        }
    }
}

const HELP: &str = "Yazelix Nova

Usage:
  yzx
  yzx --version
  yzx help
  yzx config
  yzx yazi-config materialize --user-config-dir <path> --state-dir <path>
  yzx doctor [--verbose]
  yzx radar-setup
  yzx env
  yzx enter [zellij-args...]
  yzx launch [zellij-args...]
  yzx menu
  yzx tutor [lesson]
  yzx reveal <target>
  yzx anima [style]
  yzx run <program> [args...]
  yzx status [--json]

Commands:
  config  Open Yazelix Nova config
  yazi-config  Materialize the effective Yazi configuration
  doctor  Check Yazelix runtime setup
  radar-setup  Set up agent activity in Radar
  env     Open the managed shell without launching the UI
  enter   Start Yazelix in the current terminal
  launch  Open Rio and start Yazelix
  menu    Open the Yazelix Nova command palette
  tutor   Show the guided Yazelix Nova tutor
  reveal  Reveal a file or directory in the persistent Yazi popup
  run     Run a command in the managed Yazelix environment
  anima   Show a Yazelix terminal animation
  status  Show Yazelix runtime status
  help    Show this help

Agent activity:
  yzx radar-setup   Set up Codex, Claude Code and OpenCode interactively
  Radar asks before changes; unavailable agents are skipped.
  Start a fresh agent session afterward; review Codex hook trust with /hooks.

Sessions:
  yzx enter --session NAME   Start a fresh named session in this terminal
  yzx launch --session NAME  Start a fresh named session in Rio
  yzx enter attach NAME      Attach to a live named session in this terminal
  yzx launch attach NAME     Attach to a live named session in Rio

Sponsor: https://github.com/sponsors/luccahuguet
";

const YAZI_CONFIG_HELP: &str = "Materialize Yazelix Nova's effective Yazi configuration

Usage:
  yzx yazi-config materialize --user-config-dir <path> --state-dir <path>

Commands:
  materialize  Build and print the effective Yazi config directory
";

const YAZI_CONFIG_MATERIALIZE_HELP: &str =
    "Usage: yzx yazi-config materialize --user-config-dir <path> --state-dir <path>

Options:
  --user-config-dir <path>  Exact directory containing the user's Yazi configuration
  --state-dir <path>        State directory in which to materialize the effective configuration
";
