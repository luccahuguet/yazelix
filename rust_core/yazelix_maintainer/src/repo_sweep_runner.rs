// Test lane: maintainer
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use yazelix_core::config_normalize::{NormalizeConfigRequest, normalize_config};
use yazelix_core::control_plane::state_dir_from_env;

const DEFAULT_SHELL: &str = "nu";
const DEFAULT_TERMINAL: &str = "ghostty";
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
    config_status: Option<SweepStatus>,
    config_message: Option<String>,
    config_details: Option<String>,
    env_status: Option<SweepStatus>,
    env_message: Option<String>,
    env_details: Option<String>,
}

pub fn run_sweep_tests(repo_root: &Path, verbose: bool) -> Result<(), String> {
    println!("=== Configuration Sweep Testing ===");
    println!();

    let combinations = nonvisual_test_combinations();

    println!("Running {} sweep test combinations...", combinations.len());
    println!();

    cleanup_sweep_configs()?;

    let total = combinations.len();
    let mut results = Vec::new();
    for combo in combinations {
        let test_id = format!("{}_{}_{}", combo.kind, combo.shell, combo.terminal);
        let completed = results.len();

        if !verbose {
            println!(
                "  Starting {}/{}: {}+{}",
                completed + 1,
                total,
                combo.shell,
                combo.terminal
            );
        }

        let result = run_nonvisual_sweep_test(repo_root, &combo, &test_id, verbose);
        let result = match result {
            Ok(result) => result,
            Err(error) => SweepResult {
                test_id: test_id.clone(),
                shell: combo.shell.to_string(),
                terminal: combo.terminal.to_string(),
                status: SweepStatus::Error,
                config_status: None,
                config_message: None,
                config_details: Some(format!("Sweep runner error: {error}")),
                env_status: None,
                env_message: None,
                env_details: None,
            },
        };

        if !verbose {
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
        if verbose || result.status != SweepStatus::Pass {
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
        config_status: Some(config_status),
        config_message: Some(config_message),
        config_details,
        env_status: Some(env_status),
        env_message: Some(env_message),
        env_details,
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
        default_config_path: runtime_root.join("settings_default.jsonc"),
        contract_path: runtime_root
            .join("config_metadata")
            .join("main_config_contract.toml"),
        include_missing: true,
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
