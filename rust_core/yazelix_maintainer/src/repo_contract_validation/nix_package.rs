use super::{
    build_flake_output_path, build_nix_file_output_path, command_output_summary,
    create_unique_temp_dir, format_json_value, prepare_temp_home, require_non_empty_dir_abs,
    require_path_exists_abs, require_path_missing_abs,
};
use crate::repo_validation::ValidationReport;
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[derive(Debug, Clone)]
pub struct ColdProfileInstallOptions {
    pub phase: String,
    pub temp_home: Option<PathBuf>,
}

impl Default for ColdProfileInstallOptions {
    fn default() -> Self {
        Self {
            phase: "all".to_string(),
            temp_home: None,
        }
    }
}

pub fn validate_nixpkgs_package(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let package_root = build_flake_output_path(
        repo_root,
        "yazelix",
        "building .#yazelix during nixpkgs package validation",
    )?;
    verify_yazelix_package(&package_root, &mut report.errors)?;
    Ok(report)
}

pub fn validate_nixpkgs_submission(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let package_root = build_nix_file_output_path(
        repo_root,
        Path::new("packaging").join("nixpkgs").join("default.nix"),
        "building packaging/nixpkgs/default.nix during nixpkgs submission validation",
    )?;
    verify_yazelix_package(&package_root, &mut report.errors)?;
    Ok(report)
}

pub fn validate_flake_profile_install(
    repo_root: &Path,
    options: &ColdProfileInstallOptions,
) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    match options.phase.as_str() {
        "all" => {
            let temp_home = match &options.temp_home {
                Some(path) => {
                    prepare_temp_home(path)?;
                    path.clone()
                }
                None => create_unique_temp_dir("yazelix_profile_install")?,
            };
            run_profile_install(repo_root, &temp_home, &mut report.errors)?;
            if report.errors.is_empty() {
                verify_profile_installed_runtime(repo_root, &temp_home, &mut report.errors)?;
            }
            let _ = fs::remove_dir_all(&temp_home);
        }
        "install" => {
            let Some(temp_home) = &options.temp_home else {
                return Err("The `install` phase requires an explicit temp_home path".to_string());
            };
            prepare_temp_home(temp_home)?;
            run_profile_install(repo_root, temp_home, &mut report.errors)?;
        }
        "verify" => {
            let Some(temp_home) = &options.temp_home else {
                return Err("The `verify` phase requires an explicit temp_home path".to_string());
            };
            require_path_exists_abs(
                temp_home,
                "cold profile-install temp home",
                &mut report.errors,
            );
            if report.errors.is_empty() {
                verify_profile_installed_runtime(repo_root, temp_home, &mut report.errors)?;
            }
        }
        other => {
            return Err(format!(
                "Unsupported cold profile-install phase `{}`. Expected one of: all, install, verify",
                other
            ));
        }
    }
    Ok(report)
}

