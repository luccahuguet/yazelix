use std::{env, fs, path::Path, path::PathBuf, process::Command};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzn, out] = args.as_slice() else {
        panic!("usage: yzn-contracts-check <yzn-package> <out>");
    };

    let config = fs::read_to_string(Path::new(yzn).join("share/yazelix-next/config.kdl")).unwrap();
    let yzn_nu = default_shell(&config);
    assert!(
        yzn_nu.is_file(),
        "default_shell is not a file: {}",
        yzn_nu.display()
    );
    expect_keybinds(&config);
    expect_mars_config_override(Path::new(yzn));

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
        "exec /nix/store/",
    ] {
        assert!(
            launcher.contains(expected),
            "bin/yzn is missing Mars config override fragment: {expected}",
        );
    }
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
