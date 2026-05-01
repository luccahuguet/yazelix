// Test lane: maintainer
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use yazelix_core::config_normalize::{NormalizeConfigRequest, normalize_config};
use yazelix_core::control_plane::state_dir_from_env;

const DEFAULT_SHELL: &str = "nu";
const DEFAULT_TERMINAL: &str = "ghostty";
const SUPPORTED_TERMINALS: &[&str] = &["ghostty", "wezterm", "kitty", "alacritty", "foot"];
const NONVISUAL_SHELLS: &[&str] = &["nu", "bash", "fish", "zsh"];

#[derive(Debug, Clone, Copy)]
struct SweepFeatures {
    hide_sidebar_on_file_open: bool,
    persistent_sessions: bool,
}

#[derive(Debug, Clone)]
struct SweepCombination {
    kind: &'static str,
    shell: &'static str,
    terminal: &'static str,
    features: SweepFeatures,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SweepStatus {
    Pass,
    Fail,
    Error,
    Skip,
}

impl SweepStatus {
    fn label(self) -> &'static str {
        match self {
            SweepStatus::Pass => "pass",
            SweepStatus::Fail => "fail",
            SweepStatus::Error => "error",
            SweepStatus::Skip => "skip",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            SweepStatus::Pass => "✅",
            SweepStatus::Fail => "❌",
            SweepStatus::Error => "💥",
            SweepStatus::Skip => "⏭️",
        }
    }
}

#[derive(Debug, Clone)]
struct SweepResult {
    test_id: String,
    shell: String,
    terminal: String,
    status: SweepStatus,
    message: String,
    details: Option<String>,
    config_status: Option<SweepStatus>,
    config_message: Option<String>,
    config_details: Option<String>,
    env_status: Option<SweepStatus>,
    env_message: Option<String>,
    env_details: Option<String>,
}

pub fn run_sweep_tests(
    repo_root: &Path,
    verbose: bool,
    visual: bool,
    delay_secs: u64,
) -> Result<(), String> {
    if visual {
        println!("=== Visual Configuration Sweep Testing ===");
        println!("🖥️  Each configuration will launch in a new window");
        println!(
            "⏱️  Delay between launches: {:?}",
            Duration::from_secs(delay_secs)
        );
    } else {
        println!("=== Configuration Sweep Testing ===");
    }
    println!();

    let combinations = if visual {
        visual_test_combinations()
    } else {
        nonvisual_test_combinations()
    };

    println!("Running {} sweep test combinations...", combinations.len());
    println!();

    cleanup_sweep_configs()?;

    let total = combinations.len();
    let mut results = Vec::new();
    for combo in combinations {
        let test_id = format!("{}_{}_{}", combo.kind, combo.shell, combo.terminal);
        let completed = results.len();

        if !verbose && !visual {
            println!(
                "  Starting {}/{}: {}+{}",
                completed + 1,
                total,
                combo.shell,
                combo.terminal
            );
        }

        let result = if visual {
            run_visual_sweep_test(repo_root, &combo, &test_id, delay_secs)
        } else {
            run_nonvisual_sweep_test(repo_root, &combo, &test_id, verbose)
        };
        let result = match result {
            Ok(result) => result,
            Err(error) => SweepResult {
                test_id: test_id.clone(),
                shell: combo.shell.to_string(),
                terminal: combo.terminal.to_string(),
                status: SweepStatus::Error,
                message: format!("Sweep runner error: {error}"),
                details: Some(error),
                config_status: None,
                config_message: None,
                config_details: None,
                env_status: None,
                env_message: None,
                env_details: None,
            },
        };

        if !verbose && !visual {
            println!(
                "  Progress: {}/{} - {} {}+{}",
                results.len() + 1,
                total,
                result.status.label().to_ascii_uppercase(),
                combo.shell,
                combo.terminal
            );
        }

        results.push(result);
    }

    println!();
    println!("=== Sweep Test Results ===");

    let passed = results
        .iter()
        .filter(|result| result.status == SweepStatus::Pass)
        .count();
    let failed = results
        .iter()
        .filter(|result| result.status == SweepStatus::Fail)
        .count();
    let errors = results
        .iter()
        .filter(|result| result.status == SweepStatus::Error)
        .count();
    let skipped = results
        .iter()
        .filter(|result| result.status == SweepStatus::Skip)
        .count();

    for result in &results {
        println!(
            "{} {}: {} + {}",
            result.status.icon(),
            result.test_id,
            result.shell,
            result.terminal
        );
        if visual {
            if verbose || result.status != SweepStatus::Pass {
                println!("   Message: {}", result.message);
                if let Some(details) = &result.details {
                    if !details.trim().is_empty() {
                        println!("   Details: {details}");
                    }
                }
            }
        } else if verbose || result.status != SweepStatus::Pass {
            if let (Some(status), Some(message)) = (result.config_status, &result.config_message) {
                println!("   Config: {} - {}", status.label(), message);
            }
            if let Some(details) = &result.config_details {
                if !details.trim().is_empty() {
                    println!("   Config details: {details}");
                }
            }
            if let (Some(status), Some(message)) = (result.env_status, &result.env_message) {
                println!("   Environment: {} - {}", status.label(), message);
            }
            if let Some(details) = &result.env_details {
                if !details.trim().is_empty() {
                    println!("   Environment details: {details}");
                }
            }
            if result.status != SweepStatus::Pass {
                println!();
            }
        }
    }

    println!();
    println!(
        "Summary: {} passed, {} failed, {} errors, {} skipped",
        passed, failed, errors, skipped
    );

    cleanup_sweep_configs()?;

    if failed + errors > 0 {
        println!();
        println!("❌ Some sweep tests failed");
        return Err("Sweep test failures detected".to_string());
    }

    println!();
    println!("✅ All sweep tests passed!");
    Ok(())
}