fn verify_yazelix_package(package_root: &Path, errors: &mut Vec<String>) -> Result<(), String> {
    let temp_home = create_unique_temp_dir("yazelix_nixpkgs_package")?;
    let validation = (|| -> Result<(), String> {
        require_path_exists_abs(
            &package_root.join("bin").join("yzx"),
            "packaged yzx wrapper",
            errors,
        );
        require_path_missing_abs(
            &package_root.join("yazelix_packs_default.toml"),
            "legacy packaged packs config",
            errors,
        );
        require_path_absent_even_if_broken_symlink(
            &package_root.join("rust_plugins"),
            "packaged Rust plugin source tree",
            errors,
        );
        if !errors.is_empty() {
            return Ok(());
        }

        let version_result = run_packaged_yzx(package_root, &temp_home, &["--version-short"])?;
        if !version_result.status.success() {
            errors.push(format!(
                "Packaged yzx --version-short failed\n{}",
                command_output_summary(&version_result)
            ));
            return Ok(());
        }
        let version_text = String::from_utf8_lossy(&version_result.stdout)
            .trim()
            .to_string();
        if !version_text.starts_with("Yazelix (v") {
            errors.push(format!(
                "Unexpected packaged yzx version output: {}",
                version_text
            ));
        }

        let doctor_result = run_packaged_yzx(package_root, &temp_home, &["doctor", "--verbose"])?;
        if !doctor_result.status.success() {
            errors.push(format!(
                "Packaged yzx doctor --verbose failed\n{}",
                command_output_summary(&doctor_result)
            ));
            return Ok(());
        }

        let runtime_probe = run_packaged_yzx(
            package_root,
            &temp_home,
            &["run", "nu", "-c", RUNTIME_ENV_PROBE_NU],
        )?;
        if !runtime_probe.status.success() {
            errors.push(format!(
                "Packaged yzx run probe failed\n{}",
                command_output_summary(&runtime_probe)
            ));
            return Ok(());
        }
        let probe: JsonValue = serde_json::from_slice(&runtime_probe.stdout)
            .map_err(|error| format!("Failed to parse packaged runtime probe JSON: {}", error))?;
        let expected_bin = package_root.join("bin").display().to_string();
        let expected_toolbin = package_root.join("toolbin").display().to_string();
        validate_runtime_env_probe(
            &probe,
            package_root,
            &expected_toolbin,
            &expected_bin,
            None,
            "Packaged Yazelix runtime probe",
            errors,
        );

        require_path_exists_abs(
            &package_root.join("toolbin").join("rg"),
            "exported runtime tool `rg`",
            errors,
        );
        require_path_missing_abs(
            &package_root.join("toolbin").join("dirname"),
            "runtime-private helper leaked into exported toolbin",
            errors,
        );
        Ok(())
    })();
    let _ = fs::remove_dir_all(&temp_home);
    validation
}

fn require_path_absent_even_if_broken_symlink(path: &Path, label: &str, errors: &mut Vec<String>) {
    if fs::symlink_metadata(path).is_ok() {
        errors.push(format!("Unexpected {}: {}", label, path.display()));
    }
}

fn run_profile_install(
    repo_root: &Path,
    temp_home: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let state_root = temp_home.join(".local").join("share");
    let config_root = temp_home.join(".config");
    let profile_root = temp_home.join(".nix-profile");
    let output = Command::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "profile",
            "add",
            "--profile",
        ])
        .arg(&profile_root)
        .arg(".#yazelix")
        .current_dir(repo_root)
        .env("HOME", temp_home)
        .env("XDG_CONFIG_HOME", &config_root)
        .env("XDG_DATA_HOME", &state_root)
        .env_remove("YAZELIX_CONFIG_DIR")
        .env_remove("YAZELIX_CONFIG_OVERRIDE")
        .env_remove("YAZELIX_LOGS_DIR")
        .env_remove("YAZELIX_RUNTIME_DIR")
        .env_remove("YAZELIX_STATE_DIR")
        .output()
        .map_err(|error| format!("Failed to run cold profile install: {}", error))?;
    if !output.status.success() {
        errors.push(format!(
            "Cold profile-install validation failed while running `nix profile add --profile ... .#yazelix`\n{}",
            command_output_summary(&output)
        ));
    }
    Ok(())
}

