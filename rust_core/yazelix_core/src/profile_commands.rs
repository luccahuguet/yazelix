// Test lane: default
//! Startup profile orchestration for `yzx_control profile`.
//!
//! Crate decision: Added `time` crate for RFC3339 timestamp formatting.
//! Rejected: `chrono` (heavier, more features than needed), manual calendar math
//! (error-prone and wasteful for a solved problem), shelling out to `date`
//! (adds ~1ms overhead per call which distorts profiling). `time` is the minimal
//! correct choice for ISO8601/RFC3339 formatting in Rust.

use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::state_dir_from_env;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: i32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunRecord {
    #[serde(rename = "type")]
    record_type: String,
    schema_version: i32,
    run_id: String,
    scenario: String,
    created_at: String,
    #[serde(skip_serializing_if = "serde_json::Value::is_null", default)]
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StepRecord {
    #[serde(rename = "type")]
    record_type: String,
    schema_version: i32,
    run_id: String,
    scenario: String,
    component: String,
    step: String,
    started_ns: i64,
    ended_ns: i64,
    duration_ms: f64,
    recorded_at: String,
    #[serde(skip_serializing_if = "serde_json::Value::is_null", default)]
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileRunInfo {
    run_id: String,
    report_path: PathBuf,
    env: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileSummary {
    run: serde_json::Value,
    steps: Vec<serde_json::Value>,
    total_duration_ms: f64,
    report_path: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct StepComparisonKey {
    phase: String,
    component: String,
    step: String,
}

#[derive(Debug, Clone)]
struct StepComparison {
    key: StepComparisonKey,
    baseline_ms: Option<f64>,
    candidate_ms: Option<f64>,
    delta_ms: Option<f64>,
    delta_percent: Option<f64>,
}

#[derive(Debug, Clone)]
struct ProfileComparison {
    baseline: ProfileSummary,
    candidate: ProfileSummary,
    total_delta_ms: f64,
    total_delta_percent: Option<f64>,
    steps: Vec<StepComparison>,
}

fn now_rfc3339() -> String {
    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    let format = time::format_description::well_known::Rfc3339;
    now.format(&format).unwrap_or_else(|_| {
        time::OffsetDateTime::now_utc()
            .format(&format)
            .unwrap_or_default()
    })
}

fn generate_run_id() -> String {
    let now = time::OffsetDateTime::now_utc();
    let format = time::format_description::parse(
        "[year][month][day]_[hour][minute][second]_[subsecond digits:3]",
    )
    .unwrap_or_else(|_| {
        time::format_description::parse("[year][month][day]_[hour][minute][second]").unwrap()
    });
    let timestamp = now.format(&format).unwrap_or_default();
    format!("startup_profile_{}", timestamp)
}

fn round_ms(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn percent_delta(baseline: f64, candidate: f64) -> Option<f64> {
    if baseline.abs() < f64::EPSILON {
        None
    } else {
        Some(((candidate - baseline) / baseline * 100.0 * 10.0).round() / 10.0)
    }
}

fn append_jsonl(path: &Path, value: &serde_json::Value) -> Result<(), CoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| {
            CoreError::io(
                "profile_append",
                &format!(
                    "Could not open profile report for append: {}",
                    path.display()
                ),
                "Check directory permissions and disk space.",
                path.display().to_string(),
                source,
            )
        })?;

    let line = format!(
        "{}\n",
        serde_json::to_string(value).map_err(|e| {
            CoreError::classified(
                ErrorClass::Internal,
                "profile_serialize",
                &format!("Failed to serialize profile record: {}", e),
                "This is a bug; please report it.",
                serde_json::Value::Null,
            )
        })?
    );

    file.write_all(line.as_bytes()).map_err(|source| {
        CoreError::io(
            "profile_write",
            &format!("Could not write profile record: {}", path.display()),
            "Check disk space and permissions.",
            path.display().to_string(),
            source,
        )
    })?;

    Ok(())
}

fn parse_metadata(raw: Option<&str>) -> Result<serde_json::Value, CoreError> {
    match raw {
        Some(s) if !s.trim().is_empty() => serde_json::from_str(s)
            .map_err(|e| CoreError::usage(format!("Invalid metadata JSON: {}", e))),
        _ => Ok(serde_json::Value::Null),
    }
}

fn load_report_data(report_path: &Path) -> Result<ProfileSummary, CoreError> {
    if !report_path.exists() {
        return Err(CoreError::io(
            "profile_load",
            &format!(
                "Startup profile report not found: {}",
                report_path.display()
            ),
            "Check the report path and run the profiler first.",
            report_path.display().to_string(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing report"),
        ));
    }

    let content = std::fs::read_to_string(report_path).map_err(|source| {
        CoreError::io(
            "profile_read",
            &format!("Could not read profile report: {}", report_path.display()),
            "Check file permissions.",
            report_path.display().to_string(),
            source,
        )
    })?;

    let mut run_record: Option<serde_json::Value> = None;
    let mut step_records: Vec<serde_json::Value> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let record: serde_json::Value = serde_json::from_str(line).map_err(|e| {
            CoreError::classified(
                ErrorClass::Internal,
                "profile_parse",
                &format!("Invalid JSON in profile report: {}", e),
                "Check for corrupt report files.",
                serde_json::json!({"line": line}),
            )
        })?;

        match record.get("type").and_then(|v| v.as_str()) {
            Some("run") => run_record = Some(record),
            Some("step") => step_records.push(record),
            _ => {}
        }
    }

    let run = run_record.ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Internal,
            "profile_schema",
            "Startup profile report is missing a run header",
            "The report file may be corrupt or empty.",
            serde_json::Value::Null,
        )
    })?;

    step_records.sort_by(|a, b| {
        let a_ns = a.get("started_ns").and_then(|v| v.as_i64()).unwrap_or(0);
        let b_ns = b.get("started_ns").and_then(|v| v.as_i64()).unwrap_or(0);
        a_ns.cmp(&b_ns)
    });

    let total_duration_ms = if step_records.is_empty() {
        0.0
    } else {
        let started_ns = step_records
            .iter()
            .filter_map(|r| r.get("started_ns").and_then(|v| v.as_i64()))
            .min()
            .unwrap_or(0);
        let ended_ns = step_records
            .iter()
            .filter_map(|r| r.get("ended_ns").and_then(|v| v.as_i64()))
            .max()
            .unwrap_or(0);
        let duration_ns = ended_ns.saturating_sub(started_ns);
        round_ms((duration_ns as f64) / 1_000_000.0)
    };

    Ok(ProfileSummary {
        run,
        steps: step_records,
        total_duration_ms,
        report_path: report_path.to_string_lossy().to_string(),
    })
}

