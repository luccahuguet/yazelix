use crate::atomic_fs::is_executable_file;
use crate::bridge::{CoreError, ErrorClass};
use serde_json::json;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const CODEX_AGENT_COMMAND: &str = "codex";
const RTK_COMMAND: &str = "rtk";
const AGENT_INIT_DEFAULT_META_ROOT: &str = "/home/flexnetos/meta";
const PLACEHOLDER_SHELL_CANDIDATES: &[&str] = &["nu", "bash", "sh"];
const MISSING_CODEX_PLACEHOLDER: &str = "\
Yazelix right sidebar

Codex is not installed or is not on PATH.
This pane is a normal shell; run any command here, or configure the managed right sidebar.

Agent examples:
  yzx config set workspace.right_sidebar.command opencode
  yzx config set workspace.right_sidebar.command claude

The right sidebar command does not have to be an AI agent:
  yzx config set workspace.right_sidebar.command nu
  yzx config ui

Starting a shell...
";

pub fn run_yzx_agent(args: &[String]) -> Result<i32, CoreError> {
    if matches!(args.first().map(String::as_str), Some("init")) {
        return run_agent_init(args);
    }

    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h" | "help") {
        print_agent_help();
        return Ok(0);
    }

    if !args.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Usage,
            "unexpected_agent_argument",
            format!("Unexpected argument for yzx agent: {}", args[0]),
            "Run `yzx agent` without arguments for the managed right agent pane.",
            json!({ "argument": args[0] }),
        ));
    }

    let path = std::env::var_os("PATH").unwrap_or_default();
    let Some(agent_command) = resolve_agent_command(&path)? else {
        print_missing_codex_placeholder()?;
        return run_placeholder_shell(&path);
    };

    let status = Command::new(&agent_command)
        .arg(CODEX_AGENT_COMMAND)
        .status()
        .map_err(|source| {
        CoreError::io(
            "rtk_codex_agent",
            "Failed to launch the Codex agent command through RTK.",
            "Install RTK and Codex on the host, make sure `rtk` and `codex` are executable on PATH, then restart Yazelix.",
            format!("{} {}", agent_command.display(), CODEX_AGENT_COMMAND),
            source,
        )
    })?;

    Ok(status.code().unwrap_or(1))
}