fn verify_profile_installed_runtime(
    repo_root: &Path,
    temp_home: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let profile_root = temp_home.join(".nix-profile");
    let yzx_path = profile_root.join("bin").join("yzx");
    let local_wrapper = temp_home.join(".local").join("bin").join("yzx");
    let legacy_runtime_link = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("runtime")
        .join("current");
    let desktop_entry = temp_home
        .join(".local")
        .join("share")
        .join("applications")
        .join("com.yazelix.Yazelix.desktop");
    let user_config = temp_home
        .join(".config")
        .join("yazelix")
        .join("yazelix.toml");
    let pack_config = temp_home
        .join(".config")
        .join("yazelix")
        .join("yazelix_packs.toml");
    let nushell_config = temp_home.join(".config").join("nushell").join("config.nu");

    require_path_exists_abs(&yzx_path, "profile-installed yzx wrapper", errors);
    require_path_missing_abs(&local_wrapper, "legacy user-local yzx wrapper", errors);
    require_path_missing_abs(
        &legacy_runtime_link,
        "legacy installed runtime symlink",
        errors,
    );
    require_path_missing_abs(
        &desktop_entry,
        "default user-local desktop entry before explicit desktop install",
        errors,
    );
    require_path_missing_abs(
        &user_config,
        "managed user config before first runtime entry",
        errors,
    );
    require_path_missing_abs(&pack_config, "legacy managed pack config", errors);
    require_path_missing_abs(
        &nushell_config,
        "host Nushell hook config before first runtime entry",
        errors,
    );
    if !errors.is_empty() {
        return Ok(());
    }

    let wrapper_target = fs::canonicalize(&yzx_path).map_err(|error| {
        format!(
            "Failed to resolve installed yzx wrapper target {}: {}",
            yzx_path.display(),
            error
        )
    })?;
    let runtime_root = wrapper_target
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| {
            format!(
                "Installed yzx wrapper target has no runtime root: {}",
                wrapper_target.display()
            )
        })?
        .to_path_buf();
    let runtime_bin = runtime_root.join("bin");
    let runtime_toolbin = runtime_root.join("toolbin");
    let runtime_libexec = runtime_root.join("libexec");
    let runtime_variant = fs::read_to_string(runtime_root.join("runtime_variant"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());
    let runtime_terminal = match runtime_variant.as_str() {
        "kitty" => "kitty",
        "rio" => "rio",
        "wezterm" => "wezterm",
        "ratty" => "ratty",
        "yzxterm" => "yzxterm",
        _ => "ghostty",
    };
    let runtime_terminal_command = match runtime_terminal {
        "yzxterm" => "yazelix-terminal-desktop",
        other => other,
    };
    let runtime_yzx_cli = runtime_root.join("shells").join("posix").join("yzx_cli.sh");
    let runtime_yzx_core = runtime_libexec.join("yzx_core");
    let runtime_ghostty_wrapper = runtime_root
        .join("shells")
        .join("posix")
        .join("yazelix_ghostty.sh");
    let runtime_settings_default = runtime_root.join("settings_default.jsonc");
    let runtime_ghostty_shader_root = runtime_root
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty")
        .join("shaders");
    let generated_ghostty_root = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("ghostty");
    let generated_ghostty_config = generated_ghostty_root.join("config");
    let generated_ghostty_effect_dir = generated_ghostty_root
        .join("shaders")
        .join("generated_effects");
    let generated_wezterm_config = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("wezterm")
        .join(".wezterm.lua");
    let generated_kitty_config = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("kitty")
        .join("kitty.conf");
    let generated_rio_config = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("rio")
        .join("config.toml");
    let generated_yzxterm_config = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("yzxterm")
        .join("config.toml");
    let generated_ratty_config = temp_home
        .join(".local")
        .join("share")
        .join("yazelix")
        .join("configs")
        .join("terminal_emulators")
        .join("ratty")
        .join("ratty.toml");

    for (path, label) in [
        (runtime_toolbin.clone(), "runtime toolbin"),
        (runtime_libexec.join("nu"), "runtime-local Nushell binary"),
        (
            runtime_libexec.join("yzx"),
            "runtime-local Rust yzx root helper",
        ),
        (runtime_yzx_core.clone(), "runtime-local yzx_core helper"),
        (
            runtime_libexec.join("yzx_control"),
            "runtime-local yzx_control helper",
        ),
        (runtime_yzx_cli.clone(), "runtime-local POSIX yzx launcher"),
        (
            runtime_ghostty_wrapper.clone(),
            "runtime-local Ghostty env wrapper",
        ),
        (
            runtime_settings_default.clone(),
            "runtime-local default config",
        ),
        (
            runtime_ghostty_shader_root.join("build_shaders.nu"),
            "runtime-local Ghostty shader builder",
        ),
        (
            runtime_ghostty_shader_root
                .join("variants")
                .join("reef.glsl"),
            "runtime-local Ghostty trail shader variant",
        ),
        (
            runtime_ghostty_shader_root
                .join("upstream_effects")
                .join("ripple_rectangle_cursor.glsl"),
            "runtime-local Ghostty cursor effect template",
        ),
    ] {
        require_path_exists_abs(&path, label, errors);
    }
    if runtime_terminal == "yzxterm" {
        require_path_exists_abs(
            &runtime_root
                .join("share")
                .join("yazelix-terminal")
                .join("config.toml"),
            "runtime-local Yazelix Terminal packaged config",
            errors,
        );
    }

    for expected_tool in [
        "zellij",
        runtime_terminal_command,
        "yazi",
        "hx",
        "steel",
        "steel-language-server",
        "nvim",
        "fish",
        "zsh",
        "bash",
        "nix",
        "jq",
        "fd",
        "rg",
    ] {
        require_path_exists_abs(
            &runtime_libexec.join(expected_tool),
            &format!("runtime tool `{expected_tool}`"),
            errors,
        );
    }
    for expected_exported_tool in [
        "nu",
        "zellij",
        "yazi",
        "hx",
        "steel",
        "steel-language-server",
        "nvim",
        "bash",
        "jq",
        "fd",
        "rg",
    ] {
        require_path_exists_abs(
            &runtime_toolbin.join(expected_exported_tool),
            &format!("exported runtime tool `{expected_exported_tool}`"),
            errors,
        );
    }
    require_path_missing_abs(
        &runtime_toolbin.join("dirname"),
        "runtime-private helper leaked into exported toolbin",
        errors,
    );
    if cfg!(target_os = "linux") {
        require_path_exists_abs(
            &runtime_libexec.join("nixGLMesa"),
            "runtime tool `nixGLMesa`",
            errors,
        );
        if (runtime_terminal == "ratty" || runtime_terminal == "yzxterm")
            && !runtime_libexec.join("nixVulkanMesa").exists()
            && !runtime_libexec.join("nixVulkanIntel").exists()
        {
            errors.push(format!(
                "Missing runtime tool `nixVulkanMesa` or `nixVulkanIntel`: {}",
                runtime_libexec.display()
            ));
        }
        require_path_exists_abs(
            &runtime_libexec.join("pgrep"),
            "runtime tool `pgrep`",
            errors,
        );
    }

    let expected_wrapper_target = runtime_root.join("bin").join("yzx");
    if wrapper_target != expected_wrapper_target {
        errors.push(format!(
            "Installed yzx wrapper should point at the packaged runtime. Expected {}, got {}",
            expected_wrapper_target.display(),
            wrapper_target.display()
        ));
    }
    if !errors.is_empty() {
        return Ok(());
    }

    let version_result = run_installed_yzx(repo_root, temp_home, &["--version-short"])?;
    if !version_result.status.success() {
        errors.push(format!(
            "Installed yzx --version-short failed during cold profile-install validation\n{}",
            command_output_summary(&version_result)
        ));
        return Ok(());
    }
    let version_text = String::from_utf8_lossy(&version_result.stdout)
        .trim()
        .to_string();
    if !version_text.starts_with("Yazelix (v") {
        errors.push(format!(
            "Unexpected installed yzx version output: {}",
            version_text
        ));
    }

    let posix_launcher_result =
        run_runtime_posix_launcher_minimal_env(repo_root, temp_home, &runtime_yzx_cli)?;
    if !posix_launcher_result.status.success() {
        errors.push(format!(
            "Runtime-local POSIX yzx launcher failed under minimal PATH during cold profile-install validation\n{}",
            command_output_summary(&posix_launcher_result)
        ));
        return Ok(());
    }
    let posix_version_text = String::from_utf8_lossy(&posix_launcher_result.stdout)
        .trim()
        .to_string();
    if !posix_version_text.starts_with("Yazelix (v") {
        errors.push(format!(
            "Unexpected runtime-local POSIX yzx output: {}",
            posix_version_text
        ));
    }

    let runtime_probe = run_installed_yzx(
        repo_root,
        temp_home,
        &["run", "nu", "-c", INSTALLED_ENV_PROBE_NU],
    )?;
    if !runtime_probe.status.success() {
        errors.push(format!(
            "Installed yzx run probe failed during cold profile-install validation\n{}",
            command_output_summary(&runtime_probe)
        ));
        return Ok(());
    }
    let probe: JsonValue = serde_json::from_slice(&runtime_probe.stdout).map_err(|error| {
        format!(
            "Failed to parse installed runtime probe JSON during cold profile-install validation: {}",
            error
        )
    })?;
    validate_runtime_env_probe(
        &probe,
        &runtime_root,
        &runtime_toolbin.display().to_string(),
        &runtime_bin.display().to_string(),
        Some(&runtime_bin.join("yzx").display().to_string()),
        "Installed runtime probe",
        errors,
    );
    if !probe
        .get("editor")
        .and_then(JsonValue::as_str)
        .unwrap_or_default()
        .contains("yazelix_hx.sh")
    {
        errors.push(format!(
            "Installed runtime probe did not set EDITOR to the managed Helix wrapper: {}",
            format_json_value(&probe)
        ));
    }
    if !errors.is_empty() {
        return Ok(());
    }

    let yzx_core_arg = runtime_yzx_core.display().to_string();
    let materialization_result = run_installed_yzx(
        repo_root,
        temp_home,
        &[
            "run",
            &yzx_core_arg,
            "launch-materialization.prepare",
            "--from-env",
        ],
    )?;
    if !materialization_result.status.success() {
        errors.push(format!(
            "Installed runtime failed to materialize the selected terminal config during cold profile-install validation\n{}",
            command_output_summary(&materialization_result)
        ));
        return Ok(());
    }

    if runtime_terminal == "ghostty" {
        require_ghostty_shader_references_exist(&generated_ghostty_config, errors)?;
        require_non_empty_dir_abs(
            &generated_ghostty_effect_dir,
            "generated Ghostty cursor effect shaders directory",
            errors,
        )?;
    } else if runtime_terminal == "kitty" {
        require_path_exists_abs(
            &generated_kitty_config,
            "generated Kitty config for selected runtime variant",
            errors,
        );
    } else if runtime_terminal == "rio" {
        require_path_exists_abs(
            &generated_rio_config,
            "generated Rio config for selected runtime variant",
            errors,
        );
    } else if runtime_terminal == "wezterm" {
        require_path_exists_abs(
            &generated_wezterm_config,
            "generated WezTerm config for selected runtime variant",
            errors,
        );
    } else if runtime_terminal == "ratty" {
        require_path_exists_abs(
            &generated_ratty_config,
            "generated Ratty config for selected runtime variant",
            errors,
        );
    } else if runtime_terminal == "yzxterm" {
        require_path_exists_abs(
            &generated_yzxterm_config,
            "generated Yazelix Terminal config for selected runtime variant",
            errors,
        );
    }
    if errors.is_empty() {
        verify_profile_desktop_install_path(
            repo_root,
            temp_home,
            &yzx_path,
            &desktop_entry,
            errors,
        )?;
    }
    Ok(())
}

