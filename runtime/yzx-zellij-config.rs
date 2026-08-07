use std::{
    env, fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
    process,
};

const FORBIDDEN: &[&str] = &[
    "keybinds",
    "default_shell",
    "default_layout",
    "layout_dir",
    "layout",
    "plugins",
    "load_plugins",
    "support_kitty_keyboard_protocol",
    "env",
    "session_name",
    "attach_to_session",
];

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args = env::args_os().map(PathBuf::from).collect::<Vec<_>>();
    let [_, packaged, sidecar, runtime_config] = args.as_slice() else {
        return Err(invalid_input(
            "usage: yzx-zellij-config <packaged-config> <sidecar> <runtime-config>",
        ));
    };

    if !sidecar.is_file() {
        println!("{}", packaged.display());
        return Ok(());
    }

    let sidecar_config = fs::read_to_string(&sidecar)?;
    validate_sidecar(&sidecar, &sidecar_config)?;
    let applied_sidecar = without_top_level_nodes(&sidecar_config, &["theme"]);
    let pair_overrides = ["theme_dark", "theme_light"]
        .into_iter()
        .filter(|token| has_top_level_node(&applied_sidecar, token))
        .collect::<Vec<_>>();
    let packaged_config = without_top_level_nodes(&fs::read_to_string(packaged)?, &pair_overrides);

    fs::create_dir_all(runtime_config.parent().unwrap())?;
    fs::write(
        runtime_config,
        format!(
            "{}\n{}{}",
            packaged_config.trim_end(),
            applied_sidecar,
            if applied_sidecar.ends_with('\n') {
                ""
            } else {
                "\n"
            }
        ),
    )?;
    println!("{}", runtime_config.display());
    Ok(())
}

fn without_top_level_nodes(text: &str, removed: &[&str]) -> String {
    let mut depth = 0;
    let mut output = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let delta = brace_delta(line);
        let remove = depth == 0
            && delta == 0
            && first_token(line).is_some_and(|token| removed.contains(&token));
        if !remove {
            output.push_str(line);
        }
        depth += delta;
    }
    output
}

fn has_top_level_node(text: &str, expected: &str) -> bool {
    let mut depth = 0;
    for line in text.lines() {
        let delta = brace_delta(line);
        if depth == 0 && delta == 0 && first_token(line) == Some(expected) {
            return true;
        }
        depth += delta;
    }
    false
}

fn brace_delta(line: &str) -> isize {
    let line = line.trim_start();
    if line.starts_with('#') {
        return 0;
    }
    let mut delta = 0;
    let mut quoted = false;
    let mut escaped = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' if quoted && !escaped => {
                escaped = true;
                continue;
            }
            '"' if !escaped => quoted = !quoted,
            '/' if !quoted && chars.peek() == Some(&'/') => break,
            '{' if !quoted => delta += 1,
            '}' if !quoted => delta -= 1,
            _ => {}
        }
        escaped = false;
    }
    delta
}

fn validate_sidecar(path: &Path, text: &str) -> io::Result<()> {
    for (index, line) in text.lines().enumerate() {
        let Some(name) = first_token(line) else {
            continue;
        };
        if FORBIDDEN.contains(&name) {
            return Err(invalid_input(format!(
                "{}:{}: forbidden Zellij sidecar item `{name}`",
                path.display(),
                index + 1
            )));
        }
    }
    Ok(())
}

fn first_token(line: &str) -> Option<&str> {
    let line = line.trim_start();
    if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
        return None;
    }
    line.split(|ch: char| ch.is_whitespace() || ch == '{' || ch == ';')
        .next()
        .filter(|token| !token.is_empty())
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, message.into())
}
