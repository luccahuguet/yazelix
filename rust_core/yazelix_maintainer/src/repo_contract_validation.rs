use crate::repo_validation::ValidationReport;
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::{Table as TomlTable, Value as TomlValue};

mod config_surface;
mod readme_surface;
mod upgrade_contract;

pub use config_surface::{
    validate_config_surface_contract, validate_home_manager_option_declaration_contract,
};
pub use readme_surface::{ReadmeSyncResult, sync_readme_surface, validate_readme_version};
pub use upgrade_contract::{UpgradeContractOptions, validate_upgrade_contract};

const MAIN_TEMPLATE_RELATIVE_PATH: &str = "yazelix_default.toml";
const MODULE_RELATIVE_PATH: &str = "home_manager/module.nix";
const HOME_MANAGER_MODULE_DECLARATION_PATH: &str = "yazelix/home_manager/module.nix";
const MAIN_CONTRACT_RELATIVE_PATH: &str = "config_metadata/main_config_contract.toml";
const TOML_TOOLING_CONFIG_RELATIVE_PATH: &str = "tombi.toml";

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

pub fn validate_installed_runtime_contract(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    report
        .errors
        .extend(validate_installed_runtime_contract_inner(repo_root)?);
    Ok(report)
}

pub fn validate_flake_interface(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let ok = run_nix_eval(repo_root, &build_flake_interface_expr(repo_root))?
        .as_bool()
        .ok_or("Top-level flake interface validation did not return a boolean")?;
    if !ok {
        report.errors.push(
            "Top-level flake interface is missing required package/app/Home Manager outputs, still exposes legacy install outputs, or still points packages.default at the lower-level runtime."
                .to_string(),
        );
    }

    let platform_rows = run_nix_eval(repo_root, &build_flake_package_platform_expr(repo_root))?;
    let rows = platform_rows
        .as_array()
        .ok_or("First-party flake package platform validation did not return a JSON array")?;
    let unavailable = rows
        .iter()
        .filter(|row| {
            !row.get("available")
                .and_then(JsonValue::as_bool)
                .unwrap_or(false)
        })
        .map(|row| {
            let system = row
                .get("system")
                .and_then(JsonValue::as_str)
                .unwrap_or("<unknown>");
            let platforms = row.get("platforms").unwrap_or(&JsonValue::Null);
            format!("{system} (meta.platforms={})", format_json_value(platforms))
        })
        .collect::<Vec<_>>();
    if !unavailable.is_empty() {
        report.errors.push(format!(
            "First-party flake package reports as unavailable on exported systems: {}. Each system exported in flake.nix must be included in the package meta.platforms.",
            unavailable.join(", ")
        ));
    }

    Ok(report)
}

pub fn validate_nix_customization_api(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let result = run_nix_eval(repo_root, &build_nix_customization_api_expr(repo_root))?;
    let object = result
        .as_object()
        .ok_or("Nix customization API validation did not return a JSON object")?;

    require_json_bool(
        object,
        "has_mk_yazelix",
        "flake lib must expose lib.<system>.mkYazelix",
        &mut report.errors,
    );
    require_json_string(
        object,
        "default_main_program",
        "yzx",
        "default flake package must expose yzx as the main program",
        &mut report.errors,
    );
    require_json_string(
        object,
        "mk_default_main_program",
        "yzx",
        "lib.<system>.mkYazelix default package must expose yzx as the main program",
        &mut report.errors,
    );
    require_json_string(
        object,
        "overlay_main_program",
        "yzx",
        "overlays.default must expose a yazelix package with yzx as the main program",
        &mut report.errors,
    );
    require_json_string(
        object,
        "home_manager_runtime_tool_source",
        "host",
        "Home Manager runtime_tool_sources must pass typed host values through evaluation",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "home_manager_has_package",
        "Home Manager evaluation must install a Yazelix package",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "invalid_runtime_tool_rejected",
        "invalid runtimeToolSources host modes must fail during Nix evaluation",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "invalid_component_rejected",
        "unsupported component disabling must fail during Nix evaluation",
        &mut report.errors,
    );

    Ok(report)
}

fn require_json_bool(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    message: &str,
    errors: &mut Vec<String>,
) {
    if object.get(key).and_then(JsonValue::as_bool) != Some(true) {
        errors.push(message.to_string());
    }
}

fn require_json_string(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    expected: &str,
    message: &str,
    errors: &mut Vec<String>,
) {
    if object.get(key).and_then(JsonValue::as_str) != Some(expected) {
        errors.push(message.to_string());
    }
}

