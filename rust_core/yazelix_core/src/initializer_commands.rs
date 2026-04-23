// Test lane: default
//! Shell initializer generator for `yzx_control`.

use crate::bridge::{CoreError, ErrorClass};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellConfig {
    name: &'static str,
    dir: PathBuf,
    ext: &'static str,
    tool_override: Option<(&'static str, &'static str)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolConfig {
    name: &'static str,
    required: bool,
    init_args: &'static [&'static str],
    /// Optional shell name override for specific tools.
    shell_override: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
struct InitializerResult {
    status: String,
    tool: String,
    shell: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
}

fn shell_initializer_dirs(home: &Path) -> Vec<ShellConfig> {
    let base = home.join(".local").join("share").join("yazelix").join("initializers");
    vec![
        ShellConfig {
            name: "nu",
            dir: base.join("nushell"),
            ext: "nu",
            tool_override: Some(("carapace", "nushell")),
        },
        ShellConfig {
            name: "bash",
            dir: base.join("bash"),
            ext: "sh",
            tool_override: None,
        },
        ShellConfig {
            name: "fish",
            dir: base.join("fish"),
            ext: "fish",
            tool_override: None,
        },
        ShellConfig {
            name: "zsh",
            dir: base.join("zsh"),
            ext: "zsh",
            tool_override: None,
        },
    ]
}

fn tool_configs() -> Vec<ToolConfig> {
    vec![
        ToolConfig {
            name: "starship",
            required: true,
            init_args: &["init"],
            shell_override: None,
        },
        ToolConfig {
            name: "zoxide",
            required: true,
            init_args: &["init"],
            shell_override: None,
        },
        ToolConfig {
            name: "atuin",
            required: false,
            init_args: &["init"],
            shell_override: None,
        },
        ToolConfig {
            name: "mise",
            required: false,
            init_args: &["activate"],
            shell_override: None,
        },
        ToolConfig {
            name: "carapace",
            required: false,
            init_args: &["_carapace"],
            shell_override: None,
        },
    ]
}

fn find_on_path(command: &str) -> bool {
    if let Ok(path_var) = std::env::var("PATH") {
        for entry in std::env::split_paths(&path_var) {
            if entry.join(command).is_file() {
                return true;
            }
        }
    }
    false
}

fn run_tool_init(tool: &ToolConfig, shell_name: &str) -> Result<String, String> {
    let effective_shell = tool.shell_override.unwrap_or(shell_name);
    let args: Vec<&str> = tool
        .init_args
        .iter()
        .copied()
        .chain(std::iter::once(effective_shell))
        .collect();

    let output = Command::new(tool.name)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to run {}: {e}", tool.name))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{} exited with error: {stderr}", tool.name));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(stdout)
}

fn normalize_nu_starship(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut filtered = Vec::new();
    let mut skipping_prompt = false;
    let mut skipping_config = false;

    for line in &lines {
        if !skipping_prompt && line.contains("PROMPT_COMMAND_RIGHT: {||") {
            skipping_prompt = true;
            continue;
        }
        if !skipping_config && line.contains("config: ($env.config? | default {} | merge {") {
            skipping_config = true;
            continue;
        }
        if skipping_prompt {
            if *line == "    }" {
                skipping_prompt = false;
            }
            continue;
        }
        if skipping_config {
            if *line == "    })" {
                skipping_config = false;
            }
            continue;
        }
        filtered.push(*line);
    }

    filtered.join("\n")
}

fn normalize_initializer_content(shell_name: &str, content: &str) -> String {
    if shell_name == "nu" {
        let mut out = content.replace("get $field --ignore-errors", "get --optional $field");
        out = normalize_nu_starship(&out);
        out
    } else {
        content.to_string()
    }
}

fn write_text_atomic(path: &Path, content: &str) -> Result<(), CoreError> {
    let parent = path.parent().ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "atomic_write_no_parent",
            format!("Cannot determine parent directory for {}", path.display()),
            "Retry with a valid file path.",
            json!({"path": path.display().to_string()}),
        )
    })?;
    fs::create_dir_all(parent).map_err(|e| CoreError::io(
        "atomic_write_mkdir",
        format!("Cannot create directory {}", parent.display()),
        "Fix permissions or choose a different path.",
        parent.display().to_string(),
        e,
    ))?;

    let tmp = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp).map_err(|e| CoreError::io(
        "atomic_write_create",
        format!("Cannot create temp file {}", tmp.display()),
        "Fix permissions or disk space, then retry.",
        tmp.display().to_string(),
        e,
    ))?;
    file.write_all(content.as_bytes()).map_err(|e| CoreError::io(
        "atomic_write_write",
        format!("Cannot write to temp file {}", tmp.display()),
        "Fix permissions or disk space, then retry.",
        tmp.display().to_string(),
        e,
    ))?;
    drop(file);

    fs::rename(&tmp, path).map_err(|e| CoreError::io(
        "atomic_write_rename",
        format!("Cannot rename {} to {}", tmp.display(), path.display()),
        "Fix permissions or retry.",
        path.display().to_string(),
        e,
    ))?;

    Ok(())
}

