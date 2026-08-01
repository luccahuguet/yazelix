use anyhow::{Context, Result, bail};
use serde_json::json;
use std::{env, ffi::OsString, path::PathBuf, process::ExitCode};
use yzx_open::sidebar::{Config, popup_pipe};

#[cfg(test)]
#[path = "../test_support.rs"]
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
    let target = target.to_str().context("target path is not valid UTF-8")?;
    let payload = json!({
        "id": "yazi",
        "args": [target],
    })
    .to_string();
    let result = popup_pipe(config, "replace", &payload)?;
    match result.as_str() {
        "opened" => Ok(()),
        "" => bail!("persistent Yazi popup returned no response"),
        result => bail!("persistent Yazi popup reveal failed: {result}"),
    }
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
        "Open the persistent Yazi popup at a file or directory\n\nUsage:\n  yzx reveal <target>"
    );
}

#[cfg(test)]
mod tests {
    // Test lane: default
    use super::*;
    use crate::test_support::{TestDir, write_executable};
    use std::fs;

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
    fn reveal_replaces_the_configured_popup_with_the_exact_target() {
        let fixture = TestDir::new();
        let target = fixture.path.join("target with spaces.txt");
        let zellij_log = fixture.path.join("zellij.log");
        fs::write(&target, "").unwrap();
        write_executable(
            &fixture.path.join("zellij"),
            format!(
                "#!/bin/sh\nprintf '<%s>\\n' \"$@\" > \"{}\"\nprintf '%s\\n' opened\n",
                zellij_log.display()
            ),
        );
        let config = Config {
            ya: "unused-ya".into(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: Some("saved-session".into()),
        };

        run(&config, [target.clone().into_os_string()]).unwrap();

        let log = fs::read_to_string(zellij_log).unwrap();
        let expected_payload = json!({
            "id": "yazi",
            "args": [target.to_str().unwrap()],
        })
        .to_string();
        assert_eq!(
            log,
            format!(
                "<action>\n<pipe>\n<--plugin>\n<yzpp>\n<--name>\n<replace>\n<-->\n<{expected_payload}>\n"
            )
        );
    }

    #[test]
    fn popup_failure_is_reported_without_retrying_another_owner() {
        let fixture = TestDir::new();
        let target = fixture.path.join("target.txt");
        let zellij_log = fixture.path.join("zellij.log");
        fs::write(&target, "").unwrap();
        write_executable(
            &fixture.path.join("zellij"),
            format!(
                "#!/bin/sh\nprintf call >> \"{}\"\nprintf '%s\\n' invalid_payload\n",
                zellij_log.display()
            ),
        );
        let config = Config {
            ya: "unused-ya".into(),
            zellij: fixture.path.join("zellij").into_os_string(),
            zellij_session_name: None,
        };

        assert_eq!(
            run(&config, [target.into_os_string()])
                .unwrap_err()
                .to_string(),
            "persistent Yazi popup reveal failed: invalid_payload"
        );
        assert_eq!(fs::read_to_string(zellij_log).unwrap(), "call");
    }
}
