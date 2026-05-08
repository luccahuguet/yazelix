//! Agent-usage cache refresh, external probes, and cache locks.

use super::super::write_json_value_atomic;
use super::*;
use crate::bridge::CoreError;
use rusqlite::{Connection, OpenFlags, params};
use serde_json::{Value, json};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, UNIX_EPOCH};

pub(crate) fn refresh_tokenusage_windowed_usage_shared_cache(
    shared_path: &Path,
    provider: TokenusageWindowedProvider,
    path_var: Option<&OsStr>,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
    timeout: Duration,
) -> Result<bool, CoreError> {
    if tokenusage_windowed_usage_shared_cache_is_fresh(shared_path, provider, now, max_age_seconds)
    {
        return Ok(false);
    }
    if tokenusage_windowed_usage_shared_cache_is_backing_off(shared_path, provider, now) {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_tokenusage_windowed_usage_cache_lock(shared_path, provider, now)?
    else {
        return Ok(false);
    };
    if tokenusage_windowed_usage_shared_cache_is_fresh(shared_path, provider, now, max_age_seconds)
        || tokenusage_windowed_usage_shared_cache_is_backing_off(shared_path, provider, now)
    {
        return Ok(false);
    }

    let quota_backoff_until =
        tokenusage_windowed_usage_quota_backoff_until(shared_path, provider, now);
    let previous_facts = read_tokenusage_windowed_usage_shared_cache_value(shared_path, provider)
        .and_then(|cache| {
            cache
                .get(tokenusage_windowed_usage_cache_key(provider))
                .map(windowed_usage_facts_from_cache_entry)
        });
    let mut facts = collect_tokenusage_windowed_usage_facts(
        provider,
        path_var,
        timeout,
        quota_backoff_until.is_none(),
    );
    let quota_probe_failed = quota_backoff_until.is_none() && !facts.has_quota();
    preserve_previous_tokenusage_window_tokens(provider, &mut facts, previous_facts.as_ref());
    preserve_previous_tokenusage_window_quota(provider, &mut facts, previous_facts.as_ref(), now);
    let mut entry = serde_json::Map::new();
    entry.insert("updated_at_unix_seconds".to_string(), json!(now));
    if let Some(tokens) = facts.five_hour_tokens {
        entry.insert("five_hour_tokens".to_string(), json!(tokens));
    }
    if let Some(tokens) = facts.weekly_tokens {
        entry.insert("weekly_tokens".to_string(), json!(tokens));
    }
    if let Some(percent) = facts.five_hour_remaining_percent {
        entry.insert("five_hour_remaining_percent".to_string(), json!(percent));
    }
    if let Some(percent) = facts.weekly_remaining_percent {
        entry.insert("weekly_remaining_percent".to_string(), json!(percent));
    }
    if let Some(reset_at) = facts.five_hour_reset_at_unix_seconds {
        entry.insert(
            "five_hour_reset_at_unix_seconds".to_string(),
            json!(reset_at),
        );
    }
    if let Some(reset_at) = facts.weekly_reset_at_unix_seconds {
        entry.insert("weekly_reset_at_unix_seconds".to_string(), json!(reset_at));
    }
    if let Some(window_seconds) = facts.five_hour_window_seconds {
        entry.insert(
            "five_hour_window_seconds".to_string(),
            json!(window_seconds),
        );
    }
    if let Some(window_seconds) = facts.weekly_window_seconds {
        entry.insert("weekly_window_seconds".to_string(), json!(window_seconds));
    }
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        entry.insert("error".to_string(), json!(error));
        if facts.is_empty() {
            entry.insert(
                "backoff_until_unix_seconds".to_string(),
                json!(now.saturating_add(error_backoff_seconds)),
            );
        }
    }
    if let Some(backoff_until) = quota_backoff_until {
        entry.insert(
            "quota_backoff_until_unix_seconds".to_string(),
            json!(backoff_until),
        );
    } else if facts.has_tokens() && (quota_probe_failed || !facts.has_quota()) {
        entry.insert(
            "quota_backoff_until_unix_seconds".to_string(),
            json!(now.saturating_add(error_backoff_seconds)),
        );
    }
    let status = if facts.is_empty() {
        "error"
    } else if facts.has_tokens()
        && facts.has_quota()
        && !quota_probe_failed
        && quota_backoff_until.is_none()
    {
        "ok"
    } else {
        "partial"
    };
    entry.insert("status".to_string(), json!(status));

    let cache = json!({
        "schema_version": tokenusage_windowed_usage_cache_schema_version(provider),
        tokenusage_windowed_usage_cache_key(provider): Value::Object(entry),
    });
    write_json_value_atomic(
        shared_path,
        &cache,
        tokenusage_windowed_usage_error_prefix(provider),
    )?;
    Ok(true)
}

