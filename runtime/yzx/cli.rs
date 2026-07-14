use std::{env, ffi::OsString, path::Path, process::Command};

use crate::{
    command::exec,
    doctor::print_doctor,
    error::AppError,
    paths::{enter_terminal_label, nonempty_env, runtime_path},
    runtime::Runtime,
    status::{print_status, print_status_json},
    MARS, VERSION, YZX_CONFIG_UI, YZX_ENV_SUPERVISOR, YZX_MENU, YZX_REVEAL, YZX_SCREEN, YZX_SHELL,
    YZX_TUTOR, YZX_WELCOME, YZX_YA, ZELLIJ,
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
    let runtime = Runtime::prepare()?;
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
    let runtime = Runtime::prepare()?;
    let mut command = if program == "ya" {
        Command::new(YZX_YA)
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
        .env("YZX_YA", YZX_YA)
        .env("YZX_ZELLIJ", ZELLIJ)
        .env("PATH", runtime_path());
    exec(command, "yzx reveal")
}

fn exec_screen(args: Vec<OsString>) -> Result<(), AppError> {
    let mut command = Command::new(YZX_SCREEN);
    command
        .args(args)
        .env("YAZELIX_SCREEN_COMMAND_NAME", "yzx screen")
        .env("PATH", runtime_path());
    exec(command, "yzx screen")
}

fn exec_managed(through_mars: bool, zellij_args: Vec<OsString>) -> Result<(), AppError> {
    let program = managed_program(through_mars, MARS)?;
    let runtime = Runtime::prepare()?;
    let mut command = Command::new(program);
    if through_mars {
        command.arg("-e").arg(YZX_WELCOME).arg(ZELLIJ);
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
    apply_mars_cursor_config(
        &mut command,
        through_mars,
        &runtime.config_home.join("cursors.toml"),
    );
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

fn managed_program(through_mars: bool, mars: &'static str) -> Result<&'static str, AppError> {
    match (through_mars, mars) {
        (true, "") => Err(AppError::Usage(
            "yzx launch is unavailable in the Mars-free runtime package; use yzx enter or install the complete Yazelix Nova package\n".to_string(),
        )),
        (true, mars) => Ok(mars),
        (false, _) => Ok(YZX_WELCOME),
    }
}

fn apply_mars_cursor_config(command: &mut Command, through_mars: bool, path: &Path) {
    if through_mars {
        command.env("YAZELIX_CURSOR_CONFIG", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_entry_respects_the_fixed_package_variant() {
        assert_eq!(managed_program(false, "").ok(), Some(YZX_WELCOME));
        assert!(matches!(managed_program(true, ""), Err(AppError::Usage(_))));
        assert_eq!(managed_program(true, MARS).ok(), Some(MARS));
        let path = Path::new("/tmp/cursors.toml");
        let mut launch = Command::new(MARS);
        apply_mars_cursor_config(&mut launch, true, path);
        assert_eq!(
            launch.get_envs().next(),
            Some(("YAZELIX_CURSOR_CONFIG".as_ref(), Some(path.as_os_str())))
        );
        let mut enter = Command::new(YZX_WELCOME);
        apply_mars_cursor_config(&mut enter, false, path);
        assert_eq!(enter.get_envs().next(), None);
    }
}

const HELP: &str = "Yazelix Nova

Usage:
  yzx
  yzx --version
  yzx help
  yzx config
  yzx doctor
  yzx env
  yzx enter [zellij-args...]
  yzx launch [zellij-args...]
  yzx menu
  yzx tutor [lesson]
  yzx reveal <target>
  yzx screen [style]
  yzx run <program> [args...]
  yzx status [--json]

Commands:
  config  Open Yazelix Nova config
  doctor  Check Yazelix runtime setup
  env     Open the managed shell without launching the UI
  enter   Start Yazelix in the current terminal
  launch  Open Mars and start Yazelix
  menu    Open the Yazelix Nova command palette
  tutor   Show the guided Yazelix Nova tutor
  reveal  Reveal a file or directory in the managed Yazi sidebar
  run     Run a command in the managed Yazelix environment
  screen  Show a Yazelix terminal screen
  status  Show Yazelix runtime status
  help    Show this help

Sponsor: https://github.com/sponsors/luccahuguet
";
