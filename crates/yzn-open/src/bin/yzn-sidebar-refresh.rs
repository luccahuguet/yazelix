use anyhow::{Context, Result, bail};
use std::{
    env,
    ffi::OsString,
    process::{Command, ExitCode},
};
use yzn_open::sidebar::{Config, ensure_success, optional_sidebar_yazi_state, orchestrator_query};

#[cfg(test)]
#[path = "support/test_dir.rs"]
mod test_support;

fn main() -> ExitCode {
    match run(&Config::from_env(), env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzn sidebar refresh: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(config: &Config, raw_args: impl IntoIterator<Item = OsString>) -> Result<()> {
    let args = raw_args.into_iter().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        if args.len() == 1 {
            print_help();
            return Ok(());
        }
        bail!("--help does not accept extra arguments");
    }
    if !args.is_empty() {
        bail!("expected no arguments. Try `yzn-sidebar-refresh --help`.");
    }

    let session_state = orchestrator_query(config, "get_active_tab_session_state")?;
    let Some(sidebar) = optional_sidebar_yazi_state(&session_state)? else {
        return Ok(());
    };

    ya_emit_to(config, &sidebar.yazi_id, ["refresh"])?;
    ya_emit_to(
        config,
        &sidebar.yazi_id,
        ["plugin", "git", "refresh-sidebar"],
    )?;
    if let Some(cwd) = sidebar.cwd {
        ya_emit_to(
            config,
            &sidebar.yazi_id,
            ["plugin", "starship", cwd.as_str()],
        )?;
    }

    Ok(())
}

fn ya_emit_to<'a>(
    config: &Config,
    yazi_id: &str,
    args: impl IntoIterator<Item = &'a str>,
) -> Result<()> {
    let output = Command::new(&config.ya)
        .arg("emit-to")
        .arg(yazi_id)
        .args(args)
        .output()
        .context("could not run ya")?;
    ensure_success(&output, "ya sidebar refresh failed")
}

fn print_help() {
    println!("Refresh the managed Yazi sidebar\n\nUsage:\n  yzn-sidebar-refresh");
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use crate::test_support::{TestDir, write_executable};
    use std::{ffi::OsStr, fs};

    #[test]
    fn refresh_emits_yazi_sidebar_refresh_git_and_starship_events() {
        let fixture = TestDir::new();
        let ya_log = fixture.path.join("ya.log");
        let zellij_log = fixture.path.join("zellij.log");
        write_executable(
            &fixture.path.join("zellij"),
            &format!(
                r#"#!/bin/sh
printf '%s\n' "$* session=$ZELLIJ_SESSION_NAME" >> "{}"
case "$6" in
  get_active_tab_session_state)
    printf '%s\n' '{{"sidebar_yazi":{{"yazi_id":"plugin-yazi-id","cwd":"/repo"}}}}'
    exit 0
    ;;
esac
printf 'unexpected zellij args: %s\n' "$*" >&2
exit 1
"#,
                zellij_log.display()
            ),
        );
        write_executable(
            &fixture.path.join("ya"),
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"{}\"\n",
                ya_log.display()
            ),
        );
        let config = Config {
            ya: fixture.path.join("ya").into_os_string(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: Some("saved-session".into()),
        };

        run(&config, Vec::<OsString>::new()).unwrap();

        assert_eq!(
            fs::read_to_string(zellij_log).unwrap(),
            "action pipe --plugin yazelix_pane_orchestrator --name get_active_tab_session_state --  session=saved-session\n"
        );
        assert_eq!(
            fs::read_to_string(ya_log).unwrap(),
            "emit-to plugin-yazi-id refresh\n\
emit-to plugin-yazi-id plugin git refresh-sidebar\n\
emit-to plugin-yazi-id plugin starship /repo\n"
        );
    }

    #[test]
    fn missing_sidebar_state_is_a_noop() {
        let fixture = TestDir::new();
        let ya_log = fixture.path.join("ya.log");
        write_executable(
            &fixture.path.join("zellij"),
            "#!/bin/sh\nprintf '%s\n' '{\"sidebar_yazi\":null}'\n",
        );
        write_executable(
            &fixture.path.join("ya"),
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" > \"{}\"\n",
                ya_log.display()
            ),
        );
        let config = Config {
            ya: fixture.path.join("ya").into_os_string(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: None,
        };

        run(&config, Vec::<OsString>::new()).unwrap();

        assert!(!ya_log.exists());
    }

    #[test]
    fn rejects_unexpected_arguments() {
        let config = Config {
            ya: OsStr::new("ya").into(),
            zellij: OsStr::new("zellij").into(),
            zellij_session_name: None,
        };

        assert!(run(&config, ["extra".into()]).is_err());
        assert!(run(&config, ["--help".into(), "extra".into()]).is_err());
    }
}