fn standard_features() -> SweepFeatures {
    SweepFeatures {
        hide_sidebar_on_file_open: false,
        persistent_sessions: false,
    }
}

fn minimal_features() -> SweepFeatures {
    SweepFeatures {
        hide_sidebar_on_file_open: true,
        persistent_sessions: false,
    }
}

fn persistent_features() -> SweepFeatures {
    SweepFeatures {
        hide_sidebar_on_file_open: false,
        persistent_sessions: true,
    }
}

fn nonvisual_test_combinations() -> Vec<SweepCombination> {
    let mut combinations = Vec::new();

    for shell in NONVISUAL_SHELLS {
        combinations.push(SweepCombination {
            kind: "cross_shell",
            shell,
            terminal: DEFAULT_TERMINAL,
            features: standard_features(),
        });
    }

    combinations.push(SweepCombination {
        kind: "minimal_config",
        shell: DEFAULT_SHELL,
        terminal: DEFAULT_TERMINAL,
        features: minimal_features(),
    });
    combinations.push(SweepCombination {
        kind: "persistent_config",
        shell: DEFAULT_SHELL,
        terminal: DEFAULT_TERMINAL,
        features: persistent_features(),
    });

    combinations
}

fn visual_test_combinations() -> Vec<SweepCombination> {
    SUPPORTED_TERMINALS
        .iter()
        .map(|terminal| SweepCombination {
            kind: "cross_terminal",
            shell: DEFAULT_SHELL,
            terminal,
            features: SweepFeatures {
                hide_sidebar_on_file_open: true,
                persistent_sessions: false,
            },
        })
        .collect()
}

