use anyhow::{Context, Result, bail};
use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    process::{Command, ExitCode},
};
use yzn_open::sidebar::{
    Config, ensure_success, orchestrator_action, orchestrator_query, sidebar_yazi_id,
};

#[cfg(test)]
#[path = "support/test_dir.rs"]
mod test_support;

fn main() -> ExitCode {
    match run(&Config::from_env(), env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzn reveal: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(config: &Config, raw_args: impl IntoIterator<Item = OsString>) -> Result<()> {
    let target = parse_target(raw_args)?;
    if target == "-h" || target == "--help" {
        print_help();
        return Ok(());
    }

    let target = existing_absolute_path(&target)?;
    let session_state = orchestrator_query(config, "get_active_tab_session_state")?;
    let yazi_id = sidebar_yazi_id(&session_state)?;
    let output = Command::new(&config.ya)
        .arg("emit-to")
        .arg(&yazi_id)
        .arg("reveal")
        .arg(&target)
        .output()
        .context("could not run ya")?;
    ensure_success(&output, "ya reveal failed")?;

    let focus_status = orchestrator_action(config, "focus_sidebar")?;
    let focus_status = focus_status.trim();
    if !focus_status.is_empty()
        && !matches!(
            focus_status,
            "ok" | "opened" | "focused" | "focused_sidebar" | "opened_sidebar"
        )
    {
        bail!("managed sidebar focus failed: {focus_status}");
    }
    Ok(())
}

fn parse_target(raw_args: impl IntoIterator<Item = OsString>) -> Result<OsString> {
    let mut args = raw_args.into_iter();
    let Some(target) = args.next() else {
        bail!("missing target path. Try `yzn reveal --help`.");
    };
    if target.is_empty() {
        bail!("missing target path. Try `yzn reveal --help`.");
    }
    if args.next().is_some() {
        bail!("expected exactly one target path. Try `yzn reveal --help`.");
    }
    Ok(target)
}

fn existing_absolute_path(target: &OsString) -> Result<PathBuf> {
    let path =
        std::path::absolute(PathBuf::from(target)).context("could not resolve target path")?;
    if !path.exists() {
        bail!("target does not exist: {}", path.display());
    }
    Ok(path)
}

fn print_help() {
    println!(
        "Reveal a file or directory in the managed Yazi sidebar\n\nUsage:\n  yzn reveal <target>"
    );
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use crate::test_support::{TestDir, write_executable};
    use std::fs;

    #[test]
    fn parses_sidebar_yazi_id_and_reports_missing_state() {
        assert_eq!(
            sidebar_yazi_id(r#"{"sidebar_yazi":{"yazi_id":" yazi-7 ","cwd":"/tmp"}}"#).unwrap(),
            "yazi-7"
        );
        assert!(
            sidebar_yazi_id(r#"{"sidebar_yazi":null}"#)
                .unwrap_err()
                .to_string()
                .contains("managed sidebar Yazi is not registered")
        );
    }

    #[test]
    fn target_parser_requires_one_argument_except_help() {
        assert_eq!(
            parse_target(["--help".into()]).unwrap(),
            OsString::from("--help")
        );
        assert!(parse_target(Vec::<OsString>::new()).is_err());
        assert!(parse_target([OsString::new()]).is_err());
        assert!(parse_target(["one".into(), "two".into()]).is_err());
    }

    #[test]
    fn reveal_uses_registered_sidebar_yazi_and_focuses_sidebar() {
        let fixture = TestDir::new();
        let target = fixture.path.join("target.txt");
        let zellij_log = fixture.path.join("zellij.log");
        let ya_log = fixture.path.join("ya.log");
        fs::write(&target, "").unwrap();
        write_executable(
            &fixture.path.join("zellij"),
            &format!(
                r#"#!/bin/sh
printf '%s\n' "$* session=$ZELLIJ_SESSION_NAME" >> "{}"
case "$6" in
  get_active_tab_session_state)
    printf '%s\n' '{{"sidebar_yazi":{{"yazi_id":"plugin-yazi-id"}}}}'
    exit 0
    ;;
  focus_sidebar)
    printf '%s\n' 'focused_sidebar'
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
                "#!/bin/sh\nprintf '%s\\n' \"$*\" > \"{}\"\n",
                ya_log.display()
            ),
        );

        let config = Config {
            ya: fixture.path.join("ya").into_os_string(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: Some("saved-session".into()),
        };

        run(&config, [target.clone().into_os_string()]).unwrap();

        assert_eq!(
            fs::read_to_string(zellij_log).unwrap(),
            "action pipe --plugin yazelix_pane_orchestrator --name get_active_tab_session_state --  session=saved-session\n\
action pipe --plugin yazelix_pane_orchestrator --name focus_sidebar --  session=saved-session\n"
        );
        assert_eq!(
            fs::read_to_string(ya_log).unwrap(),
            format!("emit-to plugin-yazi-id reveal {}\n", target.display())
        );
    }

    #[test]
    fn reveal_allows_empty_focus_response_after_successful_command() {
        let fixture = TestDir::new();
        let target = fixture.path.join("target.txt");
        let ya_log = fixture.path.join("ya.log");
        fs::write(&target, "").unwrap();
        write_executable(
            &fixture.path.join("zellij"),
            r#"#!/bin/sh
case "$6" in
  get_active_tab_session_state)
    printf '%s\n' '{"sidebar_yazi":{"yazi_id":"plugin-yazi-id"}}'
    exit 0
    ;;
  focus_sidebar)
    exit 0
    ;;
esac
printf 'unexpected zellij args: %s\n' "$*" >&2
exit 1
"#,
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
            zellij_session_name: Some("saved-session".into()),
        };

        run(&config, [target.clone().into_os_string()]).unwrap();

        assert_eq!(
            fs::read_to_string(ya_log).unwrap(),
            format!("emit-to plugin-yazi-id reveal {}\n", target.display())
        );
    }
}