fn step_key(record: &serde_json::Value) -> StepComparisonKey {
    let phase = record
        .get("metadata")
        .and_then(|m| m.get("phase"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let component = record
        .get("component")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let step = record
        .get("step")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    StepComparisonKey {
        phase,
        component,
        step,
    }
}

fn step_duration_map(summary: &ProfileSummary) -> BTreeMap<StepComparisonKey, f64> {
    let mut durations = BTreeMap::new();
    for record in &summary.steps {
        let duration = record
            .get("duration_ms")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let entry = durations.entry(step_key(record)).or_insert(0.0);
        *entry = round_ms(*entry + duration);
    }
    durations
}

fn compare_profile_summaries(
    baseline: ProfileSummary,
    candidate: ProfileSummary,
) -> ProfileComparison {
    let baseline_steps = step_duration_map(&baseline);
    let candidate_steps = step_duration_map(&candidate);
    let keys: BTreeSet<StepComparisonKey> = baseline_steps
        .keys()
        .chain(candidate_steps.keys())
        .cloned()
        .collect();

    let steps = keys
        .into_iter()
        .map(|key| {
            let baseline_ms = baseline_steps.get(&key).copied();
            let candidate_ms = candidate_steps.get(&key).copied();
            let delta_ms = baseline_ms
                .zip(candidate_ms)
                .map(|(baseline, candidate)| round_ms(candidate - baseline));
            let delta_percent = baseline_ms
                .zip(candidate_ms)
                .and_then(|(baseline, candidate)| percent_delta(baseline, candidate));
            StepComparison {
                key,
                baseline_ms,
                candidate_ms,
                delta_ms,
                delta_percent,
            }
        })
        .collect();

    let total_delta_ms = round_ms(candidate.total_duration_ms - baseline.total_duration_ms);
    let total_delta_percent =
        percent_delta(baseline.total_duration_ms, candidate.total_duration_ms);

    ProfileComparison {
        baseline,
        candidate,
        total_delta_ms,
        total_delta_percent,
        steps,
    }
}

fn format_optional_ms(value: Option<f64>) -> String {
    value
        .map(|ms| format!("{:.2}ms", ms))
        .unwrap_or_else(|| "-".to_string())
}

fn format_delta_ms(value: f64) -> String {
    if value >= 0.0 {
        format!("+{:.2}ms", value)
    } else {
        format!("{:.2}ms", value)
    }
}

fn format_optional_delta(value: Option<f64>) -> String {
    value
        .map(format_delta_ms)
        .unwrap_or_else(|| "new/removed".to_string())
}

fn format_optional_percent(value: Option<f64>) -> String {
    value
        .map(|percent| {
            if percent >= 0.0 {
                format!("+{:.1}%", percent)
            } else {
                format!("{:.1}%", percent)
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn scenario(summary: &ProfileSummary) -> &str {
    summary
        .run
        .get("scenario")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
}

fn render_profile_comparison(comparison: &ProfileComparison) -> String {
    let mut lines = Vec::new();
    lines.push("Startup Profile Comparison".to_string());
    lines.push(format!(
        "baseline: {} ({})",
        scenario(&comparison.baseline),
        comparison.baseline.report_path
    ));
    lines.push(format!(
        "candidate: {} ({})",
        scenario(&comparison.candidate),
        comparison.candidate.report_path
    ));
    lines.push(format!(
        "total: {:.2}ms -> {:.2}ms  {} ({})",
        comparison.baseline.total_duration_ms,
        comparison.candidate.total_duration_ms,
        format_delta_ms(comparison.total_delta_ms),
        format_optional_percent(comparison.total_delta_percent)
    ));
    lines.push(String::new());
    lines.push(format!(
        "{:>14}  {:>20}  {:>24}  {:>12}  {:>12}  {:>12}  {:>9}",
        "Phase", "Component", "Step", "Baseline", "Candidate", "Delta", "Delta %"
    ));

    for row in &comparison.steps {
        lines.push(format!(
            "{:>14}  {:>20}  {:>24}  {:>12}  {:>12}  {:>12}  {:>9}",
            if row.key.phase.is_empty() {
                "-"
            } else {
                row.key.phase.as_str()
            },
            row.key.component.as_str(),
            row.key.step.as_str(),
            format_optional_ms(row.baseline_ms),
            format_optional_ms(row.candidate_ms),
            format_optional_delta(row.delta_ms),
            format_optional_percent(row.delta_percent)
        ));
    }

    lines.join("\n")
}

fn render_summary_table(summary: &ProfileSummary) -> String {
    let has_context = summary.steps.iter().any(|record| {
        let phase = record
            .get("metadata")
            .and_then(|m| m.get("phase"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let pid = record
            .get("metadata")
            .and_then(|m| m.get("pid"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        !phase.is_empty() || !pid.is_empty()
    });

    if summary.steps.is_empty() {
        return "No startup profile steps recorded.".to_string();
    }

    let mut lines = Vec::new();

    for record in &summary.steps {
        let phase = record
            .get("metadata")
            .and_then(|m| m.get("phase"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let pid = record
            .get("metadata")
            .and_then(|m| m.get("pid"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let component = record
            .get("component")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let step = record.get("step").and_then(|v| v.as_str()).unwrap_or("");
        let duration_ms = record
            .get("duration_ms")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let context = if !phase.is_empty() && !pid.is_empty() {
            format!("{}#{}", phase, pid)
        } else if !phase.is_empty() {
            phase.to_string()
        } else {
            pid.to_string()
        };

        if has_context {
            lines.push(format!(
                "{:>12}  {:>20}  {:>20}  {:>10}",
                context,
                component,
                step,
                format!("{:.2}ms", duration_ms)
            ));
        } else {
            lines.push(format!(
                "{:>20}  {:>20}  {:>10}",
                component,
                step,
                format!("{:.2}ms", duration_ms)
            ));
        }
    }

    let header = if has_context {
        format!(
            "{:>12}  {:>20}  {:>20}  {:>10}",
            "Context", "Component", "Step", "Duration"
        )
    } else {
        format!("{:>20}  {:>20}  {:>10}", "Component", "Step", "Duration")
    };

    format!("{}\n{}", header, lines.join("\n"))
}

fn validate_baseline_name(name: &str) -> Result<(), CoreError> {
    if name.is_empty() {
        return Err(CoreError::usage(
            "Baseline name cannot be empty".to_string(),
        ));
    }
    if name == "." || name == ".." {
        return Err(CoreError::usage(format!("Invalid baseline name: {}", name)));
    }
    if !name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        return Err(CoreError::usage(format!(
            "Invalid baseline name: {}. Use letters, numbers, dots, dashes, or underscores.",
            name
        )));
    }
    Ok(())
}

fn baseline_path(name: &str) -> Result<PathBuf, CoreError> {
    validate_baseline_name(name)?;
    Ok(state_dir_from_env()?
        .join("profiles")
        .join("startup")
        .join("baselines")
        .join(format!("{}.jsonl", name)))
}

pub fn run_profile_create_run(args: &[String]) -> Result<i32, CoreError> {
    let mut scenario: Option<String> = None;
    let mut metadata_raw: Option<String> = None;
    let mut help = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--metadata" => {
                metadata_raw = iter.next().cloned();
            }
            "-h" | "--help" | "help" => help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile create-run: {other}"
                )));
            }
            other => {
                if scenario.is_some() {
                    return Err(CoreError::usage(
                        "profile create-run accepts only one scenario name".to_string(),
                    ));
                }
                scenario = Some(other.to_string());
            }
        }
    }

    if help {
        println!("Create a new startup profile run");
        println!();
        println!("Usage:");
        println!("  yzx_control profile create-run <scenario> [--metadata <json>]");
        return Ok(0);
    }

    let scenario = scenario.ok_or_else(|| {
        CoreError::usage("profile create-run requires a scenario name".to_string())
    })?;

    let run_id = generate_run_id();
    let state_dir = state_dir_from_env()?;
    let report_dir = state_dir.join("profiles").join("startup");
    std::fs::create_dir_all(&report_dir).map_err(|source| {
        CoreError::io(
            "profile_mkdir",
            &format!(
                "Could not create profile directory: {}",
                report_dir.display()
            ),
            "Check directory permissions.",
            report_dir.display().to_string(),
            source,
        )
    })?;

    let report_path = report_dir.join(format!("{}.jsonl", run_id));

    let _ = std::fs::remove_file(&report_path);

    let metadata = parse_metadata(metadata_raw.as_deref())?;

    let run_record = RunRecord {
        record_type: "run".to_string(),
        schema_version: SCHEMA_VERSION,
        run_id: run_id.clone(),
        scenario: scenario.clone(),
        created_at: now_rfc3339(),
        metadata: metadata.clone(),
    };

    append_jsonl(
        &report_path,
        &serde_json::to_value(&run_record).map_err(|e| {
            CoreError::classified(
                ErrorClass::Internal,
                "profile_serialize",
                &format!("Failed to serialize run record: {}", e),
                "This is a bug; please report it.",
                serde_json::Value::Null,
            )
        })?,
    )?;

    let mut env = serde_json::Map::new();
    env.insert(
        "YAZELIX_STARTUP_PROFILE".to_string(),
        serde_json::Value::String("true".to_string()),
    );
    env.insert(
        "YAZELIX_STARTUP_PROFILE_RUN_ID".to_string(),
        serde_json::Value::String(run_id.clone()),
    );
    env.insert(
        "YAZELIX_STARTUP_PROFILE_REPORT".to_string(),
        serde_json::Value::String(report_path.to_string_lossy().to_string()),
    );
    env.insert(
        "YAZELIX_STARTUP_PROFILE_SCENARIO".to_string(),
        serde_json::Value::String(scenario),
    );

    let info = ProfileRunInfo {
        run_id,
        report_path,
        env,
    };

    println!("{}", serde_json::to_string(&info).unwrap());
    Ok(0)
}

pub fn run_profile_record_step(args: &[String]) -> Result<i32, CoreError> {
    let mut component: Option<String> = None;
    let mut step: Option<String> = None;
    let mut started_ns: Option<i64> = None;
    let mut ended_ns: Option<i64> = None;
    let mut metadata_raw: Option<String> = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--metadata" => {
                metadata_raw = iter.next().cloned();
            }
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile record-step: {other}"
                )));
            }
            other => {
                if component.is_none() {
                    component = Some(other.to_string());
                } else if step.is_none() {
                    step = Some(other.to_string());
                } else if started_ns.is_none() {
                    started_ns =
                        Some(other.parse().map_err(|_| {
                            CoreError::usage(format!("Invalid started_ns: {}", other))
                        })?);
                } else if ended_ns.is_none() {
                    ended_ns =
                        Some(other.parse().map_err(|_| {
                            CoreError::usage(format!("Invalid ended_ns: {}", other))
                        })?);
                } else {
                    return Err(CoreError::usage(
                        "profile record-step accepts too many positional arguments".to_string(),
                    ));
                }
            }
        }
    }

    let component = component.ok_or_else(|| {
        CoreError::usage(
            "profile record-step requires component, step, started_ns, ended_ns".to_string(),
        )
    })?;
    let step = step.ok_or_else(|| {
        CoreError::usage("profile record-step requires step, started_ns, ended_ns".to_string())
    })?;
    let started_ns = started_ns.ok_or_else(|| {
        CoreError::usage("profile record-step requires started_ns and ended_ns".to_string())
    })?;
    let ended_ns = ended_ns
        .ok_or_else(|| CoreError::usage("profile record-step requires ended_ns".to_string()))?;

    let report_path = std::env::var("YAZELIX_STARTUP_PROFILE_REPORT")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| CoreError::usage("YAZELIX_STARTUP_PROFILE_REPORT not set".to_string()))?;

    let run_id = std::env::var("YAZELIX_STARTUP_PROFILE_RUN_ID").unwrap_or_default();
    let scenario = std::env::var("YAZELIX_STARTUP_PROFILE_SCENARIO").unwrap_or_default();

    let duration_ns = ended_ns.saturating_sub(started_ns);
    let duration_ms = ((duration_ns as f64) / 1_000_000.0 * 100.0).round() / 100.0;

    let metadata = parse_metadata(metadata_raw.as_deref())?;

    let step_record = StepRecord {
        record_type: "step".to_string(),
        schema_version: SCHEMA_VERSION,
        run_id,
        scenario,
        component,
        step,
        started_ns,
        ended_ns,
        duration_ms,
        recorded_at: now_rfc3339(),
        metadata,
    };

    append_jsonl(
        &report_path,
        &serde_json::to_value(&step_record).map_err(|e| {
            CoreError::classified(
                ErrorClass::Internal,
                "profile_serialize",
                &format!("Failed to serialize step record: {}", e),
                "This is a bug; please report it.",
                serde_json::Value::Null,
            )
        })?,
    )?;
    Ok(0)
}

