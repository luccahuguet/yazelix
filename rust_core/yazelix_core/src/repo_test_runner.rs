use crate::repo_contract_validation::validate_nushell_syntax;
use crate::repo_plugin_build::validate_pane_orchestrator_sync;
use crate::repo_sweep_runner::run_sweep_tests;
use crate::repo_validation::validate_package_rust_test_purity;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct RepoTestOptions {
    pub verbose: bool,
    pub new_window: bool,
    pub lint_only: bool,
    pub profile: bool,
    pub sweep: bool,
    pub visual: bool,
    pub all: bool,
    pub delay: u64,
}

impl Default for RepoTestOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            new_window: false,
            lint_only: false,
            profile: false,
            sweep: false,
            visual: false,
            all: false,
            delay: 3,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TestSuiteInventory {
    default: DefaultSuiteInventory,
}

#[derive(Debug, Deserialize)]
struct DefaultSuiteInventory {
    #[serde(default)]
    nextest_suites: Vec<TestSuite>,
    #[serde(default)]
    default_cargo_test_exceptions: Vec<TestSuite>,
}

#[derive(Debug, Deserialize)]
struct TestSuite {
    name: String,
    manifest_path: String,
    #[serde(default)]
    args: Vec<String>,
}

#[derive(Debug)]
struct SuiteResult {
    status: SuiteStatus,
    suite: String,
    error: Option<String>,
    elapsed_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SuiteStatus {
    Pass,
    Fail,
}

impl SuiteStatus {
    fn label(self) -> &'static str {
        match self {
            SuiteStatus::Pass => "✅ PASS",
            SuiteStatus::Fail => "❌ FAIL",
        }
    }
}

pub fn run_repo_tests(repo_root: &Path, options: &RepoTestOptions) -> Result<(), String> {
    if options.new_window {
        return run_new_window(repo_root, options);
    }

    let run_only_sweep = options.sweep && !options.visual && !options.all;
    let run_only_visual = options.visual && !options.sweep && !options.all;
    let run_only_both_sweeps = options.sweep && options.visual && !options.all;

    if run_only_visual {
        run_visual_sweep_tests(repo_root, options.verbose, options.delay)?;
        return Ok(());
    }
    if run_only_sweep {
        run_nonvisual_sweep_tests(repo_root, options.verbose)?;
        return Ok(());
    }
    if run_only_both_sweeps {
        run_nonvisual_sweep_tests(repo_root, options.verbose)?;
        run_visual_sweep_tests(repo_root, options.verbose, options.delay)?;
        return Ok(());
    }

    let log_file = create_log_file(repo_root, options)?;
    if options.lint_only {
        let syntax_passed = run_syntax_validation(repo_root, options.verbose, &log_file)?;
        println!("📝 Full log: {}", log_file.display());
        if !syntax_passed {
            return Err("Syntax validation failed".to_string());
        }
        return Ok(());
    }

    println!("=== Yazelix Default Test Suite ===");
    println!("Running fixed Rust nextest suites...");
    println!("📝 Logging to: {}", log_file.display());
    println!();
    append_log(
        &log_file,
        "=== Yazelix Default Test Suite ===\nRunning fixed Rust nextest suites...\n\n",
    )?;

    if !run_syntax_validation(repo_root, options.verbose, &log_file)? {
        println!();
        println!("❌ Test suite aborted due to syntax errors");
        println!("   Fix syntax errors and try again");
        println!("📝 Full log: {}", log_file.display());
        return Err("Syntax validation failed".to_string());
    }

    if !run_static_maintainer_validations(repo_root, options.verbose, &log_file)? {
        println!();
        println!("❌ Test suite aborted due to static maintainer validation errors");
        println!("   Fix the reported validator errors and try again");
        println!("📝 Full log: {}", log_file.display());
        return Err("Static maintainer validation failed".to_string());
    }

    let inventory = load_test_suite_inventory(repo_root)?;
    let results = run_default_functional_suites(repo_root, &inventory, &log_file, options.verbose)?;
    render_suite_summary(&results, &log_file, profiling_enabled(options))?;

    if options.sweep || options.all {
        run_nonvisual_sweep_tests(repo_root, options.verbose)?;
    }
    if options.visual || options.all {
        run_visual_sweep_tests(repo_root, options.verbose, options.delay)?;
    }

    Ok(())
}

