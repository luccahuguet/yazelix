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
use crate::doctor_zellij_plugin_health::{
    SEED_ZELLIJ_PLUGIN_PERMISSIONS_FIX_ACTION, evaluate_zellij_plugin_health,
    seed_zellij_plugin_permissions, zellij_plugin_health_request_from_env,
};
use crate::helix_external::HelixExternalPair;
use crate::initializer_commands::generate_shell_initializers_for_env;
use crate::install_ownership_env::install_ownership_request_from_env_with_runtime_dir;
use crate::native_config_status::{
    NativeConfigStatusEntry, NativeConfigStatusRequest, classify_native_config_statuses,
    current_platform_name, highest_doctor_severity, path_owned_by_home_manager,
    status_code_for_entry, xdg_config_home_from_env,
};
use crate::runtime_materialization::{
    RuntimeMaterializationPlanData, RuntimeMaterializationRepairEvaluateRequest,
    repair_runtime_materialization,
};
use crate::settings_surface::render_default_settings_jsonc;
use crate::terminal_variant::active_terminal_from_runtime_dir;
use crate::user_config_paths;
use crate::workspace_asset_contract::{
    WorkspaceAssetEvaluateRequest, evaluate_workspace_asset_report,
};
use crate::{
    DoctorConfigEvaluateRequest, NormalizeConfigRequest, evaluate_doctor_config_report,
    evaluate_install_ownership_report, normalize_config, plan_runtime_materialization,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum DoctorTarget {
    #[default]
    All,
    HelixSteel,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DoctorCliArgs {
    target: DoctorTarget,
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

#[derive(Debug, Clone, Copy)]
struct DoctorJson<'a>(&'a Value);

type DoctorFinding<'a> = DoctorJson<'a>;

impl<'a> DoctorJson<'a> {
    fn new(value: &'a Value) -> Self {
        Self(value)
    }
    fn field(self, field: &str) -> &'a str {
        self.0.get(field).and_then(Value::as_str).unwrap_or("")
    }
    fn bool_field(self, field: &str) -> bool {
        self.0.get(field).and_then(Value::as_bool).unwrap_or(false)
    }
    fn status(self) -> &'a str {
        self.field("status")
    }
    fn message(self) -> &'a str {
        self.field("message")
    }
    fn fix_available(self) -> bool {
        self.bool_field("fix_available")
    }
    fn fix_action(self) -> Option<&'a str> {
        self.0.get("fix_action").and_then(Value::as_str)
    }
    fn conflicts(self) -> impl Iterator<Item = DoctorJson<'a>> + 'a {
        self.0
            .get("conflicts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .map(DoctorJson::new)
    }
}

fn doctor_findings(results: &[Value]) -> impl Iterator<Item = DoctorFinding<'_>> {
    results.iter().map(DoctorFinding::new)
}

fn doctor_finding_json(
    status: impl Into<String>,
    message: impl Into<String>,
    details: impl Into<Value>,
) -> Value {
    json!({
        "status": status.into(),
        "message": message.into(),
        "details": details.into(),
        "fix_available": false
    })
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
    id: &'static str,
    severity: String,
    problem: &'static str,
    evidence: Vec<String>,
    commands: &'static [&'static str],
    safe_to_run_automatically: bool,
    rationale: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RecoveryPlanReport {
    schema_version: u8,
    title: String,
    inspect_command: String,
    summary: RecoveryPlanSummary,
    actions: Vec<RecoveryPlanAction>,
}

const CREATE_DEFAULT_SETTINGS_CONFIG_FIX_ACTION: &str = "create_default_settings_config";
const HELIX_RUNTIME_CONFLICT_REPAIR_ACTION: &str = "backup_helix_runtime_conflicts";
const REPAIR_GENERATED_RUNTIME_STATE_FIX_ACTION: &str = "repair_generated_runtime_state";

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

    let report = match parsed.target {
        DoctorTarget::All => compute_doctor_report_from_env()?,
        DoctorTarget::HelixSteel => compute_helix_steel_doctor_report_from_env()?,
    };
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
    println!("  yzx doctor helix-steel [--verbose] [--json]");
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
            "helix-steel" => {
                if out.target != DoctorTarget::All {
                    return Err(CoreError::classified(
                        ErrorClass::Usage,
                        "duplicate_doctor_target",
                        "Only one yzx doctor target can be selected.",
                        "Run `yzx doctor helix-steel` or `yzx doctor`, not both target forms.",
                        json!({}),
                    ));
                }
                out.target = DoctorTarget::HelixSteel;
            }
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