fn run_nonvisual_sweep_test(
    repo_root: &Path,
    combo: &SweepCombination,
    test_id: &str,
    verbose: bool,
) -> Result<SweepResult, String> {
    if verbose {
        println!(
            "🧪 Testing: {} + {} ({test_id})",
            combo.shell, combo.terminal
        );
    }

    let config_path = generate_sweep_config(combo.shell, combo.terminal, combo.features, test_id)?;
    let result = (|| {
        let config_result = validate_generated_config(&config_path, combo, repo_root);
        if config_result.status != SweepStatus::Pass {
            return Ok(nonvisual_result(
                test_id,
                combo,
                config_result.status,
                config_result.message,
                config_result.details,
                SweepStatus::Skip,
                "Skipped due to config failure".to_string(),
                None,
                SweepStatus::Fail,
            ));
        }

        let env_result = if combo.terminal == "foot" && platform_name() != "linux" {
            StepResult::new(
                SweepStatus::Skip,
                "Foot only supported on Linux".to_string(),
                None,
            )
        } else {
            validate_environment(repo_root, &config_path)
        };

        let overall = if config_result.status == SweepStatus::Pass
            && matches!(env_result.status, SweepStatus::Pass | SweepStatus::Skip)
        {
            SweepStatus::Pass
        } else {
            SweepStatus::Fail
        };

        Ok(nonvisual_result(
            test_id,
            combo,
            config_result.status,
            config_result.message,
            config_result.details,
            env_result.status,
            env_result.message,
            env_result.details,
            overall,
        ))
    })();

    cleanup_sweep_config(&config_path)?;
    result
}

fn run_visual_sweep_test(
    repo_root: &Path,
    combo: &SweepCombination,
    test_id: &str,
    delay_secs: u64,
) -> Result<SweepResult, String> {
    println!(
        "🖥️  Launching visual test: {} + {} ({test_id})",
        combo.shell, combo.terminal
    );

    let config_path = generate_sweep_config(combo.shell, combo.terminal, combo.features, test_id)?;
    let session_name = format!("sweep_test_{test_id}");
    let result = (|| {
        let before_pids = terminal_pids(combo.terminal);
        let launch_result = launch_visual_test(repo_root, &config_path, test_id, combo.terminal);

        if launch_result.exit_code == 99 {
            println!(
                "⏭️  Skipped {} + {} - terminal not installed",
                combo.shell, combo.terminal
            );
            return Ok(visual_result(
                test_id,
                combo,
                SweepStatus::Skip,
                "Terminal not installed".to_string(),
                None,
            ));
        }

        if launch_result.exit_code != 0 {
            println!("❌ Failed to launch {} + {}", combo.shell, combo.terminal);
            return Ok(visual_result(
                test_id,
                combo,
                SweepStatus::Fail,
                "Launch failed".to_string(),
                Some(launch_error_details(
                    &launch_result.stdout,
                    &launch_result.stderr,
                )),
            ));
        }

        println!(
            "✅ Launched {} + {} successfully",
            combo.shell, combo.terminal
        );

        let demo = run_visual_verification(test_id);
        cleanup_visual_test(&session_name, combo.terminal, &before_pids, delay_secs);

        if demo.verified {
            Ok(visual_result(
                test_id,
                combo,
                SweepStatus::Pass,
                "Visual launch and verification successful".to_string(),
                None,
            ))
        } else {
            Ok(visual_result(
                test_id,
                combo,
                SweepStatus::Fail,
                format!(
                    "Launch succeeded but verification failed: {}",
                    demo.status_label
                ),
                Some(demo.output),
            ))
        }
    })();

    cleanup_sweep_config(&config_path)?;
    result
}

#[derive(Debug, Clone)]
struct StepResult {
    status: SweepStatus,
    message: String,
    details: Option<String>,
}

impl StepResult {
    fn new(status: SweepStatus, message: String, details: Option<String>) -> Self {
        Self {
            status,
            message,
            details,
        }
    }
}

