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
      cargoGitOutputHashes = import ./packaging/cargo_git_output_hashes.nix {
        inherit yazelixCursors yazelixYaziAssets;
      };
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
      repoSource = import ./packaging/repo_source.nix {
        inherit lib;
        src = ./.;
      };
      homeManagerModule = import ./home_manager/module.nix {
        defaultPackageFor = yazelixPackageFor;
      };
      homeManagerDefaultActivationPackage =
        system:
        let
          pkgs = mkPkgs system;
          homeDirectory =
            if pkgs.stdenv.hostPlatform.isDarwin then "/Users/yazelix-ci" else "/home/yazelix-ci";
        in
        (home-manager.lib.homeManagerConfiguration {
          inherit pkgs;
          modules = [
            homeManagerModule
            {
              home.username = "yazelix-ci";
              home.homeDirectory = homeDirectory;
              home.stateVersion = "24.11";
              programs.yazelix.enable = true;
            }
          ];
        }).activationPackage;
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
      yazelixPackageFor =
        system:
        let
          pkgs = kgpPackages.graphicsPkgs (kgpPackages.helixPkgs system (mkPkgs system));
        in
        import ./packaging/mk_yazelix_package.nix {
          inherit cargoGitOutputHashes nixgl pkgs;
          src = repoSource;
          rust_core_src = ./.;
          runtimeIdentity = defaultRuntimeIdentity;
          fenixPkgs = fenix.packages.${system};
          extraRuntimePackages = [
            yazelixZellijBar.packages.${system}.yazelix_zellij_bar
          ] ++ agentUsagePackages system;
          yaziAssets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
          yazelixHelixPackage = kgpPackages.helixPackage system;
          yazelixCursorsPackage = yazelixCursors.packages.${system}.yazelix_cursors;
          marsTerminalPackage = mars.packages.${system}.mars;
          zellijPluginArtifacts = zellijPluginArtifactsFor system;
          enableZellijKittyPassthrough = true;
          metaPlatforms = systems;
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
          inherit cargoGitOutputHashes pkgs nixgl;
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
    in
    {
      packages = forAllSystems (system: {
        default = yazelixPackageFor system;
        yazelix = yazelixPackageFor system;
      });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${yazelixPackageFor system}/bin/yzx";
        };
        yazelix = {
          type = "app";
          program = "${yazelixPackageFor system}/bin/yzx";
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

      checks = forAllSystems (system: {
        cargo_git_output_hash_contracts = import ./packaging/cargo_git_output_hash_contracts.nix {
          inherit yazelixCursors yazelixYaziAssets;
          pkgs = mkPkgs system;
        };
        home_manager_default = homeManagerDefaultActivationPackage system;
        kgp_package_contracts = import ./packaging/kgp_package_contracts.nix {
          inherit nixpkgs system kgpPackages;
        };
      });

      homeManagerModules.default = homeManagerModule;
    };
}