const RUNTIME_ENV_PROBE_NU: &str = r#"let runtime_dir = ($env.YAZELIX_RUNTIME_DIR | default ""); let path_entries = ($env.PATH | default []); let runtime_libexec = (if ($runtime_dir | is-empty) { "" } else { $runtime_dir | path join "libexec" }); print ({shell: ($env.IN_YAZELIX_SHELL | default ""), runtime: $runtime_dir, path0: ($path_entries | get -o 0 | default ""), path1: ($path_entries | get -o 1 | default ""), libexec_on_path: (if ($runtime_libexec | is-empty) { false } else { $path_entries | any {|entry| $entry == $runtime_libexec } }), yzx: ((which yzx | get -o 0.path | default ""))} | to json -r)"#;
const INSTALLED_ENV_PROBE_NU: &str = r#"let runtime_dir = ($env.YAZELIX_RUNTIME_DIR | default ""); let path_entries = ($env.PATH | default []); let runtime_libexec = (if ($runtime_dir | is-empty) { "" } else { $runtime_dir | path join "libexec" }); print ({shell: ($env.IN_YAZELIX_SHELL | default ""), runtime: $runtime_dir, path0: ($path_entries | get -o 0 | default ""), path1: ($path_entries | get -o 1 | default ""), libexec_on_path: (if ($runtime_libexec | is-empty) { false } else { $path_entries | any {|entry| $entry == $runtime_libexec } }), yzx: ((which yzx | get -o 0.path | default "")), editor: ($env.EDITOR | default "")} | to json -r)"#;