fn print_agent_help() {
    println!("Open the managed right agent pane");
    println!();
    println!("Usage:");
    println!("  yzx agent");
    println!("  yzx agent init [--apply] [--meta-root <path>] [--repo <path>]");
    println!();
    println!("When host-installed Codex is available, this command launches `rtk codex`.");
    println!("When Codex is missing, it opens a normal shell with setup guidance.");
    println!();
    println!("`yzx agent init` previews the bounded agent-harness setup without mutation.");
    println!(
        "Pass `--apply` to create a missing Meta GitKB, initialize Grit and ICM, and apply RTK setup."
    );
    println!(
        "It never enables Codex hooks or plugins, installs user-local shims, or rewrites Git commands."
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentInitArgs {
    apply: bool,
    meta_root: PathBuf,
    repo: Option<PathBuf>,
    help: bool,
}

#[derive(Debug, Clone)]
struct AgentInitEnvironment {
    path: std::ffi::OsString,
    cwd: PathBuf,
}

fn run_agent_init(args: &[String]) -> Result<i32, CoreError> {
    let parsed = parse_agent_init_args(&args[1..])?;
    if parsed.help {
        print_agent_init_help();
        return Ok(0);
    }

    let env = AgentInitEnvironment {
        path: std::env::var_os("PATH").unwrap_or_default(),
        cwd: std::env::current_dir().map_err(|source| {
            CoreError::io(
                "agent_init_current_dir",
                "Could not determine the current directory for `yzx agent init`.",
                "Run the command from a working directory, or pass `--repo <path>`.",
                ".",
                source,
            )
        })?,
    };
    execute_agent_init(&parsed, &env)
}

fn parse_agent_init_args(args: &[String]) -> Result<AgentInitArgs, CoreError> {
    let mut parsed = AgentInitArgs {
        apply: false,
        meta_root: std::env::var_os("META_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(AGENT_INIT_DEFAULT_META_ROOT)),
        repo: None,
        help: false,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--apply" => parsed.apply = true,
            "--meta-root" => {
                index += 1;
                parsed.meta_root = required_agent_init_path(args, index, "--meta-root")?;
            }
            "--repo" => {
                index += 1;
                parsed.repo = Some(required_agent_init_path(args, index, "--repo")?);
            }
            "-h" | "--help" | "help" => parsed.help = true,
            other => {
                return Err(CoreError::usage(format!(
                    "Unknown argument for yzx agent init: {other}. Try `yzx agent init --help`."
                )));
            }
        }
        index += 1;
    }
    Ok(parsed)
}

fn required_agent_init_path(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<PathBuf, CoreError> {
    args.get(index)
        .filter(|value| !value.starts_with('-'))
        .map(PathBuf::from)
        .ok_or_else(|| CoreError::usage(format!("{flag} requires a path.")))
}

fn print_agent_init_help() {
    println!("Preview or apply the bounded profile-owned agent harness initialization");
    println!();
    println!("Usage:");
    println!("  yzx agent init [--apply] [--meta-root <path>] [--repo <path>]");
    println!();
    println!("Flags:");
    println!("      --apply             Permit the bounded mutation steps");
    println!(
        "      --meta-root <path>  Meta root for GitKB and fleet validation (default: $META_ROOT or {AGENT_INIT_DEFAULT_META_ROOT})"
    );
    println!(
        "      --repo <path>       Git repository for Grit and ICM (default: current Git repository)"
    );
    println!();
    println!(
        "Preview validates existing GitKB state, verifies Meta fleet dispatch, and uses RTK dry-run."
    );
    println!(
        "Apply creates a missing GitKB, runs `grit -r <repo> init`, `icm init --mode cli --force`, and `rtk init --global --codex`."
    );
    println!(
        "This command does not enable Codex hooks or plugins, create user-local shims, or rewrite Git commands."
    );
}

fn execute_agent_init(args: &AgentInitArgs, env: &AgentInitEnvironment) -> Result<i32, CoreError> {
    let git_kb = resolve_required_agent_init_tool("git-kb", &env.path)?;
    let grit = resolve_required_agent_init_tool("grit", &env.path)?;
    let icm = resolve_required_agent_init_tool("icm", &env.path)?;
    let meta = resolve_required_agent_init_tool("meta", &env.path)?;
    let rtk = resolve_required_agent_init_tool(RTK_COMMAND, &env.path)?;
    let git = resolve_required_agent_init_tool("git", &env.path)?;
    let repo = resolve_agent_init_repo(args.repo.as_deref(), &env.cwd, &git)?;

    println!(
        "yzx agent init: {}",
        if args.apply {
            "apply"
        } else {
            "read-only preview"
        }
    );
    run_git_kb_step(args, &git_kb)?;
    run_grit_step(args, &grit, &repo)?;
    run_icm_step(args, &icm, &repo)?;
    run_meta_step(&meta, &args.meta_root)?;
    run_rtk_step(args, &rtk, &env.cwd)?;
    println!("yzx agent init: complete");
    Ok(0)
}

fn resolve_required_agent_init_tool(command: &str, path: &OsStr) -> Result<PathBuf, CoreError> {
    resolve_command_on_path(command, path).ok_or_else(|| {
        CoreError::classified(
            ErrorClass::Runtime,
            "agent_init_missing_tool",
            format!("`yzx agent init` requires `{command}` on the active profile/PATH."),
            format!(
                "Install or expose `{command}` through the active Yazelix/Nix profile, then retry."
            ),
            json!({ "missing_command": command, "path": path.to_string_lossy() }),
        )
    })
}

fn resolve_agent_init_repo(
    explicit_repo: Option<&Path>,
    cwd: &Path,
    git: &Path,
) -> Result<PathBuf, CoreError> {
    if let Some(repo) = explicit_repo {
        return git_toplevel(git, repo);
    }
    git_toplevel(git, cwd)
}

fn git_toplevel(git: &Path, path: &Path) -> Result<PathBuf, CoreError> {
    let output = Command::new(git)
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|source| {
            CoreError::io(
                "agent_init_git_probe",
                format!("Could not resolve a Git repository from {}.", path.display()),
                "Run from a Git repository or pass `--repo <path>` for the local Grit and ICM steps.",
                git.display().to_string(),
                source,
            )
        })?;
    if !output.status.success() {
        return Err(agent_init_command_failed(
            "local_git_repository",
            git,
            &[
                "-C".to_string(),
                path.display().to_string(),
                "rev-parse".to_string(),
                "--show-toplevel".to_string(),
            ],
            output.status.code(),
        ));
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if root.is_empty() {
        return Err(CoreError::classified(
            ErrorClass::Runtime,
            "agent_init_git_repository",
            format!(
                "Git did not report a repository root for {}.",
                path.display()
            ),
            "Run from a Git repository or pass `--repo <path>` for the local Grit and ICM steps.",
            json!({ "path": path, "git": git }),
        ));
    }
    Ok(PathBuf::from(root))
}

fn run_git_kb_step(args: &AgentInitArgs, git_kb: &Path) -> Result<(), CoreError> {
    let kb_dir = args.meta_root.join(".kb");
    if kb_dir.is_dir() {
        println!(
            "[1/5] GitKB: validating existing initialization at {}",
            args.meta_root.display()
        );
        run_agent_init_command(
            "GitKB validation",
            git_kb,
            &["verify", "--full", "--json"],
            &args.meta_root,
        )?;
        run_agent_init_command(
            "GitKB Codex scaffold preview",
            git_kb,
            &["init", "codex", "--dry-run"],
            &args.meta_root,
        )?;
    } else if args.apply {
        println!(
            "[1/5] GitKB: creating missing initialization at {}",
            args.meta_root.display()
        );
        run_agent_init_command(
            "GitKB initialization",
            git_kb,
            &["init", "--no-verify"],
            &args.meta_root,
        )?;
        run_agent_init_command(
            "GitKB Codex scaffold",
            git_kb,
            &["init", "codex"],
            &args.meta_root,
        )?;
    } else {
        println!(
            "[1/5] GitKB: preview leaves missing {} unchanged",
            kb_dir.display()
        );
    }
    Ok(())
}

fn run_grit_step(args: &AgentInitArgs, grit: &Path, repo: &Path) -> Result<(), CoreError> {
    if args.apply {
        println!("[2/5] Grit: initializing repository {}", repo.display());
        let repo_arg = repo.display().to_string();
        run_agent_init_command(
            "Grit initialization",
            grit,
            &["-r", &repo_arg, "init"],
            repo,
        )?;
    } else {
        println!(
            "[2/5] Grit: preview would run `grit -r {} init`",
            repo.display()
        );
    }
    Ok(())
}

fn run_icm_step(args: &AgentInitArgs, icm: &Path, repo: &Path) -> Result<(), CoreError> {
    if args.apply {
        println!(
            "[3/5] ICM: initializing CLI-only integration for {}",
            repo.display()
        );
        run_agent_init_command(
            "ICM CLI initialization",
            icm,
            &["init", "--mode", "cli", "--force"],
            repo,
        )?;
    } else {
        println!(
            "[3/5] ICM: preview would run CLI-only initialization for {}",
            repo.display()
        );
    }
    Ok(())
}

fn run_meta_step(meta: &Path, meta_root: &Path) -> Result<(), CoreError> {
    println!(
        "[4/5] Meta: validating fleet dispatch from {}",
        meta_root.display()
    );
    run_agent_init_command(
        "Meta fleet dispatch dry-run",
        meta,
        &[
            "--dry-run",
            "exec",
            "--",
            "git",
            "status",
            "--short",
            "--branch",
        ],
        meta_root,
    )
}

fn run_rtk_step(args: &AgentInitArgs, rtk: &Path, cwd: &Path) -> Result<(), CoreError> {
    let command = if args.apply {
        ["init", "--global", "--codex"].as_slice()
    } else {
        ["init", "--global", "--codex", "--dry-run"].as_slice()
    };
    println!(
        "[5/5] RTK: {} global Codex initialization",
        if args.apply { "applying" } else { "previewing" }
    );
    run_agent_init_command("RTK initialization", rtk, command, cwd)
}

fn run_agent_init_command(
    step: &str,
    executable: &Path,
    args: &[&str],
    current_dir: &Path,
) -> Result<(), CoreError> {
    let status = Command::new(executable)
        .args(args)
        .current_dir(current_dir)
        .status()
        .map_err(|source| {
            CoreError::io(
                "agent_init_command_launch",
                format!("{step} could not launch {}.", executable.display()),
                "Confirm the profile-owned executable is runnable, then retry.",
                executable.display().to_string(),
                source,
            )
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(agent_init_command_failed(
            step,
            executable,
            &args
                .iter()
                .map(|arg| (*arg).to_string())
                .collect::<Vec<_>>(),
            status.code(),
        ))
    }
}

fn agent_init_command_failed(
    step: &str,
    executable: &Path,
    args: &[String],
    exit_code: Option<i32>,
) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "agent_init_step_failed",
        format!("{step} failed with {}.", executable.display()),
        "Resolve the failing tool before retrying; no later initialization steps were run.",
        json!({ "step": step, "executable": executable, "args": args, "exit_code": exit_code }),
    )
}

fn resolve_agent_command(path: &OsStr) -> Result<Option<PathBuf>, CoreError> {
    if resolve_command_on_path(CODEX_AGENT_COMMAND, path).is_none() {
        return Ok(None);
    }

    resolve_command_on_path(RTK_COMMAND, path)
        .map(Some)
        .ok_or_else(|| missing_rtk_error(path))
}

fn missing_rtk_error(path: &OsStr) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_rtk",
        "Codex is available, but Yazelix could not find RTK for the managed agent command.",
        "Install or package upstream RTK so `rtk` is executable on PATH before launching `yzx agent`.",
        json!({
            "missing_command": RTK_COMMAND,
            "required_for": CODEX_AGENT_COMMAND,
            "path": path.to_string_lossy(),
        }),
    )
}

