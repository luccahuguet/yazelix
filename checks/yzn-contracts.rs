use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SPONSOR_URL: &str = "https://github.com/sponsors/luccahuguet";

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzn, out] = args.as_slice() else {
        panic!("usage: yzn-contracts-check <yzn-package> <out>");
    };

    let yzn = Path::new(yzn);
    let config = fs::read_to_string(yzn.join("share/yazelix-next/config.kdl")).unwrap();
    let yzn_shell = default_shell(&config);
    assert!(
        yzn_shell.is_file(),
        "default_shell is not a file: {}",
        yzn_shell.display()
    );
    expect_shell_selection(&yzn_shell);
    expect_keybinds(&config);
    expect_first_party_plugins(&config);
    expect_front_door(yzn);
    expect_config_ui(yzn);
    expect_startup_diagnostics(yzn);
    expect_mars_config_override(yzn);
    expect_zellij_config_sidecar(yzn);
    expect_yazi_alt_z(yzn);

    let temp = TempDir::new();
    let user_config = temp.path.join("config");
    let user_nu = user_config.join("nu");
    let user_starship = user_config.join("starship.toml");
    let runtime = temp.path.join("run");
    fs::create_dir_all(&user_nu).unwrap();
    fs::write(
        user_nu.join("env.nu"),
        "$env.YZN_USER_ENV_TEST = \"env-ok\"\n",
    )
    .unwrap();
    fs::write(
        user_nu.join("config.nu"),
        "$env.YZN_USER_CONFIG_TEST = \"config-ok\"\n",
    )
    .unwrap();
    fs::write(
        &user_starship,
        "format = \"$character\"\nright_format = \"::<>\"\n",
    )
    .unwrap();

    let stdout = run_nu(
        &yzn_shell,
        &user_config,
        &runtime,
        "print $env.STARSHIP_SHELL; print $env.STARSHIP_CONFIG; print (do $env.PROMPT_COMMAND_RIGHT); print $env.YZN_USER_ENV_TEST; print $env.YZN_USER_CONFIG_TEST; ^carapace --version | ignore; ^zoxide --version | ignore; print ok",
    );
    assert_eq!(
        stdout,
        format!(
            "nu\n{}\n::<>\nenv-ok\nconfig-ok\nok",
            user_starship.display()
        )
    );
    let empty_config = temp.path.join("empty-config");
    fs::create_dir(&empty_config).unwrap();
    let fallback_starship = run_nu(
        &yzn_shell,
        &empty_config,
        &temp.path.join("empty-run"),
        "print $env.STARSHIP_CONFIG",
    );
    assert_ne!(fallback_starship, "ambient-starship.toml");
    assert!(
        fs::read_to_string(&fallback_starship).unwrap().is_empty(),
        "fallback Starship config is not empty: {fallback_starship}"
    );

    expect_line(
        &runtime.join("yazelix-next/nu/env.nu"),
        &format!("source-env \"{}\"", user_nu.join("env.nu").display()),
    );
    expect_line(
        &runtime.join("yazelix-next/nu/config.nu"),
        &format!("source \"{}\"", user_nu.join("config.nu").display()),
    );
    fs::write(out, "ok\n").unwrap();
}

