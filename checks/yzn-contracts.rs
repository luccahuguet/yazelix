use std::{env, fs, path::Path, path::PathBuf, process::Command};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzn, out] = args.as_slice() else {
        panic!("usage: yzn-contracts-check <yzn-package> <out>");
    };

    let yzn = Path::new(yzn);
    let config = fs::read_to_string(yzn.join("share/yazelix-next/config.kdl")).unwrap();
    let yzn_nu = default_shell(&config);
    assert!(
        yzn_nu.is_file(),
        "default_shell is not a file: {}",
        yzn_nu.display()
    );
    expect_keybinds(&config);
    expect_lazygit_popup(&config);
    expect_front_door(yzn);
    expect_mars_config_override(yzn);
    expect_zellij_config_sidecar(yzn);

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
        &yzn_nu,
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
        &yzn_nu,
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
        "yzn enter [zellij-args...]",
        "yzn launch [zellij-args...]",
    ] {
        assert!(
            help.contains(expected),
            "yzn help is missing {expected:?}\n{help}",
        );
    }

    let yzn_launcher = fs::read_to_string(&yzn_bin).unwrap();
    for expected in [
        "--new-session-with-layout",
        "/bin/zellij --config",
        "/bin/mars -e",
    ] {
        assert!(
            yzn_launcher.contains(expected),
            "bin/yzn does not contain launch fragment {expected}",
        );
    }
}

fn run_help(bin: &Path, args: &[&str]) -> String {
    let output = Command::new(bin).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{} {:?} failed with status {}\n{}",
        bin.display(),
        args,
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_nu(yzn_nu: &Path, config_home: &Path, runtime: &Path, commands: &str) -> String {
    fs::create_dir_all(runtime).unwrap();
    let output = Command::new(yzn_nu)
        .arg("--commands")
        .arg(commands)
        .env("XDG_RUNTIME_DIR", runtime)
        .env("YAZELIX_NEXT_CONFIG_HOME", config_home)
        .env("STARSHIP_CONFIG", "ambient-starship.toml")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{} failed with status {}\n{}",
        yzn_nu.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_owned()
}

fn expect_mars_config_override(yzn: &Path) {
    let packaged_config = yzn.join("share/yazelix-next/mars/config.toml");
    assert!(
        packaged_config.is_file(),
        "packaged Mars config is not a file: {}",
        packaged_config.display()
    );

    let launcher = fs::read_to_string(yzn.join("bin/yzn")).unwrap();
    for expected in [
        "YAZELIX_NEXT_CONFIG_HOME",
        "XDG_CONFIG_HOME",
        "$yzn_config_home/mars/config.toml",
        "MARS_CONFIG_HOME=\"$yzn_config_home/mars\"",
        "MARS_CONFIG_HOME=/nix/store/",
    ] {
        assert!(
            launcher.contains(expected),
            "bin/yzn is missing Mars config override fragment: {expected}",
        );
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

    fs::write(&sidecar, "keybinds {}\n").unwrap();
    let output = Command::new(&helper)
        .arg(&packaged_config)
        .arg(&sidecar)
        .arg(&generated_path)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "dangerous Zellij sidecar unexpectedly succeeded"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("forbidden Zellij sidecar item `keybinds`"),
        "unexpected Zellij sidecar rejection: {stderr}",
    );
}

fn run_zellij_config(
    helper: &Path,
    packaged_config: &Path,
    sidecar: &Path,
    generated: &Path,
) -> String {
    let output = Command::new(helper)
        .arg(packaged_config)
        .arg(sidecar)
        .arg(generated)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{} failed with status {}\n{}",
        helper.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .trim_end_matches('\n')
        .to_owned()
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
    expect_no_block_binds_and_unbinds_same_key(config);
    assert!(
        !config.contains(r#"SwitchToMode "Move""#),
        "config.kdl must not reintroduce move mode"
    );
}

fn expect_lazygit_popup(config: &str) {
    for expected in [
        "share/yazelix_zellij_popup/yzpp.wasm",
        "load_plugins",
        "popup {",
        "pane_title \"lazygit_popup\"",
        "support_kitty_keyboard_protocol true",
        "bind \"Alt Shift J\"",
        "MessagePlugin \"yzpp\"",
        "name \"toggle\"",
        "/bin/lazygit\"",
    ] {
        assert!(
            config.contains(expected),
            "config.kdl is missing LazyGit popup fragment {expected:?}",
        );
    }
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
