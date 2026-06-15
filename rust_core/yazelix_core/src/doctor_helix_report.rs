//! Helix-focused doctor findings (runtime conflicts, runtime health, managed integration).
//! Bead: yazelix-ulb2.4.2

use crate::helix_external::HelixExternalPair;
use crate::helix_materialization::{
    MANAGED_COMMAND_MODE_COMMAND, MANAGED_COMMAND_MODE_KEY, MANAGED_REVEAL_COMMAND, REVEAL_KEY,
    STEEL_CONFIG_MODULE, STEEL_INIT_MODULE, build_managed_helix_contract_json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const STEEL_BINARY_MARKER: &[u8] = b"HELIX_STEEL_CONFIG";

fn default_reveal_binding_expected() -> String {
    MANAGED_REVEAL_COMMAND.into()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HelixDoctorEvaluateRequest {
    pub home_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub config_dir: PathBuf,
    pub user_config_helix_runtime_dir: PathBuf,
    #[serde(default)]
    pub hx_exe_path: Option<PathBuf>,
    #[serde(default)]
    pub helix_external: Option<HelixExternalPair>,
    pub include_runtime_health: bool,
    /// When `None`, managed Helix integration checks are skipped (e.g. settings.jsonc could not be parsed).
    #[serde(default)]
    pub editor_command: Option<String>,
    pub managed_helix_user_config_path: PathBuf,
    pub native_helix_config_path: PathBuf,
    pub generated_helix_config_path: PathBuf,
    #[serde(default)]
    pub expected_managed_config: Option<Value>,
    #[serde(default)]
    pub build_managed_config_error: Option<String>,
    #[serde(default = "default_reveal_binding_expected")]
    pub reveal_binding_expected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HelixRuntimeConflictEntry {
    pub path: String,
    pub priority: i32,
    pub name: String,
    pub severity: String,
}

#[derive(Debug, Serialize)]
pub struct HelixDoctorFinding {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub fix_available: bool,
    pub fix_commands: Vec<String>,
    pub conflicts: Vec<HelixRuntimeConflictEntry>,
}

impl HelixDoctorFinding {
    fn new(status: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: status.into(),
            message: message.into(),
            details: None,
            fix_available: false,
            fix_commands: Vec::new(),
            conflicts: Vec::new(),
        }
    }

    fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    fn with_fix_commands(mut self, commands: Vec<String>) -> Self {
        self.fix_available = !commands.is_empty();
        self.fix_commands = commands;
        self
    }

    fn with_conflicts(mut self, conflicts: Vec<HelixRuntimeConflictEntry>) -> Self {
        self.conflicts = conflicts;
        self
    }
}

#[derive(Debug, Serialize)]
pub struct HelixDoctorEvaluateData {
    pub runtime_conflicts: HelixDoctorFinding,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_health: Option<HelixDoctorFinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_pair: Option<HelixDoctorFinding>,
    pub managed_integration: Vec<HelixDoctorFinding>,
}

pub fn evaluate_helix_doctor_report(
    request: &HelixDoctorEvaluateRequest,
) -> HelixDoctorEvaluateData {
    let runtime_conflicts = evaluate_runtime_conflicts(request);
    let runtime_health = if request.include_runtime_health {
        Some(evaluate_runtime_health(request))
    } else {
        None
    };
    let external_pair = evaluate_external_pair(request);
    let managed_integration = evaluate_managed_integration(request);

    HelixDoctorEvaluateData {
        runtime_conflicts,
        runtime_health,
        external_pair,
        managed_integration,
    }
}

fn is_helix_editor_command(editor: &str) -> bool {
    let t = editor.trim();
    t.is_empty() || t.ends_with("/hx") || t == "hx" || t.ends_with("/helix") || t == "helix"
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn helix_health_runtime_directories(request: &HelixDoctorEvaluateRequest) -> Vec<PathBuf> {
    let output = match &request.hx_exe_path {
        Some(p) => Command::new(p).arg("--health").output(),
        None => Command::new("hx").arg("--health").output(),
    };
    let Ok(out) = output else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("Runtime directories:") else {
            continue;
        };
        let rest = rest.trim();
        return rest
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .filter(|p| Path::new(p).exists())
            .map(PathBuf::from)
            .collect();
    }
    Vec::new()
}

fn evaluate_runtime_conflicts(request: &HelixDoctorEvaluateRequest) -> HelixDoctorFinding {
    let user_runtime = &request.user_config_helix_runtime_dir;
    let mut conflicts: Vec<HelixRuntimeConflictEntry> = Vec::new();
    let mut has_high_priority_conflict = false;

    if user_runtime.exists() {
        conflicts.push(HelixRuntimeConflictEntry {
            path: path_to_string(user_runtime),
            priority: 2,
            name: "User config runtime".into(),
            severity: "error".into(),
        });
        has_high_priority_conflict = true;
    }

    let all_runtimes = helix_health_runtime_directories(request);
    let effective_runtime = all_runtimes.first().cloned();

    if let Some(ref hx_path) = request.hx_exe_path {
        if hx_path.exists() {
            let exe_runtime = hx_path
                .parent()
                .map(|p| p.join("runtime"))
                .unwrap_or_else(|| PathBuf::from("runtime"));
            if exe_runtime.exists() {
                if effective_runtime.as_ref() != Some(&exe_runtime) {
                    conflicts.push(HelixRuntimeConflictEntry {
                        path: path_to_string(&exe_runtime),
                        priority: 5,
                        name: "Executable sibling runtime".into(),
                        severity: "warning".into(),
                    });
                }
            }
        }
    }

    if conflicts.is_empty() {
        return HelixDoctorFinding::new("ok", "No conflicting Helix runtime directories found")
            .with_details("Helix runtime search order will behave as intended");
    }

    let status = if has_high_priority_conflict {
        "error"
    } else {
        "warning"
    };
    let conflict_details = conflicts
        .iter()
        .map(|c| format!("{}: {} (priority {})", c.name, c.path, c.priority))
        .collect::<Vec<_>>()
        .join(", ");

    let message = if has_high_priority_conflict {
        "HIGH PRIORITY: ~/.config/helix/runtime will override the intended Helix runtime"
    } else {
        "Lower priority runtime directories found"
    };

    let fix_commands = if has_high_priority_conflict {
        let ur = path_to_string(user_runtime);
        vec![
            "# Backup and remove conflicting runtime:".into(),
            format!("mv {ur} {ur}.backup"),
            "# Or if you want to delete it:".into(),
            format!("rm -rf {ur}"),
        ]
    } else {
        vec![]
    };

    HelixDoctorFinding::new(status, message)
        .with_details(format!(
            "Conflicting runtimes: {conflict_details}. Helix searches in priority order and will use files from higher priority directories, potentially breaking syntax highlighting."
        ))
        .with_fix_commands(fix_commands)
        .with_conflicts(conflicts)
}

fn evaluate_runtime_health(request: &HelixDoctorEvaluateRequest) -> HelixDoctorFinding {
    let all_runtimes = helix_health_runtime_directories(request);
    let primary_runtime = all_runtimes.first().cloned();

    let Some(primary) = primary_runtime else {
        return HelixDoctorFinding::new("error", "Helix runtime could not be resolved")
            .with_details("Helix did not report any valid runtime directory in `hx --health`");
    };

    let required_dirs = ["grammars", "queries", "themes"];
    let missing_dirs: Vec<&str> = required_dirs
        .iter()
        .copied()
        .filter(|&required_dir| {
            !all_runtimes
                .iter()
                .any(|runtime_path| runtime_path.join(required_dir).exists())
        })
        .collect();

    if !missing_dirs.is_empty() {
        return HelixDoctorFinding::new(
            "error",
            format!("Missing required directories: {}", missing_dirs.join(", ")),
        )
        .with_details(format!(
                "The effective Helix runtime at {} is incomplete (note: Nix may split runtime across multiple paths)",
                primary.display()
            ));
    }

    let grammar_count: usize = all_runtimes
        .iter()
        .map(|runtime_path| {
            let g = runtime_path.join("grammars");
            fs::read_dir(&g).map(|d| d.count()).unwrap_or(0)
        })
        .sum();

    if grammar_count < 200 {
        return HelixDoctorFinding::new(
            "warning",
            format!("Only {grammar_count} grammar files found (expected 200+)"),
        )
        .with_details("Some languages may not have syntax highlighting");
    }

    let tutor_exists = all_runtimes
        .iter()
        .any(|runtime_path| runtime_path.join("tutor").exists());

    if !tutor_exists {
        return HelixDoctorFinding::new("warning", "Helix tutor file missing")
            .with_details("Tutorial will not be available");
    }

    HelixDoctorFinding::new(
        "ok",
        format!("Helix runtime healthy with {grammar_count} grammars"),
    )
    .with_details(format!("Primary runtime directory: {}", primary.display()))
}

fn evaluate_external_pair(request: &HelixDoctorEvaluateRequest) -> Option<HelixDoctorFinding> {
    let external = request.helix_external.as_ref()?;
    let binary = Path::new(&external.binary);
    let runtime = Path::new(&external.runtime_path);
    let mut problems = Vec::new();

    if !binary.exists() {
        problems.push(format!("binary does not exist: {}", binary.display()));
    } else if !is_executable_file(binary) {
        problems.push(format!("binary is not executable: {}", binary.display()));
    }

    if !runtime.exists() {
        problems.push(format!(
            "runtime_path does not exist: {}",
            runtime.display()
        ));
    } else if !runtime.is_dir() {
        problems.push(format!(
            "runtime_path is not a directory: {}",
            runtime.display()
        ));
    }

    if !problems.is_empty() {
        return Some(
            HelixDoctorFinding::new("error", "External Helix binary/runtime pair is invalid")
                .with_details(format!(
                "Binary: {}\nRuntime: {}\nProblems:\n- {}\nNext: set helix.external to a matching Helix binary and runtime_path, or null to use the bundled Yazelix Helix.",
                binary.display(),
                runtime.display(),
                problems.join("\n- ")
            )),
        );
    }

    Some(
        HelixDoctorFinding::new("warning", "External Helix binary/runtime pair is user-owned")
            .with_details(format!(
            "Binary: {}\nRuntime: {}\nYazelix will launch this binary with HELIX_RUNTIME set to this runtime path, but cannot prove both came from the same Helix revision. Binary/runtime mismatches are user-owned risk; run `{} --health` after changing either path.",
            binary.display(),
            runtime.display(),
            binary.display()
        )),
    )
}

fn normal_binding_from_json(config: &Value, key: &str) -> Option<String> {
    config
        .get("keys")?
        .get("normal")?
        .get(key)?
        .as_str()
        .map(str::to_owned)
}

fn read_normal_bindings_from_toml_file(path: &Path) -> Result<BTreeMap<String, String>, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("failed to read generated config: {error}"))?;
    let v: toml::Value =
        toml::from_str(&raw).map_err(|error| format!("failed to parse TOML: {error}"))?;

    let Some(normal) = v
        .get("keys")
        .and_then(|keys| keys.get("normal"))
        .and_then(|normal| normal.as_table())
    else {
        return Ok(BTreeMap::new());
    };

    Ok(normal
        .iter()
        .filter_map(|(key, binding)| {
            binding
                .as_str()
                .map(|binding| (key.to_string(), binding.to_string()))
        })
        .collect())
}

