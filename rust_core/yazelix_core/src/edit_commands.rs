// Test lane: default
//! `yzx edit` family implemented in Rust for `yzx_control`.

use crate::active_config_surface::resolve_active_config_paths;
use crate::bridge::{CoreError, ErrorClass};
use crate::compute_runtime_env;
use crate::control_plane::{
    config_dir_from_env, config_override_from_env, load_normalized_config_for_control,
    runtime_dir_from_env, runtime_env_request,
};
use crate::ghostty_cursor_registry::CursorRegistry;
use crate::user_config_paths;
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditTarget {
    id: &'static str,
    label: String,
    path: PathBuf,
    aliases: &'static [&'static str],
    search: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EditArgs {
    query: Vec<String>,
    print: bool,
    help: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct EditConfigArgs {
    print: bool,
    help: bool,
}

fn get_edit_targets(config_dir: &Path) -> Vec<EditTarget> {
    let helix_path = user_config_paths::helix_config(config_dir);
    let zellij_path = user_config_paths::zellij_config(config_dir);
    let yazi_toml_path = user_config_paths::yazi_config(config_dir);
    let yazi_keymap_path = user_config_paths::yazi_keymap(config_dir);
    let yazi_init_path = user_config_paths::yazi_init(config_dir);

    let runtime_dir = runtime_dir_from_env().unwrap_or_else(|_| PathBuf::from("."));
    let active_paths = resolve_active_config_paths(&runtime_dir, config_dir, None).ok();
    let user_config = active_paths
        .as_ref()
        .map(|p| p.user_config.clone())
        .unwrap_or_else(|| user_config_paths::main_config(config_dir));
    let cursor_config = active_paths
        .as_ref()
        .map(|p| p.user_cursor_config.clone())
        .unwrap_or_else(|| CursorRegistry::user_config_path(config_dir));

    vec![
        EditTarget {
            id: "config",
            label: format!("config  - main Yazelix config → {}", user_config.display()),
            path: user_config,
            aliases: &["config", "main", "settings", "settings.jsonc"],
            search: "config main yazelix settings settings.jsonc",
        },
        EditTarget {
            id: "cursors",
            label: format!(
                "cursors  - Ghostty cursor registry → {}",
                cursor_config.display()
            ),
            path: cursor_config,
            aliases: &["cursors", "cursor", "ghostty-cursors", "ghostty cursors"],
            search: "cursors cursor ghostty cursor trail shader settings settings.jsonc",
        },
        EditTarget {
            id: "helix",
            label: format!(
                "helix  - managed Helix user config → {}",
                helix_path.display()
            ),
            path: helix_path,
            aliases: &["helix", "hx", "editor"],
            search: "helix hx editor config config.toml",
        },
        EditTarget {
            id: "zellij",
            label: format!(
                "zellij  - managed Zellij user config → {}",
                zellij_path.display()
            ),
            path: zellij_path,
            aliases: &["zellij", "terminal", "config.kdl"],
            search: "zellij terminal config.kdl multiplexer",
        },
        EditTarget {
            id: "yazi",
            label: format!(
                "yazi  - managed Yazi main config (yazi.toml) → {}",
                yazi_toml_path.display()
            ),
            path: yazi_toml_path,
            aliases: &["yazi", "yazi.toml", "file-manager"],
            search: "yazi yazi.toml file-manager file manager",
        },
        EditTarget {
            id: "yazi-keymap",
            label: format!(
                "yazi-keymap  - managed Yazi keymap (keymap.toml) → {}",
                yazi_keymap_path.display()
            ),
            path: yazi_keymap_path,
            aliases: &["yazi-keymap", "keymap", "keymap.toml", "yazi keymap"],
            search: "yazi keymap keymap.toml file-manager bindings",
        },
        EditTarget {
            id: "yazi-init",
            label: format!(
                "yazi-init  - managed Yazi init.lua → {}",
                yazi_init_path.display()
            ),
            path: yazi_init_path,
            aliases: &["yazi-init", "init", "init.lua", "yazi init", "lua"],
            search: "yazi init init.lua lua file-manager plugins",
        },
    ]
}

fn filter_edit_targets<'a>(targets: &'a [EditTarget], query: &str) -> Vec<&'a EditTarget> {
    let normalized = query.trim().to_lowercase();
    if normalized.is_empty() {
        return targets.iter().collect();
    }

    let exact: Vec<_> = targets
        .iter()
        .filter(|t| {
            t.id.to_lowercase() == normalized
                || t.aliases.iter().any(|a| a.to_lowercase() == normalized)
        })
        .collect();
    if !exact.is_empty() {
        return exact;
    }

    let tokens: Vec<_> = normalized.split_whitespace().collect();
    targets
        .iter()
        .filter(|t| {
            let haystack = format!("{} {} {}", t.id, t.aliases.join(" "), t.search).to_lowercase();
            tokens.iter().all(|token| haystack.contains(token))
        })
        .collect()
}