pub fn run_profile_load_report(args: &[String]) -> Result<i32, CoreError> {
    let mut report_path: Option<String> = None;
    let mut help = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile load-report: {other}"
                )));
            }
            other => {
                if report_path.is_some() {
                    return Err(CoreError::usage(
                        "profile load-report accepts only one report path".to_string(),
                    ));
                }
                report_path = Some(other.to_string());
            }
        }
    }

    if help {
        println!("Load a startup profile report as JSON");
        println!();
        println!("Usage:");
        println!("  yzx_control profile load-report <report_path>");
        return Ok(0);
    }

    let report_path = report_path.ok_or_else(|| {
        CoreError::usage("profile load-report requires a report path".to_string())
    })?;

    let summary = load_report_data(Path::new(&report_path))?;
    println!("{}", serde_json::to_string(&summary).unwrap());
    Ok(0)
}

pub fn run_profile_compare_reports(args: &[String]) -> Result<i32, CoreError> {
    let mut baseline_report: Option<String> = None;
    let mut candidate_report: Option<String> = None;
    let mut help = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile compare-reports: {other}"
                )));
            }
            other => {
                if baseline_report.is_none() {
                    baseline_report = Some(other.to_string());
                } else if candidate_report.is_none() {
                    candidate_report = Some(other.to_string());
                } else {
                    return Err(CoreError::usage(
                        "profile compare-reports accepts baseline_report and candidate_report"
                            .to_string(),
                    ));
                }
            }
        }
    }

    if help {
        println!("Compare two saved startup profile reports");
        println!();
        println!("Usage:");
        println!("  yzx_control profile compare-reports <baseline_report> <candidate_report>");
        return Ok(0);
    }

    let baseline_report = baseline_report.ok_or_else(|| {
        CoreError::usage(
            "profile compare-reports requires baseline_report and candidate_report".to_string(),
        )
    })?;
    let candidate_report = candidate_report.ok_or_else(|| {
        CoreError::usage("profile compare-reports requires candidate_report".to_string())
    })?;

    let baseline = load_report_data(Path::new(&baseline_report))?;
    let candidate = load_report_data(Path::new(&candidate_report))?;
    let comparison = compare_profile_summaries(baseline, candidate);
    println!("{}", render_profile_comparison(&comparison));
    Ok(0)
}

