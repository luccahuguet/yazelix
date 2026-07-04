use std::{
    env, fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

const SPONSOR_URL: &str = "https://github.com/sponsors/luccahuguet";
const DEFAULT_CONFIG: &str = "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n";

macro_rules! expect_contains_all {
    ($haystack:expr, $context:expr; $($needle:expr),+ $(,)?) => {
        $(expect_contains($haystack, &$needle, $context);)+
    };
}

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
    let host_bin = temp.path.join("host-bin");
    fs::create_dir(&host_bin).unwrap();
    let fake_mise = host_bin.join("mise");
    write_executable(
        &fake_mise,
        "#!/bin/sh\n[ \"$1\" = activate ] && [ \"$2\" = nu ] || exit 64\nprintf '%s\\n' '$env.YZN_MISE_TEST = \"mise-ok\"'\n",
    );
    let mise_runtime = temp.path.join("mise-run");
    let mise_stdout = run_nu_with_path(
        &yzn_shell,
        &user_config,
        &mise_runtime,
        "print $env.YZN_MISE_TEST",
        &host_bin,
    );
    assert_eq!(mise_stdout, "mise-ok");
    expect_line(
        &mise_runtime.join("yazelix-next/nu/config.nu"),
        "$env.YZN_MISE_TEST = \"mise-ok\"",
    );
    let generated_mise_config =
        fs::read_to_string(mise_runtime.join("yazelix-next/nu/config.nu")).unwrap();
    let user_config_source = format!("source \"{}\"", user_nu.join("config.nu").display());
    expect_order(
        &generated_mise_config,
        &[
            "source \"/nix/store/",
            "$env.YZN_MISE_TEST = \"mise-ok\"",
            &user_config_source,
        ],
        "managed Nu mise layering",
    );
    fs::write(out, "ok\n").unwrap();
}