fn nonvisual_result(
    test_id: &str,
    combo: &SweepCombination,
    config_status: SweepStatus,
    config_message: String,
    config_details: Option<String>,
    env_status: SweepStatus,
    env_message: String,
    env_details: Option<String>,
    status: SweepStatus,
) -> SweepResult {
    SweepResult {
        test_id: test_id.to_string(),
        shell: combo.shell.to_string(),
        terminal: combo.terminal.to_string(),
        status,
        message: status.label().to_string(),
        details: None,
        config_status: Some(config_status),
        config_message: Some(config_message),
        config_details,
        env_status: Some(env_status),
        env_message: Some(env_message),
        env_details,
    }
}

fn visual_result(
    test_id: &str,
    combo: &SweepCombination,
    status: SweepStatus,
    message: String,
    details: Option<String>,
) -> SweepResult {
    SweepResult {
        test_id: test_id.to_string(),
        shell: combo.shell.to_string(),
        terminal: combo.terminal.to_string(),
        status,
        message,
        details,
        config_status: None,
        config_message: None,
        config_details: None,
        env_status: None,
        env_message: None,
        env_details: None,
    }
}

fn generate_sweep_config(
    shell: &str,
    terminal: &str,
    features: SweepFeatures,
    test_id: &str,
) -> Result<PathBuf, String> {
    let temp_dir = sweep_temp_dir()?;
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("Failed to create {}: {error}", temp_dir.display()))?;
    let config_path = temp_dir.join(format!("yazelix_test_{test_id}.toml"));
    let config_content = build_sweep_config(shell, terminal, features, test_id);
    fs::write(&config_path, config_content)
        .map_err(|error| format!("Failed to write {}: {error}", config_path.display()))?;
    Ok(config_path)
}

fn build_sweep_config(
    shell: &str,
    terminal: &str,
    features: SweepFeatures,
    test_id: &str,
) -> String {
    let mut terminals = Vec::new();
    for candidate in [terminal, "ghostty", "wezterm", "kitty", "alacritty", "foot"] {
        if !terminals.contains(&candidate) {
            terminals.push(candidate);
        }
    }

    let terminals_rendered = terminals
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "[core]\n\
debug_mode = false\n\
skip_welcome_screen = true\n\
show_macchina_on_welcome = false\n\
welcome_style = \"static\"\n\
welcome_duration_seconds = 2.0\n\
\n\
[editor]\n\
command = \"\"\n\
hide_sidebar_on_file_open = {}\n\
\n\
[shell]\n\
default_shell = \"{}\"\n\
\n\
[terminal]\n\
terminals = [{}]\n\
config_mode = \"yazelix\"\n\
transparency = \"none\"\n\
\n\
[zellij]\n\
disable_tips = true\n\
rounded_corners = true\n\
persistent_sessions = {}\n\
session_name = \"sweep_test_{}\"\n",
        features.hide_sidebar_on_file_open,
        shell,
        terminals_rendered,
        features.persistent_sessions,
        test_id,
    )
}

fn cleanup_sweep_config(config_path: &Path) -> Result<(), String> {
    if config_path.exists() {
        fs::remove_file(config_path)
            .map_err(|error| format!("Failed to remove {}: {error}", config_path.display()))?;
    }
    Ok(())
}

fn cleanup_sweep_configs() -> Result<(), String> {
    let temp_dir = sweep_temp_dir()?;
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)
            .map_err(|error| format!("Failed to remove {}: {error}", temp_dir.display()))?;
    }
    Ok(())
}

fn sweep_temp_dir() -> Result<PathBuf, String> {
    Ok(state_dir_from_env()
        .map_err(|error| error.message().to_string())?
        .join("sweep_tests"))
}