fn stale_generated_config_finding(path: &Path) -> HelixDoctorFinding {
    HelixDoctorFinding::new(
        "warning",
        "Managed Helix generated config is stale or invalid",
    )
    .with_details(format!(
            "Generated config: {}\nExpected `A-r` to run `yzx reveal` and `:` to enter Helix command mode.\nLaunch a managed Helix session again to regenerate it.",
            path.display()
        ))
}

fn unreadable_generated_config_finding(path: &Path, error: &str) -> HelixDoctorFinding {
    HelixDoctorFinding::new(
        "warning",
        "Managed Helix generated config could not be read",
    )
    .with_details(format!(
        "Generated config: {}\nUnderlying error: {}",
        path.display(),
        error
    ))
}

fn provided_steel_symbols(module: &str) -> Vec<String> {
    module
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("(provide ")
                .and_then(|rest| rest.strip_suffix(')'))
        })
        .flat_map(|symbols| symbols.split_whitespace().map(str::to_string))
        .collect()
}

fn steel_command_metadata_lines(module: &str) -> Vec<String> {
    module
        .lines()
        .filter_map(|line| line.trim().strip_prefix(";; - ").map(str::to_string))
        .collect()
}

fn file_contains_bytes(path: &Path, needle: &[u8]) -> io::Result<bool> {
    let mut reader = io::BufReader::new(fs::File::open(path)?);
    let mut buffer = [0; 8192];
    let mut carry = Vec::new();

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(false);
        }

        let mut window = Vec::with_capacity(carry.len() + read);
        window.extend_from_slice(&carry);
        window.extend_from_slice(&buffer[..read]);
        if window
            .windows(needle.len())
            .any(|candidate| candidate == needle)
        {
            return Ok(true);
        }

        let keep = needle.len().saturating_sub(1).min(window.len());
        carry.clear();
        carry.extend_from_slice(&window[window.len() - keep..]);
    }
}

