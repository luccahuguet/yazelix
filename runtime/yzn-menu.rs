use std::{
    env, io,
    io::{IsTerminal, Write},
    process::{exit, Command},
};

const COMMANDS: &[(&str, &str)] = &[
    ("config", "Open config UI"),
    ("doctor", "Check runtime setup"),
    ("status", "Show runtime status"),
    ("screen", "Show terminal screen"),
    ("sponsor", "Open sponsor page"),
    ("launch", "Open Mars and start Yazelix"),
    ("help", "Show command help"),
    ("tutor", "Show guided lessons"),
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
    print_menu(interactive);
    let Some(selection) = read_selection(interactive) else {
        return 0;
    };
    let Some(id) = selected_command(&selection) else {
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

fn print_menu(interactive: bool) {
    println!("Yazelix command pane\n");
    for (index, (id, label)) in COMMANDS.iter().enumerate() {
        println!("{:>2}. {:<8} {}", index + 1, id, label);
    }
    if interactive {
        print!(
            "\nSelect command [1-{}], or Enter to close: ",
            COMMANDS.len()
        );
        let _ = io::stdout().flush();
    }
}

fn read_selection(interactive: bool) -> Option<String> {
    if !interactive && io::stdin().is_terminal() {
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
            .and_then(|index| COMMANDS.get(index).map(|(id, _label)| *id));
    }
    COMMANDS
        .iter()
        .find_map(|(id, _label)| (*id == selection).then_some(*id))
}

fn pause_if_tty(interactive: bool) {
    if interactive {
        eprint!("\nPress Enter to close...");
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }
}