fn run_packaged_yzx(
    package_root: &Path,
    temp_home: &Path,
    args: &[&str],
) -> Result<Output, String> {
    Command::new(package_root.join("bin").join("yzx"))
        .args(args)
        .env("HOME", temp_home)
        .env("XDG_CONFIG_HOME", temp_home.join(".config"))
        .env("XDG_DATA_HOME", temp_home.join(".local").join("share"))
        .env("SHELL", "/usr/bin/true")
        .env_remove("YAZELIX_DIR")
        .env_remove("YAZELIX_CONFIG_DIR")
        .env_remove("YAZELIX_CONFIG_OVERRIDE")
        .env_remove("YAZELIX_LOGS_DIR")
        .env_remove("YAZELIX_RUNTIME_DIR")
        .env_remove("YAZELIX_STATE_DIR")
        .output()
        .map_err(|error| format!("Failed to run packaged yzx: {}", error))
}

fn run_installed_yzx(repo_root: &Path, temp_home: &Path, args: &[&str]) -> Result<Output, String> {
    Command::new(temp_home.join(".nix-profile").join("bin").join("yzx"))
        .args(args)
        .current_dir(repo_root)
        .env("HOME", temp_home)
        .env("XDG_CONFIG_HOME", temp_home.join(".config"))
        .env("XDG_DATA_HOME", temp_home.join(".local").join("share"))
        .env_remove("YAZELIX_CONFIG_DIR")
        .env_remove("YAZELIX_CONFIG_OVERRIDE")
        .env_remove("YAZELIX_LOGS_DIR")
        .env_remove("YAZELIX_RUNTIME_DIR")
        .env_remove("YAZELIX_STATE_DIR")
        .output()
        .map_err(|error| format!("Failed to run profile-installed yzx: {}", error))
}