fn print_missing_codex_placeholder() -> Result<(), CoreError> {
    let mut stdout = io::stdout();
    stdout
        .write_all(MISSING_CODEX_PLACEHOLDER.as_bytes())
        .map_err(render_placeholder_error)?;
    stdout.flush().map_err(render_placeholder_error)
}

fn render_placeholder_error(source: io::Error) -> CoreError {
    CoreError::io(
        "agent_placeholder_render_failed",
        "Failed to render the Yazelix agent placeholder.",
        "Retry the command; if stdout is closed, reopen the right sidebar pane.",
        "stdout",
        source,
    )
}

fn run_placeholder_shell(path: &OsStr) -> Result<i32, CoreError> {
    let Some(shell) = resolve_placeholder_shell(path) else {
        return Err(missing_placeholder_shell_error(path));
    };
    let status = Command::new(&shell).status().map_err(|source| {
        CoreError::io(
            "agent_placeholder_shell",
            "Failed to launch the right sidebar placeholder shell.",
            "Install Codex, configure workspace.right_sidebar.command to another command, or make a shell available on PATH.",
            shell.display().to_string(),
            source,
        )
    })?;
    Ok(status.code().unwrap_or(1))
}

fn missing_placeholder_shell_error(path: &OsStr) -> CoreError {
    CoreError::classified(
        ErrorClass::Runtime,
        "missing_agent_placeholder_shell",
        "Codex is not on PATH, and Yazelix could not find a shell for the right sidebar placeholder.",
        "Install Codex, configure workspace.right_sidebar.command to another command, or make `nu`, `bash`, or `sh` available on PATH.",
        json!({
            "missing_command": CODEX_AGENT_COMMAND,
            "path": path.to_string_lossy(),
        }),
    )
}

