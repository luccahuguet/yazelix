//! Public `yzx doctor` owner for report collection, JSON output, and human rendering.

use crate::active_config_surface::{primary_config_paths, resolve_active_config_paths};
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, home_dir_from_env, runtime_dir_from_env,
    runtime_materialization_plan_request_from_env, state_dir_from_env,
};
use crate::doctor_helix_report::{HelixDoctorEvaluateRequest, evaluate_helix_doctor_report};
use crate::doctor_runtime_report::{
    DoctorRuntimeEvaluateRequest, SharedRuntimePreflightInput, evaluate_doctor_runtime_report,
};
use crate::install_ownership_env::install_ownership_request_from_env_with_runtime_dir;
use crate::runtime_materialization::{
    RuntimeMaterializationRepairEvaluateRequest, repair_runtime_materialization,
};
use crate::settings_surface::render_default_settings_jsonc;
use crate::user_config_paths;
use crate::workspace_asset_contract::{
    WorkspaceAssetEvaluateRequest, evaluate_workspace_asset_report,
};
use crate::zellij_materialization::{
    ZellijMaterializationRequest, generate_zellij_materialization, zellij_permissions_cache_path,
};
use crate::{
    DoctorConfigEvaluateRequest, NormalizeConfigRequest, evaluate_doctor_config_report,
    evaluate_install_ownership_report, normalize_config, plan_runtime_materialization,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DoctorCliArgs {
    verbose: bool,
    fix: bool,
    fix_plan: bool,
    json: bool,
    help: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReportSummary {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub ok_count: usize,
    pub fixable_count: usize,
    pub healthy: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReportData {
    pub title: String,
    pub results: Vec<Value>,
    pub summary: DoctorReportSummary,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RecoveryPlanSummary {
    action_count: usize,
    automatic_action_count: usize,
    manual_action_count: usize,
    highest_severity: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RecoveryPlanAction {
    id: String,
    severity: String,
    problem: String,
    evidence: Vec<String>,
    commands: Vec<String>,
    safe_to_run_automatically: bool,
    rationale: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RecoveryPlanReport {
    schema_version: u8,
    title: String,
    inspect_command: String,
    summary: RecoveryPlanSummary,
    actions: Vec<RecoveryPlanAction>,
}

#[derive(Debug, Deserialize)]
struct SessionManagedPanes {
    #[serde(default)]
    editor_pane_id: Option<String>,
    #[serde(default)]
    sidebar_pane_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SessionLayout {
    #[serde(default)]
    active_swap_layout_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ActiveTabSessionStateV1 {
    active_tab_position: usize,
    managed_panes: SessionManagedPanes,
    layout: SessionLayout,
}

#[derive(Debug, Clone)]
struct ZellijPluginState {
    permissions_granted: bool,
    active_tab_position: Option<usize>,
    sidebar_pane_id: String,
    editor_pane_id: String,
    active_swap_layout_name: Option<String>,
}

pub fn run_yzx_doctor(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_doctor_cli_args(args)?;
    if parsed.help {
        print_doctor_help();
        return Ok(0);
    }

    if parsed.fix && parsed.fix_plan {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "doctor_fix_plan_cannot_fix",
            "`yzx doctor --fix-plan` is a dry recovery plan and cannot be combined with `--fix`.",
            "Run `yzx doctor --fix-plan` to inspect recovery steps or `yzx doctor --fix` to run safe automatic repairs.",
            json!({}),
        ));
    }

    if parsed.json && parsed.fix {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "doctor_json_fix_unsupported",
            "`yzx doctor --json` does not support `--fix` yet. Run `yzx doctor --json` for machine-readable diagnostics or `yzx doctor --fix` for the current interactive repair flow.",
            "Run `yzx doctor --json` for machine-readable diagnostics or `yzx doctor --fix` for the current interactive repair flow.",
            json!({}),
        ));
    }

    let report = compute_doctor_report_from_env()?;
    if parsed.fix_plan {
        let recovery = build_recovery_plan(&report);
        if parsed.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&recovery).unwrap_or_else(|_| "{}".to_string())
            );
        } else {
            render_recovery_plan(&recovery);
        }
        return Ok(0);
    }

    if parsed.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
        );
        return Ok(0);
    }

    render_doctor_report(&report, parsed.verbose);
    if report.summary.healthy {
        return Ok(0);
    }

    print_runtime_conflict_fix_commands(&report.results);

    if parsed.fix {
        return run_doctor_fix_flow(parsed.verbose, &report.results);
    }

    if report.summary.fixable_count > 0 {
        println!("\n💡 Some issues can be auto-fixed. Run 'yzx doctor --fix' to resolve them.");
    }

    Ok(0)
}

