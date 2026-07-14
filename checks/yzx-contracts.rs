use std::{
    env, fs,
    io::Write,
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
    let config = fs::read_to_string(yzx.join("share/yazelix/config.kdl")).unwrap();
    let yzx_shell = default_shell(&config);
    assert!(
        yzx_shell.is_file(),
        "default_shell is not a file: {}",
        yzx_shell.display()
    );
    expect_keybinds(&config);
    expect_first_party_plugins(git, &config);
    expect_front_door(yzx, Path::new(jq));
    expect_headless_enter(yzx);
    expect_narrow_path_launches(yzx, &yzx_shell);
    expect_config_ui(yzx);
    expect_startup_diagnostics(yzx);
    expect_mars_config_override(yzx);
    expect_cursor_config(yzx);
    expect_zellij_config_sidecar(yzx);
    expect_yazi_alt_z(yzx);

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
        "format = \"$character\"\nright_format = \"::<>\"\n",
    )
    .unwrap();

    let stdout = run_nu(
        &yzx_shell,
        &user_config,
        &runtime,
        "print $env.STARSHIP_SHELL; print $env.STARSHIP_CONFIG; print (do $env.PROMPT_COMMAND_RIGHT); print $env.YZX_USER_ENV_TEST; print $env.YZX_USER_CONFIG_TEST; ^carapace --version | ignore; ^zoxide --version | ignore; print ok",
    );
    assert_eq!(
        stdout,
        format!(
            "nu\n{}\n::<>\nenv-ok\nconfig-ok\nok",
            runtime.join("yazelix/starship.toml").display()
        )
    );
    let effective_starship = fs::read_to_string(runtime.join("yazelix/starship.toml")).unwrap();
    expect_contains_all! {
        &effective_starship, "effective user Starship config";
        "format = \"$character\"",
        "right_format = \"::<>\"",
        "add_newline = true",
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
    expect_contains_all! {
        &fallback_starship, "effective default Starship config";
        "format = \":: \"",
        "right_format = \"\"",
        "add_newline = true",
    }

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
        ],
        "managed Nu mise layering",
    );
    fs::write(out, "ok\n").unwrap();
}

