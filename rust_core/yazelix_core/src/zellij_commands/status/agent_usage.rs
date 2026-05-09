use super::{
    STATUS_BUS_SCHEMA_VERSION, mark_status_cache_refresh_finished, render_zjstatus_cursor_widget,
    render_zjstatus_workspace_widget, session_config_path_from_env,
    session_config_path_from_values,
};
use crate::bridge::CoreError;
use serde_json::Value;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use yazelix_bar::{
    AgentUsageDisplay as BarAgentUsageDisplay, WindowedAgentUsageFacts,
    render_codex_usage_status_widget,
};

pub(crate) const CLAUDE_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
pub(crate) const CODEX_USAGE_CACHE_SCHEMA_VERSION: i64 = 2;
pub(crate) const OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION: i64 = 1;
pub(crate) const CLAUDE_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub(crate) const CODEX_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub(crate) const OPENCODE_GO_USAGE_LOCK_STALE_AFTER_SECONDS: u64 = 300;
pub(crate) const OPENCODE_GO_PROVIDER_ID: &str = "opencode-go";
pub(crate) const OPENCODE_GO_FIVE_HOUR_SECONDS: u64 = 5 * 60 * 60;
pub(crate) const OPENCODE_GO_WEEK_SECONDS: u64 = 7 * 24 * 60 * 60;
pub(crate) const OPENCODE_GO_MONTH_SECONDS: u64 = 30 * 24 * 60 * 60;
pub(crate) const OPENCODE_GO_FIVE_HOUR_LIMIT_USD: f64 = 12.0;
pub(crate) const OPENCODE_GO_WEEKLY_LIMIT_USD: f64 = 30.0;
pub(crate) const OPENCODE_GO_MONTHLY_LIMIT_USD: f64 = 60.0;
pub(crate) const MINUTE_SECONDS: u64 = 60;
#[cfg(test)]
pub(crate) const HOUR_SECONDS: u64 = 60 * MINUTE_SECONDS;
#[cfg(test)]
pub(crate) const DAY_SECONDS: u64 = 24 * HOUR_SECONDS;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowedUsageDisplay {
    Both,
    Token,
    Quota,
}