fn print_doctor_help() {
    println!("Run health checks and diagnostics");
    println!();
    println!("Usage:");
    println!("  yzx doctor [--verbose] [--json]");
    println!("  yzx doctor --fix-plan [--json]");
    println!("  yzx doctor --fix [--verbose]");
    println!();
    println!("Flags:");
    println!("  -v, --verbose  Show detailed information");
    println!("  -f, --fix      Attempt to auto-fix issues");
    println!("      --fix-plan Print exact recovery commands without mutating anything");
    println!("      --json     Emit machine-readable doctor data");
}

fn parse_doctor_cli_args(args: &[String]) -> Result<DoctorCliArgs, CoreError> {
    let mut out = DoctorCliArgs::default();
    for token in args {
        match token.as_str() {
            "--verbose" | "-v" => out.verbose = true,
            "--fix" | "-f" => out.fix = true,
            "--fix-plan" => out.fix_plan = true,
            "--json" => out.json = true,
            "--help" | "-h" | "help" => out.help = true,
            other => {
                return Err(CoreError::classified(
                    ErrorClass::Usage,
                    "unexpected_doctor_token",
                    format!("Unexpected argument for yzx doctor: {other}"),
                    "Run `yzx doctor`, `yzx doctor --json`, `yzx doctor --fix-plan`, or `yzx doctor --fix`.",
                    json!({}),
                ));
            }
        }
    }
    Ok(out)
}

fn compute_doctor_report_from_env() -> Result<DoctorReportData, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let home_dir = home_dir_from_env()?;
    let install_request = install_ownership_request_from_env_with_runtime_dir(runtime_dir.clone())?;
    let install_report = evaluate_install_ownership_report(&install_request);
    let normalized_config = load_optional_doctor_normalized_config(&runtime_dir, &config_dir);

    let runtime_findings = collect_runtime_doctor_findings(
        &runtime_dir,
        &state_dir,
        &install_report,
        normalized_config.as_ref(),
    );
    let helix_findings = collect_helix_doctor_findings(
        &runtime_dir,
        &config_dir,
        &state_dir,
        &home_dir,
        normalized_config.as_ref(),
    );
    let config_findings = collect_config_doctor_findings(&runtime_dir, &config_dir);
    let workspace_asset_findings =
        collect_workspace_asset_doctor_findings(&runtime_dir, &state_dir);
    let zellij_findings = collect_zellij_plugin_health_findings(normalized_config.as_ref());

    let mut results = Vec::new();
    results.extend(runtime_findings);
    results.extend(helix_findings);
    results.extend(config_findings);
    results.extend(workspace_asset_findings);
    results.extend(
        install_report
            .wrapper_shadowing
            .iter()
            .map(serialize_value)
            .collect::<Result<Vec<_>, _>>()?,
    );
    if let Some(collision) = install_report.home_manager_profile_collision.as_ref() {
        results.push(serialize_value(collision)?);
    }
    results.push(serialize_value(&install_report.desktop_entry_freshness)?);
    results.extend(zellij_findings);

    let summary = summarize_doctor_results(&results);
    Ok(DoctorReportData {
        title: "Yazelix Health Checks".to_string(),
        results,
        summary,
    })
}

fn xdg_config_home(home_dir: &Path) -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".config"))
}

