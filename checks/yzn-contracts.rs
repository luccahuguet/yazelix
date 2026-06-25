use std::{env, fs, path::Path, path::PathBuf, process::Command};

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzn, out] = args.as_slice() else {
        panic!("usage: yzn-contracts-check <yzn-package> <out>");
    };

    let yzn_nu = default_shell(Path::new(yzn));
    assert!(
        yzn_nu.is_file(),
        "default_shell is not a file: {}",
        yzn_nu.display()
    );

    let temp = TempDir::new();
    let user_nu = temp.path.join("config/nu");
    let runtime = temp.path.join("run");
    fs::create_dir_all(&user_nu).unwrap();
    fs::create_dir_all(&runtime).unwrap();
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

    let output = Command::new(&yzn_nu)
        .arg("--commands")
        .arg("print $env.STARSHIP_SHELL; print $env.YZN_USER_ENV_TEST; print $env.YZN_USER_CONFIG_TEST; ^carapace --version | ignore; ^zoxide --version | ignore; print ok")
        .env("XDG_RUNTIME_DIR", &runtime)
        .env("YAZELIX_NEXT_CONFIG_HOME", temp.path.join("config"))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{} failed with status {}\n{}",
        yzn_nu.display(),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim_end_matches('\n'), "nu\nenv-ok\nconfig-ok\nok");

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

fn default_shell(yzn: &Path) -> PathBuf {
    let config = fs::read_to_string(yzn.join("share/yazelix-next/config.kdl")).unwrap();
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
