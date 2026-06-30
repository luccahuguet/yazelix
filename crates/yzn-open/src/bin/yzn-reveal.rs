use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    process::{Command, ExitCode, Output},
};

const ORCHESTRATOR_PLUGIN: &str = "yazelix_pane_orchestrator";
const ZELLIJ_SESSION_NAME_ENV: &str = "ZELLIJ_SESSION_NAME";

struct Config {
    ya: OsString,
    zellij: OsString,
    zellij_session_name: Option<OsString>,
}

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
    if target == OsString::from("-h") || target == OsString::from("--help") {
        print_help();
        return Ok(());
    }

    let target = existing_absolute_path(&target)?;
    let session_state = orchestrator_command(config, "get_active_tab_session_state")?;
    let yazi_id = sidebar_yazi_id(&session_state)?;
    let output = Command::new(&config.ya)
        .arg("emit-to")
        .arg(&yazi_id)
        .arg("reveal")
        .arg(&target)
        .output()
        .context("could not run ya")?;
    ensure_success(&output, "ya reveal failed")?;

    let focus_status = orchestrator_command(config, "focus_sidebar")?;
    if !matches!(
        focus_status.trim(),
        "ok" | "opened" | "focused" | "focused_sidebar" | "opened_sidebar"
    ) {
        bail!("managed sidebar focus failed: {}", focus_status.trim());
    }
    Ok(())
}

impl Config {
    fn from_env() -> Self {
        Self {
            ya: nonempty_env("YZN_YA").unwrap_or_else(|| "ya".into()),
            zellij: nonempty_env("YZN_ZELLIJ").unwrap_or_else(|| "zellij".into()),
            zellij_session_name: nonempty_env(ZELLIJ_SESSION_NAME_ENV)
                .or_else(|| nonempty_env("YAZELIX_ZELLIJ_SESSION_NAME")),
        }
    }
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

fn sidebar_yazi_id(raw: &str) -> Result<String> {
    serde_json::from_str::<Value>(raw)
        .context("pane orchestrator returned invalid session JSON")?
        .pointer("/sidebar_yazi/yazi_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .context("managed sidebar Yazi is not registered in the active tab")
}

fn orchestrator_command(config: &Config, name: &str) -> Result<String> {
    let mut command = Command::new(&config.zellij);
    if let Some(session_name) = &config.zellij_session_name {
        command.env(ZELLIJ_SESSION_NAME_ENV, session_name);
    }
    let output = command
        .args([
            "action",
            "pipe",
            "--plugin",
            ORCHESTRATOR_PLUGIN,
            "--name",
            name,
            "--",
            "",
        ])
        .output()
        .with_context(|| format!("could not pipe {name} to pane orchestrator"))?;
    ensure_success(&output, "pane orchestrator command failed")?;
    let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if response.is_empty() {
        bail!("pane orchestrator returned no response for {name}");
    }
    Ok(response)
}

fn ensure_success(output: &Output, context: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let message = [stderr.trim(), stdout.trim()]
        .into_iter()
        .find(|part| !part.is_empty())
        .unwrap_or("no output");
    bail!("{context}: {message}");
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
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
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

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

    fn write_executable(path: &Path, contents: &str) {
        fs::write(path, contents).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            let path = env::temp_dir().join(format!("yzn-reveal-{}-{millis}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
