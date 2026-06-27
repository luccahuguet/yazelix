use std::{
    env, fs,
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

const FORBIDDEN: &[&str] = &[
    "keybinds",
    "default_shell",
    "default_layout",
    "layout",
    "plugins",
    "load_plugins",
    "support_kitty_keyboard_protocol",
    "env",
    "session_name",
    "attach_to_session",
];

fn main() -> io::Result<()> {
    let args = env::args_os().map(PathBuf::from).collect::<Vec<_>>();
    let [_, packaged, sidecar, runtime_config] = args.as_slice() else {
        return Err(invalid_input(
            "usage: yzn-zellij-config <packaged-config> <sidecar> <runtime-config>",
        ));
    };

    if !sidecar.is_file() {
        println!("{}", packaged.display());
        return Ok(());
    }

    let sidecar_config = fs::read_to_string(&sidecar)?;
    validate_sidecar(&sidecar, &sidecar_config)?;

    fs::create_dir_all(runtime_config.parent().unwrap())?;
    fs::write(
        runtime_config,
        format!(
            "{}\n{}{}",
            fs::read_to_string(&packaged)?.trim_end(),
            sidecar_config,
            if sidecar_config.ends_with('\n') {
                ""
            } else {
                "\n"
            }
        ),
    )?;
    println!("{}", runtime_config.display());
    Ok(())
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