pub fn validate_nushell_syntax(
    repo_root: &Path,
    verbose: bool,
) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let files = collect_nushell_syntax_files(repo_root)?;
    if files.is_empty() {
        report
            .errors
            .push("No Nushell scripts found to validate".to_string());
        return Ok(report);
    }

    for path in files {
        let relative = relative_display(repo_root, &path);
        let output = Command::new("nu")
            .args(["--no-config-file", "--ide-check", "100"])
            .arg(&path)
            .current_dir(repo_root)
            .output()
            .map_err(|error| format!("Failed to run `nu --ide-check`: {}", error))?;
        if !output.status.success() {
            report.errors.push(format!(
                "Nushell syntax check failed to inspect {}\n{}",
                relative,
                command_output_summary(&output)
            ));
            continue;
        }

        let diagnostics = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<JsonValue>(line).ok())
            .filter(|item| {
                item.get("type").and_then(JsonValue::as_str) == Some("diagnostic")
                    && item.get("severity").and_then(JsonValue::as_str) == Some("Error")
            })
            .map(|item| {
                item.get("message")
                    .and_then(JsonValue::as_str)
                    .unwrap_or("unknown Nushell parser diagnostic")
                    .to_string()
            })
            .collect::<Vec<_>>();
        if diagnostics.is_empty() {
            if verbose {
                report
                    .warnings
                    .push(format!("nu --ide-check passed: {relative}"));
            }
        } else {
            report.errors.push(format!(
                "Nushell syntax error in {}\n{}",
                relative,
                diagnostics.join("\n")
            ));
        }
    }

    Ok(report)
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

fn validate_installed_runtime_contract_inner(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    let cli_wrapper = "shells/posix/yzx_cli.sh";
    let desktop_deferred_launch_probe = "shells/posix/desktop_deferred_launch_probe.sh";
    let detached_launch_probe = "shells/posix/detached_launch_probe.sh";
    let runtime_env = "shells/posix/runtime_env.sh";
    let environment_setup = "nushell/scripts/setup/environment.nu";
    let runtime_tree = "packaging/mk_runtime_tree.nix";
    let flake_path = "flake.nix";

    require_path_exists(repo_root, flake_path, "flake definition", &mut errors);
    require_path_missing(
        repo_root,
        "shells/posix/install_yazelix.sh.in",
        "legacy flake installer template",
        &mut errors,
    );
    require_path_exists(
        repo_root,
        cli_wrapper,
        "stable POSIX CLI wrapper",
        &mut errors,
    );
    require_path_exists(
        repo_root,
        desktop_deferred_launch_probe,
        "desktop deferred launch probe helper",
        &mut errors,
    );
    require_path_exists(
        repo_root,
        detached_launch_probe,
        "detached launch probe helper",
        &mut errors,
    );
    require_path_exists(repo_root, runtime_env, "runtime env helper", &mut errors);
    require_path_exists(
        repo_root,
        environment_setup,
        "environment setup script",
        &mut errors,
    );
    require_path_exists(repo_root, runtime_tree, "runtime tree builder", &mut errors);

    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"export YAZELIX_BOOTSTRAP_RUNTIME_DIR="$RUNTIME_DIR""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"runtime_env_script="$RUNTIME_DIR/shells/posix/runtime_env.sh""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"yzx_root_bin="${YAZELIX_YZX_BIN:-$RUNTIME_DIR/libexec/yzx}""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        cli_wrapper,
        r#"exec "$yzx_root_bin" "$@""#,
        "stable POSIX CLI wrapper",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        runtime_env,
        "export YAZELIX_DIR=",
        "runtime env helper",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        environment_setup,
        "get_installed_yazelix_runtime_reference_dir",
        "environment setup script",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        environment_setup,
        "ensure_user_cli_wrapper",
        "environment setup script",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        "import ./runtime_deps.nix",
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        r#"ln -s ${src}/yazelix_default.toml "$out/yazelix_default.toml""#,
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        "for bin_dir in ${escapedRuntimeBinDirs}; do",
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_contains(
        repo_root,
        runtime_tree,
        r#"cat > "$out/bin/yzx" <<EOF"#,
        "runtime tree builder",
        &mut errors,
    )?;
    require_file_not_contains(
        repo_root,
        runtime_tree,
        "yazelix_packs_default.toml",
        "runtime tree builder",
        &mut errors,
    )?;

    if !errors.is_empty() {
        return Ok(errors);
    }

    let flake_show = run_repo_command(repo_root, "nix", &["flake", "show", "--json"])?;
    if !flake_show.status.success() {
        errors.push(format!(
            "Failed to evaluate flake outputs during installed-runtime contract validation\n{}",
            command_stderr(&flake_show)
        ));
        return Ok(errors);
    }
    let flake: JsonValue = serde_json::from_slice(&flake_show.stdout)
        .map_err(|error| format!("Failed to parse `nix flake show --json`: {}", error))?;
    let package_keys = json_object_keys(
        flake
            .pointer("/packages/x86_64-linux")
            .ok_or("Missing packages.x86_64-linux in flake output")?,
    );
    for expected in ["default", "runtime", "yazelix"] {
        require_list_contains(
            &package_keys,
            expected,
            "x86_64-linux package outputs",
            &mut errors,
        );
    }
    for forbidden in ["install", "locked_devenv"] {
        require_list_not_contains(
            &package_keys,
            forbidden,
            "x86_64-linux package outputs",
            &mut errors,
        );
    }
    let app_keys = json_object_keys(
        flake
            .pointer("/apps/x86_64-linux")
            .ok_or("Missing apps.x86_64-linux in flake output")?,
    );
    for expected in ["default", "yazelix"] {
        require_list_contains(&app_keys, expected, "x86_64-linux app outputs", &mut errors);
    }
    require_list_not_contains(
        &app_keys,
        "install",
        "x86_64-linux app outputs",
        &mut errors,
    );

    if !errors.is_empty() {
        return Ok(errors);
    }

    let runtime_out = build_flake_output_path(
        repo_root,
        "runtime",
        "building runtime package for installed-runtime validation",
    )?;
    validate_rust_routed_nu_modules(&runtime_out, "built runtime package", &mut errors);
    require_path_exists_abs(
        &runtime_out.join(desktop_deferred_launch_probe),
        "built runtime desktop deferred launch probe helper",
        &mut errors,
    );
    require_path_exists_abs(
        &runtime_out.join(detached_launch_probe),
        "built runtime detached launch probe helper",
        &mut errors,
    );

    let yazelix_out = build_flake_output_path(
        repo_root,
        "yazelix",
        "building yazelix package for installed-runtime validation",
    )?;
    validate_rust_routed_nu_modules(&yazelix_out, "built yazelix package", &mut errors);
    require_path_exists_abs(
        &yazelix_out.join(desktop_deferred_launch_probe),
        "built yazelix desktop deferred launch probe helper",
        &mut errors,
    );
    require_path_exists_abs(
        &yazelix_out.join(detached_launch_probe),
        "built yazelix detached launch probe helper",
        &mut errors,
    );
    errors.extend(validate_home_manager_activation_contract(repo_root)?);

    let built_yzx = yazelix_out.join("bin").join("yzx");
    require_path_exists_abs(&built_yzx, "built yazelix CLI wrapper", &mut errors);
    if !errors.is_empty() {
        return Ok(errors);
    }

    let built_yzx_string = built_yzx.display().to_string();
    let smoke_result = Command::new("env")
        .args([
            "YAZELIX_SKIP_STABLE_WRAPPER_REDIRECT=1",
            built_yzx_string.as_str(),
            "why",
        ])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to smoke-run built yazelix public CLI: {}", error))?;
    if !smoke_result.status.success() {
        errors.push(format!(
            "Built yazelix package failed the public CLI smoke check\n{}",
            command_output_summary(&smoke_result)
        ));
        return Ok(errors);
    }
    require_file_contains_abs(
        &built_yzx,
        "shells/posix/yzx_cli.sh",
        "built yazelix CLI wrapper",
        &mut errors,
    )?;
    let smoke_stdout = String::from_utf8_lossy(&smoke_result.stdout);
    if !smoke_stdout.contains("Yazelix is a reproducible terminal IDE") {
        errors.push(
            "Built yazelix public CLI smoke check returned unexpected output for `yzx why`"
                .to_string(),
        );
    }

    Ok(errors)
}

