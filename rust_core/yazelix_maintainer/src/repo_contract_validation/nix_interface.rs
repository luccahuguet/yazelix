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
    require_json_bool(
        object,
        "default_package_allows_substitutes",
        "default flake package must allow substitutes so published Cachix paths can be used",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "default_package_does_not_prefer_local_build",
        "default flake package must not prefer local builds over published substitutes",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "mk_default_package_allows_substitutes",
        "lib.<system>.mkYazelix default package must allow substitutes so published Cachix paths can be used",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "mk_default_package_does_not_prefer_local_build",
        "lib.<system>.mkYazelix default package must not prefer local builds over published substitutes",
        &mut report.errors,
    );
    require_json_string(
        object,
        "home_manager_runtime_tool_source",
        "host",
        "Home Manager runtime_tool_sources must pass typed host values through evaluation",
        &mut report.errors,
    );
    require_json_string(
        object,
        "home_manager_steel_tool_source",
        "off",
        "Home Manager runtime_tool_sources must pass typed Steel off values through evaluation",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "steel_bundled_exports_authoring_commands",
        "bundled Steel runtime tools must export Steel authoring commands",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "steel_off_omits_authoring_commands",
        "runtimeToolSources.steel = off must omit Steel authoring commands from exports",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "mise_defaults_to_host",
        "runtimeToolSources.mise must default to host mode",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "tombi_defaults_to_host",
        "runtimeToolSources.tombi must default to host mode",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "host_default_tools_not_exported",
        "default host-sourced mise and tombi commands must not be exported from the runtime",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "host_default_tools_can_be_bundled",
        "mise and tombi must remain explicitly bundlable through runtimeToolSources",
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
        "home_manager_terminal_option_selects_yzxterm",
        "Home Manager programs.yazelix.terminal must select the packaged terminal variant",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "home_manager_terminal_option_omits_fallback_terminal_packages",
        "Home Manager terminal selection must not install additional terminal fallback packages",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "home_manager_yzxterm_package_override_option",
        "Home Manager yzxterm_package must accept a terminal child package without replacing the whole Yazelix package",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "yzxterm_package_override_is_yzxterm_scoped",
        "yzxterm package overrides must not affect non-yzxterm terminal variants",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "yzxterm_package_override_uses_package_metadata",
        "yzxterm package overrides must derive the terminal command from child package metadata",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "yzxterm_package_override_rejects_missing_metadata",
        "yzxterm package overrides must reject packages without marsPackageMetadata",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "rio_runtime_uses_configured_upstream_package",
        "Rio runtime must use the configured upstream Rio package and must not depend on Mars metadata",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "yzxterm_fast_child_metadata_marks_unchecked",
        "yzxterm fast child package metadata must mark the package as fast and unchecked",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "yzxterm_release_child_metadata_marks_checked",
        "yzxterm release child package metadata must mark the package as release and checked",
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
        "unsupported_component_rejected",
        "unsupported component toggles must fail during Nix evaluation",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "kgp_zellij_owns_cargo_deps",
        "KGP Zellij package must own source-coupled Cargo vendor deps instead of inheriting consumer pkgs.zellij cargoDeps",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "kgp_zellij_clears_consumer_patches",
        "KGP Zellij package must clear consumer pkgs.zellij patch hooks when it swaps to the KGP source",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "kgp_zellij_owns_install_check",
        "KGP Zellij package must own install checks coupled to its patch policy",
        &mut report.errors,
    );
    require_json_bool(
        object,
        "kgp_zellij_uses_unwrapped_package_when_wrapper_lacks_passthru",
        "KGP Zellij package must fall back to pkgs.zellij-unwrapped when pkgs.zellij is a wrapper without passthru.unwrapped",
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
    let repo_root_literal = escape_nix_string(&repo_root.display().to_string());
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
        "        programs.yazelix.runtime_tool_sources.steel = \"off\";".to_string(),
        "      }".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        "  hmYzxterm = flake.inputs.home-manager.lib.homeManagerConfiguration {".to_string(),
        "    inherit pkgs;".to_string(),
        "    modules = [".to_string(),
        "      flake.homeManagerModules.yazelix".to_string(),
        "      {".to_string(),
        "        home.username = \"validator\";".to_string(),
        "        home.homeDirectory = \"/home/validator\";".to_string(),
        "        home.stateVersion = \"24.11\";".to_string(),
        "        programs.yazelix.enable = true;".to_string(),
        "        programs.yazelix.manage_config = true;".to_string(),
        "        programs.yazelix.terminal = \"yzxterm\";".to_string(),
        "      }".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        r#"  fakeYzxtermPackage = pkgs.runCommand "validator-mars-fast" {
    passthru.marsPackageMetadata = {
      schema_version = 1;
      terminal = "mars";
      package_name = "validator-mars-fast";
      package_profile = "fast";
      checked_package = false;
      metadata_path = "share/mars/package-metadata.json";
      config_roots = {
        full = "share/mars";
        baseline = "share/mars/baseline";
        shaders = "share/mars/profiles/shaders";
      };
      supported_emoji_fonts = [ "noto" "twitter" "serenityos" ];
      default_emoji_font = "noto";
      supported_appearance_modes = [ "dark" "light" "auto" ];
      default_appearance_mode = "dark";
      emoji_fonts = {
        noto = { family = "Noto Color Emoji"; config_roots = { full = "share/mars"; baseline = "share/mars/baseline"; shaders = "share/mars/profiles/shaders"; }; };
        twitter = { family = "Twitter Color Emoji"; config_roots = { full = "share/mars/emoji/twitter"; baseline = "share/mars/emoji/twitter/baseline"; shaders = "share/mars/emoji/twitter/profiles/shaders"; }; };
        serenityos = { family = "SerenityOS Emoji"; config_roots = { full = "share/mars/emoji/serenityos"; baseline = "share/mars/emoji/serenityos/baseline"; shaders = "share/mars/emoji/serenityos/profiles/shaders"; }; };
      };
      wrapper_commands = {
        terminal = "bin/validator-mars";
        desktop = "bin/validator-mars-desktop";
      };
      wrapper_env = {
        appearance = "MARS_APPEARANCE";
        emoji_font = "MARS_EMOJI_FONT";
      };
    };
  } ''
    mkdir -p "$out/bin" "$out/share/mars"
    touch "$out/bin/validator-mars-desktop"
    chmod +x "$out/bin/validator-mars-desktop"
    printf '{}' > "$out/share/mars/package-metadata.json"
  '';
  invalidYzxtermPackage = pkgs.runCommand "validator-mars-invalid" { } ''
    mkdir -p "$out"
  '';
  fakeRioPackage = pkgs.runCommand "validator-rio-0.4.7" { } ''
    mkdir -p "$out/bin"
    touch "$out/bin/rio"
    chmod +x "$out/bin/rio"
  '';
  hmYzxtermPackageOverride = flake.inputs.home-manager.lib.homeManagerConfiguration {
    inherit pkgs;
    modules = [
      flake.homeManagerModules.yazelix
      {
        home.username = "validator";
        home.homeDirectory = "/home/validator";
        home.stateVersion = "24.11";
        programs.yazelix.enable = true;
        programs.yazelix.terminal = "yzxterm";
        programs.yazelix.yzxterm_package = fakeYzxtermPackage;
      }
    ];
  };"#
            .to_string(),
        format!(
            "  steelBundledRegistry = import \"{}/packaging/runtime_tool_registry.nix\" {{",
            repo_root_literal
        ),
        "    inherit pkgs;".to_string(),
        "  };".to_string(),
        format!(
            "  steelOffRegistry = import \"{}/packaging/runtime_tool_registry.nix\" {{",
            repo_root_literal
        ),
        "    inherit pkgs;".to_string(),
        "    runtimeToolSources = { steel = \"off\"; };".to_string(),
        "  };".to_string(),
        format!(
            "  hostDefaultToolsBundledRegistry = import \"{}/packaging/runtime_tool_registry.nix\" {{",
            repo_root_literal
        ),
        "    inherit pkgs;".to_string(),
        "    runtimeToolSources = { mise = \"bundled\"; tombi = \"bundled\"; };".to_string(),
        "  };".to_string(),
        format!(
            r#"  ghosttyRegistryWithInvalidYzxtermOverride = import "{}/packaging/runtime_tool_registry.nix" {{
    inherit pkgs;
    runtimeVariant = "ghostty";
    marsTerminalPackage = invalidYzxtermPackage;
  }};
  yzxtermOverrideRegistry = import "{}/packaging/runtime_tool_registry.nix" {{
    inherit pkgs;
    runtimeVariant = "yzxterm";
    marsTerminalPackage = fakeYzxtermPackage;
  }};
  invalidYzxtermPackageRegistry = builtins.tryEval (builtins.concatStringsSep "," ((import "{}/packaging/runtime_tool_registry.nix" {{
    inherit pkgs;
    runtimeVariant = "yzxterm";
    marsTerminalPackage = invalidYzxtermPackage;
  }}).exportedCommands));
  rioOverrideRegistry = import "{}/packaging/runtime_tool_registry.nix" {{
    inherit pkgs;
    runtimeVariant = "rio";
    rioPackage = fakeRioPackage;
  }};"#,
            repo_root_literal, repo_root_literal, repo_root_literal, repo_root_literal
        ),
        "  yzxtermFastChildMetadata = flake.inputs.marsTerminal.packages.${system}.mars-fast.passthru.marsPackageMetadata or {};".to_string(),
        "  yzxtermReleaseChildMetadata = flake.inputs.marsTerminal.packages.${system}.mars.passthru.marsPackageMetadata or {};".to_string(),
        "  steelAuthoringCommands = [ \"steel\" \"steel-language-server\" \"forge\" \"cargo-steel-lib\" \"repl-connect\" ];".to_string(),
        "  invalidRuntimeTool = builtins.tryEval ((flake.lib.${system}.mkYazelix { runtimeToolSources = { zellij = \"host\"; }; }).drvPath);".to_string(),
        "  unsupportedComponent = builtins.tryEval ((flake.lib.${system}.mkYazelix { components = { status_bar = false; }; }).drvPath);".to_string(),
        "  zellijBuildBase = pkgs: zellij: if zellij ? unwrapped then zellij.unwrapped else if builtins.hasAttr \"zellij-unwrapped\" pkgs then pkgs.\"zellij-unwrapped\" else zellij;".to_string(),
        "  poisonedConsumerPkgs = import flake.inputs.nixpkgs {".to_string(),
        "    inherit system;".to_string(),
        "    overlays = [".to_string(),
        "      (_final: prev: {".to_string(),
        "        zellij = prev.zellij.overrideAttrs (_old: {".to_string(),
        "          __intentionallyOverridingVersion = true;".to_string(),
        "          version = \"0.44.1\";".to_string(),
        "          cargoDeps = throw \"consumer pkgs.zellij cargoDeps leaked into Yazelix graphics runtime\";".to_string(),
        "          patches = throw \"consumer pkgs.zellij patches leaked into Yazelix graphics runtime\";".to_string(),
        "          prePatch = throw \"consumer pkgs.zellij prePatch leaked into Yazelix graphics runtime\";".to_string(),
        "          postPatch = throw \"consumer pkgs.zellij postPatch leaked into Yazelix graphics runtime\";".to_string(),
        "          installCheckPhase = throw \"consumer pkgs.zellij installCheckPhase leaked into Yazelix graphics runtime\";".to_string(),
        "        });".to_string(),
        "      } // (if builtins.hasAttr \"zellij-unwrapped\" prev then {".to_string(),
        "        zellij-unwrapped = prev.\"zellij-unwrapped\".overrideAttrs (_old: {".to_string(),
        "          cargoDeps = throw \"consumer pkgs.zellij-unwrapped cargoDeps leaked into Yazelix graphics runtime\";".to_string(),
        "          patches = throw \"consumer pkgs.zellij-unwrapped patches leaked into Yazelix graphics runtime\";".to_string(),
        "          prePatch = throw \"consumer pkgs.zellij-unwrapped prePatch leaked into Yazelix graphics runtime\";".to_string(),
        "          postPatch = throw \"consumer pkgs.zellij-unwrapped postPatch leaked into Yazelix graphics runtime\";".to_string(),
        "          installCheckPhase = throw \"consumer pkgs.zellij-unwrapped installCheckPhase leaked into Yazelix graphics runtime\";".to_string(),
        "        });".to_string(),
        "      } else { }))".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        "  wrappedNoPassthruConsumerPkgs = import flake.inputs.nixpkgs {".to_string(),
        "    inherit system;".to_string(),
        "    overlays = [".to_string(),
        "      (_final: prev:".to_string(),
        "        let".to_string(),
        "          fallbackUnwrapped = if builtins.hasAttr \"zellij-unwrapped\" prev then prev.\"zellij-unwrapped\" else prev.zellij;".to_string(),
        "        in {".to_string(),
        "        zellij = prev.zellij.overrideAttrs (old: {".to_string(),
        "          passthru = (builtins.removeAttrs (old.passthru or {}) [ \"unwrapped\" ]) // {".to_string(),
        "            __yazelix_test_base = \"wrapper\";".to_string(),
        "          };".to_string(),
        "        });".to_string(),
        "        zellij-unwrapped = fallbackUnwrapped.overrideAttrs (old: {".to_string(),
        "          passthru = (old.passthru or {}) // {".to_string(),
        "            __yazelix_test_base = \"zellij-unwrapped\";".to_string(),
        "          };".to_string(),
        "        });".to_string(),
        "      })".to_string(),
        "    ];".to_string(),
        "  };".to_string(),
        "  wrappedNoPassthruZellijBase = zellijBuildBase wrappedNoPassthruConsumerPkgs wrappedNoPassthruConsumerPkgs.zellij;".to_string(),
        format!(
            "  kgpZellij = import \"{}/packaging/yazelix_kgp_zellij.nix\" {{",
            repo_root_literal
        ),
        "    pkgs = poisonedConsumerPkgs;".to_string(),
        "    baseZellij = zellijBuildBase poisonedConsumerPkgs poisonedConsumerPkgs.zellij;".to_string(),
        "    src = flake.inputs.yazelixZellij;".to_string(),
        "  };".to_string(),
        format!(
            "  kgpZellijWrappedNoPassthru = import \"{}/packaging/yazelix_kgp_zellij.nix\" {{",
            repo_root_literal
        ),
        "    pkgs = wrappedNoPassthruConsumerPkgs;".to_string(),
        "    baseZellij = wrappedNoPassthruZellijBase;".to_string(),
        "    src = flake.inputs.yazelixZellij;".to_string(),
        "  };".to_string(),
        "in {".to_string(),
        "  has_mk_yazelix = builtins.hasAttr \"mkYazelix\" flake.lib.${system};".to_string(),
        "  default_main_program = defaultPackage.meta.mainProgram or \"\";".to_string(),
        "  mk_default_main_program = mkDefaultPackage.meta.mainProgram or \"\";".to_string(),
        "  overlay_main_program = overlayPkgs.yazelix.meta.mainProgram or \"\";".to_string(),
        "  default_package_allows_substitutes = (defaultPackage.allowSubstitutes or true) == true;".to_string(),
        "  default_package_does_not_prefer_local_build = (defaultPackage.preferLocalBuild or false) == false;".to_string(),
        "  mk_default_package_allows_substitutes = (mkDefaultPackage.allowSubstitutes or true) == true;".to_string(),
        "  mk_default_package_does_not_prefer_local_build = (mkDefaultPackage.preferLocalBuild or false) == false;".to_string(),
        "  home_manager_runtime_tool_source = hm.config.programs.yazelix.runtime_tool_sources.helix or \"\";".to_string(),
        "  home_manager_steel_tool_source = hm.config.programs.yazelix.runtime_tool_sources.steel or \"\";".to_string(),
        "  steel_bundled_exports_authoring_commands = builtins.all (command: builtins.elem command steelBundledRegistry.exportedCommands) steelAuthoringCommands;".to_string(),
        "  steel_off_omits_authoring_commands = steelOffRegistry.manifest.steel.source == \"off\" && builtins.all (command: !(builtins.elem command steelOffRegistry.exportedCommands)) steelAuthoringCommands;".to_string(),
        "  mise_defaults_to_host = steelBundledRegistry.manifest.mise.source == \"host\";".to_string(),
        "  tombi_defaults_to_host = steelBundledRegistry.manifest.tombi.source == \"host\";".to_string(),
        "  host_default_tools_not_exported = !(builtins.elem \"mise\" steelBundledRegistry.exportedCommands) && !(builtins.elem \"tombi\" steelBundledRegistry.exportedCommands);".to_string(),
        "  host_default_tools_can_be_bundled = hostDefaultToolsBundledRegistry.manifest.mise.source == \"bundled\" && hostDefaultToolsBundledRegistry.manifest.tombi.source == \"bundled\" && builtins.elem \"mise\" hostDefaultToolsBundledRegistry.exportedCommands && builtins.elem \"tombi\" hostDefaultToolsBundledRegistry.exportedCommands;".to_string(),
        "  home_manager_has_package = builtins.length hm.config.home.packages > 0;".to_string(),
        "  home_manager_terminal_option_selects_yzxterm = hmYzxterm.config.programs.yazelix.terminal == \"yzxterm\" && builtins.any (pkg: (pkg.meta.mainProgram or \"\") == \"yzx\") hmYzxterm.config.home.packages;".to_string(),
        "  home_manager_terminal_option_omits_fallback_terminal_packages = !(builtins.any (pkg: let name = pkg.name or \"\"; in pkgs.lib.hasPrefix \"ghostty-\" name || pkgs.lib.hasPrefix \"foot-\" name || pkgs.lib.hasPrefix \"kitty-\" name || pkgs.lib.hasPrefix \"rio-\" name || pkgs.lib.hasPrefix \"wezterm-\" name || pkgs.lib.hasPrefix \"ratty-\" name) hmYzxterm.config.home.packages);".to_string(),
        r#"  home_manager_yzxterm_package_override_option = hmYzxtermPackageOverride.config.programs.yazelix.yzxterm_package.passthru.marsPackageMetadata.package_name == "validator-mars-fast";
  yzxterm_package_override_is_yzxterm_scoped = ghosttyRegistryWithInvalidYzxtermOverride.manifest.terminal.commands == [ "ghostty" ];
  yzxterm_package_override_uses_package_metadata = yzxtermOverrideRegistry.terminalPackageMetadata.package_name == "validator-mars-fast" && builtins.elem "validator-mars-desktop" yzxtermOverrideRegistry.exportedCommands && yzxtermOverrideRegistry.terminalPackageRuntimeIdentity.package_profile == "yzxterm-fast" && yzxtermOverrideRegistry.terminalPackageRuntimeIdentity.yzxterm_terminal_supported_appearance_modes == [ "dark" "light" "auto" ] && yzxtermOverrideRegistry.terminalPackageRuntimeIdentity.yzxterm_terminal_default_appearance_mode == "dark";
  yzxterm_package_override_rejects_missing_metadata = !invalidYzxtermPackageRegistry.success;
  rio_runtime_uses_configured_upstream_package = rioOverrideRegistry.tools.terminal.package == fakeRioPackage && rioOverrideRegistry.tools.terminal.commands == [ "rio" ] && rioOverrideRegistry.terminalPackageMetadata == null;
  yzxterm_fast_child_metadata_marks_unchecked = (yzxtermFastChildMetadata.package_profile or "") == "fast" && (yzxtermFastChildMetadata.checked_package or true) == false && (yzxtermFastChildMetadata.supported_appearance_modes or []) == [ "dark" "light" "auto" ] && (yzxtermFastChildMetadata.default_appearance_mode or "") == "dark" && (yzxtermFastChildMetadata.wrapper_env.appearance or "") == "MARS_APPEARANCE";
  yzxterm_release_child_metadata_marks_checked = (yzxtermReleaseChildMetadata.package_profile or "") == "release" && (yzxtermReleaseChildMetadata.checked_package or false) == true && (yzxtermReleaseChildMetadata.supported_appearance_modes or []) == [ "dark" "light" "auto" ] && (yzxtermReleaseChildMetadata.default_appearance_mode or "") == "dark" && (yzxtermReleaseChildMetadata.wrapper_env.appearance or "") == "MARS_APPEARANCE";"#
            .to_string(),
        "  invalid_runtime_tool_rejected = !invalidRuntimeTool.success;".to_string(),
        "  unsupported_component_rejected = !unsupportedComponent.success;".to_string(),
        "  kgp_zellij_owns_cargo_deps = (kgpZellij.version or \"\") == \"0.44.3\" && (kgpZellij.cargoDeps.name or \"\") == \"zellij-0.44.3-vendor\";".to_string(),
        "  kgp_zellij_clears_consumer_patches = (kgpZellij.patches or []) == [] && (kgpZellij.prePatch or \"\") == \"\" && (kgpZellij.postPatch or \"\") == \"\";".to_string(),
        "  kgp_zellij_owns_install_check = (kgpZellij.installCheckPhase or \"\") == \"runHook preInstallCheck\\nrunHook postInstallCheck\\n\";".to_string(),
        "  kgp_zellij_uses_unwrapped_package_when_wrapper_lacks_passthru = (wrappedNoPassthruZellijBase.__yazelix_test_base or \"\") == \"zellij-unwrapped\" && (kgpZellijWrappedNoPassthru.version or \"\") == \"0.44.3\" && (kgpZellijWrappedNoPassthru.cargoDeps.name or \"\") == \"zellij-0.44.3-vendor\";".to_string(),
        "}".to_string(),
    ]
    .join("\n")
}