fn validate_generated_config(
    config_path: &Path,
    combo: &SweepCombination,
    repo_root: &Path,
) -> StepResult {
    let runtime_root = runtime_root(repo_root);
    let request = NormalizeConfigRequest {
        config_path: config_path.to_path_buf(),
        default_config_path: runtime_root.join("yazelix_default.toml"),
        contract_path: runtime_root
            .join("config_metadata")
            .join("main_config_contract.toml"),
        include_missing: false,
    };

    match normalize_config(&request) {
        Ok(data) => {
            let parsed_shell = data
                .normalized_config
                .get("default_shell")
                .and_then(JsonValue::as_str)
                .unwrap_or("");
            let parsed_terminal = data
                .normalized_config
                .get("terminals")
                .and_then(JsonValue::as_array)
                .and_then(|values| values.first())
                .and_then(JsonValue::as_str)
                .unwrap_or("");

            if parsed_shell == combo.shell && parsed_terminal == combo.terminal {
                StepResult::new(
                    SweepStatus::Pass,
                    "Config parsing successful".to_string(),
                    None,
                )
            } else {
                StepResult::new(
                    SweepStatus::Fail,
                    "Config parsing mismatch".to_string(),
                    Some(format!(
                        "Expected shell={}, terminal={}; got shell={}, terminal={}",
                        combo.shell, combo.terminal, parsed_shell, parsed_terminal
                    )),
                )
            }
        }
        Err(error) => StepResult::new(
            SweepStatus::Error,
            format!("Config parsing failed: {}", error.message()),
            Some(error.message().to_string()),
        ),
    }
}

