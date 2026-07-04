use std::{
    env,
    ffi::OsString,
    process::{Command, Stdio},
};

use crate::{
    command::exec,
    doctor::print_doctor,
    error::AppError,
    paths::{enter_terminal_label, nonempty_env, runtime_path},
    runtime::Runtime,
    status::print_status,
    MARS, SPONSOR_URL, YZN_CONFIG_UI, YZN_ENV_SUPERVISOR, YZN_MENU, YZN_REVEAL, YZN_SCREEN,
    YZN_SHELL, YZN_TUTOR, YZN_WELCOME, YZN_YA, ZELLIJ,
};

pub(crate) fn run() -> Result<(), AppError> {
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