fn expect_front_door(yzn: &Path) {
    let yzn_bin = yzn.join("bin/yzn");
    let help = run_help(&yzn_bin, &["help"]);
    for arg in ["-h", "--help"] {
        assert_eq!(run_help(&yzn_bin, &[arg]), help);
    }
    for expected in [
        "Usage:",
        "yzn config",
        "yzn doctor",
        "yzn enter [zellij-args...]",
        "yzn launch [zellij-args...]",
        "yzn menu",
        "yzn sponsor",
        "yzn status",
    ] {
        expect_contains(&help, expected, "yzn help");
    }
    let menu = run_help(&yzn_bin, &["menu"]);
    for expected in [
        "Yazelix Next Menu",
        "yzn doctor",
        "yzn status",
        "yzn sponsor",
        "Alt Shift L",
        "Alt z",
        "Codex resume",
    ] {
        expect_contains(&menu, expected, "yzn menu");
    }

    let yzn_launcher = binary_text(&yzn_bin);
    for expected in [
        "Yazelix could not start.",
        "YAZELIX_STATUS_BAR_CACHE_PATH",
        "ZELLIJ_PLUGIN_PERMISSIONS_CACHE",
        "YAZELIX_SESSION_TERMINAL",
        "bar.widgets",
        "popup.size",
        "yzn-bar-render",
        "yazelix_pane_orchestrator.wasm",
        "/bin/zellij",
        "/bin/mars",
        "tokenusage",
        "--new-session-with-layout",
    ] {
        expect_contains(&yzn_launcher, expected, "bin/yzn runtime fragment");
    }

    let temp = TempDir::new();
    let config_home = temp.path.join("status-config");
    let state_dir = temp.path.join("status-state");
    let doctor_config_home = temp.path.join("doctor-config");
    let doctor_state_dir = temp.path.join("doctor-state");
    let status = run_yzn_with_config(&yzn_bin, "status", &config_home, &state_dir, "yzn status");
    for expected in [
        "Yazelix status".to_string(),
        format!("config home: {}", config_home.display()),
        format!("state dir: {}", state_dir.display()),
        "shell: nu".to_string(),
        "open log: info".to_string(),
        r#"bar widgets: ["editor","shell","term","codex_usage","cpu","ram"]"#.to_string(),
        "popup size: 95".to_string(),
        "layout: packaged (/nix/store/".to_string(),
        "inside zellij: no".to_string(),
    ] {
        expect_contains(&status, &expected, "yzn status");
    }
    let data_home = temp.path.join("data-home");
    let data_status = successful_stdout(
        Command::new(&yzn_bin)
            .arg("status")
            .env("YAZELIX_NEXT_CONFIG_HOME", &config_home)
            .env("XDG_DATA_HOME", &data_home)
            .env_remove("YAZELIX_STATE_DIR"),
        "yzn status XDG data state",
    );
    expect_contains(
        &data_status,
        &format!("state dir: {}", data_home.join("yazelix-next").display()),
        "yzn status XDG data state",
    );

    let permissions = fs::read_to_string(state_dir.join("zellij/permissions.kdl")).unwrap();
    let runtime_config = fs::read_to_string(state_dir.join("zellij/config.kdl")).unwrap();
    let home = format!("{:?}", env::var("HOME").expect("HOME is required by yzn"));
    expect_contains(
        &runtime_config,
        &format!("cwd {home};"),
        "runtime new-tab config",
    );
    assert!(
        !runtime_config.contains("__YZN_HOME__"),
        "runtime config kept the unresolved home cwd placeholder"
    );
    for expected in [
        "yazelix_pane_orchestrator.wasm",
        "MessageAndLaunchOtherPlugins",
        "ReadSessionEnvironmentVariables",
    ] {
        expect_contains(&permissions, expected, "runtime plugin permissions");
    }

    let custom_popup_config = temp.path.join("custom-popup-config");
    let custom_popup_state = temp.path.join("custom-popup-state");
    write_config_home(
        &custom_popup_config,
        "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[popup]\nsize = 88\n",
    );
    let status = run_yzn_with_config(
        &yzn_bin,
        "status",
        &custom_popup_config,
        &custom_popup_state,
        "custom popup status",
    );
    expect_contains(&status, "popup size: 88", "custom popup status");
    expect_contains(&status, "zellij config: runtime (", "custom popup status");
    expect_contains(
        &status,
        "layout: packaged (/nix/store/",
        "custom popup status",
    );
    let custom_popup_config =
        fs::read_to_string(custom_popup_state.join("zellij/config.kdl")).unwrap();
    assert_eq!(custom_popup_config.matches("width_percent 88").count(), 4);
    assert_eq!(custom_popup_config.matches("height_percent 88").count(), 4);

    let custom_config = temp.path.join("custom-bar-config");
    let custom_state = temp.path.join("custom-bar-state");
    write_config_home(
        &custom_config,
        "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[bar]\nwidgets = [\"editor\", \"claude_usage\", \"cpu\"]\n",
    );
    let status = run_yzn_with_config(
        &yzn_bin,
        "status",
        &custom_config,
        &custom_state,
        "custom bar status",
    );
    expect_contains(
        &status,
        r#"bar widgets: ["editor","claude_usage","cpu"]"#,
        "custom bar status",
    );
    expect_contains(&status, "popup size: 95", "custom bar status");
    expect_contains(&status, "zellij config: runtime (", "custom bar status");
    expect_contains(&status, "layout: runtime (", "custom bar status");
    let custom_layout = fs::read_to_string(custom_state.join("zellij/layout.kdl")).unwrap();
    expect_contains(
        &custom_layout,
        r#"new_tab_template cwd="$HOME" {"#,
        "custom bar layout",
    );
    let format_right = custom_layout
        .lines()
        .find(|line| line.contains("format_right"))
        .expect("custom layout is missing format_right");
    expect_contains(format_right, "{command_claude_usage}", "custom bar layout");
    expect_contains(format_right, "{command_cpu}", "custom bar layout");
    assert!(
        !format_right.contains("{command_codex_usage}"),
        "custom visible bar kept a Codex widget omitted by bar.widgets"
    );
    let custom_swap = fs::read_to_string(custom_state.join("zellij/layout.swap.kdl")).unwrap();
    for expected in [
        "swap_tiled_layout name=\"single_open\"",
        "swap_tiled_layout name=\"single_closed\"",
        "pane name=\"sidebar\" command=\"/nix/store/",
        "stacked=true",
    ] {
        expect_contains(&custom_swap, expected, "custom bar swap layout");
    }
    assert!(
        !custom_swap.contains("@yazi@"),
        "custom bar swap layout kept the unresolved Yazi placeholder"
    );
    let custom_config = fs::read_to_string(custom_state.join("zellij/config.kdl")).unwrap();
    expect_contains(
        &custom_config,
        &format!(
            r#"layout "{}""#,
            custom_state.join("zellij/layout.kdl").display()
        ),
        "custom bar new-tab config",
    );
    expect_contains(
        &custom_config,
        &format!("cwd {home};"),
        "custom bar new-tab config",
    );

    let doctor = run_yzn_with_config(
        &yzn_bin,
        "doctor",
        &doctor_config_home,
        &doctor_state_dir,
        "yzn doctor",
    );
    for expected in [
        "Yazelix doctor".to_string(),
        format!("ok config home: {}", doctor_config_home.display()),
        "ok open.log_level: info".to_string(),
        r#"ok bar.widgets: ["editor","shell","term","codex_usage","cpu","ram"]"#.to_string(),
        "ok popup.size: 95".to_string(),
        "ok yazi opener: /nix/store/".to_string(),
        "ok pane orchestrator plugin: /nix/store/".to_string(),
        "warn session: not inside zellij".to_string(),
    ] {
        expect_contains(&doctor, &expected, "yzn doctor");
    }

    expect_sponsor_fallback(
        Command::new(&yzn_bin).arg("sponsor").env("PATH", ""),
        "without opener",
    );

    let fake_path = temp.path.join("fake-path");
    fs::create_dir(&fake_path).unwrap();
    let fake_xdg_open = fake_path.join("xdg-open");
    fs::write(
        &fake_xdg_open,
        "#!/bin/sh\necho noisy opener >&2\nexit 42\n",
    )
    .unwrap();
    fs::set_permissions(&fake_xdg_open, fs::Permissions::from_mode(0o755)).unwrap();
    expect_sponsor_fallback(
        Command::new(&yzn_bin)
            .arg("sponsor")
            .env("PATH", &fake_path),
        "with failing opener",
    );

    expect_command_error(
        &yzn_bin,
        &["doctor", "extra"],
        "yzn doctor does not accept arguments yet",
        "yzn doctor argument error",
    );
    expect_command_error(
        &yzn_bin,
        &["sponsor", "extra"],
        "yzn sponsor does not accept arguments yet",
        "yzn sponsor argument error",
    );
    expect_command_error(
        &yzn_bin,
        &["menu", "extra"],
        "yzn menu does not accept arguments yet",
        "yzn menu argument error",
    );
    expect_command_error(
        &yzn_bin,
        &["wat"],
        "yzn: unknown command: wat",
        "unknown yzn command error",
    );
    assert!(
        yzn.join("share/yazelix-next/runtime_identity.json")
            .is_file(),
        "yzn package is missing runtime_identity.json"
    );
}