fn resolve_placeholder_shell(path: &OsStr) -> Option<PathBuf> {
    resolve_placeholder_shell_for(path, std::env::var_os("SHELL").as_deref())
}

fn resolve_placeholder_shell_for(path: &OsStr, env_shell: Option<&OsStr>) -> Option<PathBuf> {
    if let Some(shell) = env_shell
        .map(PathBuf::from)
        .filter(|candidate| is_executable_file(candidate))
    {
        return Some(shell);
    }

    PLACEHOLDER_SHELL_CANDIDATES
        .iter()
        .find_map(|candidate| resolve_command_on_path(candidate, path))
}

fn resolve_command_on_path(command: &str, path: &OsStr) -> Option<PathBuf> {
    if command.contains('/') {
        let candidate = PathBuf::from(command);
        return is_executable_file(&candidate).then_some(candidate);
    }

    std::env::split_paths(path)
        .map(|entry| entry.join(command))
        .find(|candidate| is_executable_file(candidate))
}

// Test lane: default
#[cfg(test)]
mod tests {
    use super::{
        AgentInitArgs, AgentInitEnvironment, execute_agent_init, resolve_agent_command,
        resolve_command_on_path, resolve_placeholder_shell_for,
    };
    use std::ffi::OsStr;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    // Defends: yzx agent only launches a real executable from PATH, so missing host Codex does not accidentally exec arbitrary names.
    #[test]
    fn resolves_only_executable_commands_on_path() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let codex = bin.join("codex");
        fs::write(&codex, "#!/bin/sh\nexit 0\n").unwrap();