pub fn run_profile_save_baseline(args: &[String]) -> Result<i32, CoreError> {
    let mut name: Option<String> = None;
    let mut report_path: Option<String> = None;
    let mut help = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile save-baseline: {other}"
                )));
            }
            other => {
                if name.is_none() {
                    name = Some(other.to_string());
                } else if report_path.is_none() {
                    report_path = Some(other.to_string());
                } else {
                    return Err(CoreError::usage(
                        "profile save-baseline accepts name and report_path".to_string(),
                    ));
                }
            }
        }
    }

    if help {
        println!("Save a startup profile report as a named local baseline");
        println!();
        println!("Usage:");
        println!("  yzx_control profile save-baseline <name> <report_path>");
        return Ok(0);
    }

    let name = name.ok_or_else(|| {
        CoreError::usage("profile save-baseline requires name and report_path".to_string())
    })?;
    let report_path = report_path.ok_or_else(|| {
        CoreError::usage("profile save-baseline requires report_path".to_string())
    })?;
    let source = PathBuf::from(&report_path);
    if !source.exists() {
        return Err(CoreError::io(
            "profile_baseline_source",
            &format!("Startup profile report not found: {}", source.display()),
            "Check the report path before saving it as a baseline.",
            source.display().to_string(),
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing report"),
        ));
    }

    let destination = baseline_path(&name)?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "profile_baseline_mkdir",
                &format!(
                    "Could not create profile baseline directory: {}",
                    parent.display()
                ),
                "Check directory permissions.",
                parent.display().to_string(),
                source,
            )
        })?;
    }

    std::fs::copy(&source, &destination).map_err(|source_error| {
        CoreError::io(
            "profile_baseline_save",
            &format!(
                "Could not save profile baseline {} to {}.",
                name,
                destination.display()
            ),
            "Check directory permissions and disk space.",
            destination.display().to_string(),
            source_error,
        )
    })?;

    println!("Saved startup profile baseline `{}`", name);
    println!("{}", destination.display());
    Ok(0)
}