pub(crate) fn preserve_previous_tokenusage_window_tokens(
    provider: TokenusageWindowedProvider,
    facts: &mut WindowedUsageFacts,
    previous: Option<&WindowedUsageFacts>,
) {
    let Some(previous) = previous else {
        return;
    };
    if !tokenusage_windowed_usage_facts_are_complete(provider, previous) {
        return;
    }

    if facts.five_hour_tokens.is_none()
        && tokenusage_window_identity_matches(
            facts.five_hour_reset_at_unix_seconds,
            facts.five_hour_window_seconds,
            previous.five_hour_reset_at_unix_seconds,
            previous.five_hour_window_seconds,
        )
    {
        facts.five_hour_tokens = previous.five_hour_tokens;
    }
    if facts.weekly_tokens.is_none()
        && tokenusage_window_identity_matches(
            facts.weekly_reset_at_unix_seconds,
            facts.weekly_window_seconds,
            previous.weekly_reset_at_unix_seconds,
            previous.weekly_window_seconds,
        )
    {
        facts.weekly_tokens = previous.weekly_tokens;
    }
}

pub(crate) fn preserve_previous_tokenusage_window_quota(
    provider: TokenusageWindowedProvider,
    facts: &mut WindowedUsageFacts,
    previous: Option<&WindowedUsageFacts>,
    now: u64,
) {
    let Some(previous) = previous else {
        return;
    };

    if facts.five_hour_remaining_percent.is_none()
        && previous_quota_window_is_still_valid(
            provider,
            previous.five_hour_reset_at_unix_seconds,
            previous.five_hour_window_seconds,
            now,
        )
    {
        facts.five_hour_remaining_percent = previous.five_hour_remaining_percent;
        facts.five_hour_reset_at_unix_seconds = previous.five_hour_reset_at_unix_seconds;
        facts.five_hour_window_seconds = previous.five_hour_window_seconds;
    }
    if facts.weekly_remaining_percent.is_none()
        && previous_quota_window_is_still_valid(
            provider,
            previous.weekly_reset_at_unix_seconds,
            previous.weekly_window_seconds,
            now,
        )
    {
        facts.weekly_remaining_percent = previous.weekly_remaining_percent;
        facts.weekly_reset_at_unix_seconds = previous.weekly_reset_at_unix_seconds;
        facts.weekly_window_seconds = previous.weekly_window_seconds;
    }
}

pub(crate) fn previous_quota_window_is_still_valid(
    provider: TokenusageWindowedProvider,
    reset_at_unix_seconds: Option<u64>,
    window_seconds: Option<u64>,
    now: u64,
) -> bool {
    match provider {
        TokenusageWindowedProvider::Claude => true,
        TokenusageWindowedProvider::Codex => {
            reset_at_unix_seconds.is_some_and(|reset_at| now < reset_at)
                && window_seconds.is_some_and(|seconds| seconds > 0)
        }
    }
}

pub(crate) fn tokenusage_window_identity_matches(
    reset_at: Option<u64>,
    window_seconds: Option<u64>,
    previous_reset_at: Option<u64>,
    previous_window_seconds: Option<u64>,
) -> bool {
    reset_at.is_some()
        && window_seconds.is_some()
        && reset_at == previous_reset_at
        && window_seconds == previous_window_seconds
}

#[cfg(test)]
pub(crate) fn codex_usage_shared_cache_is_fresh(
    path: &Path,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    tokenusage_windowed_usage_shared_cache_is_fresh(
        path,
        TokenusageWindowedProvider::Codex,
        now,
        max_age_seconds,
    )
}

pub(crate) fn tokenusage_windowed_usage_facts_are_complete(
    provider: TokenusageWindowedProvider,
    facts: &WindowedUsageFacts,
) -> bool {
    let has_token_and_quota = facts.five_hour_tokens.is_some()
        && facts.weekly_tokens.is_some()
        && facts.five_hour_remaining_percent.is_some()
        && facts.weekly_remaining_percent.is_some();
    let has_reset_window = match provider {
        TokenusageWindowedProvider::Claude => true,
        TokenusageWindowedProvider::Codex => {
            facts.five_hour_reset_at_unix_seconds.is_some()
                && facts.weekly_reset_at_unix_seconds.is_some()
                && facts.five_hour_window_seconds.is_some()
                && facts.weekly_window_seconds.is_some()
        }
    };
    has_token_and_quota && has_reset_window
}

#[cfg(test)]
pub(crate) fn codex_usage_shared_cache_is_backing_off(path: &Path, now: u64) -> bool {
    tokenusage_windowed_usage_shared_cache_is_backing_off(
        path,
        TokenusageWindowedProvider::Codex,
        now,
    )
}