        #[cfg(unix)]
        {
            fs::set_permissions(&codex, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert_eq!(
            resolve_command_on_path("codex", bin.as_os_str()),
            Some(codex)
        );
        assert_eq!(resolve_command_on_path("opencode", bin.as_os_str()), None);
    }

    // Defends: managed Yazelix agent sessions route Codex through RTK instead of launching Codex directly.
    #[test]
    fn resolves_rtk_for_codex_agent() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let rtk = bin.join("rtk");
        let codex = bin.join("codex");
        fs::write(&rtk, "#!/bin/sh\nexit 0\n").unwrap();
        fs::write(&codex, "#!/bin/sh\nexit 0\n").unwrap();

        #[cfg(unix)]
        {
            fs::set_permissions(&rtk, fs::Permissions::from_mode(0o755)).unwrap();
            fs::set_permissions(&codex, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert_eq!(resolve_agent_command(bin.as_os_str()).unwrap(), Some(rtk));
    }

    // Defends: missing RTK is a visible runtime error when Codex would otherwise launch unmanaged.
    #[test]
    fn codex_without_rtk_is_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let codex = bin.join("codex");
        fs::write(&codex, "#!/bin/sh\nexit 0\n").unwrap();

        #[cfg(unix)]
        {
            fs::set_permissions(&codex, fs::Permissions::from_mode(0o755)).unwrap();
        }

        let error = resolve_agent_command(bin.as_os_str()).unwrap_err();
        assert_eq!(error.code(), "missing_rtk");
    }

    // Defends: missing Codex still opens the existing guided shell placeholder instead of requiring RTK.
    #[test]
    fn missing_codex_keeps_placeholder_path() {
        let temp = tempfile::tempdir().unwrap();
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let rtk = bin.join("rtk");
        fs::write(&rtk, "#!/bin/sh\nexit 0\n").unwrap();

        #[cfg(unix)]
        {
            fs::set_permissions(&rtk, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert_eq!(resolve_agent_command(bin.as_os_str()).unwrap(), None);
    }

    // Defends: the missing-Codex placeholder becomes an interactive pane by selecting an explicit shell.
    #[test]
    fn placeholder_shell_prefers_executable_user_shell() {
        let temp = tempfile::tempdir().unwrap();
        let shell = temp.path().join("shell");
        fs::write(&shell, "#!/bin/sh\nexit 0\n").unwrap();

        #[cfg(unix)]
        {
            fs::set_permissions(&shell, fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert_eq!(
            resolve_placeholder_shell_for(OsStr::new(""), Some(shell.as_os_str())),
            Some(shell)
        );
    }

    fn write_fake_executable(bin: &Path, name: &str, repo_root: &Path, exit_code: i32) {
        let executable = bin.join(name);
        let log = bin.join("agent_init.argv");
        fs::write(
            &executable,
            format!(
                "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$(basename \"$0\")\" \"$PWD\" \"$*\" >> '{}'\nif [ \"$(basename \"$0\")\" = git ]; then\n  printf '{}\\n'\nfi\nexit {exit_code}\n",
                log.display(),
                repo_root.display(),
            ),
        )
        .unwrap();
        #[cfg(unix)]
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn fake_agent_init_environment(
        temp: &tempfile::TempDir,
        repo_root: &Path,
    ) -> (AgentInitEnvironment, PathBuf) {
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        for tool in ["git-kb", "grit", "icm", "meta", "rtk", "git"] {
            write_fake_executable(&bin, tool, repo_root, 0);
        }
        (
            AgentInitEnvironment {
                path: bin.into_os_string(),
                cwd: repo_root.to_path_buf(),
            },
            temp.path().join("bin").join("agent_init.argv"),
        )
    }

    fn agent_init_args(apply: bool, meta_root: PathBuf, repo: Option<PathBuf>) -> AgentInitArgs {
        AgentInitArgs {
            apply,
            meta_root,
            repo,
            help: false,
        }
    }

    // Defends: preview resolves the profile/PATH tool set but does not create a missing GitKB or invoke mutable Grit and ICM initialization.
    #[test]
    fn agent_init_preview_has_no_mutations_when_gitkb_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("local_repo");
        let meta = temp.path().join("meta");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&meta).unwrap();
        let (env, log) = fake_agent_init_environment(&temp, &repo);

        execute_agent_init(&agent_init_args(false, meta.clone(), None), &env).unwrap();

        assert!(!meta.join(".kb").exists());
        let output = fs::read_to_string(log).unwrap();
        assert!(output.contains("git|"));
        assert!(output.contains("meta|"));
        assert!(output.contains("rtk|"));
        assert!(!output.contains("git-kb|"));
        assert!(!output.contains("grit|"));
        assert!(!output.contains("icm|"));
        assert!(output.contains("rtk|"));
        assert!(output.contains("init --global --codex --dry-run"));
    }

    // Defends: apply keeps local repo initialization separate from Meta fleet validation and uses only the bounded argv contract.
    #[test]
    fn agent_init_apply_uses_exact_bounded_argv_and_roots() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("local_repo");
        let meta = temp.path().join("meta");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&meta).unwrap();
        let (env, log) = fake_agent_init_environment(&temp, &repo);

        execute_agent_init(
            &agent_init_args(true, meta.clone(), Some(repo.clone())),
            &env,
        )
        .unwrap();

        let output = fs::read_to_string(log).unwrap();
        assert!(output.contains(&format!("git-kb|{}|init --no-verify", meta.display())));
        assert!(output.contains(&format!("git-kb|{}|init codex", meta.display())));
        assert!(output.contains(&format!(
            "grit|{}|-r {} init",
            repo.display(),
            repo.display()
        )));
        assert!(output.contains(&format!("icm|{}|init --mode cli --force", repo.display())));
        assert!(output.contains(&format!(
            "meta|{}|--dry-run exec -- git status --short --branch",
            meta.display()
        )));
        assert!(output.contains(&format!("rtk|{}|init --global --codex\n", repo.display())));
    }

    // Defends: missing profile/PATH tools stop initialization before any step can mutate state.
    #[test]
    fn agent_init_missing_tool_fails_clearly() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        let meta = temp.path().join("meta");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&meta).unwrap();
        let env = AgentInitEnvironment {
            path: temp.path().join("empty_bin").into_os_string(),
            cwd: repo,
        };

        let error = execute_agent_init(&agent_init_args(false, meta, None), &env).unwrap_err();

        assert_eq!(error.code(), "agent_init_missing_tool");
        assert!(error.message().contains("git-kb"));
    }