fn validate_home_manager_activation_contract(repo_root: &Path) -> Result<Vec<String>, String> {
    let mut errors = Vec::new();
    errors.extend(validate_home_manager_activation_mode(repo_root, false)?);
    errors.extend(validate_home_manager_activation_mode(repo_root, true)?);
    Ok(errors)
}

fn validate_home_manager_activation_mode(
    repo_root: &Path,
    manage_config: bool,
) -> Result<Vec<String>, String> {
    let temp_root = create_unique_temp_dir("yazelix_home_manager_activation")?;
    let cleanup_result = (|| {
        let flake_root = temp_root.join("flake");
        let home_root = temp_root.join("home");
        let system = resolve_nix_current_system(repo_root)?;
        let xdg_config_home = home_root.join(".config");
        let xdg_data_home = home_root.join(".local").join("share");
        fs::create_dir_all(&flake_root)
            .map_err(|error| format!("Failed to create {}: {}", flake_root.display(), error))?;
        fs::create_dir_all(&xdg_config_home).map_err(|error| {
            format!("Failed to create {}: {}", xdg_config_home.display(), error)
        })?;
        fs::create_dir_all(&xdg_data_home)
            .map_err(|error| format!("Failed to create {}: {}", xdg_data_home.display(), error))?;
        fs::write(
            flake_root.join("flake.nix"),
            build_home_manager_activation_validation_flake(
                repo_root,
                &home_root,
                &system,
                manage_config,
            ),
        )
        .map_err(|error| {
            format!(
                "Failed to write Home Manager activation validation flake: {}",
                error
            )
        })?;

        let build_output = Command::new("nix")
            .args([
                "build",
                "--no-link",
                "--print-out-paths",
                ".#homeConfigurations.validation.activationPackage",
            ])
            .current_dir(&flake_root)
            .output()
            .map_err(|error| {
                format!(
                    "Failed to build the temporary Home Manager activation package: {}",
                    error
                )
            })?;
        if !build_output.status.success() {
            return Ok(vec![format!(
                "Temporary Home Manager activation package failed to build\n{}",
                command_output_summary(&build_output)
            )]);
        }

        let stdout = String::from_utf8_lossy(&build_output.stdout);
        let Some(activation_package) = stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(PathBuf::from)
        else {
            return Ok(vec![
                "Temporary Home Manager activation build did not return an activation package path"
                    .to_string(),
            ]);
        };

        let activate_script = activation_package.join("activate");
        let mut errors = Vec::new();
        require_path_exists_abs(
            &activate_script,
            "temporary Home Manager activation script",
            &mut errors,
        );
        if !errors.is_empty() {
            return Ok(errors);
        }

        let activate_output = Command::new(&activate_script)
            .env("HOME", &home_root)
            .env("USER", "validator")
            .env("XDG_CONFIG_HOME", &xdg_config_home)
            .env("XDG_DATA_HOME", &xdg_data_home)
            .env(
                "PATH",
                env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string()),
            )
            .current_dir(&flake_root)
            .output()
            .map_err(|error| {
                format!(
                    "Failed to run the temporary Home Manager activation script: {}",
                    error
                )
            })?;
        if !activate_output.status.success() {
            return Ok(vec![format!(
                "Temporary Home Manager activation script failed\n{}",
                command_output_summary(&activate_output)
            )]);
        }

        require_path_exists_abs(
            &home_root
                .join(".config")
                .join("yazelix")
                .join("settings.jsonc"),
            if manage_config {
                "Home Manager managed settings.jsonc surface after activation"
            } else {
                "Yazelix bootstrapped mutable settings.jsonc surface after Home Manager activation"
            },
            &mut errors,
        );
        let main_config_path = home_root
            .join(".config")
            .join("yazelix")
            .join("settings.jsonc");
        if let Ok(metadata) = fs::symlink_metadata(&main_config_path) {
            if manage_config && !metadata.file_type().is_symlink() {
                errors.push(format!(
                    "Home Manager managed settings.jsonc should be a profile symlink: {}",
                    main_config_path.display()
                ));
            }
            if !manage_config && metadata.file_type().is_symlink() {
                errors.push(format!(
                    "Home Manager manage_config=false should leave settings.jsonc mutable, got symlink: {}",
                    main_config_path.display()
                ));
            }
        }
        require_path_exists_abs(
            &home_root
                .join(".local")
                .join("share")
                .join("yazelix")
                .join("configs")
                .join("zellij")
                .join("config.kdl"),
            "generated Zellij config after Home Manager activation",
            &mut errors,
        );
        require_path_exists_abs(
            &home_root
                .join(".local")
                .join("share")
                .join("yazelix")
                .join("configs")
                .join("yazi")
                .join("yazi.toml"),
            "generated Yazi config after Home Manager activation",
            &mut errors,
        );
        // Home Manager activation repairs generated runtime state, while terminal
        // configs remain launch-time materialization outputs.
        Ok(errors)
    })();
    let _ = fs::remove_dir_all(&temp_root);
    cleanup_result
}