impl WindowedUsageDisplay {
    fn parse(raw: &str) -> Self {
        match raw.trim() {
            "token" | "tokens" => Self::Token,
            "quota" => Self::Quota,
            _ => Self::Both,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AgentUsageWidgetSettings {
    pub(crate) claude_display: WindowedUsageDisplay,
    pub(crate) codex_display: WindowedUsageDisplay,
    pub(crate) opencode_go_display: WindowedUsageDisplay,
    pub(crate) claude_periods: Vec<WindowedUsagePeriod>,
    pub(crate) opencode_go_periods: Vec<WindowedUsagePeriod>,
}

impl Default for AgentUsageWidgetSettings {
    fn default() -> Self {
        Self {
            claude_display: WindowedUsageDisplay::Both,
            codex_display: WindowedUsageDisplay::Quota,
            opencode_go_display: WindowedUsageDisplay::Both,
            claude_periods: default_windowed_usage_periods().to_vec(),
            opencode_go_periods: default_opencode_go_usage_periods(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WindowedUsageFacts {
    pub(crate) updated_at_unix_seconds: Option<u64>,
    pub(crate) five_hour_tokens: Option<u64>,
    pub(crate) weekly_tokens: Option<u64>,
    pub(crate) monthly_tokens: Option<u64>,
    pub(crate) five_hour_remaining_percent: Option<u64>,
    pub(crate) weekly_remaining_percent: Option<u64>,
    pub(crate) monthly_remaining_percent: Option<u64>,
    pub(crate) five_hour_reset_at_unix_seconds: Option<u64>,
    pub(crate) weekly_reset_at_unix_seconds: Option<u64>,
    pub(crate) five_hour_window_seconds: Option<u64>,
    pub(crate) weekly_window_seconds: Option<u64>,
    pub(crate) error: Option<String>,
}

impl WindowedUsageFacts {
    fn has_tokens(&self) -> bool {
        self.five_hour_tokens.is_some()
            || self.weekly_tokens.is_some()
            || self.monthly_tokens.is_some()
    }

    fn has_quota(&self) -> bool {
        self.five_hour_remaining_percent.is_some()
            || self.weekly_remaining_percent.is_some()
            || self.monthly_remaining_percent.is_some()
    }

    fn is_empty(&self) -> bool {
        !self.has_tokens() && !self.has_quota()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenusageWindowedProvider {
    Claude,
    Codex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowedUsagePeriod {
    FiveHour,
    Weekly,
    Monthly,
}

impl WindowedUsagePeriod {
    fn parse_config(raw: &str) -> Option<Self> {
        match raw.trim() {
            "5h" | "five_hour" | "five-hour" | "rolling" => Some(Self::FiveHour),
            "week" | "weekly" | "wk" => Some(Self::Weekly),
            "month" | "monthly" | "mon" | "mo" => Some(Self::Monthly),
            _ => None,
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            Self::FiveHour => "5h",
            Self::Weekly => "wk",
            Self::Monthly => "mo",
        }
    }
}

pub(crate) fn default_windowed_usage_periods() -> &'static [WindowedUsagePeriod] {
    &[WindowedUsagePeriod::FiveHour, WindowedUsagePeriod::Weekly]
}

pub(crate) fn default_opencode_go_usage_periods() -> Vec<WindowedUsagePeriod> {
    vec![
        WindowedUsagePeriod::FiveHour,
        WindowedUsagePeriod::Weekly,
        WindowedUsagePeriod::Monthly,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentUsageRefreshTarget {
    Claude,
    Codex,
    OpenCodeGo,
}

impl AgentUsageRefreshTarget {
    pub(crate) fn command_name(self) -> &'static str {
        match self {
            Self::Claude => "status-cache-refresh-claude-usage",
            Self::Codex => "status-cache-refresh-codex-usage",
            Self::OpenCodeGo => "status-cache-refresh-opencode-go-usage",
        }
    }

    pub(crate) fn widget_name(self) -> &'static str {
        match self {
            Self::Claude => "claude_usage",
            Self::Codex => "codex_usage",
            Self::OpenCodeGo => "opencode_go_usage",
        }
    }

    pub(crate) fn allow_timeout(self) -> bool {
        matches!(self, Self::Claude | Self::Codex)
    }
}

pub(crate) fn refresh_agent_usage_shared_cache_for_status_cache_path(
    target: AgentUsageRefreshTarget,
    status_cache_path: &Path,
    path_var: Option<&OsStr>,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
    timeout: Duration,
) -> Result<(), CoreError> {
    if !usage_widget_enabled_from_status_cache_path(status_cache_path, target.widget_name()) {
        return Ok(());
    }
    match target {
        AgentUsageRefreshTarget::Claude => {
            let Some(shared_path) =
                claude_usage_shared_cache_path_from_status_cache_path(status_cache_path)
            else {
                return Ok(());
            };
            refresh_tokenusage_windowed_usage_shared_cache(
                &shared_path,
                TokenusageWindowedProvider::Claude,
                path_var,
                now,
                max_age_seconds,
                error_backoff_seconds,
                timeout,
            )?;
        }
        AgentUsageRefreshTarget::Codex => {
            let Some(shared_path) =
                codex_usage_shared_cache_path_from_status_cache_path(status_cache_path)
            else {
                return Ok(());
            };
            refresh_tokenusage_windowed_usage_shared_cache(
                &shared_path,
                TokenusageWindowedProvider::Codex,
                path_var,
                now,
                max_age_seconds,
                error_backoff_seconds,
                timeout,
            )?;
        }
        AgentUsageRefreshTarget::OpenCodeGo => {
            let Some(shared_path) =
                opencode_go_usage_shared_cache_path_from_status_cache_path(status_cache_path)
            else {
                return Ok(());
            };
            refresh_opencode_go_usage_shared_cache(
                &shared_path,
                now,
                max_age_seconds,
                error_backoff_seconds,
            )?;
        }
    }
    mark_status_cache_refresh_finished(status_cache_path, target.widget_name())
}

pub(crate) fn codex_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    agent_usage_shared_cache_path_from_status_cache_path(
        status_cache_path,
        "codex_usage_cache",
        CODEX_USAGE_CACHE_SCHEMA_VERSION,
    )
}

pub(crate) fn claude_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    agent_usage_shared_cache_path_from_status_cache_path(
        status_cache_path,
        "claude_usage_cache",
        CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
    )
}

pub(crate) fn opencode_go_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<PathBuf> {
    agent_usage_shared_cache_path_from_status_cache_path(
        status_cache_path,
        "opencode_go_usage_cache",
        OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
    )
}

pub(crate) fn agent_usage_shared_cache_path_from_status_cache_path(
    status_cache_path: &Path,
    file_stem: &str,
    schema_version: i64,
) -> Option<PathBuf> {
    let state_dir = status_cache_path.parent()?.parent()?.parent()?;
    Some(
        state_dir
            .join("agent_usage")
            .join(format!("{file_stem}_v{schema_version}.json")),
    )
}

pub(crate) fn hydrate_status_cache_codex_usage(cache: &mut Value, status_cache_path: &Path) {
    let Some(shared_path) = codex_usage_shared_cache_path_from_status_cache_path(status_cache_path)
    else {
        return;
    };
    let Some(shared_cache) = read_codex_usage_shared_cache_value(&shared_path) else {
        return;
    };
    let Some(codex) = shared_cache.get("codex").cloned() else {
        return;
    };
    cache["codex_usage"] = codex;
}

pub(crate) fn hydrate_status_cache_claude_usage(cache: &mut Value, status_cache_path: &Path) {
    let Some(shared_path) =
        claude_usage_shared_cache_path_from_status_cache_path(status_cache_path)
    else {
        return;
    };
    let Some(shared_cache) = read_claude_usage_shared_cache_value(&shared_path) else {
        return;
    };
    let Some(claude) = shared_cache.get("claude").cloned() else {
        return;
    };
    cache["claude_usage"] = claude;
}

pub(crate) fn hydrate_status_cache_opencode_go_usage(cache: &mut Value, status_cache_path: &Path) {
    let Some(shared_path) =
        opencode_go_usage_shared_cache_path_from_status_cache_path(status_cache_path)
    else {
        return;
    };
    let Some(shared_cache) = read_opencode_go_usage_shared_cache_value(&shared_path) else {
        return;
    };
    let Some(opencode_go) = shared_cache.get("opencode_go").cloned() else {
        return;
    };
    cache["opencode_go_usage"] = opencode_go;
}

pub(crate) fn read_codex_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64) != Some(CODEX_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub(crate) fn read_claude_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64)
        != Some(CLAUDE_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub(crate) fn read_opencode_go_usage_shared_cache_value(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    let cache: Value = serde_json::from_str(&raw).ok()?;
    if cache.get("schema_version").and_then(Value::as_i64)
        != Some(OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION)
    {
        return None;
    }
    Some(cache)
}

pub(crate) fn read_tokenusage_windowed_usage_shared_cache_value(
    path: &Path,
    provider: TokenusageWindowedProvider,
) -> Option<Value> {
    match provider {
        TokenusageWindowedProvider::Claude => read_claude_usage_shared_cache_value(path),
        TokenusageWindowedProvider::Codex => read_codex_usage_shared_cache_value(path),
    }
}

pub(crate) fn tokenusage_windowed_usage_cache_schema_version(
    provider: TokenusageWindowedProvider,
) -> i64 {
    match provider {
        TokenusageWindowedProvider::Claude => CLAUDE_USAGE_CACHE_SCHEMA_VERSION,
        TokenusageWindowedProvider::Codex => CODEX_USAGE_CACHE_SCHEMA_VERSION,
    }
}

pub(crate) fn tokenusage_windowed_usage_cache_key(
    provider: TokenusageWindowedProvider,
) -> &'static str {
    match provider {
        TokenusageWindowedProvider::Claude => "claude",
        TokenusageWindowedProvider::Codex => "codex",
    }
}

pub(crate) fn tokenusage_windowed_usage_label(
    provider: TokenusageWindowedProvider,
) -> &'static str {
    match provider {
        TokenusageWindowedProvider::Claude => "Claude",
        TokenusageWindowedProvider::Codex => "Codex",
    }
}

pub(crate) fn tokenusage_windowed_usage_error_prefix(
    provider: TokenusageWindowedProvider,
) -> &'static str {
    match provider {
        TokenusageWindowedProvider::Claude => "claude_usage_cache",
        TokenusageWindowedProvider::Codex => "codex_usage_cache",
    }
}

pub(crate) fn tokenusage_windowed_usage_lock_name(provider: TokenusageWindowedProvider) -> String {
    match provider {
        TokenusageWindowedProvider::Claude => format!(
            ".claude_usage_cache_v{}.lock",
            CLAUDE_USAGE_CACHE_SCHEMA_VERSION
        ),
        TokenusageWindowedProvider::Codex => format!(
            ".codex_usage_cache_v{}.lock",
            CODEX_USAGE_CACHE_SCHEMA_VERSION
        ),
    }
}

pub(crate) fn tokenusage_windowed_usage_lock_stale_after_seconds(
    provider: TokenusageWindowedProvider,
) -> u64 {
    match provider {
        TokenusageWindowedProvider::Claude => CLAUDE_USAGE_LOCK_STALE_AFTER_SECONDS,
        TokenusageWindowedProvider::Codex => CODEX_USAGE_LOCK_STALE_AFTER_SECONDS,
    }
}

pub(crate) fn status_bar_cache_status_bus(cache: &Value) -> Option<&Value> {
    let status_bus = cache.get("status_bus")?;
    if status_bus.get("schema_version").and_then(Value::as_i64) == Some(STATUS_BUS_SCHEMA_VERSION) {
        Some(status_bus)
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) fn render_status_cache_widget(cache: &Value, widget: &str) -> Result<String, CoreError> {
    render_status_cache_widget_with_agent_usage_settings(
        cache,
        widget,
        &AgentUsageWidgetSettings::default(),
    )
}

pub(crate) fn render_status_cache_widget_with_agent_usage_settings(
    cache: &Value,
    widget: &str,
    settings: &AgentUsageWidgetSettings,
) -> Result<String, CoreError> {
    let status_bus = status_bar_cache_status_bus(cache);
    match widget {
        "workspace" => Ok(status_bus
            .map(render_zjstatus_workspace_widget)
            .unwrap_or_default()),
        "cursor" => Ok(render_zjstatus_cursor_widget(cache)),
        "claude_usage" => Ok(render_windowed_usage_segment(
            cache,
            "claude_usage",
            "claude",
            settings.claude_periods.as_slice(),
            settings.claude_display,
        )),
        "codex_usage" => Ok(render_codex_usage_segment(cache, settings.codex_display)),
        "opencode_go_usage" => Ok(render_windowed_usage_segment(
            cache,
            "opencode_go_usage",
            "go",
            settings.opencode_go_periods.as_slice(),
            settings.opencode_go_display,
        )),
        _ => Err(CoreError::usage(format!(
            "zellij status-cache-widget requires one of: {}",
            status_cache_widget_names().join(", ")
        ))),
    }
}

pub(crate) fn status_cache_widget_names() -> Vec<&'static str> {
    vec![
        "workspace",
        "cursor",
        "claude_usage",
        "codex_usage",
        "opencode_go_usage",
    ]
}

pub(crate) fn render_codex_usage_segment(cache: &Value, display: WindowedUsageDisplay) -> String {
    let Some(entry) = cache.get("codex_usage") else {
        return String::new();
    };
    render_codex_usage_status_widget(
        &bar_windowed_usage_facts_from_cache_entry(entry),
        bar_agent_usage_display(display),
    )
}

pub(crate) fn render_windowed_usage_segment(
    cache: &Value,
    cache_key: &str,
    label: &str,
    periods: &[WindowedUsagePeriod],
    display: WindowedUsageDisplay,
) -> String {
    let Some(entry) = cache.get(cache_key) else {
        return String::new();
    };
    let facts = windowed_usage_facts_from_cache_entry(entry);
    let summary = render_windowed_usage_summary(&facts, periods, display);
    if summary.is_empty() {
        String::new()
    } else {
        render_agent_usage_widget(label, &summary)
    }
}

pub(crate) fn render_windowed_usage_summary(
    facts: &WindowedUsageFacts,
    periods: &[WindowedUsagePeriod],
    display: WindowedUsageDisplay,
) -> String {
    let mut parts = Vec::new();
    for period in periods {
        let (tokens, remaining_percent) = match period {
            WindowedUsagePeriod::FiveHour => {
                (facts.five_hour_tokens, facts.five_hour_remaining_percent)
            }
            WindowedUsagePeriod::Weekly => (facts.weekly_tokens, facts.weekly_remaining_percent),
            WindowedUsagePeriod::Monthly => (facts.monthly_tokens, facts.monthly_remaining_percent),
        };
        if let Some(part) =
            render_windowed_usage_window(period.short_label(), tokens, remaining_percent, display)
        {
            parts.push(part);
        }
    }
    parts.join(" ")
}

pub(crate) fn render_windowed_usage_window(
    label: &str,
    tokens: Option<u64>,
    remaining_percent: Option<u64>,
    display: WindowedUsageDisplay,
) -> Option<String> {
    let mut pieces = vec![label.to_string()];
    match display {
        WindowedUsageDisplay::Token => {
            pieces.push(format_agent_usage_token_count(tokens?));
        }
        WindowedUsageDisplay::Quota => {
            pieces.push(match remaining_percent {
                Some(percent) => format_quota_percent(percent),
                None if tokens.is_some() => "n/a".to_string(),
                None => return None,
            });
        }
        WindowedUsageDisplay::Both => {
            if let Some(tokens) = tokens {
                pieces.push(format_agent_usage_token_count(tokens));
            }
            if let Some(remaining_percent) = remaining_percent {
                pieces.push(format_quota_percent(remaining_percent));
            }
            if pieces.len() == 1 {
                return None;
            }
        }
    }
    Some(pieces.join("|"))
}

pub(crate) fn windowed_usage_facts_from_cache_entry(entry: &Value) -> WindowedUsageFacts {
    WindowedUsageFacts {
        updated_at_unix_seconds: entry.get("updated_at_unix_seconds").and_then(Value::as_u64),
        five_hour_tokens: entry.get("five_hour_tokens").and_then(Value::as_u64),
        weekly_tokens: entry.get("weekly_tokens").and_then(Value::as_u64),
        monthly_tokens: entry.get("monthly_tokens").and_then(Value::as_u64),
        five_hour_remaining_percent: entry
            .get("five_hour_remaining_percent")
            .and_then(Value::as_u64),
        weekly_remaining_percent: entry
            .get("weekly_remaining_percent")
            .and_then(Value::as_u64),
        monthly_remaining_percent: entry
            .get("monthly_remaining_percent")
            .and_then(Value::as_u64),
        five_hour_reset_at_unix_seconds: entry
            .get("five_hour_reset_at_unix_seconds")
            .and_then(Value::as_u64),
        weekly_reset_at_unix_seconds: entry
            .get("weekly_reset_at_unix_seconds")
            .and_then(Value::as_u64),
        five_hour_window_seconds: entry
            .get("five_hour_window_seconds")
            .and_then(Value::as_u64),
        weekly_window_seconds: entry.get("weekly_window_seconds").and_then(Value::as_u64),
        error: entry
            .get("error")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

pub(crate) fn bar_windowed_usage_facts_from_cache_entry(entry: &Value) -> WindowedAgentUsageFacts {
    WindowedAgentUsageFacts {
        updated_at_unix_seconds: entry.get("updated_at_unix_seconds").and_then(Value::as_u64),
        five_hour_tokens: entry.get("five_hour_tokens").and_then(Value::as_u64),
        weekly_tokens: entry.get("weekly_tokens").and_then(Value::as_u64),
        monthly_tokens: entry.get("monthly_tokens").and_then(Value::as_u64),
        five_hour_remaining_percent: entry
            .get("five_hour_remaining_percent")
            .and_then(Value::as_u64),
        weekly_remaining_percent: entry
            .get("weekly_remaining_percent")
            .and_then(Value::as_u64),
        monthly_remaining_percent: entry
            .get("monthly_remaining_percent")
            .and_then(Value::as_u64),
        five_hour_reset_at_unix_seconds: entry
            .get("five_hour_reset_at_unix_seconds")
            .and_then(Value::as_u64),
        weekly_reset_at_unix_seconds: entry
            .get("weekly_reset_at_unix_seconds")
            .and_then(Value::as_u64),
        five_hour_window_seconds: entry
            .get("five_hour_window_seconds")
            .and_then(Value::as_u64),
        weekly_window_seconds: entry.get("weekly_window_seconds").and_then(Value::as_u64),
    }
}

pub(crate) fn bar_agent_usage_display(display: WindowedUsageDisplay) -> BarAgentUsageDisplay {
    match display {
        WindowedUsageDisplay::Both => BarAgentUsageDisplay::Both,
        WindowedUsageDisplay::Token => BarAgentUsageDisplay::Token,
        WindowedUsageDisplay::Quota => BarAgentUsageDisplay::Quota,
    }
}

mod refresh;
pub(crate) use refresh::*;

pub(crate) fn format_agent_usage_token_count(tokens: u64) -> String {
    if tokens >= 1_000_000_000 {
        format_scaled_agent_usage_count(tokens as f64 / 1_000_000_000.0, "B")
    } else if tokens >= 1_000_000 {
        format_scaled_agent_usage_count(tokens as f64 / 1_000_000.0, "M")
    } else if tokens >= 1_000 {
        format!("{}k", tokens / 1_000)
    } else {
        tokens.to_string()
    }
}

pub(crate) fn format_scaled_agent_usage_count(value: f64, suffix: &str) -> String {
    let raw = if value >= 100.0 {
        format!("{value:.0}")
    } else if value >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    };
    let trimmed = if raw.contains('.') {
        raw.trim_end_matches('0').trim_end_matches('.')
    } else {
        raw.as_str()
    };
    format!("{trimmed}{suffix}")
}

pub(crate) fn format_quota_percent(percent: u64) -> String {
    format!("{}%", percent.min(100))
}
