use std::{
    env, fs,
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

mod support;

use support::{
    RuntimeCase, TempDir, binary_text, embedded_store_path, excerpt, expect_contains, expect_order,
    successful_output, successful_stdout, write_config_home, write_executable,
};

macro_rules! expect_contains_all {
    ($haystack:expr, $context:expr; $($needle:expr),+ $(,)?) => {
        $(expect_contains($haystack, &$needle, $context);)+
    };
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzx, git, jq, out] = args.as_slice() else {
        panic!("usage: yzx-contracts-check <yzx-package> <git> <jq> <out>");
    };

    let yzx = Path::new(yzx);
    let git = Path::new(git);
    expect_read_only_diagnostics(yzx);
    let config = fs::read_to_string(yzx.join("share/yazelix/config.kdl")).unwrap();
    let yzx_shell = default_shell(&config);
    assert!(
        yzx_shell.is_file(),
        "default_shell is not a file: {}",
        yzx_shell.display()
    );
    expect_session_config(&config);
    expect_shell_selection(&yzx_shell);
    expect_keybinds(&config);
    expect_first_party_plugins(git, &config);
    expect_front_door(yzx, Path::new(jq));
    expect_headless_enter(yzx);
    expect_narrow_path_launches(yzx, &yzx_shell);
    expect_config_ui(yzx);
    expect_startup_diagnostics(yzx);
    expect_rio_config(yzx);
    expect_zellij_config_sidecar(yzx);
    expect_yazi_managed_keys(yzx);

    let temp = TempDir::new();
    let user_config = temp.path.join("config");
    let user_nu = user_config.join("nu");
    let user_starship = user_config.join("starship.toml");
    let runtime = temp.path.join("run");
    fs::create_dir_all(&user_nu).unwrap();
    fs::write(
        user_nu.join("env.nu"),
        "$env.YZX_USER_ENV_TEST = \"env-ok\"\n",
    )
    .unwrap();
    fs::write(
        user_nu.join("config.nu"),
        "$env.YZX_USER_CONFIG_TEST = \"config-ok\"\n",
    )
    .unwrap();
    fs::write(
        &user_starship,
        "right_format = \"::<>\"\n\n[character]\nformat = \">> \"\n",
    )
    .unwrap();

    let stdout = run_nu(
        &yzx_shell,
        &user_config,
        &runtime,
        "print $env.STARSHIP_SHELL; print $env.STARSHIP_CONFIG; print (do $env.PROMPT_COMMAND_RIGHT); print ((^starship print-config) | str contains 'format = \"$all\"'); print $env.YZX_USER_ENV_TEST; print $env.YZX_USER_CONFIG_TEST; print ('ATUIN_SESSION' in $env); print ($env.config.keybindings | where name == 'atuin' | length); print ($env.config.keybindings | where name == 'atuin' | get keycode.0); print ($env.config.hooks.pre_execution | length); print ($env.config.hooks.pre_prompt | length); print (do $env.config.completions.external.completer [git che] | where display == 'checkout' | length); ^zoxide --version | ignore; ^atuin --version",
    );
    assert_eq!(
        stdout,
        format!(
            "nu\n{}\n::<>\ntrue\nenv-ok\nconfig-ok\ntrue\n1\nchar_r\n1\n1\n1\natuin 18.16.1 (NO_GIT)",
            runtime.join("yazelix/starship.toml").display()
        )
    );
    let effective_starship = fs::read_to_string(runtime.join("yazelix/starship.toml")).unwrap();
    expect_contains_all! {
        &effective_starship, "effective user Starship config";
        "right_format = \"::<>\"",
        "[character]",
        "format = \">> \"",
    }
    let empty_config = temp.path.join("empty-config");
    fs::create_dir(&empty_config).unwrap();
    let fallback_starship = run_nu(
        &yzx_shell,
        &empty_config,
        &temp.path.join("empty-run"),
        "print $env.STARSHIP_CONFIG",
    );
    assert_ne!(fallback_starship, "ambient-starship.toml");
    let fallback_starship = fs::read_to_string(&fallback_starship).unwrap();
    assert_eq!(fallback_starship, "[character]\nformat = \":: \"\n");

    expect_line(
        &runtime.join("yazelix/nu/env.nu"),
        &format!("source-env \"{}\"", user_nu.join("env.nu").display()),
    );
    expect_line(
        &runtime.join("yazelix/nu/config.nu"),
        &format!("source \"{}\"", user_nu.join("config.nu").display()),
    );
    let host_bin = temp.path.join("host-bin");
    fs::create_dir(&host_bin).unwrap();
    let fake_mise = host_bin.join("mise");
    write_executable(
        &fake_mise,
        "#!/bin/sh\n[ \"$1\" = activate ] && [ \"$2\" = nu ] || exit 64\nprintf '%s\\n' '$env.YZX_MISE_TEST = \"mise-ok\"'\n",
    );
    let mise_runtime = temp.path.join("mise-run");
    let mise_stdout = run_nu_with_path(
        &yzx_shell,
        &user_config,
        &mise_runtime,
        "print $env.YZX_MISE_TEST",
        &host_bin,
    );
    assert_eq!(mise_stdout, "mise-ok");
    expect_line(
        &mise_runtime.join("yazelix/nu/config.nu"),
        "$env.YZX_MISE_TEST = \"mise-ok\"",
    );
    let generated_mise_config =
        fs::read_to_string(mise_runtime.join("yazelix/nu/config.nu")).unwrap();
    let user_config_source = format!("source \"{}\"", user_nu.join("config.nu").display());
    expect_order(
        &generated_mise_config,
        &[
            "source \"/nix/store/",
            "$env.YZX_MISE_TEST = \"mise-ok\"",
            &user_config_source,
            "managed Atuin init failed",
        ],
        "managed Nu mise layering",
    );

    let atuin = PathBuf::from(run_nu(
        &yzx_shell,
        &empty_config,
        &temp.path.join("atuin-path-run"),
        "which atuin | first | get path | print",
    ));
    assert!(
        atuin.starts_with("/nix/store") && atuin.ends_with("bin/atuin"),
        "managed Nu resolved non-packaged Atuin: {}",
        atuin.display()
    );
    let user_atuin_init = user_nu.join("atuin.nu");
    fs::write(
        &user_atuin_init,
        successful_stdout(
            Command::new(&atuin)
                .args(["init", "nu", "--disable-up-arrow"])
                .env("HOME", &temp.path)
                .env("XDG_CONFIG_HOME", &temp.path),
            "packaged atuin init nu",
        ),
    )
    .unwrap();
    fs::write(
        user_nu.join("config.nu"),
        format!(
            "source \"{}\"\n$env.YZX_USER_CONFIG_TEST = \"config-ok\"\n",
            user_atuin_init.display()
        ),
    )
    .unwrap();
    let user_atuin_stdout = run_nu(
        &yzx_shell,
        &user_config,
        &temp.path.join("user-atuin-run"),
        "print $env.YZX_USER_CONFIG_TEST; print ($env.config.keybindings | where name == 'atuin' | length); print ($env.config.hooks.pre_execution | length); print ($env.config.hooks.pre_prompt | length)",
    );
    assert_eq!(user_atuin_stdout, "config-ok\n1\n1\n1");

    let native_config = temp.path.join("native-config");
    write_config_home(&native_config, "[shell]\nprogram = \"nu\"\natuin = false\n");
    let native_stdout = run_nu(
        &yzx_shell,
        &native_config,
        &temp.path.join("native-run"),
        "print ('ATUIN_SESSION' in $env); print ($env.config.keybindings | where name == 'atuin' | length); print ($env.config.hooks.pre_execution? | default [] | length); print ($env.config.hooks.pre_prompt? | default [] | length); print ($nu | get history-enabled? | default true)",
    );
    assert_eq!(native_stdout, "false\n0\n0\n0\ntrue");

    let nobind_config = temp.path.join("nobind-config");
    let nobind_nu = nobind_config.join("nu");
    fs::create_dir_all(&nobind_nu).unwrap();
    fs::write(nobind_nu.join("config.nu"), "$env.ATUIN_NOBIND = \"1\"\n").unwrap();
    let nobind_stdout = run_nu(
        &yzx_shell,
        &nobind_config,
        &temp.path.join("nobind-run"),
        "print ('ATUIN_SESSION' in $env); print ($env.config.keybindings | where name == 'atuin' | length); print ($env.config.hooks.pre_execution | length); print ($env.config.hooks.pre_prompt | length)",
    );
    assert_eq!(nobind_stdout, "true\n0\n1\n1");
    fs::write(out, "ok\n").unwrap();
}

