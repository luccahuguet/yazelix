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

fn now_rfc3339() -> String {
    let now = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
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
        "[year][month][day]_[hour][minute][second]_[subsecond digits:3]"
    )
    .unwrap_or_else(|_| time::format_description::parse("[year][month][day]_[hour][minute][second]").unwrap());
    let timestamp = now.format(&format).unwrap_or_default();
    format!("startup_profile_{}", timestamp)
}

fn append_jsonl(path: &Path, value: &serde_json::Value) -> Result<(), CoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| {
            CoreError::io(
                "profile_append",
                &format!("Could not open profile report for append: {}", path.display()),
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
        Some(s) if !s.trim().is_empty() => serde_json::from_str(s).map_err(|e| {
            CoreError::usage(format!("Invalid metadata JSON: {}", e))
        }),
        _ => Ok(serde_json::Value::Null),
    }
}

fn load_report_data(report_path: &Path) -> Result<ProfileSummary, CoreError> {
    if !report_path.exists() {
        return Err(CoreError::io(
            "profile_load",
            &format!("Startup profile report not found: {}", report_path.display()),
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
        ((duration_ns as f64) / 1_000_000.0 * 100.0).round() / 100.0
    };

    Ok(ProfileSummary {
        run,
        steps: step_records,
        total_duration_ms,
        report_path: report_path.to_string_lossy().to_string(),
    })
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
                context, component, step, format!("{:.2}ms", duration_ms)
            ));
        } else {
            lines.push(format!(
                "{:>20}  {:>20}  {:>10}",
                component, step, format!("{:.2}ms", duration_ms)
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
                    started_ns = Some(other.parse().map_err(|_| {
                        CoreError::usage(format!("Invalid started_ns: {}", other))
                    })?);
                } else if ended_ns.is_none() {
                    ended_ns = Some(other.parse().map_err(|_| {
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
        CoreError::usage(
            "profile record-step requires step, started_ns, ended_ns".to_string(),
        )
    })?;
    let started_ns = started_ns.ok_or_else(|| {
        CoreError::usage(
            "profile record-step requires started_ns and ended_ns".to_string(),
        )
    })?;
    let ended_ns = ended_ns.ok_or_else(|| {
        CoreError::usage("profile record-step requires ended_ns".to_string())
    })?;

    let report_path = std::env::var("YAZELIX_STARTUP_PROFILE_REPORT")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| {
            CoreError::usage("YAZELIX_STARTUP_PROFILE_REPORT not set".to_string())
        })?;

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

pub fn run_profile_wait_step(args: &[String]) -> Result<i32, CoreError> {
    let mut report_path: Option<String> = None;
    let mut component: Option<String> = None;
    let mut step: Option<String> = None;
    let mut timeout_ms: u64 = 15000;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--timeout-ms" => {
                let raw = iter.next().ok_or_else(|| {
                    CoreError::usage("--timeout-ms requires a value".to_string())
                })?;
                timeout_ms = raw.parse().map_err(|_| {
                    CoreError::usage(format!("Invalid timeout value: {}", raw))
                })?;
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
        CoreError::usage(
            "profile wait-step requires report_path, component, step".to_string(),
        )
    })?;
    let component = component.ok_or_else(|| {
        CoreError::usage("profile wait-step requires component and step".to_string())
    })?;
    let step = step.ok_or_else(|| {
        CoreError::usage("profile wait-step requires step".to_string())
    })?;

    let report_path = PathBuf::from(&report_path);
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);

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
                            && record.get("component").and_then(|v| v.as_str())
                                == Some(&component)
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

    // Defends: profile run ID uses the expected prefix format.
    // Strength: defect=2 behavior=1 resilience=1 cost=1 uniqueness=2 total=7/10
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
    // Strength: defect=1 behavior=2 resilience=1 cost=1 uniqueness=2 total=7/10
    #[test]
    fn render_summary_table_shows_steps() {
        let summary = ProfileSummary {
            run: serde_json::json!({"scenario": "test"}),
            steps: vec![
                serde_json::json!({
                    "component": "inner",
                    "step": "init",
                    "duration_ms": 1.5,
                    "metadata": {}
                }),
            ],
            total_duration_ms: 1.5,
            report_path: "/tmp/test.jsonl".to_string(),
        };

        let table = render_summary_table(&summary);
        assert!(table.contains("inner"));
        assert!(table.contains("init"));
        assert!(table.contains("1.50ms"));
    }
}