fn generated_steel_dir(request: &HelixDoctorEvaluateRequest) -> Option<PathBuf> {
    request
        .generated_helix_config_path
        .parent()
        .map(Path::to_path_buf)
}

fn evaluate_managed_steel_surface(request: &HelixDoctorEvaluateRequest) -> HelixDoctorFinding {
    let Some(steel_dir) = generated_steel_dir(request) else {
        return HelixDoctorFinding::new(
            "warning",
            "Managed Helix Steel config path could not be resolved",
        )
        .with_details(format!(
            "Generated Helix config path has no parent directory: {}",
            request.generated_helix_config_path.display()
        ));
    };

    let raw_hx = request.runtime_dir.join("libexec").join("hx");
    let helix_module_path = steel_dir.join(STEEL_CONFIG_MODULE);
    let init_module_path = steel_dir.join(STEEL_INIT_MODULE);

    if !helix_module_path.exists() || !init_module_path.exists() {
        let mut missing = Vec::new();
        if !helix_module_path.exists() {
            missing.push(helix_module_path.display().to_string());
        }
        if !init_module_path.exists() {
            missing.push(init_module_path.display().to_string());
        }
        return HelixDoctorFinding::new("warning", "Managed Helix Steel entrypoints are missing")
            .with_details(format!(
                "Missing files:\n- {}\nLaunch a managed Helix session again to regenerate the Steel config surface.",
                missing.join("\n- ")
            ));
    }

    let helix_module = match fs::read_to_string(&helix_module_path) {
        Ok(raw) => raw,
        Err(error) => {
            return HelixDoctorFinding::new(
                "warning",
                "Managed Helix Steel command module could not be read",
            )
            .with_details(format!(
                "Steel module: {}\nUnderlying error: {}",
                helix_module_path.display(),
                error
            ));
        }
    };
    let init_module = match fs::read_to_string(&init_module_path) {
        Ok(raw) => raw,
        Err(error) => {
            return HelixDoctorFinding::new(
                "warning",
                "Managed Helix Steel init module could not be read",
            )
            .with_details(format!(
                "Steel init: {}\nUnderlying error: {}",
                init_module_path.display(),
                error
            ));
        }
    };

    let provided = provided_steel_symbols(&helix_module);
    let metadata = steel_command_metadata_lines(&helix_module);
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if !raw_hx.exists() {
        errors.push(format!(
            "bundled raw Helix binary is missing: {}",
            raw_hx.display()
        ));
    } else if !is_executable_file(&raw_hx) {
        errors.push(format!(
            "bundled raw Helix binary is not executable: {}",
            raw_hx.display()
        ));
    } else {
        match file_contains_bytes(&raw_hx, STEEL_BINARY_MARKER) {
            Ok(true) => {}
            Ok(false) => errors.push(format!(
                "bundled raw Helix binary is not built with Steel support: {}; update or rebuild the Yazelix runtime",
                raw_hx.display()
            )),
            Err(error) => errors.push(format!(
                "bundled raw Helix binary could not be inspected for Steel support: {}; underlying error: {}",
                raw_hx.display(),
                error
            )),
        }
    }

    for required in ["eval-buffer", "evalp", "yzx-new-shell"] {
        if !provided.iter().any(|name| name == required) {
            errors.push(format!("public Steel command is missing: {required}"));
        }
    }

    if helix_module.contains("cogs/recentf.scm")
        && !provided.iter().any(|name| name == "recentf-open-files")
    {
        errors.push("recentf is loaded but recentf-open-files is not public".into());
    }

    for internal in [
        "recentf-snapshot",
        "show-splash",
        "refresh-files",
        "flush-recent-files",
        "get-recent-files",
        "set-recent-file-location!",
    ] {
        if provided.iter().any(|name| name == internal) {
            warnings.push(format!(
                "internal Steel command leaked publicly: {internal}"
            ));
        }
    }

    if provided.iter().any(|name| name.starts_with("yazelix.")) {
        warnings.push("module-prefixed yazelix.* Steel commands leaked publicly".into());
    }

    if init_module.contains("prefix-in")
        || init_module.contains("yazelix.")
        || init_module.contains("show-splash")
    {
        warnings.push(
            "init.scm contains command-surface bindings that should stay in helix.scm".into(),
        );
    }

    if !helix_module.contains("yzx-new-shell-command")
        || !helix_module.contains("yzx_control\\\" zellij open-terminal")
    {
        errors.push("yzx-new-shell is not wired to the Yazelix terminal opener".into());
    }

    if errors.is_empty() && warnings.is_empty() {
        return HelixDoctorFinding::new("ok", "Managed Helix Steel command surface is healthy")
            .with_details(format!(
                "Steel module: {}\nSteel init: {}\nPublic commands: {}\nCommand metadata:\n- {}",
                helix_module_path.display(),
                init_module_path.display(),
                provided.join(", "),
                metadata.join("\n- ")
            ));
    }

    let status = if errors.is_empty() {
        "warning"
    } else {
        "error"
    };
    let mut details = Vec::new();
    if !errors.is_empty() {
        details.push(format!("Errors:\n- {}", errors.join("\n- ")));
    }
    if !warnings.is_empty() {
        details.push(format!("Warnings:\n- {}", warnings.join("\n- ")));
    }
    details.push(format!(
        "Steel module: {}\nSteel init: {}\nPublic commands: {}\nCommand metadata:\n- {}",
        helix_module_path.display(),
        init_module_path.display(),
        provided.join(", "),
        metadata.join("\n- ")
    ));

    HelixDoctorFinding::new(status, "Managed Helix Steel command surface is unhealthy")
        .with_details(details.join("\n\n"))
}