fn verify_profile_desktop_install_path(
    repo_root: &Path,
    temp_home: &Path,
    yzx_path: &Path,
    desktop_entry: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let desktop_install = run_installed_yzx(repo_root, temp_home, &["desktop", "install"])?;
    if !desktop_install.status.success() {
        errors.push(format!(
            "Profile-installed yzx desktop install failed during cold profile-install validation\n{}",
            command_output_summary(&desktop_install)
        ));
        return Ok(());
    }

    require_path_exists_abs(desktop_entry, "profile-installed desktop entry", errors);
    if desktop_entry.exists() {
        validate_profile_desktop_entry_contents(desktop_entry, yzx_path, errors)?;
    }
    Ok(())
}

fn validate_profile_desktop_entry_contents(
    desktop_entry: &Path,
    yzx_path: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let raw = fs::read_to_string(desktop_entry).map_err(|error| {
        format!(
            "Failed to read profile-installed desktop entry {}: {}",
            desktop_entry.display(),
            error
        )
    })?;
    let exec_line = raw
        .lines()
        .find(|line| line.trim().starts_with("Exec="))
        .unwrap_or("")
        .trim()
        .to_string();
    let expected_exec = format!("Exec=\"{}\" desktop launch", yzx_path.display());
    for required in ["Name=Yazelix", "Terminal=true", "X-Yazelix-Managed=true"] {
        if !raw.lines().any(|line| line.trim() == required) {
            errors.push(format!(
                "Profile-installed desktop entry is missing `{required}`: {}",
                desktop_entry.display()
            ));
        }
    }
    if exec_line != expected_exec {
        errors.push(format!(
            "Profile-installed desktop entry Exec should target the profile yzx wrapper. Expected `{expected_exec}`, got `{exec_line}`"
        ));
    }
    Ok(())
}

fn run_runtime_posix_launcher_minimal_env(
    repo_root: &Path,
    temp_home: &Path,
    runtime_yzx_cli: &Path,
) -> Result<Output, String> {
    Command::new("env")
        .arg("-i")
        .arg(format!("HOME={}", temp_home.display()))
        .arg("PATH=/usr/bin:/bin")
        .arg(format!(
            "XDG_CONFIG_HOME={}",
            temp_home.join(".config").display()
        ))
        .arg(format!(
            "XDG_DATA_HOME={}",
            temp_home.join(".local").join("share").display()
        ))
        .arg("sh")
        .arg(runtime_yzx_cli)
        .arg("--version-short")
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to run runtime-local POSIX yzx launcher: {}", error))
}

