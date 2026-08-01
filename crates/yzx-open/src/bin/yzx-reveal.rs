use anyhow::{Context, Result, bail};
use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    process::{Command, ExitCode},
};
use yzx_open::sidebar::{Config, ensure_success, orchestrator_query, workspace_popup_yazi_id};

#[cfg(test)]
#[path = "support/test_dir.rs"]
mod test_support;

fn main() -> ExitCode {
    match run(&Config::from_env(), env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzx reveal: {error:#}");
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
    let popup_state = orchestrator_query(config, "focus_workspace_popup_yazi")?;
    let yazi_id = workspace_popup_yazi_id(&popup_state)?;
    let output = Command::new(&config.ya)
        .arg("emit-to")
        .arg(&yazi_id)
        .arg("reveal")
        .arg(&target)
        .output()
        .context("could not run ya")?;
    ensure_success(&output, "ya reveal failed")?;

    Ok(())
}

fn parse_target(raw_args: impl IntoIterator<Item = OsString>) -> Result<OsString> {
    let mut args = raw_args.into_iter();
    let Some(target) = args.next() else {
        bail!("missing target path. Try `yzx reveal --help`.");
    };
    if target.is_empty() {
        bail!("missing target path. Try `yzx reveal --help`.");
    }
    if args.next().is_some() {
        bail!("expected exactly one target path. Try `yzx reveal --help`.");
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
        "Reveal a file or directory in the persistent Yazi popup\n\nUsage:\n  yzx reveal <target>"
    );
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use crate::test_support::{TestDir, write_executable};
    use std::fs;

    #[test]
    fn parses_popup_yazi_address_and_reports_bounded_failure() {
        assert_eq!(
            workspace_popup_yazi_id(r#"{"status":"ok","yazi_id":" yazi-7 "}"#).unwrap(),
            "yazi-7"
        );
        assert_eq!(
            workspace_popup_yazi_id(&"x".repeat(4096))
                .unwrap_err()
                .to_string(),
            "persistent Yazi popup is not ready"
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
    fn reveal_delivers_exact_file_and_directory_paths_with_spaces() {
        let fixture = TestDir::new();
        let file_target = fixture.path.join("target with spaces.txt");
        let directory_target = fixture.path.join("target directory");
        let zellij_log = fixture.path.join("zellij.log");
        let ya_log = fixture.path.join("ya.log");
        fs::write(&file_target, "").unwrap();
        fs::create_dir(&directory_target).unwrap();
        write_executable(
            &fixture.path.join("zellij"),
            &format!(
                r#"#!/bin/sh
printf '%s\n' "$* session=$ZELLIJ_SESSION_NAME" >> "{}"
case "$6" in
  focus_workspace_popup_yazi)
    printf '%s\n' '{{"status":"ok","yazi_id":"plugin-yazi-id"}}'
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

        run(&config, [file_target.clone().into_os_string()]).unwrap();
        run(&config, [directory_target.clone().into_os_string()]).unwrap();

        let expected_pipe = "action pipe --plugin yazelix_pane_orchestrator --name focus_workspace_popup_yazi --  session=saved-session\n";
        assert_eq!(
            fs::read_to_string(zellij_log).unwrap(),
            expected_pipe.repeat(2)
        );
        assert_eq!(
            fs::read_to_string(ya_log).unwrap(),
            format!(
                "emit-to plugin-yazi-id reveal {}\nemit-to plugin-yazi-id reveal {}\n",
                file_target.display(),
                directory_target.display()
            )
        );
    }

    #[test]
    fn popup_readiness_failure_does_not_fall_back_to_yazi() {
        let fixture = TestDir::new();
        let target = fixture.path.join("target.txt");
        fs::write(&target, "").unwrap();
        write_executable(
            &fixture.path.join("zellij"),
            "#!/bin/sh\nprintf '%s\\n' not_ready\n",
        );
        write_executable(
            &fixture.path.join("ya"),
            "#!/bin/sh\nprintf 'unexpected ya call\\n' >&2\nexit 1\n",
        );
        let config = Config {
            ya: fixture.path.join("ya").into_os_string(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: None,
        };

        assert!(
            run(&config, [target.into_os_string()])
                .unwrap_err()
                .to_string()
                .contains("persistent Yazi popup is not ready")
        );
    }
}
