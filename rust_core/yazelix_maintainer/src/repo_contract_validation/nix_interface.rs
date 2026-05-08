use super::{escape_nix_string, format_json_value, run_nix_eval};
use crate::repo_validation::ValidationReport;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::Path;

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