fn compute_helix_steel_doctor_report_from_env() -> Result<DoctorReportData, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    let state_dir = state_dir_from_env()?;
    let home_dir = home_dir_from_env()?;
    let normalized_config = load_optional_doctor_normalized_config(&runtime_dir, &config_dir);
    let results = collect_helix_doctor_findings(
        &runtime_dir,
        &config_dir,
        &state_dir,
        &home_dir,
        normalized_config.as_ref(),
    );
    let summary = summarize_doctor_results(&results);
    Ok(DoctorReportData {
        title: "Yazelix Helix Steel Checks".to_string(),
        results,
        summary,
    })
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
    let native_config_findings = collect_native_config_status_findings(
        &runtime_dir,
        &home_dir,
        &config_dir,
        &state_dir,
        normalized_config.as_ref(),
    )?;
    let workspace_asset_findings =
        collect_workspace_asset_doctor_findings(&runtime_dir, &state_dir);
    let zellij_findings = collect_zellij_plugin_health_findings(normalized_config.as_ref())?;

    let mut results = Vec::new();
    results.extend(runtime_findings);
    results.extend(helix_findings);
    results.extend(config_findings);
    results.extend(native_config_findings);
    results.extend(workspace_asset_findings);
    results.extend(serialize_values(&install_report.wrapper_shadowing)?);
    results.push(serialize_value(&install_report.install_owner_diagnostic)?);
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
        include_missing: true,
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
                    extra.push(runtime_materialization_plan_doctor_finding(&plan));
                    match active_terminal_from_runtime_dir(runtime_dir) {
                        Ok(active_terminal) => Some(SharedRuntimePreflightInput {
                            zellij_layout_path: PathBuf::from(plan.zellij_layout_path),
                            terminals: vec![active_terminal],
                            launch_script_path: runtime_dir
                                .join("shells")
                                .join("posix")
                                .join("start_yazelix.sh"),
                            command_search_paths: env::var_os("PATH")
                                .map(|raw| env::split_paths(&raw).collect())
                                .unwrap_or_default(),
                            platform_name: platform_name_for_runtime_doctor(),
                        }),
                        Err(error) => {
                            extra.push(doctor_finding_json(
                                "error",
                                "Could not resolve the selected packaged terminal from runtime metadata",
                                error.message(),
                            ));
                            None
                        }
                    }
                }
                Err(error) => {
                    extra.push(doctor_finding_json(
                        "error",
                        "Could not resolve the managed Zellij layout path from the Rust materialization plan",
                        error.message(),
                    ));
                    None
                }
            },
            Err(error) => {
                extra.push(doctor_finding_json(
                    "error",
                    "Could not resolve the managed Zellij layout path from the Rust materialization plan",
                    error.message(),
                ));
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
    results.push(expect_serialized_value(&data.distribution));
    results.extend(extra);
    results.extend(expect_serialized_values(&data.shared_runtime_preflight));
    results
}

