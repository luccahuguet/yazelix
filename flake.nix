{
  description = "Yazelix Next";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    mars = {
      url = "github:luccahuguet/mars";
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
    yazelixZellijPopup = {
      url = "github:luccahuguet/yazelix-zellij-popup";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    autoLayoutYazi = {
      url = "github:luccahuguet/auto-layout.yazi";
      flake = false;
    };
    starshipYazi = {
      url = "github:Rolv-Apneseth/starship.yazi";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    mars,
    yazelixZellij,
    yazelixHelix,
    yazelixZellijPopup,
    autoLayoutYazi,
    starshipYazi,
  }: let
    eachSystem = nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"];
    rustBinFor = pkgs: name: src: pkgs.runCommand name {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
      mkdir -p "$out/bin"
      rustc --edition=2024 ${src} -o "$out/bin/${name}"
    '';
    bridgeSessionEnv = prefix: ''
      if [ -z "''${YAZELIX_HELIX_BRIDGE_SESSION_ID:-}" ]; then
        YAZELIX_HELIX_BRIDGE_SESSION_ID="${prefix}-$(date +%s)-$$"
      fi
      export YAZELIX_HELIX_BRIDGE_SESSION_ID
    '';
  in {
    packages = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      rustBin = rustBinFor pkgs;
      marsPackage = mars.packages.${system}.mars;
      yznMarsToml = pkgs.replaceVars ./mars.toml {
        jetbrainsMonoDir = "${pkgs.jetbrains-mono}/share/fonts/truetype";
        symbolsNerdDir = "${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols";
        notoSymbolsDir = "${pkgs.noto-fonts}/share/fonts/noto";
        notoEmojiDir = "${pkgs.noto-fonts-color-emoji}/share/fonts/noto";
      };
      yznMarsConfig = pkgs.runCommand "yzn-mars-config" {} ''
        install -D -m 644 ${yznMarsToml} "$out/config.toml"
      '';
      yznCarapaceInit = pkgs.runCommand "yzn-carapace-init" {} ''
        ${pkgs.carapace}/bin/carapace _carapace nushell > "$out"
      '';
      yznZoxideInit = pkgs.runCommand "yzn-zoxide-init" {} ''
        ${pkgs.zoxide}/bin/zoxide init nushell > "$out"
      '';
      yznNuConfigNu = pkgs.replaceVars ./nu/config.nu {
        carapaceInit = "${yznCarapaceInit}";
        starship = "${pkgs.starship}/bin/starship";
        zoxideInit = "${yznZoxideInit}";
      };
      yznNuConfig = pkgs.runCommand "yzn-nu-config" {} ''
        install -D -m 644 ${yznNuConfigNu} "$out/config.nu"
        install -D -m 644 ${./nu/env.nu} "$out/env.nu"
      '';
      yznNuRs = pkgs.replaceVars ./runtime/yzn-nu.rs {
        nu = "${pkgs.nushell}/bin/nu";
        packagedNu = "${yznNuConfig}";
        pathPrefix = pkgs.lib.makeBinPath [pkgs.nushell pkgs.starship pkgs.carapace pkgs.zoxide];
      };
      yznNuShell = rustBin "yzn-nu" yznNuRs;
      yazelixZellijPopupPackage = yazelixZellijPopup.packages.${system}.yzpp;
      yznConfigKdl = pkgs.replaceVars ./config.kdl {
        nuShell = "${yznNuShell}/bin/yzn-nu";
        yzpp = "file:${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
        lazygit = "${pkgs.lazygit}/bin/lazygit";
      };
      yznZellijConfig = rustBin "yzn-zellij-config" ./runtime/yzn-zellij-config.rs;
      yazelixHelixPackage = yazelixHelix.packages.${system}.yazelix_helix;
      yznHelixConfig = pkgs.runCommand "yzn-helix-config" {} ''
        install -D -m 644 ${./helix/config.toml} "$out/config.toml"
      '';
      yznHelix = pkgs.writeShellApplication {
        name = "yzn-hx";
        runtimeInputs = [pkgs.coreutils];
        text = ''
          export YAZELIX_STATE_DIR="''${YAZELIX_STATE_DIR:-''${XDG_RUNTIME_DIR:-/tmp}/yazelix-next}"
          ${bridgeSessionEnv "yzn-helper"}
          export YAZELIX_HELIX_BRIDGE=1
          YAZELIX_HELIX_BRIDGE_INSTANCE_ID="hx-$(date +%s)-$$"
          export YAZELIX_HELIX_BRIDGE_INSTANCE_ID
          YAZELIX_HELIX_BRIDGE_AUTH_TOKEN="$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')"
          export YAZELIX_HELIX_BRIDGE_AUTH_TOKEN
          export YAZELIX_HELIX_MANAGED_CONFIG_PATH=${yznHelixConfig}
          mkdir -p "$YAZELIX_STATE_DIR"
          exec ${yazelixHelixPackage}/bin/hx --config-dir ${yznHelixConfig} "$@"
        '';
      };
      yaziAssetsSelection = pkgs.fetchFromGitHub {
        owner = "luccahuguet";
        repo = "yazelix-yazi-assets";
        rev = "aea0703247479e1fa373be6b305e24e568cb30c7";
        sparseCheckout = ["plugins/git.yazi" "yazelix_starship.toml"];
        nonConeMode = true;
        hash = "sha256-eHt6kRaLcXgjhdnmhI2QY2O1tF9wGFXbIjXc4pObF4U=";
      };
      yznOpenCore = pkgs.rustPlatform.buildRustPackage {
        pname = "yzn-open";
        version = "0.1.0";
        src = ./crates/yzn-open;
        cargoLock.lockFile = ./crates/yzn-open/Cargo.lock;
      };
      yznYaziToml = pkgs.replaceVars ./yazi/yazi.toml {
        opener = "YZN_EDITOR=${yznHelix}/bin/yzn-hx YZN_ZELLIJ=${yazelixZellijPackage}/bin/zellij ${yznOpenCore}/bin/yzn-open";
      };
      yznYaziConfig = pkgs.runCommand "yzn-yazi-config" {} ''
        install -D -m 644 ${./yazi/init.lua} "$out/init.lua"
        install -D -m 644 ${yznYaziToml} "$out/yazi.toml"
        install -D -m 644 ${yaziAssetsSelection}/yazelix_starship.toml "$out/yazelix_starship.toml"
        mkdir -p "$out/plugins"
        install -D -m 644 ${./yazi/plugins/sidebar-status.yazi/main.lua} "$out/plugins/sidebar-status.yazi/main.lua"
        ln -s ${autoLayoutYazi} "$out/plugins/auto-layout.yazi"
        ln -s ${yaziAssetsSelection}/plugins/git.yazi "$out/plugins/git.yazi"
        ln -s ${starshipYazi} "$out/plugins/starship.yazi"
      '';
      yznYazi = pkgs.writeShellApplication {
        name = "yzn-yazi";
        runtimeInputs = [pkgs.git pkgs.starship];
        text = ''
          export YAZELIX_STATE_DIR="''${YAZELIX_STATE_DIR:-''${XDG_RUNTIME_DIR:-/tmp}/yazelix-next}"
          ${bridgeSessionEnv "yzn-helper"}
          export YAZI_CONFIG_HOME=${yznYaziConfig}
          export YZN_YAZI_STARSHIP_CONFIG=${yznYaziConfig}/yazelix_starship.toml
          export EDITOR=${yznHelix}/bin/yzn-hx
          export VISUAL=${yznHelix}/bin/yzn-hx
          exec ${pkgs.yazi}/bin/yazi "$@"
        '';
      };
      yznLayoutKdl = pkgs.replaceVars ./layout.kdl {
        yazi = "${yznYazi}/bin/yzn-yazi";
      };
      yznLayoutSwapKdl = pkgs.replaceVars ./layout.swap.kdl {
        yazi = "${yznYazi}/bin/yzn-yazi";
      };
      yznLayoutCheck = rustBin "yzn-layout-check" ./checks/zellij-layout.rs;
      yznZellijLayout = pkgs.runCommand "yzn-zellij-layout" {} ''
        ${yznLayoutCheck}/bin/yzn-layout-check ${yznLayoutKdl} ${yznLayoutSwapKdl}
        install -D -m 644 ${yznLayoutKdl} "$out/layout.kdl"
        install -D -m 644 ${yznLayoutSwapKdl} "$out/layout.swap.kdl"
      '';
      yazelixZellijPackage = pkgs."zellij-unwrapped".overrideAttrs (_old: {
        pname = "zellij";
        version = "0.44.3";
        src = yazelixZellij;
        patches = [];
        prePatch = "";
        postPatch = "";
        installCheckPhase = ''
          runHook preInstallCheck
          runHook postInstallCheck
        '';
        cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
          pname = "zellij";
          version = "0.44.3";
          src = yazelixZellij;
          hash = "sha256-966FpfSsF9I10SrYe3+YNsfM2kLLv+gd0/Aw8vLp4Lk=";
        };
        doCheck = false;
      });
      yznRuntimeEnv = ''
        export YAZELIX_STATE_DIR="''${YAZELIX_STATE_DIR:-''${XDG_RUNTIME_DIR:-/tmp}/yazelix-next}"
        mkdir -p "$YAZELIX_STATE_DIR"
        ${bridgeSessionEnv "yzn"}
        export EDITOR=${yznHelix}/bin/yzn-hx
        export VISUAL=${yznHelix}/bin/yzn-hx
        if [ -n "''${YAZELIX_NEXT_CONFIG_HOME:-}" ]; then
          yzn_config_home="$YAZELIX_NEXT_CONFIG_HOME"
        elif [ -n "''${XDG_CONFIG_HOME:-}" ]; then
          yzn_config_home="$XDG_CONFIG_HOME/yazelix-next"
        else
          yzn_config_home="''${HOME:?HOME is required}/.config/yazelix-next"
        fi
        if [ -f "$yzn_config_home/mars/config.toml" ]; then
          export MARS_CONFIG_HOME="$yzn_config_home/mars"
        else
          export MARS_CONFIG_HOME=${yznMarsConfig}
        fi
        zellij_config="$(${yznZellijConfig}/bin/yzn-zellij-config ${yznConfigKdl} "$yzn_config_home/zellij/config.kdl" "$YAZELIX_STATE_DIR/zellij/config.kdl")"
      '';
      yznCommand = pkgs.writeShellApplication {
        name = "yzn";
        runtimeInputs = [pkgs.coreutils];
        text = ''
          show_help() {
            cat <<'EOF'
Yazelix

Usage:
  yzn
  yzn help
  yzn enter [zellij-args...]
  yzn launch [zellij-args...]

Commands:
  enter   Start Yazelix in the current terminal
  launch  Open Mars and start Yazelix
  help    Show this help
EOF
          }

          [ "$#" -gt 0 ] || set -- launch
          case "$1" in
            help|-h|--help)
              show_help
              ;;
            enter)
              shift
              ${yznRuntimeEnv}
              exec ${yazelixZellijPackage}/bin/zellij --config "$zellij_config" --new-session-with-layout ${yznZellijLayout}/layout.kdl "$@"
              ;;
            launch)
              shift
              ${yznRuntimeEnv}
              exec ${marsPackage}/bin/mars -e ${yazelixZellijPackage}/bin/zellij --config "$zellij_config" --new-session-with-layout ${yznZellijLayout}/layout.kdl "$@"
              ;;
            *)
              printf 'yzn: unknown command: %s\n\n' "$1" >&2
              show_help >&2
              exit 64
              ;;
          esac
        '';
      };
      yznDesktop = pkgs.makeDesktopItem {
        name = "yzn";
        desktopName = "Yazelix Next";
        genericName = "Terminal Emulator";
        comment = "Open Yazelix Next";
        exec = "${yznCommand}/bin/yzn";
        icon = "yzn";
        terminal = false;
        categories = ["System" "TerminalEmulator"];
        startupNotify = true;
        startupWMClass = "mars";
      };
      yzn = pkgs.symlinkJoin {
        name = "yzn";
        paths = [yznCommand yznDesktop];
        postBuild = ''
          install -d "$out/libexec/yazelix-next"
          ln -s ${yznZellijConfig}/bin/yzn-zellij-config "$out/libexec/yazelix-next/yzn-zellij-config"
          install -D -m 644 ${yznConfigKdl} "$out/share/yazelix-next/config.kdl"
          install -D -m 644 ${yznMarsConfig}/config.toml "$out/share/yazelix-next/mars/config.toml"
          install -D -m 644 ${yznZellijLayout}/layout.kdl "$out/share/yazelix-next/layout.kdl"
          install -D -m 644 ${yznZellijLayout}/layout.swap.kdl "$out/share/yazelix-next/layout.swap.kdl"
          install -D -m 644 ${yznNuConfig}/config.nu "$out/share/yazelix-next/nu/config.nu"
          install -D -m 644 ${yznNuConfig}/env.nu "$out/share/yazelix-next/nu/env.nu"
          for icon in ${marsPackage}/share/icons/hicolor/*/apps/mars.png; do
            size="$(basename "$(dirname "$(dirname "$icon")")")"
            install -d "$out/share/icons/hicolor/$size/apps"
            ln -s "$icon" "$out/share/icons/hicolor/$size/apps/yzn.png"
          done
          install -d "$out/share/pixmaps"
          ln -s ${marsPackage}/share/pixmaps/mars.png "$out/share/pixmaps/yzn.png"
        '';
      };
    in {
      yazelix_helix = yazelixHelixPackage;
      yazelix_zellij = yazelixZellijPackage;
      inherit yzn;
      default = yzn;
    });

    checks = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      yzn = self.packages.${system}.yzn;
      yznContractsCheck = rustBinFor pkgs "yzn-contracts-check" ./checks/yzn-contracts.rs;
    in {
      inherit yzn;
      contracts = pkgs.runCommand "yzn-contracts" {} ''
        ${yznContractsCheck}/bin/yzn-contracts-check ${yzn} "$out"
      '';
    });

    apps = eachSystem (system: rec {
      yzn = {
        type = "app";
        program = "${self.packages.${system}.yzn}/bin/yzn";
      };
      default = yzn;
    });
  };
}
