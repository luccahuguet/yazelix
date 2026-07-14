use anyhow::{bail, Context, Result};
use std::{
    env,
    ffi::OsString,
    process::{Command, ExitCode},
};
use yzx_open::sidebar::{ensure_success, optional_sidebar_yazi_state, orchestrator_query, Config};

#[cfg(test)]
#[path = "support/test_dir.rs"]
mod test_support;

fn main() -> ExitCode {
    match run(&Config::from_env(), env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzx sidebar refresh: {error:#}");
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
        bail!("expected no arguments. Try `yzx-sidebar-refresh --help`.");
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
    println!("Refresh the managed Yazi sidebar\n\nUsage:\n  yzx-sidebar-refresh");
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use crate::test_support::{write_nu_executable, TestDir};
    use std::{ffi::OsStr, fs};

    #[test]
    fn refresh_emits_yazi_sidebar_refresh_git_and_starship_events() {
        let fixture = TestDir::new();
        let ya_log = fixture.path.join("ya.log");
        let zellij_log = fixture.path.join("zellij.log");
        let zellij_body = format!("const LOG = {:?}\n", zellij_log.to_string_lossy())
            + r#"def --wrapped main [...args: string] {
    let joined = $args | str join " "
    let session = $env.ZELLIJ_SESSION_NAME? | default ""
    $"($joined) session=($session)\n" | save --append $LOG
    if ($args | get -o 5 | default "") == "get_active_tab_session_state" {
        print '{"sidebar_yazi":{"yazi_id":"plugin-yazi-id","cwd":"/repo"}}'
        return
    }
    print --stderr $"unexpected zellij args: ($joined)"
    exit 1
}
"#;
        write_nu_executable(&fixture.path.join("zellij"), &zellij_body);
        let ya_body = format!("const LOG = {:?}\n", ya_log.to_string_lossy())
            + r#"def --wrapped main [...args: string] {
    (($args | str join " ") + "\n") | save --append $LOG
}
"#;
        write_nu_executable(&fixture.path.join("ya"), &ya_body);
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
        write_nu_executable(
            &fixture.path.join("zellij"),
            "def --wrapped main [..._args: string] { print '{\"sidebar_yazi\":null}' }\n",
        );
        let ya_body = format!("const LOG = {:?}\n", ya_log.to_string_lossy())
            + r#"def --wrapped main [...args: string] {
    (($args | str join " ") + "\n") | save --force $LOG
}
"#;
        write_nu_executable(&fixture.path.join("ya"), &ya_body);
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