fn resolve_nix_current_system(repo_root: &Path) -> Result<String, String> {
    let output = run_repo_command(
        repo_root,
        "nix",
        &[
            "eval",
            "--raw",
            "--impure",
            "--expr",
            "builtins.currentSystem",
        ],
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed to resolve the current Nix system for Home Manager activation validation\n{}",
            command_output_summary(&output)
        ));
    }
    let system = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if system.is_empty() {
        return Err(
            "Home Manager activation validation could not resolve the current Nix system"
                .to_string(),
        );
    }
    Ok(system)
}

fn build_home_manager_activation_validation_flake(
    repo_root: &Path,
    home_root: &Path,
    system: &str,
    manage_config: bool,
) -> String {
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
    let home_root_literal = escape_nix_string(&home_root.display().to_string());
    let system_literal = escape_nix_string(system);
    [
        "{".to_string(),
        "  description = \"Yazelix Home Manager activation validation\";".to_string(),
        String::new(),
        "  inputs = {".to_string(),
        format!("    yazelix.url = \"path:{}\";", repo_root_literal),
        "    nixpkgs.follows = \"yazelix/nixpkgs\";".to_string(),
        "    home-manager.follows = \"yazelix/home-manager\";".to_string(),
        "  };".to_string(),
        String::new(),
        "  outputs = { nixpkgs, home-manager, yazelix, ... }:".to_string(),
        "    let".to_string(),
        format!("      system = \"{}\";", system_literal),
        "      pkgs = import nixpkgs { inherit system; };".to_string(),
        "    in {".to_string(),
        "      homeConfigurations.validation = home-manager.lib.homeManagerConfiguration {"
            .to_string(),
        "        inherit pkgs;".to_string(),
        "        modules = [".to_string(),
        "          yazelix.homeManagerModules.default".to_string(),
        "          ({ ... }: {".to_string(),
        "            home.username = \"validator\";".to_string(),
        format!(
            "            home.homeDirectory = \"{}\";",
            home_root_literal
        ),
        "            home.stateVersion = \"24.11\";".to_string(),
        "            programs.home-manager.enable = true;".to_string(),
        "            programs.yazelix.enable = true;".to_string(),
        if manage_config {
            "            programs.yazelix.manage_config = true;".to_string()
        } else {
            "            # manage_config=false default is intentionally exercised here.".to_string()
        },
        "          })".to_string(),
        "        ];".to_string(),
        "      };".to_string(),
        "    };".to_string(),
        "}".to_string(),
    ]
    .join("\n")
}