pub fn run_profile_compare_baseline(args: &[String]) -> Result<i32, CoreError> {
    let mut name: Option<String> = None;
    let mut candidate_report: Option<String> = None;
    let mut help = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile compare-baseline: {other}"
                )));
            }
            other => {
                if name.is_none() {
                    name = Some(other.to_string());
                } else if candidate_report.is_none() {
                    candidate_report = Some(other.to_string());
                } else {
                    return Err(CoreError::usage(
                        "profile compare-baseline accepts name and candidate_report".to_string(),
                    ));
                }
            }
        }
    }

    if help {
        println!("Compare a saved startup profile baseline with a report");
        println!();
        println!("Usage:");
        println!("  yzx_control profile compare-baseline <name> <candidate_report>");
        return Ok(0);
    }

    let name = name.ok_or_else(|| {
        CoreError::usage("profile compare-baseline requires name and candidate_report".to_string())
    })?;
    let candidate_report = candidate_report.ok_or_else(|| {
        CoreError::usage("profile compare-baseline requires candidate_report".to_string())
    })?;

    let baseline = load_report_data(&baseline_path(&name)?)?;
    let candidate = load_report_data(Path::new(&candidate_report))?;
    let comparison = compare_profile_summaries(baseline, candidate);
    println!("{}", render_profile_comparison(&comparison));
    Ok(0)
}