fn evaluate_managed_integration(request: &HelixDoctorEvaluateRequest) -> Vec<HelixDoctorFinding> {
    let Some(editor) = request.editor_command.as_deref() else {
        return Vec::new();
    };
    if !is_helix_editor_command(editor) {
        return Vec::new();
    }

    let mut out: Vec<HelixDoctorFinding> = Vec::new();

    let managed = &request.managed_helix_user_config_path;
    let native = &request.native_helix_config_path;
    let expected_reveal_binding = request.reveal_binding_expected.as_str();

    if !managed.exists() && native.exists() {
        out.push(
            HelixDoctorFinding::new(
                "info",
                "Personal Helix config has not been imported into Yazelix-managed Helix",
            )
            .with_details(format!(
                "Native config: {}\nManaged config: {}\nRun `yzx import helix` if you want Yazelix-managed Helix sessions to reuse that personal config.",
                native.display(),
                managed.display()
            )),
        );
    }

    if let Some(ref err) = request.build_managed_config_error {
        if !err.trim().is_empty() {
            out.push(
                HelixDoctorFinding::new(
                    "error",
                    "Managed Helix config contract could not be built",
                )
                .with_details(err.clone()),
            );
            return out;
        }
    }

    let expected = if let Some(ref expected) = request.expected_managed_config {
        expected.clone()
    } else {
        match build_managed_helix_contract_json(&request.runtime_dir, &request.config_dir) {
            Ok(expected) => expected,
            Err(error) => {
                out.push(
                    HelixDoctorFinding::new(
                        "error",
                        "Managed Helix config contract could not be built",
                    )
                    .with_details(format!(
                        "{}\nNext: {}",
                        error.message(),
                        error.remediation()
                    )),
                );
                return out;
            }
        }
    };

    if normal_binding_from_json(&expected, REVEAL_KEY).as_deref()
        != Some(expected_reveal_binding.trim())
    {
        out.push(
            HelixDoctorFinding::new(
                "error",
                "Managed Helix config contract lost the Yazelix reveal binding",
            )
            .with_details(
                "The expected managed Helix config no longer enforces `A-r = :sh yzx reveal \"%{buffer_name}\"`."
            ),
        );
        return out;
    }

    if normal_binding_from_json(&expected, MANAGED_COMMAND_MODE_KEY).as_deref()
        != Some(MANAGED_COMMAND_MODE_COMMAND)
    {
        out.push(
            HelixDoctorFinding::new(
                "error",
                "Managed Helix config contract lost the command-mode binding",
            )
            .with_details(
                "The expected managed Helix config no longer enforces `: = command_mode`, which the Yazi-to-Helix opener requires."
            ),
        );
        return out;
    }

    let generated = &request.generated_helix_config_path;
    if !generated.exists() {
        out.push(
            HelixDoctorFinding::new(
                "info",
                "Managed Helix config has not been materialized yet",
            )
            .with_details(format!(
                "Expected generated config: {}\nThis is normal before the first managed Helix launch. Yazelix will generate it on demand.",
                generated.display()
            )),
        );
        return out;
    }

    let generated_bindings = match read_normal_bindings_from_toml_file(generated) {
        Ok(bindings) => bindings,
        Err(error) => {
            out.push(unreadable_generated_config_finding(generated, &error));
            return out;
        }
    };

    if generated_bindings
        .get(REVEAL_KEY)
        .map(|binding| binding.trim())
        != Some(expected_reveal_binding.trim())
        || generated_bindings
            .get(MANAGED_COMMAND_MODE_KEY)
            .map(|binding| binding.trim())
            != Some(MANAGED_COMMAND_MODE_COMMAND)
    {
        out.push(stale_generated_config_finding(generated));
        return out;
    }

    out.push(
        HelixDoctorFinding::new("ok", "Managed Helix reveal integration is healthy")
            .with_details(generated.display().to_string()),
    );
    out.push(evaluate_managed_steel_surface(request));

    out
}