pub(crate) fn tokenusage_windowed_usage_shared_cache_is_fresh(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    let Some(cache) = read_tokenusage_windowed_usage_shared_cache_value(path, provider) else {
        return false;
    };
    let cache_key = tokenusage_windowed_usage_cache_key(provider);
    cache
        .get(cache_key)
        .and_then(|entry| entry.get("updated_at_unix_seconds"))
        .and_then(Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get(cache_key)
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| {
                        tokenusage_windowed_usage_facts_are_complete(provider, &facts)
                    })
        })
}

pub(crate) fn tokenusage_windowed_usage_shared_cache_is_backing_off(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> bool {
    read_tokenusage_windowed_usage_shared_cache_value(path, provider)
        .and_then(|cache| {
            let entry = cache.get(tokenusage_windowed_usage_cache_key(provider))?;
            let facts = windowed_usage_facts_from_cache_entry(entry);
            if !facts.is_empty() && !tokenusage_windowed_usage_facts_are_complete(provider, &facts)
            {
                return None;
            }
            entry.get("backoff_until_unix_seconds")?.as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

#[cfg(test)]
pub(crate) fn tokenusage_windowed_usage_quota_is_backing_off(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> bool {
    tokenusage_windowed_usage_quota_backoff_until(path, provider, now).is_some()
}

pub(crate) fn tokenusage_windowed_usage_quota_backoff_until(
    path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> Option<u64> {
    read_tokenusage_windowed_usage_shared_cache_value(path, provider)
        .and_then(|cache| {
            cache
                .get(tokenusage_windowed_usage_cache_key(provider))?
                .get("quota_backoff_until_unix_seconds")?
                .as_u64()
        })
        .filter(|backoff_until| now < *backoff_until)
}

pub(crate) struct AgentUsageCacheLock {
    path: PathBuf,
}

impl Drop for AgentUsageCacheLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

pub(crate) fn try_acquire_tokenusage_windowed_usage_cache_lock(
    shared_path: &Path,
    provider: TokenusageWindowedProvider,
    now: u64,
) -> Result<Option<AgentUsageCacheLock>, CoreError> {
    let error_prefix = tokenusage_windowed_usage_error_prefix(provider);
    let provider_label = tokenusage_windowed_usage_label(provider);
    try_acquire_agent_usage_cache_lock(
        shared_path.with_file_name(tokenusage_windowed_usage_lock_name(provider)),
        now,
        tokenusage_windowed_usage_lock_stale_after_seconds(provider),
        &format!("{error_prefix}_lock_parent_create_failed"),
        &format!("Failed to create the Yazelix {provider_label} usage cache lock directory."),
        &format!("{error_prefix}_lock_create_failed"),
        &format!("Failed to acquire the Yazelix {provider_label} usage cache lock."),
    )
}

pub(crate) fn refresh_opencode_go_usage_shared_cache(
    shared_path: &Path,
    now: u64,
    max_age_seconds: u64,
    error_backoff_seconds: u64,
) -> Result<bool, CoreError> {
    if opencode_go_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds) {
        return Ok(false);
    }
    if opencode_go_usage_shared_cache_is_backing_off(shared_path, now) {
        return Ok(false);
    }
    let Some(_lock) = try_acquire_opencode_go_usage_cache_lock(shared_path, now)? else {
        return Ok(false);
    };
    if opencode_go_usage_shared_cache_is_fresh(shared_path, now, max_age_seconds)
        || opencode_go_usage_shared_cache_is_backing_off(shared_path, now)
    {
        return Ok(false);
    }

    let db_paths = opencode_db_candidates_from_env();
    refresh_opencode_go_usage_shared_cache_from_dbs(
        shared_path,
        &db_paths,
        now,
        error_backoff_seconds,
    )
}

pub(crate) fn refresh_opencode_go_usage_shared_cache_from_dbs(
    shared_path: &Path,
    db_paths: &[PathBuf],
    now: u64,
    error_backoff_seconds: u64,
) -> Result<bool, CoreError> {
    let facts = collect_opencode_go_usage_facts_from_dbs(db_paths, now);
    let mut opencode_go = serde_json::Map::new();
    opencode_go.insert("updated_at_unix_seconds".to_string(), json!(now));
    if let Some(tokens) = facts.five_hour_tokens {
        opencode_go.insert("five_hour_tokens".to_string(), json!(tokens));
    }
    if let Some(tokens) = facts.weekly_tokens {
        opencode_go.insert("weekly_tokens".to_string(), json!(tokens));
    }
    if let Some(tokens) = facts.monthly_tokens {
        opencode_go.insert("monthly_tokens".to_string(), json!(tokens));
    }
    if let Some(percent) = facts.five_hour_remaining_percent {
        opencode_go.insert("five_hour_remaining_percent".to_string(), json!(percent));
    }
    if let Some(percent) = facts.weekly_remaining_percent {
        opencode_go.insert("weekly_remaining_percent".to_string(), json!(percent));
    }
    if let Some(percent) = facts.monthly_remaining_percent {
        opencode_go.insert("monthly_remaining_percent".to_string(), json!(percent));
    }
    if let Some(error) = facts.error.as_deref().filter(|value| !value.is_empty()) {
        opencode_go.insert("error".to_string(), json!(error));
        opencode_go.insert(
            "backoff_until_unix_seconds".to_string(),
            json!(now.saturating_add(error_backoff_seconds)),
        );
    }
    let status = if facts.is_empty() {
        "error"
    } else if facts.has_tokens() && facts.has_quota() {
        "ok"
    } else {
        "partial"
    };
    opencode_go.insert("status".to_string(), json!(status));

    let cache = json!({
        "schema_version": OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION,
        "opencode_go": Value::Object(opencode_go),
    });
    write_json_value_atomic(shared_path, &cache, "opencode_go_usage_cache")?;
    Ok(true)
}

pub(crate) fn opencode_go_usage_shared_cache_is_fresh(
    path: &Path,
    now: u64,
    max_age_seconds: u64,
) -> bool {
    let Some(cache) = read_opencode_go_usage_shared_cache_value(path) else {
        return false;
    };
    cache
        .get("opencode_go")
        .and_then(|opencode_go| opencode_go.get("updated_at_unix_seconds"))
        .and_then(Value::as_u64)
        .is_some_and(|updated_at| {
            now.saturating_sub(updated_at) < max_age_seconds
                && cache
                    .get("opencode_go")
                    .map(windowed_usage_facts_from_cache_entry)
                    .is_some_and(|facts| opencode_go_usage_facts_are_complete(&facts))
        })
}

pub(crate) fn opencode_go_usage_facts_are_complete(facts: &WindowedUsageFacts) -> bool {
    facts.five_hour_tokens.is_some()
        && facts.weekly_tokens.is_some()
        && facts.monthly_tokens.is_some()
        && facts.five_hour_remaining_percent.is_some()
        && facts.weekly_remaining_percent.is_some()
        && facts.monthly_remaining_percent.is_some()
}

pub(crate) fn opencode_go_usage_shared_cache_is_backing_off(path: &Path, now: u64) -> bool {
    read_opencode_go_usage_shared_cache_value(path)
        .and_then(|cache| {
            cache
                .get("opencode_go")?
                .get("backoff_until_unix_seconds")?
                .as_u64()
        })
        .is_some_and(|backoff_until| now < backoff_until)
}

pub(crate) fn try_acquire_opencode_go_usage_cache_lock(
    shared_path: &Path,
    now: u64,
) -> Result<Option<AgentUsageCacheLock>, CoreError> {
    let lock_path = shared_path.with_file_name(opencode_go_usage_lock_name());
    try_acquire_agent_usage_cache_lock(
        lock_path,
        now,
        OPENCODE_GO_USAGE_LOCK_STALE_AFTER_SECONDS,
        "opencode_go_usage_cache_lock_parent_create_failed",
        "Failed to create the Yazelix OpenCode Go usage cache lock directory.",
        "opencode_go_usage_cache_lock_create_failed",
        "Failed to acquire the Yazelix OpenCode Go usage cache lock.",
    )
}

pub(crate) fn opencode_go_usage_lock_name() -> String {
    format!(".opencode_go_usage_cache_v{OPENCODE_GO_USAGE_CACHE_SCHEMA_VERSION}.lock")
}

pub(crate) fn try_acquire_agent_usage_cache_lock(
    lock_path: PathBuf,
    now: u64,
    stale_after_seconds: u64,
    parent_error_code: &str,
    parent_error_message: &str,
    create_error_code: &str,
    create_error_message: &str,
) -> Result<Option<AgentUsageCacheLock>, CoreError> {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                parent_error_code,
                parent_error_message,
                "Check permissions for the Yazelix state directory, then retry.",
                &parent.display().to_string(),
                source,
            )
        })?;
    }
    match fs::create_dir(&lock_path) {
        Ok(()) => Ok(Some(AgentUsageCacheLock { path: lock_path })),
        Err(source) if source.kind() == ErrorKind::AlreadyExists => {
            if agent_usage_cache_lock_is_stale(&lock_path, now, stale_after_seconds) {
                let _ = fs::remove_dir(&lock_path);
                return match fs::create_dir(&lock_path) {
                    Ok(()) => Ok(Some(AgentUsageCacheLock { path: lock_path })),
                    Err(source) if source.kind() == ErrorKind::AlreadyExists => Ok(None),
                    Err(source) => Err(CoreError::io(
                        create_error_code,
                        create_error_message,
                        "Check permissions for the Yazelix state directory, then retry.",
                        &lock_path.display().to_string(),
                        source,
                    )),
                };
            }
            Ok(None)
        }
        Err(source) => Err(CoreError::io(
            create_error_code,
            create_error_message,
            "Check permissions for the Yazelix state directory, then retry.",
            &lock_path.display().to_string(),
            source,
        )),
    }
}

pub(crate) fn agent_usage_cache_lock_is_stale(
    lock_path: &Path,
    now: u64,
    stale_after_seconds: u64,
) -> bool {
    fs::metadata(lock_path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| now.saturating_sub(duration.as_secs()) > stale_after_seconds)
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct OpenCodeGoUsageWindow {
    tokens: u64,
    cost_usd: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct OpenCodeGoUsageWindows {
    five_hour: OpenCodeGoUsageWindow,
    weekly: OpenCodeGoUsageWindow,
    monthly: OpenCodeGoUsageWindow,
}

pub(crate) fn collect_opencode_go_usage_facts_from_dbs(
    db_paths: &[PathBuf],
    now: u64,
) -> WindowedUsageFacts {
    if db_paths.is_empty() {
        return WindowedUsageFacts {
            error: Some("missing OpenCode DB".to_string()),
            ..WindowedUsageFacts::default()
        };
    }

    let mut five_hour = OpenCodeGoUsageWindow::default();
    let mut weekly = OpenCodeGoUsageWindow::default();
    let mut monthly = OpenCodeGoUsageWindow::default();
    let mut opened_any = false;
    let mut first_error = None;

    for path in db_paths {
        match collect_opencode_go_usage_windows_from_db(path, now) {
            Ok(db_windows) => {
                opened_any = true;
                five_hour.tokens = five_hour.tokens.saturating_add(db_windows.five_hour.tokens);
                five_hour.cost_usd += db_windows.five_hour.cost_usd;
                weekly.tokens = weekly.tokens.saturating_add(db_windows.weekly.tokens);
                weekly.cost_usd += db_windows.weekly.cost_usd;
                monthly.tokens = monthly.tokens.saturating_add(db_windows.monthly.tokens);
                monthly.cost_usd += db_windows.monthly.cost_usd;
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(format!("{}: {error}", path.display()));
                }
            }
        }
    }

    if !opened_any {
        return WindowedUsageFacts {
            error: first_error.or_else(|| Some("OpenCode DB unavailable".to_string())),
            ..WindowedUsageFacts::default()
        };
    }

    let mut facts = WindowedUsageFacts::default();
    facts.five_hour_tokens = Some(five_hour.tokens);
    facts.five_hour_remaining_percent = Some(remaining_percent_from_cost_limit(
        five_hour.cost_usd,
        OPENCODE_GO_FIVE_HOUR_LIMIT_USD,
    ));
    facts.weekly_tokens = Some(weekly.tokens);
    facts.weekly_remaining_percent = Some(remaining_percent_from_cost_limit(
        weekly.cost_usd,
        OPENCODE_GO_WEEKLY_LIMIT_USD,
    ));
    facts.monthly_tokens = Some(monthly.tokens);
    facts.monthly_remaining_percent = Some(remaining_percent_from_cost_limit(
        monthly.cost_usd,
        OPENCODE_GO_MONTHLY_LIMIT_USD,
    ));
    if facts.is_empty() {
        facts.error = Some("OpenCode Go usage unavailable".to_string());
    }
    facts
}

pub(crate) fn collect_opencode_go_usage_windows_from_db(
    path: &Path,
    now: u64,
) -> Result<OpenCodeGoUsageWindows, String> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("failed to open OpenCode DB read-only: {error}"))?;
    connection
        .busy_timeout(Duration::from_millis(250))
        .map_err(|error| format!("failed to configure OpenCode DB read timeout: {error}"))?;
    let five_hour = query_opencode_go_usage_window(
        &connection,
        now.saturating_sub(OPENCODE_GO_FIVE_HOUR_SECONDS),
    )?;
    let weekly =
        query_opencode_go_usage_window(&connection, now.saturating_sub(OPENCODE_GO_WEEK_SECONDS))?;
    let monthly =
        query_opencode_go_usage_window(&connection, now.saturating_sub(OPENCODE_GO_MONTH_SECONDS))?;
    Ok(OpenCodeGoUsageWindows {
        five_hour,
        weekly,
        monthly,
    })
}