fn resolve_editor(runtime_dir: &Path) -> Result<(String, Vec<(String, String)>), CoreError> {
    let config_dir = config_dir_from_env()?;
    let normalized = load_normalized_config_for_control(
        runtime_dir,
        &config_dir,
        config_override_from_env().as_deref(),
    )?;

    let req = runtime_env_request(runtime_dir.to_path_buf(), &normalized)?;
    let data = compute_runtime_env(&req)?;

    let mut editor = data
        .runtime_env
        .get("EDITOR")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if editor.is_empty() {
        editor = std::env::var("EDITOR").unwrap_or_default();
    }

    if editor.trim().is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "missing_editor",
            "EDITOR is not set. Set it in settings.jsonc under editor.command, or export EDITOR in your shell.",
            "Update settings.jsonc or your shell environment, then retry.",
            json!({}),
        ));
    }

    let editor = normalize_editor_command(&editor, runtime_dir);

    let mut env_vars: Vec<(String, String)> = Vec::new();
    for (key, value) in &data.runtime_env {
        if let Some(s) = value.as_str() {
            env_vars.push((key.clone(), s.to_string()));
        }
    }

    // Ensure YAZELIX_RUNTIME_DIR is set for yazelix_hx.sh
    if Path::new(&editor)
        .file_name()
        .map(|n| n == "yazelix_hx.sh")
        .unwrap_or(false)
    {
        env_vars.push((
            "YAZELIX_RUNTIME_DIR".to_string(),
            runtime_dir.to_string_lossy().to_string(),
        ));
    }

    Ok((editor, env_vars))
}

fn normalize_editor_command(editor: &str, runtime_dir: &Path) -> String {
    let trimmed = editor.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if Path::new(trimmed)
        .file_name()
        .map(|n| n == "yazelix_hx.sh")
        .unwrap_or(false)
    {
        return runtime_dir
            .join("shells")
            .join("posix")
            .join("yazelix_hx.sh")
            .to_string_lossy()
            .to_string();
    }
    trimmed.to_string()
}

fn exec_editor(editor: &str, path: &Path, env_vars: &[(String, String)]) -> Result<i32, CoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| {
            CoreError::io(
                "edit_mkdir",
                format!("Could not create parent directory {}.", parent.display()),
                "Fix permissions or choose a different path, then retry.",
                parent.display().to_string(),
                source,
            )
        })?;
    }

    let mut cmd = Command::new(editor);
    cmd.arg(path);
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        return Err(CoreError::io(
            "edit_exec",
            format!("Could not exec editor {} with {}.", editor, path.display()),
            "Ensure the editor is installed and on PATH, then retry.",
            path.display().to_string(),
            err,
        ));
    }
    #[cfg(not(unix))]
    {
        let status = cmd.status().map_err(|source| {
            CoreError::io(
                "edit_exec",
                format!("Could not run editor {} with {}.", editor, path.display()),
                "Ensure the editor is installed and on PATH, then retry.",
                path.display().to_string(),
                source,
            )
        })?;
        Ok(status.code().unwrap_or(1))
    }
}