    // Defends: a failed bounded step returns a runtime error and prevents later setup steps from running.
    #[test]
    fn agent_init_stops_after_a_failing_step() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("local_repo");
        let meta = temp.path().join("meta");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&meta).unwrap();
        let (env, log) = fake_agent_init_environment(&temp, &repo);
        write_fake_executable(&temp.path().join("bin"), "grit", &repo, 23);

        let error = execute_agent_init(&agent_init_args(true, meta, Some(repo)), &env).unwrap_err();

        assert_eq!(error.code(), "agent_init_step_failed");
        assert_eq!(error.details()["step"], "Grit initialization");
        assert_eq!(error.details()["exit_code"], 23);
        let output = fs::read_to_string(log).unwrap();
        assert!(output.contains("grit|"));
        assert!(!output.lines().any(|line| line.starts_with("icm|")));
        assert!(!output.lines().any(|line| line.starts_with("meta|")));
        assert!(!output.lines().any(|line| line.starts_with("rtk|")));
    }

    // Defends: an existing Meta KB is verified and scaffold-checked in dry-run mode without conflating it with the explicitly selected local repository.
    #[test]
    fn agent_init_preview_validates_existing_meta_kb_separately_from_local_repo() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("local_repo");
        let meta = temp.path().join("meta");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(meta.join(".kb")).unwrap();
        let (env, log) = fake_agent_init_environment(&temp, &repo);

        execute_agent_init(
            &agent_init_args(false, meta.clone(), Some(repo.clone())),
            &env,
        )
        .unwrap();

        let output = fs::read_to_string(log).unwrap();
        assert!(output.contains(&format!("git-kb|{}|verify --full --json", meta.display())));
        assert!(output.contains(&format!("git-kb|{}|init codex --dry-run", meta.display())));
        assert!(output.contains(&format!(
            "meta|{}|--dry-run exec -- git status --short --branch",
            meta.display()
        )));
        assert!(!output.contains("grit|"));
        assert!(!output.contains("icm|"));
    }
}