fn run_static_maintainer_validations(
    repo_root: &Path,
    verbose: bool,
    log_file: &Path,
) -> Result<bool, String> {
    println!();
    println!("🔒 Phase 2: Static Maintainer Validations");
    println!("─────────────────────────────────────");
    append_log(log_file, "=== Static Maintainer Validations ===\n")?;

    let mut errors = Vec::new();
    let package_test_report = validate_package_rust_test_purity(repo_root)?;
    errors.extend(package_test_report.errors);
    let pane_orchestrator_errors = validate_pane_orchestrator_sync(repo_root)?;
    errors.extend(pane_orchestrator_errors);

    if errors.is_empty() {
        println!("✅ Package-test purity and pane-orchestrator sync checks passed");
        append_log(log_file, "✅ Static maintainer validations passed\n\n")?;
        Ok(true)
    } else {
        println!("❌ Static maintainer validations failed");
        for error in &errors {
            eprintln!("{error}");
        }
        if verbose {
            println!(
                "   Validators: validate-package-rust-test-purity, validate-pane-orchestrator-sync"
            );
        }
        append_log(
            log_file,
            &format!(
                "❌ Static maintainer validations failed\n{}\n\n",
                errors.join("\n")
            ),
        )?;
        Ok(false)
    }
}

fn run_new_window(repo_root: &Path, options: &RepoTestOptions) -> Result<(), String> {
    println!("🚀 Launching new Yazelix window for testing...");
    println!();

    let mut test_args = vec!["yzx".to_string(), "dev".to_string(), "test".to_string()];
    if options.verbose {
        test_args.push("--verbose".to_string());
    }
    if options.lint_only {
        test_args.push("--lint-only".to_string());
    }
    if options.profile {
        test_args.push("--profile".to_string());
    }
    if options.sweep {
        test_args.push("--sweep".to_string());
    }
    if options.visual {
        test_args.push("--visual".to_string());
    }
    if options.all {
        test_args.push("--all".to_string());
    }
    if options.visual || options.all {
        test_args.push("--delay".to_string());
        test_args.push(options.delay.to_string());
    }
    println!("💡 In the new window, run: {}", test_args.join(" "));
    println!(
        "📝 Test logs will be saved to: {}",
        repo_root.join("logs").display()
    );
    println!();

    let status = Command::new(repo_root.join("shells").join("posix").join("yzx_cli.sh"))
        .arg("launch")
        .current_dir(repo_root)
        .env("YAZELIX_SHELLHOOK_SKIP_WELCOME", "true")
        .status()
        .map_err(|error| format!("Failed to launch Yazelix test window: {error}"))?;
    if !status.success() {
        return Err(format!(
            "Yazelix test window launch failed with exit code {}",
            status.code().unwrap_or(1)
        ));
    }
    Ok(())
}

fn create_log_file(repo_root: &Path, options: &RepoTestOptions) -> Result<PathBuf, String> {
    let log_dir = repo_root.join("logs");
    fs::create_dir_all(&log_dir)
        .map_err(|error| format!("Failed to create {}: {}", log_dir.display(), error))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("System clock error: {error}"))?
        .as_millis();
    let log_file = log_dir.join(format!("test_run_{timestamp}.log"));
    let header = format!(
        "=== Yazelix Test Run ===\nDate: {:?}\nVerbose: {}\n\n",
        SystemTime::now(),
        options.verbose
    );
    fs::write(&log_file, header)
        .map_err(|error| format!("Failed to write {}: {}", log_file.display(), error))?;
    Ok(log_file)
}