pub(crate) fn query_opencode_go_usage_window(
    connection: &Connection,
    since_unix_seconds: u64,
) -> Result<OpenCodeGoUsageWindow, String> {
    connection
        .query_row(
            r#"
            SELECT
              COALESCE(SUM(
                COALESCE(
                  json_extract(data, '$.tokens.total'),
                  COALESCE(json_extract(data, '$.tokens.input'), 0) +
                  COALESCE(json_extract(data, '$.tokens.output'), 0) +
                  COALESCE(json_extract(data, '$.tokens.reasoning'), 0) +
                  COALESCE(json_extract(data, '$.tokens.cache.read'), 0) +
                  COALESCE(json_extract(data, '$.tokens.cache.write'), 0)
                )
              ), 0),
              COALESCE(SUM(COALESCE(json_extract(data, '$.cost'), 0.0)), 0.0)
            FROM message
            WHERE time_created >= ?1
              AND json_extract(data, '$.role') = 'assistant'
              AND json_extract(data, '$.providerID') = ?2
            "#,
            params![
                unix_millis_from_seconds_saturating(since_unix_seconds),
                OPENCODE_GO_PROVIDER_ID
            ],
            |row| {
                Ok(OpenCodeGoUsageWindow {
                    tokens: row.get::<_, i64>(0)?.max(0) as u64,
                    cost_usd: row.get::<_, f64>(1)?.max(0.0),
                })
            },
        )
        .map_err(|error| format!("failed to read OpenCode Go usage window: {error}"))
}