fn build_flake_interface_expr(repo_root: &Path) -> String {
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{}\";", repo_root_literal),
        "  system = builtins.currentSystem;".to_string(),
        "in".to_string(),
        "  builtins.hasAttr \"packages\" flake &&".to_string(),
        "  builtins.hasAttr system flake.packages &&".to_string(),
        "  builtins.hasAttr \"default\" flake.packages.${system} &&".to_string(),
        "  builtins.hasAttr \"runtime\" flake.packages.${system} &&".to_string(),
        "  builtins.hasAttr \"yazelix\" flake.packages.${system} &&".to_string(),
        "  !builtins.hasAttr \"install\" flake.packages.${system} &&".to_string(),
        "  (flake.packages.${system}.default.name or \"\") == (flake.packages.${system}.yazelix.name or \"\") &&"
            .to_string(),
        "  (flake.packages.${system}.default.name or \"\") != \"yazelix-runtime\" &&".to_string(),
        "  builtins.hasAttr \"apps\" flake &&".to_string(),
        "  builtins.hasAttr system flake.apps &&".to_string(),
        "  builtins.hasAttr \"default\" flake.apps.${system} &&".to_string(),
        "  builtins.hasAttr \"yazelix\" flake.apps.${system} &&".to_string(),
        "  !builtins.hasAttr \"install\" flake.apps.${system} &&".to_string(),
        "  builtins.hasAttr \"homeManagerModules\" flake &&".to_string(),
        "  builtins.hasAttr \"default\" flake.homeManagerModules &&".to_string(),
        "  builtins.hasAttr \"yazelix\" flake.homeManagerModules &&".to_string(),
        "  builtins.isFunction flake.homeManagerModules.default &&".to_string(),
        "  builtins.isFunction flake.homeManagerModules.yazelix".to_string(),
    ]
    .join("\n")
}

fn build_flake_package_platform_expr(repo_root: &Path) -> String {
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{}\";", repo_root_literal),
        "  lib = flake.inputs.nixpkgs.lib;".to_string(),
        "  systems = builtins.attrNames flake.packages;".to_string(),
        "  check = system:".to_string(),
        "    let".to_string(),
        "      pkg = flake.packages.${system}.yazelix;".to_string(),
        "      platformEntry = lib.systems.elaborate { inherit system; };".to_string(),
        "    in {".to_string(),
        "      inherit system;".to_string(),
        "      available = lib.meta.availableOn platformEntry pkg;".to_string(),
        "      platforms = pkg.meta.platforms or [];".to_string(),
        "    };".to_string(),
        "in".to_string(),
        "  builtins.map check systems".to_string(),
    ]
    .join("\n")
}

fn collect_nushell_syntax_files(repo_root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for relative_dir in [
        "nushell/scripts/core",
        "nushell/scripts/integrations",
        "nushell/scripts/setup",
        "nushell/scripts/utils",
        "nushell/scripts/dev",
        "nushell/scripts/dev/sweep",
        "nushell/config",
    ] {
        let dir = repo_root.join(relative_dir);
        if !dir.exists() {
            continue;
        }
        collect_nushell_files_in_dir(&dir, &mut files)?;
    }
    files.sort();
    Ok(files)
}

