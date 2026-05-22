{
  description = "Yazelix flake interface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixgl.url = "github:guibou/nixGL";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixScreen = {
      url = "github:luccahuguet/yazelix-screen";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
    yazelixGhosttyCursors = {
      url = "github:luccahuguet/yazelix-ghostty-cursors";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
    yazelixZellijBar = {
      url = "github:luccahuguet/yazelix-zellij-bar";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
      inputs.zjstatus.follows = "zjstatus";
    };
    yazelixYaziAssets = {
      url = "github:luccahuguet/yazelix-yazi-assets";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixZellij = {
      url = "github:luccahuguet/yazelix-zellij/yazelix-kgp-preview-0";
      flake = false;
    };
    yazelixYazi = {
      url = "github:luccahuguet/yazelix-yazi/yazelix-kgp-preview-0";
      flake = false;
    };
    yazelixZellijPaneOrchestrator = {
      url = "github:luccahuguet/yazelix-zellij-pane-orchestrator";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
    yazelixZellijPopup = {
      url = "github:luccahuguet/yazelix-zellij-popup";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
    beads = {
      url = "github:steveyegge/beads/v1.0.0";
    };
    zjstatus = {
      url = "github:dj95/zjstatus";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      home-manager,
      nixgl,
      fenix,
      yazelixScreen,
      yazelixGhosttyCursors,
      yazelixZellijBar,
      yazelixYaziAssets,
      yazelixZellij,
      yazelixYazi,
      yazelixZellijPaneOrchestrator,
      yazelixZellijPopup,
      beads,
      zjstatus,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      lib = nixpkgs.lib;
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkPkgs = system: nixpkgs.legacyPackages.${system};
      homeManagerModule = { pkgs, ... }: {
        _module.args.nixgl = nixgl;
        _module.args.fenixPkgs = fenix.packages.${pkgs.stdenv.hostPlatform.system};
        _module.args.mkYazelixPackage = mkYazelix pkgs.stdenv.hostPlatform.system;
        imports = [ ./home_manager/module.nix ];
      };
      agentUsagePackages = system:
        let
          pkgs = mkPkgs system;
        in
        [
          (import ./packaging/tokenusage.nix { inherit pkgs; })
        ];
      zellijPluginArtifactsFor =
        system:
        let
          paneOrchestrator =
            yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
          yzpp = yazelixZellijPopup.packages.${system}.yzpp;
        in
        {
          pane_orchestrator = "${paneOrchestrator}/${paneOrchestrator.wasmPath}";
          yzpp = "${yzpp}/${yzpp.wasmPath}";
        };
      mkYazelix =
        system:
        {
          pkgs ? mkPkgs system,
          src ? null,
          rust_core_src ? src,
          runtimeVariant ? "ghostty",
          runtimeToolSources ? { },
          components ? { },
          extraRuntimePackages ? [ ],
          yaziAssets ? yazelixYaziAssets.packages.${system}.yazelix_yazi_assets,
          zellijPluginArtifacts ? zellijPluginArtifactsFor system,
          enableZellijKittyPassthrough ? false,
        }:
        import ./yazelix_package.nix (
          {
            inherit
              pkgs
              nixgl
              runtimeVariant
              runtimeToolSources
              components
              yaziAssets
              zellijPluginArtifacts
              enableZellijKittyPassthrough
              ;
            extraRuntimePackages = [
              yazelixZellijBar.packages.${system}.yazelix_zellij_bar
            ] ++ extraRuntimePackages;
            fenixPkgs = fenix.packages.${pkgs.stdenv.hostPlatform.system};
          }
          // lib.optionalAttrs (src != null) { inherit src; }
          // lib.optionalAttrs (rust_core_src != null) { inherit rust_core_src; }
        );
      runtimePackage = system: pkgs: runtimeVariant: extraRuntimePackages:
        import ./yazelix_runtime_package.nix {
          inherit pkgs nixgl runtimeVariant;
          fenixPkgs = fenix.packages.${system};
          extraRuntimePackages = [
            yazelixZellijBar.packages.${system}.yazelix_zellij_bar
          ] ++ extraRuntimePackages;
          yaziAssets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          zellijPluginArtifacts = zellijPluginArtifactsFor system;
        };
      runtimePackageWith = system: pkgs: args:
        import ./yazelix_runtime_package.nix ({
          inherit pkgs nixgl;
          fenixPkgs = fenix.packages.${system};
          yaziAssets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          zellijPluginArtifacts = zellijPluginArtifactsFor system;
        } // args // {
          extraRuntimePackages = [
            yazelixZellijBar.packages.${system}.yazelix_zellij_bar
          ] ++ (args.extraRuntimePackages or [ ]);
        });
      yazelixPackage = system: pkgs: runtimeVariant: extraRuntimePackages:
        mkYazelix system {
          inherit pkgs runtimeVariant extraRuntimePackages;
        };
      kgpPreviewPkgs =
        system: pkgs:
        let
          yaziCodeSrc = builtins.path {
            path = yazelixYazi;
            name = "yazi-yazelix-kgp-preview-src";
          };
        in
        pkgs.extend (final: prev: {
          zellij = prev.zellij.overrideAttrs (_old: {
            src = yazelixZellij;
          });
          yazi-unwrapped = prev.yazi-unwrapped.overrideAttrs (old: {
            srcs = [
              yaziCodeSrc
              old.passthru.srcs.man_src
            ];
            sourceRoot = "yazi-yazelix-kgp-preview-src";
            passthru = old.passthru // {
              srcs = old.passthru.srcs // {
                code_src = yaziCodeSrc;
              };
            };
          });
          yazi = prev.yazi.override {
            yazi-unwrapped = final.yazi-unwrapped;
          };
        });
      defaultOverlay = final: _prev: {
        yazelix = mkYazelix final.stdenv.hostPlatform.system { pkgs = final; };
        yazelix_zellij_bar = yazelixZellijBar.packages.${final.stdenv.hostPlatform.system}.yazelix_zellij_bar;
        yazelix_yazi_assets =
          yazelixYaziAssets.packages.${final.stdenv.hostPlatform.system}.yazelix_yazi_assets;
        yazelix_zellij_pane_orchestrator =
          yazelixZellijPaneOrchestrator.packages.${final.stdenv.hostPlatform.system}.yazelix_zellij_pane_orchestrator;
        yazelix_zellij_popup = yazelixZellijPopup.packages.${final.stdenv.hostPlatform.system}.yzpp;
      };
      maintainerShell =
        system: pkgs:
        import ./maintainer_shell.nix {
          inherit pkgs nixgl;
          lib = nixpkgs.lib;
          fenixPkgs = fenix.packages.${system};
          bdPackage = (pkgs.callPackage "${beads}/default.nix" { self = beads; }).overrideAttrs (old: {
            vendorHash = "sha256-7DJgqJX2HDa9gcGD8fLNHLIXvGAEivYeDYx3snCUyCE=";
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ pkgs.pkg-config ];
            buildInputs = (old.buildInputs or [ ]) ++ [ pkgs.icu ];
          });
          repoRoot = ./.;
        };
      ciValidationShell =
        system: pkgs:
        let
          rustToolchain = fenix.packages.${system}.combine [
            fenix.packages.${system}.stable.cargo
            fenix.packages.${system}.stable.rustc
            fenix.packages.${system}.stable.rustfmt
          ];
        in
        pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.git
            pkgs.nix
            pkgs.nushell
          ];
        };
    in
    {
      lib = forAllSystems (system: {
        mkYazelix = mkYazelix system;
      });

      overlays.default = defaultOverlay;
      overlays.yazelix = defaultOverlay;

      packages = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
          defaultRuntimeVariant = "ghostty";
          noExtraRuntimePackages = [ ];
          agentUsageRuntimePackages = agentUsagePackages system;
          runtime_default = runtimePackage system pkgs defaultRuntimeVariant noExtraRuntimePackages;
          runtime_ghostty = runtimePackage system pkgs "ghostty" noExtraRuntimePackages;
          runtime_wezterm = runtimePackage system pkgs "wezterm" noExtraRuntimePackages;
          runtime_ghostty_kgp_preview = runtimePackageWith system (kgpPreviewPkgs system pkgs) {
            runtimeVariant = "ghostty";
            extraRuntimePackages = noExtraRuntimePackages;
            enableZellijKittyPassthrough = true;
          };
          runtime_ghostty_kgp_preview_agent_tools = runtimePackageWith system (kgpPreviewPkgs system pkgs) {
            runtimeVariant = "ghostty";
            extraRuntimePackages = agentUsageRuntimePackages;
            enableZellijKittyPassthrough = true;
          };
          runtime_agent_tools = runtimePackage system pkgs defaultRuntimeVariant agentUsageRuntimePackages;
          yazelix_default = yazelixPackage system pkgs defaultRuntimeVariant noExtraRuntimePackages;
          yazelix_ghostty = yazelixPackage system pkgs "ghostty" noExtraRuntimePackages;
          yazelix_wezterm = yazelixPackage system pkgs "wezterm" noExtraRuntimePackages;
          yazelix_ghostty_kgp_preview = mkYazelix system {
            pkgs = kgpPreviewPkgs system pkgs;
            runtimeVariant = "ghostty";
            extraRuntimePackages = noExtraRuntimePackages;
            enableZellijKittyPassthrough = true;
          };
          yazelix_ghostty_kgp_preview_agent_tools = mkYazelix system {
            pkgs = kgpPreviewPkgs system pkgs;
            runtimeVariant = "ghostty";
            extraRuntimePackages = agentUsageRuntimePackages;
            enableZellijKittyPassthrough = true;
          };
          yazelix_agent_tools = yazelixPackage system pkgs defaultRuntimeVariant agentUsageRuntimePackages;
          yazelix_zellij_bar = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
          yazelix_screen = yazelixScreen.packages.${system}.yzs;
          yazelix_ghostty_cursors = yazelixGhosttyCursors.packages.${system}.yazelix_ghostty_cursors;
          yazelix_zellij_pane_orchestrator =
            yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
          yazelix_zellij_popup = yazelixZellijPopup.packages.${system}.yzpp;
          yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
        in
        {
          default = yazelix_default;
          ghostty_cursor_shaders = yazelix_ghostty_cursors;
          runtime = runtime_default;
          runtime_agent_tools = runtime_agent_tools;
          runtime_ghostty = runtime_ghostty;
          runtime_ghostty_kgp_preview = runtime_ghostty_kgp_preview;
          runtime_ghostty_kgp_preview_agent_tools = runtime_ghostty_kgp_preview_agent_tools;
          runtime_wezterm = runtime_wezterm;
          yazelix = yazelix_default;
          yazelix_agent_tools = yazelix_agent_tools;
          yazelix_zellij_bar = yazelix_zellij_bar;
          yazelix_ghostty_cursors = yazelix_ghostty_cursors;
          yazelix_ghostty = yazelix_ghostty;
          yazelix_ghostty_kgp_preview = yazelix_ghostty_kgp_preview;
          yazelix_ghostty_kgp_preview_agent_tools = yazelix_ghostty_kgp_preview_agent_tools;
          yazelix_screen = yazelix_screen;
          yazelix_wezterm = yazelix_wezterm;
          yazelix_yazi_assets = yazelix_yazi_assets;
          yazelix_zellij_pane_orchestrator = yazelix_zellij_pane_orchestrator;
          yazelix_zellij_popup = yazelix_zellij_popup;
          yzs = yazelix_screen;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.yazelix}/bin/yzx";
        };
        yazelix = {
          type = "app";
          program = "${self.packages.${system}.yazelix}/bin/yzx";
        };
        yazelix_ghostty = {
          type = "app";
          program = "${self.packages.${system}.yazelix_ghostty}/bin/yzx";
        };
        yazelix_ghostty_kgp_preview = {
          type = "app";
          program = "${self.packages.${system}.yazelix_ghostty_kgp_preview}/bin/yzx";
        };
        yazelix_ghostty_kgp_preview_agent_tools = {
          type = "app";
          program = "${self.packages.${system}.yazelix_ghostty_kgp_preview_agent_tools}/bin/yzx";
        };
        yazelix_wezterm = {
          type = "app";
          program = "${self.packages.${system}.yazelix_wezterm}/bin/yzx";
        };
        yazelix_agent_tools = {
          type = "app";
          program = "${self.packages.${system}.yazelix_agent_tools}/bin/yzx";
        };
        yazelix_screen = {
          type = "app";
          program = "${self.packages.${system}.yazelix_screen}/bin/yzs";
        };
        yzs = {
          type = "app";
          program = "${self.packages.${system}.yazelix_screen}/bin/yzs";
        };
        yazelix_ghostty_cursors = {
          type = "app";
          program = "${self.packages.${system}.yazelix_ghostty_cursors}/bin/yzc";
        };
        yzc = {
          type = "app";
          program = "${self.packages.${system}.yazelix_ghostty_cursors}/bin/yzc";
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
        in
        {
          ci = ciValidationShell system pkgs;
          default = maintainerShell system pkgs;
        }
      );

      homeManagerModules.default = homeManagerModule;
      homeManagerModules.yazelix = homeManagerModule;
    };
}