pub(crate) fn unix_millis_from_seconds_saturating(seconds: u64) -> i64 {
    seconds.saturating_mul(1000).min(i64::MAX as u64) as i64
}

pub(crate) fn remaining_percent_from_cost_limit(cost_usd: f64, limit_usd: f64) -> u64 {
    if limit_usd <= 0.0 {
        return 0;
    }
    (100.0 - (cost_usd / limit_usd * 100.0))
        .clamp(0.0, 100.0)
        .round() as u64
}

pub(crate) fn opencode_db_candidates_from_env() -> Vec<PathBuf> {
    opencode_db_candidates_from_values(
        env::var_os("OPENCODE_DB").map(PathBuf::from),
        env::var_os("OPENCODE_DATA_DIR").map(PathBuf::from),
        env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        env::var_os("HOME").map(PathBuf::from),
    )
}

pub(crate) fn opencode_db_candidates_from_values(
    opencode_db: Option<PathBuf>,
    opencode_data_dir: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> Vec<PathBuf> {
    let data_dir = opencode_data_dir
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| {
            xdg_data_home
                .filter(|path| !path.as_os_str().is_empty())
                .map(|path| path.join("opencode"))
        })
        .or_else(|| {
            home.filter(|path| !path.as_os_str().is_empty())
                .map(|path| path.join(".local").join("share").join("opencode"))
        });

    let mut candidates = Vec::new();
    if let Some(path) = opencode_db.filter(|path| !path.as_os_str().is_empty()) {
        if path.is_absolute() {
            push_unique_path(&mut candidates, path);
        } else if let Some(data_dir) = data_dir.as_ref() {
            push_unique_path(&mut candidates, data_dir.join(path));
        } else {
            push_unique_path(&mut candidates, path);
        }
    }

    if let Some(data_dir) = data_dir {
        push_unique_path(&mut candidates, data_dir.join("opencode.db"));
        if let Ok(entries) = fs::read_dir(data_dir) {
            let mut channel_dbs = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with("opencode-") && name.ends_with(".db"))
                })
                .collect::<Vec<_>>();
            channel_dbs.sort();
            for path in channel_dbs {
                push_unique_path(&mut candidates, path);
            }
        }
    }

    candidates
}