fn validate_environment(repo_root: &Path, config_path: &Path) -> StepResult {
    let runtime_root = runtime_root(repo_root);
    let yzx_cli = runtime_root.join("shells").join("posix").join("yzx_cli.sh");
    let validation_helper = runtime_root
        .join("shells")
        .join("posix")
        .join("sweep_validate_runtime_tools.sh");

    let output = Command::new("sh")
        .arg(&yzx_cli)
        .args(["run", "sh"])
        .arg(&validation_helper)
        .env("YAZELIX_CONFIG_OVERRIDE", config_path)
        .env("YAZELIX_RUNTIME_DIR", &runtime_root)
        .current_dir(repo_root)
        .output();

    let output = match output {
        Ok(output) => output,
        Err(error) => {
            return StepResult::new(
                SweepStatus::Error,
                format!("Test execution failed: {error}"),
                None,
            );
        }
    };

    if !output.status.success() {
        return StepResult::new(
            SweepStatus::Fail,
            "Environment validation failed".to_string(),
            Some(String::from_utf8_lossy(&output.stderr).trim().to_string()),
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !stdout.contains("TOOLS_START") || !stdout.contains("TOOLS_END") {
        return StepResult::new(
            SweepStatus::Fail,
            "Tool availability incomplete".to_string(),
            Some(stdout),
        );
    }
    if !stdout.contains("VERSION_START") || !stdout.contains("VERSION_END") {
        return StepResult::new(
            SweepStatus::Fail,
            "Version check incomplete".to_string(),
            Some(stdout),
        );
    }

    let stdout_lower = stdout.to_ascii_lowercase();
    if !stdout_lower.contains("zellij")
        || !stdout_lower.contains("yazi")
        || !stdout_lower.contains("helix")
    {
        return StepResult::new(
            SweepStatus::Fail,
            "Missing expected tool versions".to_string(),
            Some(stdout),
        );
    }

    StepResult::new(
        SweepStatus::Pass,
        "All environment tests passed".to_string(),
        None,
    )
}

fn platform_name() -> String {
    std::env::var("YAZELIX_TEST_OS")
        .unwrap_or_else(|_| std::env::consts::OS.to_string())
        .trim()
        .to_ascii_lowercase()
}

struct LaunchResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn launch_visual_test(
    repo_root: &Path,
    config_path: &Path,
    test_id: &str,
    terminal: &str,
) -> LaunchResult {
    if !command_exists(terminal) {
        return LaunchResult {
            exit_code: 99,
            stdout: String::new(),
            stderr: format!("Terminal not installed: {terminal}"),
        };
    }

    let runtime_root = runtime_root(repo_root);
    let yzx_cli = runtime_root.join("shells").join("posix").join("yzx_cli.sh");
    let output = Command::new("sh")
        .arg(&yzx_cli)
        .args(["launch", "--terminal", terminal])
        .env("YAZELIX_CONFIG_OVERRIDE", config_path)
        .env("YAZELIX_RUNTIME_DIR", &runtime_root)
        .env("YAZELIX_SHELLHOOK_SKIP_WELCOME", "true")
        .env("YAZELIX_LAYOUT_OVERRIDE", "yzx_sweep_test")
        .env("YAZELIX_SWEEP_TEST_ID", test_id)
        .current_dir(repo_root)
        .output();

    match output {
        Ok(output) => LaunchResult {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(error) => LaunchResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn launch_error_details(stdout: &str, stderr: &str) -> String {
    let combined = [stderr.trim(), stdout.trim()]
        .into_iter()
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if combined.is_empty() {
        return "No output captured".to_string();
    }

    let lower = combined.to_ascii_lowercase();
    let missing_terminal = lower.contains("specified terminal")
        || lower.contains("none of the supported terminals")
        || lower.contains("terminal.terminals must include");

    if !missing_terminal {
        return combined
            .lines()
            .next()
            .unwrap_or("No output captured")
            .to_string();
    }

    let matched_line = combined
        .lines()
        .find(|line| {
            let lower_line = line.to_ascii_lowercase();
            lower_line.contains("specified terminal")
                || lower_line.contains("none of the supported terminals")
                || lower_line.contains("terminal.terminals must include")
        })
        .unwrap_or("");
    if matched_line.is_empty() {
        "Missing terminal. Install one of the configured host terminals, or update [terminal].terminals and rerun yzx launch.".to_string()
    } else {
        format!(
            "Missing terminal. Install one of the configured host terminals, or update [terminal].terminals and rerun yzx launch. {matched_line}"
        )
    }
}

struct VisualVerification {
    status_label: String,
    output: String,
    verified: bool,
}

fn run_visual_verification(test_id: &str) -> VisualVerification {
    let result_file = PathBuf::from(format!("/tmp/yazelix_sweep_result_{test_id}.json"));
    println!("   Waiting for verification script in session to complete...");

    if result_file.exists() {
        let _ = fs::remove_file(&result_file);
    }

    let mut file_found = false;
    for _ in 0..20 {
        if result_file.exists() {
            file_found = true;
            break;
        }
        sleep(Duration::from_millis(500));
    }

    if !file_found {
        println!("   ✗ Verification timeout - script didn't create result file");
        return VisualVerification {
            status_label: "fail".to_string(),
            output: "Verification script timeout".to_string(),
            verified: false,
        };
    }

    sleep(Duration::from_millis(500));

    let raw = match fs::read_to_string(&result_file) {
        Ok(raw) => raw,
        Err(error) => {
            println!("   ✗ Failed to read verification file: {error}");
            let _ = fs::remove_file(&result_file);
            return VisualVerification {
                status_label: "error".to_string(),
                output: error.to_string(),
                verified: false,
            };
        }
    };

    let parsed: JsonValue = match serde_json::from_str(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            println!("   ✗ Failed to parse verification file: {error}");
            println!("   File path: {}", result_file.display());
            let _ = fs::remove_file(&result_file);
            return VisualVerification {
                status_label: "error".to_string(),
                output: format!("Parse error: {error}"),
                verified: false,
            };
        }
    };

    let zellij_ok = parsed["tools"]["zellij"]["available"]
        .as_bool()
        .unwrap_or(false);
    let yazi_ok = parsed["tools"]["yazi"]["available"]
        .as_bool()
        .unwrap_or(false);
    let helix_ok = parsed["tools"]["helix"]["available"]
        .as_bool()
        .unwrap_or(false);
    if zellij_ok && yazi_ok && helix_ok {
        println!("   ✓ Verification passed - all tools available in launched session");
        println!(
            "     - Terminal: {}",
            parsed["terminal"].as_str().unwrap_or("unknown")
        );
        println!(
            "     - Zellij: {}",
            parsed["tools"]["zellij"]["version"]
                .as_str()
                .unwrap_or("unknown")
        );
        println!(
            "     - Yazi: {}",
            parsed["tools"]["yazi"]["version"]
                .as_str()
                .unwrap_or("unknown")
        );
        println!(
            "     - Helix: {}",
            parsed["tools"]["helix"]["version"]
                .as_str()
                .unwrap_or("unknown")
        );
        let _ = fs::remove_file(&result_file);
        return VisualVerification {
            status_label: "pass".to_string(),
            output: raw,
            verified: true,
        };
    }

    println!("   ✗ Verification failed - some tools not available in session");
    let _ = fs::remove_file(&result_file);
    VisualVerification {
        status_label: "fail".to_string(),
        output: raw,
        verified: false,
    }
}

fn cleanup_visual_test(session_name: &str, terminal: &str, before_pids: &[i32], delay_secs: u64) {
    println!(
        "   Waiting {:?} before cleanup...",
        Duration::from_secs(delay_secs)
    );
    sleep(Duration::from_secs(delay_secs));
    cleanup_zellij_session(session_name);
    cleanup_terminal_processes(terminal, before_pids);
}

fn cleanup_zellij_session(session_pattern: &str) {
    let output = match Command::new("zellij").arg("list-sessions").output() {
        Ok(output) => output,
        Err(_) => {
            println!("   Session cleanup skipped");
            return;
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    let session_line = stdout
        .lines()
        .find(|line| line.contains(session_pattern))
        .unwrap_or("");
    if session_line.is_empty() {
        return;
    }

    let session_id = session_line
        .trim_start_matches('>')
        .split_whitespace()
        .next()
        .unwrap_or("");
    if session_id.is_empty() {
        return;
    }

    println!("   Cleaning up session: {session_id}");
    let _ = Command::new("zellij")
        .args(["kill-session", session_id])
        .output();
}

fn cleanup_terminal_processes(terminal: &str, before_pids: &[i32]) {
    sleep(Duration::from_secs(1));
    let after_pids = terminal_pids(terminal);
    let new_pids = after_pids
        .into_iter()
        .filter(|pid| !before_pids.contains(pid))
        .collect::<Vec<_>>();

    if new_pids.is_empty() {
        println!("   No new terminal processes detected for cleanup");
        return;
    }

    for pid in new_pids {
        println!("   Terminating terminal process: {pid}");
        let _ = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output();
        sleep(Duration::from_millis(300));
        if process_exists(pid) {
            let _ = Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .output();
        }
    }
}

fn process_exists(pid: i32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn terminal_pids(terminal: &str) -> Vec<i32> {
    let output = match Command::new("ps").args(["-eo", "pid=,comm="]).output() {
        Ok(output) => output,
        Err(_) => return Vec::new(),
    };

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let pid = parts.next()?.parse::<i32>().ok()?;
            let command = parts.next().unwrap_or("");
            command.contains(terminal).then_some(pid)
        })
        .collect()
}

fn command_exists(name: &str) -> bool {
    Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn runtime_root(repo_root: &Path) -> PathBuf {
    std::env::var_os("YAZELIX_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .unwrap_or_else(|| repo_root.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: Rust sweep config generation keeps the requested shell and terminal first while rendering the managed toggle matrix for the sweep lane.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn renders_requested_shell_terminal_and_toggle_matrix() {
        let rendered = build_sweep_config(
            "zsh",
            "kitty",
            SweepFeatures {
                hide_sidebar_on_file_open: true,
                persistent_sessions: true,
            },
            "cross_shell_zsh_kitty",
        );

        assert!(rendered.contains("default_shell = \"zsh\""));
        assert!(rendered.contains(
            "terminals = [\"kitty\", \"ghostty\", \"wezterm\", \"alacritty\", \"foot\"]"
        ));
        assert!(rendered.contains("hide_sidebar_on_file_open = true"));
        assert!(rendered.contains("persistent_sessions = true"));
    }
}
