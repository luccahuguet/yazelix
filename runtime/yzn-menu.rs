use std::{
    env, io,
    io::{IsTerminal, Write},
    process::{Command, Stdio, exit},
};

const FZF: &str = "@fzf@";

const COMMANDS: &[(&str, &str, &str)] = &[
    ("config", "config", "Open Yazelix Nova config"),
    ("doctor", "system", "Check Yazelix runtime setup"),
    ("status", "system", "Show Yazelix runtime status"),
    ("screen", "help", "Show a Yazelix terminal screen"),
    ("launch", "session", "Open Mars and start Yazelix"),
    ("help", "help", "Show this help"),
    ("tutor", "help", "Show the guided Yazelix Nova tutor"),
];

fn main() {
    exit(run());
}

fn run() -> i32 {
    if env::args_os().len() > 1 {
        eprintln!("yzn-menu does not accept arguments");
        return 64;
    }

    let interactive = io::stdin().is_terminal() && io::stdout().is_terminal();
    let selection = if interactive {
        select_with_fzf()
    } else {
        print_menu();
        read_selection()
    };
    let Some(selection) = selection else {
        return 0;
    };
    let Some(id) = selected_command(selection.trim()) else {
        eprintln!("Unknown menu selection: {selection}");
        pause_if_tty(interactive);
        return 64;
    };

    let status = Command::new(env::var_os("YZN_MENU_YZN").unwrap_or_else(|| "yzn".into()))
        .arg(id)
        .status();
    let code = match status {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            eprintln!("Failed to run `yzn {id}`: {error}");
            127
        }
    };

    pause_if_tty(interactive);
    code
}

fn select_with_fzf() -> Option<String> {
    let mut child = Command::new(FZF)
        .args([
            "--border",
            "rounded",
            "--header",
            "  Yazelix Nova Command Palette",
            "--prompt",
            "  yzn> ",
            "--pointer",
            ">",
            "--layout",
            "reverse",
            "--cycle",
            "--color",
            "border:blue,header:bold:blue,prompt:bold:yellow,pointer:bold:cyan,hl:bold:magenta,hl+:bold:magenta,info:dim",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| {
            eprintln!("Failed to launch fzf for the Yazelix Nova command palette: {error}");
            exit(127);
        });

    let mut stdin = child.stdin.take().expect("missing fzf stdin");
    for (id, category, label) in COMMANDS {
        writeln!(stdin, "{id}  [{category}]  - {label}").unwrap_or_else(|error| {
            eprintln!("Failed to write command palette entries to fzf: {error}");
            exit(1);
        });
    }
    drop(stdin);

    let output = child.wait_with_output().unwrap_or_else(|error| {
        eprintln!("Failed to read fzf command palette selection: {error}");
        exit(1);
    });

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|selection| !selection.is_empty())
}

fn print_menu() {
    println!("Yazelix Nova command palette\n");
    for (index, (id, _, label)) in COMMANDS.iter().enumerate() {
        println!("{:>2}. {:<8} {}", index + 1, id, label);
    }
}

fn read_selection() -> Option<String> {
    if io::stdin().is_terminal() {
        return None;
    }

    let mut selection = String::new();
    io::stdin().read_line(&mut selection).ok()?;
    let trimmed = selection.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn selected_command(selection: &str) -> Option<&'static str> {
    if let Ok(index) = selection.parse::<usize>() {
        return index
            .checked_sub(1)
            .and_then(|index| COMMANDS.get(index).map(|(id, _, _)| *id));
    }
    COMMANDS.iter().find_map(|(id, _, _)| {
        (*id == selection
            || selection
                .strip_prefix(id)
                .is_some_and(|rest| rest.starts_with("  ")))
        .then_some(*id)
    })
}

fn pause_if_tty(interactive: bool) {
    if interactive {
        eprint!("\nPress Enter to close...");
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }
}