pub(crate) fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

pub(crate) fn collect_tokenusage_windowed_usage_facts(
    provider: TokenusageWindowedProvider,
    path_var: Option<&OsStr>,
    timeout: Duration,
    include_quota: bool,
) -> WindowedUsageFacts {
    let Some(path_var) = path_var else {
        return WindowedUsageFacts {
            error: Some("missing PATH".to_string()),
            ..WindowedUsageFacts::default()
        };
    };
    let Some(binary_path) = find_command_in_path_var(path_var, "tu") else {
        return WindowedUsageFacts {
            error: Some("missing tu".to_string()),
            ..WindowedUsageFacts::default()
        };
    };

    let mut facts = WindowedUsageFacts::default();
    match run_tokenusage_json_command(
        &binary_path,
        tokenusage_active_block_args(provider).as_slice(),
        timeout,
    ) {
        Ok(Some(value)) => {
            facts.five_hour_tokens = tokenusage_active_block_tokens_from_json(&value);
        }
        Ok(None) => facts.error = Some("active block unavailable".to_string()),
        Err(error) => facts.error = Some(error),
    }

    match run_tokenusage_json_command(
        &binary_path,
        tokenusage_weekly_args(provider).as_slice(),
        timeout,
    ) {
        Ok(Some(value)) => {
            facts.weekly_tokens = tokenusage_weekly_tokens_from_json(&value);
        }
        Ok(None) => {
            facts.error = facts
                .error
                .or_else(|| Some("weekly usage unavailable".to_string()))
        }
        Err(error) => facts.error = facts.error.or(Some(error)),
    }

    if include_quota {
        match run_tokenusage_json_command(
            &binary_path,
            tokenusage_official_limits_args(provider).as_slice(),
            timeout,
        ) {
            Ok(Some(value)) => {
                let quota = tokenusage_quota_from_official_json(&value, provider);
                facts.five_hour_remaining_percent = quota.five_hour_remaining_percent;
                facts.weekly_remaining_percent = quota.weekly_remaining_percent;
                facts.five_hour_reset_at_unix_seconds = quota.five_hour_reset_at_unix_seconds;
                facts.weekly_reset_at_unix_seconds = quota.weekly_reset_at_unix_seconds;
                facts.five_hour_window_seconds = quota.five_hour_window_seconds;
                facts.weekly_window_seconds = quota.weekly_window_seconds;
                if !quota.has_quota() {
                    facts.error = facts
                        .error
                        .or_else(|| Some("quota unavailable".to_string()));
                }
            }
            Ok(None) => {
                facts.error = facts
                    .error
                    .or_else(|| Some("quota unavailable".to_string()))
            }
            Err(error) => facts.error = facts.error.or(Some(error)),
        }
    }

    facts
}