fn select_target_interactive<'a>(
    targets: &'a [&'a EditTarget],
) -> Result<Option<&'a EditTarget>, CoreError> {
    if targets.is_empty() {
        return Ok(None);
    }
    if targets.len() == 1 {
        return Ok(Some(targets[0]));
    }

    // Try fzf first
    let fzf_available = Command::new("fzf").arg("--version").output().is_ok();
    if fzf_available {
        let mut child = Command::new("fzf")
            .arg("--height")
            .arg("40%")
            .arg("--reverse")
            .arg("--prompt")
            .arg("yzx edit> ")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|source| {
                CoreError::io(
                    "fzf_spawn",
                    "Could not start fzf for interactive selection.",
                    "Install fzf or provide an exact target query, then retry.",
                    "fzf",
                    source,
                )
            })?;

        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            for target in targets {
                let _ = writeln!(stdin, "{}", target.label);
            }
        }

        let output = child.wait_with_output().map_err(|source| {
            CoreError::io(
                "fzf_wait",
                "fzf process failed.",
                "Retry or provide an exact target query.",
                "fzf",
                source,
            )
        })?;

        if !output.status.success() {
            return Ok(None);
        }

        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(targets.iter().find(|t| t.label == selected).copied());
    }

    // Fallback: numbered list
    eprintln!("Available edit targets:");
    for (i, target) in targets.iter().enumerate() {
        eprintln!("  {}. {}  → {}", i + 1, target.id, target.path.display());
    }
    eprint!("Select target (1-{}, or empty to cancel): ", targets.len());
    io::stderr().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return Ok(None);
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    match trimmed.parse::<usize>() {
        Ok(n) if n > 0 && n <= targets.len() => Ok(Some(targets[n - 1])),
        _ => Ok(None),
    }
}

fn parse_edit_args(args: &[String]) -> Result<EditArgs, CoreError> {
    let mut parsed = EditArgs::default();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--print" => parsed.print = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other if other.starts_with('-') => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx edit: {other}. Try `yzx edit --help`."
                )));
            }
            other => parsed.query.push(other.to_string()),
        }
    }

    Ok(parsed)
}

fn parse_edit_config_args(args: &[String]) -> Result<EditConfigArgs, CoreError> {
    let mut parsed = EditConfigArgs::default();

    for arg in args {
        match arg.as_str() {
            "--print" => parsed.print = true,
            "-h" | "--help" | "help" => parsed.help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx edit config: {other}. Try `yzx edit config --help`."
                )));
            }
        }
    }

    Ok(parsed)
}

pub fn run_yzx_edit(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_edit_args(args)?;
    if parsed.help {
        print_edit_help();
        return Ok(0);
    }

    let config_dir = config_dir_from_env()?;
    let targets = get_edit_targets(&config_dir);
    let query = parsed.query.join(" ");

    if query.trim().is_empty() {
        if parsed.print {
            return Err(CoreError::usage(format!(
                "yzx edit --print requires a target query. Supported managed surfaces:\n{}",
                format_target_list(&targets)
            )));
        }

        let target_refs: Vec<_> = targets.iter().collect();
        let selected = select_target_interactive(&target_refs)?;
        let Some(target) = selected else {
            return Ok(0);
        };

        let runtime_dir = runtime_dir_from_env()?;
        let (editor, env_vars) = resolve_editor(&runtime_dir)?;
        return exec_editor(&editor, &target.path, &env_vars);
    }

    let matches = filter_edit_targets(&targets, &query);

    if matches.is_empty() {
        return Err(CoreError::usage(format!(
            "No managed Yazelix config surface matched `{query}`. Supported surfaces:\n{}",
            format_target_list(&targets)
        )));
    }

    if matches.len() == 1 {
        let target = matches[0];
        if parsed.print {
            println!("{}", target.path.display());
            return Ok(0);
        }
        let runtime_dir = runtime_dir_from_env()?;
        let (editor, env_vars) = resolve_editor(&runtime_dir)?;
        return exec_editor(&editor, &target.path, &env_vars);
    }

    if parsed.print {
        return Err(CoreError::usage(format!(
            "Query `{query}` matched multiple managed config surfaces. Refine it to one of:\n{}",
            format_target_list_refs(&matches)
        )));
    }

    let selected = select_target_interactive(&matches)?;
    let Some(target) = selected else {
        return Ok(0);
    };

    let runtime_dir = runtime_dir_from_env()?;
    let (editor, env_vars) = resolve_editor(&runtime_dir)?;
    exec_editor(&editor, &target.path, &env_vars)
}