fn runtime_materialization_plan_doctor_finding(plan: &RuntimeMaterializationPlanData) -> Value {
    let missing_artifacts = plan
        .missing_artifacts
        .iter()
        .map(|artifact| artifact.label.as_str())
        .collect::<Vec<_>>();
    let input_freshness = json!({
        "config_changed": plan.config_state.config_changed,
        "runtime_inputs_changed": plan.config_state.inputs_changed,
        "requires_refresh": plan.config_state.inputs_require_refresh,
    });

    if plan.should_regenerate {
        return json!({
            "status": "warning",
            "message": "Generated runtime state needs repair",
            "details": plan.reason,
            "fix_available": true,
            "fix_action": REPAIR_GENERATED_RUNTIME_STATE_FIX_ACTION,
            "owner_surface": "doctor",
            "generated_state_check": "runtime_materialization_plan",
            "materialization_status": plan.status,
            "missing_artifacts": missing_artifacts,
            "input_freshness": input_freshness,
        });
    }

    json!({
        "status": "ok",
        "message": "Generated runtime state is current",
        "details": plan.reason,
        "fix_available": false,
        "owner_surface": "doctor",
        "generated_state_check": "runtime_materialization_plan",
        "materialization_status": plan.status,
        "missing_artifacts": missing_artifacts,
        "input_freshness": input_freshness,
    })
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
    let helix_external = normalized_config
        .and_then(|cfg| cfg.get("helix_external"))
        .and_then(HelixExternalPair::from_json);
    let hx_exe_path = helix_external
        .as_ref()
        .map(|external| PathBuf::from(&external.binary))
        .or_else(|| find_external_command("hx"));
    let include_runtime_health = helix_external.is_some()
        || env::var("EDITOR")
            .ok()
            .map(|value| value.contains("hx"))
            .unwrap_or(false);
    let request = HelixDoctorEvaluateRequest {
        home_dir: home_dir.to_path_buf(),
        runtime_dir: runtime_dir.to_path_buf(),
        config_dir: config_dir.to_path_buf(),
        user_config_helix_runtime_dir: home_dir.join(".config").join("helix").join("runtime"),
        hx_exe_path,
        helix_external,
        include_runtime_health,
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
    results.push(expect_serialized_value(&data.runtime_conflicts));
    if let Some(runtime_health) = &data.runtime_health {
        results.push(expect_serialized_value(runtime_health));
    }
    if let Some(external_pair) = &data.external_pair {
        results.push(expect_serialized_value(external_pair));
    }
    results.extend(expect_serialized_values(&data.managed_integration));
    results
}

fn collect_config_doctor_findings(runtime_dir: &Path, config_dir: &Path) -> Vec<Value> {
    let data = evaluate_doctor_config_report(&DoctorConfigEvaluateRequest {
        config_dir: config_dir.to_path_buf(),
        runtime_dir: runtime_dir.to_path_buf(),
    });
    expect_serialized_values(&data.findings)
}

fn collect_native_config_status_findings(
    runtime_dir: &Path,
    home_dir: &Path,
    config_dir: &Path,
    state_dir: &Path,
    normalized_config: Option<&serde_json::Map<String, Value>>,
) -> Result<Vec<Value>, CoreError> {
    let settings_path = user_config_paths::main_config(config_dir);
    let entries = classify_native_config_statuses(&NativeConfigStatusRequest {
        home_dir: home_dir.to_path_buf(),
        xdg_config_home: xdg_config_home_from_env(home_dir),
        config_dir: config_dir.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        platform: current_platform_name(),
        terminal_config_mode: normalized_string_config(
            normalized_config,
            "terminal_config_mode",
            "yazelix",
        ),
        active_terminal: active_terminal_from_runtime_dir(runtime_dir)?,
        settings_home_manager_read_only: path_owned_by_home_manager(&settings_path),
    });
    Ok(vec![native_config_status_finding(entries)])
}

fn normalized_string_config(
    config: Option<&serde_json::Map<String, Value>>,
    key: &str,
    fallback: &str,
) -> String {
    config
        .and_then(|config| config.get(key))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn native_config_status_finding(entries: Vec<NativeConfigStatusEntry>) -> Value {
    let severity = highest_doctor_severity(&entries);
    let warning_count = entries
        .iter()
        .filter_map(status_code_for_entry)
        .filter(|status| status.doctor_severity() == "warning")
        .count();
    let error_count = entries
        .iter()
        .filter_map(status_code_for_entry)
        .filter(|status| status.doctor_severity() == "error")
        .count();
    let import_count = entries
        .iter()
        .filter(|entry| entry.status == "native_available")
        .count();
    let details = format!(
        "{error_count} required native config errors; {warning_count} read-only native/Home Manager surfaces; {import_count} native config files available to import."
    );

    json!({
        "status": severity,
        "message": "Native config integration status",
        "details": details,
        "fix_available": false,
        "native_config_statuses": entries,
    })
}

fn collect_workspace_asset_doctor_findings(runtime_dir: &Path, state_dir: &Path) -> Vec<Value> {
    expect_serialized_values(&evaluate_workspace_asset_report(
        &WorkspaceAssetEvaluateRequest {
            runtime_dir: runtime_dir.to_path_buf(),
            state_dir: state_dir.to_path_buf(),
        },
    ))
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

fn serialize_values<T: Serialize>(values: &[T]) -> Result<Vec<Value>, CoreError> {
    values.iter().map(serialize_value).collect()
}

fn expect_serialized_value<T: Serialize>(value: &T) -> Value {
    serialize_value(value).expect("doctor finding should serialize")
}

fn expect_serialized_values<T: Serialize>(values: &[T]) -> Vec<Value> {
    serialize_values(values).expect("doctor findings should serialize")
}

fn summarize_doctor_results(results: &[Value]) -> DoctorReportSummary {
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;
    let mut ok_count = 0;
    let mut fixable_count = 0;

    for finding in doctor_findings(results) {
        match finding.status() {
            "error" => error_count += 1,
            "warning" => warning_count += 1,
            "info" => info_count += 1,
            "ok" => ok_count += 1,
            _ => {}
        }
        if finding.fix_available() {
            fixable_count += 1;
        }
    }

    DoctorReportSummary {
        error_count,
        warning_count,
        info_count,
        ok_count,
        fixable_count,
        healthy: error_count == 0 && warning_count == 0 && fixable_count == 0,
    }
}

fn has_fix_action(results: &[Value], action: &str) -> bool {
    doctor_findings(results).any(|finding| finding.fix_action() == Some(action))
}

fn needs_helix_runtime_conflict_backup(results: &[Value]) -> bool {
    doctor_findings(results).any(|finding| {
        matches!(finding.status(), "error" | "warning")
            && finding.message().contains("runtime")
            && finding.fix_available()
            && finding.conflicts().any(|conflict| {
                conflict.field("severity") == "error" && !conflict.field("path").is_empty()
            })
    })
}

fn doctor_repair_action_ids(results: &[Value]) -> Vec<&'static str> {
    let mut actions = Vec::new();
    if needs_helix_runtime_conflict_backup(results) {
        actions.push(HELIX_RUNTIME_CONFLICT_REPAIR_ACTION);
    }
    if has_fix_action(results, CREATE_DEFAULT_SETTINGS_CONFIG_FIX_ACTION) {
        actions.push(CREATE_DEFAULT_SETTINGS_CONFIG_FIX_ACTION);
    }
    if has_fix_action(results, REPAIR_GENERATED_RUNTIME_STATE_FIX_ACTION) {
        actions.push(REPAIR_GENERATED_RUNTIME_STATE_FIX_ACTION);
    }
    if has_fix_action(results, SEED_ZELLIJ_PLUGIN_PERMISSIONS_FIX_ACTION) {
        actions.push(SEED_ZELLIJ_PLUGIN_PERMISSIONS_FIX_ACTION);
    }
    actions
}

fn create_default_settings_config_from_template(
    runtime_dir: &Path,
    config_dir: &Path,
) -> Result<bool, CoreError> {
    let paths = primary_config_paths(runtime_dir, config_dir);
    if paths.user_config.exists() {
        return Ok(false);
    }

    if let Some(parent) = paths.user_config.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "doctor_create_config_parent",
                "Could not create settings.jsonc parent directory.",
                "Fix permissions for the Yazelix config directory, then rerun `yzx doctor --fix`.",
                parent.to_string_lossy().into_owned(),
                source,
            )
        })?;
    }

    let rendered = render_default_settings_jsonc(&paths.default_config_path)?;
    let mut file = match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&paths.user_config)
    {
        Ok(file) => file,
        Err(source) if source.kind() == io::ErrorKind::AlreadyExists => return Ok(false),
        Err(source) => {
            return Err(CoreError::io(
                "doctor_create_settings_jsonc",
                "Could not create settings.jsonc from shipped defaults.",
                "Fix permissions for the Yazelix config directory, then rerun `yzx doctor --fix`.",
                paths.user_config.to_string_lossy().into_owned(),
                source,
            ));
        }
    };

    file.write_all(rendered.as_bytes()).map_err(|source| {
        CoreError::io(
            "doctor_write_settings_jsonc",
            "Could not write settings.jsonc from shipped defaults.",
            "Fix permissions for the Yazelix config directory, then rerun `yzx doctor --fix`.",
            paths.user_config.to_string_lossy().into_owned(),
            source,
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&paths.user_config, fs::Permissions::from_mode(0o644));
    }

    Ok(true)
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
    let finding = DoctorFinding::new(result);
    let message = finding.message();
    let details = finding.field("details");
    let fix_action = finding.fix_action().unwrap_or("");
    let evidence = evidence_lines(message, details);

    if fix_action == "repair_generated_runtime_state"
        || message.contains("Generated workspace assets are missing or stale")
    {
        return Some(recovery_action(
            "repair_generated_runtime_state",
            finding,
            "Generated workspace assets are missing, stale, or out of sync with the active runtime",
            evidence,
            &["yzx doctor --fix", "yzx restart"],
            true,
            "`yzx doctor --fix` only regenerates Yazelix-owned generated runtime state for this finding; restart makes Zellij load the fresh assets.",
        ));
    }

    if message.contains("default Nix profile still contains standalone Yazelix packages")
        || details.contains("Home Manager now owns this Yazelix install")
    {
        return Some(recovery_action(
            "resolve_home_manager_profile_collision",
            finding,
            "Home Manager ownership conflicts with standalone Yazelix packages in the default Nix profile",
            evidence,
            &["yzx home_manager prepare --apply", "home-manager switch"],
            false,
            "This changes package ownership and can remove profile entries, so the user should run it deliberately from the Home Manager-owned setup.",
        ));
    }

    if message.contains("stale host-shell yzx function or alias")
        || message.contains("stale user-local yzx wrapper")
        || message.contains("shadows the profile-owned Yazelix command")
    {
        return Some(recovery_action(
            "remove_shadowed_yzx_launcher",
            finding,
            "A stale shell function, alias, or local wrapper is shadowing the current Yazelix command",
            evidence,
            &[
                "command yzx doctor --fix-plan",
                "yzx home_manager prepare --apply",
            ],
            false,
            "The exact stale definition usually lives in a user shell startup file, so Yazelix should not edit it implicitly.",
        ));
    }

    if message.contains("pane-orchestrator plugin permissions not granted") {
        return Some(recovery_action(
            "repair_zellij_plugin_permissions",
            finding,
            "The active Zellij session has not granted Yazelix pane-orchestrator permissions",
            evidence,
            &["yzx doctor --fix", "yzx restart"],
            false,
            "Permission seeding is safe, but restarting the interactive session should be an explicit user action.",
        ));
    }

    if message.contains("pane-orchestrator session state is not ready")
        || message.contains("Could not contact the Yazelix pane-orchestrator plugin")
        || message.contains("pane-orchestrator returned an unexpected response")
    {
        return Some(recovery_action(
            "restart_broken_zellij_session",
            finding,
            "The active Zellij session is stale, initializing, or returning invalid Yazelix plugin state",
            evidence,
            &["yzx restart", "yzx doctor --verbose"],
            false,
            "Restarting can close panes, so the plan reports the exact recovery path without doing it automatically.",
        ));
    }

    if details.contains("Failure class: host-dependency problem")
        || message.contains("missing required")
        || message.contains("command not found")
    {
        return Some(recovery_action(
            "repair_missing_runtime_tool",
            finding,
            "A required runtime command or host dependency is missing for the active Yazelix mode",
            evidence,
            &["yzx inspect --json", "yzx doctor --verbose"],
            false,
            "Missing tools depend on the install owner and platform; inspect plus verbose doctor gives the exact active runtime and failing dependency before package changes.",
        ));
    }

    None
}