fn find_external_command(command_name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for entry in env::split_paths(&path_var) {
        let candidate = entry.join(command_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn load_optional_doctor_normalized_config(
    runtime_dir: &Path,
    config_dir: &Path,
) -> Option<serde_json::Map<String, Value>> {
    let config_override = config_override_from_env();
    let paths =
        resolve_active_config_paths(runtime_dir, config_dir, config_override.as_deref()).ok()?;
    let data = normalize_config(&NormalizeConfigRequest {
        config_path: paths.config_file,
        default_config_path: paths.default_config_path,
        contract_path: paths.contract_path,
        include_missing: false,
    })
    .ok()?;
    Some(data.normalized_config)
}

fn collect_runtime_doctor_findings(
    runtime_dir: &Path,
    state_dir: &Path,
    install_report: &crate::InstallOwnershipEvaluateData,
    normalized_config: Option<&serde_json::Map<String, Value>>,
) -> Vec<Value> {
    let mut extra = Vec::new();
    let shared_runtime = if normalized_config.is_some() {
        match runtime_materialization_plan_request_from_env(config_override_from_env().as_deref()) {
            Ok(request) => match plan_runtime_materialization(&request) {
                Ok(plan) => {
                    let terminals = plan
                        .config_state
                        .config
                        .get("terminals")
                        .and_then(Value::as_array)
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(Value::as_str)
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                        })
                        .filter(|items| !items.is_empty())
                        .unwrap_or_else(|| vec!["ghostty".to_string()]);
                    Some(SharedRuntimePreflightInput {
                        zellij_layout_path: PathBuf::from(plan.zellij_layout_path),
                        terminals,
                        startup_script_path: runtime_dir
                            .join("nushell")
                            .join("scripts")
                            .join("core")
                            .join("start_yazelix_inner.nu"),
                        launch_script_path: runtime_dir
                            .join("shells")
                            .join("posix")
                            .join("start_yazelix.sh"),
                        command_search_paths: env::var_os("PATH")
                            .map(|raw| env::split_paths(&raw).collect())
                            .unwrap_or_default(),
                        platform_name: platform_name_for_runtime_doctor(),
                    })
                }
                Err(error) => {
                    extra.push(json!({
                        "status": "error",
                        "message": "Could not resolve the managed Zellij layout path from the Rust materialization plan",
                        "details": error.message(),
                        "fix_available": false
                    }));
                    None
                }
            },
            Err(error) => {
                extra.push(json!({
                    "status": "error",
                    "message": "Could not resolve the managed Zellij layout path from the Rust materialization plan",
                    "details": error.message(),
                    "fix_available": false
                }));
                None
            }
        }
    } else {
        None
    };

    let data = evaluate_doctor_runtime_report(&DoctorRuntimeEvaluateRequest {
        runtime_dir: runtime_dir.to_path_buf(),
        yazelix_state_dir: state_dir.to_path_buf(),
        has_home_manager_managed_install: install_report.has_home_manager_managed_install,
        is_manual_runtime_reference_path: install_report.is_manual_runtime_reference_path,
        shared_runtime,
    });

    let mut results = Vec::new();
    results.push(
        serialize_value(&data.distribution).expect("runtime distribution finding should serialize"),
    );
    results.extend(extra);
    results.extend(
        data.shared_runtime_preflight
            .iter()
            .map(serialize_value)
            .collect::<Result<Vec<_>, _>>()
            .expect("runtime preflight findings should serialize"),
    );
    results
}

fn collect_helix_doctor_findings(
    runtime_dir: &Path,
    config_dir: &Path,
    state_dir: &Path,
    home_dir: &Path,
    normalized_config: Option<&serde_json::Map<String, Value>>,
) -> Vec<Value> {
    let editor_command = normalized_config.map(|cfg| {
        cfg.get("editor_command")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    });
    let request = HelixDoctorEvaluateRequest {
        home_dir: home_dir.to_path_buf(),
        runtime_dir: runtime_dir.to_path_buf(),
        config_dir: config_dir.to_path_buf(),
        user_config_helix_runtime_dir: home_dir.join(".config").join("helix").join("runtime"),
        hx_exe_path: find_external_command("hx"),
        include_runtime_health: env::var("EDITOR")
            .ok()
            .map(|value| value.contains("hx"))
            .unwrap_or(false),
        editor_command,
        managed_helix_user_config_path: user_config_paths::helix_config(config_dir),
        native_helix_config_path: xdg_config_home(home_dir).join("helix").join("config.toml"),
        generated_helix_config_path: state_dir.join("configs").join("helix").join("config.toml"),
        expected_managed_config: None,
        build_managed_config_error: None,
        reveal_binding_expected: crate::helix_materialization::MANAGED_REVEAL_COMMAND.into(),
    };
    let data = evaluate_helix_doctor_report(&request);
    let mut results = Vec::new();
    results.push(
        serialize_value(&data.runtime_conflicts).expect("helix runtime finding should serialize"),
    );
    if let Some(runtime_health) = &data.runtime_health {
        results
            .push(serialize_value(runtime_health).expect("helix runtime health should serialize"));
    }
    results.extend(
        data.managed_integration
            .iter()
            .map(serialize_value)
            .collect::<Result<Vec<_>, _>>()
            .expect("helix managed findings should serialize"),
    );
    results
}

fn collect_config_doctor_findings(runtime_dir: &Path, config_dir: &Path) -> Vec<Value> {
    let data = evaluate_doctor_config_report(&DoctorConfigEvaluateRequest {
        config_dir: config_dir.to_path_buf(),
        runtime_dir: runtime_dir.to_path_buf(),
    });
    data.findings
        .iter()
        .map(serialize_value)
        .collect::<Result<Vec<_>, _>>()
        .expect("config findings should serialize")
}

fn collect_workspace_asset_doctor_findings(runtime_dir: &Path, state_dir: &Path) -> Vec<Value> {
    evaluate_workspace_asset_report(&WorkspaceAssetEvaluateRequest {
        runtime_dir: runtime_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
    })
    .iter()
    .map(serialize_value)
    .collect::<Result<Vec<_>, _>>()
    .expect("workspace asset findings should serialize")
}

fn platform_name_for_runtime_doctor() -> String {
    env::var("YAZELIX_TEST_OS")
        .ok()
        .map(|raw| raw.trim().to_lowercase())
        .filter(|raw| !raw.is_empty())
        .unwrap_or_else(|| env::consts::OS.to_lowercase())
}

fn serialize_value<T: Serialize>(value: &T) -> Result<Value, CoreError> {
    serde_json::to_value(value).map_err(|error| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_doctor_result",
            format!("Failed to serialize doctor result: {error}"),
            "Rebuild or reinstall Yazelix so the Rust doctor report surface can render its structured findings.",
            json!({}),
        )
    })
}

