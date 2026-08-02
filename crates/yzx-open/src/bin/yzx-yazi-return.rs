use anyhow::Result;
use std::{env, process::ExitCode};
use yzx_open::sidebar::{Config, orchestrator_pipe, popup_pipe};

#[cfg(test)]
#[path = "../test_support.rs"]
mod test_support;

fn main() -> ExitCode {
    match run(
        &Config::from_env(),
        env::var("YZX_YAZI_ROLE").ok().as_deref(),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzx yazi return: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(config: &Config, role: Option<&str>) -> Result<()> {
    if role == Some("workspace-popup") {
        popup_pipe(config, "hide", "yazi").map(|_| ())
    } else {
        orchestrator_pipe(config, "focus_editor", "").map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use crate::test_support::{TestDir, write_executable};
    use std::fs;

    #[test]
    fn popup_hides_and_tiled_yazi_returns_to_the_existing_editor() {
        let fixture = TestDir::new();
        let zellij_log = fixture.path.join("zellij.log");
        write_executable(
            &fixture.path.join("zellij"),
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\nprintf '%s\\n' ok\n",
                zellij_log.display()
            ),
        );
        let config = Config {
            ya: "unused-ya".into(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: Some("saved-session".into()),
        };

        run(&config, Some("workspace-popup")).unwrap();
        run(&config, None).unwrap();

        assert_eq!(
            fs::read_to_string(zellij_log).unwrap(),
            "action pipe --plugin yzpp --name hide -- yazi\n\
action pipe --plugin yazelix_pane_orchestrator --name focus_editor -- \n"
        );
    }
}