pub(crate) fn tokenusage_active_block_args(
    provider: TokenusageWindowedProvider,
) -> Vec<&'static str> {
    let mut args = vec!["blocks", "--active", "--json", "--offline"];
    args.extend(tokenusage_disabled_source_args(provider));
    args
}

pub(crate) fn tokenusage_weekly_args(provider: TokenusageWindowedProvider) -> Vec<&'static str> {
    let mut args = vec!["weekly", "--json", "--offline"];
    args.extend(tokenusage_disabled_source_args(provider));
    args.extend(["--order", "desc"]);
    args
}

pub(crate) fn tokenusage_official_limits_args(
    provider: TokenusageWindowedProvider,
) -> Vec<&'static str> {
    let mut args = vec!["blocks", "--active", "--json", "--official-limits"];
    args.extend(tokenusage_disabled_source_args(provider));
    args
}

pub(crate) fn tokenusage_disabled_source_args(
    provider: TokenusageWindowedProvider,
) -> &'static [&'static str] {
    match provider {
        TokenusageWindowedProvider::Claude => &["--no-codex", "--no-antigravity"],
        TokenusageWindowedProvider::Codex => &["--no-claude", "--no-antigravity"],
    }
}

pub(crate) fn run_tokenusage_json_command(
    binary_path: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<Option<Value>, String> {
    let output = run_agent_usage_command_with_timeout(binary_path, args, timeout)
        .map_err(|error| error.to_string())?;
    let Some(output) = output else {
        return Ok(None);
    };
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(json_raw) = extract_json_object(&stdout) else {
        return Ok(None);
    };
    serde_json::from_str::<Value>(json_raw)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub(crate) fn tokenusage_active_block_tokens_from_json(value: &Value) -> Option<u64> {
    value
        .get("blocks")
        .and_then(Value::as_array)
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|block| {
                    block
                        .get("isActive")
                        .or_else(|| block.get("is_active"))
                        .and_then(Value::as_bool)
                        == Some(true)
                })
                .or_else(|| blocks.first())
        })
        .and_then(|block| {
            first_u64_at(
                block,
                &[
                    &["totals", "total_tokens"],
                    &["totals", "totalTokens"],
                    &["total_tokens"],
                    &["totalTokens"],
                ],
            )
        })
}

pub(crate) fn tokenusage_weekly_tokens_from_json(value: &Value) -> Option<u64> {
    value
        .get("weekly")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| {
            first_u64_at(
                row,
                &[
                    &["totals", "total_tokens"],
                    &["totals", "totalTokens"],
                    &["total_tokens"],
                    &["totalTokens"],
                ],
            )
        })
}

pub(crate) fn tokenusage_quota_from_official_json(
    value: &Value,
    provider: TokenusageWindowedProvider,
) -> WindowedUsageFacts {
    let official_key = match provider {
        TokenusageWindowedProvider::Claude => "official_claude",
        TokenusageWindowedProvider::Codex => "official_codex",
    };
    let Some(official) = value.get(official_key) else {
        return WindowedUsageFacts::default();
    };
    WindowedUsageFacts {
        five_hour_remaining_percent: official
            .get("primary_used_percent")
            .and_then(Value::as_f64)
            .map(remaining_percent_from_used),
        weekly_remaining_percent: official
            .get("secondary_used_percent")
            .and_then(Value::as_f64)
            .map(remaining_percent_from_used),
        five_hour_reset_at_unix_seconds: official.get("primary_resets_at").and_then(Value::as_u64),
        weekly_reset_at_unix_seconds: official.get("secondary_resets_at").and_then(Value::as_u64),
        five_hour_window_seconds: official
            .get("primary_window_mins")
            .and_then(Value::as_u64)
            .and_then(window_minutes_to_seconds),
        weekly_window_seconds: official
            .get("secondary_window_mins")
            .and_then(Value::as_u64)
            .and_then(window_minutes_to_seconds),
        ..WindowedUsageFacts::default()
    }
}

