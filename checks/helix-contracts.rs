use std::{env, fs, path::Path, process::Command};

mod support;

use support::{
    binary_text, embedded_store_path, excerpt, expect_contains, expect_order, write_executable,
    RuntimeCase, TempDir,
};

macro_rules! expect_contains_all {
    ($haystack:expr, $context:expr; $($needle:expr),+ $(,)?) => {
        $(expect_contains($haystack, &$needle, $context);)+
    };
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let [_, yzn, out] = args.as_slice() else {
        panic!("usage: helix-contracts-check <yzn-package> <out>");
    };

    let yzn = Path::new(yzn);
    let yzn_launcher = binary_text(&yzn.join("bin/yzn"));
    let helix = embedded_store_path(&yzn_launcher, "/bin/yzn-hx");

    expect_helix_wrapper(&helix);
    expect_helix_doctor_warnings(yzn);

    fs::write(out, "ok\n").unwrap();
}

fn expect_helix_wrapper(helix: &Path) {
    let helix_script = fs::read_to_string(helix).unwrap();
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
    expect_order(
        &helix_config,
        &["A-ret = [", "ret = [", "C-j = ["],
        "managed Helix enter movement bindings",
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
}

fn expect_helix_doctor_warnings(yzn: &Path) {
    let yzn_bin = yzn.join("bin/yzn");
    let temp = TempDir::new();

    let default = RuntimeCase::new(&temp.path, "default");
    default.write_default_config("");
    let doctor = default.run_yzn(&yzn_bin, "doctor", "default Helix doctor");
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
    let doctor = helix_override.run_yzn(&yzn_bin, "doctor", "Helix preference doctor");
    assert!(
        !doctor.contains("warn helix config:"),
        "ordinary Helix preference override should not warn\n{}",
        excerpt(&doctor)
    );

    fs::write(&helix_override_config, "[keys.normal]\nA-r = \":noop\"\n").unwrap();
    let doctor = helix_override.run_yzn(&yzn_bin, "doctor", "Helix Alt r doctor");
    expect_contains_all! {
        &doctor, "Helix Alt r doctor";
        r#"warn helix config: helix config override sets reserved Alt r; generated config keeps ':sh yzn reveal "%{buffer_name}"'"#,
        helix_override_config.display().to_string(),
    }
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

    for (name, files, uses_user_steel) in [
        ("packaged", &[] as &[(&str, &str)], false),
        (
            "languages",
            &[("languages.toml", "# managed languages\n")] as &[(&str, &str)],
            false,
        ),
        (
            "toml",
            &[(
                "config.toml",
                "[editor]\nline-number = \"relative\"\n\n[keys.normal]\nA-r = \":noop\"\nC-r = \":noop\"\n",
            )] as &[(&str, &str)],
            false,
        ),
        (
            "steel",
            &[("helix.scm", ";; module\n"), ("init.scm", ";; init\n")] as &[(&str, &str)],
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
    let expected_config_file = state.join("helix/config.toml");
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
    let generated_config = fs::read_to_string(&expected_config_file).unwrap();
    expect_contains_all! {
        &generated_config, &format!("{name} generated Helix reveal binding");
        "A-r = ",
        ":sh yzn reveal",
        "%{buffer_name}",
    }
    if name == "toml" {
        expect_contains_all! {
            &generated_config, "user Helix TOML merge";
            "line-number = \"relative\"",
            "C-r = \":noop\"",
        }
        assert!(
            !generated_config.contains("A-r = \":noop\""),
            "generated config kept user Alt r override\n{}",
            excerpt(&generated_config)
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