fn summarize_doctor_results(results: &[Value]) -> DoctorReportSummary {
    let error_count = results
        .iter()
        .filter(|result| result_status(result) == "error")
        .count();
    let warning_count = results
        .iter()
        .filter(|result| result_status(result) == "warning")
        .count();
    let info_count = results
        .iter()
        .filter(|result| result_status(result) == "info")
        .count();
    let ok_count = results
        .iter()
        .filter(|result| result_status(result) == "ok")
        .count();
    let fixable_count = results
        .iter()
        .filter(|result| result_fix_available(result))
        .count();

    DoctorReportSummary {
        error_count,
        warning_count,
        info_count,
        ok_count,
        fixable_count,
        healthy: error_count == 0 && warning_count == 0 && fixable_count == 0,
    }
}

fn result_status(result: &Value) -> &str {
    result.get("status").and_then(Value::as_str).unwrap_or("")
}

fn result_fix_available(result: &Value) -> bool {
    result
        .get("fix_available")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn result_message(result: &Value) -> &str {
    result.get("message").and_then(Value::as_str).unwrap_or("")
}

fn result_details(result: &Value) -> Option<&str> {
    result.get("details").and_then(Value::as_str)
}

fn build_recovery_plan(report: &DoctorReportData) -> RecoveryPlanReport {
    let mut actions = Vec::new();
    for result in &report.results {
        if let Some(action) = recovery_action_for_doctor_result(result) {
            if !actions
                .iter()
                .any(|existing: &RecoveryPlanAction| existing.id == action.id)
            {
                actions.push(action);
            }
        }
    }

    let highest_severity = actions
        .iter()
        .map(|action| action.severity.as_str())
        .max_by_key(|severity| severity_rank(severity))
        .unwrap_or("none")
        .to_string();
    let automatic_action_count = actions
        .iter()
        .filter(|action| action.safe_to_run_automatically)
        .count();

    RecoveryPlanReport {
        schema_version: 1,
        title: "Yazelix Recovery Fix Plan".into(),
        inspect_command: "yzx inspect --json".into(),
        summary: RecoveryPlanSummary {
            action_count: actions.len(),
            automatic_action_count,
            manual_action_count: actions.len().saturating_sub(automatic_action_count),
            highest_severity,
        },
        actions,
    }
}

fn recovery_action_for_doctor_result(result: &Value) -> Option<RecoveryPlanAction> {
    let message = result_message(result);
    let details = result_details(result).unwrap_or("");
    let fix_action = result
        .get("fix_action")
        .and_then(Value::as_str)
        .unwrap_or("");
    let evidence = evidence_lines(message, details);

    if fix_action == "repair_generated_runtime_state"
        || message.contains("Generated workspace assets are missing or stale")
    {
        return Some(RecoveryPlanAction {
            id: "repair_generated_runtime_state".into(),
            severity: normalize_recovery_severity(result_status(result)),
            problem: "Generated workspace assets are missing, stale, or out of sync with the active runtime".into(),
            evidence,
            commands: vec!["yzx doctor --fix".into(), "yzx restart".into()],
            safe_to_run_automatically: true,
            rationale: "`yzx doctor --fix` only regenerates Yazelix-owned generated runtime state for this finding; restart makes Zellij load the fresh assets.".into(),
        });
    }

    if message.contains("default Nix profile still contains standalone Yazelix packages")
        || details.contains("Home Manager now owns this Yazelix install")
    {
        return Some(RecoveryPlanAction {
            id: "resolve_home_manager_profile_collision".into(),
            severity: normalize_recovery_severity(result_status(result)),
            problem: "Home Manager ownership conflicts with standalone Yazelix packages in the default Nix profile".into(),
            evidence,
            commands: vec![
                "yzx home_manager prepare --apply".into(),
                "home-manager switch".into(),
            ],
            safe_to_run_automatically: false,
            rationale: "This changes package ownership and can remove profile entries, so the user should run it deliberately from the Home Manager-owned setup.".into(),
        });
    }

    if message.contains("stale host-shell yzx function or alias")
        || message.contains("stale user-local yzx wrapper")
        || message.contains("shadows the profile-owned Yazelix command")
    {
        return Some(RecoveryPlanAction {
            id: "remove_shadowed_yzx_launcher".into(),
            severity: normalize_recovery_severity(result_status(result)),
            problem: "A stale shell function, alias, or local wrapper is shadowing the current Yazelix command".into(),
            evidence,
            commands: vec![
                "command yzx doctor --fix-plan".into(),
                "yzx home_manager prepare --apply".into(),
            ],
            safe_to_run_automatically: false,
            rationale: "The exact stale definition usually lives in a user shell startup file, so Yazelix should not edit it implicitly.".into(),
        });
    }

    if message.contains("pane-orchestrator plugin permissions not granted") {
        return Some(RecoveryPlanAction {
            id: "repair_zellij_plugin_permissions".into(),
            severity: normalize_recovery_severity(result_status(result)),
            problem: "The active Zellij session has not granted Yazelix pane-orchestrator permissions".into(),
            evidence,
            commands: vec!["yzx doctor --fix".into(), "yzx restart".into()],
            safe_to_run_automatically: false,
            rationale: "Permission seeding is safe, but restarting the interactive session should be an explicit user action.".into(),
        });
    }

    if message.contains("pane-orchestrator session state is not ready")
        || message.contains("Could not contact the Yazelix pane-orchestrator plugin")
        || message.contains("pane-orchestrator returned an unexpected response")
    {
        return Some(RecoveryPlanAction {
            id: "restart_broken_zellij_session".into(),
            severity: normalize_recovery_severity(result_status(result)),
            problem: "The active Zellij session is stale, initializing, or returning invalid Yazelix plugin state".into(),
            evidence,
            commands: vec!["yzx restart".into(), "yzx doctor --verbose".into()],
            safe_to_run_automatically: false,
            rationale: "Restarting can close panes, so the plan reports the exact recovery path without doing it automatically.".into(),
        });
    }

    if details.contains("Failure class: host-dependency problem")
        || message.contains("missing required")
        || message.contains("command not found")
    {
        return Some(RecoveryPlanAction {
            id: "repair_missing_runtime_tool".into(),
            severity: normalize_recovery_severity(result_status(result)),
            problem: "A required runtime command or host dependency is missing for the active Yazelix mode".into(),
            evidence,
            commands: vec!["yzx inspect --json".into(), "yzx doctor --verbose".into()],
            safe_to_run_automatically: false,
            rationale: "Missing tools depend on the install owner and platform; inspect plus verbose doctor gives the exact active runtime and failing dependency before package changes.".into(),
        });
    }

    None
}

fn evidence_lines(message: &str, details: &str) -> Vec<String> {
    let mut lines = Vec::new();
    if !message.trim().is_empty() {
        lines.push(message.trim().to_string());
    }
    for line in details
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        lines.push(line.to_string());
    }
    lines
}