fn expect_sponsor_fallback(command: &mut Command, context: &str) {
    let output = successful_output(command, &format!("yzn sponsor {context}"));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), SPONSOR_URL);
    assert!(
        output.stderr.is_empty(),
        "yzn sponsor {context} leaked stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn expect_command_error(yzn_bin: &Path, args: &[&str], expected: &str, context: &str) {
    let output = Command::new(yzn_bin).args(args).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(64),
        "yzn {args:?} should fail with usage status"
    );
    expect_contains(&String::from_utf8_lossy(&output.stderr), expected, context);
}

fn run_yzn_with_config(
    yzn_bin: &Path,
    command: &str,
    config_home: &Path,
    state_dir: &Path,
    context: &str,
) -> String {
    successful_stdout(
        Command::new(yzn_bin)
            .arg(command)
            .env("YAZELIX_NEXT_CONFIG_HOME", config_home)
            .env("YAZELIX_STATE_DIR", state_dir)
            .env_remove("ZELLIJ_SESSION_NAME"),
        context,
    )
}

fn write_config_home(config_home: &Path, contents: impl AsRef<[u8]>) -> PathBuf {
    fs::create_dir_all(config_home).unwrap();
    let config = config_home.join("config.toml");
    fs::write(&config, contents).unwrap();
    config
}