fn run_syntax_validation(repo_root: &Path, verbose: bool, log_file: &Path) -> Result<bool, String> {
    println!("🔍 Phase 1: Syntax Validation");
    println!("─────────────────────────────────────");
    append_log(log_file, "=== Syntax Validation ===\n")?;

    let report = validate_nushell_syntax(repo_root, verbose)?;
    for warning in &report.warnings {
        if verbose {
            println!("⚠️ {warning}");
        }
    }

    if report.errors.is_empty() {
        println!("✅ All scripts passed syntax validation");
        append_log(log_file, "✅ Syntax validation passed\n\n")?;
        Ok(true)
    } else {
        println!("❌ Syntax validation failed");
        for error in &report.errors {
            eprintln!("{error}");
        }
        append_log(
            log_file,
            &format!(
                "❌ Syntax validation failed\n{}\n\n",
                report.errors.join("\n")
            ),
        )?;
        Ok(false)
    }
}

fn load_test_suite_inventory(repo_root: &Path) -> Result<TestSuiteInventory, String> {
    let path = repo_root
        .join("nushell")
        .join("scripts")
        .join("maintainer")
        .join("test_suite_inventory.toml");
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    toml::from_str(&raw).map_err(|error| format!("Invalid TOML in {}: {}", path.display(), error))
}

fn run_default_functional_suites(
    repo_root: &Path,
    inventory: &TestSuiteInventory,
    log_file: &Path,
    verbose: bool,
) -> Result<Vec<SuiteResult>, String> {
    println!();
    println!("🧪 Phase 3: Functional Tests");
    println!("─────────────────────────────────────");
    append_log(log_file, "=== Functional Tests ===\n")?;

    let mut results = Vec::new();
    for suite in &inventory.default.nextest_suites {
        let mut cargo_args = vec![
            "nextest".to_string(),
            "run".to_string(),
            "--profile".to_string(),
            "ci".to_string(),
            "--manifest-path".to_string(),
            repo_root.join(&suite.manifest_path).display().to_string(),
        ];
        cargo_args.extend(suite.args.clone());
        results.push(run_logged_suite(
            &suite.name,
            &format!("Rust nextest: {}", suite.name),
            repo_root,
            "nix",
            &nix_develop_cargo_args(cargo_args),
            log_file,
            verbose,
        )?);
    }

    for suite in &inventory.default.default_cargo_test_exceptions {
        let mut cargo_args = vec![
            "test".to_string(),
            "--manifest-path".to_string(),
            repo_root.join(&suite.manifest_path).display().to_string(),
        ];
        cargo_args.extend(suite.args.clone());
        results.push(run_logged_suite(
            &suite.name,
            &format!("Rust cargo test exception: {}", suite.name),
            repo_root,
            "nix",
            &nix_develop_cargo_args(cargo_args),
            log_file,
            verbose,
        )?);
    }

    Ok(results)
}

fn nix_develop_cargo_args(cargo_args: Vec<String>) -> Vec<String> {
    ["develop", "-c", "cargo"]
        .into_iter()
        .map(ToOwned::to_owned)
        .chain(cargo_args)
        .collect()
}

