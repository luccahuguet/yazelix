use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::{
    env,
    ffi::OsString,
    process::{Command, Output},
};

const ORCHESTRATOR_PLUGIN: &str = "yazelix_pane_orchestrator";
const ZELLIJ_SESSION_NAME_ENV: &str = "ZELLIJ_SESSION_NAME";

#[derive(Clone, Debug)]
pub struct Config {
    pub ya: OsString,
    pub zellij: OsString,
    pub zellij_session_name: Option<OsString>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SidebarYaziState {
    pub yazi_id: String,
    pub cwd: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            ya: nonempty_env("YZN_YA").unwrap_or_else(|| "ya".into()),
            zellij: nonempty_env("YZN_ZELLIJ").unwrap_or_else(|| "zellij".into()),
            zellij_session_name: nonempty_env(ZELLIJ_SESSION_NAME_ENV)
                .or_else(|| nonempty_env("YAZELIX_ZELLIJ_SESSION_NAME")),
        }
    }
}

pub fn sidebar_yazi_id(raw: &str) -> Result<String> {
    Ok(sidebar_yazi_state(raw)?.yazi_id)
}

pub fn sidebar_yazi_state(raw: &str) -> Result<SidebarYaziState> {
    optional_sidebar_yazi_state(raw)?
        .context("managed sidebar Yazi is not registered in the active tab")
}

pub fn optional_sidebar_yazi_state(raw: &str) -> Result<Option<SidebarYaziState>> {
    let value = serde_json::from_str::<Value>(raw)
        .context("pane orchestrator returned invalid session JSON")?;
    let Some(sidebar) = value
        .pointer("/sidebar_yazi")
        .filter(|value| !value.is_null())
    else {
        return Ok(None);
    };
    let Some(yazi_id) = sidebar
        .get("yazi_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|id| !id.is_empty())
    else {
        return Ok(None);
    };
    let cwd = sidebar
        .get("cwd")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|cwd| !cwd.is_empty())
        .map(str::to_string);

    Ok(Some(SidebarYaziState {
        yazi_id: yazi_id.to_string(),
        cwd,
    }))
}

pub fn orchestrator_query(config: &Config, name: &str) -> Result<String> {
    let response = orchestrator_action(config, name)?;
    if response.is_empty() {
        bail!("pane orchestrator returned no response for {name}");
    }
    Ok(response)
}

pub fn orchestrator_action(config: &Config, name: &str) -> Result<String> {
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
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn ensure_success(output: &Output, context: &str) -> Result<()> {
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
