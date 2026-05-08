use super::{
    build_flake_output_path, command_output_summary, create_unique_temp_dir, escape_nix_string,
    require_file_not_contains, require_list_contains, require_list_not_contains,
    require_path_exists, require_path_exists_abs, require_path_missing, run_nix_eval,
    run_repo_command, validate_rust_routed_nu_modules,
};
use crate::repo_validation::ValidationReport;
use serde_json::Value as JsonValue;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn validate_installed_runtime_contract(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    report
        .errors
        .extend(validate_installed_runtime_contract_inner(repo_root)?);
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

    let flake_surface = run_nix_eval(
        repo_root,
        &build_installed_runtime_flake_surface_expr(repo_root),
    )?;
    let package_keys = json_string_array(
        flake_surface.get("packageKeys"),
        "current-system package outputs",
    )?;
    for expected in ["default", "runtime", "yazelix"] {
        require_list_contains(
            &package_keys,
            expected,
            "current-system package outputs",
            &mut errors,
        );
    }
    for forbidden in ["install", "locked_devenv"] {
        require_list_not_contains(
            &package_keys,
            forbidden,
            "current-system package outputs",
            &mut errors,
        );
    }
    let app_keys = json_string_array(flake_surface.get("appKeys"), "current-system app outputs")?;
    for expected in ["default", "yazelix"] {
        require_list_contains(
            &app_keys,
            expected,
            "current-system app outputs",
            &mut errors,
        );
    }
    require_list_not_contains(
        &app_keys,
        "install",
        "current-system app outputs",
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
    let smoke_stdout = String::from_utf8_lossy(&smoke_result.stdout);
    if !smoke_stdout.contains("Yazelix is a reproducible terminal IDE") {
        errors.push(
            "Built yazelix public CLI smoke check returned unexpected output for `yzx why`"
                .to_string(),
        );
    }

    Ok(errors)
}

fn build_installed_runtime_flake_surface_expr(repo_root: &Path) -> String {
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{}\";", repo_root_literal),
        "  system = builtins.currentSystem;".to_string(),
        "in {".to_string(),
        "  packageKeys = builtins.attrNames (flake.packages.${system} or {});".to_string(),
        "  appKeys = builtins.attrNames (flake.apps.${system} or {});".to_string(),
        "}".to_string(),
    ]
    .join("\n")
}

fn json_string_array(value: Option<&JsonValue>, label: &str) -> Result<Vec<String>, String> {
    let Some(items) = value.and_then(JsonValue::as_array) else {
        return Err(format!(
            "Installed-runtime flake surface validation did not return an array for {label}"
        ));
    };
    let mut strings = Vec::with_capacity(items.len());
    for item in items {
        let Some(value) = item.as_str() else {
            return Err(format!(
                "Installed-runtime flake surface validation returned a non-string entry for {label}"
            ));
        };
        strings.push(value.to_string());
    }
    strings.sort();
    Ok(strings)
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