fn collect_nushell_files_in_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in
        fs::read_dir(dir).map_err(|error| format!("Failed to read {}: {}", dir.display(), error))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("nu") {
            files.push(path);
        }
    }
    Ok(())
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
        "wezterm" => "wezterm",
        _ => "ghostty",
    };
    let runtime_yzx_cli = runtime_root.join("shells").join("posix").join("yzx_cli.sh");
    let runtime_yzx_core = runtime_libexec.join("yzx_core");
    let runtime_ghostty_wrapper = runtime_root
        .join("shells")
        .join("posix")
        .join("yazelix_ghostty.sh");
    let runtime_yazelix_default = runtime_root.join("yazelix_default.toml");
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
            runtime_yazelix_default.clone(),
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

    for expected_tool in [
        "zellij",
        runtime_terminal,
        "yazi",
        "hx",
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
        "nu", "zellij", "yazi", "hx", "nvim", "bash", "jq", "fd", "rg",
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
    let selected_terminal_json = format!("[\"{runtime_terminal}\"]");
    let materialization_result = run_installed_yzx(
        repo_root,
        temp_home,
        &[
            "run",
            &yzx_core_arg,
            "launch-materialization.prepare",
            "--from-env",
            "--selected-terminals-json",
            &selected_terminal_json,
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
    } else {
        require_path_exists_abs(
            &generated_wezterm_config,
            "generated WezTerm config for selected runtime variant",
            errors,
        );
    }
    Ok(())
}

fn require_path_exists(
    repo_root: &Path,
    relative_path: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    if !repo_root.join(relative_path).exists() {
        errors.push(format!("Missing {}: {}", label, relative_path));
    }
}

fn require_path_missing(
    repo_root: &Path,
    relative_path: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    if repo_root.join(relative_path).exists() {
        errors.push(format!("Unexpected {}: {}", label, relative_path));
    }
}

fn require_path_missing_abs(path: &Path, label: &str, errors: &mut Vec<String>) {
    if path.exists() {
        errors.push(format!("Unexpected {}: {}", label, path.display()));
    }
}

fn require_path_exists_abs(path: &Path, label: &str, errors: &mut Vec<String>) {
    if !path.exists() {
        errors.push(format!("Missing {}: {}", label, path.display()));
    }
}

fn require_non_empty_dir_abs(
    path: &Path,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    require_path_exists_abs(path, label, errors);
    if !path.exists() {
        return Ok(());
    }
    let has_file = fs::read_dir(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?
        .filter_map(Result::ok)
        .any(|entry| entry.path().is_file());
    if !has_file {
        errors.push(format!("{} is empty: {}", label, path.display()));
    }
    Ok(())
}

fn require_file_contains(
    repo_root: &Path,
    relative_path: &str,
    needle: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    require_file_contains_abs(&repo_root.join(relative_path), needle, label, errors)
}

fn require_file_contains_abs(
    path: &Path,
    needle: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    if !content.contains(needle) {
        errors.push(format!(
            "{} does not contain expected text `{}`: {}",
            label,
            needle,
            path.display()
        ));
    }
    Ok(())
}

fn require_file_not_contains(
    repo_root: &Path,
    relative_path: &str,
    needle: &str,
    label: &str,
    errors: &mut Vec<String>,
) -> Result<(), String> {
    let path = repo_root.join(relative_path);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    if content.contains(needle) {
        errors.push(format!(
            "{} still contains forbidden text `{}`: {}",
            label, needle, relative_path
        ));
    }
    Ok(())
}

fn require_list_contains(items: &[String], expected: &str, label: &str, errors: &mut Vec<String>) {
    if !items.iter().any(|item| item == expected) {
        errors.push(format!(
            "{} is missing expected entry `{}`. Found: {}",
            label,
            expected,
            items.join(", ")
        ));
    }
}

fn require_list_not_contains(
    items: &[String],
    forbidden: &str,
    label: &str,
    errors: &mut Vec<String>,
) {
    if items.iter().any(|item| item == forbidden) {
        errors.push(format!(
            "{} unexpectedly contains forbidden entry `{}`. Found: {}",
            label,
            forbidden,
            items.join(", ")
        ));
    }
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

fn run_repo_command(repo_root: &Path, program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|error| {
            format!(
                "Failed to run `{}` for installed-runtime validation: {}",
                format_command(program, args),
                error
            )
        })
}

fn build_flake_output_path(repo_root: &Path, attr: &str, label: &str) -> Result<PathBuf, String> {
    let output = run_repo_command(
        repo_root,
        "nix",
        &[
            "build",
            "--no-link",
            "--print-out-paths",
            &format!(".#{attr}"),
        ],
    )?;
    if !output.status.success() {
        return Err(format!(
            "Failed while {}\n{}",
            label,
            command_output_summary(&output)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(path) = stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
    else {
        return Err(format!("{} did not return an output path", label));
    };
    if !path.exists() {
        return Err(format!(
            "{} returned missing output path {}",
            label,
            path.display()
        ));
    }
    Ok(path)
}

fn build_nix_file_output_path(
    repo_root: &Path,
    relative_file: PathBuf,
    label: &str,
) -> Result<PathBuf, String> {
    let output = Command::new("nix")
        .args([
            "build",
            "--no-link",
            "--print-out-paths",
            "--extra-experimental-features",
            "nix-command flakes",
            "--file",
        ])
        .arg(repo_root.join(&relative_file))
        .current_dir(repo_root)
        .output()
        .map_err(|error| {
            format!(
                "Failed to run nix build for {}: {}",
                relative_file.display(),
                error
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "Failed while {}\n{}",
            label,
            command_output_summary(&output)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(path) = stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
    else {
        return Err(format!("{} did not return an output path", label));
    };
    if !path.exists() {
        return Err(format!(
            "{} returned missing output path {}",
            label,
            path.display()
        ));
    }
    Ok(path)
}

fn validate_rust_routed_nu_modules(runtime_root: &Path, label: &str, errors: &mut Vec<String>) {
    let scripts_dir = runtime_root.join("nushell").join("scripts");
    for relative_path in [["yzx", "menu.nu"]] {
        let path = scripts_dir.join(relative_path.iter().collect::<PathBuf>());
        if !path.exists() {
            errors.push(format!(
                "Missing {} Rust-routed Nu module: {}",
                label,
                path.display()
            ));
        }
    }
}

fn json_object_keys(value: &JsonValue) -> Vec<String> {
    let mut keys = value
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    keys.sort();
    keys
}

fn command_stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_string()
}

fn command_output_summary(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = command_stderr(output);
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => "No subprocess output captured".to_string(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("Stdout:\n{}\nStderr:\n{}", stdout, stderr),
    }
}

fn format_command(program: &str, args: &[&str]) -> String {
    std::iter::once(program)
        .chain(args.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}

fn build_nix_customization_api_expr(repo_root: &Path) -> String {
    let flake_ref = format!(
        "path:{}",
        escape_nix_string(&repo_root.display().to_string())
    );
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{}\";", flake_ref),
        "  system = \"x86_64-linux\";".to_string(),
        "  pkgs = import flake.inputs.nixpkgs { inherit system; };".to_string(),
        "  defaultPackage = flake.packages.${system}.yazelix;".to_string(),
        "  mkDefaultPackage = flake.lib.${system}.mkYazelix {};".to_string(),
        "  overlayPkgs = import flake.inputs.nixpkgs { inherit system; overlays = [ flake.overlays.default ]; };".to_string(),
        "  hm = flake.inputs.home-manager.lib.homeManagerConfiguration {".to_string(),
        "    inherit pkgs;".to_string(),
        "    modules = [".to_string(),
        "      flake.homeManagerModules.yazelix".to_string(),
        "      {".to_string(),
        "        home.username = \"validator\";".to_string(),
        "        home.homeDirectory = \"/home/validator\";".to_string(),
        "        home.stateVersion = \"24.11\";".to_string(),
        "        programs.yazelix.enable = true;".to_string(),
        "        programs.yazelix.runtime_tool_sources.helix = \"host\";".to_string(),
        "      }".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        "  invalidRuntimeTool = builtins.tryEval ((flake.lib.${system}.mkYazelix { runtimeToolSources = { zellij = \"host\"; }; }).drvPath);".to_string(),
        "  invalidComponent = builtins.tryEval ((flake.lib.${system}.mkYazelix { components = { screen = false; }; }).drvPath);".to_string(),
        "in {".to_string(),
        "  has_mk_yazelix = builtins.hasAttr \"mkYazelix\" flake.lib.${system};".to_string(),
        "  default_main_program = defaultPackage.meta.mainProgram or \"\";".to_string(),
        "  mk_default_main_program = mkDefaultPackage.meta.mainProgram or \"\";".to_string(),
        "  overlay_main_program = overlayPkgs.yazelix.meta.mainProgram or \"\";".to_string(),
        "  home_manager_runtime_tool_source = hm.config.programs.yazelix.runtime_tool_sources.helix or \"\";".to_string(),
        "  home_manager_has_package = builtins.length hm.config.home.packages > 0;".to_string(),
        "  invalid_runtime_tool_rejected = !invalidRuntimeTool.success;".to_string(),
        "  invalid_component_rejected = !invalidComponent.success;".to_string(),
        "}".to_string(),
    ]
    .join("\n")
}

fn run_nix_eval(repo_root: &Path, expr: &str) -> Result<JsonValue, String> {
    let output = Command::new("nix")
        .args(["eval", "--impure", "--json", "--expr", expr])
        .current_dir(repo_root)
        .output()
        .map_err(|error| format!("Failed to run `nix eval`: {}", error))?;
    if !output.status.success() {
        return Err(format!(
            "Failed to evaluate Nix expression for validator.\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice::<JsonValue>(&output.stdout)
        .map_err(|error| format!("Failed to parse `nix eval` JSON output: {}", error))
}

fn create_unique_temp_dir(prefix: &str) -> Result<PathBuf, String> {
    let base = env::temp_dir();
    for attempt in 0..100u32 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("System clock error: {}", error))?
            .as_nanos();
        let candidate = base.join(format!(
            "{}_{}_{}_{}",
            prefix,
            process::id(),
            nanos,
            attempt
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!(
                    "Failed to create temporary directory {}: {}",
                    candidate.display(),
                    error
                ));
            }
        }
    }
    Err(format!(
        "Failed to create unique temporary directory for {}",
        prefix
    ))
}

fn prepare_temp_home(temp_home: &Path) -> Result<(), String> {
    if let Some(parent) = temp_home.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {}", parent.display(), error))?;
    }
    if temp_home.exists() {
        fs::remove_dir_all(temp_home)
            .map_err(|error| format!("Failed to remove {}: {}", temp_home.display(), error))?;
    }
    Ok(())
}

fn relative_display(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .display()
        .to_string()
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

fn read_toml_file(path: &Path) -> Result<TomlTable, String> {
    let raw = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read {}: {}", path.display(), error))?;
    parse_toml_from_str(&raw, &path.display().to_string())
}

fn parse_toml_from_str(raw: &str, label: &str) -> Result<TomlTable, String> {
    toml::from_str::<TomlTable>(raw)
        .map_err(|error| format!("Failed to parse {} as TOML: {}", label, error))
}

fn split_field_path(path: &str) -> Vec<&str> {
    path.split('.').collect()
}

fn get_nested_toml_value<'a>(table: &'a TomlTable, path: &[&str]) -> Option<&'a TomlValue> {
    if path.is_empty() {
        return None;
    }
    let mut current = table.get(path[0])?;
    for segment in &path[1..] {
        current = current.as_table()?.get(*segment)?;
    }
    Some(current)
}

fn set_nested_toml_value(table: &mut TomlTable, path: &[&str], value: TomlValue) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        table.insert(path[0].to_string(), value);
        return;
    }
    let entry = table
        .entry(path[0].to_string())
        .or_insert_with(|| TomlValue::Table(TomlTable::new()));
    if !entry.is_table() {
        *entry = TomlValue::Table(TomlTable::new());
    }
    if let Some(child) = entry.as_table_mut() {
        set_nested_toml_value(child, &path[1..], value);
    }
}

fn toml_to_json(value: &TomlValue) -> JsonValue {
    match value {
        TomlValue::String(value) => JsonValue::String(value.clone()),
        TomlValue::Integer(value) => JsonValue::Number((*value).into()),
        TomlValue::Float(value) => JsonNumber::from_f64(*value)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        TomlValue::Boolean(value) => JsonValue::Bool(*value),
        TomlValue::Datetime(value) => JsonValue::String(value.to_string()),
        TomlValue::Array(values) => JsonValue::Array(values.iter().map(toml_to_json).collect()),
        TomlValue::Table(table) => JsonValue::Object(
            table
                .iter()
                .map(|(key, value)| (key.clone(), toml_to_json(value)))
                .collect(),
        ),
    }
}

fn json_values_equal(left: &JsonValue, right: &JsonValue) -> bool {
    match (left, right) {
        (JsonValue::Null, JsonValue::Null) => true,
        (JsonValue::Bool(left), JsonValue::Bool(right)) => left == right,
        (JsonValue::Number(left), JsonValue::Number(right)) => left
            .as_f64()
            .zip(right.as_f64())
            .map(|(l, r)| l == r)
            .unwrap_or(false),
        (JsonValue::String(left), JsonValue::String(right)) => left == right,
        (JsonValue::Array(left), JsonValue::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| json_values_equal(left, right))
        }
        (JsonValue::Object(left), JsonValue::Object(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, left_value)| {
                    right
                        .get(key)
                        .map(|right_value| json_values_equal(left_value, right_value))
                        .unwrap_or(false)
                })
        }
        _ => false,
    }
}