fn expect_front_door(yzx: &Path, jq: &Path) {
    let yzx_bin = yzx.join("bin/yzx");
    expect_radar_setup(&yzx_bin);
    let help = run_help(&yzx_bin, &["help"]);
    for arg in ["-h", "--help"] {
        assert_eq!(run_help(&yzx_bin, &[arg]), help);
    }
    let version = run_help(&yzx_bin, &["--version"]);
    assert_eq!(run_help(&yzx_bin, &[]), help);
    expect_contains_all! {
        &help, "yzx help";
        "Yazelix Nova",
        "Usage:",
        "yzx --version",
        "yzx config",
        "yzx yazi-config materialize --user-config-dir <path> --state-dir <path>",
        "yzx doctor",
        "yzx radar-setup",
        "yzx env",
        "yzx enter [zellij-args...]",
        "yzx launch [zellij-args...]",
        "yzx enter --session NAME",
        "yzx launch --session NAME",
        "yzx enter attach NAME",
        "yzx launch attach NAME",
        "yzx menu",
        "yzx tutor [lesson]",
        "yzx reveal <target>",
        "yzx anima [style]",
        "yzx run <program> [args...]",
        "yzx status [--json]",
        "https://github.com/sponsors/luccahuguet",
    }
    let menu = run_help(&yzx_bin, &["menu"]);
    expect_contains(&menu, "Yazelix Nova command palette", "yzx menu");
    let menu_ids = menu
        .lines()
        .filter_map(|line| {
            let (_, command) = line.trim_start().split_once('.')?;
            command.split_whitespace().next()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        menu_ids,
        [
            "config",
            "doctor",
            "status",
            "anima",
            "launch",
            "help",
            "tutor",
            "radar-setup"
        ],
        "yzx menu command allowlist changed\n{menu}"
    );
    expect_menu_descriptions_match_help(&help, &menu);
    for forbidden in [
        "yzx env",
        "yzx enter",
        "yzx reveal",
        "Alt Shift",
        "Ctrl Alt",
        "Git popup",
        "Agent popup",
    ] {
        assert!(
            !menu.contains(forbidden),
            "yzx menu exposes non-allowlisted reference `{forbidden}`\n{menu}"
        );
    }
    let reveal_help = run_help(&yzx_bin, &["reveal", "--help"]);
    expect_contains(&reveal_help, "yzx reveal <target>", "yzx reveal help");
    let anima_help = run_help(&yzx_bin, &["anima", "--help"]);
    expect_contains_all! {
        &anima_help, "yzx anima help";
        "yzx anima [STYLE]",
        "static",
        "logo",
        "asciiquarium",
        "aquarium",
        "boids_schools",
        "friends_and_enemies",
        "primordial",
        "physarum",
        "chladni",
        "plasma",
        "game_of_life_gliders",
        "game_of_life_tumblers",
        "mandelbrot",
        "matrix",
        "random",
        "--cell-style",
        "--duration-seconds",
        "Animations, including Aquarium: Left/h/p = previous; Right/l/n = next",
    }
    let temp = TempDir::new();
    let styles = anima_help.split_once("Styles:\n").unwrap().1;
    for style in styles.split_once("\nNotes:").unwrap().0.split_whitespace() {
        write_config_home(&temp.path, format!("[welcome]\nstyle = {style:?}\n"));
        let selected = successful_stdout(
            Command::new(yzx.join("libexec/yazelix/yzx-config"))
                .args(["--get", "welcome.style"])
                .env("YAZELIX_CONFIG_HOME", &temp.path),
            &format!("welcome accepts packaged Anima style {style}"),
        );
        assert_eq!(selected.trim(), style);
    }
    let tutor_help = run_help(&yzx_bin, &["tutor", "--help"]);
    expect_contains_all! {
        &tutor_help, "yzx tutor help";
        "yzx tutor",
        "yzx tutor begin",
        "yzx tutor list",
        "yzx tutor workspace",
        "yzx tutor files",
        "yzx tutor panes",
        "yzx tutor modes",
        "yzx tutor discovery",
        "yzx tutor troubleshooting",
        "yzx tutor tool_tutors",
        "yzx tutor hx",
        "yzx tutor helix",
        "yzx tutor nu",
        "yzx tutor nushell",
    }
    let tutor_root = run_help(&yzx_bin, &["tutor"]);
    expect_contains_all! {
        &tutor_root, "yzx tutor";
        "Yazelix Nova tutor",
        "yzx tutor begin",
        "yzx tutor list",
    }
    let tutor_list = run_help(&yzx_bin, &["tutor", "list"]);
    expect_contains_all! {
        &tutor_list, "yzx tutor list";
        "yzx tutor workspace",
        "yzx tutor files",
        "yzx tutor panes",
        "yzx tutor modes",
        "yzx tutor discovery",
        "yzx tutor troubleshooting",
        "yzx tutor tool_tutors",
    }
    for (lesson, expected) in [
        ("begin", "Start in the right directory"),
        ("workspace", "current tab workspace root matters most"),
        ("files", "full Yazi popup"),
        ("panes", "move the current tab"),
        ("modes", "quit the session"),
        ("discovery", "Alt Shift M"),
        ("troubleshooting", "yzx doctor"),
        ("tool_tutors", "print the managed Helix tutor command"),
    ] {
        let output = run_help(&yzx_bin, &["tutor", lesson]);
        expect_contains(&output, expected, &format!("yzx tutor {lesson}"));
        assert!(
            !output.contains("env --no-shell") && !output.contains("launch --path"),
            "yzx tutor {lesson} leaked unsupported command syntax\n{}",
            excerpt(&output)
        );
    }
    let helix_tutor = run_help(&yzx_bin, &["tutor", "hx"]);
    expect_contains_all! {
        &helix_tutor, "yzx tutor hx";
        "/bin/yzx-hx --tutor",
        "yzx-hx --tutor",
        "package omits managed Helix",
    }
    let nushell_tutor = run_help(&yzx_bin, &["tutor", "nu"]);
    expect_contains_all! {
        &nushell_tutor, "yzx tutor nu";
        "/bin/nu -c 'tutor begin'",
        "tutor begin",
    }

    let yzx_launcher = binary_text(&yzx_bin);
    let menu_helper = embedded_store_path(&yzx_launcher, "/bin/yzx-menu");
    let zellij = embedded_store_path(&yzx_launcher, "/bin/zellij");
    let popup_wasm = embedded_store_path(&yzx_launcher, "share/yazelix_zellij_popup/yzpp.wasm");
    let packaged_zellij_config = fs::read_to_string(yzx.join("share/yazelix/config.kdl")).unwrap();
    let radar_wasm = packaged_zellij_config
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(r#"radar location="file:"#)?
                .strip_suffix(r#"" {"#)
                .map(PathBuf::from)
        })
        .expect("packaged Zellij config is missing the Radar alias");
    assert!(radar_wasm.is_file(), "packaged Radar WASM is missing");
    assert!(
        yzx.join("bin/zj-radar").is_file(),
        "packaged Radar CLI is missing"
    );
    for license in [
        "share/licenses/zj-radar/LICENSE",
        "share/licenses/yazelix-forest/LICENSE",
        "share/licenses/notify-hx/LICENSE",
        "share/licenses/glyph-hx/LICENSE",
    ] {
        assert!(
            yzx.join(license).is_file(),
            "full package is missing dependency license {license}"
        );
    }
    expect_menu_dispatch(&menu_helper);
    expect_contains_all! {
        &yzx_launcher, "bin/yzx runtime fragment";
        "Yazelix Nova could not start.",
        "YAZELIX_STATUS_BAR_CACHE_PATH",
        "ZELLIJ_PLUGIN_PERMISSIONS_CACHE",
        "YAZELIX_SESSION_TERMINAL",
        "YZX_WELCOME_ENABLED",
        "YZX_WELCOME_STYLE",
        "YZX_WELCOME_DURATION_SECONDS",
        "YZX_RADAR_ENABLED",
        "YZX_APPEARANCE_MODE",
        "YZX_APPEARANCE_LIVE",
        "YZX_MENU_YZX",
        "YZX_YA",
        "YZX_ZELLIJ",
        "appearance.mode",
        "welcome.enabled",
        "welcome.style",
        "welcome.duration_seconds",
        "YAZELIX_EDITOR",
        "YZX_EDITOR",
        "GIT_EDITOR",
        "editor.command",
        "agent.command",
        "agent.args",
        "agent.popup.kdl",
        "sidebar.command",
        "sidebar.args",
        "sidebar.pane.kdl",
        "bar.widgets",
        "popup.side_margin",
        "popup.vertical_margin",
        "popups.kdl",
        "popups.keybindings.kdl",
        "keybindings.config",
        "keybindings.agent",
        "keybindings.git",
        "keybindings.menu",
        "keybindings.screen",
        "keybindings.sidebar",
        "keybindings.sidebar_focus",
        "lazygit",
        "yzx-bar-render",
        "yzx-env-supervisor",
        "yzx-tutor",
        "yzx-welcome",
        "yzx-shell",
        "yzx-reveal",
        "/bin/anima",
        "yazelix_pane_orchestrator.wasm",
        "/bin/ya",
        "/bin/zellij",
        "/bin/rio",
        "tokenusage",
        "--theme-mode",
        "--project-rio-appearance",
        "--new-session-with-layout",
    }
    let env_supervisor = embedded_store_path(&yzx_launcher, "/bin/yzx-env-supervisor");
    let env_supervisor_script = fs::read_to_string(&env_supervisor).unwrap();
    expect_contains_all! {
        &env_supervisor_script, "yzx env supervisor";
        "#!/nix/store/",
        "trap cleanup HUP INT TERM EXIT",
        "\"$1\" < /dev/tty &",
        "wait \"$child\"",
    }

    let temp = TempDir::new();
    let status_case = RuntimeCase::new(&temp.path, "status");
    let doctor_case = RuntimeCase::new(&temp.path, "doctor");
    fs::create_dir_all(status_case.zellij_path("permissions.kdl").parent().unwrap()).unwrap();
    fs::write(
        status_case.zellij_path("permissions.kdl"),
        format!(
            "\"{0}\" {{\n    ReadApplicationState\n    ChangeApplicationState\n    OpenTerminalsOrPlugins\n    RunCommands\n    ReadCliPipes\n}}\n\"{0}\" {{\n}}\n\"third-party.wasm\" {{\n    WebAccess\n}}\n",
            popup_wasm.display()
        ),
    )
    .unwrap();
    let status = status_case.prepared_status(&yzx_bin, "yzx status");
    expect_contains_all! {
        &status, "yzx status";
        "Yazelix Nova status",
        "package: full",
        format!("config home: {}", status_case.config_home.display()),
        format!("state dir: {}", status_case.state_dir.display()),
        "shell: nu",
        "editor command: yzx-hx",
        "editor: /nix/store/",
        "agent command: auto",
        "agent args: []",
        "sidebar command: radar",
        "sidebar args: []",
        "open log: info",
        "welcome enabled: true",
        "welcome style: random",
        "welcome duration: 3s",
        r#"bar widgets: ["editor","shell","term","codex_usage","cpu","ram"]"#,
        "popup side margin: 1",
        "popup vertical margin: 0",
        "config keybinding: Alt Shift K",
        "agent keybinding: Alt Shift L",
        "git keybinding: Alt Shift J",
        "menu keybinding: Alt Shift M",
        "screen keybinding: Alt Shift A",
        "sidebar keybinding: Alt Shift H",
        "forest keybinding: Ctrl y",
        "layout: packaged (/nix/store/",
        "yazi source: bundled",
        "yazi: /nix/store/",
        "ya: /nix/store/",
        "yazi version: ",
        "inside zellij: no",
    }

    let json_case = RuntimeCase::new(&temp.path, "json-\"\\\n");
    let json = successful_stdout(
        json_case.yzx_command(&yzx_bin, "status").arg("--json"),
        "yzx status --json",
    );
    assert_eq!(
        jq_output(jq, ".config_home", &json),
        json_case.config_home.to_string_lossy()
    );
    assert_eq!(
        jq_output(jq, ".state_dir", &json),
        json_case.state_dir.to_string_lossy()
    );
    assert_eq!(
        jq_output(
            jq,
            ".schema_version == 1 and .inside_zellij == false",
            &json
        ),
        "true"
    );
    assert_eq!(
        jq_output(jq, "keys | sort | join(\",\")", &json),
        "agent_command,config_home,editor,editor_command,inside_zellij,name,package,schema_version,shell,state_dir,version"
    );

    let run_child = temp.path.join("run-child");
    write_executable(
        &run_child,
        "#!/bin/sh\nprintf 'arg=<%s>\\n' \"$@\"\nprintf 'config=<%s>\\n' \"$YAZELIX_CONFIG_HOME\"\nprintf 'editor=<%s>\\n' \"$EDITOR\"\nexit 23\n",
    );
    let run_case = RuntimeCase::new(&temp.path, "run");
    let output = run_case
        .yzx_command(&yzx_bin, "run")
        .args([
            run_child.as_os_str(),
            "alpha beta".as_ref(),
            "quote\"slash\\".as_ref(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(23));
    let run_record = String::from_utf8_lossy(&output.stdout);
    expect_contains_all! {
        &run_record, "yzx run environment";
        "arg=<alpha beta>",
        "arg=<quote\"slash\\>",
        format!("config=<{}>", run_case.config_home.display()),
        "editor=</nix/store/",
        "/bin/yzx-editor>",
    }
    let data_home = temp.path.join("data-home");
    let data_status = successful_stdout(
        Command::new(&yzx_bin)
            .arg("status")
            .env("YAZELIX_CONFIG_HOME", &status_case.config_home)
            .env("XDG_DATA_HOME", &data_home)
            .env_remove("YAZELIX_STATE_DIR"),
        "yzx status XDG data state",
    );
    expect_contains(
        &data_status,
        &format!("state dir: {}", data_home.join("yazelix").display()),
        "yzx status XDG data state",
    );

    let runtime_config = status_case.zellij_file("config.kdl");
    let home = format!("{:?}", env::var("HOME").expect("HOME is required by yzx"));
    expect_contains(
        &runtime_config,
        &format!("cwd {home};"),
        "runtime new-tab config",
    );
    assert!(
        !runtime_config.contains("__YZX_HOME__"),
        "runtime config kept the unresolved home cwd placeholder"
    );
    expect_contains_all! {
        &runtime_config, "runtime bar controller";
        "// BEGIN NOVA BAR CONTROLLER",
        r#"role "controller""#,
        "// END NOVA BAR CONTROLLER",
    }
    let permissions = status_case.zellij_file("permissions.kdl");
    expect_contains_all! {
        &permissions, "runtime plugin permissions";
        "\"third-party.wasm\" {\n    WebAccess\n}",
        "share/yazelix_zellij_popup/yzpp.wasm\" {",
        "share/nova_bar/zjstatus.wasm\" {",
        &format!(
            "\"{}\" {{\n    ReadApplicationState\n    ChangeApplicationState\n    RunCommands\n    ReadCliPipes\n    MessageAndLaunchOtherPlugins\n}}",
            radar_wasm.display()
        ),
        "share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm\" {",
        "WriteToStdin",
        "ReadSessionEnvironmentVariables",
        "MessageAndLaunchOtherPlugins",
    }
    for (permission, count) in [
        ("ReadApplicationState", 5),
        ("ChangeApplicationState", 5),
        ("RunCommands", 5),
        ("OpenTerminalsOrPlugins", 3),
        ("ReadCliPipes", 4),
        ("MessageAndLaunchOtherPlugins", 3),
    ] {
        assert_eq!(
            permissions.matches(permission).count(),
            count,
            "runtime plugin permissions have the wrong {permission} grants\n{permissions}"
        );
    }

    let custom_sidebar = RuntimeCase::new(&temp.path, "custom-sidebar");
    custom_sidebar.write_default_config(
        "\n[sidebar]\ncommand = \"true\"\nargs = [\"two words\", \"--basic\"]\n",
    );
    let status = custom_sidebar.prepared_status(&yzx_bin, "custom sidebar status");
    expect_contains_all! {
        &status, "custom sidebar status";
        "sidebar command: true",
        r#"sidebar args: ["two words","--basic"]"#,
        "zellij config: runtime (",
        "layout: runtime (",
    }
    let custom_sidebar_layout = custom_sidebar.zellij_file("layout.kdl");
    let custom_sidebar_swap = custom_sidebar.zellij_file("layout.swap.kdl");
    for (text, context) in [
        (&custom_sidebar_layout, "custom sidebar layout"),
        (&custom_sidebar_swap, "custom sidebar swap layout"),
    ] {
        assert_eq!(text.matches(r#"pane name="sidebar" size="#).count(), 2);
        assert_eq!(text.matches(r#"command="true""#).count(), 2);
        assert_eq!(text.matches(r#"args "two words" "--basic""#).count(), 2);
        assert!(
            !text.contains(r#"plugin location="radar""#) && !text.contains("@sidebar@"),
            "{context} kept Radar or an unresolved sidebar placeholder"
        );
    }
    let custom_sidebar_config = custom_sidebar.zellij_file("config.kdl");
    assert!(
        !custom_sidebar_config.contains(r#"MessagePlugin "radar_controller""#)
            && !custom_sidebar_config.contains("radar_controller location=")
            && !custom_sidebar_config
                .lines()
                .any(|line| line.trim() == "radar_controller"),
        "custom sidebar config kept Radar-only key routes"
    );
    expect_contains(
        &custom_sidebar_config,
        r#"MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar"; }"#,
        "custom sidebar config",
    );
    successful_output(
        Command::new(&zellij)
            .arg("--config")
            .arg(custom_sidebar.zellij_path("config.kdl"))
            .args(["setup", "--check"]),
        "custom sidebar Zellij config check",
    );
    let custom_sidebar_permissions = custom_sidebar.zellij_file("permissions.kdl");
    assert!(
        !custom_sidebar_permissions.contains(&radar_wasm.display().to_string()),
        "custom sidebar permissions kept the Radar grant"
    );
    expect_contains_all! {
        &custom_sidebar_permissions, "custom sidebar permissions";
        "share/yazelix_zellij_popup/yzpp.wasm\" {",
        "share/nova_bar/zjstatus.wasm\" {",
        "share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm\" {",
    }
    let doctor = custom_sidebar.run_yzx(&yzx_bin, "doctor", "custom sidebar doctor");
    assert!(
        !doctor.contains("Radar"),
        "custom sidebar doctor kept Radar integration diagnostics: {doctor}"
    );

    let custom_popup = RuntimeCase::new(&temp.path, "custom-popup");
    custom_popup.write_default_config("\n[popup]\nside_margin = 2\nvertical_margin = 1\n");
    let status = custom_popup.prepared_status(&yzx_bin, "custom popup status");
    expect_contains_all! {
        &status, "custom popup status";
        "popup side margin: 2",
        "popup vertical margin: 1",
        "zellij config: runtime (",
        "layout: packaged (/nix/store/",
    }
    let custom_popup_config = custom_popup.zellij_file("config.kdl");
    expect_popup_defaults(&custom_popup_config, "2", "1", "custom popup status config");
    assert_eq!(custom_popup_config.matches("side_margin 2").count(), 1);
    assert_eq!(custom_popup_config.matches("vertical_margin 1").count(), 1);

    let custom_agent = RuntimeCase::new(&temp.path, "custom-agent");
    custom_agent.write_default_config("\n[agent]\ncommand = \"codex\"\nargs = [\"resume\", \"--dangerously-bypass-approvals-and-sandbox\"]\n");
    let status = custom_agent.prepared_status(&yzx_bin, "custom agent status");
    expect_contains_all! {
        &status, "custom agent status";
        "agent command: codex",
        r#"agent args: ["resume","--dangerously-bypass-approvals-and-sandbox"]"#,
        "zellij config: runtime (",
    }
    let custom_agent_config = custom_agent.zellij_file("config.kdl");
    let agent_launcher = popup_command(&custom_agent_config, "/bin/yzx-agent");
    expect_contains(
        &custom_agent_config,
        &format!(
            "agent {{\n                command \"{}\"\n                arg_1 \"codex\"\n                arg_2 \"resume\"\n                arg_3 \"--dangerously-bypass-approvals-and-sandbox\"\n                pane_title \"agent_popup\"\n                command_marker \"/bin/yzx-agent\"\n                preserve_terminal_title true\n                toggle_close_behavior \"hide\"\n            }}",
            agent_launcher.display(),
        ),
        "custom agent config",
    );
    expect_contains(
        &custom_agent_config,
        "managed_agent_command_marker \"/bin/yzx-agent\"",
        "custom agent command marker",
    );

    let custom_popup_spec_case = RuntimeCase::new(&temp.path, "custom-popup-spec");
    custom_popup_spec_case.write_default_config("\n[popup]\nside_margin = 2\nvertical_margin = 1\n\n[popups.btm]\ncommand = \"btm\"\nargs = [\"--basic\"]\ntitle = \"btm_popup\"\nkeybinding = \"Alt Shift B\"\nkeep_alive = true\n");
    custom_popup_spec_case.prepared_status(&yzx_bin, "custom popup spec status");
    let custom_popup_spec = custom_popup_spec_case.zellij_file("config.kdl");
    expect_contains(
        &custom_popup_spec,
        "btm {\n                command \"btm\"\n                arg_1 \"--basic\"\n                pane_title \"btm_popup\"\n                command_marker \"btm_popup\"\n                toggle_close_behavior \"hide\"\n            }",
        "custom popup spec config",
    );
    expect_popup_binding(
        &custom_popup_spec,
        "Alt Shift B",
        "btm",
        "custom popup spec config",
    );
    assert_eq!(custom_popup_spec.matches("side_margin 2").count(), 1);
    assert_eq!(custom_popup_spec.matches("vertical_margin 1").count(), 1);

    let zellij_plugins = RuntimeCase::new(&temp.path, "zellij-plugins");
    zellij_plugins.write_default_config("");
    let zellij_plugins_sidecar = zellij_plugins.config_home.join("zellij/plugins.kdl");
    fs::create_dir_all(zellij_plugins_sidecar.parent().unwrap()).unwrap();
    fs::write(
        &zellij_plugins_sidecar,
        "plugins {\n    // User plugin comments survive injection.\n    my_plugin location=\"file:/tmp/my_plugin.wasm\" {\n        payload \"{\\\"ok\\\": true}\" // Braces in strings must not change block depth.\n    } // plugin config close\n} // plugins close\n\nload_plugins {\n    my_plugin\n} // load_plugins close\n",
    )
    .unwrap();
    zellij_plugins.prepared_status(&yzx_bin, "Zellij plugin sidecar status");
    let zellij_plugin_config = zellij_plugins.zellij_file("config.kdl");
    expect_contains_all! {
        &zellij_plugin_config, "Zellij plugin sidecar config";
        "payload \"{\\\"ok\\\": true}\" // Braces in strings must not change block depth.",
        "    } // plugin config close\n    yazelix_pane_orchestrator location=",
        "    my_plugin\n    yazelix_pane_orchestrator\n}",
    }

    let custom_keys = RuntimeCase::new(&temp.path, "custom-keys");
    custom_keys.write_default_config("\n[keybindings]\nconfig = \"Alt Shift C\"\nagent = \"Ctrl Shift A\"\ngit = \"Alt Shift G\"\nmenu = \"Alt Shift U\"\nscreen = \"Alt Shift S\"\nsidebar = \"Ctrl Shift B\"\nsidebar_focus = \"Ctrl Shift E\"\n");
    let status = custom_keys.prepared_status(&yzx_bin, "custom key status");
    expect_contains_all! {
        &status, "custom key status";
        "config keybinding: Alt Shift C",
        "agent keybinding: Ctrl Shift A",
        "git keybinding: Alt Shift G",
        "menu keybinding: Alt Shift U",
        "screen keybinding: Alt Shift S",
        "sidebar keybinding: Ctrl Shift B",
        "forest keybinding: Ctrl Shift E",
        "zellij config: runtime (",
    }
    let custom_key_config = custom_keys.zellij_file("config.kdl");
    for (key, payload, default) in [
        ("Alt Shift C", "config", "Alt Shift K"),
        ("Ctrl Shift A", "agent", "Alt Shift L"),
        ("Alt Shift G", "git", "Alt Shift J"),
        ("Alt Shift U", "menu", "Alt Shift M"),
        ("Alt Shift S", "screen", "Alt Shift A"),
    ] {
        expect_popup_binding(&custom_key_config, key, payload, "custom key config");
        assert!(
            !custom_key_config.contains(&format!(r#"bind "{default}" {{"#)),
            "custom key kept the default {payload} binding"
        );
    }
    expect_contains_all! {
        &custom_key_config, "custom key config";
        r#"bind "Ctrl Shift B" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar"; }; }"#,
    }
    for default in ["Alt Shift H", "Ctrl y", "Ctrl Shift E"] {
        assert!(
            !custom_key_config.contains(&format!(r#"bind "{default}" {{"#)),
            "custom key kept the default {default} binding"
        );
    }

    let unmapped_keys = RuntimeCase::new(&temp.path, "unmapped-keys");
    unmapped_keys.write_default_config("\n[keybindings]\nconfig = false\nagent = \"Ctrl Shift A\"\nscreen = false\nsidebar = false\n");
    let status = unmapped_keys.prepared_status(&yzx_bin, "unmapped key status");
    expect_contains_all! {
        &status, "unmapped key status";
        "config keybinding: unmapped",
        "agent keybinding: Ctrl Shift A (remapped)",
        "screen keybinding: unmapped",
        "sidebar keybinding: unmapped",
        "menu keybinding: Alt Shift M",
    }
    let unmapped_key_config = unmapped_keys.zellij_file("config.kdl");
    expect_popup_binding(
        &unmapped_key_config,
        "Ctrl Shift A",
        "agent",
        "unmapped key config",
    );
    for omitted in ["Alt Shift K", "Alt Shift A", "Alt Shift H"] {
        assert!(
            !unmapped_key_config.contains(&format!(r#"bind "{omitted}" {{"#)),
            "unmapped key config kept {omitted}"
        );
    }
    assert!(
        !unmapped_key_config.contains("__YZX_MANAGED_KEY_"),
        "unmapped key config leaked a runtime marker"
    );
    successful_output(
        Command::new(&zellij)
            .arg("--config")
            .arg(unmapped_keys.zellij_path("config.kdl"))
            .args(["setup", "--check"]),
        "unmapped key Zellij config check",
    );

    let swapped_keys = RuntimeCase::new(&temp.path, "swapped-keys");
    swapped_keys.write_default_config("\n[keybindings]\nconfig = \"Alt Shift H\"\nagent = \"Ctrl y\"\ngit = \"Alt Shift M\"\nmenu = \"Alt Shift J\"\nsidebar = \"Alt Shift K\"\nsidebar_focus = \"Alt Shift L\"\n");
    swapped_keys.prepared_status(&yzx_bin, "swapped key status");
    let swapped_key_config = swapped_keys.zellij_file("config.kdl");
    for (key, payload) in [
        ("Alt Shift H", "config"),
        ("Ctrl y", "agent"),
        ("Alt Shift M", "git"),
        ("Alt Shift J", "menu"),
    ] {
        expect_popup_binding(&swapped_key_config, key, payload, "swapped key config");
    }
    expect_contains_all! {
        &swapped_key_config, "swapped key config";
        r#"bind "Alt Shift K" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar"; }; }"#,
    }
    assert!(!swapped_key_config.contains(r#"bind "Alt Shift L""#));

    let custom_editor = RuntimeCase::new(&temp.path, "custom-editor");
    custom_editor.write_default_config("\n[editor]\ncommand = \"nvim\"\n");
    let status = custom_editor.run_yzx(&yzx_bin, "status", "custom editor status");
    expect_contains_all! {
        &status, "custom editor status";
        "editor command: nvim",
        "editor: nvim",
    }

    let custom_bar = RuntimeCase::new(&temp.path, "custom-bar");
    custom_bar.write_default_config("\n[bar]\nwidgets = [\"editor\", \"claude_usage\", \"cpu\"]\n");
    let status = custom_bar.prepared_status(&yzx_bin, "custom bar status");
    expect_contains_all! {
        &status, "custom bar status";
        r#"bar widgets: ["editor","claude_usage","cpu"]"#,
        "popup side margin: 1",
        "popup vertical margin: 0",
        "zellij config: runtime (",
        "layout: runtime (",
    }
    let custom_layout = custom_bar.zellij_file("layout.kdl");
    expect_contains(
        &custom_layout,
        r#"new_tab_template cwd="$HOME" {"#,
        "custom bar layout",
    );
    assert!(
        !custom_layout.contains("@yazi@")
            && !custom_layout.contains("@editor@")
            && custom_layout.matches("/bin/yzx-yazi").count() == 2
            && !custom_layout.contains("/bin/yzx-editor"),
        "custom bar layout did not materialize only the two tiled startup Yazi pickers"
    );
    let custom_bar_config = custom_bar.zellij_file("config.kdl");
    let format_right = custom_bar_config
        .lines()
        .find(|line| line.contains("format_right"))
        .expect("custom controller is missing format_right");
    expect_contains_all! {
        format_right, "custom bar controller";
        "{command_claude_usage}",
        "{command_cpu}",
    }
    assert!(
        !format_right.contains("{command_codex_usage}"),
        "custom visible bar kept a Codex widget omitted by bar.widgets"
    );
    let custom_swap = custom_bar.zellij_file("layout.swap.kdl");
    expect_contains_all! {
        &custom_swap, "custom bar swap layout";
        "swap_tiled_layout name=\"single_open\"",
        "swap_tiled_layout name=\"single_closed\"",
        "plugin location=\"radar\"",
        "pane name=\"sidebar\" size=32 borderless=false {",
        "pane name=\"sidebar\" size=1 borderless=false {",
        "stacked=true",
    }
    assert!(
        !custom_swap.contains('@'),
        "custom bar swap layout kept an unresolved placeholder"
    );
    let custom_config = custom_bar.zellij_file("config.kdl");
    expect_contains_all! {
        &custom_config, "custom bar new-tab config";
        r#"default_layout "layout""#,
        format!(r#"layout_dir "{}""#, custom_bar.zellij_path("layout.kdl").parent().unwrap().display()),
        format!(r#"layout "{}""#, custom_bar.zellij_path("layout.kdl").display()),
        format!("cwd {home};"),
    }

    let light_bar = RuntimeCase::new(&temp.path, "light-bar");
    light_bar.write_default_config("\n[appearance]\nmode = \"light\"\n");
    let status = light_bar.prepared_status(&yzx_bin, "light appearance status");
    expect_contains_all! {
        &status, "light appearance status";
        "layout: runtime (",
    }
    let light_config = light_bar.zellij_file("config.kdl");
    expect_contains_all! {
        &light_config, "light appearance bar controller";
        r#"host_theme_mode "light""#,
        r##"host_theme_dark_tab_normal "#[fg=#ffff00] [{index}] {name} ""##,
        r##"host_theme_light_tab_normal "#[fg=#5c5f77] [{index}] {name} ""##,
        r##"#[fg=#2f7d32,bold] hx"##,
    }

    let custom_shell_bar = RuntimeCase::new(&temp.path, "custom-shell-bar");
    custom_shell_bar.write_config("[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"fish\"\n");
    let status = custom_shell_bar.prepared_status(&yzx_bin, "custom shell bar status");
    expect_contains_all! {
        &status, "custom shell bar status";
        "shell: fish",
        "layout: runtime (",
    }
    let custom_shell_config = custom_shell_bar.zellij_file("config.kdl");
    expect_contains_all! {
        &custom_shell_config, "custom shell bar controller";
        "❯fish",
    }
    assert!(
        !custom_shell_config.contains("❯nu"),
        "custom shell bar controller kept the default shell label"
    );
    assert!(
        !custom_shell_config.contains("❯ fish"),
        "custom shell bar controller inserted unwanted shell label spacing"
    );

    let doctor = doctor_case.run_yzx(&yzx_bin, "doctor", "yzx doctor");
    expect_contains_all! {
        &doctor, "yzx doctor";
        "Yazelix Nova doctor",
        "Core",
        "ok    Configuration    settings valid",
        "ok    Commands         shell nu · editor yzx-hx · agent auto",
        "ok    Interface        7 keybindings · bar widgets configured",
        "Runtime",
        "ok    Yazi             ",
        "ok    Components       all packaged helpers and plugins found",
        "Integrations",
        "Cleanup",
        "ok    Classic residue  none found",
        "info  Session          outside Zellij",
    }
    assert!(
        !doctor.contains("yazi lookup PATH") && !doctor.contains("\x1b["),
        "default redirected doctor output must stay concise plain text: {doctor}"
    );

    let doctor_codex_bin = temp.path.join("doctor-codex-bin");
    fs::create_dir(&doctor_codex_bin).unwrap();
    write_fake_agent(&doctor_codex_bin, "codex");
    write_executable(&doctor_codex_bin.join("zj-radar"), "#!/bin/sh\nexit 0\n");
    let doctor_codex_home = temp.path.join("doctor-codex-home");
    fs::create_dir(&doctor_codex_home).unwrap();
    let doctor_codex = successful_stdout(
        doctor_case
            .yzx_command(&yzx_bin, "doctor")
            .env("PATH", &doctor_codex_bin)
            .env("CODEX_HOME", &doctor_codex_home),
        "yzx doctor missing Radar Codex hooks",
    );
    expect_contains_all! {
        &doctor_codex, "yzx doctor missing Radar Codex hooks";
        "warn  Radar            Codex hooks need attention",
        "missing hooks.json: zj-radar Codex hooks are not installed",
        "action: resolve the warning, then run yzx radar-setup",
    }
    assert!(
        !doctor_codex.contains("Codex trust"),
        "doctor offered a trust review before Radar installed any hooks"
    );

    for current in ["yazi", "zellij", "helix", "helix-steel", "logs"] {
        fs::create_dir_all(doctor_case.state_dir.join(current)).unwrap();
    }
    let configs = doctor_case.state_dir.join("configs");
    let sessions = doctor_case.state_dir.join("sessions");
    fs::create_dir(&configs).unwrap();
    let outside_sessions = temp.path.join("doctor-outside-sessions");
    fs::create_dir(&outside_sessions).unwrap();
    let snapshot = outside_sessions.join("config_snapshot.json");
    fs::write(&snapshot, "untouched").unwrap();
    symlink(&outside_sessions, &sessions).unwrap();
    let nushell = doctor_case.state_dir.join("initializers/nushell");
    fs::create_dir_all(&nushell).unwrap();
    let extern_file = nushell.join("yazelix_extern.nu");
    let fingerprint = nushell.join("yazelix_extern.fingerprint.json");
    fs::write(&extern_file, "classic").unwrap();
    symlink(temp.path.join("missing-fingerprint"), &fingerprint).unwrap();
    fs::create_dir_all(&doctor_case.config_home).unwrap();
    let config_backup = doctor_case.config_home.join("config.toml.backup-20260712");
    let settings_backup = doctor_case
        .config_home
        .join("settings.jsonc.backup-20260711");
    fs::write(&config_backup, "classic").unwrap();
    symlink(temp.path.join("missing-backup"), &settings_backup).unwrap();
    let residue_doctor = successful_stdout(
        doctor_case.yzx_command(&yzx_bin, "doctor").arg("--verbose"),
        "yzx doctor residue",
    );
    expect_contains_all! {
        &residue_doctor, "yzx doctor residue";
        classic_residue_warning(&configs, "ambiguous"),
        classic_residue_warning(&sessions, "ambiguous"),
        classic_residue_warning(&extern_file, "certain"),
        classic_residue_warning(&fingerprint, "ambiguous"),
        classic_residue_warning(&config_backup, "ambiguous"),
        classic_residue_warning(&settings_backup, "ambiguous"),
        "external scripts may still reference these paths",
    }
    assert!(
        !residue_doctor.contains("yazi lookup PATH:"),
        "verbose doctor duplicated configuration already owned by yzx status"
    );
    assert_eq!(fs::read_to_string(snapshot).unwrap(), "untouched");
    for current in ["yazi", "zellij", "helix", "helix-steel", "logs"] {
        let path = doctor_case.state_dir.join(current);
        assert!(
            !residue_doctor.contains(&format!("path={}", path.display())),
            "yzx doctor reported current Nova {current} state as Classic residue"
        );
    }

    let linked_parent = RuntimeCase::new(&temp.path, "doctor-linked-parent");
    let linked_target = temp.path.join("doctor-linked-target");
    fs::create_dir_all(&linked_parent.state_dir).unwrap();
    fs::create_dir_all(linked_target.join("nushell")).unwrap();
    fs::write(linked_target.join("nushell/yazelix_extern.nu"), "classic").unwrap();
    symlink(linked_target, linked_parent.state_dir.join("initializers")).unwrap();
    let linked_parent_doctor =
        linked_parent.run_yzx(&yzx_bin, "doctor", "yzx doctor symlinked parent");
    expect_contains(
        &linked_parent_doctor,
        "ok    Classic residue  none found",
        "yzx doctor symlinked parent",
    );

    for (args, expected, context) in [
        (
            &["env", "extra"][..],
            "yzx env does not accept arguments yet",
            "yzx env argument error",
        ),
        (
            &["doctor", "extra"][..],
            "yzx doctor accepts only --verbose",
            "yzx doctor argument error",
        ),
        (
            &["status", "extra"][..],
            "yzx status accepts only --json",
            "yzx status argument error",
        ),
        (
            &["menu", "extra"][..],
            "yzx menu does not accept arguments yet",
            "yzx menu argument error",
        ),
        (
            &["tutor", "continue"][..],
            "Unknown yzx tutor target: continue",
            "yzx tutor unknown lesson error",
        ),
        (
            &["tutor", "workspace", "extra"][..],
            "Unexpected arguments for yzx tutor",
            "yzx tutor extra argument error",
        ),
        (
            &["run"][..],
            "Usage: yzx run <program> [args...]",
            "yzx run missing program",
        ),
        (
            &["sponsor"][..],
            "yzx: unknown command: sponsor",
            "removed yzx sponsor command",
        ),
        (
            &["screen"][..],
            "yzx: unknown command: screen",
            "renamed yzx screen command",
        ),
    ] {
        expect_command_error(&yzx_bin, args, expected, context);
    }
    let identity = fs::read_to_string(yzx.join("share/yazelix/runtime_identity.json"))
        .expect("yzx package is missing runtime_identity.json");
    let identity_version = jq_output(jq, ".version", &identity);
    assert_eq!(version.trim(), format!("Yazelix Nova ({identity_version})"));
    expect_contains_all! {
        &identity, "yzx runtime identity";
        r#""name":"Yazelix Nova""#,
    }
    assert!(
        yzx.join("libexec/yazelix/yzx-tutor").is_file(),
        "yzx package is missing the tutor helper"
    );
}

fn classic_residue_warning(path: &Path, ownership: &str) -> String {
    format!(
        "warn classic residue: ownership={ownership} nova=unused path={}",
        path.display()
    )
}

fn expect_headless_enter(yzx: &Path) {
    let temp = TempDir::new();
    let case = RuntimeCase::new(&temp.path, "headless-enter");
    case.write_default_config("\n[welcome]\nenabled = false\n");
    let output = successful_stdout(
        case.yzx_command(&yzx.join("bin/yzx"), "enter")
            .arg("--version")
            .env("TERM", "xterm-256color")
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("ZELLIJ")
            .env_remove("ZELLIJ_PANE_ID"),
        "headless yzx enter --version",
    );
    assert!(
        output.starts_with("zellij "),
        "headless yzx enter did not reach Zellij: {output:?}"
    );
}

fn expect_narrow_path_launches(yzx: &Path, yzx_shell: &Path) {
    let yzx_bin = yzx.join("bin/yzx");
    let temp = TempDir::new();
    for (command, expected) in [
        ("help", "Usage:"),
        ("status", "Yazelix Nova status"),
        ("doctor", "Yazelix Nova doctor"),
    ] {
        let case = RuntimeCase::new(&temp.path, &format!("narrow-path-{command}"));
        let mut yzx = case.yzx_command(&yzx_bin, command);
        yzx.env("PATH", "/private/tmp");
        let output = successful_stdout(&mut yzx, &format!("narrow PATH yzx {command}"));
        expect_contains(&output, expected, &format!("narrow PATH yzx {command}"));
    }

    for (program, args, context) in [
        (
            yzx_shell.to_path_buf(),
            &["--version"][..],
            "narrow PATH yzx-shell --version",
        ),
        (
            embedded_store_path(&binary_text(&yzx_bin), "/bin/yzx-hx"),
            &["--version"][..],
            "narrow PATH yzx-hx --version",
        ),
    ] {
        let case = RuntimeCase::new(&temp.path, context);
        let stdout = successful_stdout(
            Command::new(program)
                .args(args)
                .env("PATH", "/private/tmp")
                .env("YAZELIX_CONFIG_HOME", case.config_home)
                .env("YAZELIX_STATE_DIR", case.state_dir)
                .env_remove("ZELLIJ_SESSION_NAME"),
            context,
        );
        assert!(
            !stdout.trim().is_empty(),
            "{context} succeeded without printing a version"
        );
    }

    let case = RuntimeCase::new(&temp.path, "managed-hx-alias");
    write_executable(&temp.path.join("yzx"), "#!/bin/sh\nexit 99\n");
    let mut command = case.yzx_command(&yzx_bin, "run");
    command.args(["printenv", "PATH"]).env("PATH", &temp.path);
    let output = successful_stdout(&mut command, "managed hx PATH");
    let path = output.trim();
    let resolve = |name| {
        env::split_paths(path)
            .map(|dir| dir.join(name))
            .find(|candidate| candidate.is_file())
            .unwrap_or_else(|| panic!("managed PATH is missing {name}"))
    };
    assert_eq!(
        fs::canonicalize(resolve("hx")).unwrap(),
        fs::canonicalize(resolve("yzx-hx")).unwrap(),
        "managed hx must resolve to yzx-hx"
    );
    assert_eq!(
        fs::canonicalize(resolve("yzx")).unwrap(),
        fs::canonicalize(&yzx_bin).unwrap(),
        "managed PATH must resolve to its invoking yzx"
    );
    assert!(
        resolve("zj-radar").is_file(),
        "managed PATH is missing the Radar producer CLI"
    );
    assert_eq!(
        fs::canonicalize(resolve("yzx-yazi")).unwrap(),
        fs::canonicalize(yzx.join("bin/yzx-yazi")).unwrap(),
        "managed PATH must resolve the public Yazi wrapper"
    );
}

fn expect_radar_setup(yzx_bin: &Path) {
    let temp = TempDir::new();
    let launcher = temp.path.join("yzx");
    fs::copy(yzx_bin, &launcher).unwrap();
    let record = temp.path.join("radar-args");
    let codex_home = temp.path.join("codex home");
    let config_home = temp.path.join("config home");
    let state = temp.path.join("state");
    fs::create_dir_all(state.join("agent")).unwrap();
    fs::write(state.join("agent/radar-codex-setup-offered"), "1\n").unwrap();
    write_executable(
        &temp.path.join("zj-radar"),
        "#!/bin/sh\nprintf '%s\\n' \"$*\" \"$CODEX_HOME\" \"$XDG_CONFIG_HOME\" >\"$YZX_RADAR_TEST_OUT\"\nprintf 'provider setup output\\n'\nprintf 'provider diagnostic\\n' >&2\nexit \"$YZX_RADAR_TEST_EXIT\"\n",
    );
    for command in ["radar-setup", "menu"] {
        for code in [0, 23] {
            let mut child = Command::new(&launcher)
                .arg(command)
                .env_clear()
                .env("HOME", &temp.path)
                .env("CODEX_HOME", &codex_home)
                .env("XDG_CONFIG_HOME", &config_home)
                .env("YAZELIX_STATE_DIR", &state)
                .env("YZX_RADAR_TEST_OUT", &record)
                .env("YZX_RADAR_TEST_EXIT", code.to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            if command == "menu" {
                child
                    .stdin
                    .as_mut()
                    .unwrap()
                    .write_all(b"radar-setup\n")
                    .unwrap();
            }
            drop(child.stdin.take());
            let output = child.wait_with_output().unwrap();
            assert_eq!(output.status.code(), Some(code), "{command}: {output:?}");
            assert_eq!(
                fs::read_to_string(&record).unwrap(),
                format!(
                    "setup codex claude opencode\n{}\n{}\n",
                    codex_home.display(),
                    config_home.display()
                ),
                "setup must delegate exact targets without blanket consent and preserve config roots"
            );
            expect_contains(
                &String::from_utf8_lossy(&output.stdout),
                "provider setup output",
                command,
            );
            expect_contains(
                &String::from_utf8_lossy(&output.stderr),
                "provider diagnostic",
                command,
            );
        }
    }
    fs::remove_file(&record).unwrap();
    expect_command_error(
        &launcher,
        &["radar-setup", "--yes"],
        "does not accept arguments",
        "radar setup usage",
    );
    assert!(!record.exists(), "invalid arguments must not invoke setup");
}

fn expect_menu_dispatch(menu: &Path) {
    expect_contains(&binary_text(menu), "/bin/fzf", "yzx-menu packaged fzf path");

    let temp = TempDir::new();
    let fake_yzx = temp.path.join("fake-yzx");
    let output_file = temp.path.join("selected-command");
    write_executable(
        &fake_yzx,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >\"$YZX_MENU_TEST_OUT\"\n",
    );

    let mut child = Command::new(menu)
        .env("YZX_MENU_YZX", &fake_yzx)
        .env("YZX_MENU_TEST_OUT", &output_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"3\n4\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "menu selection failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read_to_string(output_file).unwrap(), "status\n");
}

fn expect_command_error(yzx_bin: &Path, args: &[&str], expected: &str, context: &str) {
    let output = Command::new(yzx_bin).args(args).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(64),
        "yzx {args:?} should fail with usage status"
    );
    expect_contains(&String::from_utf8_lossy(&output.stderr), expected, context);
}

fn jq_output(jq: &Path, query: &str, json: &str) -> String {
    let filter = format!("$input | {query}");
    successful_stdout(
        Command::new(jq).args(["-nr", "--argjson", "input", json, &filter]),
        "status JSON",
    )
    .trim_end()
    .to_string()
}

impl RuntimeCase {
    fn prepared_status(&self, yzx: &Path, context: &str) -> String {
        successful_output(self.yzx_command(yzx, "run").arg("true"), context);
        self.run_yzx(yzx, "status", context)
    }

    fn zellij_file(&self, file: &str) -> String {
        fs::read_to_string(self.zellij_path(file)).unwrap()
    }

    fn zellij_path(&self, file: &str) -> PathBuf {
        self.state_dir.join("zellij").join(file)
    }
}

fn expect_config_ui(yzx: &Path) {
    let packaged_config = yzx.join("share/yazelix/config.toml");
    assert!(
        packaged_config.is_file(),
        "yzx package is missing config.toml"
    );
    let packaged_config = fs::read_to_string(&packaged_config).unwrap();
    expect_contains_all! {
        &packaged_config, "packaged config.toml";
        "log_level = \"info\"",
        "program = \"nu\"",
        "atuin = true",
        "command = \"yzx-hx\"",
        "command = \"auto\"",
        "[sidebar]",
        "command = \"radar\"",
        "args = []",
        "enabled = true",
        "style = \"random\"",
        "duration_seconds = 3",
        "side_margin = 1",
        "vertical_margin = 0",
        "side = \"right\"",
        "config = \"Alt Shift K\"",
        "agent = \"Alt Shift L\"",
        "git = \"Alt Shift J\"",
        "menu = \"Alt Shift M\"",
        "screen = \"Alt Shift A\"",
        "sidebar = \"Alt Shift H\"",
        "sidebar_focus = \"Ctrl y\"",
        "widgets = [\"editor\", \"shell\", \"term\", \"codex_usage\", \"cpu\", \"ram\"]",
    }

    let helper = yzx.join("libexec/yazelix/yzx-config");
    assert!(helper.is_file(), "missing yzx-config helper");
    let temp = TempDir::new();
    for (path, expected) in [
        ("open.log_level", "info"),
        ("shell.program", "nu"),
        ("shell.atuin", "true"),
        ("editor.command", "yzx-hx"),
        ("agent.command", "auto"),
        ("agent.args", "[]"),
        ("sidebar.command", "radar"),
        ("sidebar.args", "[]"),
        ("welcome.enabled", "true"),
        ("welcome.style", "random"),
        ("welcome.duration_seconds", "3"),
        ("popup.side_margin", "1"),
        ("popup.vertical_margin", "0"),
        ("forest.side", "right"),
        ("keybindings.config", "Alt Shift K"),
        ("keybindings.agent", "Alt Shift L"),
        ("keybindings.git", "Alt Shift J"),
        ("keybindings.menu", "Alt Shift M"),
        ("keybindings.screen", "Alt Shift A"),
        ("keybindings.sidebar", "Alt Shift H"),
        ("keybindings.sidebar_focus", "Ctrl y"),
        (
            "bar.widgets",
            r#"["editor","shell","term","codex_usage","cpu","ram"]"#,
        ),
    ] {
        let output = successful_stdout(
            Command::new(&helper)
                .arg("--get")
                .arg(path)
                .env("YAZELIX_CONFIG_HOME", &temp.path),
            &format!("yzx-config --get {path}"),
        );
        assert_eq!(output.trim(), expected);
    }

    let unknown_temp = TempDir::new();
    let output = Command::new(&helper)
        .arg("--get")
        .arg("shell.typo")
        .env("YAZELIX_CONFIG_HOME", &unknown_temp.path)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "unknown yzx-config --get path unexpectedly succeeded"
    );
    expect_contains(
        &String::from_utf8_lossy(&output.stderr),
        "unknown config path: shell.typo",
        "unknown yzx-config --get path",
    );
    assert!(
        !unknown_temp.path.join("config.toml").exists(),
        "unknown yzx-config --get path created config.toml"
    );

    assert!(
        !temp.path.join("config.toml").exists(),
        "default config reads created config.toml"
    );

    let projection = successful_stdout(
        Command::new(&helper)
            .args(["--project-rio-appearance", "light"])
            .env("YAZELIX_CONFIG_HOME", &temp.path),
        "writable Rio appearance projection",
    );
    assert_eq!(projection.trim(), "live");
    let rio_config = temp.path.join("rio/config.toml");
    expect_contains(
        &fs::read_to_string(&rio_config).unwrap(),
        "force-theme = \"light\"",
        "writable Rio appearance projection",
    );
    let mut permissions = fs::metadata(&rio_config).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&rio_config, permissions).unwrap();
    let projection = successful_stdout(
        Command::new(&helper)
            .args(["--project-rio-appearance", "dark"])
            .env("YAZELIX_CONFIG_HOME", &temp.path),
        "read-only Rio appearance projection",
    );
    assert_eq!(projection.trim(), "next-launch");
    expect_contains(
        &fs::read_to_string(rio_config).unwrap(),
        "force-theme = \"light\"",
        "read-only Rio appearance projection",
    );
}

fn expect_startup_diagnostics(yzx: &Path) {
    let yzx_bin = yzx.join("bin/yzx");
    let temp = TempDir::new();

    let sidecar_config = temp.path.join("sidecar-config");
    fs::create_dir_all(sidecar_config.join("zellij")).unwrap();
    let sidecar = sidecar_config.join("zellij/config.kdl");
    fs::write(&sidecar, "default_shell \"nu\"\n").unwrap();

    let mut failure_cases = vec![(
        sidecar_config,
        sidecar,
        "forbidden Zellij sidecar item `default_shell`",
        "forbidden sidecar",
    )];
    for (dir, config, reason, label) in [
        (
            "bad-config",
            "[open]\nlog_level = \"loud\"\n\n[shell]\nprogram = \"nu\"\n",
            "open.log_level must be one of: off, error, info, debug",
            "invalid config",
        ),
        (
            "bad-bar-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[bar]\nwidgets = [\"weather\"]\n",
            "bar.widgets must be one of: session, editor, shell, term, claude_usage, codex_usage, opencode_go_usage, cpu, ram.",
            "invalid bar widgets",
        ),
        (
            "bad-editor-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[editor]\ncommand = \"nvim --clean\"\n",
            "editor.command must be one executable command without arguments",
            "invalid editor command",
        ),
        (
            "bad-agent-command-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[agent]\ncommand = \"codex resume\"\n",
            "agent.command must be auto or one executable command without arguments",
            "invalid agent command",
        ),
        (
            "bad-sidebar-command-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[sidebar]\ncommand = \"btm --basic\"\n",
            "sidebar.command must be radar or one executable command without arguments",
            "invalid sidebar command",
        ),
        (
            "bad-radar-args-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[sidebar]\ncommand = \"radar\"\nargs = [\"unused\"]\n",
            "sidebar.args requires sidebar.command to be a custom command",
            "invalid Radar arguments",
        ),
        (
            "bad-popup-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[popup]\nside_margin = -1\n",
            "popup.side_margin must be zero or greater",
            "invalid popup margin",
        ),
        (
            "bad-welcome-style-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[welcome]\nstyle = \"snow\"\n",
            "welcome.style must be one of: static, logo, asciiquarium, aquarium, boids, boids_predator, boids_schools, friends_and_enemies, primordial, physarum, chladni, plasma, mandelbrot, matrix, game_of_life_gliders, game_of_life_tumblers, random",
            "invalid welcome style",
        ),
        (
            "bad-welcome-duration-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[welcome]\nduration_seconds = 0\n",
            "welcome.duration_seconds must be between 1 and 60",
            "invalid welcome duration",
        ),
        (
            "bad-key-syntax-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[keybindings]\nagent = \"Alt+Shift+A\"\n",
            "keybindings.agent must be a key chord like Alt Shift A",
            "invalid agent key syntax",
        ),
        (
            "bad-key-conflict-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[keybindings]\nagent = \"Alt Shift f\"\n",
            "keybindings.agent conflicts with packaged key Alt Shift f",
            "conflicting agent key",
        ),
        (
            "bad-new-tab-key-config",
            "[keybindings]\nconfig = \"Alt Shift t\"\n",
            "keybindings.config conflicts with packaged key Alt Shift t",
            "conflicting new-tab key",
        ),
        (
            "bad-close-tab-popup-key-config",
            "[popups.btm]\ncommand = \"btm\"\nkeybinding = \"Alt Shift W\"\n",
            "popups.btm.keybinding conflicts with packaged key Alt Shift W",
            "conflicting close-tab key",
        ),
        (
            "bad-key-duplicate-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[keybindings]\nconfig = \"Alt Shift A\"\nagent = \"Alt Shift A\"\n",
            "keybindings.agent conflicts with keybindings.config: Alt Shift A",
            "duplicate popup key",
        ),
        (
            "bad-custom-popup-command-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[popups.btm]\ncommand = \"btm --basic\"\nkeybinding = \"Alt Shift B\"\n",
            "popups.btm.command must be one executable command without arguments; use args for arguments",
            "invalid custom popup command",
        ),
        (
            "bad-custom-popup-key-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[popups.btm]\ncommand = \"btm\"\nkeybinding = \"Alt r\"\n",
            "popups.btm.keybinding conflicts with packaged key Alt r",
            "invalid custom popup key",
        ),
    ] {
        let config_home = temp.path.join(dir);
        let check = write_config_home(&config_home, config);
        failure_cases.push((config_home, check, reason, label));
    }

    for (config_home, check, reason, label) in failure_cases {
        for command in ["enter", "status", "doctor"] {
            let runtime = temp.path.join(format!("{label}-{command}-runtime"));
            expect_startup_failure(
                &yzx_bin,
                command,
                &config_home,
                &runtime,
                &check,
                reason,
                label,
            );
        }
    }

    for (name, sidecar_text, reason) in [
        (
            "bad-zellij-plugin-top-level",
            "keybinds {\n}\n",
            "Zellij plugin sidecar supports only top-level `plugins` and `load_plugins`, found `keybinds`",
        ),
        (
            "bad-zellij-plugin-owned-id",
            "plugins {\n    yzpp location=\"file:/tmp/owned.wasm\"\n}\n",
            "Zellij plugin sidecar plugins entry `yzpp` is owned by Yazelix",
        ),
        (
            "bad-zellij-plugin-radar-id",
            "plugins {\n    radar location=\"file:/tmp/other-radar.wasm\"\n}\n",
            "Zellij plugin sidecar plugins entry `radar` is owned by Yazelix",
        ),
        (
            "bad-zellij-plugin-radar-controller-id",
            "load_plugins {\n    radar_controller\n}\n",
            "Zellij plugin sidecar load_plugins entry `radar_controller` is owned by Yazelix",
        ),
    ] {
        let case = RuntimeCase::new(&temp.path, name);
        case.write_default_config("");
        let plugins = case.config_home.join("zellij/plugins.kdl");
        fs::create_dir_all(plugins.parent().unwrap()).unwrap();
        fs::write(&plugins, sidecar_text).unwrap();
        for command in ["enter", "status", "doctor"] {
            expect_startup_failure(
                &yzx_bin,
                command,
                &case.config_home,
                &case.state_dir,
                &plugins,
                reason,
                name,
            );
        }
    }

    let state_file = temp.path.join("state-file");
    fs::write(&state_file, "").unwrap();
    expect_startup_failure(
        &yzx_bin,
        "doctor",
        &temp.path.join("state-config"),
        &state_file,
        &state_file,
        "state path is not a directory",
        "unwritable state",
    );
}

fn expect_menu_descriptions_match_help(help: &str, menu: &str) {
    for (id, label) in menu.lines().filter_map(menu_command_line) {
        assert!(
            help.lines().any(|line| {
                line.trim_start()
                    .strip_prefix(id)
                    .is_some_and(|rest| rest.trim_start() == label)
            }),
            "yzx menu command `{id}` description drifted from yzx help"
        );
    }
}

fn menu_command_line(line: &str) -> Option<(&str, &str)> {
    let (_, command) = line.trim_start().split_once('.')?;
    let trimmed = command.trim_start();
    let (id, label) = trimmed.split_once(char::is_whitespace)?;
    Some((id, label.trim_start()))
}

fn expect_startup_failure(
    yzx_bin: &Path,
    command: &str,
    config_home: &Path,
    runtime: &Path,
    check: &Path,
    reason: &str,
    label: &str,
) {
    if !runtime.exists() {
        fs::create_dir_all(runtime).unwrap();
    }
    let output = Command::new(yzx_bin)
        .arg(command)
        .env("YAZELIX_CONFIG_HOME", config_home)
        .env("YAZELIX_STATE_DIR", runtime)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "yzx {command} unexpectedly succeeded with config {}\nstdout:\n{}\nstderr:\n{}",
        config_home.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    expect_contains_all! {
        stderr.as_ref(), &format!("{label} {command} diagnostic");
        "Yazelix Nova could not start.",
        "Reason:",
        reason,
        "Check:",
        check.to_str().unwrap(),
    }
    if command == "doctor" {
        assert!(
            stdout.is_empty(),
            "{label} doctor duplicated its startup failure on stdout: {stdout}"
        );
    }
}

fn run_help(bin: &Path, args: &[&str]) -> String {
    successful_stdout(Command::new(bin).args(args), "yzx help")
}

fn run_nu(yzx_nu: &Path, config_home: &Path, runtime: &Path, commands: &str) -> String {
    run_nu_with_path(yzx_nu, config_home, runtime, commands, Path::new(""))
}

fn run_nu_with_path(
    yzx_nu: &Path,
    config_home: &Path,
    runtime: &Path,
    commands: &str,
    path: &Path,
) -> String {
    fs::create_dir_all(runtime).unwrap();
    successful_stdout_trimmed(
        Command::new(yzx_nu)
            .arg("--commands")
            .arg(commands)
            .env("XDG_DATA_HOME", runtime)
            .env("XDG_CONFIG_HOME", runtime)
            .env("HOME", runtime)
            .env("YAZELIX_CONFIG_HOME", config_home)
            .env("YAZELIX_STATE_DIR", "")
            .env("STARSHIP_CONFIG", "ambient-starship.toml")
            .env("PATH", path),
        &yzx_nu.display().to_string(),
    )
}

fn expect_shell_selection(shell: &Path) {
    for program in ["bash", "zsh", "fish"] {
        let temp = TempDir::new();
        let config_home = temp.path.join("config");
        let home = temp.path.join("home");
        let (startup, startup_text, probe, user, user_init, user_probe) = match program {
            "bash" => (
                home.join(".bashrc"),
                "export YZX_USER_RC=bash\n",
                "session=no; [ -n \"${ATUIN_SESSION:-}\" ] && session=yes; search=no; declare -F __atuin_history >/dev/null && search=yes; binding=no; [[ \"$(bind -S)\" == *'\\C-r outputs'* ]] && binding=yes; printf '%s\\n' \"user=$YZX_USER_RC\" \"session=$session\" \"search=$search\" \"binding=$binding\" \"atuin=$(command -v atuin)\" \"sed=$(command -v sed)\"",
                "bash",
                "__atuin_history() { printf '%s\\n' user-atuin; }\n",
                "__atuin_history",
            ),
            "zsh" => (
                home.join(".zshrc"),
                "export YZX_USER_RC=$YZX_USER_RC-zsh\n",
                "session=no; [[ -n ${ATUIN_SESSION:-} ]] && session=yes; search=no; (( $+functions[_atuin_search] )) && search=yes; binding=no; [[ \"$(bindkey '^R')\" == *atuin-search* ]] && binding=yes; print -r -- \"user=$YZX_USER_RC\" \"session=$session\" \"search=$search\" \"binding=$binding\" \"atuin=$commands[atuin]\" \"sed=$commands[sed]\"",
                "env-zsh",
                "_atuin_search() { print -r -- user-atuin; }\n",
                "_atuin_search",
            ),
            "fish" => (
                home.join(".config/fish/config.fish"),
                "set -gx YZX_USER_RC fish\nset -g fish_greeting\n",
                "set session no; set -q ATUIN_SESSION; and set session yes; set search no; functions -q _atuin_search; and set search yes; set binding no; bind ctrl-r 2>/dev/null | string match -q '*_atuin_search*'; and set binding yes; echo user=$YZX_USER_RC session=$session search=$search binding=$binding atuin=(command -v atuin) sed=(command -v sed)",
                "fish",
                "function _atuin_search\n  echo user-atuin\nend\n",
                "_atuin_search",
            ),
            _ => unreachable!(),
        };
        fs::create_dir_all(startup.parent().unwrap()).unwrap();
        if program == "zsh" {
            let zdot = home.parent().unwrap();
            fs::write(zdot.join(".zshenv"), "YZX_USER_RC=env\nunset ZDOTDIR\n").unwrap();
        }
        fs::write(&startup, startup_text).unwrap();

        for (atuin, no_bind, session, search, binding) in [
            (true, false, "yes", "yes", "yes"),
            (true, true, "yes", "yes", "no"),
            (false, false, "no", "no", "no"),
        ] {
            write_shell_selection(&config_home, program, atuin);
            let output = run_selected_shell(shell, program, &config_home, &home, probe, no_bind);
            assert_shell_state(&output, user, session, search, binding);
        }

        write_shell_selection(&config_home, program, true);
        fs::write(&startup, user_init).unwrap();
        let user_owned = run_selected_shell(shell, program, &config_home, &home, user_probe, false);
        assert_eq!(
            user_owned, "user-atuin",
            "managed {program} replaced user Atuin"
        );
    }
}

fn write_shell_selection(config_home: &Path, program: &str, atuin: bool) {
    write_config_home(
        config_home,
        format!("[shell]\nprogram = \"{program}\"\natuin = {atuin}\n"),
    );
}

fn assert_shell_state(output: &str, user: &str, session: &str, search: &str, binding: &str) {
    for expected in [
        format!("user={user}"),
        format!("session={session}"),
        format!("search={search}"),
        format!("binding={binding}"),
        "atuin=/nix/store/".to_string(),
        "/bin/atuin".to_string(),
        "sed=/nix/store/".to_string(),
        "/bin/sed".to_string(),
    ] {
        expect_contains(output, &expected, "managed shell Atuin state");
    }
}

fn run_selected_shell(
    shell: &Path,
    program: &str,
    config_home: &Path,
    home: &Path,
    commands: &str,
    no_bind: bool,
) -> String {
    let mut command = Command::new(shell);
    command
        .args(["-c", commands])
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("YAZELIX_CONFIG_HOME", config_home)
        .env("PATH", "");
    if program == "zsh" {
        command.env("ZDOTDIR", home.parent().unwrap());
    }
    if no_bind {
        command.env("ATUIN_NOBIND", "1");
    }
    successful_stdout_trimmed(&mut command, &format!("yzx-shell dispatch to {program}"))
}

fn expect_rio_config(yzx: &Path) {
    let packaged_config = yzx.join("share/yazelix/rio/config.toml");
    let packaged_dark = yzx.join("share/yazelix/rio/themes/nova-dark.toml");
    let packaged_light = yzx.join("share/yazelix/rio/themes/nova-light.toml");
    let yzx_bin = yzx.join("bin/yzx");
    assert!(
        packaged_config.is_file(),
        "packaged Rio config is not a file: {}",
        packaged_config.display()
    );

    let launcher = binary_text(&yzx_bin);
    expect_contains_all! {
        &launcher, "runtime Rio config fragment";
        "YAZELIX_CONFIG_HOME",
        "RIO_CONFIG_HOME",
        "--app-id",
        "--theme-mode",
        "/bin/rio",
    }

    let temp = TempDir::new();
    let case = RuntimeCase::new(&temp.path, "rio");
    let legacy_mars = case.config_home.join("mars/config.toml");
    let legacy_cursors = case.config_home.join("cursors.toml");
    fs::create_dir_all(legacy_mars.parent().unwrap()).unwrap();
    fs::write(&legacy_mars, "# preserved Mars config\n").unwrap();
    fs::write(&legacy_cursors, "# preserved cursor config\n").unwrap();

    let status = case.prepared_status(&yzx_bin, "Rio config initialization");
    let rio_config = case.config_home.join("rio/config.toml");
    let rio_dark = case.config_home.join("rio/themes/nova-dark.toml");
    let rio_light = case.config_home.join("rio/themes/nova-light.toml");
    assert_eq!(
        fs::read_to_string(&rio_config).unwrap(),
        fs::read_to_string(&packaged_config).unwrap()
    );
    assert_eq!(
        fs::read_to_string(&rio_dark).unwrap(),
        fs::read_to_string(&packaged_dark).unwrap()
    );
    assert_eq!(
        fs::read_to_string(&rio_light).unwrap(),
        fs::read_to_string(&packaged_light).unwrap()
    );
    expect_contains_all! {
        &status, "Rio config status";
        "rio config: user",
        rio_config.display().to_string(),
    }
    let custom = format!(
        "{}\n# preserved user Rio config\n",
        fs::read_to_string(&rio_config).unwrap()
    );
    fs::write(&rio_config, &custom).unwrap();
    fs::write(&rio_dark, "# custom dark theme\n").unwrap();
    fs::write(&rio_light, "# custom light theme\n").unwrap();
    case.prepared_status(&yzx_bin, "Rio config preservation");
    assert_eq!(fs::read_to_string(rio_config).unwrap(), custom);
    assert_eq!(
        fs::read_to_string(rio_dark).unwrap(),
        "# custom dark theme\n"
    );
    assert_eq!(
        fs::read_to_string(rio_light).unwrap(),
        "# custom light theme\n"
    );
    assert_eq!(
        fs::read_to_string(legacy_mars).unwrap(),
        "# preserved Mars config\n"
    );
    assert_eq!(
        fs::read_to_string(legacy_cursors).unwrap(),
        "# preserved cursor config\n"
    );
}

fn expect_read_only_diagnostics(yzx: &Path) {
    fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>, std::time::SystemTime)> {
        let mut entries = Vec::new();
        if root.is_dir() {
            for entry in fs::read_dir(root).unwrap() {
                let path = entry.unwrap().path();
                let metadata = fs::symlink_metadata(&path).unwrap();
                let contents = if metadata.is_file() {
                    fs::read(&path).unwrap()
                } else if metadata.is_symlink() {
                    fs::read_link(&path)
                        .unwrap()
                        .as_os_str()
                        .as_encoded_bytes()
                        .to_vec()
                } else {
                    entries.extend(snapshot(&path));
                    Vec::new()
                };
                entries.push((path, contents, metadata.modified().unwrap()));
            }
        }
        entries.sort();
        entries
    }

    for state in [
        "fresh",
        "configured",
        "invalid-root",
        "invalid-sidecar",
        "invalid-state",
    ] {
        let temp = TempDir::new();
        let case = RuntimeCase::new(&temp.path, state);
        if state != "fresh" {
            case.write_default_config("\n[appearance]\nmode = \"light\"\n");
            fs::create_dir_all(case.zellij_path("config.kdl").parent().unwrap()).unwrap();
            fs::write(case.zellij_path("config.kdl"), "// previous runtime\n").unwrap();
            fs::write(case.zellij_path("permissions.kdl"), "\"foreign.wasm\" {}\n").unwrap();
            fs::create_dir_all(case.config_home.join("zellij")).unwrap();
            fs::write(
                case.config_home.join("zellij/config.kdl"),
                "scroll_buffer_size 1234\n",
            )
            .unwrap();
        }
        if state == "invalid-root" {
            case.write_config("[broken");
        } else if state == "invalid-state" {
            fs::remove_dir_all(&case.state_dir).unwrap();
            fs::write(&case.state_dir, "preserve this file").unwrap();
        } else if state == "invalid-sidecar" {
            fs::write(case.config_home.join("zellij/config.kdl"), "keybinds {}\n").unwrap();
        }
        let before = snapshot(&temp.path);
        for args in [
            &["doctor"][..],
            &["doctor", "--verbose"],
            &["status"],
            &["status", "--json"],
        ] {
            let output = case
                .yzx_command(&yzx.join("bin/yzx"), args[0])
                .args(&args[1..])
                .env("CODEX_HOME", temp.path.join("codex"))
                .output()
                .unwrap();
            assert_eq!(
                output.status.success(),
                !state.starts_with("invalid"),
                "{state} {args:?}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                snapshot(&temp.path) == before,
                "{state} {args:?} mutated diagnostic inputs"
            );
            if state == "fresh" && args[0] == "doctor" {
                expect_contains(
                    &String::from_utf8_lossy(&output.stdout),
                    "runtime files missing; initialized by yzx enter or yzx launch",
                    "fresh doctor guidance",
                );
            }
        }
        if !state.starts_with("invalid") {
            successful_output(
                case.yzx_command(&yzx.join("bin/yzx"), "run").arg("true"),
                "runtime preparation after diagnostics",
            );
            assert!(case.zellij_path("config.kdl").is_file());
            assert!(case.zellij_file("permissions.kdl").contains("ReadCliPipes"));
            if state == "configured" {
                assert!(
                    case.zellij_file("config.kdl")
                        .contains("scroll_buffer_size 1234")
                );
                assert!(
                    case.zellij_file("config.kdl")
                        .contains("host_theme_mode \"light\"")
                );
            }
        }
    }
}

fn expect_zellij_config_sidecar(yzx: &Path) {
    let packaged_config = yzx.join("share/yazelix/config.kdl");
    let helper = yzx.join("libexec/yazelix/yzx-zellij-config");
    let temp = TempDir::new();
    let sidecar = temp.path.join("config.kdl");

    let no_sidecar = run_zellij_config(&helper, &packaged_config, &sidecar);
    assert_eq!(no_sidecar, fs::read_to_string(&packaged_config).unwrap());

    let packaged_text = fs::read_to_string(&packaged_config).unwrap();
    assert!(packaged_text.contains("theme_dark \"ansi\""));
    assert!(packaged_text.contains("theme_light \"gruvbox-light\""));
    assert!(packaged_text.contains("pane_frame_style \"full\""));
    assert!(packaged_text.contains("stacked_pane_list false"));

    let sidecar_config = "# { preserved comment\ntheme \"dracula\"\nfuture_label \"{opaque}\"\ntheme_dark \"custom-dark\"\nscroll_buffer_size 1234\npane_frames false\n";
    fs::write(&sidecar, sidecar_config).unwrap();
    let generated = run_zellij_config(&helper, &packaged_config, &sidecar);
    let applied_sidecar = "# { preserved comment\nfuture_label \"{opaque}\"\ntheme_dark \"custom-dark\"\nscroll_buffer_size 1234\npane_frames false\n";
    let inherited_pair_removed = packaged_text.replace("theme_dark \"ansi\"\n", "");
    let expected_config = format!("{}\n{}", inherited_pair_removed.trim_end(), applied_sidecar);
    assert_eq!(generated, expected_config);
    assert_eq!(expected_config.matches("theme_dark ").count(), 1);
    assert_eq!(expected_config.matches("theme_light ").count(), 1);
    assert_eq!(fs::read_to_string(&sidecar).unwrap(), sidecar_config);

    for forbidden in [
        ("keybinds", "keybinds {}\n"),
        (
            "support_kitty_keyboard_protocol",
            "support_kitty_keyboard_protocol false\n",
        ),
        ("layout_dir", "layout_dir \"/tmp/layouts\"\n"),
        ("env", "env { YZX_OPEN_LOG \"off\" }\n"),
    ] {
        fs::write(&sidecar, forbidden.1).unwrap();
        let output = Command::new(&helper)
            .arg(&packaged_config)
            .arg(&sidecar)
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

fn expect_yazi_managed_keys(yzx: &Path) {
    let keymap = fs::read_to_string(yzx.join("share/yazelix/yazi/keymap.toml")).unwrap();
    expect_contains_all! {
        &keymap, "Yazi managed keymap fragment";
        r#"on = ["<A-z>"]"#,
        r#"run = "plugin zoxide-editor""#,
        r#"on = ["<A-r>"]"#,
        r#"run = 'shell "$YZX_YAZI_RETURN"'"#,
    }

    let yazi_toml = fs::read_to_string(yzx.join("share/yazelix/yazi/yazi.toml")).unwrap();
    expect_contains_all! {
        &yazi_toml, "Yazi config fragment";
        "YZX_ZELLIJ=",
        "url = \"*\"\nrun = \"git\"\ngroup = \"git\"",
        "url = \"*/\"\nrun = \"git\"\ngroup = \"git\"",
    }
    assert!(
        !yazi_toml.contains("YZX_EDITOR="),
        "packaged Yazi opener should inherit YZX_EDITOR from yzx-yazi"
    );

    let init = fs::read_to_string(yzx.join("share/yazelix/yazi/init.lua")).unwrap();
    assert!(!init.contains("sidebar-state") && !init.contains("sidebar-status"));
    assert!(
        !yzx.join("share/yazelix/yazi/plugins/sidebar-state.yazi")
            .exists()
    );
    assert!(
        !yzx.join("share/yazelix/yazi/plugins/sidebar-status.yazi")
            .exists()
    );
    assert!(
        yzx.join("share/yazelix/yazi/plugins/git.yazi").is_dir(),
        "packaged Yazi config is missing git.yazi",
    );

    let plugin =
        fs::read_to_string(yzx.join("share/yazelix/yazi/plugins/zoxide-editor.yazi/main.lua"))
            .unwrap();
    expect_contains_all! {
        &plugin, "Yazi zoxide editor plugin fragment";
        r#":arg({ "--retarget-workspace", target_dir })"#,
        r#"Command("zoxide")"#,
        r#"emit("cd", { target_dir, raw = true })"#,
        "YZX_OPEN is not set",
    }

    let layout = fs::read_to_string(yzx.join("share/yazelix/layout.kdl")).unwrap();
    expect_contains_all! {
        &layout, "packaged dark appearance layout";
        r#"plugin location="radar""#,
        r#"pane name="yazi_picker" command="/nix/store/"#,
        r#"args "--yzx-startup-picker""#,
    }
    assert!(
        !layout.contains("floating_panes {")
            && !layout.contains(r#"pane name="editor" command="#)
            && !layout.contains(r#"pane name="sidebar" command="#),
        "packaged layout kept a floating picker, prestarted editor, or tiled Yazi sidebar"
    );
    let config = fs::read_to_string(yzx.join("share/yazelix/config.kdl")).unwrap();
    expect_contains_all! {
        &config, "packaged dark appearance bar controller";
        r#"host_theme_mode "dark""#,
        r##"host_theme_dark_tab_normal "#[fg=#ffff00] [{index}] {name} ""##,
        r##"host_theme_light_tab_normal "#[fg=#5c5f77] [{index}] {name} ""##,
    }
    let yzx_yazi = popup_command(&config, "/bin/yzx-yazi");
    let public_yzx_yazi = yzx.join("bin/yzx-yazi");
    assert_eq!(
        fs::canonicalize(&public_yzx_yazi).unwrap(),
        fs::canonicalize(&yzx_yazi).unwrap(),
        "the public yzx-yazi command must be Nova's managed Yazi wrapper"
    );
    assert_eq!(
        layout
            .matches(&format!(
                r#"pane name="yazi_picker" command="{}""#,
                yzx_yazi.display()
            ))
            .count(),
        2,
        "startup and new-tab templates must launch the configured Yazi picker command"
    );
    let wrapper = binary_text(&yzx_yazi);
    let materializer = embedded_store_path(&wrapper, "/bin/yzx-yazi-config");
    assert!(materializer.is_file());
    let context = format!("{} Yazi integration fragment", yzx_yazi.display());
    expect_contains_all! {
        &wrapper, &context;
        "YZX_OPEN",
        "YZX_YAZI_RETURN",
        "YZX_ZELLIJ",
        "YZX_EDITOR",
        "YAZELIX_EDITOR",
        "GIT_EDITOR",
        "editor.command",
        "appearance.mode",
        "--yzx-workspace-popup",
        "--yzx-startup-picker",
        "YZX_YAZI_ROLE",
        "YZX_YAZI_BIN",
        "YZX_APPEARANCE_MODE",
        "YZX_APPEARANCE_LIVE",
        "workspace-popup",
        "YAZI_CONFIG_HOME",
        "/bin/yzx-yazi-config",
        "/bin/yzx-yazi-return",
        "yazelix_starship.toml",
        "YAZELIX_ZELLIJ_SESSION_NAME",
        "ZELLIJ_SESSION_NAME",
        "KITTY_WINDOW_ID",
        "git",
        "zoxide",
        "fzf",
    }
}

fn run_zellij_config(helper: &Path, packaged_config: &Path, sidecar: &Path) -> String {
    successful_stdout(
        Command::new(helper).arg(packaged_config).arg(sidecar),
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

fn expect_session_config(config: &str) {
    assert_eq!(
        config
            .lines()
            .filter(|line| line.trim() == r#"default_layout "layout""#)
            .count(),
        1,
        "config.kdl must select the managed layout for native session creation",
    );
    let layout_dir = config
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(r#"layout_dir ""#)?
                .strip_suffix('"')
                .map(PathBuf::from)
        })
        .expect("config.kdl is missing layout_dir");
    assert!(
        layout_dir.join("layout.kdl").is_file(),
        "managed layout_dir is missing layout.kdl: {}",
        layout_dir.display(),
    );
    assert!(
        layout_dir.join("layout.swap.kdl").is_file(),
        "managed layout_dir is missing layout.swap.kdl: {}",
        layout_dir.display(),
    );
}

fn expect_keybinds(config: &str) {
    for expected in [
        r#"unbind "Alt i" "Alt o" "Ctrl g""#,
        r#"bind "Alt m" { NewPane; }"#,
        r#"bind "Alt Shift W" { CloseTab; SwitchToMode "Normal"; }"#,
        r#"bind "Alt h" "Alt Left" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_left_or_tab"; }; }"#,
        r#"bind "Alt j" "Alt Down" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_down"; }; }"#,
        r#"bind "Alt k" "Alt Up" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_up"; }; }"#,
        r#"bind "Alt l" "Alt Right" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_right_or_tab"; }; }"#,
        r#"bind "Ctrl Alt n" { MessagePlugin "radar_controller" { name "zj_radar.cmd.v1"; payload "attention-next"; }; }"#,
        r#"bind "Ctrl Alt p" { MessagePlugin "radar_controller" { name "zj_radar.cmd.v1"; payload "attention-prev"; }; }"#,
        r#"bind "Ctrl Tab" { MessagePlugin "radar_controller" { name "zj_radar.cmd.v1"; payload "session-next"; }; }"#,
        r#"bind "Ctrl Shift Tab" { MessagePlugin "radar_controller" { name "zj_radar.cmd.v1"; payload "session-prev"; }; }"#,
        r#"bind "Alt Shift F" { ToggleFocusFullscreen; }"#,
        r#"bind "Alt Shift A" {"#,
        r#"bind "Alt Shift H" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar"; }; }"#,
        r#"bind "Ctrl Alt g" { SwitchToMode "Locked"; }"#,
        r#"bind "Ctrl p" { SwitchToMode "Pane"; }"#,
        r#"unbind "Ctrl t""#,
        r#"bind "Ctrl Alt t" { SwitchToMode "Tab"; }"#,
        r#"bind "Ctrl Alt t" { SwitchToMode "Normal"; }"#,
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
        !config.contains("smart_reveal") && !config.contains(r#"bind "Alt r""#),
        "config.kdl must leave Alt r to the focused application"
    );
    assert!(
        !config.lines().any(|line| {
            let line = line.trim();
            line.starts_with("bind ") && quoted_keys(line).any(|key| key == "Ctrl t")
        }),
        "ZELLIJ-TAB-MODE-CHORD-001: Ctrl t must remain available to the focused application"
    );
    assert_eq!(config.matches("ToggleFocusFullscreen").count(), 1);
    assert!(!config.contains("toggle_editor_sidebar_focus"));
    for tab in 1..=9 {
        let expected = format!(r#"bind "Alt {tab}" {{ GoToTab {tab}; }}"#);
        assert!(
            config.lines().any(|line| line.trim() == expected),
            "config.kdl is missing {expected}",
        );
    }
    for key in ["n", "Alt Shift T"] {
        assert!(
            config.lines().any(|line| {
                let line = line.trim();
                line.starts_with(&format!(r#"bind "{key}" {{ NewTab {{ layout "/nix/store/"#))
                    && line.ends_with(
                        r#"/layout.kdl"; cwd "__YZX_HOME__"; }; SwitchToMode "Normal"; }"#,
                    )
            }),
            "NOVA-DIRECT-TABS-001: {key} must create a managed tab with a runtime home cwd",
        );
    }
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

fn expect_first_party_plugins(git_bin: &Path, config: &str) {
    expect_contains_all! {
        config, "config.kdl first-party plugin fragment";
        "share/yazelix_zellij_popup/yzpp.wasm",
        "share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm",
        r#"yazelix_pane_orchestrator location="file:/nix/store/"#,
        r#"radar_controller location="file:/nix/store/"#,
        "role \"view\"",
        "role \"controller\"",
        "load_plugins {\n    yzpp\n    radar_controller",
        "load_plugins",
        "support_kitty_keyboard_protocol true",
        "screen_saver_enabled false",
        "popup_plugin_url \"yzpp\"",
        "managed_agent_command_marker \"/bin/yzx-agent\"",
    }
    expect_popup_defaults(config, "1", "0", "packaged popup config");
    for (id, pane_title, command_suffix, extra) in [
        (
            "config",
            "config_popup",
            "/bin/yzx-config-ui",
            "\n                toggle_close_behavior \"hide\"",
        ),
        (
            "agent",
            "agent_popup",
            "/bin/yzx-agent",
            "\n                command_marker \"/bin/yzx-agent\"\n                preserve_terminal_title true\n                toggle_close_behavior \"hide\"",
        ),
        ("git", "git_popup", "/bin/yzx-git", ""),
        ("menu", "menu_popup", "/bin/yzx-menu", ""),
        (
            "screen",
            "anima",
            "/bin/anima",
            "\n                arg_1 \"random\"",
        ),
        (
            "yazi",
            "yazi_popup",
            "/bin/yzx-yazi",
            "\n                arg_1 \"--yzx-workspace-popup\"\n                toggle_close_behavior \"hide\"\n                preserve_on_cwd_change true",
        ),
    ] {
        let command = popup_command(config, command_suffix);
        let expected = format!(
            "{id} {{\n                command \"{}\"\n                pane_title \"{pane_title}\"{extra}\n            }}",
            command.display()
        );
        assert!(
            config.contains(&expected),
            "config.kdl is missing {id} popup block\n{expected}",
        );
    }
    assert!(
        !config.contains("width_percent") && !config.contains("height_percent"),
        "packaged popup config must not use removed percentage fields",
    );
    assert_eq!(config.matches("side_margin 1").count(), 1);
    assert_eq!(config.matches("left_margin 33").count(), 1);
    assert_eq!(
        config.matches("left_margin_pane_title \"sidebar\"").count(),
        1
    );
    assert_eq!(config.matches("vertical_margin 0").count(), 1);
    for (key, payload) in [
        ("Alt Shift J", "git"),
        ("Alt Shift K", "config"),
        ("Alt Shift L", "agent"),
        ("Alt Shift M", "menu"),
        ("Alt Shift A", "screen"),
        ("Alt Shift Y", "yazi"),
    ] {
        expect_popup_binding(config, key, payload, "packaged popup config");
    }

    let agent = popup_command(config, "/bin/yzx-agent");
    expect_agent_bootstrap(&agent);

    let git = popup_command(config, "/bin/yzx-git");
    let git_script = fs::read_to_string(&git).unwrap();
    let context = format!("{} managed Git popup wrapper", git.display());
    expect_contains_all! {
        &git_script, &context;
        "/bin/lazygit",
        "LG_CONFIG_FILE",
        "--print-config-dir",
    }
    let editor = embedded_store_path(&git_script, "/bin/yzx-editor");
    let lazygit_config = embedded_store_path(&git_script, "-yzx-lazygit.yml");
    expect_git_editor(&editor, &lazygit_config, git_bin);

    let config_ui = popup_command(config, "/bin/yzx-config-ui");
    let config_ui_script = fs::read_to_string(&config_ui).unwrap();
    let context = format!("{} config UI wrapper", config_ui.display());
    expect_contains_all! {
        &config_ui_script, &context;
        "/bin/yzx-editor",
        "GIT_EDITOR",
        "unset YAZELIX_EDITOR",
    }
    assert!(
        !config_ui_script.contains("/bin/yzx-hx"),
        "config UI bypasses the selected editor\n{config_ui_script}"
    );

    assert!(popup_command(config, "/bin/yzx-menu").is_file());
    assert!(popup_command(config, "/bin/anima").is_file());
}

fn expect_git_editor(editor: &Path, lazygit_config: &Path, git: &Path) {
    let config = fs::read_to_string(lazygit_config).unwrap();
    assert_eq!(
        config.matches("/bin/yzx-editor {{filename}}").count(),
        3,
        "LazyGit file edits bypass yzx-editor\n{config}"
    );
    expect_contains_all! {
        &config, "managed LazyGit editor config";
        "editInTerminal: true",
        "/bin/yzx-editor {{dir}}",
    }

    let temp = TempDir::new();
    let git_editor = temp.path.join("git-editor");
    write_executable(
        &git_editor,
        "#!/bin/sh\n[ \"$YAZELIX_HELIX_BRIDGE\" = 0 ] || exit 64\nprintf '%s\\n' 'configured editor commit' >\"$1\"\n",
    );
    let git_config = temp.path.join("git-config");
    write_config_home(
        &git_config,
        format!("[editor]\ncommand = \"{}\"\n", git_editor.display()),
    );
    let repo = temp.path.join("repo with spaces");
    successful_output(Command::new(git).arg("init").arg(&repo), "Git init");
    let output = successful_output(
        Command::new(git)
            .arg("-C")
            .arg(&repo)
            .args([
                "-c",
                "user.name=Yazelix Test",
                "-c",
                "user.email=yazelix@example.invalid",
                "commit",
                "--allow-empty",
            ])
            .env("GIT_EDITOR", editor)
            .env("ZELLIJ", "test-session")
            .env("YAZELIX_CONFIG_HOME", &git_config)
            .env_remove("YAZELIX_EDITOR"),
        "Git commit through configured editor",
    );
    assert!(
        output
            .stdout
            .windows(b"\x1b]111\x07".len())
            .any(|window| window == b"\x1b]111\x07"),
        "yzx-editor did not restore Zellij's default background",
    );
}

fn expect_popup_binding(config: &str, key: &str, payload: &str, context: &str) {
    let (plugin, action) = if matches!(payload, "git" | "agent" | "yazi") {
        ("yazelix_pane_orchestrator", "toggle_workspace_popup")
    } else {
        ("yzpp", "toggle")
    };
    let expected = format!(
        "bind \"{key}\" {{\n            MessagePlugin \"{plugin}\" {{\n                name \"{action}\"\n                payload \"{payload}\"\n            }}\n        }}"
    );
    assert!(
        config.contains(&expected),
        "{context} is missing {key} popup binding\n{expected}",
    );
}

fn expect_popup_defaults(config: &str, side_margin: &str, vertical_margin: &str, context: &str) {
    let expected = format!(
        "popup_defaults {{\n            side_margin {side_margin}\n            left_margin 33\n            vertical_margin {vertical_margin}\n        }}",
    );
    expect_contains(config, &expected, context);
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
    let mut blocks = Vec::<(i32, KeyBlock)>::new();
    let mut depth = 0i32;
    for (line_number, line) in config.lines().map(str::trim).enumerate() {
        if opens_keybind_block(line) {
            blocks.push((depth + 1, KeyBlock::default()));
        }
        if let Some((_, block)) = blocks.last_mut() {
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
        depth += line.matches('{').count() as i32 - line.matches('}').count() as i32;
        while blocks
            .last()
            .is_some_and(|(block_depth, _)| *block_depth > depth)
        {
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

fn expect_agent_bootstrap(agent: &Path) {
    let temp = TempDir::new();
    let empty_state = temp.path.join("empty-state");
    let output = Command::new(agent)
        .env("PATH", "")
        .env("YAZELIX_STATE_DIR", &empty_state)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "agent popup without providers should exit cleanly, got {:?}",
        output.status.code(),
    );
    assert_eq!(output.stdout, b"\x1b]0;agent popup\x07");
    assert!(output.stderr.is_empty());
    assert!(
        !empty_state.join("agent/provider").exists(),
        "missing-provider bootstrap should not write a provider default"
    );

    let title_agent = temp.path.join("title-agent");
    write_executable(
        &title_agent,
        "#!/bin/sh\nprintf '\\033]0;⠋ codex\\007\\033]0;codex\\007'\nprintf '%s\\n' \"$*\" >\"$YAZELIX_AGENT_TEST_OUT\"\n",
    );
    let title_output_file = temp.path.join("title-agent-output");
    let output = Command::new(agent)
        .arg(&title_agent)
        .args(["resume", "session"])
        .env("YAZELIX_AGENT_TEST_OUT", &title_output_file)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        "\x1b]0;agent popup\x07\x1b]0;⠋ codex\x07\x1b]0;codex\x07".as_bytes(),
        "the fallback must precede provider busy and idle titles",
    );
    assert_eq!(
        fs::read_to_string(title_output_file).unwrap(),
        "resume session\n"
    );

    for (name, available, expected_output) in [
        (
            "codex-first",
            &["codex", "opencode"][..],
            "radar setup codex --check\ncodex resume\n",
        ),
        ("grok-fallback", &["grok", "opencode"], "grok\n"),
        (
            "opencode-fallback",
            &["opencode", "pi", "claude"],
            "opencode\n",
        ),
        ("pi-fallback", &["pi", "claude"], "pi\n"),
        ("claude-fallback", &["claude"], "claude --resume\n"),
    ] {
        expect_agent_bootstrap_case(agent, &temp.path, name, available, expected_output);
    }

    let missing_hooks_bin = temp.path.join("missing-hooks-bin");
    fs::create_dir(&missing_hooks_bin).unwrap();
    write_fake_agent(&missing_hooks_bin, "codex");
    write_fake_radar(&missing_hooks_bin);
    let missing_hooks_state = temp.path.join("missing-hooks-state");
    let missing_hooks_output = temp.path.join("missing-hooks-output");
    let output = successful_output(
        Command::new(agent)
            .env("PATH", &missing_hooks_bin)
            .env("YAZELIX_STATE_DIR", &missing_hooks_state)
            .env("YAZELIX_AGENT_TEST_OUT", &missing_hooks_output)
            .env("YAZELIX_AGENT_TEST_RADAR_FAIL", "1"),
        "noninteractive Codex launch with missing Radar hooks",
    );
    assert_eq!(
        fs::read_to_string(&missing_hooks_output).unwrap(),
        "radar setup codex --check\ncodex resume\n"
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("Enable Codex activity in Radar?"),
        "noninteractive launch must not prompt"
    );
    assert!(
        !missing_hooks_state
            .join("agent/radar-codex-setup-offered")
            .exists(),
        "a skipped noninteractive offer must not be remembered"
    );

    let custom_sidebar_state = temp.path.join("custom-sidebar-agent-state");
    let custom_sidebar_output = temp.path.join("custom-sidebar-agent-output");
    successful_output(
        Command::new(agent)
            .env("PATH", &missing_hooks_bin)
            .env("YAZELIX_STATE_DIR", &custom_sidebar_state)
            .env("YAZELIX_AGENT_TEST_OUT", &custom_sidebar_output)
            .env("YAZELIX_AGENT_TEST_RADAR_FAIL", "1")
            .env("YZX_RADAR_ENABLED", "false"),
        "Codex launch with a custom sidebar",
    );
    assert_eq!(
        fs::read_to_string(&custom_sidebar_output).unwrap(),
        "codex resume\n"
    );
    assert!(
        !custom_sidebar_state
            .join("agent/radar-codex-setup-offered")
            .exists(),
        "a custom sidebar must not trigger or remember Radar setup"
    );

    let explicit_codex_bin = temp.path.join("explicit-codex-bin");
    let pinned_radar_bin = temp.path.join("pinned-radar-bin");
    fs::create_dir(&explicit_codex_bin).unwrap();
    fs::create_dir(&pinned_radar_bin).unwrap();
    write_fake_agent(&explicit_codex_bin, "codex");
    write_executable(
        &explicit_codex_bin.join("zj-radar"),
        "#!/bin/sh\nprintf 'wrong radar %s\\n' \"$*\" >>\"$YAZELIX_AGENT_TEST_OUT\"\n",
    );
    write_fake_radar(&pinned_radar_bin);
    let explicit_state = temp.path.join("explicit-codex-state");
    let explicit_output = temp.path.join("explicit-codex-output");
    successful_output(
        Command::new(agent)
            .arg(explicit_codex_bin.join("codex"))
            .env("PATH", &pinned_radar_bin)
            .env("YAZELIX_STATE_DIR", &explicit_state)
            .env("YAZELIX_AGENT_TEST_OUT", &explicit_output),
        "explicit Codex path with sibling Radar",
    );
    assert_eq!(
        fs::read_to_string(&explicit_output).unwrap(),
        "radar setup codex --check\ncodex\n",
        "an explicit Codex directory shadowed Nova's Radar CLI"
    );

    let persisted_state = temp.path.join("persisted-state");
    let persisted_agent = persisted_state.join("agent");
    fs::create_dir_all(&persisted_agent).unwrap();
    fs::write(persisted_agent.join("provider"), "opencode\n").unwrap();
    let persisted_bin = temp.path.join("persisted-bin");
    fs::create_dir(&persisted_bin).unwrap();
    write_fake_agent(&persisted_bin, "codex");
    write_fake_agent(&persisted_bin, "opencode");
    let output_file = temp.path.join("persisted-output");
    successful_output(
        Command::new(agent)
            .env("PATH", &persisted_bin)
            .env("YAZELIX_STATE_DIR", &persisted_state)
            .env("YAZELIX_AGENT_TEST_OUT", &output_file),
        "agent popup persisted provider",
    );
    assert_eq!(fs::read_to_string(&output_file).unwrap(), "opencode\n");

    let missing_state = temp.path.join("missing-state");
    let missing_agent = missing_state.join("agent");
    fs::create_dir_all(&missing_agent).unwrap();
    fs::write(missing_agent.join("provider"), "opencode\n").unwrap();
    let output = Command::new(agent)
        .env("PATH", temp.path.join("missing-bin"))
        .env("YAZELIX_STATE_DIR", &missing_state)
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(127),
        "agent popup with a configured missing provider should exit 127, got {:?}",
        output.status.code(),
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Configured agent provider `opencode` is not available on PATH"),
        "agent popup configured-missing output is unclear: {stderr}",
    );
}

fn expect_agent_bootstrap_case(
    agent: &Path,
    root: &Path,
    name: &str,
    available: &[&str],
    expected_output: &str,
) {
    let bin = root.join(format!("{name}-bin"));
    fs::create_dir(&bin).unwrap();
    for provider in available {
        write_fake_agent(&bin, provider);
    }
    write_fake_radar(&bin);

    let state = root.join(format!("{name}-state"));
    let output_file = root.join(format!("{name}-output"));
    successful_output(
        Command::new(agent)
            .env("PATH", &bin)
            .env("YAZELIX_STATE_DIR", &state)
            .env("YAZELIX_AGENT_TEST_OUT", &output_file),
        &format!("agent popup {name} bootstrap"),
    );
    assert_eq!(fs::read_to_string(&output_file).unwrap(), expected_output);
    assert_eq!(
        fs::read_to_string(state.join("agent/provider")).unwrap(),
        format!("{}\n", available[0])
    );
    if available[0] == "codex" {
        assert!(
            state.join("agent/radar-codex-setup-offered").is_file(),
            "a healthy first Codex check must suppress later offers"
        );
        fs::write(&output_file, "").unwrap();
        successful_output(
            Command::new(agent)
                .env("PATH", &bin)
                .env("YAZELIX_STATE_DIR", &state)
                .env("YAZELIX_AGENT_TEST_OUT", &output_file),
            "second Codex launch",
        );
        assert_eq!(
            fs::read_to_string(&output_file).unwrap(),
            "codex resume\n",
            "the second Codex launch must not check or offer setup again"
        );
    }
}

fn write_fake_agent(bin: &Path, name: &str) {
    let path = bin.join(name);
    write_executable(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$#\" -eq 0 ]; then\n  printf '%s\\n' \"{name}\" >>\"$YAZELIX_AGENT_TEST_OUT\"\nelse\n  printf '%s %s\\n' \"{name}\" \"$*\" >>\"$YAZELIX_AGENT_TEST_OUT\"\nfi\n"
        ),
    );
}

fn write_fake_radar(bin: &Path) {
    write_executable(
        &bin.join("zj-radar"),
        "#!/bin/sh\nprintf 'radar %s\\n' \"$*\" >>\"$YAZELIX_AGENT_TEST_OUT\"\n[ \"${YAZELIX_AGENT_TEST_RADAR_FAIL:-}\" != 1 ]\n",
    );
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

fn successful_stdout_trimmed(command: &mut Command, context: &str) -> String {
    successful_stdout(command, context)
        .trim_end_matches('\n')
        .to_owned()
}