pub fn run_profile_wait_step(args: &[String]) -> Result<i32, CoreError> {
    let mut report_path: Option<String> = None;
    let mut component: Option<String> = None;
    let mut step: Option<String> = None;
    let mut timeout_ms: u64 = 15000;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--timeout-ms" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| CoreError::usage("--timeout-ms requires a value".to_string()))?;
                timeout_ms = raw
                    .parse()
                    .map_err(|_| CoreError::usage(format!("Invalid timeout value: {}", raw)))?;
            }
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile wait-step: {other}"
                )));
            }
            other => {
                if report_path.is_none() {
                    report_path = Some(other.to_string());
                } else if component.is_none() {
                    component = Some(other.to_string());
                } else if step.is_none() {
                    step = Some(other.to_string());
                } else {
                    return Err(CoreError::usage(
                        "profile wait-step accepts too many positional arguments".to_string(),
                    ));
                }
            }
        }
    }

    let report_path = report_path.ok_or_else(|| {
        CoreError::usage("profile wait-step requires report_path, component, step".to_string())
    })?;
    let component = component.ok_or_else(|| {
        CoreError::usage("profile wait-step requires component and step".to_string())
    })?;
    let step =
        step.ok_or_else(|| CoreError::usage("profile wait-step requires step".to_string()))?;

    let report_path = PathBuf::from(&report_path);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

    loop {
        if report_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&report_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(record) = serde_json::from_str::<serde_json::Value>(line) {
                        if record.get("type").and_then(|v| v.as_str()) == Some("step")
                            && record.get("component").and_then(|v| v.as_str()) == Some(&component)
                            && record.get("step").and_then(|v| v.as_str()) == Some(&step)
                        {
                            println!("true");
                            return Ok(0);
                        }
                    }
                }
            }
        }

        if std::time::Instant::now() >= deadline {
            println!("false");
            return Ok(0);
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

pub fn run_profile_print_report(args: &[String]) -> Result<i32, CoreError> {
    let mut report_path: Option<String> = None;
    let mut help = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" | "help" => help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for profile print-report: {other}"
                )));
            }
            other => {
                if report_path.is_some() {
                    return Err(CoreError::usage(
                        "profile print-report accepts only one report path".to_string(),
                    ));
                }
                report_path = Some(other.to_string());
            }
        }
    }

    if help {
        println!("Print a startup profile report as a formatted table");
        println!();
        println!("Usage:");
        println!("  yzx_control profile print-report <report_path>");
        return Ok(0);
    }

    let report_path = report_path.ok_or_else(|| {
        CoreError::usage("profile print-report requires a report path".to_string())
    })?;

    let summary = load_report_data(Path::new(&report_path))?;

    println!();
    println!("📊 Startup Profile Report");
    println!(
        "   scenario: {}",
        summary
            .run
            .get("scenario")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
    );
    println!("   report: {}", summary.report_path);
    println!("   total: {:.2}ms", summary.total_duration_ms);
    println!();
    println!("{}", render_summary_table(&summary));

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // Defends: profile report data loads run and step records correctly.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn load_report_data_parses_run_and_steps() {
        let temp_dir = tempfile::tempdir().unwrap();
        let report_path = temp_dir.path().join("test.jsonl");

        let run = serde_json::json!({
            "type": "run",
            "schema_version": 1,
            "run_id": "test_run",
            "scenario": "test_scenario",
            "created_at": "2024-01-01T00:00:00Z",
            "metadata": {}
        });

        let step1 = serde_json::json!({
            "type": "step",
            "schema_version": 1,
            "run_id": "test_run",
            "scenario": "test_scenario",
            "component": "inner",
            "step": "init",
            "started_ns": 1000000,
            "ended_ns": 2000000,
            "duration_ms": 1.0,
            "recorded_at": "2024-01-01T00:00:01Z",
            "metadata": {}
        });

        let step2 = serde_json::json!({
            "type": "step",
            "schema_version": 1,
            "run_id": "test_run",
            "scenario": "test_scenario",
            "component": "inner",
            "step": "finish",
            "started_ns": 3000000,
            "ended_ns": 5000000,
            "duration_ms": 2.0,
            "recorded_at": "2024-01-01T00:00:02Z",
            "metadata": {}
        });

        let mut file = std::fs::File::create(&report_path).unwrap();
        writeln!(file, "{}", serde_json::to_string(&run).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&step1).unwrap()).unwrap();
        writeln!(file, "{}", serde_json::to_string(&step2).unwrap()).unwrap();

        let summary = load_report_data(&report_path).unwrap();
        assert_eq!(
            summary.run.get("run_id").and_then(|v| v.as_str()),
            Some("test_run")
        );
        assert_eq!(summary.steps.len(), 2);
        assert_eq!(summary.total_duration_ms, 4.0);
    }

    // Defends: startup profile comparison matches saved reports by phase/component/step and surfaces total plus per-step deltas.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn compare_profile_summaries_reports_total_and_step_deltas() {
        let baseline = ProfileSummary {
            run: serde_json::json!({"scenario": "enter_warm"}),
            steps: vec![
                serde_json::json!({
                    "component": "inner",
                    "step": "config",
                    "duration_ms": 10.0,
                    "metadata": {"phase": "startup"}
                }),
                serde_json::json!({
                    "component": "inner",
                    "step": "materialize",
                    "duration_ms": 5.0,
                    "metadata": {"phase": "startup"}
                }),
            ],
            total_duration_ms: 20.0,
            report_path: "/tmp/baseline.jsonl".to_string(),
        };
        let candidate = ProfileSummary {
            run: serde_json::json!({"scenario": "enter_warm"}),
            steps: vec![
                serde_json::json!({
                    "component": "inner",
                    "step": "config",
                    "duration_ms": 14.5,
                    "metadata": {"phase": "startup"}
                }),
                serde_json::json!({
                    "component": "inner",
                    "step": "zellij_handoff_ready",
                    "duration_ms": 2.0,
                    "metadata": {"phase": "startup"}
                }),
            ],
            total_duration_ms: 30.0,
            report_path: "/tmp/candidate.jsonl".to_string(),
        };

        let comparison = compare_profile_summaries(baseline, candidate);
        assert_eq!(comparison.total_delta_ms, 10.0);
        assert_eq!(comparison.total_delta_percent, Some(50.0));

        let config = comparison
            .steps
            .iter()
            .find(|row| row.key.step == "config")
            .unwrap();
        assert_eq!(config.baseline_ms, Some(10.0));
        assert_eq!(config.candidate_ms, Some(14.5));
        assert_eq!(config.delta_ms, Some(4.5));
        assert_eq!(config.delta_percent, Some(45.0));

        let removed = comparison
            .steps
            .iter()
            .find(|row| row.key.step == "materialize")
            .unwrap();
        assert_eq!(removed.baseline_ms, Some(5.0));
        assert_eq!(removed.candidate_ms, None);

        let rendered = render_profile_comparison(&comparison);
        assert!(rendered.contains("Startup Profile Comparison"));
        assert!(rendered.contains("total: 20.00ms -> 30.00ms  +10.00ms (+50.0%)"));
        assert!(rendered.contains("new/removed"));
        assert!(rendered.contains("zellij_handoff_ready"));
    }

    // Defends: profile run ID uses the expected prefix format.
    // Strength: defect=2 behavior=1 resilience=1 cost=2 uniqueness=2 total=8/10
    #[test]
    fn generate_run_id_has_expected_prefix() {
        let run_id = generate_run_id();
        assert!(
            run_id.starts_with("startup_profile_"),
            "run_id should start with 'startup_profile_': got {}",
            run_id
        );
    }

    // Defends: summary table renders steps with and without context.
    // Strength: defect=1 behavior=2 resilience=1 cost=2 uniqueness=2 total=8/10
    #[test]
    fn render_summary_table_shows_steps() {
        let summary = ProfileSummary {
            run: serde_json::json!({"scenario": "test"}),
            steps: vec![serde_json::json!({
                "component": "inner",
                "step": "init",
                "duration_ms": 1.5,
                "metadata": {}
            })],
            total_duration_ms: 1.5,
            report_path: "/tmp/test.jsonl".to_string(),
        };

        let table = render_summary_table(&summary);
        assert!(table.contains("inner"));
        assert!(table.contains("init"));
        assert!(table.contains("1.50ms"));
    }
}