fn expect_front_door(yzx: &Path, jq: &Path) {
    let yzx_bin = yzx.join("bin/yzx");
    let desktop = fs::read_to_string(yzx.join("share/applications/yzx.desktop")).unwrap();
    assert!(
        desktop.lines().any(|line| {
            line.starts_with("Exec=/nix/store/") && line.ends_with("/bin/yzx launch")
        }),
        "desktop entry must launch explicitly\n{desktop}"
    );
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
        "yzx doctor",
        "yzx env",
        "yzx enter [zellij-args...]",
        "yzx launch [zellij-args...]",
        "yzx menu",
        "yzx tutor [lesson]",
        "yzx reveal <target>",
        "yzx screen [style]",
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
            "config", "doctor", "status", "screen", "launch", "help", "tutor"
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
    let screen_help = run_help(&yzx_bin, &["screen", "--help"]);
    expect_contains_all! {
        &screen_help, "yzx screen help";
        "yzx screen [STYLE]",
        "static",
        "logo",
        "boids_schools",
        "game_of_life_gliders",
        "mandelbrot",
        "random",
        "--cell-style",
        "--duration-seconds",
    }
    let tutor_help = run_help(&yzx_bin, &["tutor", "--help"]);
    expect_contains_all! {
        &tutor_help, "yzx tutor help";
        "yzx tutor",
        "yzx tutor begin",
        "yzx tutor list",
        "yzx tutor workspace",
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
        "yzx tutor discovery",
        "yzx tutor troubleshooting",
        "yzx tutor tool_tutors",
    }
    for (lesson, expected) in [
        ("begin", "Workspace roots and managed panes"),
        ("workspace", "current tab workspace root matters most"),
        ("discovery", "Alt Shift M"),
        ("troubleshooting", "yzx doctor"),
        ("tool_tutors", "print the packaged Helix tutor command"),
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
    }
    let nushell_tutor = run_help(&yzx_bin, &["tutor", "nu"]);
    expect_contains_all! {
        &nushell_tutor, "yzx tutor nu";
        "/bin/nu -c 'tutor begin'",
        "tutor begin",
    }

    let yzx_launcher = binary_text(&yzx_bin);
    let menu_helper = embedded_store_path(&yzx_launcher, "/bin/yzx-menu");
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
        "YZX_MENU_YZX",
        "YZX_YA",
        "YZX_ZELLIJ",
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
        "bar.widgets",
        "popup.side_margin",
        "popup.vertical_margin",
        "popups.kdl",
        "popups.keybindings.kdl",
        "keybindings.config",
        "keybindings.agent",
        "keybindings.git",
        "keybindings.menu",
        "lazygit",
        "yzx-bar-render",
        "yzx-env-supervisor",
        "yzx-tutor",
        "yzx-welcome",
        "yzx-shell",
        "yzx-reveal",
        "/bin/yzs",
        "yazelix_pane_orchestrator.wasm",
        "/bin/ya",
        "/bin/zellij",
        "/bin/mars",
        "tokenusage",
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
    let status = status_case.run_yzx(&yzx_bin, "status", "yzx status");
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
        "layout: packaged (/nix/store/",
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

    let permissions = status_case.zellij_file("permissions.kdl");
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
        &permissions, "runtime plugin permissions";
        "yazelix_pane_orchestrator.wasm",
        "MessageAndLaunchOtherPlugins",
        "ReadSessionEnvironmentVariables",
    }

    let custom_popup = RuntimeCase::new(&temp.path, "custom-popup");
    custom_popup.write_default_config("\n[popup]\nside_margin = 2\nvertical_margin = 1\n");
    let status = custom_popup.run_yzx(&yzx_bin, "status", "custom popup status");
    expect_contains_all! {
        &status, "custom popup status";
        "popup side margin: 2",
        "popup vertical margin: 1",
        "zellij config: runtime (",
        "layout: packaged (/nix/store/",
    }
    let custom_popup_config = custom_popup.zellij_file("config.kdl");
    expect_popup_defaults(&custom_popup_config, "2", "1", "custom popup status config");
    assert_eq!(custom_popup_config.matches("width_percent 100").count(), 4);
    assert_eq!(custom_popup_config.matches("height_percent 100").count(), 4);
    assert_eq!(custom_popup_config.matches("side_margin 2").count(), 1);
    assert_eq!(custom_popup_config.matches("vertical_margin 1").count(), 1);

    let custom_agent = RuntimeCase::new(&temp.path, "custom-agent");
    custom_agent.write_default_config("\n[agent]\ncommand = \"codex\"\nargs = [\"resume\", \"--dangerously-bypass-approvals-and-sandbox\"]\n");
    let status = custom_agent.run_yzx(&yzx_bin, "status", "custom agent status");
    expect_contains_all! {
        &status, "custom agent status";
        "agent command: codex",
        r#"agent args: ["resume","--dangerously-bypass-approvals-and-sandbox"]"#,
        "zellij config: runtime (",
    }
    let custom_agent_config = custom_agent.zellij_file("config.kdl");
    expect_contains(
        &custom_agent_config,
        "agent {\n                command \"codex\"\n                arg_1 \"resume\"\n                arg_2 \"--dangerously-bypass-approvals-and-sandbox\"\n                pane_title \"agent_popup\"\n                width_percent 100\n                height_percent 100\n                toggle_close_behavior \"hide\"\n            }",
        "custom agent config",
    );

    let custom_popup_spec_case = RuntimeCase::new(&temp.path, "custom-popup-spec");
    custom_popup_spec_case.write_default_config("\n[popup]\nside_margin = 2\nvertical_margin = 1\n\n[popups.btm]\ncommand = \"btm\"\nargs = [\"--basic\"]\ntitle = \"btm_popup\"\nkeybinding = \"Alt Shift B\"\nkeep_alive = true\n");
    custom_popup_spec_case.run_yzx(&yzx_bin, "status", "custom popup spec status");
    let custom_popup_spec = custom_popup_spec_case.zellij_file("config.kdl");
    expect_contains(
        &custom_popup_spec,
        "btm {\n                command \"btm\"\n                arg_1 \"--basic\"\n                pane_title \"btm_popup\"\n                command_marker \"btm_popup\"\n                width_percent 100\n                height_percent 100\n                toggle_close_behavior \"hide\"\n            }",
        "custom popup spec config",
    );
    expect_popup_binding(
        &custom_popup_spec,
        "Alt Shift B",
        "btm",
        "custom popup spec config",
    );
    assert_eq!(custom_popup_spec.matches("width_percent 100").count(), 5);
    assert_eq!(custom_popup_spec.matches("height_percent 100").count(), 5);
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
    zellij_plugins.run_yzx(&yzx_bin, "status", "Zellij plugin sidecar status");
    let zellij_plugin_config = zellij_plugins.zellij_file("config.kdl");
    expect_contains_all! {
        &zellij_plugin_config, "Zellij plugin sidecar config";
        "payload \"{\\\"ok\\\": true}\" // Braces in strings must not change block depth.",
        "    } // plugin config close\n    yazelix_pane_orchestrator location=",
        "load_plugins {\n    yzpp\n    my_plugin\n    yazelix_pane_orchestrator\n}",
    }

    let custom_popup_key = RuntimeCase::new(&temp.path, "custom-popup-key");
    custom_popup_key.write_default_config("\n[keybindings]\nconfig = \"Alt Shift C\"\nagent = \"Alt Shift A\"\ngit = \"Alt Shift G\"\nmenu = \"Alt Shift U\"\n");
    let status = custom_popup_key.run_yzx(&yzx_bin, "status", "custom popup key status");
    expect_contains_all! {
        &status, "custom popup key status";
        "config keybinding: Alt Shift C",
        "agent keybinding: Alt Shift A",
        "git keybinding: Alt Shift G",
        "menu keybinding: Alt Shift U",
        "zellij config: runtime (",
    }
    let custom_key_config = custom_popup_key.zellij_file("config.kdl");
    for (key, payload, default) in [
        ("Alt Shift C", "config", "Alt Shift K"),
        ("Alt Shift A", "agent", "Alt Shift L"),
        ("Alt Shift G", "git", "Alt Shift J"),
        ("Alt Shift U", "menu", "Alt Shift M"),
    ] {
        expect_popup_binding(&custom_key_config, key, payload, "custom popup key config");
        assert!(
            !custom_key_config.contains(&format!(r#"bind "{default}" {{"#)),
            "custom popup key kept the default {payload} binding"
        );
    }

    let swapped_popup_key = RuntimeCase::new(&temp.path, "swapped-popup-key");
    swapped_popup_key.write_default_config("\n[keybindings]\nconfig = \"Alt Shift L\"\nagent = \"Alt Shift K\"\ngit = \"Alt Shift M\"\nmenu = \"Alt Shift J\"\n");
    swapped_popup_key.run_yzx(&yzx_bin, "status", "swapped popup key status");
    let swapped_key_config = swapped_popup_key.zellij_file("config.kdl");
    for (key, payload) in [
        ("Alt Shift L", "config"),
        ("Alt Shift K", "agent"),
        ("Alt Shift M", "git"),
        ("Alt Shift J", "menu"),
    ] {
        expect_popup_binding(
            &swapped_key_config,
            key,
            payload,
            "swapped popup key config",
        );
    }

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
    let status = custom_bar.run_yzx(&yzx_bin, "status", "custom bar status");
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
    let format_right = custom_layout
        .lines()
        .find(|line| line.contains("format_right"))
        .expect("custom layout is missing format_right");
    expect_contains_all! {
        format_right, "custom bar layout";
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
        "pane name=\"sidebar\" command=\"/nix/store/",
        "stacked=true",
    }
    assert!(
        !custom_swap.contains("@yazi@"),
        "custom bar swap layout kept the unresolved Yazi placeholder"
    );
    let custom_config = custom_bar.zellij_file("config.kdl");
    expect_contains_all! {
        &custom_config, "custom bar new-tab config";
        format!(r#"layout "{}""#, custom_bar.zellij_path("layout.kdl").display()),
        format!("cwd {home};"),
    }

    let doctor = doctor_case.run_yzx(&yzx_bin, "doctor", "yzx doctor");
    expect_contains_all! {
        &doctor, "yzx doctor";
        "Yazelix Nova doctor",
        format!("ok config home: {}", doctor_case.config_home.display()),
        "ok editor.command: yzx-hx",
        "ok editor: /nix/store/",
        "ok agent.command: auto",
        "ok agent.args: []",
        "ok open.log_level: info",
        "ok welcome.enabled: true",
        "ok welcome.style: random",
        "ok welcome.duration_seconds: 3",
        r#"ok bar.widgets: ["editor","shell","term","codex_usage","cpu","ram"]"#,
        "ok popup.side_margin: 1",
        "ok popup.vertical_margin: 0",
        "ok keybindings.config: Alt Shift K",
        "ok keybindings.agent: Alt Shift L",
        "ok keybindings.git: Alt Shift J",
        "ok keybindings.menu: Alt Shift M",
        "ok tutor helper: /nix/store/",
        "ok screen helper: /nix/store/",
        "ok welcome helper: /nix/store/",
        "ok yazi opener: /nix/store/",
        "ok reveal helper: /nix/store/",
        "ok yazi cli: /nix/store/",
        "ok pane orchestrator plugin: /nix/store/",
        "warn session: not inside zellij",
    }

    for (args, expected, context) in [
        (
            &["env", "extra"][..],
            "yzx env does not accept arguments yet",
            "yzx env argument error",
        ),
        (
            &["doctor", "extra"][..],
            "yzx doctor does not accept arguments yet",
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
            &["wat"][..],
            "yzx: unknown command: wat",
            "unknown yzx command error",
        ),
    ] {
        expect_command_error(&yzx_bin, args, expected, context);
    }
    let identity = fs::read_to_string(yzx.join("share/yazelix/runtime_identity.json"))
        .expect("yzx package is missing runtime_identity.json");
    let generated_identity =
        fs::read_to_string(doctor_case.state_dir.join("runtime_identity.json"))
            .expect("yzx runtime did not materialize runtime_identity.json");
    assert_eq!(generated_identity, identity);
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
    let mut command = case.yzx_command(&yzx_bin, "run");
    command
        .args(["printenv", "PATH"])
        .env("PATH", "/private/tmp");
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
        "command = \"yzx-hx\"",
        "command = \"auto\"",
        "args = []",
        "enabled = true",
        "style = \"random\"",
        "duration_seconds = 3",
        "side_margin = 1",
        "vertical_margin = 0",
        "config = \"Alt Shift K\"",
        "agent = \"Alt Shift L\"",
        "git = \"Alt Shift J\"",
        "menu = \"Alt Shift M\"",
        "widgets = [\"editor\", \"shell\", \"term\", \"codex_usage\", \"cpu\", \"ram\"]",
    }

    let helper = yzx.join("libexec/yazelix/yzx-config");
    assert!(helper.is_file(), "missing yzx-config helper");
    let temp = TempDir::new();
    for (path, expected) in [
        ("open.log_level", "info"),
        ("shell.program", "nu"),
        ("editor.command", "yzx-hx"),
        ("agent.command", "auto"),
        ("agent.args", "[]"),
        ("welcome.enabled", "true"),
        ("welcome.style", "random"),
        ("welcome.duration_seconds", "3"),
        ("popup.side_margin", "1"),
        ("popup.vertical_margin", "0"),
        ("keybindings.config", "Alt Shift K"),
        ("keybindings.agent", "Alt Shift L"),
        ("keybindings.git", "Alt Shift J"),
        ("keybindings.menu", "Alt Shift M"),
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
            "bad-popup-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[popup]\nside_margin = -1\n",
            "popup.side_margin must be zero or greater",
            "invalid popup margin",
        ),
        (
            "bad-welcome-style-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[welcome]\nstyle = \"matrix\"\n",
            "welcome.style must be one of: static, logo, boids, boids_predator, boids_schools, mandelbrot, game_of_life_gliders, game_of_life_oscillators, game_of_life_bloom, random",
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
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[keybindings]\nagent = \"Alt Shift h\"\n",
            "keybindings.agent conflicts with packaged key Alt Shift h",
            "conflicting agent key",
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
        "failed to create",
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
        let context = format!("{label} doctor stdout");
        expect_contains_all! {
            stdout.as_ref(), &context;
            "Yazelix Nova doctor",
            "fail runtime preflight:",
        }
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
            .env("YAZELIX_CONFIG_HOME", config_home)
            .env("YAZELIX_STATE_DIR", "")
            .env("STARSHIP_CONFIG", "ambient-starship.toml")
            .env("PATH", path),
        &yzx_nu.display().to_string(),
    )
}

fn expect_mars_config_override(yzx: &Path) {
    let packaged_config = yzx.join("share/yazelix/mars/config.toml");
    let yzx_bin = yzx.join("bin/yzx");
    assert!(
        packaged_config.is_file(),
        "packaged Mars config is not a file: {}",
        packaged_config.display()
    );

    let launcher = binary_text(&yzx_bin);
    expect_contains_all! {
        &launcher, "runtime Mars config override fragment";
        "YAZELIX_CONFIG_HOME",
        "MARS_BASE_CONFIG_HOME",
        "MARS_CONFIG_HOME",
        "yzx-mars-config",
    }

    let temp = TempDir::new();
    let mars_case = RuntimeCase::new(&temp.path, "mars");
    let status = mars_case.run_yzx(&yzx_bin, "status", "packaged Mars config status");
    expect_contains_all! {
        &status, "packaged Mars config status";
        "mars config: packaged",
        "yzx-mars-config/config.toml",
    }

    let mars_config = mars_case.config_home.join("mars/config.toml");
    fs::create_dir_all(mars_config.parent().unwrap()).unwrap();
    fs::write(&mars_config, "# user Mars config\n").unwrap();

    let status = mars_case.run_yzx(&yzx_bin, "status", "Mars config override status");
    expect_contains_all! {
        &status, "Mars config override status";
        "mars config: user",
        mars_config.display().to_string(),
    }
}

fn expect_cursor_config(yzx: &Path) {
    let template = fs::read_to_string(yzx.join("share/yazelix/cursors.toml")).unwrap();
    let yzx_bin = yzx.join("bin/yzx");
    let temp = TempDir::new();
    let case = RuntimeCase::new(&temp.path, "cursors");
    case.run_yzx(&yzx_bin, "status", "cursor config initialization");
    let cursor_config = case.config_home.join("cursors.toml");
    assert_eq!(fs::read_to_string(&cursor_config).unwrap(), template);

    let custom = format!("{template}\n# preserved user cursor config\n");
    fs::write(&cursor_config, &custom).unwrap();
    case.run_yzx(&yzx_bin, "status", "cursor config preservation");
    assert_eq!(fs::read_to_string(cursor_config).unwrap(), custom);
}

fn expect_zellij_config_sidecar(yzx: &Path) {
    let packaged_config = yzx.join("share/yazelix/config.kdl");
    let helper = yzx.join("libexec/yazelix/yzx-zellij-config");
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
        ("env", "env { YZX_OPEN_LOG \"off\" }\n"),
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

fn expect_yazi_alt_z(yzx: &Path) {
    let keymap = fs::read_to_string(yzx.join("share/yazelix/yazi/keymap.toml")).unwrap();
    expect_contains_all! {
        &keymap, "Yazi Alt-z keymap fragment";
        r#"on = ["<A-z>"]"#,
        r#"run = "plugin zoxide-editor""#,
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
    expect_contains(
        &init,
        r#"require("sidebar-state"):setup()"#,
        "Yazi init sidebar-state fragment",
    );
    let sidebar_state =
        fs::read_to_string(yzx.join("share/yazelix/yazi/plugins/sidebar-state.yazi/main.lua"))
            .unwrap();
    expect_contains_all! {
        &sidebar_state, "Yazi sidebar-state plugin fragment";
        "register_sidebar_yazi_state",
        "YAZELIX_ZELLIJ_SESSION_NAME",
        "ZELLIJ_SESSION_NAME",
        "YZX_ZELLIJ",
        "emit(\"plugin\", { \"git\", \"refresh-sidebar\" })",
    }
    assert!(
        yzx.join("share/yazelix/yazi/plugins/git.yazi").is_dir(),
        "packaged Yazi config is missing git.yazi",
    );

    let plugin =
        fs::read_to_string(yzx.join("share/yazelix/yazi/plugins/zoxide-editor.yazi/main.lua"))
            .unwrap();
    expect_contains_all! {
        &plugin, "Yazi zoxide editor plugin fragment";
        r#"Command(yzx_open):arg(target_dir)"#,
        r#"Command("zoxide")"#,
        r#"emit("cd", { target_dir, raw = true })"#,
        "YZX_OPEN is not set",
    }

    let layout = fs::read_to_string(yzx.join("share/yazelix/layout.kdl")).unwrap();
    let yzx_yazi = layout
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(r#"pane name="sidebar" command=""#)?
                .split('"')
                .next()
                .filter(|path| !path.is_empty())
                .map(PathBuf::from)
        })
        .expect("layout is missing sidebar yzx-yazi command");
    let wrapper = binary_text(&yzx_yazi);
    let materializer = embedded_store_path(&wrapper, "/bin/yzx-yazi-config");
    assert!(materializer.is_file());
    let context = format!("{} Yazi integration fragment", yzx_yazi.display());
    expect_contains_all! {
        &wrapper, &context;
        "YZX_OPEN",
        "YZX_ZELLIJ",
        "YZX_EDITOR",
        "YAZELIX_EDITOR",
        "GIT_EDITOR",
        "editor.command",
        "YAZI_CONFIG_HOME",
        "/bin/yzx-yazi-config",
        "yazelix_starship.toml",
        "YAZELIX_ZELLIJ_SESSION_NAME",
        "ZELLIJ_SESSION_NAME",
        "KITTY_WINDOW_ID",
        "git",
        "zoxide",
        "fzf",
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
        r#"unbind "Alt n" "Alt i" "Alt o" "Ctrl g""#,
        r#"bind "Alt m" { NewPane; }"#,
        r#"bind "Alt h" "Alt Left" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_left_or_tab"; }; }"#,
        r#"bind "Alt l" "Alt Right" { MessagePlugin "yazelix_pane_orchestrator" { name "move_focus_right_or_tab"; }; }"#,
        r#"bind "Alt r" { MessagePlugin "yazelix_pane_orchestrator" { name "smart_reveal"; }; }"#,
        r#"bind "Alt Shift F" { ToggleFocusFullscreen; }"#,
        r#"bind "Alt Shift h" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar"; }; }"#,
        r#"bind "Ctrl y" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_editor_sidebar_focus"; }; }"#,
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
    for expected in ["ToggleFocusFullscreen", "toggle_editor_sidebar_focus"] {
        assert_eq!(config.matches(expected).count(), 1, "duplicate {expected}");
    }
    for tab in 1..=9 {
        let expected = format!(r#"bind "Alt {tab}" {{ GoToTab {tab}; }}"#);
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
                    .ends_with(r#"/layout.kdl"; cwd "__YZX_HOME__"; }; SwitchToMode "Normal"; }"#)
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

fn expect_first_party_plugins(git_bin: &Path, config: &str) {
    expect_contains_all! {
        config, "config.kdl first-party plugin fragment";
        "share/yazelix_zellij_popup/yzpp.wasm",
        "share/yazelix_zellij_pane_orchestrator/yazelix_pane_orchestrator.wasm",
        r#"yazelix_pane_orchestrator location="file:/nix/store/"#,
        "load_plugins",
        "support_kitty_keyboard_protocol true",
        "screen_saver_enabled false",
    }
    expect_popup_defaults(config, "1", "0", "packaged popup config");
    for (id, pane_title, command_suffix, extra) in [
        ("config", "config_popup", "/bin/yzx-config-ui", ""),
        (
            "agent",
            "agent_popup",
            "/bin/yzx-agent",
            "\n                toggle_close_behavior \"hide\"",
        ),
        ("git", "git_popup", "/bin/yzx-git", ""),
        ("menu", "menu_popup", "/bin/yzx-menu", ""),
    ] {
        let command = popup_command(config, command_suffix);
        let expected = format!(
            "{id} {{\n                command \"{}\"\n                pane_title \"{pane_title}\"\n                width_percent 100\n                height_percent 100{extra}\n            }}",
            command.display()
        );
        assert!(
            config.contains(&expected),
            "config.kdl is missing {id} popup block\n{expected}",
        );
    }
    assert_eq!(config.matches("width_percent 100").count(), 4);
    assert_eq!(config.matches("height_percent 100").count(), 4);
    assert_eq!(config.matches("side_margin 1").count(), 1);
    assert_eq!(config.matches("vertical_margin 0").count(), 1);
    for (key, payload) in [
        ("Alt Shift J", "git"),
        ("Alt Shift K", "config"),
        ("Alt Shift L", "agent"),
        ("Alt Shift M", "menu"),
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
    let context = format!("{} managed editor wrapper", config_ui.display());
    expect_contains_all! {
        &config_ui_script, &context;
        "/bin/yzx-editor",
        "GIT_EDITOR",
    }

    assert!(popup_command(config, "/bin/yzx-menu").is_file());
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
    let expected = format!(
        "bind \"{key}\" {{\n            MessagePlugin \"yzpp\" {{\n                name \"toggle\"\n                payload \"{payload}\"\n            }}\n        }}"
    );
    assert!(
        config.contains(&expected),
        "{context} is missing {key} popup binding\n{expected}",
    );
}

fn expect_popup_defaults(config: &str, side_margin: &str, vertical_margin: &str, context: &str) {
    let refresh = popup_command(config, "/bin/yzx-sidebar-refresh");
    let expected = format!(
        "popup_defaults {{\n            side_margin {side_margin}\n            vertical_margin {vertical_margin}\n            on_close {{\n                command \"{}\"\n            }}\n            on_hide {{\n                command \"{}\"\n            }}\n        }}",
        refresh.display(),
        refresh.display(),
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
    assert!(
        output.stdout.is_empty() && output.stderr.is_empty(),
        "agent popup without providers should leave the pane empty\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        !empty_state.join("agent/provider").exists(),
        "missing-provider bootstrap should not write a provider default"
    );

    for (name, available, expected_output) in [
        ("codex-first", &["codex", "opencode"][..], "codex resume\n"),
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
}

fn write_fake_agent(bin: &Path, name: &str) {
    let path = bin.join(name);
    write_executable(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$#\" -eq 0 ]; then\n  printf '%s\\n' \"{name}\" >\"$YAZELIX_AGENT_TEST_OUT\"\nelse\n  printf '%s %s\\n' \"{name}\" \"$*\" >\"$YAZELIX_AGENT_TEST_OUT\"\nfi\n"
        ),
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
