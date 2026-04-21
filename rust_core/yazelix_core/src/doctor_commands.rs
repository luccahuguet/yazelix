//! Public `yzx doctor` owner for report collection, JSON output, and human rendering.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, home_dir_from_env, runtime_dir_from_env,
    runtime_materialization_plan_request_from_env, state_dir_from_env,
};
use crate::doctor_helix_report::{HelixDoctorEvaluateRequest, evaluate_helix_doctor_report};
use crate::doctor_runtime_report::{
    DoctorRuntimeEvaluateRequest, SharedRuntimePreflightInput, evaluate_doctor_runtime_report,
};
use crate::internal_nu_runner::run_internal_nu_module_command;
use crate::{
    DoctorConfigEvaluateRequest, InstallOwnershipEvaluateRequest, NormalizeConfigRequest,
    evaluate_doctor_config_report, evaluate_install_ownership_report, normalize_config,
    plan_runtime_materialization,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

const DOCTOR_FIX_MODULE_RELATIVE_PATH: &[&str] = &["nushell", "scripts", "utils", "doctor_fix.nu"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DoctorCliArgs {
    verbose: bool,
    fix: bool,
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
    println!("  yzx doctor --fix [--verbose]");
    println!();
    println!("Flags:");
    println!("  -v, --verbose  Show detailed information");
    println!("  -f, --fix      Attempt to auto-fix issues");
    println!("      --json     Emit machine-readable doctor data");
}

fn parse_doctor_cli_args(args: &[String]) -> Result<DoctorCliArgs, CoreError> {
    let mut out = DoctorCliArgs::default();
    for token in args {
        match token.as_str() {
            "--verbose" | "-v" => out.verbose = true,
            "--fix" | "-f" => out.fix = true,
            "--json" => out.json = true,
            "--help" | "-h" | "help" => out.help = true,
            other => {
                return Err(CoreError::classified(
                    ErrorClass::Usage,
                    "unexpected_doctor_token",
                    format!("Unexpected argument for yzx doctor: {other}"),
                    "Run `yzx doctor`, `yzx doctor --json`, or `yzx doctor --fix`.",
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
    let install_request = build_install_ownership_request(
        runtime_dir.clone(),
        config_dir.clone(),
        state_dir.clone(),
        home_dir.clone(),
    )?;
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
    let zellij_findings = collect_zellij_plugin_health_findings(normalized_config.as_ref());

    let mut results = Vec::new();
    results.extend(runtime_findings);
    results.extend(helix_findings);
    results.extend(config_findings);
    results.extend(
        install_report
            .wrapper_shadowing
            .iter()
            .map(serialize_value)
            .collect::<Result<Vec<_>, _>>()?,
    );
    results.push(serialize_value(&install_report.desktop_entry_freshness)?);
    results.extend(zellij_findings);

    let summary = summarize_doctor_results(&results);
    Ok(DoctorReportData {
        title: "Yazelix Health Checks".to_string(),
        results,
        summary,
    })
}

fn build_install_ownership_request(
    runtime_dir: PathBuf,
    config_dir: PathBuf,
    state_dir: PathBuf,
    home_dir: PathBuf,
) -> Result<InstallOwnershipEvaluateRequest, CoreError> {
    Ok(InstallOwnershipEvaluateRequest {
        runtime_dir,
        home_dir: home_dir.clone(),
        user: env::var("USER").ok(),
        xdg_config_home: xdg_config_home(&home_dir),
        xdg_data_home: xdg_data_home(&home_dir),
        yazelix_state_dir: state_dir,
        main_config_path: config_dir.join("user_configs").join("yazelix.toml"),
        invoked_yzx_path: env::var("YAZELIX_INVOKED_YZX_PATH").ok(),
        redirected_from_stale_yzx_path: env::var("YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH").ok(),
        shell_resolved_yzx_path: shell_resolved_yzx_path_for_report(),
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

fn xdg_data_home(home_dir: &Path) -> PathBuf {
    env::var("XDG_DATA_HOME")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".local").join("share"))
}

fn shell_resolved_yzx_path_for_report() -> Option<String> {
    env::var("YAZELIX_INVOKED_YZX_PATH")
        .ok()
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
        .or_else(|| find_external_command("yzx").map(path_to_string))
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

fn path_to_string(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().into_owned()
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
                            .join("nushell")
                            .join("scripts")
                            .join("core")
                            .join("launch_yazelix.nu"),
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
        managed_helix_user_config_path: config_dir
            .join("user_configs")
            .join("helix")
            .join("config.toml"),
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
    let runtime_dir = runtime_dir_from_env()?;
    let results_json = serde_json::to_string(results).map_err(|error| {
        CoreError::classified(
            ErrorClass::Internal,
            "invalid_doctor_fix_payload",
            format!("Failed to serialize doctor fix payload: {error}"),
            "Rebuild or reinstall Yazelix so the Rust doctor owner and fix helper agree on the fix payload.",
            json!({}),
        )
    })?;
    let mut args = Vec::new();
    if verbose {
        args.push("--verbose".to_string());
    }

    run_internal_nu_module_command(
        &runtime_dir,
        DOCTOR_FIX_MODULE_RELATIVE_PATH,
        "apply_doctor_fixes_internal",
        &args,
        &[
            ("YAZELIX_ACCEPT_USER_CONFIG_RELOCATION", "true"),
            ("YAZELIX_DOCTOR_RESULTS_JSON", &results_json),
        ],
    )
}

fn collect_zellij_plugin_health_findings(
    normalized_config: Option<&serde_json::Map<String, Value>>,
) -> Vec<Value> {
    if env::var_os("ZELLIJ").is_none() {
        return vec![json!({
            "status": "info",
            "message": "Zellij plugin health check skipped (not inside Zellij)",
            "details": "Run `yzx doctor` from inside the affected Yazelix session to verify Yazelix orchestrator permissions and managed pane detection.",
            "fix_available": false
        })];
    }

    let sidebar_enabled = normalized_config
        .and_then(|cfg| cfg.get("enable_sidebar"))
        .and_then(Value::as_bool)
        .unwrap_or(true);

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
            sidebar_enabled,
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
                sidebar_enabled,
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
            "details": "Grant the required Yazelix Zellij plugin permissions: focus the top zjstatus bar and press `y` if it prompts, and also answer yes to the Yazelix orchestrator permission popup. If permission state gets out of sync after an update, run `yzx doctor --fix` and restart Yazelix. Yazelix workspace bindings like `Alt+m`, `Alt+y`, `Ctrl+y`, `Alt+r`, `Alt+[`, and `Alt+]` depend on the orchestrator.",
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
                "details": "If sidebar mode is enabled, `Alt+y` and `Ctrl+y` may not work until the current tab uses a Yazelix sidebar layout.",
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
}