fn normalize_recovery_severity(status: &str) -> String {
    match status {
        "error" => "error".into(),
        "warn" | "warning" => "warning".into(),
        "info" => "info".into(),
        _ => "notice".into(),
    }
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "error" => 4,
        "warning" => 3,
        "info" => 2,
        "notice" => 1,
        _ => 0,
    }
}

fn render_recovery_plan(plan: &RecoveryPlanReport) {
    println!("{}", plan.title);
    println!("Inspect source: {}", plan.inspect_command);
    println!();

    if plan.actions.is_empty() {
        println!("No recovery actions found. Run `yzx doctor --verbose` if a problem persists.");
        return;
    }

    for action in &plan.actions {
        println!("[{}] {}", action.severity, action.problem);
        if let Some(first) = action.evidence.first() {
            println!("  Evidence: {first}");
        }
        println!(
            "  Safe to auto-run: {}",
            if action.safe_to_run_automatically {
                "yes"
            } else {
                "no"
            }
        );
        println!("  Commands:");
        for command in &action.commands {
            println!("    {command}");
        }
        println!("  Why: {}", action.rationale);
        println!();
    }
}

fn render_doctor_report(report: &DoctorReportData, verbose: bool) {
    println!("🔍 Running Yazelix Health Checks...\n");

    for result in &report.results {
        let icon = match result_status(result) {
            "ok" => "✅",
            "info" => "ℹ️ ",
            "warning" => "⚠️ ",
            "error" => "❌",
            _ => "•",
        };
        println!("{icon} {}", result_message(result));
        if verbose {
            if let Some(details) =
                result_details(result).filter(|details| !details.trim().is_empty())
            {
                println!("   {details}");
            }
        }
    }

    println!();

    if report.summary.error_count > 0 {
        println!("❌ Found {} errors", report.summary.error_count);
    }
    if report.summary.warning_count > 0 {
        println!("⚠️  Found {} warnings", report.summary.warning_count);
    }
    if report.summary.healthy {
        println!("🎉 All checks passed! Yazelix is healthy.");
    }
}

fn print_runtime_conflict_fix_commands(results: &[Value]) {
    for result in results {
        if result_status(result) != "error" || !result_message(result).contains("runtime") {
            continue;
        }
        let Some(commands) = result.get("fix_commands").and_then(Value::as_array) else {
            continue;
        };
        if commands.is_empty() {
            continue;
        }
        println!("\n🔧 To fix runtime conflicts, run these commands:");
        for command in commands.iter().filter_map(Value::as_str) {
            println!("  {command}");
        }
    }
}

