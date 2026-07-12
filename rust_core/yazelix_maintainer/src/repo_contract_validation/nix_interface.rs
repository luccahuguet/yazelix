use super::{escape_nix_string, run_nix_eval};
use crate::repo_validation::ValidationReport;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::Path;

pub fn validate_flake_interface(repo_root: &Path) -> Result<ValidationReport, String> {
    let mut report = ValidationReport::default();
    let exact = run_nix_eval(repo_root, &build_flake_interface_expr(repo_root))?
        .as_bool()
        .ok_or("Top-level flake interface validation did not return a boolean")?;
    if !exact {
        report.errors.push(
            "The flake product API must expose only default/yazelix packages and apps plus homeManagerModules.default on all four supported systems; checks and devShells remain maintainer-only outputs"
                .to_string(),
        );
    }

    let rows = run_nix_eval(repo_root, &build_flake_package_platform_expr(repo_root))?;
    let rows = rows
        .as_array()
        .ok_or("First-party flake package platform validation did not return a JSON array")?;
    let unavailable = rows
        .iter()
        .filter(|row| row.get("available").and_then(JsonValue::as_bool) != Some(true))
        .filter_map(|row| row.get("system").and_then(JsonValue::as_str))
        .collect::<Vec<_>>();
    if !unavailable.is_empty() {
        report.errors.push(format!(
            "The Yazelix package is unavailable on exported systems: {}",
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

    for (key, message) in [
        (
            "default_main_program",
            "The complete Yazelix package must expose yzx as its main program",
        ),
        (
            "default_package_allows_substitutes",
            "The complete Yazelix package must allow published substitutes",
        ),
        (
            "default_package_does_not_prefer_local_build",
            "The complete Yazelix package must not prefer local builds over substitutes",
        ),
        (
            "home_manager_uses_default_package",
            "Home Manager must install exactly one copy of the complete default Yazelix package",
        ),
        (
            "home_manager_package_override",
            "Home Manager programs.yazelix.package must install exactly one copy of the selected complete package",
        ),
        (
            "home_manager_default_has_no_config",
            "Enabling Home Manager without config declarations must not create Yazelix config files",
        ),
    ] {
        require_json_bool(object, key, message, &mut report.errors);
    }

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

fn build_flake_interface_expr(repo_root: &Path) -> String {
    let flake_ref = escape_nix_string(&format!("git+file://{}", repo_root.display()));
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{flake_ref}\";"),
        "  systems = [ \"aarch64-darwin\" \"aarch64-linux\" \"x86_64-darwin\" \"x86_64-linux\" ];".to_string(),
        "  packageSystems = builtins.attrNames flake.packages;".to_string(),
        "  appSystems = builtins.attrNames flake.apps;".to_string(),
        "  packageOk = system: let packages = flake.packages.${system}; in".to_string(),
        "    builtins.attrNames packages == [ \"default\" \"yazelix\" ] &&".to_string(),
        "    packages.default.outPath == packages.yazelix.outPath &&".to_string(),
        "    (packages.yazelix.meta.mainProgram or \"\") == \"yzx\";".to_string(),
        "  appOk = system: let apps = flake.apps.${system}; package = flake.packages.${system}.yazelix; in".to_string(),
        "    builtins.attrNames apps == [ \"default\" \"yazelix\" ] &&".to_string(),
        "    (apps.default.type or \"\") == \"app\" &&".to_string(),
        "    (apps.yazelix.type or \"\") == \"app\" &&".to_string(),
        "    (apps.default.program or \"\") == \"${package}/bin/yzx\" &&".to_string(),
        "    (apps.yazelix.program or \"\") == \"${package}/bin/yzx\";".to_string(),
        "in".to_string(),
        "  packageSystems == systems &&".to_string(),
        "  appSystems == systems &&".to_string(),
        "  builtins.all packageOk systems &&".to_string(),
        "  builtins.all appOk systems &&".to_string(),
        "  builtins.attrNames flake.homeManagerModules == [ \"default\" ] &&".to_string(),
        "  builtins.isFunction flake.homeManagerModules.default &&".to_string(),
        "  !(builtins.hasAttr \"lib\" flake) &&".to_string(),
        "  !(builtins.hasAttr \"overlays\" flake)".to_string(),
    ]
    .join("\n")
}

fn build_flake_package_platform_expr(repo_root: &Path) -> String {
    let flake_ref = escape_nix_string(&format!("git+file://{}", repo_root.display()));
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{flake_ref}\";"),
        "  lib = flake.inputs.nixpkgs.lib;".to_string(),
        "  check = system: let".to_string(),
        "    package = flake.packages.${system}.yazelix;".to_string(),
        "    platform = lib.systems.elaborate { inherit system; };".to_string(),
        "  in { inherit system; available = lib.meta.availableOn platform package; };".to_string(),
        "in builtins.map check (builtins.attrNames flake.packages)".to_string(),
    ]
    .join("\n")
}

fn build_nix_customization_api_expr(repo_root: &Path) -> String {
    let flake_ref = escape_nix_string(&format!("git+file://{}", repo_root.display()));
    [
        "let".to_string(),
        format!("  flake = builtins.getFlake \"{flake_ref}\";"),
        "  system = \"x86_64-linux\";".to_string(),
        "  pkgs = flake.inputs.nixpkgs.legacyPackages.${system};".to_string(),
        "  defaultPackage = flake.packages.${system}.yazelix;".to_string(),
        "  customPackage = pkgs.runCommand \"custom-yazelix\" { meta.mainProgram = \"yzx\"; } \"mkdir -p $out/bin; touch $out/bin/yzx\";".to_string(),
        "  home = package: flake.inputs.home-manager.lib.homeManagerConfiguration {".to_string(),
        "    inherit pkgs;".to_string(),
        "    modules = [ flake.homeManagerModules.default {".to_string(),
        "      home.username = \"validator\";".to_string(),
        "      home.homeDirectory = \"/home/validator\";".to_string(),
        "      home.stateVersion = \"24.11\";".to_string(),
        "      programs.yazelix.enable = true;".to_string(),
        "      programs.yazelix.package = package;".to_string(),
        "    } ];".to_string(),
        "  };".to_string(),
        "  defaultHome = home defaultPackage;".to_string(),
        "  customHome = home customPackage;".to_string(),
        "  packageCount = package: home: builtins.length (builtins.filter (candidate: candidate.outPath == package.outPath) home.config.home.packages);".to_string(),
        "in {".to_string(),
        "  default_main_program = (defaultPackage.meta.mainProgram or \"\") == \"yzx\";".to_string(),
        "  default_package_allows_substitutes = (defaultPackage.allowSubstitutes or true) == true;".to_string(),
        "  default_package_does_not_prefer_local_build = (defaultPackage.preferLocalBuild or false) == false;".to_string(),
        "  home_manager_uses_default_package = packageCount defaultPackage defaultHome == 1;".to_string(),
        "  home_manager_package_override = packageCount customPackage customHome == 1 && packageCount defaultPackage customHome == 0;".to_string(),
        "  home_manager_default_has_no_config = !(builtins.hasAttr \"yazelix/config.toml\" defaultHome.config.xdg.configFile);".to_string(),
        "}".to_string(),
    ]
    .join("\n")
}