pub(crate) fn window_minutes_to_seconds(minutes: u64) -> Option<u64> {
    minutes
        .checked_mul(MINUTE_SECONDS)
        .filter(|seconds| *seconds > 0)
}

pub(crate) fn remaining_percent_from_used(used_percent: f64) -> u64 {
    (100.0 - used_percent).clamp(0.0, 100.0).round() as u64
}

pub(crate) fn normalized_session_config_from_env() -> Option<Value> {
    let path = session_config_path_from_env()?;
    normalized_session_config_from_path(&path)
}

pub(crate) fn normalized_session_config_for_status_cache_path(
    status_cache_path: &Path,
) -> Option<Value> {
    normalized_session_config_from_status_cache_path(status_cache_path)
        .or_else(normalized_session_config_from_env)
}

pub(crate) fn normalized_session_config_from_status_cache_path(
    status_cache_path: &Path,
) -> Option<Value> {
    let path = session_config_path_from_values(None, Some(status_cache_path.to_path_buf()))?;
    normalized_session_config_from_path(&path)
}

pub(crate) fn normalized_session_config_from_path(path: &Path) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&raw)
        .ok()?
        .get("normalized_config")
        .cloned()
}

pub(crate) fn agent_usage_widget_settings_from_status_cache_path(
    status_cache_path: &Path,
) -> AgentUsageWidgetSettings {
    let Some(config) = normalized_session_config_for_status_cache_path(status_cache_path) else {
        return AgentUsageWidgetSettings::default();
    };
    AgentUsageWidgetSettings {
        claude_display: config
            .get("zellij_claude_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Both),
        codex_display: config
            .get("zellij_codex_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Quota),
        opencode_go_display: config
            .get("zellij_opencode_go_usage_display")
            .and_then(Value::as_str)
            .map(WindowedUsageDisplay::parse)
            .unwrap_or(WindowedUsageDisplay::Both),
        claude_periods: windowed_usage_periods_from_config(
            &config,
            "zellij_claude_usage_periods",
            default_windowed_usage_periods(),
        ),
        opencode_go_periods: windowed_usage_periods_from_config(
            &config,
            "zellij_opencode_go_usage_periods",
            &default_opencode_go_usage_periods(),
        ),
    }
}

pub(crate) fn usage_widget_enabled_from_status_cache_path(
    status_cache_path: &Path,
    widget: &str,
) -> bool {
    normalized_session_config_for_status_cache_path(status_cache_path)
        .and_then(|config| agent_usage_widget_names_from_config(&config))
        .is_some_and(|widgets| widgets.contains(widget))
}

pub(crate) fn agent_usage_widget_names_from_config(config: &Value) -> Option<BTreeSet<String>> {
    Some(
        config
            .get("zellij_widget_tray")?
            .as_array()?
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|widget| !widget.is_empty())
            .map(str::to_string)
            .collect(),
    )
}

pub(crate) fn windowed_usage_periods_from_config(
    config: &Value,
    key: &str,
    default_periods: &[WindowedUsagePeriod],
) -> Vec<WindowedUsagePeriod> {
    let periods = config
        .get(key)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .filter_map(WindowedUsagePeriod::parse_config)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if periods.is_empty() {
        default_periods.to_vec()
    } else {
        periods
    }
}

pub(crate) fn run_agent_usage_command_with_timeout(
    binary_path: &Path,
    args: &[&str],
    timeout: Duration,
) -> std::io::Result<Option<std::process::Output>> {
    let mut child = Command::new(binary_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let started = Instant::now();
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output().map(Some);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(25));
    }
}

pub(crate) fn find_command_in_path_var(path_var: &OsStr, command_name: &str) -> Option<PathBuf> {
    env::split_paths(path_var)
        .map(|entry| entry.join(command_name))
        .find(|candidate| candidate.is_file())
}

pub(crate) fn extract_json_object(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    (start <= end).then_some(&raw[start..=end])
}

pub(crate) fn render_agent_usage_widget(label: &str, summary: &str) -> String {
    format!(" [{label} {summary}]")
}

pub(crate) fn first_u64_at(value: &Value, paths: &[&[&str]]) -> Option<u64> {
    paths
        .iter()
        .find_map(|path| nested_value(value, path)?.as_u64())
}

pub(crate) fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}