fn run_doctor_fix_flow(verbose: bool, results: &[Value]) -> Result<i32, CoreError> {
    println!("\n🔧 Attempting to auto-fix issues...\n");

    let mut any_failed = false;

    // 1. Helix runtime conflicts
    for result in results {
        let status = result.get("status").and_then(Value::as_str).unwrap_or("");
        let message = result.get("message").and_then(Value::as_str).unwrap_or("");
        let fix_available = result
            .get("fix_available")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let conflicts = result.get("conflicts").and_then(Value::as_array);

        if !matches!(status, "error" | "warning") || !message.contains("runtime") || !fix_available
        {
            continue;
        }
        let Some(conflicts) = conflicts else { continue };

        for conflict in conflicts {
            let severity = conflict
                .get("severity")
                .and_then(Value::as_str)
                .unwrap_or("");
            let path = conflict.get("path").and_then(Value::as_str).unwrap_or("");
            let name = conflict.get("name").and_then(Value::as_str).unwrap_or("");
            if severity != "error" || path.is_empty() {
                continue;
            }
            let backup = format!("{path}.backup");
            match fs::rename(path, &backup) {
                Ok(()) => println!("✅ Moved {name} from {path} to {backup}"),
                Err(err) => {
                    println!("❌ Failed to move {name} from {path}: {err}");
                    any_failed = true;
                }
            }
        }
    }

    // 2. Config creation from template
    for result in results {
        let status = result.get("status").and_then(Value::as_str).unwrap_or("");
        let message = result.get("message").and_then(Value::as_str).unwrap_or("");
        if status != "info" || !message.contains("default") {
            continue;
        }
        let runtime_dir = runtime_dir_from_env()?;
        let config_dir = config_dir_from_env()?;
        let paths = primary_config_paths(&runtime_dir, &config_dir);
        if let Some(parent) = paths.user_config.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                println!("❌ Failed to create config parent directory: {err}");
                any_failed = true;
                continue;
            }
        }
        let rendered = match render_default_settings_jsonc(
            &paths.default_config_path,
            &paths.default_cursor_config_path,
        ) {
            Ok(rendered) => rendered,
            Err(err) => {
                println!("❌ Failed to render settings.jsonc from defaults: {err}");
                any_failed = true;
                continue;
            }
        };
        match fs::write(&paths.user_config, rendered) {
            Ok(_) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Err(err) =
                        fs::set_permissions(&paths.user_config, fs::Permissions::from_mode(0o644))
                    {
                        println!("⚠️  Could not set permissions on created config: {err}");
                    }
                }
                println!("✅ Created settings.jsonc from shipped defaults");
            }
            Err(err) => {
                println!("❌ Failed to create settings.jsonc: {err}");
                any_failed = true;
            }
        }
    }

    // 3. Generated runtime state repair
    let needs_runtime_repair = results.iter().any(|r| {
        r.get("fix_action")
            .and_then(Value::as_str)
            .map(|a| a == "repair_generated_runtime_state")
            .unwrap_or(false)
    });
    if needs_runtime_repair {
        let plan_request =
            runtime_materialization_plan_request_from_env(config_override_from_env().as_deref())?;
        let repair_req = RuntimeMaterializationRepairEvaluateRequest {
            plan: plan_request,
            force: false,
        };
        match repair_runtime_materialization(&repair_req) {
            Ok(data) => match &data.repair {
                crate::runtime_materialization::RuntimeRepairDirective::Noop { lines } => {
                    if verbose {
                        for line in lines {
                            println!("{line}");
                        }
                    }
                }
                crate::runtime_materialization::RuntimeRepairDirective::Regenerate {
                    progress_message,
                    missing_artifacts_detail_line,
                    success_lines,
                    ..
                } => {
                    if verbose {
                        if !progress_message.is_empty() {
                            println!("{progress_message}");
                        }
                        if let Some(detail) = missing_artifacts_detail_line {
                            println!("{detail}");
                        }
                    }
                    for line in success_lines {
                        println!("{line}");
                    }
                }
            },
            Err(err) => {
                println!(
                    "❌ Failed to repair generated runtime state: {}",
                    err.message()
                );
                any_failed = true;
            }
        }
    }

    // 4. Zellij plugin permission seeding
    let needs_plugin_seed = results.iter().any(|r| {
        r.get("fix_action")
            .and_then(Value::as_str)
            .map(|a| a == "seed_zellij_plugin_permissions")
            .unwrap_or(false)
    });
    if needs_plugin_seed {
        let runtime_dir = runtime_dir_from_env()?;
        let config_dir = config_dir_from_env()?;
        let state_dir = state_dir_from_env()?;
        let paths = resolve_active_config_paths(
            &runtime_dir,
            &config_dir,
            config_override_from_env().as_deref(),
        )?;
        let zellij_config_dir = state_dir.join("configs").join("zellij");
        let req = ZellijMaterializationRequest {
            config_path: paths.config_file,
            default_config_path: paths.default_config_path,
            contract_path: paths.contract_path,
            runtime_dir: runtime_dir.clone(),
            zellij_config_dir,
            seed_plugin_permissions: true,
        };
        match generate_zellij_materialization(&req) {
            Ok(_) => {
                let cache_path = zellij_permissions_cache_path()?;
                println!(
                    "✅ Seeded Yazelix plugin permissions in: {}",
                    cache_path.display()
                );
            }
            Err(err) => {
                println!(
                    "❌ Failed to seed Yazelix plugin permissions: {}",
                    err.message()
                );
                any_failed = true;
            }
        }
    }

    println!("\n✅ Auto-fix completed. Run 'yzx doctor' again to verify.");
    Ok(if any_failed { 1 } else { 0 })
}