// Test lane: default

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    fn write_executable(path: &Path, body: &str) {
        fs::write(path, body).unwrap();
        let mut perms = fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).unwrap();
    }

    // Defends: Helix doctor flags runtime conflicts when a user config runtime shadows the managed runtime.
    #[test]
    fn runtime_conflicts_flag_user_config_runtime() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let ur = home.join(".config/helix/runtime");
        fs::create_dir_all(&ur).unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: home.clone(),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: ur.clone(),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: Some("nvim".into()),
            managed_helix_user_config_path: home.join("m.toml"),
            native_helix_config_path: home.join("n.toml"),
            generated_helix_config_path: home.join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };
        let f = evaluate_runtime_conflicts(&req);
        assert_eq!(f.status, "error");
        assert!(!f.conflicts.is_empty());
    }

    // Defends: Helix runtime health reports `ok` when `hx --health` exposes a complete runtime tree.
    #[test]
    fn runtime_health_ok_from_fake_hx_health_output() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let rt = tmp.path().join("helix-runtime");
        fs::create_dir_all(rt.join("grammars")).unwrap();
        fs::create_dir_all(rt.join("queries")).unwrap();
        fs::create_dir_all(rt.join("themes")).unwrap();
        fs::create_dir_all(rt.join("tutor")).unwrap();
        for i in 0..250 {
            let _ = fs::File::create(rt.join("grammars").join(format!("g{i}.so")));
        }

        let fake_hx = tmp.path().join("hx");
        let health_line = format!("Runtime directories: {}", rt.to_string_lossy());
        write_executable(
            &fake_hx,
            &format!("#!/bin/sh\nprintf '%s\\n' '{health_line}'\n"),
        );

        let req = HelixDoctorEvaluateRequest {
            home_dir: home,
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("norun"),
            hx_exe_path: Some(fake_hx),
            helix_external: None,
            include_runtime_health: true,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let h = evaluate_runtime_health(&req);
        assert_eq!(h.status, "ok");
    }

    // Defends: doctor rejects external Helix pairs whose binary or runtime path cannot be used.
    #[test]
    fn external_pair_reports_missing_paths_as_error() {
        let tmp = TempDir::new().unwrap();
        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: Some(HelixExternalPair {
                binary: tmp.path().join("missing-hx").display().to_string(),
                runtime_path: tmp.path().join("missing-runtime").display().to_string(),
            }),
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let finding = evaluate_external_pair(&req).unwrap();
        assert_eq!(finding.status, "error");
        assert!(
            finding
                .details
                .as_deref()
                .unwrap()
                .contains("runtime_path does not exist")
        );
    }

    // Defends: complete external Helix pairs are reported with the user-owned binary/runtime mismatch warning.
    #[test]
    fn external_pair_reports_user_owned_mismatch_risk() {
        let tmp = TempDir::new().unwrap();
        let fake_hx = tmp.path().join("hx");
        let runtime = tmp.path().join("runtime");
        write_executable(&fake_hx, "#!/bin/sh\nexit 0\n");
        fs::create_dir_all(&runtime).unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime-root"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: Some(fake_hx.clone()),
            helix_external: Some(HelixExternalPair {
                binary: fake_hx.display().to_string(),
                runtime_path: runtime.display().to_string(),
            }),
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let data = evaluate_helix_doctor_report(&req);
        let finding = data.external_pair.unwrap();
        assert_eq!(finding.status, "warning");
        assert!(
            finding
                .details
                .as_deref()
                .unwrap()
                .contains("Binary/runtime mismatches are user-owned risk")
        );
    }

    // Defends: managed Helix integration skips non-Helix editor commands instead of fabricating findings.
    #[test]
    fn managed_integration_skips_non_helix_editor() {
        let tmp = TempDir::new().unwrap();
        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: Some("nvim".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };
        assert!(evaluate_managed_integration(&req).is_empty());
    }

    // Defends: managed Helix integration skips checks when no editor command is configured.
    #[test]
    fn managed_integration_skips_when_editor_command_absent() {
        let tmp = TempDir::new().unwrap();
        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: None,
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };
        assert!(evaluate_managed_integration(&req).is_empty());
    }

    // Defends: runtime conflict detection still warns on an executable sibling runtime even without `hx --health` output.
    #[test]
    fn runtime_conflicts_warn_on_sibling_runtime_even_without_health_output() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join("home");
        let hx_dir = tmp.path().join("bin");
        let sibling_runtime = hx_dir.join("runtime");
        fs::create_dir_all(&sibling_runtime).unwrap();

        let fake_hx = hx_dir.join("hx");
        write_executable(&fake_hx, "#!/bin/sh\nexit 1\n");

        let req = HelixDoctorEvaluateRequest {
            home_dir: home,
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: Some(fake_hx),
            helix_external: None,
            include_runtime_health: false,
            editor_command: None,
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: tmp.path().join("g.toml"),
            expected_managed_config: None,
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let finding = evaluate_runtime_conflicts(&req);
        assert_eq!(finding.status, "warning");
        assert_eq!(finding.conflicts.len(), 1);
        assert_eq!(finding.conflicts[0].name, "Executable sibling runtime");
    }

    // Defends: missing managed Helix reveal bindings are classified as stale generated config.
    #[test]
    fn managed_integration_treats_missing_generated_binding_as_stale() {
        let tmp = TempDir::new().unwrap();
        let generated = tmp.path().join("generated.toml");
        fs::write(&generated, "[keys.normal]\nB-r = \":noop\"\n").unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: generated,
            expected_managed_config: Some(serde_json::json!({
                "keys": {
                    "normal": {
                        ":": "command_mode",
                        "A-r": ":sh yzx reveal \"%{buffer_name}\""
                    }
                }
            })),
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let findings = evaluate_managed_integration(&req);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].message,
            "Managed Helix generated config is stale or invalid"
        );
    }

    // Regression: stale generated Helix configs must report the missing command-mode binding before Yazi open can type commands into the buffer.
    #[test]
    fn managed_integration_treats_missing_command_mode_binding_as_stale() {
        let tmp = TempDir::new().unwrap();
        let generated = tmp.path().join("generated.toml");
        fs::write(
            &generated,
            "[keys.normal]\n\":\" = \"no_op\"\nA-r = \":sh yzx reveal \\\"%{buffer_name}\\\"\"\n",
        )
        .unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir: tmp.path().join("runtime"),
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: generated,
            expected_managed_config: Some(serde_json::json!({
                "keys": {
                    "normal": {
                        ":": "command_mode",
                        "A-r": ":sh yzx reveal \"%{buffer_name}\""
                    }
                }
            })),
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let findings = evaluate_managed_integration(&req);
        assert_eq!(findings.len(), 1);
        assert_eq!(
            findings[0].message,
            "Managed Helix generated config is stale or invalid"
        );
        assert!(
            findings[0]
                .details
                .as_deref()
                .unwrap()
                .contains("`:` to enter Helix command mode")
        );
    }

    // Defends: doctor verifies the generated Steel public command surface, including the Yazelix shell action, after managed Helix materialization.
    #[test]
    fn managed_integration_reports_healthy_steel_surface() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let generated_dir = tmp.path().join("state/configs/helix");
        let generated = generated_dir.join("config.toml");
        fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
        fs::create_dir_all(&generated_dir).unwrap();
        write_executable(
            &runtime_dir.join("libexec/hx"),
            "#!/bin/sh\n# HELIX_STEEL_CONFIG\nexit 0\n",
        );
        fs::write(
            &generated,
            "[keys.normal]\n\":\" = \"command_mode\"\nA-r = ':sh yzx reveal \"%{buffer_name}\"'\n",
        )
        .unwrap();
        fs::write(
            generated_dir.join("helix.scm"),
            r#"(provide eval-buffer evalp yzx-new-shell recentf-open-files)
(require (only-in "cogs/recentf.scm" recentf-open-files recentf-snapshot))
(define (yzx-new-shell-command target)
  (string-append "\"$YAZELIX_RUNTIME_DIR/libexec/yzx_control\" zellij open-terminal '" target "'"))
"#,
        )
        .unwrap();
        fs::write(generated_dir.join("init.scm"), ";; generated\n").unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir,
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: generated,
            expected_managed_config: Some(serde_json::json!({
                "keys": {
                    "normal": {
                        ":": "command_mode",
                        "A-r": ":sh yzx reveal \"%{buffer_name}\""
                    }
                }
            })),
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let findings = evaluate_managed_integration(&req);
        assert_eq!(findings.len(), 2);
        assert_eq!(
            findings[0].message,
            "Managed Helix reveal integration is healthy"
        );
        assert_eq!(
            findings[1].message,
            "Managed Helix Steel command surface is healthy"
        );
    }

    // Regression: generated Steel config is not enough; the managed Helix binary must be built with Steel support too.
    #[test]
    fn managed_integration_flags_non_steel_managed_hx() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let generated_dir = tmp.path().join("state/configs/helix");
        let generated = generated_dir.join("config.toml");
        fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
        fs::create_dir_all(&generated_dir).unwrap();
        write_executable(&runtime_dir.join("libexec/hx"), "#!/bin/sh\nexit 0\n");
        fs::write(
            &generated,
            "[keys.normal]\n\":\" = \"command_mode\"\nA-r = ':sh yzx reveal \"%{buffer_name}\"'\n",
        )
        .unwrap();
        fs::write(
            generated_dir.join("helix.scm"),
            r#"(provide eval-buffer evalp yzx-new-shell recentf-open-files)
(require (only-in "cogs/recentf.scm" recentf-open-files recentf-snapshot))
(define (yzx-new-shell-command target)
  (string-append "\"$YAZELIX_RUNTIME_DIR/libexec/yzx_control\" zellij open-terminal '" target "'"))
"#,
        )
        .unwrap();
        fs::write(generated_dir.join("init.scm"), ";; generated\n").unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir,
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: generated,
            expected_managed_config: Some(serde_json::json!({
                "keys": {
                    "normal": {
                        ":": "command_mode",
                        "A-r": ":sh yzx reveal \"%{buffer_name}\""
                    }
                }
            })),
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let findings = evaluate_managed_integration(&req);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[1].status, "error");
        let details = findings[1].details.as_deref().unwrap();
        assert!(details.contains("not built with Steel support"));
        assert!(details.contains("update or rebuild the Yazelix runtime"));
    }

    // Regression: internal Steel plugin helpers should not leak back into Helix command completion.
    #[test]
    fn managed_integration_flags_leaky_steel_surface() {
        let tmp = TempDir::new().unwrap();
        let runtime_dir = tmp.path().join("runtime");
        let generated_dir = tmp.path().join("state/configs/helix");
        let generated = generated_dir.join("config.toml");
        fs::create_dir_all(runtime_dir.join("libexec")).unwrap();
        fs::create_dir_all(&generated_dir).unwrap();
        write_executable(
            &runtime_dir.join("libexec/hx"),
            "#!/bin/sh\n# HELIX_STEEL_CONFIG\nexit 0\n",
        );
        fs::write(
            &generated,
            "[keys.normal]\n\":\" = \"command_mode\"\nA-r = ':sh yzx reveal \"%{buffer_name}\"'\n",
        )
        .unwrap();
        fs::write(
            generated_dir.join("helix.scm"),
            "(provide eval-buffer evalp show-splash)\n",
        )
        .unwrap();
        fs::write(generated_dir.join("init.scm"), ";; generated\n").unwrap();

        let req = HelixDoctorEvaluateRequest {
            home_dir: tmp.path().join("home"),
            runtime_dir,
            config_dir: tmp.path().join("config"),
            user_config_helix_runtime_dir: tmp.path().join("ur"),
            hx_exe_path: None,
            helix_external: None,
            include_runtime_health: false,
            editor_command: Some("hx".into()),
            managed_helix_user_config_path: tmp.path().join("m.toml"),
            native_helix_config_path: tmp.path().join("n.toml"),
            generated_helix_config_path: generated,
            expected_managed_config: Some(serde_json::json!({
                "keys": {
                    "normal": {
                        ":": "command_mode",
                        "A-r": ":sh yzx reveal \"%{buffer_name}\""
                    }
                }
            })),
            build_managed_config_error: None,
            reveal_binding_expected: ":sh yzx reveal \"%{buffer_name}\"".into(),
        };

        let findings = evaluate_managed_integration(&req);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[1].status, "error");
        let details = findings[1].details.as_deref().unwrap();
        assert!(details.contains("public Steel command is missing: yzx-new-shell"));
        assert!(details.contains("internal Steel command leaked publicly: show-splash"));
    }
}