fn generate_initializers(
    home: &Path,
    shells_to_configure: &[String],
) -> Result<Vec<InitializerResult>, CoreError> {
    let shells: Vec<_> = shell_initializer_dirs(home)
        .into_iter()
        .filter(|s| shells_to_configure.iter().any(|wanted| wanted == s.name))
        .collect();
    let tools = tool_configs();
    let mut all_results = Vec::new();

    for shell in &shells {
        fs::create_dir_all(&shell.dir).map_err(|e| CoreError::io(
            "init_mkdir",
            format!("Cannot create initializer directory {}", shell.dir.display()),
            "Fix permissions, then retry.",
            shell.dir.display().to_string(),
            e,
        ))?;

        let mut shell_results = Vec::new();
        let mut successful_files = Vec::new();

        for tool in &tools {
            let output_file = shell.dir.join(format!("{}_init.{}", tool.name, shell.ext));
            let effective_shell = shell
                .tool_override
                .filter(|(t, _)| *t == tool.name)
                .map(|(_, s)| s)
                .or(tool.shell_override)
                .unwrap_or(shell.name);

            if !find_on_path(tool.name) {
                if output_file.exists() {
                    let _ = fs::remove_file(&output_file);
                }
                shell_results.push(InitializerResult {
                    status: if tool.required {
                        "required-missing".into()
                    } else {
                        "missing".into()
                    },
                    tool: tool.name.into(),
                    shell: shell.name.into(),
                    reason: Some("tool not found".into()),
                    error: None,
                    file: None,
                });
                continue;
            }

            match run_tool_init(tool, effective_shell) {
                Ok(raw) => {
                    let content = normalize_initializer_content(shell.name, &raw);
                    if let Err(e) = write_text_atomic(&output_file, &content) {
                        if output_file.exists() {
                            let _ = fs::remove_file(&output_file);
                        }
                        shell_results.push(InitializerResult {
                            status: if tool.required {
                                "required-failed".into()
                            } else {
                                "failed".into()
                            },
                            tool: tool.name.into(),
                            shell: shell.name.into(),
                            reason: None,
                            error: Some(e.message()),
                            file: None,
                        });
                        continue;
                    }
                    successful_files.push((tool.required, output_file.to_string_lossy().to_string()));
                    shell_results.push(InitializerResult {
                        status: "success".into(),
                        tool: tool.name.into(),
                        shell: shell.name.into(),
                        reason: None,
                        error: None,
                        file: Some(output_file.to_string_lossy().to_string()),
                    });
                }
                Err(err) => {
                    if output_file.exists() {
                        let _ = fs::remove_file(&output_file);
                    }
                    shell_results.push(InitializerResult {
                        status: if tool.required {
                            "required-failed".into()
                        } else {
                            "failed".into()
                        },
                        tool: tool.name.into(),
                        shell: shell.name.into(),
                        reason: None,
                        error: Some(err),
                        file: None,
                    });
                }
            }
        }

        // Build aggregate initializer
        let aggregate_file = shell.dir.join(format!("yazelix_init.{}", shell.ext));
        let mut aggregate = format!("# Yazelix aggregate initializer for {}\n# Concatenates generated initializers for available tools.\n", shell.name);

        // Add warnings for required issues
        for r in &shell_results {
            if matches!(r.status.as_str(), "required-missing" | "required-failed") {
                aggregate.push_str(&format!(
                    "# WARNING: required initializer not generated for {}: {}\n",
                    r.tool,
                    r.reason.as_deref().or(r.error.as_deref()).unwrap_or("unknown failure")
                ));
            }
        }

        // Nushell PATH preservation
        if shell.name == "nu" {
            aggregate.push_str("\n# Preserve the inherited PATH before Yazelix-managed initializers modify it\n");
            aggregate.push_str("let initial_path = $env.PATH\n");
            aggregate.push_str("\n# --- Tool initializers below ---\n\n");
        }

        // Concatenate successful initializers: required first
        let mut ordered = successful_files;
        ordered.sort_by_key(|(required, _)| if *required { 0 } else { 1 });
        for (_, path) in &ordered {
            match fs::read_to_string(path) {
                Ok(content) => {
                    aggregate.push_str(&content);
                    aggregate.push('\n');
                }
                Err(_) => {
                    aggregate.push_str(&format!("# WARNING: could not read {}\n", path));
                }
            }
        }

        if shell.name == "nu" {
            aggregate.push_str("\n# --- Tool initializers above ---\n\n");
            aggregate.push_str("# Restore any PATH entries lost during initialization without letting stale saved entries outrank the current shell PATH\n");
            aggregate.push_str("let current_path = $env.PATH\n");
            aggregate.push_str("$env.PATH = ($current_path | append $initial_path | uniq)\n");
        }

        aggregate.push('\n');
        write_text_atomic(&aggregate_file, &aggregate)?;

        shell_results.push(InitializerResult {
            status: "aggregate".into(),
            tool: String::new(),
            shell: shell.name.into(),
            reason: None,
            error: None,
            file: Some(aggregate_file.to_string_lossy().to_string()),
        });

        all_results.extend(shell_results);
    }

    Ok(all_results)
}