fn expect_config_ui(yzn: &Path) {
    let packaged_config = yzn.join("share/yazelix-next/config.toml");
    assert!(
        packaged_config.is_file(),
        "yzn package is missing config.toml"
    );
    let packaged_config = fs::read_to_string(&packaged_config).unwrap();
    for expected in [
        "log_level = \"info\"",
        "program = \"nu\"",
        "size = 95",
        "widgets = [\"editor\", \"shell\", \"term\", \"codex_usage\", \"cpu\", \"ram\"]",
    ] {
        expect_contains(&packaged_config, expected, "packaged config.toml");
    }

    let helper = yzn.join("libexec/yazelix-next/yzn-config");
    assert!(helper.is_file(), "missing yzn-config helper");
    let temp = TempDir::new();
    for (path, expected) in [
        ("open.log_level", "info"),
        ("shell.program", "nu"),
        ("popup.size", "95"),
        (
            "bar.widgets",
            r#"["editor","shell","term","codex_usage","cpu","ram"]"#,
        ),
    ] {
        let output = successful_stdout(
            Command::new(&helper)
                .arg("--get")
                .arg(path)
                .env("YAZELIX_NEXT_CONFIG_HOME", &temp.path),
            &format!("yzn-config --get {path}"),
        );
        assert_eq!(output.trim(), expected);
    }

    let unknown_temp = TempDir::new();
    let output = Command::new(&helper)
        .arg("--get")
        .arg("shell.typo")
        .env("YAZELIX_NEXT_CONFIG_HOME", &unknown_temp.path)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "unknown yzn-config --get path unexpectedly succeeded"
    );
    expect_contains(
        &String::from_utf8_lossy(&output.stderr),
        "unknown config path: shell.typo",
        "unknown yzn-config --get path",
    );
    assert!(
        !unknown_temp.path.join("config.toml").exists(),
        "unknown yzn-config --get path created config.toml"
    );

    let config = temp.path.join("config.toml");
    let config_text = fs::read_to_string(&config).unwrap();
    for expected in [
        "[open]",
        "log_level = \"info\"",
        "[shell]",
        "program = \"nu\"",
        "[popup]",
        "size = 95",
        "[bar]",
        "widgets = [\"editor\", \"shell\", \"term\", \"codex_usage\", \"cpu\", \"ram\"]",
        "contract_id = \"yazelix-next.config\"",
    ] {
        expect_contains(&config_text, expected, "created config.toml");
    }
}