fn run_logged_suite(
    suite_name: &str,
    display_name: &str,
    repo_root: &Path,
    program: &str,
    args: &[String],
    log_file: &Path,
    verbose: bool,
) -> Result<SuiteResult, String> {
    let started = Instant::now();
    if verbose {
        println!("📋 Running: {display_name}");
        println!("─────────────────────────────────────");
        println!("Running: {program} {}", args.join(" "));
    } else {
        println!("  Running {display_name}...");
    }

    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to run {display_name}: {error}"))?;

    if verbose {
        print_command_output(&output);
        println!();
    }
    append_command_output_to_log(log_file, suite_name, &output)?;

    let status = if output.status.success() {
        SuiteStatus::Pass
    } else {
        SuiteStatus::Fail
    };
    let error = (!output.status.success()).then(|| {
        format!(
            "Exit code: {}\n{}",
            output.status.code().unwrap_or(1),
            summarize_failure_output(&output)
        )
    });

    Ok(SuiteResult {
        status,
        suite: suite_name.to_string(),
        error,
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn append_command_output_to_log(
    log_file: &Path,
    suite_name: &str,
    output: &Output,
) -> Result<(), String> {
    let mut entry = format!(
        "Suite: {suite_name}\nExit code: {}\nStdout:\n{}\n",
        output.status.code().unwrap_or(1),
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        entry.push_str(&format!("Stderr:\n{stderr}\n"));
    }
    entry.push_str("---\n");
    append_log(log_file, &entry)
}

fn render_suite_summary(
    results: &[SuiteResult],
    log_file: &Path,
    profiling: bool,
) -> Result<(), String> {
    println!();
    println!("=== Test Results Summary ===");

    let passed = results
        .iter()
        .filter(|result| result.status == SuiteStatus::Pass)
        .count();
    let failed = results
        .iter()
        .filter(|result| result.status == SuiteStatus::Fail)
        .count();

    for result in results {
        println!("{} {}", result.status.label(), result.suite);
        if result.status == SuiteStatus::Fail {
            if let Some(error) = &result.error {
                println!("   Error: {error}");
            }
        }
    }

    println!();
    let summary = format!(
        "Total: {} | Passed: {passed} | Failed: {failed}",
        results.len()
    );
    println!("{summary}");

    let mut log_entry = "\n=== Test Results Summary ===\n".to_string();
    for result in results {
        log_entry.push_str(&format!("{} {}\n", result.status.label(), result.suite));
        if result.status == SuiteStatus::Fail {
            if let Some(error) = &result.error {
                log_entry.push_str(&format!("   Error: {error}\n"));
            }
        }
    }
    log_entry.push_str(&format!("\n{summary}\n"));
    append_log(log_file, &log_entry)?;

    if profiling {
        println!();
        let profile_report = render_profile_summary(results);
        println!("{profile_report}");
        append_log(log_file, &format!("{profile_report}\n"))?;
    }

    if failed > 0 {
        println!();
        println!("❌ Some tests failed");
        append_log(log_file, "\n❌ Some tests failed\n")?;
        println!("📝 Full log: {}", log_file.display());
        println!();
        return Err("Test suite failed".to_string());
    }

    println!();
    println!("✅ All tests passed!");
    append_log(log_file, "\n✅ All tests passed!\n")?;
    println!("📝 Full log: {}", log_file.display());
    println!();
    Ok(())
}

fn run_nonvisual_sweep_tests(repo_root: &Path, verbose: bool) -> Result<(), String> {
    println!();
    println!("=== Running Non-Visual Configuration Sweep Tests ===");
    println!();
    run_sweep_tests(repo_root, verbose, false, 0)
}

fn run_visual_sweep_tests(repo_root: &Path, verbose: bool, delay: u64) -> Result<(), String> {
    println!();
    println!("=== Running Visual Terminal Sweep Tests ===");
    println!();
    run_sweep_tests(repo_root, verbose, true, delay)
}

fn profiling_enabled(options: &RepoTestOptions) -> bool {
    options.profile
        || std::env::var("YAZELIX_TEST_PROFILE")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
}

fn render_profile_summary(results: &[SuiteResult]) -> String {
    let mut sorted = results.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| right.elapsed_ms.cmp(&left.elapsed_ms));
    let mut lines = vec!["=== Default Suite Profile ===".to_string()];
    for result in sorted {
        lines.push(format!(
            "  - {}: {:.2}s",
            result.suite,
            result.elapsed_ms as f64 / 1000.0
        ));
    }
    lines.join("\n")
}

fn summarize_failure_output(output: &Output) -> String {
    let stdout_tail = tail_lines(&String::from_utf8_lossy(&output.stdout), 40);
    let stderr_tail = tail_lines(&String::from_utf8_lossy(&output.stderr), 40);
    let mut sections = Vec::new();
    if !stdout_tail.trim().is_empty() {
        sections.push(format!("Stdout tail:\n{stdout_tail}"));
    }
    if !stderr_tail.trim().is_empty() {
        sections.push(format!("Stderr tail:\n{stderr_tail}"));
    }
    sections.join("\n")
}

fn tail_lines(raw: &str, count: usize) -> String {
    let lines = raw.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(count);
    lines[start..].join("\n")
}

fn print_command_output(output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        print!("{stdout}");
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        eprint!("{stderr}");
    }
}

fn append_log(log_file: &Path, text: &str) -> Result<(), String> {
    use std::io::Write;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|error| format!("Failed to open {}: {}", log_file.display(), error))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("Failed to write {}: {}", log_file.display(), error))
}
