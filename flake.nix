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
      url = "github:luccahuguet/yazelix-zellij/yazelix-kgp-preview-1";
      flake = false;
    };
    yazelixYazi = {
      url = "github:luccahuguet/yazelix-yazi/yazelix-kgp-preview-0";
      flake = false;
    };
    yazelixHelix = {
      url = "github:luccahuguet/yazelix-helix";
      inputs.nixpkgs.follows = "nixpkgs";
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
      yazelixHelix,
      yazelixZellijPaneOrchestrator,
      yazelixZellijPopup,
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
          extraRuntimePackages ? agentUsagePackages system,
          yaziAssets ? yazelixYaziAssets.packages.${system}.yazelix_yazi_assets,
          screenAssets ? yazelixScreen.packages.${system}.yzs,
          zellijPluginArtifacts ? zellijPluginArtifactsFor system,
          enableZellijKittyPassthrough ? false,
        }:
        let
          runtimePkgs = runtimePkgsFor system pkgs runtimeVariant;
        in
        import ./yazelix_package.nix (
          {
            inherit nixgl runtimeVariant runtimeToolSources components yaziAssets zellijPluginArtifacts;
            inherit screenAssets;
            pkgs = runtimePkgs;
            enableZellijKittyPassthrough =
              enableZellijKittyPassthrough || builtins.elem runtimeVariant [
                "ghostty"
                "ratty"
              ];
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
          inherit nixgl runtimeVariant;
          pkgs = runtimePkgsFor system pkgs runtimeVariant;
          fenixPkgs = fenix.packages.${system};
          extraRuntimePackages = [
            yazelixZellijBar.packages.${system}.yazelix_zellij_bar
          ] ++ extraRuntimePackages;
          screenAssets = yazelixScreen.packages.${system}.yzs;
          yaziAssets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          zellijPluginArtifacts = zellijPluginArtifactsFor system;
          enableZellijKittyPassthrough = builtins.elem runtimeVariant [
            "ghostty"
            "ratty"
          ];
        };
      yazelixPackage = system: pkgs: runtimeVariant: extraRuntimePackages:
        mkYazelix system {
          inherit pkgs runtimeVariant extraRuntimePackages;
        };
      runtimePkgsFor =
        system: pkgs: runtimeVariant:
        let
          helixPkgs = yazelixHelixPkgs system pkgs;
        in
        if builtins.elem runtimeVariant [
          "ghostty"
          "ratty"
        ] then
          yazelixGraphicsPkgs system helixPkgs
        else
          helixPkgs;
      yazelixKgpZellij =
        pkgs: baseZellij:
        import ./packaging/yazelix_kgp_zellij.nix {
          inherit pkgs baseZellij;
          src = yazelixZellij;
        };
      yazelixKgpYazi =
        pkgs: baseYaziUnwrapped: codeSrc:
        import ./packaging/yazelix_kgp_yazi.nix {
          inherit pkgs baseYaziUnwrapped codeSrc;
        };
      yazelixHelixPackage = system: yazelixHelix.packages.${system}.yazelix_helix;
      yazelixHelixPkgs =
        system: pkgs:
        pkgs.extend (_final: _prev: {
          helix = yazelixHelixPackage system;
        });
      yazelixGraphicsPkgs =
        system: pkgs:
        let
          yaziCodeSrc = builtins.path {
            path = yazelixYazi;
            name = "yazi-yazelix-kgp-src";
          };
        in
        pkgs.extend (final: prev: {
          zellij = yazelixKgpZellij final prev.zellij;
          yazi-unwrapped = yazelixKgpYazi final prev.yazi-unwrapped yaziCodeSrc;
          yazi = prev.yazi.override {
            yazi-unwrapped = final.yazi-unwrapped;
          };
        });
      kgpPackageContractCheck =
        system:
        let
          pkgs = mkPkgs system;
          poisonedConsumerPkgs = import nixpkgs {
            inherit system;
            overlays = [
              (_final: prev: {
                zellij = prev.zellij.overrideAttrs (_old: {
                  __intentionallyOverridingVersion = true;
                  version = "0.44.1";
                  cargoDeps = throw "consumer pkgs.zellij cargoDeps leaked into Yazelix KGP Zellij";
                  patches = throw "consumer pkgs.zellij patches leaked into Yazelix KGP Zellij";
                  prePatch = throw "consumer pkgs.zellij prePatch leaked into Yazelix KGP Zellij";
                  postPatch = throw "consumer pkgs.zellij postPatch leaked into Yazelix KGP Zellij";
                  installCheckPhase =
                    throw "consumer pkgs.zellij installCheckPhase leaked into Yazelix KGP Zellij";
                });
                yazi-unwrapped = prev.yazi-unwrapped.overrideAttrs (_old: {
                  cargoDeps = throw "consumer pkgs.yazi-unwrapped cargoDeps leaked into Yazelix KGP Yazi";
                  patches = throw "consumer pkgs.yazi-unwrapped patches leaked into Yazelix KGP Yazi";
                  prePatch = throw "consumer pkgs.yazi-unwrapped prePatch leaked into Yazelix KGP Yazi";
                  postPatch = throw "consumer pkgs.yazi-unwrapped postPatch leaked into Yazelix KGP Yazi";
                });
              })
            ];
          };
          yaziCodeSrc = builtins.path {
            path = yazelixYazi;
            name = "yazi-yazelix-kgp-src";
          };
          kgpZellij = yazelixKgpZellij poisonedConsumerPkgs poisonedConsumerPkgs.zellij;
          kgpYazi = yazelixKgpYazi poisonedConsumerPkgs poisonedConsumerPkgs.yazi-unwrapped yaziCodeSrc;
        in
        assert (kgpZellij.version or "") == "0.44.3";
        assert (kgpZellij.cargoDeps.name or "") == "zellij-0.44.3-vendor";
        assert (kgpZellij.patches or [ ]) == [ ];
        assert (kgpZellij.prePatch or "") == "";
        assert (kgpZellij.postPatch or "") == "";
        assert (kgpZellij.installCheckPhase or "") == ''
          runHook preInstallCheck
          runHook postInstallCheck
        '';
        assert (kgpYazi.version or "") == "26.5.6";
        assert (kgpYazi.cargoDeps.name or "") == "yazi-26.5.6-vendor";
        assert (kgpYazi.patches or [ ]) == [ ];
        assert (kgpYazi.prePatch or "") == "";
        assert (kgpYazi.postPatch or "") == "";
        pkgs.runCommand "yazelix-kgp-package-contracts" { } ''
          touch "$out"
        '';
      defaultOverlay =
        final: _prev:
        let
          system = final.stdenv.hostPlatform.system;
        in
        {
          yazelix = mkYazelix system { pkgs = final; };
          yazelix_zellij_bar = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
          yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          yazelix_helix = yazelixHelixPackage system;
          yazelix_zellij_pane_orchestrator =
            yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
          yazelix_zellij_popup = yazelixZellijPopup.packages.${system}.yzpp;
        };
      beadsRustPackage =
        system: pkgs:
        import ./packaging/beads_rust.nix {
          inherit pkgs;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = fenix.packages.${system}.latest.cargo;
            rustc = fenix.packages.${system}.latest.rustc;
          };
        };
      maintainerShell =
        system: pkgs:
        import ./maintainer_shell.nix {
          inherit pkgs nixgl;
          lib = nixpkgs.lib;
          fenixPkgs = fenix.packages.${system};
          brPackage = beadsRustPackage system pkgs;
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
          defaultRuntimePackages = agentUsagePackages system;
          agentUsageRuntimePackages = agentUsagePackages system;
          rattyPackages = lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
            runtime_ratty = runtimePackage system pkgs "ratty" defaultRuntimePackages;
            yazelix_ratty = yazelixPackage system pkgs "ratty" defaultRuntimePackages;
          };
          runtime_default = runtimePackage system pkgs defaultRuntimeVariant defaultRuntimePackages;
          runtime_ghostty = runtimePackage system pkgs "ghostty" defaultRuntimePackages;
          runtime_wezterm = runtimePackage system pkgs "wezterm" defaultRuntimePackages;
          runtime_agent_tools = runtimePackage system pkgs defaultRuntimeVariant agentUsageRuntimePackages;
          graphicsPkgs = yazelixGraphicsPkgs system pkgs;
          yazelix_default = yazelixPackage system pkgs defaultRuntimeVariant defaultRuntimePackages;
          yazelix_ghostty = yazelixPackage system pkgs "ghostty" defaultRuntimePackages;
          yazelix_wezterm = yazelixPackage system pkgs "wezterm" defaultRuntimePackages;
          yazelix_agent_tools = yazelixPackage system pkgs defaultRuntimeVariant agentUsageRuntimePackages;
          yazelix_zellij_bar = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
          yazelix_screen = yazelixScreen.packages.${system}.yzs;
          yazelix_ghostty_cursors = yazelixGhosttyCursors.packages.${system}.yazelix_ghostty_cursors;
          yazelix_helix = yazelixHelixPackage system;
          yazelix_zellij_pane_orchestrator =
            yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
          yazelix_zellij_popup = yazelixZellijPopup.packages.${system}.yzpp;
          yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          beads_rust = beadsRustPackage system pkgs;
        in
        ({
          br = beads_rust;
          beads_rust = beads_rust;
          default = yazelix_default;
          ghostty_cursor_shaders = yazelix_ghostty_cursors;
          runtime = runtime_default;
          runtime_agent_tools = runtime_agent_tools;
          runtime_ghostty = runtime_ghostty;
          runtime_wezterm = runtime_wezterm;
          yazelix = yazelix_default;
          yazelix_agent_tools = yazelix_agent_tools;
          yazelix_zellij_bar = yazelix_zellij_bar;
          yazelix_ghostty_cursors = yazelix_ghostty_cursors;
          yazelix_ghostty = yazelix_ghostty;
          yazelix_screen = yazelix_screen;
          yazelix_helix = yazelix_helix;
          yazelix_kgp_yazi = graphicsPkgs.yazi-unwrapped;
          yazelix_kgp_zellij = graphicsPkgs.zellij;
          yazelix_wezterm = yazelix_wezterm;
          yazelix_yazi_assets = yazelix_yazi_assets;
          yazelix_zellij_pane_orchestrator = yazelix_zellij_pane_orchestrator;
          yazelix_zellij_popup = yazelix_zellij_popup;
          yzs = yazelix_screen;
        } // rattyPackages)
      );

      apps = forAllSystems (
        system:
        let
          pkgs = mkPkgs system;
        in
        {
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
        }
        // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          yazelix_ratty = {
            type = "app";
            program = "${self.packages.${system}.yazelix_ratty}/bin/yzx";
          };
        }
      );

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

      checks = forAllSystems (system: {
        kgp_package_contracts = kgpPackageContractCheck system;
      });

      homeManagerModules.default = homeManagerModule;
      homeManagerModules.yazelix = homeManagerModule;
    };
}
