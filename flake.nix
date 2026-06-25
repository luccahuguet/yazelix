{
  description = "Yazelix flake interface";

  nixConfig = {
    extra-substituters = [
      "https://yazelix.cachix.org"
    ];
    extra-trusted-public-keys = [
      "yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixgl.url = "github:guibou/nixGL";
    rio = {
      url = "github:raphamorim/rio/v0.4.7";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    mars = {
      url = "github:luccahuguet/mars";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixScreen = {
      url = "github:luccahuguet/yazelix-screen";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "fenix";
    };
    yazelixCursors = {
      url = "github:luccahuguet/yazelix-cursors";
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
      url = "github:luccahuguet/yazelix-zellij/yazelix_kgp_preview";
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
      url = "github:luccahuguet/zjstatus/yazelix-tab-activity-pipe";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      home-manager,
      nixgl,
      rio,
      mars,
      fenix,
      yazelixScreen,
      yazelixCursors,
      yazelixZellijBar,
      yazelixYaziAssets,
      yazelixZellij,
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
      releaseMetadata = builtins.fromTOML (builtins.readFile ./release_metadata.toml);
      inputIdentity = input: {
        revision = input.rev or null;
        short_revision = input.shortRev or null;
        last_modified_date = input.lastModifiedDate or null;
      };
      defaultRuntimeIdentity = {
        version = releaseMetadata.version;
        source = {
          revision = self.rev or self.dirtyRev or null;
          short_revision = self.shortRev or self.dirtyShortRev or null;
          last_modified_date = self.lastModifiedDate or null;
        };
        inputs = {
          nixpkgs = inputIdentity nixpkgs;
          home_manager = inputIdentity home-manager;
          rio = inputIdentity rio;
          mars = inputIdentity mars;
          fenix = inputIdentity fenix;
          yazelix_screen = inputIdentity yazelixScreen;
          yazelix_cursors = inputIdentity yazelixCursors;
          yazelix_zellij_bar = inputIdentity yazelixZellijBar;
          yazelix_yazi_assets = inputIdentity yazelixYaziAssets;
          yazelix_helix = inputIdentity yazelixHelix;
          yazelix_zellij_pane_orchestrator = inputIdentity yazelixZellijPaneOrchestrator;
          yazelix_zellij_popup = inputIdentity yazelixZellijPopup;
        };
      };
      homeManagerModule = { pkgs, ... }: {
        _module.args.nixgl = nixgl;
        _module.args.rioPackage = rio.packages.${pkgs.stdenv.hostPlatform.system}.rio;
        _module.args.fenixPkgs = fenix.packages.${pkgs.stdenv.hostPlatform.system};
        _module.args.mkYazelixPackage = mkYazelix pkgs.stdenv.hostPlatform.system;
        _module.args.yazelixHelixPackage =
          kgpPackages.helixPackage pkgs.stdenv.hostPlatform.system;
        _module.args.yazelixCursorsPackage =
          yazelixCursors.packages.${pkgs.stdenv.hostPlatform.system}.yazelix_cursors;
        _module.args.marsTerminalPackage = mars.packages.${pkgs.stdenv.hostPlatform.system}.mars;
        imports = [ ./home_manager/module.nix ];
      };
      agentUsagePackages = system:
        let
          pkgs = mkPkgs system;
        in
        [
          (import ./packaging/tokenusage.nix { inherit pkgs; })
        ];
      kgpPackages = import ./packaging/kgp_packages.nix {
        inherit yazelixZellij yazelixHelix;
      };
      terminalMetadataFor =
        pkgs:
        import ./packaging/terminal_variants.nix {
          inherit (pkgs.stdenv.hostPlatform) isLinux;
        };
      terminalNeedsKittyPassthrough =
        pkgs: runtimeVariant:
        builtins.elem runtimeVariant (terminalMetadataFor pkgs).kittyPassthrough;
      zellijPluginArtifactsFor =
        system:
        let
          paneOrchestrator =
            yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
          zjstatusPackage = zjstatus.packages.${system}.default;
          yzpp = yazelixZellijPopup.packages.${system}.yzpp;
        in
        {
          pane_orchestrator = "${paneOrchestrator}/${paneOrchestrator.wasmPath}";
          zjstatus = "${zjstatusPackage}/bin/zjstatus.wasm";
          yzpp = "${yzpp}/${yzpp.wasmPath}";
        };
      mkYazelix =
        system:
        {
          pkgs ? mkPkgs system,
          src ? null,
          rust_core_src ? src,
          runtimeVariant ? "mars",
          runtimeToolSources ? { },
          runtimeIdentity ? defaultRuntimeIdentity,
          name ? "yazelix",
          runtimeName ? "yazelix-runtime",
          skipStableWrapperRedirect ? false,
          components ? { },
          extraRuntimePackages ? agentUsagePackages system,
          yaziAssets ? yazelixYaziAssets.packages.${system}.yazelix_yazi_assets,
          rioPackage ? rio.packages.${system}.rio,
          yazelixHelixPackage ? kgpPackages.helixPackage system,
          yazelixCursorsPackage ? yazelixCursors.packages.${system}.yazelix_cursors,
          marsTerminalPackage ? mars.packages.${system}.mars,
          zellijPluginArtifacts ? zellijPluginArtifactsFor system,
          enableZellijKittyPassthrough ? false,
        }:
        let
          runtimePkgs = runtimePkgsFor system pkgs runtimeVariant;
        in
        import ./yazelix_package.nix (
          {
            inherit nixgl runtimeVariant runtimeToolSources components yaziAssets zellijPluginArtifacts;
            inherit runtimeIdentity;
            inherit name rioPackage runtimeName skipStableWrapperRedirect marsTerminalPackage;
            inherit yazelixHelixPackage yazelixCursorsPackage;
            pkgs = runtimePkgs;
            enableZellijKittyPassthrough =
              enableZellijKittyPassthrough || terminalNeedsKittyPassthrough pkgs runtimeVariant;
            extraRuntimePackages = [
              yazelixZellijBar.packages.${system}.yazelix_zellij_bar
            ] ++ extraRuntimePackages;
            fenixPkgs = fenix.packages.${pkgs.stdenv.hostPlatform.system};
          }
          // lib.optionalAttrs (src != null) { inherit src; }
          // lib.optionalAttrs (rust_core_src != null) { inherit rust_core_src; }
        );
      runtimePackageWith =
        system: pkgs: runtimeVariant: extraRuntimePackages:
        {
          name ? "yazelix-runtime",
          runtimeIdentity ? defaultRuntimeIdentity,
          rioPackage ? rio.packages.${system}.rio,
          yazelixHelixPackage ? kgpPackages.helixPackage system,
          yazelixCursorsPackage ? yazelixCursors.packages.${system}.yazelix_cursors,
          marsTerminalPackage ? mars.packages.${system}.mars,
        }:
        import ./yazelix_runtime_package.nix {
          inherit nixgl name rioPackage runtimeIdentity runtimeVariant yazelixHelixPackage yazelixCursorsPackage marsTerminalPackage;
          pkgs = runtimePkgsFor system pkgs runtimeVariant;
          fenixPkgs = fenix.packages.${system};
          extraRuntimePackages = [
            yazelixZellijBar.packages.${system}.yazelix_zellij_bar
          ] ++ extraRuntimePackages;
          yaziAssets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          zellijPluginArtifacts = zellijPluginArtifactsFor system;
          enableZellijKittyPassthrough = terminalNeedsKittyPassthrough pkgs runtimeVariant;
        };
      runtimePackage = system: pkgs: runtimeVariant: extraRuntimePackages:
        runtimePackageWith system pkgs runtimeVariant extraRuntimePackages { };
      yazelixPackage = system: pkgs: runtimeVariant: extraRuntimePackages:
        mkYazelix system {
          inherit pkgs runtimeVariant extraRuntimePackages;
        };
      runtimePkgsFor =
        system: pkgs: runtimeVariant:
        let
          helixPkgs = kgpPackages.helixPkgs system pkgs;
        in
        if terminalNeedsKittyPassthrough pkgs runtimeVariant then
          kgpPackages.graphicsPkgs helixPkgs
        else
          helixPkgs;
      defaultOverlay =
        final: _prev:
        let
          system = final.stdenv.hostPlatform.system;
        in
        {
          yazelix = mkYazelix system { pkgs = final; };
          yazelix_zellij_bar = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
          yazelix_zellij_config_pack = import ./packaging/yazelix_zellij_config_pack.nix {
            pkgs = final;
            fenixPkgs = fenix.packages.${system};
          };
          yazelix_yazi_assets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          yazelix_helix = kgpPackages.helixPackage system;
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
            (beadsRustPackage system pkgs)
            rustToolchain
            pkgs.git
            pkgs.github-cli
            pkgs.nushell
          ];
        };
      systemOutputs =
        system:
        let
          pkgs = mkPkgs system;
        in
        import ./packaging/flake_outputs.nix {
          inherit agentUsagePackages beadsRustPackage defaultRuntimeIdentity kgpPackages lib;
          inherit mkYazelix pkgs runtimePackage runtimePackageWith system yazelixPackage;
          inherit yazelixCursors yazelixScreen yazelixYaziAssets;
          inherit yazelixZellijBar yazelixZellijPaneOrchestrator;
          inherit yazelixZellijPopup;
          fenixPkgs = fenix.packages.${system};
          terminalMetadata = terminalMetadataFor pkgs;
        };
    in
    {
      lib = forAllSystems (system: {
        mkYazelix = mkYazelix system;
      });

      overlays.default = defaultOverlay;
      overlays.yazelix = defaultOverlay;

      packages = forAllSystems (system: (systemOutputs system).packages);

      apps = forAllSystems (system: (systemOutputs system).apps);

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
        kgp_package_contracts = import ./packaging/kgp_package_contracts.nix {
          inherit nixpkgs system kgpPackages;
        };
      });

      homeManagerModules.default = homeManagerModule;
      homeManagerModules.yazelix = homeManagerModule;
    };
}