fn expect_front_door(yzn: &Path) {
    let yzn_bin = yzn.join("bin/yzn");
    let help = run_help(&yzn_bin, &["help"]);
    for arg in ["-h", "--help"] {
        assert_eq!(run_help(&yzn_bin, &[arg]), help);
    }
    expect_contains_all! {
        &help, "yzn help";
        "Usage:",
        "yzn config",
        "yzn doctor",
        "yzn env",
        "yzn enter [zellij-args...]",
        "yzn launch [zellij-args...]",
        "yzn menu",
        "yzn tutor [lesson]",
        "yzn reveal <target>",
        "yzn screen [style]",
        "yzn sponsor",
        "yzn status",
    }
    let menu = run_help(&yzn_bin, &["menu"]);
    expect_contains(&menu, "Yazelix command palette", "yzn menu");
    let menu_ids = menu
        .lines()
        .filter_map(|line| {
            let (_, command) = line.trim_start().split_once('.')?;
            command.split_whitespace().next()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        menu_ids,
        ["config", "doctor", "status", "screen", "sponsor", "launch", "help", "tutor"],
        "yzn menu command allowlist changed\n{menu}"
    );
    expect_menu_descriptions_match_help(&help, &menu);
    for forbidden in [
        "yzn env",
        "yzn enter",
        "yzn reveal",
        "Alt Shift",
        "Ctrl Alt",
        "Git popup",
        "Agent popup",
    ] {
        assert!(
            !menu.contains(forbidden),
            "yzn menu exposes non-allowlisted reference `{forbidden}`\n{menu}"
        );
    }
    let reveal_help = run_help(&yzn_bin, &["reveal", "--help"]);
    expect_contains(&reveal_help, "yzn reveal <target>", "yzn reveal help");
    let screen_help = run_help(&yzn_bin, &["screen", "--help"]);
    expect_contains_all! {
        &screen_help, "yzn screen help";
        "yzn screen [STYLE]",
        "static",
        "logo",
        "mandelbrot",
        "random",
    }
    let tutor_help = run_help(&yzn_bin, &["tutor", "--help"]);
    expect_contains_all! {
        &tutor_help, "yzn tutor help";
        "yzn tutor",
        "yzn tutor begin",
        "yzn tutor list",
        "yzn tutor workspace",
        "yzn tutor discovery",
        "yzn tutor troubleshooting",
        "yzn tutor tool_tutors",
        "yzn tutor hx",
        "yzn tutor helix",
        "yzn tutor nu",
        "yzn tutor nushell",
    }
    let tutor_root = run_help(&yzn_bin, &["tutor"]);
    expect_contains_all! {
        &tutor_root, "yzn tutor";
        "Yazelix tutor",
        "yzn tutor begin",
        "yzn tutor list",
    }
    assert!(
        !tutor_root.contains("yzx "),
        "yzn tutor root leaked old command name\n{}",
        excerpt(&tutor_root)
    );
    let tutor_list = run_help(&yzn_bin, &["tutor", "list"]);
    expect_contains_all! {
        &tutor_list, "yzn tutor list";
        "yzn tutor workspace",
        "yzn tutor discovery",
        "yzn tutor troubleshooting",
        "yzn tutor tool_tutors",
    }
    for (lesson, expected) in [
        ("begin", "Workspace roots and managed panes"),
        ("workspace", "current tab workspace root matters most"),
        ("discovery", "Alt Shift M"),
        ("troubleshooting", "yzn doctor"),
        ("tool_tutors", "print the packaged Helix tutor command"),
    ] {
        let output = run_help(&yzn_bin, &["tutor", lesson]);
        expect_contains(&output, expected, &format!("yzn tutor {lesson}"));
        assert!(
            !output.contains("yzx ")
                && !output.contains("env --no-shell")
                && !output.contains("launch --path"),
            "yzn tutor {lesson} leaked stale main Yazelix text\n{}",
            excerpt(&output)
        );
    }
    let helix_tutor = run_help(&yzn_bin, &["tutor", "hx"]);
    expect_contains_all! {
        &helix_tutor, "yzn tutor hx";
        "/bin/yzn-hx --tutor",
        "yzn-hx --tutor",
    }
    let nushell_tutor = run_help(&yzn_bin, &["tutor", "nu"]);
    expect_contains_all! {
        &nushell_tutor, "yzn tutor nu";
        "/bin/nu -c 'tutor begin'",
        "tutor begin",
    }

    let yzn_launcher = binary_text(&yzn_bin);
    let menu_helper = embedded_store_path(&yzn_launcher, "/bin/yzn-menu");
    expect_menu_dispatch(&menu_helper);
    expect_contains_all! {
        &yzn_launcher, "bin/yzn runtime fragment";
        "Yazelix could not start.",
        "YAZELIX_STATUS_BAR_CACHE_PATH",
        "ZELLIJ_PLUGIN_PERMISSIONS_CACHE",
        "YAZELIX_SESSION_TERMINAL",
        "YAZELIX_APPEARANCE_MODE",
        "YZN_WELCOME_ENABLED",
        "YZN_WELCOME_STYLE",
        "YZN_WELCOME_DURATION_SECONDS",
        "YZN_MENU_YZN",
        "YZN_YA",
        "YZN_ZELLIJ",
        "welcome.enabled",
        "welcome.style",
        "welcome.duration_seconds",
        "appearance.mode",
        "YAZELIX_NEXT_EDITOR",
        "YZN_EDITOR",
        "GIT_EDITOR",
        "editor.command",
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
        "yzn-bar-render",
        "yzn-env-supervisor",
        "yzn-tutor",
        "yzn-welcome",
        "yzn-shell",
        "yzn-reveal",
        "/bin/yzs",
        "yazelix-helix",
        "yazelix_pane_orchestrator.wasm",
        "/bin/ya",
        "/bin/zellij",
        "/bin/mars",
        "tokenusage",
        "--new-session-with-layout",
    }
    let env_supervisor = embedded_store_path(&yzn_launcher, "/bin/yzn-env-supervisor");
    let env_supervisor_script = fs::read_to_string(&env_supervisor).unwrap();
    expect_contains_all! {
        &env_supervisor_script, "yzn env supervisor";
        "#!/nix/store/",
        "trap cleanup HUP INT TERM EXIT",
        "\"$1\" < /dev/tty &",
        "wait \"$child\"",
    }

    let temp = TempDir::new();
    let status_case = RuntimeCase::new(&temp.path, "status");
    let doctor_case = RuntimeCase::new(&temp.path, "doctor");
    let status = status_case.run_yzn(&yzn_bin, "status", "yzn status");
    expect_contains_all! {
        &status, "yzn status";
        "Yazelix status",
        format!("config home: {}", status_case.config_home.display()),
        format!("state dir: {}", status_case.state_dir.display()),
        "shell: nu",
        "editor command: yzn-hx",
        "editor: /nix/store/",
        "appearance mode: dark",
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
    let data_home = temp.path.join("data-home");
    let data_status = successful_stdout(
        Command::new(&yzn_bin)
            .arg("status")
            .env("YAZELIX_NEXT_CONFIG_HOME", &status_case.config_home)
            .env("XDG_DATA_HOME", &data_home)
            .env_remove("YAZELIX_STATE_DIR"),
        "yzn status XDG data state",
    );
    expect_contains(
        &data_status,
        &format!("state dir: {}", data_home.join("yazelix-next").display()),
        "yzn status XDG data state",
    );

    let permissions = status_case.zellij_file("permissions.kdl");
    let runtime_config = status_case.zellij_file("config.kdl");
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
    expect_contains_all! {
        &permissions, "runtime plugin permissions";
        "yazelix_pane_orchestrator.wasm",
        "MessageAndLaunchOtherPlugins",
        "ReadSessionEnvironmentVariables",
    }

    let custom_popup = RuntimeCase::new(&temp.path, "custom-popup");
    custom_popup.write_default_config("\n[popup]\nside_margin = 2\nvertical_margin = 1\n");
    let status = custom_popup.run_yzn(&yzn_bin, "status", "custom popup status");
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

    let custom_popup_spec_case = RuntimeCase::new(&temp.path, "custom-popup-spec");
    custom_popup_spec_case.write_default_config("\n[popup]\nside_margin = 2\nvertical_margin = 1\n\n[popups.btm]\ncommand = \"btm\"\nargs = [\"--basic\"]\ntitle = \"btm_popup\"\nkeybinding = \"Alt Shift B\"\nkeep_alive = true\n");
    custom_popup_spec_case.run_yzn(&yzn_bin, "status", "custom popup spec status");
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
    zellij_plugins.run_yzn(&yzn_bin, "status", "Zellij plugin sidecar status");
    let zellij_plugin_config = zellij_plugins.zellij_file("config.kdl");
    expect_contains_all! {
        &zellij_plugin_config, "Zellij plugin sidecar config";
        "payload \"{\\\"ok\\\": true}\" // Braces in strings must not change block depth.",
        "    } // plugin config close\n    yazelix_pane_orchestrator location=",
        "load_plugins {\n    yzpp\n    my_plugin\n    yazelix_pane_orchestrator\n}",
    }

    let custom_popup_key = RuntimeCase::new(&temp.path, "custom-popup-key");
    custom_popup_key.write_default_config("\n[keybindings]\nconfig = \"Alt Shift C\"\nagent = \"Alt Shift A\"\ngit = \"Alt Shift G\"\nmenu = \"Alt Shift U\"\n");
    let status = custom_popup_key.run_yzn(&yzn_bin, "status", "custom popup key status");
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
    swapped_popup_key.run_yzn(&yzn_bin, "status", "swapped popup key status");
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
    let status = custom_editor.run_yzn(&yzn_bin, "status", "custom editor status");
    expect_contains_all! {
        &status, "custom editor status";
        "editor command: nvim",
        "editor: nvim",
    }

    let light_mode = RuntimeCase::new(&temp.path, "light-mode");
    light_mode.write_default_config("\n[appearance]\nmode = \"light\"\n");
    let status = light_mode.run_yzn(&yzn_bin, "status", "light mode status");
    expect_contains_all! {
        &status, "light mode status";
        "appearance mode: light",
        "mars config: packaged-light (",
        "yzn-mars-light-config",
        "layout: runtime (",
    }
    let light_layout = light_mode.zellij_file("layout.kdl");
    expect_contains_all! {
        &light_layout, "light mode layout";
        "#[fg=#2f7d32,bold]",
        "#[bg=#ccd0da,fg=#303446,bold]",
    }

    let custom_bar = RuntimeCase::new(&temp.path, "custom-bar");
    custom_bar.write_default_config("\n[bar]\nwidgets = [\"editor\", \"claude_usage\", \"cpu\"]\n");
    let status = custom_bar.run_yzn(&yzn_bin, "status", "custom bar status");
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

    let doctor = doctor_case.run_yzn(&yzn_bin, "doctor", "yzn doctor");
    expect_contains_all! {
        &doctor, "yzn doctor";
        "Yazelix doctor",
        format!("ok config home: {}", doctor_case.config_home.display()),
        "ok editor.command: yzn-hx",
        "ok editor: /nix/store/",
        "ok appearance.mode: dark",
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
    assert!(
        !doctor.contains("warn helix config:"),
        "default doctor should not warn about packaged Helix config\n{}",
        excerpt(&doctor)
    );

    let helix_override = RuntimeCase::new(&temp.path, "helix-override");
    helix_override.write_default_config("");
    let helix_override_config = helix_override.config_home.join("helix/config.toml");
    fs::create_dir_all(helix_override_config.parent().unwrap()).unwrap();
    fs::write(&helix_override_config, "theme = \"ayu_evolve\"\n").unwrap();
    let doctor = helix_override.run_yzn(&yzn_bin, "doctor", "Helix override doctor");
    expect_contains_all! {
        &doctor, "Helix override doctor";
        r#"warn helix config: helix config override exists without the ':sh yzn reveal "%{buffer_name}"' configuration"#,
        helix_override_config.display().to_string(),
    }

    expect_sponsor_fallback(
        Command::new(&yzn_bin).arg("sponsor").env("PATH", ""),
        "without opener",
    );

    let fake_path = temp.path.join("fake-path");
    fs::create_dir(&fake_path).unwrap();
    let fake_xdg_open = fake_path.join("xdg-open");
    write_executable(
        &fake_xdg_open,
        "#!/bin/sh\necho noisy opener >&2\nexit 42\n",
    );
    expect_sponsor_fallback(
        Command::new(&yzn_bin)
            .arg("sponsor")
            .env("PATH", &fake_path),
        "with failing opener",
    );

    for (args, expected, context) in [
        (
            &["env", "extra"][..],
            "yzn env does not accept arguments yet",
            "yzn env argument error",
        ),
        (
            &["doctor", "extra"][..],
            "yzn doctor does not accept arguments yet",
            "yzn doctor argument error",
        ),
        (
            &["sponsor", "extra"][..],
            "yzn sponsor does not accept arguments yet",
            "yzn sponsor argument error",
        ),
        (
            &["menu", "extra"][..],
            "yzn menu does not accept arguments yet",
            "yzn menu argument error",
        ),
        (
            &["tutor", "continue"][..],
            "Unknown yzn tutor target: continue",
            "yzn tutor unknown lesson error",
        ),
        (
            &["tutor", "workspace", "extra"][..],
            "Unexpected arguments for yzn tutor",
            "yzn tutor extra argument error",
        ),
        (
            &["wat"][..],
            "yzn: unknown command: wat",
            "unknown yzn command error",
        ),
    ] {
        expect_command_error(&yzn_bin, args, expected, context);
    }
    assert!(
        yzn.join("share/yazelix-next/runtime_identity.json")
            .is_file(),
        "yzn package is missing runtime_identity.json"
    );
    assert!(
        yzn.join("libexec/yazelix-next/yzn-tutor").is_file(),
        "yzn package is missing the tutor helper"
    );
}

fn expect_menu_dispatch(menu: &Path) {
    expect_contains(&binary_text(menu), "/bin/fzf", "yzn-menu packaged fzf path");

    let temp = TempDir::new();
    let fake_yzn = temp.path.join("fake-yzn");
    let output_file = temp.path.join("selected-command");
    write_executable(
        &fake_yzn,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >\"$YZN_MENU_TEST_OUT\"\n",
    );

    let mut child = Command::new(menu)
        .env("YZN_MENU_YZN", &fake_yzn)
        .env("YZN_MENU_TEST_OUT", &output_file)
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

fn write_executable(path: &Path, contents: impl AsRef<[u8]>) {
    fs::write(path, contents).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

struct RuntimeCase {
    config_home: PathBuf,
    state_dir: PathBuf,
}

impl RuntimeCase {
    fn new(root: &Path, name: &str) -> Self {
        Self {
            config_home: root.join(format!("{name}-config")),
            state_dir: root.join(format!("{name}-state")),
        }
    }

    fn write_config(&self, contents: impl AsRef<[u8]>) -> PathBuf {
        write_config_home(&self.config_home, contents)
    }

    fn write_default_config(&self, extra: &str) -> PathBuf {
        self.write_config(default_config(extra))
    }

    fn run_yzn(&self, yzn_bin: &Path, command: &str, context: &str) -> String {
        run_yzn_with_config(
            yzn_bin,
            command,
            &self.config_home,
            &self.state_dir,
            context,
        )
    }

    fn zellij_file(&self, file: &str) -> String {
        fs::read_to_string(self.state_dir.join("zellij").join(file)).unwrap()
    }

    fn zellij_path(&self, file: &str) -> PathBuf {
        self.state_dir.join("zellij").join(file)
    }
}

fn default_config(extra: &str) -> String {
    format!("{DEFAULT_CONFIG}{extra}")
}

fn expect_config_ui(yzn: &Path) {
    let packaged_config = yzn.join("share/yazelix-next/config.toml");
    assert!(
        packaged_config.is_file(),
        "yzn package is missing config.toml"
    );
    let packaged_config = fs::read_to_string(&packaged_config).unwrap();
    expect_contains_all! {
        &packaged_config, "packaged config.toml";
        "log_level = \"info\"",
        "program = \"nu\"",
        "command = \"yzn-hx\"",
        "mode = \"dark\"",
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

    let helper = yzn.join("libexec/yazelix-next/yzn-config");
    assert!(helper.is_file(), "missing yzn-config helper");
    let temp = TempDir::new();
    for (path, expected) in [
        ("open.log_level", "info"),
        ("shell.program", "nu"),
        ("editor.command", "yzn-hx"),
        ("appearance.mode", "dark"),
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
    expect_contains_all! {
        &config_text, "created config.toml";
        "[open]",
        "log_level = \"info\"",
        "[shell]",
        "program = \"nu\"",
        "[editor]",
        "command = \"yzn-hx\"",
        "[appearance]",
        "mode = \"dark\"",
        "[welcome]",
        "enabled = true",
        "style = \"random\"",
        "duration_seconds = 3",
        "[popup]",
        "side_margin = 1",
        "vertical_margin = 0",
        "[keybindings]",
        "config = \"Alt Shift K\"",
        "agent = \"Alt Shift L\"",
        "git = \"Alt Shift J\"",
        "menu = \"Alt Shift M\"",
        "[bar]",
        "widgets = [\"editor\", \"shell\", \"term\", \"codex_usage\", \"cpu\", \"ram\"]",
        "contract_id = \"yazelix-next.config\"",
    }
}

fn expect_startup_diagnostics(yzn: &Path) {
    let yzn_bin = yzn.join("bin/yzn");
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
            "bad-appearance-config",
            "[open]\nlog_level = \"info\"\n\n[shell]\nprogram = \"nu\"\n\n[appearance]\nmode = \"auto\"\n",
            "appearance.mode must be one of: dark, light",
            "invalid appearance mode",
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
                &yzn_bin,
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
                &yzn_bin,
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
        &yzn_bin,
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
            "yzn menu command `{id}` description drifted from yzn help"
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
    yzn_bin: &Path,
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    expect_contains_all! {
        stderr.as_ref(), &format!("{label} {command} diagnostic");
        "Yazelix could not start.",
        "Reason:",
        reason,
        "Check:",
        check.to_str().unwrap(),
    }
    if command == "doctor" {
        let context = format!("{label} doctor stdout");
        expect_contains_all! {
            stdout.as_ref(), &context;
            "Yazelix doctor",
            "fail runtime preflight:",
        }
    }
}

fn run_help(bin: &Path, args: &[&str]) -> String {
    successful_stdout(Command::new(bin).args(args), "yzn help")
}

fn run_nu(yzn_nu: &Path, config_home: &Path, runtime: &Path, commands: &str) -> String {
    run_nu_with_path(yzn_nu, config_home, runtime, commands, Path::new(""))
}

fn run_nu_with_path(
    yzn_nu: &Path,
    config_home: &Path,
    runtime: &Path,
    commands: &str,
    path: &Path,
) -> String {
    fs::create_dir_all(runtime).unwrap();
    successful_stdout_trimmed(
        Command::new(yzn_nu)
            .arg("--commands")
            .arg(commands)
            .env("XDG_DATA_HOME", runtime)
            .env("YAZELIX_NEXT_CONFIG_HOME", config_home)
            .env("YAZELIX_STATE_DIR", "")
            .env("STARSHIP_CONFIG", "ambient-starship.toml")
            .env("PATH", path),
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
    expect_contains_all! {
        &launcher, "runtime Mars config override fragment";
        "YAZELIX_NEXT_CONFIG_HOME",
        "MARS_CONFIG_HOME",
        "yzn-mars-config",
    }

    let temp = TempDir::new();
    let mars_case = RuntimeCase::new(&temp.path, "mars");
    let mars_config = mars_case.config_home.join("mars/config.toml");
    fs::create_dir_all(mars_config.parent().unwrap()).unwrap();
    fs::write(&mars_config, "# user Mars config\n").unwrap();

    let status = mars_case.run_yzn(&yzn_bin, "status", "Mars config override status");
    expect_contains_all! {
        &status, "Mars config override status";
        "mars config: user",
        mars_config.display().to_string(),
    }
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
    expect_contains_all! {
        &keymap, "Yazi Alt-z keymap fragment";
        r#"on = ["<A-z>"]"#,
        r#"run = "plugin zoxide-editor""#,
    }

    let yazi_toml = fs::read_to_string(yzn.join("share/yazelix-next/yazi/yazi.toml")).unwrap();
    expect_contains_all! {
        &yazi_toml, "Yazi config fragment";
        "YZN_ZELLIJ=",
        "url = \"*\"\nrun = \"git\"\ngroup = \"git\"",
        "url = \"*/\"\nrun = \"git\"\ngroup = \"git\"",
    }
    assert!(
        !yazi_toml.contains("YZN_EDITOR="),
        "packaged Yazi opener should inherit YZN_EDITOR from yzn-yazi"
    );

    let init = fs::read_to_string(yzn.join("share/yazelix-next/yazi/init.lua")).unwrap();
    expect_contains(
        &init,
        r#"require("sidebar-state"):setup()"#,
        "Yazi init sidebar-state fragment",
    );
    let sidebar_state =
        fs::read_to_string(yzn.join("share/yazelix-next/yazi/plugins/sidebar-state.yazi/main.lua"))
            .unwrap();
    expect_contains_all! {
        &sidebar_state, "Yazi sidebar-state plugin fragment";
        "register_sidebar_yazi_state",
        "YAZELIX_ZELLIJ_SESSION_NAME",
        "ZELLIJ_SESSION_NAME",
        "YZN_ZELLIJ",
        "emit(\"plugin\", { \"git\", \"refresh-sidebar\" })",
    }
    assert!(
        yzn.join("share/yazelix-next/yazi/plugins/git.yazi")
            .is_dir(),
        "packaged Yazi config is missing git.yazi",
    );

    let plugin =
        fs::read_to_string(yzn.join("share/yazelix-next/yazi/plugins/zoxide-editor.yazi/main.lua"))
            .unwrap();
    expect_contains_all! {
        &plugin, "Yazi zoxide editor plugin fragment";
        r#"Command(yzn_open):arg(target_dir)"#,
        r#"Command("zoxide")"#,
        r#"emit("cd", { target_dir, raw = true })"#,
        "YZN_OPEN is not set",
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
    expect_contains_all! {
        &wrapper, &context;
        "YZN_OPEN",
        "YZN_ZELLIJ",
        "YZN_EDITOR",
        "YAZELIX_NEXT_EDITOR",
        "GIT_EDITOR",
        "editor.command",
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
        r#"bind "Alt Shift h" { MessagePlugin "yazelix_pane_orchestrator" { name "toggle_sidebar"; }; }"#,
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
        ("config", "config_popup", "/bin/yzn-config-ui", ""),
        (
            "agent",
            "agent_popup",
            "/bin/yzn-agent",
            "\n                toggle_close_behavior \"hide\"",
        ),
        ("git", "git_popup", "/bin/yzn-git", ""),
        ("menu", "menu_popup", "/bin/yzn-menu", ""),
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

    let agent = popup_command(config, "/bin/yzn-agent");
    expect_agent_bootstrap(&agent);

    let git = popup_command(config, "/bin/yzn-git");
    let git_script = fs::read_to_string(&git).unwrap();
    let context = format!("{} managed Git popup wrapper", git.display());
    expect_contains_all! {
        &git_script, &context;
        "editor.command",
        "/bin/yzn-hx",
        "/bin/yzn-config",
        "/bin/lazygit",
        "YAZELIX_NEXT_EDITOR",
        "YZN_EDITOR",
        "GIT_EDITOR",
        "EDITOR=$YAZELIX_NEXT_EDITOR",
        "VISUAL=$YAZELIX_NEXT_EDITOR",
    }

    let config_ui = popup_command(config, "/bin/yzn-config-ui");
    let config_ui_script = fs::read_to_string(&config_ui).unwrap();
    let context = format!("{} managed editor wrapper", config_ui.display());
    expect_contains_all! {
        &config_ui_script, &context;
        "editor.command",
        "YAZELIX_NEXT_EDITOR",
        "/bin/yzn-hx",
        "/bin/yzn-config",
        "YZN_EDITOR",
        "GIT_EDITOR",
        "EDITOR=$YAZELIX_NEXT_EDITOR",
        "VISUAL=$YAZELIX_NEXT_EDITOR",
    }
    let helix = embedded_store_path(&config_ui_script, "/bin/yzn-hx");
    let helix_script = fs::read_to_string(&helix).unwrap();
    let context = format!("{} managed Helix wrapper", helix.display());
    expect_contains(&helix_script, "YAZELIX_HELIX_BRIDGE=1", &context);
    let helix_config =
        fs::read_to_string(embedded_store_path(&helix_script, "-config.toml").join("config.toml"))
            .unwrap();
    expect_contains(
        &helix_config,
        r#"A-r = ':sh yzn reveal "%{buffer_name}"'"#,
        "managed Helix reveal binding",
    );
    expect_contains(
        &helix_config,
        "C-r = [\n  \":config-reload\",\n  \":reload\",\n]",
        "managed Helix reload binding",
    );
    let helix_steel = embedded_store_path(&helix_script, "-yzn-helix-steel-config");
    let helix_module = fs::read_to_string(helix_steel.join("helix.scm")).unwrap();
    expect_contains_all! {
        &helix_module, "packaged Helix Steel module";
        "(provide yzn-new-shell)",
        "(require (only-in \"helix/static.scm\" cx->current-file get-helix-cwd))",
        "(require (only-in \"helix/commands.scm\" run-shell-command))",
        "(define (yzn-new-shell-command target)",
        "/bin/yzn-open-terminal",
        "(define (yzn-new-shell)",
    }
    assert!(
        !helix_module.contains("recentf"),
        "packaged Helix Steel module still references recentf\n{}",
        excerpt(&helix_module)
    );
    let open_terminal = embedded_store_path(&helix_module, "/bin/yzn-open-terminal");
    let open_terminal_script = fs::read_to_string(&open_terminal).unwrap();
    expect_contains_all! {
        &open_terminal_script, "packaged Helix new-shell helper";
        "zellij action new-pane --cwd",
        "dirname -- \"$target\"",
    }
    expect_helix_wrapper_config_selection(&helix_script);

    assert!(popup_command(config, "/bin/yzn-menu").is_file());
}

fn expect_helix_wrapper_config_selection(helix_script: &str) {
    const FAKE_HX: &str = "#!/bin/sh\n\
printf 'HELIX_STEEL_CONFIG=%s\\n' \"${HELIX_STEEL_CONFIG-}\" > \"$YZN_FAKE_HX_OUT\"\n\
printf 'YAZELIX_HELIX_MANAGED_CONFIG_PATH=%s\\n' \"$YAZELIX_HELIX_MANAGED_CONFIG_PATH\" >> \"$YZN_FAKE_HX_OUT\"\n\
for arg do printf 'arg=%s\\n' \"$arg\" >> \"$YZN_FAKE_HX_OUT\"; done\n";

    let temp = TempDir::new();
    let packaged_config = embedded_store_path(helix_script, "-config.toml").join("config.toml");
    let packaged_steel = embedded_store_path(helix_script, "-yzn-helix-steel-config");
    let fake_hx = temp.path.join("hx");
    write_executable(&fake_hx, FAKE_HX);
    let real_hx = embedded_store_path(helix_script, "/bin/hx");
    let test_wrapper = temp.path.join("yzn-hx");
    write_executable(
        &test_wrapper,
        helix_script.replace(real_hx.to_str().unwrap(), fake_hx.to_str().unwrap()),
    );

    for (name, files, uses_user_config_file, uses_user_steel) in [
        ("packaged", &[] as &[(&str, &str)], false, false),
        (
            "languages",
            &[("languages.toml", "# managed languages\n")] as &[(&str, &str)],
            false,
            false,
        ),
        (
            "toml",
            &[("config.toml", "# managed config\n")] as &[(&str, &str)],
            true,
            false,
        ),
        (
            "steel",
            &[("helix.scm", ";; module\n"), ("init.scm", ";; init\n")] as &[(&str, &str)],
            false,
            true,
        ),
    ] {
        expect_helix_wrapper_case(
            &test_wrapper,
            &temp.path,
            &packaged_config,
            &packaged_steel,
            name,
            files,
            uses_user_config_file,
            uses_user_steel,
        );
    }
}

fn expect_helix_wrapper_case(
    wrapper: &Path,
    root: &Path,
    packaged_config: &Path,
    packaged_steel: &Path,
    name: &str,
    files: &[(&str, &str)],
    uses_user_config_file: bool,
    uses_user_steel: bool,
) {
    let home = root.join(format!("{name}-config"));
    let helix = home.join("helix");
    if !files.is_empty() {
        fs::create_dir_all(&helix).unwrap();
        for (file, contents) in files {
            fs::write(helix.join(file), contents).unwrap();
        }
    }
    let state = root.join(format!("{name}-state"));
    let output = run_helix_wrapper(wrapper, &home, &state, &root.join(format!("{name}-output")));
    let expected_config_dir = if files.is_empty() {
        packaged_config.parent().unwrap().to_path_buf()
    } else {
        helix.clone()
    };
    let expected_config_file = if uses_user_config_file {
        helix.join("config.toml")
    } else {
        packaged_config.to_path_buf()
    };
    let expected_steel_dir = if files.is_empty() {
        Some(packaged_steel.to_path_buf())
    } else if uses_user_steel {
        Some(helix)
    } else {
        Some(state.join("helix-steel"))
    };
    expect_helix_wrapper_output(
        &output,
        &expected_config_dir,
        &expected_config_file,
        expected_steel_dir.as_deref(),
        &format!("{name} Helix config selection"),
    );
    if let Some(steel_dir) = expected_steel_dir.filter(|_| !uses_user_steel) {
        assert!(
            steel_dir.is_dir(),
            "{name} Helix config should create the internal Steel fallback dir"
        );
    }
}

fn run_helix_wrapper(
    wrapper: &Path,
    config_home: &Path,
    state_dir: &Path,
    output_path: &Path,
) -> String {
    let output = Command::new(wrapper)
        .env("YAZELIX_NEXT_CONFIG_HOME", config_home)
        .env("YAZELIX_STATE_DIR", state_dir)
        .env("YZN_FAKE_HX_OUT", output_path)
        .env_remove("HELIX_STEEL_CONFIG")
        .env_remove("YAZELIX_HELIX_MANAGED_CONFIG_PATH")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Helix wrapper failed: stdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(output_path).unwrap()
}

fn expect_helix_wrapper_output(
    output: &str,
    config_dir: &Path,
    config_file: &Path,
    steel_dir: Option<&Path>,
    context: &str,
) {
    let steel_line = format!(
        "HELIX_STEEL_CONFIG={}\n",
        steel_dir
            .map(|path| path.display().to_string())
            .unwrap_or_default()
    );
    let managed_line = format!(
        "YAZELIX_HELIX_MANAGED_CONFIG_PATH={}",
        config_file.display()
    );
    let config_dir_arg = format!("arg={}", config_dir.display());
    let config_file_arg = format!("arg={}", config_file.display());
    expect_contains_all! {
        output, context;
        steel_line,
        managed_line,
    }
    expect_order(
        output,
        &[
            "arg=--config-dir",
            config_dir_arg.as_str(),
            "arg=-c",
            config_file_arg.as_str(),
        ],
        context,
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
    let refresh = popup_command(config, "/bin/yzn-sidebar-refresh");
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

fn expect_order(haystack: &str, needles: &[&str], context: &str) {
    let mut offset = 0;
    for needle in needles {
        let Some(index) = haystack[offset..].find(needle) else {
            panic!(
                "{context} is missing {needle:?} after byte {offset}\n{}",
                excerpt(haystack)
            );
        };
        offset += index + needle.len();
    }
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

fn embedded_store_path(text: &str, suffix: &str) -> PathBuf {
    let end = text
        .find(suffix)
        .unwrap_or_else(|| panic!("binary text is missing path suffix {suffix}"))
        + suffix.len();
    let start = text[..end]
        .rfind("/nix/store/")
        .unwrap_or_else(|| panic!("binary text is missing /nix/store path for {suffix}"));
    PathBuf::from(&text[start..end])
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