fn collect_zellij_plugin_health_findings(
    _normalized_config: Option<&serde_json::Map<String, Value>>,
) -> Vec<Value> {
    if env::var_os("ZELLIJ").is_none() {
        return vec![json!({
            "status": "info",
            "message": "Zellij plugin health check skipped (not inside Zellij)",
            "details": "Run `yzx doctor` from inside the affected Yazelix session to verify Yazelix orchestrator permissions and managed pane detection.",
            "fix_available": false
        })];
    }

    let output = Command::new("zellij")
        .args([
            "action",
            "pipe",
            "--plugin",
            "yazelix_pane_orchestrator",
            "--name",
            "get_active_tab_session_state",
            "--",
            "",
        ])
        .output();

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return vec![json!({
                "status": "warning",
                "message": "Could not contact the Yazelix pane-orchestrator plugin",
                "details": format!("Run this from inside the affected Yazelix session after fully restarting it. Underlying error: {error}"),
                "fix_available": false
            })];
        }
    };

    if !output.status.success() {
        return vec![json!({
            "status": "warning",
            "message": "Could not contact the Yazelix pane-orchestrator plugin",
            "details": format!(
                "Run this from inside the affected Yazelix session after fully restarting it. Underlying error: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            "fix_available": false
        })];
    }

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    match raw.as_str() {
        "permissions_denied" => build_zellij_plugin_health_findings(
            &ZellijPluginState {
                permissions_granted: false,
                active_tab_position: None,
                sidebar_pane_id: String::new(),
                editor_pane_id: String::new(),
                active_swap_layout_name: None,
            },
            true,
        ),
        "not_ready" | "missing" => vec![json!({
            "status": "warning",
            "message": "Yazelix pane-orchestrator session state is not ready yet",
            "details": "The plugin responded before tab/workspace state was available. Wait a moment and rerun `yzx doctor` inside this Yazelix session.",
            "fix_available": false
        })],
        _ => match serde_json::from_str::<ActiveTabSessionStateV1>(&raw) {
            Ok(session) => build_zellij_plugin_health_findings(
                &ZellijPluginState {
                    permissions_granted: true,
                    active_tab_position: Some(session.active_tab_position),
                    sidebar_pane_id: session.managed_panes.sidebar_pane_id.unwrap_or_default(),
                    editor_pane_id: session.managed_panes.editor_pane_id.unwrap_or_default(),
                    active_swap_layout_name: session.layout.active_swap_layout_name,
                },
                true,
            ),
            Err(_) => vec![json!({
                "status": "warning",
                "message": "Yazelix pane-orchestrator returned an unexpected response",
                "details": format!("Unexpected payload: {raw}"),
                "fix_available": false
            })],
        },
    }
}