fn expect_startup_diagnostics(yzn: &Path) {
    let yzn_bin = yzn.join("bin/yzn");
    let temp = TempDir::new();

    let sidecar_config = temp.path.join("sidecar-config");
    fs::create_dir_all(sidecar_config.join("zellij")).unwrap();
    let sidecar = sidecar_config.join("zellij/config.kdl");
    fs::write(&sidecar, "default_shell \"nu\"\n").unwrap();

    let bad_config = temp.path.join("bad-config");
    let config = write_config_home(
        &bad_config,
        "[open]\nlog_level = \"loud\"\n\n[shell]\nprogram = \"nu\"\n",
    );
    let bad_bar_config = temp.path.join("bad-bar-config");
    let bad_bar = write_config_home(
        &bad_bar_config,
        "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[bar]\nwidgets = [\"weather\"]\n",
    );
    let bad_popup_config = temp.path.join("bad-popup-config");
    let bad_popup = write_config_home(
        &bad_popup_config,
        "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[popup]\nsize = 101\n",
    );

    for (config_home, check, reason, label) in [
        (
            &sidecar_config,
            &sidecar,
            "forbidden Zellij sidecar item `default_shell`",
            "forbidden sidecar",
        ),
        (
            &bad_config,
            &config,
            "open.log_level must be one of: off, error, info, debug",
            "invalid config",
        ),
        (
            &bad_bar_config,
            &bad_bar,
            "bar.widgets must be one of: session, editor, shell, term, claude_usage, codex_usage, opencode_go_usage, cpu, ram.",
            "invalid bar widgets",
        ),
        (
            &bad_popup_config,
            &bad_popup,
            "popup.size must be between 1 and 100",
            "invalid popup size",
        ),
    ] {
        for command in ["enter", "status", "doctor"] {
            let runtime = temp.path.join(format!("{label}-{command}-runtime"));
            let (stdout, stderr) = run_startup_failure(&yzn_bin, command, config_home, &runtime);
            for expected in [
                "Yazelix could not start.",
                "Reason:",
                reason,
                "Check:",
                check.to_str().unwrap(),
            ] {
                expect_contains(&stderr, expected, &format!("{label} {command} diagnostic"));
            }
            if command == "doctor" {
                let context = format!("{label} doctor stdout");
                for expected in ["Yazelix doctor", "fail runtime preflight:"] {
                    expect_contains(&stdout, expected, &context);
                }
            }
        }
    }

    let state_file = temp.path.join("state-file");
    fs::write(&state_file, "").unwrap();
    let (stdout, stderr) = run_startup_failure(
        &yzn_bin,
        "doctor",
        &temp.path.join("state-config"),
        &state_file,
    );
    for expected in [
        "Yazelix could not start.",
        "Reason:",
        "failed to create",
        "Check:",
        state_file.to_str().unwrap(),
    ] {
        expect_contains(&stderr, expected, "unwritable state doctor diagnostic");
    }
    for expected in ["Yazelix doctor", "fail runtime preflight:"] {
        expect_contains(&stdout, expected, "unwritable state doctor stdout");
    }
}

fn run_startup_failure(
    yzn_bin: &Path,
    command: &str,
    config_home: &Path,
    runtime: &Path,
) -> (String, String) {
    if !runtime.exists() {
        fs::create_dir_all(runtime).unwrap();
    }
    let output = Command::new(yzn_bin)
        .arg(command)
        .env("YAZELIX_NEXT_CONFIG_HOME", config_home)
        .env("YAZELIX_STATE_DIR", runtime)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "yzn {command} unexpectedly succeeded with config {}\nstdout:\n{}\nstderr:\n{}",
        config_home.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_help(bin: &Path, args: &[&str]) -> String {
    successful_stdout(Command::new(bin).args(args), "yzn help")
}

fn run_nu(yzn_nu: &Path, config_home: &Path, runtime: &Path, commands: &str) -> String {
    fs::create_dir_all(runtime).unwrap();
    successful_stdout_trimmed(
        Command::new(yzn_nu)
            .arg("--commands")
            .arg(commands)
            .env("XDG_DATA_HOME", runtime)
            .env("YAZELIX_NEXT_CONFIG_HOME", config_home)
            .env_remove("YAZELIX_STATE_DIR")
            .env("STARSHIP_CONFIG", "ambient-starship.toml"),
        &yzn_nu.display().to_string(),
    )
}

fn expect_shell_selection(shell: &Path) {
    for program in ["bash", "zsh", "fish"] {
        let temp = TempDir::new();
        write_config_home(
            &temp.path,
            format!("[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"{program}\"\n"),
        );
        let output = successful_output(
            Command::new(shell)
                .arg("-c")
                .arg("echo shell-ok")
                .env("YAZELIX_NEXT_CONFIG_HOME", &temp.path),
            &format!("yzn-shell dispatch to {program}"),
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "shell-ok");
    }
}

fn expect_mars_config_override(yzn: &Path) {
    let packaged_config = yzn.join("share/yazelix-next/mars/config.toml");
    let yzn_bin = yzn.join("bin/yzn");
    assert!(
        packaged_config.is_file(),
        "packaged Mars config is not a file: {}",
        packaged_config.display()
    );

    let launcher = binary_text(&yzn_bin);
    for expected in [
        "YAZELIX_NEXT_CONFIG_HOME",
        "MARS_CONFIG_HOME",
        "yzn-mars-config",
    ] {
        expect_contains(&launcher, expected, "runtime Mars config override fragment");
    }

    let temp = TempDir::new();
    let config_home = temp.path.join("config");
    let mars_config = config_home.join("mars/config.toml");
    fs::create_dir_all(mars_config.parent().unwrap()).unwrap();
    fs::write(&mars_config, "# user Mars config\n").unwrap();

    let status = run_yzn_with_config(
        &yzn_bin,
        "status",
        &config_home,
        &temp.path.join("state"),
        "Mars config override status",
    );
    expect_contains(&status, "mars config: user", "Mars config override status");
    expect_contains(
        &status,
        &mars_config.display().to_string(),
        "Mars config override status",
    );
}

fn expect_zellij_config_sidecar(yzn: &Path) {
    let packaged_config = yzn.join("share/yazelix-next/config.kdl");
    let helper = yzn.join("libexec/yazelix-next/yzn-zellij-config");
    let temp = TempDir::new();
    let sidecar = temp.path.join("config.kdl");
    let generated_path = temp.path.join("generated.kdl");

    let no_sidecar = run_zellij_config(&helper, &packaged_config, &sidecar, &generated_path);
    assert_eq!(PathBuf::from(no_sidecar), packaged_config);

    let sidecar_config = "scroll_buffer_size 1234\npane_frames false\n";
    fs::write(&sidecar, sidecar_config).unwrap();
    let generated = run_zellij_config(&helper, &packaged_config, &sidecar, &generated_path);
    assert_eq!(PathBuf::from(&generated), generated_path);
    let packaged_text = fs::read_to_string(&packaged_config).unwrap();
    let expected_config = format!("{}\n{}", packaged_text.trim_end(), sidecar_config);
    assert_eq!(
        fs::read_to_string(&generated_path).unwrap(),
        expected_config
    );

    for forbidden in [
        ("keybinds", "keybinds {}\n"),
        (
            "support_kitty_keyboard_protocol",
            "support_kitty_keyboard_protocol false\n",
        ),
        ("env", "env { YZN_OPEN_LOG \"off\" }\n"),
    ] {
        fs::write(&sidecar, forbidden.1).unwrap();
        let output = Command::new(&helper)
            .arg(&packaged_config)
            .arg(&sidecar)
            .arg(&generated_path)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "dangerous Zellij sidecar unexpectedly succeeded for {}",
            forbidden.0
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(&format!("forbidden Zellij sidecar item `{}`", forbidden.0)),
            "unexpected Zellij sidecar rejection: {stderr}",
        );
    }
}

