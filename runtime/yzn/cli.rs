use std::{env, ffi::OsString, path::Path, process::Command};

use crate::{
    MARS, VERSION, YZN_CONFIG_UI, YZN_ENV_SUPERVISOR, YZN_MENU, YZN_REVEAL, YZN_SCREEN, YZN_SHELL,
    YZN_TUTOR, YZN_WELCOME, YZN_YA, ZELLIJ,
    command::exec,
    doctor::print_doctor,
    error::AppError,
    paths::{enter_terminal_label, nonempty_env, runtime_path},
    runtime::Runtime,
    status::{print_status, print_status_json},
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
        "status" => match args.as_slice() {
            [] => print_status(),
            [flag] if flag == "--json" => print_status_json(),
            _ => Err(AppError::Usage(
                "yzn status accepts only --json\n".to_string(),
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

fn exec_run(args: Vec<OsString>) -> Result<(), AppError> {
    let Some((program, args)) = args.split_first() else {
        return Err(AppError::Usage(
            "Usage: yzn run <program> [args...]\n".to_string(),
        ));
    };
    let runtime = Runtime::prepare()?;
    let mut command = Command::new(program);
    command.args(args);
    runtime.apply(&mut command);
    exec(command, "yzn run")
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
    let program = managed_program(through_mars);
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

fn managed_program(through_mars: bool) -> &'static str {
    if through_mars { MARS } else { YZN_WELCOME }
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
    fn enter_bypasses_mars_and_its_cursor_env() {
        assert_eq!(managed_program(false), YZN_WELCOME);
        assert_eq!(managed_program(true), MARS);
        let path = Path::new("/tmp/cursors.toml");
        let mut launch = Command::new(MARS);
        apply_mars_cursor_config(&mut launch, true, path);
        assert_eq!(
            launch.get_envs().next(),
            Some(("YAZELIX_CURSOR_CONFIG".as_ref(), Some(path.as_os_str())))
        );
        let mut enter = Command::new(YZN_WELCOME);
        apply_mars_cursor_config(&mut enter, false, path);
        assert_eq!(enter.get_envs().next(), None);
    }
}

const HELP: &str = "Yazelix Nova

Usage:
  yzn
  yzn --version
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
  yzn run <program> [args...]
  yzn status [--json]

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