fn build_zellij_plugin_health_findings(
    plugin_state: &ZellijPluginState,
    sidebar_enabled: bool,
) -> Vec<Value> {
    let mut results = Vec::new();

    if !plugin_state.permissions_granted {
        results.push(json!({
            "status": "error",
            "message": "Yazelix pane-orchestrator plugin permissions not granted",
            "details": "Yazelix normally pre-seeds bundled Zellij plugin permissions before launch. If the cache was deleted or Zellij is already prompting, run `yzx doctor --fix` and restart Yazelix; if a live prompt remains, focus the top zjstatus bar and press `y`, and answer yes to the Yazelix orchestrator popup. Yazelix workspace bindings like `Alt+m`, `Alt+y`, `Ctrl+y`, `Alt+r`, `Alt+[`, and `Alt+]` depend on the orchestrator.",
            "fix_available": true,
            "fix_action": "seed_zellij_plugin_permissions"
        }));
    } else {
        results.push(json!({
            "status": "ok",
            "message": "Yazelix pane-orchestrator permissions granted",
            "details": "The orchestrator plugin can handle Yazelix tab and pane actions in this Zellij session.",
            "fix_available": false
        }));
    }

    if plugin_state.active_tab_position.is_none() {
        results.push(json!({
            "status": "warning",
            "message": "Yazelix pane-orchestrator does not see an active tab yet",
            "details": "The plugin may still be initializing. Wait a moment and rerun `yzx doctor` inside this Yazelix session.",
            "fix_available": false
        }));
        return results;
    }

    if sidebar_enabled {
        if plugin_state.sidebar_pane_id.trim().is_empty() {
            results.push(json!({
                "status": "warning",
                "message": "Managed sidebar pane not detected in the current tab",
                "details": "`Alt+y`, `Ctrl+y`, and reveal flows may not work until the current tab uses a Yazelix managed-sidebar layout.",
                "fix_available": false
            }));
        } else {
            results.push(json!({
                "status": "ok",
                "message": format!("Managed sidebar pane detected: {}", plugin_state.sidebar_pane_id),
                "details": format!(
                    "Layout state: {}",
                    plugin_state
                        .active_swap_layout_name
                        .as_deref()
                        .unwrap_or("unknown")
                ),
                "fix_available": false
            }));
        }
    }

    if plugin_state.editor_pane_id.trim().is_empty() {
        results.push(json!({
            "status": "info",
            "message": "Managed editor pane not detected in the current tab",
            "details": "This is normal until you open a managed Helix or Neovim editor pane in the current tab. An editor started manually from an ordinary shell pane does not count as the managed editor pane.",
            "fix_available": false
        }));
    } else {
        results.push(json!({
            "status": "ok",
            "message": format!("Managed editor pane detected: {}", plugin_state.editor_pane_id),
            "details": Value::Null,
            "fix_available": false
        }));
    }

    results
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;

    // Defends: the Rust doctor summary keeps warnings and fixable findings from being treated as healthy.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn doctor_summary_tracks_fixable_warning_state() {
        let summary = summarize_doctor_results(&[
            json!({"status": "warning", "message": "warning", "fix_available": false}),
            json!({"status": "info", "message": "info", "fix_available": true}),
            json!({"status": "ok", "message": "ok", "fix_available": false}),
        ]);

        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 1);
        assert_eq!(summary.ok_count, 1);
        assert_eq!(summary.fixable_count, 1);
        assert!(!summary.healthy);
    }

    // Defends: the Rust doctor port preserves the Zellij permission-denied finding and fix action instead of dropping the live-session seam.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn zellij_permissions_denied_stays_fixable() {
        let findings = build_zellij_plugin_health_findings(
            &ZellijPluginState {
                permissions_granted: false,
                active_tab_position: None,
                sidebar_pane_id: String::new(),
                editor_pane_id: String::new(),
                active_swap_layout_name: None,
            },
            true,
        );

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0]["status"], "error");
        assert_eq!(
            findings[0]["fix_action"].as_str(),
            Some("seed_zellij_plugin_permissions")
        );
        assert_eq!(findings[1]["status"], "warning");
    }

    // Defends: the recovery plan maps known high-friction failures to exact non-mutating recovery commands.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn recovery_plan_maps_common_failure_states_to_exact_commands() {
        let report = DoctorReportData {
            title: "Yazelix Health Checks".into(),
            summary: summarize_doctor_results(&[]),
            results: vec![
                json!({
                    "status": "error",
                    "message": "Generated workspace assets are missing or stale",
                    "details": "generated Zellij plugin artifact is stale: /tmp/yazelix_pane_orchestrator.wasm",
                    "fix_available": true,
                    "fix_action": "repair_generated_runtime_state"
                }),
                json!({
                    "status": "warn",
                    "message": "The default Nix profile still contains standalone Yazelix packages alongside the Home Manager install",
                    "details": "Home Manager now owns this Yazelix install, but the default Nix profile still contains standalone Yazelix package entries."
                }),
                json!({
                    "status": "warning",
                    "message": "A stale user-local yzx wrapper shadows the profile-owned Yazelix command",
                    "details": "Shell-resolved yzx: /home/user/.local/bin/yzx"
                }),
                json!({
                    "status": "warning",
                    "message": "Yazelix pane-orchestrator returned an unexpected response",
                    "details": "Unexpected payload: not-json"
                }),
            ],
        };

        let plan = build_recovery_plan(&report);
        let ids = plan
            .actions
            .iter()
            .map(|action| action.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                "repair_generated_runtime_state",
                "resolve_home_manager_profile_collision",
                "remove_shadowed_yzx_launcher",
                "restart_broken_zellij_session",
            ]
        );
        assert_eq!(
            plan.actions[0].commands,
            vec!["yzx doctor --fix", "yzx restart"]
        );
        assert!(plan.actions[0].safe_to_run_automatically);
        assert_eq!(
            plan.actions[1].commands,
            vec!["yzx home_manager prepare --apply", "home-manager switch"]
        );
        assert!(!plan.actions[1].safe_to_run_automatically);
        assert_eq!(plan.summary.highest_severity, "error");
        assert_eq!(plan.summary.automatic_action_count, 1);
        assert_eq!(plan.summary.manual_action_count, 3);
    }
}
