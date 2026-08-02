use anyhow::{Context, Result};
use std::{
    env,
    ffi::OsStr,
    fs,
    process::{Command, ExitCode},
};
use yzx_open::sidebar::{Config, ensure_success, orchestrator_pipe, popup_pipe};

#[cfg(test)]
#[path = "../test_support.rs"]
mod test_support;

fn main() -> ExitCode {
    let target = env::args_os().nth(1);
    let yzx_open = env::var_os("YZX_OPEN");
    match run(
        &Config::from_env(),
        env::var("YZX_YAZI_ROLE").ok().as_deref(),
        target.as_deref(),
        yzx_open.as_deref(),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzx yazi return: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(
    config: &Config,
    role: Option<&str>,
    target: Option<&OsStr>,
    yzx_open: Option<&OsStr>,
) -> Result<()> {
    if role == Some("workspace-popup") {
        popup_pipe(config, "hide", "yazi")?;
    }
    match target {
        Some(target) if fs::metadata(target).is_ok_and(|metadata| metadata.is_file()) => {
            if let Err(error) = reveal_file(target, yzx_open) {
                let _ = focus_editor(config);
                return Err(error);
            }
        }
        _ => {
            focus_editor(config)?;
        }
    }
    Ok(())
}

fn reveal_file(target: &OsStr, yzx_open: Option<&OsStr>) -> Result<()> {
    let yzx_open = yzx_open.context("YZX_OPEN is not set")?;
    let output = Command::new(yzx_open)
        .arg("--reveal-editor")
        .arg(target)
        .output()
        .context("could not run yzx-open")?;
    ensure_success(&output, "yzx-open could not reveal the hovered file")
}

fn focus_editor(config: &Config) -> Result<()> {
    orchestrator_pipe(config, "focus_editor", "").map(|_| ())
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use crate::test_support::{TestDir, write_executable};
    use std::fs;

    #[test]
    fn popup_hides_and_hovered_files_reveal_without_affecting_directories() {
        let fixture = TestDir::new();
        let zellij_log = fixture.path.join("zellij.log");
        let open_log = fixture.path.join("open.log");
        write_executable(
            &fixture.path.join("zellij"),
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\nprintf '%s\\n' ok\n",
                zellij_log.display()
            ),
        );
        write_executable(
            &fixture.path.join("yzx-open"),
            format!(
                "#!/bin/sh\nfor arg in \"$@\"; do printf '<%s>\\n' \"$arg\"; done >> \"{}\"\n",
                open_log.display()
            ),
        );
        let config = Config {
            ya: "unused-ya".into(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: Some("saved-session".into()),
        };
        let file = fixture.path.join("hovered note.md");
        let directory = fixture.path.join("directory");
        fs::write(&file, "").unwrap();
        fs::create_dir(&directory).unwrap();

        run(
            &config,
            Some("workspace-popup"),
            Some(file.as_os_str()),
            Some(fixture.path.join("yzx-open").as_os_str()),
        )
        .unwrap();
        run(
            &config,
            Some("workspace-popup"),
            Some(directory.as_os_str()),
            None,
        )
        .unwrap();
        run(&config, None, Some(directory.as_os_str()), None).unwrap();

        assert_eq!(
            fs::read_to_string(zellij_log).unwrap(),
            "action pipe --plugin yzpp --name hide -- yazi\n\
action pipe --plugin yzpp --name hide -- yazi\n\
action pipe --plugin yazelix_pane_orchestrator --name focus_editor -- \n\
action pipe --plugin yazelix_pane_orchestrator --name focus_editor -- \n"
        );
        assert_eq!(
            fs::read_to_string(open_log).unwrap(),
            format!("<--reveal-editor>\n<{}>\n", file.display())
        );
    }
}