fn validate_runtime_env_probe(
    probe: &JsonValue,
    runtime_root: &Path,
    expected_path0: &str,
    expected_path1: &str,
    expected_yzx: Option<&str>,
    label: &str,
    errors: &mut Vec<String>,
) {
    let shell = probe
        .get("shell")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let runtime = probe
        .get("runtime")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let path0 = probe
        .get("path0")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let path1 = probe
        .get("path1")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let libexec_on_path = probe
        .get("libexec_on_path")
        .and_then(JsonValue::as_bool)
        .unwrap_or(true);
    let yzx = probe
        .get("yzx")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    let expected_yzx = expected_yzx.unwrap_or_else(|| {
        if path1.is_empty() {
            ""
        } else {
            path1
                .strip_suffix('/')
                .unwrap_or(path1)
                .trim_end_matches('/')
        }
    });
    let expected_yzx = if expected_yzx == path1 {
        format!("{expected_yzx}/yzx")
    } else {
        expected_yzx.to_string()
    };

    if shell != "true"
        || runtime != runtime_root.display().to_string()
        || path0 != expected_path0
        || path1 != expected_path1
        || libexec_on_path
        || yzx != expected_yzx
    {
        errors.push(format!(
            "{} saw the wrong Yazelix env: {}",
            label,
            format_json_value(probe)
        ));
    }
}

fn resolve_ghostty_shader_reference(ghostty_config_path: &Path, shader_ref: &str) -> PathBuf {
    let raw_ref = shader_ref.trim().trim_matches('"');
    let path = Path::new(raw_ref);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    let relative = raw_ref.strip_prefix("./").unwrap_or(raw_ref);
    ghostty_config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(relative)
}

fn require_ghostty_shader_references_exist(
    ghostty_config_path: &Path,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    require_path_exists_abs(ghostty_config_path, "generated Ghostty config", errors);
    if !ghostty_config_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(ghostty_config_path).map_err(|error| {
        format!(
            "Failed to read generated Ghostty config {}: {}",
            ghostty_config_path.display(),
            error
        )
    })?;
    let shader_refs = content
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("custom-shader = ")
                .map(str::trim)
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    if shader_refs.is_empty() {
        errors.push(format!(
            "Generated Ghostty config references no shader assets: {}",
            ghostty_config_path.display()
        ));
    }
    for shader_ref in shader_refs {
        let shader_path = resolve_ghostty_shader_reference(ghostty_config_path, &shader_ref);
        require_path_exists_abs(
            &shader_path,
            &format!("generated Ghostty shader `{shader_ref}`"),
            errors,
        );
    }
    Ok(())
}

// Test lane: maintainer
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Defends: cold profile-install validation checks the explicit desktop-entry install path shape without relying on a maintainer's real profile.
    #[test]
    fn desktop_entry_validation_accepts_profile_owned_exec() {
        let temp = tempdir().unwrap();
        let yzx = temp.path().join(".nix-profile").join("bin").join("yzx");
        let desktop = temp.path().join("com.yazelix.Yazelix.desktop");
        fs::create_dir_all(yzx.parent().unwrap()).unwrap();
        fs::write(&yzx, "").unwrap();
        fs::write(
            &desktop,
            format!(
                "[Desktop Entry]\nName=Yazelix\nTerminal=true\nX-Yazelix-Managed=true\nExec=\"{}\" desktop launch\n",
                yzx.display()
            ),
        )
        .unwrap();

        let mut errors = Vec::new();
        validate_profile_desktop_entry_contents(&desktop, &yzx, &mut errors).unwrap();

        assert!(errors.is_empty(), "{errors:?}");
    }

    // Regression: the profile smoke must reject desktop entries that point at an unrelated local wrapper.
    #[test]
    fn desktop_entry_validation_rejects_wrong_exec_owner() {
        let temp = tempdir().unwrap();
        let yzx = temp.path().join(".nix-profile").join("bin").join("yzx");
        let desktop = temp.path().join("com.yazelix.Yazelix.desktop");
        fs::create_dir_all(yzx.parent().unwrap()).unwrap();
        fs::write(&yzx, "").unwrap();
        fs::write(
            &desktop,
            "[Desktop Entry]\nName=Yazelix\nTerminal=true\nX-Yazelix-Managed=true\nExec=\"/old/bin/yzx\" desktop launch\n",
        )
        .unwrap();

        let mut errors = Vec::new();
        validate_profile_desktop_entry_contents(&desktop, &yzx, &mut errors).unwrap();

        assert!(
            errors
                .iter()
                .any(|error| error.contains("profile yzx wrapper"))
        );
    }
}