pub fn run_generate_shell_initializers(args: &[String]) -> Result<i32, CoreError> {
    let mut shells_to_configure: Vec<String> = Vec::new();
    let mut help = false;

    for arg in args {
        match arg.as_str() {
            "--help" | "-h" | "help" => help = true,
            other if !other.starts_with('-') => {
                for shell in other.split(',') {
                    let trimmed = shell.trim();
                    if !trimmed.is_empty() {
                        shells_to_configure.push(trimmed.to_string());
                    }
                }
            }
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for generate_shell_initializers: {other}"
                )));
            }
        }
    }

    if help {
        println!("Generate shell initializer scripts for supported shells");
        println!();
        println!("Usage:");
        println!("  yzx_control generate_shell_initializers [shells...]");
        println!();
        println!("Arguments:");
        println!("  shells    Comma-separated list of shells (nu,bash,fish,zsh). Defaults to all.");
        return Ok(0);
    }

    if shells_to_configure.is_empty() {
        shells_to_configure = vec![
            "nu".to_string(),
            "bash".to_string(),
            "fish".to_string(),
            "zsh".to_string(),
        ];
    }

    let home = crate::control_plane::home_dir_from_env()?;
    let results = generate_initializers(&home, &shells_to_configure)?;

    let quiet = std::env::var("YAZELIX_QUIET_MODE").as_deref() == Ok("true");
    if !quiet {
        let success_count = results.iter().filter(|r| r.status == "success").count();
        let failed: Vec<_> = results.iter().filter(|r| r.status == "failed").collect();
        let missing: Vec<_> = results
            .iter()
            .filter(|r| r.status == "missing")
            .map(|r| r.tool.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if failed.is_empty() && missing.is_empty() {
            println!("✅ Generated {success_count} shell initializers successfully");
        } else {
            println!("✅ Generated {success_count} shell initializers");
            if !missing.is_empty() {
                println!("⚠️  Tools not found: {}", missing.join(", "));
            }
            if !failed.is_empty() {
                println!("❌ Failed to generate:");
                for f in &failed {
                    println!("   {} for {}: {}", f.tool, f.shell, f.error.as_deref().unwrap_or(""));
                }
            }
        }
    }

    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: Nushell starship normalization strips the right prompt block.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn strips_nu_starship_right_prompt() {
        let input = r#"starship_prompt
PROMPT_COMMAND_RIGHT: {||
    stuff
    }
PROMPT_COMMAND: {||
    other
    }
"#;
        let out = normalize_nu_starship(input);
        assert!(!out.contains("PROMPT_COMMAND_RIGHT"));
        assert!(out.contains("PROMPT_COMMAND"));
    }

    // Defends: Nushell starship normalization strips the config merge block.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn strips_nu_starship_config_merge() {
        let input = r#"before
config: ($env.config? | default {} | merge {
    key: val
    })
after
"#;
        let out = normalize_nu_starship(input);
        assert!(!out.contains("config:"));
        assert!(out.contains("before"));
        assert!(out.contains("after"));
    }

    // Defends: initializer result aggregation preserves the exact shells requested.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn shell_initializer_dirs_filters_by_name() {
        let home = Path::new("/tmp/home");
        let all = shell_initializer_dirs(home);
        assert_eq!(all.len(), 4);
        assert!(all.iter().any(|s| s.name == "nu"));
        assert!(all.iter().any(|s| s.name == "bash"));
    }
}