fn toml_values_equal(left: &TomlValue, right: &TomlValue) -> bool {
    match (left, right) {
        (TomlValue::String(left), TomlValue::String(right)) => left == right,
        (TomlValue::Integer(left), TomlValue::Integer(right)) => left == right,
        (TomlValue::Float(left), TomlValue::Float(right)) => left == right,
        (TomlValue::Integer(left), TomlValue::Float(right))
        | (TomlValue::Float(right), TomlValue::Integer(left)) => (*left as f64) == *right,
        (TomlValue::Boolean(left), TomlValue::Boolean(right)) => left == right,
        (TomlValue::Datetime(left), TomlValue::Datetime(right)) => left == right,
        (TomlValue::Array(left), TomlValue::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| toml_values_equal(left, right))
        }
        (TomlValue::Table(left), TomlValue::Table(right)) => {
            left.len() == right.len()
                && left.iter().all(|(key, left_value)| {
                    right
                        .get(key)
                        .map(|right_value| toml_values_equal(left_value, right_value))
                        .unwrap_or(false)
                })
        }
        _ => false,
    }
}

fn format_json_value(value: &JsonValue) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "<unrenderable json>".to_string())
}

fn format_toml_value(value: &TomlValue) -> String {
    format_json_value(&toml_to_json(value))
}

fn as_string_list(value: Option<&TomlValue>) -> Vec<String> {
    value
        .and_then(TomlValue::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(TomlValue::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn sorted_keys(table: &TomlTable) -> Vec<String> {
    let mut keys = table.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

fn escape_nix_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