fn recovery_action(
    id: &'static str,
    finding: DoctorFinding<'_>,
    problem: &'static str,
    evidence: Vec<String>,
    commands: &'static [&'static str],
    safe_to_run_automatically: bool,
    rationale: &'static str,
) -> RecoveryPlanAction {
    RecoveryPlanAction {
        id,
        severity: normalize_recovery_severity(finding.status()),
        problem,
        evidence,
        commands,
        safe_to_run_automatically,
        rationale,
    }
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
        for command in action.commands {
            println!("    {command}");
        }
        println!("  Why: {}", action.rationale);
        println!();
    }
}

fn render_doctor_report(report: &DoctorReportData, verbose: bool) {
    println!("🔍 Running Yazelix Health Checks...\n");

    for result in &report.results {
        let finding = DoctorFinding::new(result);
        let icon = match finding.status() {
            "ok" => "✅",
            "info" => "ℹ️ ",
            "warning" => "⚠️ ",
            "error" => "❌",
            _ => "•",
        };
        println!("{icon} {}", finding.message());
        if verbose {
            let details = finding.field("details");
            if !details.trim().is_empty() {
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
        let finding = DoctorFinding::new(result);
        if finding.status() != "error" || !finding.message().contains("runtime") {
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
    for action_id in doctor_repair_action_ids(results) {
        if run_doctor_repair_action(action_id, verbose, results)? {
            any_failed = true;
        }
    }

    println!("\n✅ Auto-fix completed. Run 'yzx doctor' again to verify.");
    Ok(if any_failed { 1 } else { 0 })
}

fn run_doctor_repair_action(
    action_id: &str,
    verbose: bool,
    results: &[Value],
) -> Result<bool, CoreError> {
    match action_id {
        HELIX_RUNTIME_CONFLICT_REPAIR_ACTION => repair_helix_runtime_conflicts(results),
        CREATE_DEFAULT_SETTINGS_CONFIG_FIX_ACTION => create_missing_default_settings_config(),
        REPAIR_GENERATED_RUNTIME_STATE_FIX_ACTION => repair_generated_runtime_state(verbose),
        SEED_ZELLIJ_PLUGIN_PERMISSIONS_FIX_ACTION => seed_zellij_plugin_permissions(),
        _ => Err(CoreError::classified(
            ErrorClass::Internal,
            "unknown_doctor_repair_action",
            format!("Unsupported doctor repair action: {action_id}"),
            "Report this as a Yazelix bug.",
            json!({ "action_id": action_id }),
        )),
    }
}

fn repair_helix_runtime_conflicts(results: &[Value]) -> Result<bool, CoreError> {
    let mut any_failed = false;
    for finding in doctor_findings(results) {
        if !matches!(finding.status(), "error" | "warning")
            || !finding.message().contains("runtime")
            || !finding.fix_available()
        {
            continue;
        }

        for conflict in finding.conflicts() {
            let path = conflict.field("path");
            if conflict.field("severity") != "error" || path.is_empty() {
                continue;
            }
            let name = conflict.field("name");
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
    Ok(any_failed)
}

fn create_missing_default_settings_config() -> Result<bool, CoreError> {
    let runtime_dir = runtime_dir_from_env()?;
    let config_dir = config_dir_from_env()?;
    match create_default_settings_config_from_template(&runtime_dir, &config_dir) {
        Ok(true) => println!("✅ Created settings.jsonc from shipped defaults"),
        Ok(false) => {
            let paths = primary_config_paths(&runtime_dir, &config_dir);
            println!(
                "⚠️  Skipped settings.jsonc creation because {} already exists",
                paths.user_config.display()
            );
        }
        Err(err) => {
            println!("❌ Failed to create settings.jsonc: {}", err.message());
            return Ok(true);
        }
    }
    Ok(false)
}

fn repair_generated_runtime_state(verbose: bool) -> Result<bool, CoreError> {
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
            return Ok(true);
        }
    }

    // Shell initializers under ~/.local/share/yazelix/initializers/ are otherwise
    // only regenerated by `yzx enter` at session start. That leaves them stuck
    // referencing a pre-rebuild foundation package store hash
    // whenever a rebuild happens mid-session. Regenerate them here so the same
    // `doctor --fix` pass that heals the zellij/yazi config also heals the shell
    // side of the runtime handoff.
    let shells_to_configure = crate::initializer_commands::default_shells_to_configure();
    match generate_shell_initializers_for_env(&shells_to_configure, !verbose) {
        Ok(_) => {
            if verbose {
                println!("✅ Regenerated shell initializers against the current runtime.");
            }
        }
        Err(err) => {
            println!(
                "❌ Failed to regenerate shell initializers: {}",
                err.message()
            );
            return Ok(true);
        }
    }

    Ok(false)
}

fn collect_zellij_plugin_health_findings(
    _normalized_config: Option<&serde_json::Map<String, Value>>,
) -> Result<Vec<Value>, CoreError> {
    serialize_values(&evaluate_zellij_plugin_health(
        &zellij_plugin_health_request_from_env(),
    ))
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_runtime_default_settings(runtime_dir: &Path, body: &str) {
        fs::create_dir_all(runtime_dir).unwrap();
        fs::write(runtime_dir.join("settings_default.jsonc"), body).unwrap();
    }

    // Defends: the Rust doctor summary keeps warnings and fixable findings from being treated as healthy.
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

    // Regression: doctor reports stale generated-state input/cache freshness, not only generated workspace asset drift.
    #[test]
    fn materialization_plan_finding_reports_refresh_required_as_fixable() {
        let plan = RuntimeMaterializationPlanData {
            config_state: crate::ConfigStateData {
                config: serde_json::Map::new(),
                config_file: "/tmp/settings.jsonc".to_string(),
                needs_refresh: true,
                refresh_reason: "runtime inputs changed since last generated-state repair"
                    .to_string(),
                config_changed: false,
                inputs_changed: true,
                inputs_require_refresh: true,
                config_hash: "config".to_string(),
                runtime_hash: "runtime".to_string(),
                combined_hash: "combined".to_string(),
            },
            yazi_config_dir: "/tmp/yazi".to_string(),
            zellij_config_dir: "/tmp/zellij".to_string(),
            zellij_layout_path: "/tmp/zellij/layouts/yzx_side.kdl".to_string(),
            expected_artifacts: vec![],
            missing_artifacts: vec![],
            status: "refresh_required".to_string(),
            reason: "runtime inputs changed since last generated-state repair".to_string(),
            should_regenerate: true,
            should_sync_static_assets: true,
        };

        let finding = runtime_materialization_plan_doctor_finding(&plan);

        assert_eq!(finding["status"], "warning");
        assert_eq!(finding["message"], "Generated runtime state needs repair");
        assert_eq!(
            finding["fix_action"],
            REPAIR_GENERATED_RUNTIME_STATE_FIX_ACTION
        );
        assert_eq!(
            finding["generated_state_check"],
            "runtime_materialization_plan"
        );
        assert_eq!(finding["input_freshness"]["runtime_inputs_changed"], true);
    }

    // Defends: `yzx doctor helix-steel` is a first-class targeted doctor surface, not an unexpected argument.
    #[test]
    fn doctor_args_accept_helix_steel_target() {
        let parsed = parse_doctor_cli_args(&["helix-steel".into(), "--json".into()]).unwrap();

        assert_eq!(parsed.target, DoctorTarget::HelixSteel);
        assert!(parsed.json);
    }

    // Defends: stale or repeated doctor findings cannot overwrite an existing managed settings.jsonc.
    #[test]
    fn default_settings_config_creation_does_not_overwrite_existing_file() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let config_dir = tmp.path().join("config");
        write_runtime_default_settings(
            &runtime_dir,
            "{ \"core\": { \"welcome_style\": \"logo\" } }\n",
        );
        fs::create_dir_all(&config_dir).unwrap();
        let user_config = config_dir.join("settings.jsonc");
        let original = "{ \"core\": { \"welcome_style\": \"mandelbrot\" } }\n";
        fs::write(&user_config, original).unwrap();

        let created = create_default_settings_config_from_template(&runtime_dir, &config_dir)
            .expect("stale create finding should be harmless");

        assert!(!created);
        assert_eq!(fs::read_to_string(user_config).unwrap(), original);
    }

    // Defends: the explicit config-creation fix action still bootstraps first-run settings.jsonc.
    #[test]
    fn default_settings_config_creation_writes_missing_file() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let config_dir = tmp.path().join("config");
        let default_settings = "{ \"core\": { \"welcome_style\": \"logo\" } }\n";
        write_runtime_default_settings(&runtime_dir, default_settings);

        let created = create_default_settings_config_from_template(&runtime_dir, &config_dir)
            .expect("missing settings should be created");

        assert!(created);
        assert_eq!(
            fs::read_to_string(config_dir.join("settings.jsonc")).unwrap(),
            default_settings
        );
    }

    // Defends: doctor keeps the unsupported JSON mutation path closed.
    #[test]
    fn json_fix_remains_unsupported() {
        let error = run_yzx_doctor(&["--json".into(), "--fix".into()]).unwrap_err();

        assert_eq!(error.code(), "doctor_json_fix_unsupported");
    }

    // Defends: doctor consumes the shared native-config classifier and elevates required native terminal config misses to an error.
    #[test]
    fn native_config_status_finding_reports_terminal_user_mode_error() {
        let tmp = TempDir::new().unwrap();
        let mut config = serde_json::Map::new();
        config.insert("terminal_config_mode".to_string(), json!("user"));
        std::fs::write(tmp.path().join("runtime_variant"), "mars\n").unwrap();

        let findings = collect_native_config_status_findings(
            tmp.path(),
            &tmp.path().join("home"),
            &tmp.path().join("config"),
            &tmp.path().join("state"),
            Some(&config),
        )
        .unwrap();

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0]["status"], "error");
        assert_eq!(findings[0]["message"], "Native config integration status");
        assert!(
            findings[0]["native_config_statuses"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["surface"] == "terminal.mars.input"
                    && entry["status"] == "native_required_missing")
        );
    }

    // Defends: the recovery plan maps known high-friction failures to exact non-mutating recovery commands.
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
            .map(|action| action.id)
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
            &["yzx doctor --fix", "yzx restart"]
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