fn expect_yazi_alt_z(yzn: &Path) {
    let keymap = fs::read_to_string(yzn.join("share/yazelix-next/yazi/keymap.toml")).unwrap();
    for expected in [r#"on = ["<A-z>"]"#, r#"run = "plugin zoxide-editor""#] {
        expect_contains(&keymap, expected, "Yazi Alt-z keymap fragment");
    }

    let plugin =
        fs::read_to_string(yzn.join("share/yazelix-next/yazi/plugins/zoxide-editor.yazi/main.lua"))
            .unwrap();
    for expected in [
        r#"Command(yzn_open):arg(target_dir)"#,
        r#"Command("zoxide")"#,
        r#"emit("cd", { target_dir, raw = true })"#,
        "YZN_OPEN is not set",
    ] {
        expect_contains(&plugin, expected, "Yazi zoxide editor plugin fragment");
    }

    let layout = fs::read_to_string(yzn.join("share/yazelix-next/layout.kdl")).unwrap();
    let yzn_yazi = layout
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(r#"pane name="sidebar" command=""#)?
                .split('"')
                .next()
                .filter(|path| !path.is_empty())
                .map(PathBuf::from)
        })
        .expect("layout is missing sidebar yzn-yazi command");
    let wrapper = binary_text(&yzn_yazi);
    let context = format!("{} Yazi integration fragment", yzn_yazi.display());
    for expected in [
        "YZN_OPEN",
        "YZN_ZELLIJ",
        "YZN_EDITOR",
        "YAZI_CONFIG_HOME",
        "init.lua",
        "keymap.toml",
        "yazelix_starship.toml",
        "-- Yazelix Next user init.lua",
        "# Yazelix Next user keymap.toml",
        "YAZELIX_ZELLIJ_SESSION_NAME",
        "ZELLIJ_SESSION_NAME",
        "KITTY_WINDOW_ID",
        "git",
        "zoxide",
        "fzf",
    ] {
        expect_contains(&wrapper, expected, &context);
    }
}