pub fn run_yzx_edit_config(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_edit_config_args(args)?;
    if parsed.help {
        print_edit_config_help();
        return Ok(0);
    }

    let config_dir = config_dir_from_env()?;
    let targets = get_edit_targets(&config_dir);
    let target = targets
        .iter()
        .find(|t| t.id == "config")
        .expect("config target always exists");

    if parsed.print {
        println!("{}", target.path.display());
        return Ok(0);
    }

    let runtime_dir = runtime_dir_from_env()?;
    let (editor, env_vars) = resolve_editor(&runtime_dir)?;
    exec_editor(&editor, &target.path, &env_vars)
}

fn format_target_list(targets: &[EditTarget]) -> String {
    targets
        .iter()
        .map(|t| format!("  - {}: {}", t.id, t.path.display()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_target_list_refs(targets: &[&EditTarget]) -> String {
    targets
        .iter()
        .map(|t| format!("  - {}: {}", t.id, t.path.display()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn print_edit_help() {
    println!("Open a Yazelix-managed config surface in the configured editor");
    println!();
    println!("Usage:");
    println!("  yzx edit [query...] [--print]");
    println!();
    println!("Arguments:");
    println!("  query    Optional managed config surface name or alias");
    println!();
    println!("Flags:");
    println!("  --print  Print the resolved config path without opening");
    println!();
    println!("Supported surfaces:");
    println!("  config, cursors, helix, zellij, yazi, yazi-keymap, yazi-init");
}

fn print_edit_config_help() {
    println!("Open the main Yazelix config in the configured editor");
    println!();
    println!("Usage:");
    println!("  yzx edit config [--print]");
    println!();
    println!("Flags:");
    println!("  --print  Print the config path without opening");
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: edit target filtering keeps exact id match, alias match, and token search.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn filters_edit_targets_by_exact_and_fuzzy_match() {
        let targets = get_edit_targets(Path::new("/tmp/cfg"));
        assert!(!targets.is_empty());

        let helix = filter_edit_targets(&targets, "helix");
        assert_eq!(helix.len(), 1);
        assert_eq!(helix[0].id, "helix");

        let hx = filter_edit_targets(&targets, "hx");
        assert_eq!(hx.len(), 1);
        assert_eq!(hx[0].id, "helix");

        let cursors = filter_edit_targets(&targets, "cursors");
        assert_eq!(cursors.len(), 1);
        assert_eq!(cursors[0].id, "cursors");
        assert_eq!(
            cursors[0].path,
            Path::new("/tmp/cfg").join("settings.jsonc")
        );

        let settings = filter_edit_targets(&targets, "settings.jsonc");
        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].id, "config");

        let yazi = filter_edit_targets(&targets, "yazi");
        assert_eq!(yazi.len(), 1); // exact id match takes precedence
        assert_eq!(yazi[0].id, "yazi");

        // Fuzzy fallback when no exact match
        let file_mgr = filter_edit_targets(&targets, "file manager");
        assert_eq!(file_mgr.len(), 3); // yazi, yazi-keymap, yazi-init

        let keymap = filter_edit_targets(&targets, "keymap");
        assert_eq!(keymap.len(), 1);
        assert_eq!(keymap[0].id, "yazi-keymap");

        let empty = filter_edit_targets(&targets, "not-a-thing");
        assert!(empty.is_empty());
    }

    // Defends: edit argument parsing keeps query tokens, --print, and rejects unknown flags.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn parses_edit_args() {
        let parsed = parse_edit_args(&["helix".into(), "--print".into()]).unwrap();
        assert_eq!(parsed.query, vec!["helix"]);
        assert!(parsed.print);

        let parsed = parse_edit_args(&["yazi".into(), "keymap".into()]).unwrap();
        assert_eq!(parsed.query, vec!["yazi", "keymap"]);

        assert!(parse_edit_args(&["--unknown".into()]).is_err());
    }

    // Defends: editor normalization resolves yazelix_hx.sh relative to runtime_dir.
    // Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
    #[test]
    fn normalizes_yazelix_hx_editor_path() {
        let rt = Path::new("/tmp/rt");
        assert_eq!(
            normalize_editor_command("yazelix_hx.sh", rt),
            "/tmp/rt/shells/posix/yazelix_hx.sh"
        );
        assert_eq!(
            normalize_editor_command("/abs/path/to/hx", rt),
            "/abs/path/to/hx"
        );
        assert_eq!(normalize_editor_command("  hx  ", rt), "hx");
    }
}