fn run_zellij_config(
    helper: &Path,
    packaged_config: &Path,
    sidecar: &Path,
    generated: &Path,
) -> String {
    successful_stdout_trimmed(
        Command::new(helper)
            .arg(packaged_config)
            .arg(sidecar)
            .arg(generated),
        &helper.display().to_string(),
    )
}

fn default_shell(config: &str) -> PathBuf {
    config
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("default_shell \"")?
                .strip_suffix('"')
                .map(PathBuf::from)
        })
        .expect("missing default_shell")
}

fn expect_keybinds(config: &str) {
    for expected in [
        r#"unbind "Alt n" "Ctrl g""#,
        r#"bind "Alt m" { NewPane; }"#,
        r#"bind "Alt h" "Alt Left" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_left_or_tab"; }; }"#,
        r#"bind "Alt l" "Alt Right" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_right_or_tab"; }; }"#,
        r#"bind "Alt Shift h" { NextSwapLayout; }"#,
        r#"bind "Ctrl Alt g" { SwitchToMode "Locked"; }"#,
        r#"bind "Ctrl p" { SwitchToMode "Pane"; }"#,
        r#"bind "Ctrl t" { SwitchToMode "Tab"; }"#,
        r#"bind "Ctrl n" { SwitchToMode "Resize"; }"#,
        r#"bind "Ctrl Alt s" { SwitchToMode "Scroll"; }"#,
        r#"bind "Ctrl Alt o" { SwitchToMode "Session"; }"#,
        r#"bind "Ctrl q" { Quit; }"#,
        r#"unbind "Ctrl h""#,
    ] {
        assert!(
            config.lines().any(|line| line.trim() == expected),
            "config.kdl is missing {expected}",
        );
    }
    assert!(
        config.lines().any(|line| {
            let line = line.trim();
            line.starts_with(r#"bind "n" { NewTab { layout "/nix/store/"#)
                && line
                    .ends_with(r#"/layout.kdl"; cwd "__YZN_HOME__"; }; SwitchToMode "Normal"; }"#)
        }),
        "config.kdl must create new tabs from the packaged layout with a runtime home cwd",
    );
    expect_no_block_binds_and_unbinds_same_key(config);
    assert!(
        !config.contains(r#"SwitchToMode "Move""#),
        "config.kdl must not reintroduce move mode"
    );
    assert!(
        !config.contains("MoveFocusOrTab"),
        "Alt h/l must use the pane orchestrator instead of native MoveFocusOrTab"
    );
}

fn expect_first_party_plugins(config: &str) {
    for expected in [
        "share/yazelix_zellij_popup/yzpp.wasm",
        "share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm",
        r#"yazelix_pane_orchestrator location="file:/nix/store/"#,
        "load_plugins",
        "support_kitty_keyboard_protocol true",
        "screen_saver_enabled false",
    ] {
        expect_contains(config, expected, "config.kdl first-party plugin fragment");
    }
    for (id, pane_title, command_suffix, extra) in [
        ("config", "config_popup", "/bin/yzn-config-ui", ""),
        (
            "agent",
            "agent_popup",
            "/bin/yzn-agent",
            "\n                toggle_close_behavior \"hide\"",
        ),
        ("lazygit", "lazygit_popup", "/bin/lazygit", ""),
        ("menu", "menu_popup", "/bin/yzn-menu-popup", ""),
    ] {
        let command = popup_command(config, command_suffix);
        let expected = format!(
            "{id} {{\n                command \"{}\"\n                pane_title \"{pane_title}\"\n                width_percent 95\n                height_percent 95{extra}\n            }}",
            command.display()
        );
        assert!(
            config.contains(&expected),
            "config.kdl is missing {id} popup block\n{expected}",
        );
    }
    assert_eq!(config.matches("width_percent 95").count(), 4);
    assert_eq!(config.matches("height_percent 95").count(), 4);
    for (key, payload) in [
        ("Alt Shift J", "lazygit"),
        ("Alt Shift K", "config"),
        ("Alt Shift L", "agent"),
        ("Alt Shift M", "menu"),
    ] {
        let expected = format!(
            "bind \"{key}\" {{\n            MessagePlugin \"yzpp\" {{\n                name \"toggle\"\n                payload \"{payload}\"\n            }}\n        }}"
        );
        assert!(
            config.contains(&expected),
            "config.kdl is missing {key} popup binding\n{expected}",
        );
    }

    let agent = popup_command(config, "/bin/yzn-agent");
    let agent_script = fs::read_to_string(&agent).unwrap();
    let context = format!("{} guarded Codex fragment", agent.display());
    expect_contains(&agent_script, "command -v codex", &context);
    expect_contains(&agent_script, "codex is not available on PATH", &context);
    expect_contains(&agent_script, "exec codex resume", &context);
    let output = Command::new(&agent).env("PATH", "").output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(127),
        "agent popup without codex should exit 127, got {:?}",
        output.status.code(),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("codex is not available on PATH"),
        "agent popup missing-codex output is unclear: {stderr}",
    );

    let config_ui = popup_command(config, "/bin/yzn-config-ui");
    let config_ui_script = fs::read_to_string(&config_ui).unwrap();
    let context = format!("{} managed editor wrapper", config_ui.display());
    for expected in ["YAZELIX_NEXT_EDITOR=", "/bin/yzn-hx", "/bin/yzn-config"] {
        expect_contains(&config_ui_script, expected, &context);
    }

    let menu_popup = popup_command(config, "/bin/yzn-menu-popup");
    let menu_popup_script = fs::read_to_string(&menu_popup).unwrap();
    assert!(
        menu_popup_script.contains("/bin/yzn-menu"),
        "{} does not delegate to yzn-menu",
        menu_popup.display(),
    );
}

fn popup_command(config: &str, suffix: &str) -> PathBuf {
    config
        .lines()
        .find_map(|line| {
            let command = line.trim().strip_prefix("command \"")?.strip_suffix('"')?;
            command.ends_with(suffix).then(|| PathBuf::from(command))
        })
        .unwrap_or_else(|| panic!("config.kdl is missing popup command ending in {suffix}"))
}

fn expect_no_block_binds_and_unbinds_same_key(config: &str) {
    let mut blocks = Vec::<KeyBlock>::new();
    for (line_number, line) in config.lines().map(str::trim).enumerate() {
        if opens_keybind_block(line) {
            blocks.push(KeyBlock::default());
        }
        if let Some(block) = blocks.last_mut() {
            if line.starts_with("bind ") {
                block.binds.extend(quoted_keys(line));
            } else if line.starts_with("unbind ") {
                block.unbinds.extend(quoted_keys(line));
            }
            for key in block.binds.iter().filter(|key| block.unbinds.contains(key)) {
                panic!(
                    "config.kdl binds and unbinds {key} in the same block near line {}",
                    line_number + 1
                );
            }
        }
        if line == "}" {
            blocks.pop();
        }
    }
}

fn opens_keybind_block(line: &str) -> bool {
    line.ends_with('{') && !line.starts_with("bind ")
}

fn quoted_keys(line: &str) -> impl Iterator<Item = String> + '_ {
    line.split('"').skip(1).step_by(2).map(str::to_string)
}

fn successful_output(command: &mut Command, context: &str) -> Output {
    let output = command.output().unwrap();
    assert!(
        output.status.success(),
        "{context} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn successful_stdout(command: &mut Command, context: &str) -> String {
    String::from_utf8_lossy(&successful_output(command, context).stdout).into_owned()
}

fn successful_stdout_trimmed(command: &mut Command, context: &str) -> String {
    successful_stdout(command, context)
        .trim_end_matches('\n')
        .to_owned()
}

fn expect_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context} is missing {needle:?}\n{}",
        excerpt(haystack)
    );
}

fn excerpt(text: &str) -> String {
    const LIMIT: usize = 4000;
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(LIMIT).collect();
    let omitted = chars.count();
    if omitted == 0 {
        head
    } else {
        format!("{head}...\n[{omitted} chars omitted]")
    }
}

fn binary_text(path: &Path) -> String {
    String::from_utf8_lossy(&fs::read(path).unwrap()).into_owned()
}

#[derive(Default)]
struct KeyBlock {
    binds: Vec<String>,
    unbinds: Vec<String>,
}

fn expect_line(path: &Path, expected: &str) {
    let contents = fs::read_to_string(path).unwrap();
    assert!(
        contents.lines().any(|line| line == expected),
        "{} does not contain {expected}",
        path.display()
    );
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let mut path = env::temp_dir();
        path.push(format!(
            "yzn-contracts-{}-{}",
            std::process::id(),
            unix_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn unix_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
