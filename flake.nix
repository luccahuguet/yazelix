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
    yazelixZellijBar = {
      url = "github:luccahuguet/yazelix-zellij-bar";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixZellijPaneOrchestrator = {
      url = "github:luccahuguet/yazelix-zellij-pane-orchestrator";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ratconfig = {
      url = "github:luccahuguet/ratconfig";
      flake = false;
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
    yazelixZellijBar,
    yazelixZellijPaneOrchestrator,
    ratconfig,
    autoLayoutYazi,
    starshipYazi,
  }: let
    eachSystem = nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"];
    rustBinFor = pkgs: name: src: pkgs.runCommand name {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
      mkdir -p "$out/bin"
      rustc --edition=2024 ${src} -o "$out/bin/${name}"
    '';
    helperBridgeSessionEnv = ''
      if [ -z "''${YAZELIX_HELIX_BRIDGE_SESSION_ID:-}" ]; then
        YAZELIX_HELIX_BRIDGE_SESSION_ID="yzn-helper-$(date +%s)-$$"
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
      yznConfigSrc = pkgs.runCommand "yzn-config-src" {} ''
        mkdir -p "$out"
        cp -R ${pkgs.lib.cleanSource ./crates/yzn-config}/. "$out/"
        chmod -R u+w "$out"
        ln -s ${ratconfig} "$out/ratconfig"
        cp ${./config.toml} "$out/config.toml"
        cp ${yznMarsConfig}/config.toml "$out/mars.toml"
        substituteInPlace "$out/Cargo.toml" \
          --replace-fail '../../../ratconfig' './ratconfig'
        substituteInPlace "$out/src/catalog.rs" \
          --replace-fail '../../../config.toml' '../config.toml' \
          --replace-fail '../../../mars.toml' '../mars.toml'
      '';
      yznConfig = pkgs.rustPlatform.buildRustPackage {
        pname = "yzn-config";
        version = "0.1.0";
        src = yznConfigSrc;
        cargoLock.lockFile = ./crates/yzn-config/Cargo.lock;
      };
      yznShell = pkgs.writeShellApplication {
        name = "yzn-shell";
        text = ''
          shell_program="$(${yznConfig}/bin/yzn-config --get shell.program)"
          case "$shell_program" in
            nu) exec ${yznNuShell}/bin/yzn-nu "$@" ;;
            bash) exec ${pkgs.bashInteractive}/bin/bash -i "$@" ;;
            zsh) exec ${pkgs.zsh}/bin/zsh -i "$@" ;;
            fish) exec ${pkgs.fish}/bin/fish -i "$@" ;;
          esac
        '';
      };
      yznAgent = pkgs.writeShellApplication {
        name = "yzn-agent";
        text = ''
          if ! command -v codex >/dev/null 2>&1; then
            printf '%s\n' "Yazelix Next agent popup

codex is not available on PATH.
Install Codex or make \`codex\` executable on PATH before using Alt Shift L." >&2
            if [ -t 0 ]; then
              printf '\nPress Enter to close this popup...' >&2
              read -r _ || true
            fi
            exit 127
          fi

          exec codex resume "$@"
        '';
      };
      yznMenu = pkgs.writeShellApplication {
        name = "yzn-menu";
        text = ''
          printf '%s\n' 'Yazelix Next Menu

Commands
  yzn config        Open config UI
  yzn doctor        Check runtime setup
  yzn enter         Start managed runtime in this terminal
  yzn launch        Open Mars and start Yazelix
  yzn menu          Show this menu
  yzn sponsor       Open sponsor page or print URL
  yzn status        Show runtime status

Popups
  Alt Shift J       LazyGit
  Alt Shift K       Config
  Alt Shift L       Codex resume
  Alt Shift M       Menu

Workspace
  Ctrl p/t/n/q      Pane, tab, resize, quit
  Ctrl Alt h/l      Move tab left/right
  Ctrl Alt j/k      Move pane down/up
  Alt Shift h       Toggle Yazi sidebar layout
  Alt z             Yazi zoxide jump into editor'
        '';
      };
      yznMenuPopup = pkgs.writeShellApplication {
        name = "yzn-menu-popup";
        text = ''
          ${yznMenu}/bin/yzn-menu
          if [ -t 0 ]; then
            printf '\nPress Enter to close this popup...'
            read -r _ || true
          fi
        '';
      };
      yazelixZellijPopupPackage = yazelixZellijPopup.packages.${system}.yzpp;
      yazelixZellijBarPackage = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
      yazelixZellijPaneOrchestratorPackage =
        yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
      tokenusage = import ./packaging/tokenusage.nix {inherit pkgs;};
      yznZellijConfig = rustBin "yzn-zellij-config" ./runtime/yzn-zellij-config.rs;
      yazelixHelixPackage = yazelixHelix.packages.${system}.yazelix_helix;
      yznHelixConfig = pkgs.writeTextDir "config.toml" (builtins.readFile ./helix/config.toml);
      yznHelix = pkgs.writeShellApplication {
        name = "yzn-hx";
        runtimeInputs = [pkgs.coreutils];
        text = ''
          export YAZELIX_STATE_DIR="''${YAZELIX_STATE_DIR:-''${XDG_DATA_HOME:-''${HOME:-/tmp}/.local/share}/yazelix-next}"
          ${helperBridgeSessionEnv}
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
      yznConfigUi = pkgs.writeShellApplication {
        name = "yzn-config-ui";
        text = ''
          export YAZELIX_NEXT_EDITOR=${yznHelix}/bin/yzn-hx
          export EDITOR=$YAZELIX_NEXT_EDITOR
          export VISUAL=$YAZELIX_NEXT_EDITOR
          exec ${yznConfig}/bin/yzn-config "$@"
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
        install -D -m 644 ${./yazi/keymap.toml} "$out/keymap.toml"
        install -D -m 644 ${yznYaziToml} "$out/yazi.toml"
        install -D -m 644 ${yaziAssetsSelection}/yazelix_starship.toml "$out/yazelix_starship.toml"
        mkdir -p "$out/plugins"
        install -D -m 644 ${./yazi/plugins/sidebar-status.yazi/main.lua} "$out/plugins/sidebar-status.yazi/main.lua"
        install -D -m 644 ${./yazi/plugins/zoxide-editor.yazi/main.lua} "$out/plugins/zoxide-editor.yazi/main.lua"
        ln -s ${autoLayoutYazi} "$out/plugins/auto-layout.yazi"
        ln -s ${yaziAssetsSelection}/plugins/git.yazi "$out/plugins/git.yazi"
        ln -s ${starshipYazi} "$out/plugins/starship.yazi"
      '';
      yznYaziSrc = pkgs.replaceVars ./runtime/yzn-yazi.rs {
        yazi = "${pkgs.yazi}/bin/yazi";
        yznYaziConfig = "${yznYaziConfig}";
        yznOpen = "${yznOpenCore}/bin/yzn-open";
        zellij = "${yazelixZellijPackage}/bin/zellij";
        yznHelix = "${yznHelix}/bin/yzn-hx";
        yznConfig = "${yznConfig}/bin/yzn-config";
        pathPrefix = pkgs.lib.makeBinPath [pkgs.fzf pkgs.git pkgs.starship pkgs.zoxide];
      };
      yznYazi = rustBin "yzn-yazi" yznYaziSrc;
      yznRuntimeIdentity = pkgs.writeTextDir "runtime_identity.json" (builtins.toJSON {
        name = "Yazelix Next";
        version = "next";
      });
      defaultBarWidgets = ["editor" "shell" "term" "codex_usage" "cpu" "ram"];
      barRenderRequest = import ./packaging/bar-render-request.nix {
        inherit (pkgs) coreutils nushell;
        runtimeIdentity = yznRuntimeIdentity;
        zellijBar = yazelixZellijBarPackage;
      };
      yznBarRenderRequest =
        pkgs.writeText "yzn-bar-render-request.json" (builtins.toJSON (barRenderRequest defaultBarWidgets));
      yznBarRenderRequestTemplate =
        pkgs.writeText "yzn-bar-render-request-template.json" (builtins.toJSON (barRenderRequest "__YZN_BAR_WIDGET_TRAY__"));
      yznBarRender = pkgs.writeShellApplication {
        name = "yzn-bar-render";
        runtimeInputs = [pkgs.jq];
        text = ''
          ${yazelixZellijBarPackage}/${yazelixZellijBarPackage.widgetPath} render-yazelix-runtime --json "$1" \
            | jq -er '.plugin_block' \
            | ${pkgs.gnused}/bin/sed 's/YZX {command_version}/YZN/g'
        '';
      };
      yznBarKdl = pkgs.runCommand "yzn-zellij-bar.kdl" {} ''
        ${yznBarRender}/bin/yzn-bar-render "$(<${yznBarRenderRequest})" > "$out"
      '';
      yznLayoutKdl = pkgs.runCommand "layout.kdl" {} ''
        substitute ${./layout.kdl} "$out" \
          --replace-fail '@yazi@' '${yznYazi}/bin/yzn-yazi' \
          --replace-fail '@bar@' "$(<${yznBarKdl})"
      '';
      yznLayoutSwapKdl = pkgs.replaceVars ./layout.swap.kdl {
        yazi = "${yznYazi}/bin/yzn-yazi";
      };
      yznLayoutCheck = rustBin "yzn-layout-check" ./checks/zellij-layout.rs;
      yznZellijLayout = pkgs.runCommand "yzn-zellij-layout" {} ''
        ${yznLayoutCheck}/bin/yzn-layout-check ${yznLayoutKdl} ${yznLayoutSwapKdl}
        install -D -m 644 ${yznLayoutKdl} "$out/layout.kdl"
        install -D -m 644 ${yznLayoutSwapKdl} "$out/layout.swap.kdl"
      '';
      yznConfigKdl = pkgs.replaceVars ./config.kdl {
        yznShell = "${yznShell}/bin/yzn-shell";
        yzpp = "file:${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
        yznPaneOrchestrator = "file:${yazelixZellijPaneOrchestratorPackage}/${yazelixZellijPaneOrchestratorPackage.wasmPath}";
        yznAgent = "${yznAgent}/bin/yzn-agent";
        yznConfig = "${yznConfigUi}/bin/yzn-config-ui";
        yznMenu = "${yznMenuPopup}/bin/yzn-menu-popup";
        lazygit = "${pkgs.lazygit}/bin/lazygit";
        layout = "${yznZellijLayout}/layout.kdl";
      };
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
      yznCommandSrc = pkgs.replaceVars ./runtime/yzn.rs {
        yznConfigUi = "${yznConfigUi}/bin/yzn-config-ui";
        yznMenu = "${yznMenu}/bin/yzn-menu";
        zellij = "${yazelixZellijPackage}/bin/zellij";
        mars = "${marsPackage}/bin/mars";
        layout = "${yznZellijLayout}/layout.kdl";
        layoutTemplate = "${./layout.kdl}";
        layoutSwapTemplate = "${./layout.swap.kdl}";
        yznYazi = "${yznYazi}/bin/yzn-yazi";
        yznHelix = "${yznHelix}/bin/yzn-hx";
        yznConfig = "${yznConfig}/bin/yzn-config";
        yznMarsConfig = "${yznMarsConfig}";
        yznZellijConfig = "${yznZellijConfig}/bin/yzn-zellij-config";
        yznConfigKdl = "${yznConfigKdl}";
        yznBarRenderRequest = "${yznBarRenderRequestTemplate}";
        yznBarRender = "${yznBarRender}/bin/yzn-bar-render";
        yazelixZellijPopupWasm = "${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
        yazelixZellijBarWasm = "${yazelixZellijBarPackage}/share/yazelix_zellij_bar/zjstatus.wasm";
        yazelixZellijPaneOrchestratorWasm = "${yazelixZellijPaneOrchestratorPackage}/${yazelixZellijPaneOrchestratorPackage.wasmPath}";
        defaultBarWidgetsJson = builtins.toJSON defaultBarWidgets;
        pathPrefix = pkgs.lib.makeBinPath [pkgs.coreutils tokenusage];
      };
      yznCommand = rustBin "yzn" yznCommandSrc;
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
          ${yazelixZellijPackage}/bin/zellij --config ${yznConfigKdl} setup --check >/dev/null
          install -d "$out/libexec/yazelix-next"
          ln -s ${yznZellijConfig}/bin/yzn-zellij-config "$out/libexec/yazelix-next/yzn-zellij-config"
          ln -s ${yznConfig}/bin/yzn-config "$out/libexec/yazelix-next/yzn-config"
          install -D -m 644 ${yznConfigKdl} "$out/share/yazelix-next/config.kdl"
          install -D -m 644 ${yznRuntimeIdentity}/runtime_identity.json "$out/share/yazelix-next/runtime_identity.json"
          install -D -m 644 ${yznMarsConfig}/config.toml "$out/share/yazelix-next/mars/config.toml"
          install -D -m 644 ${./config.toml} "$out/share/yazelix-next/config.toml"
          install -D -m 644 ${yznZellijLayout}/layout.kdl "$out/share/yazelix-next/layout.kdl"
          install -D -m 644 ${yznZellijLayout}/layout.swap.kdl "$out/share/yazelix-next/layout.swap.kdl"
          install -D -m 644 ${yznYaziConfig}/init.lua "$out/share/yazelix-next/yazi/init.lua"
          install -D -m 644 ${yznYaziConfig}/keymap.toml "$out/share/yazelix-next/yazi/keymap.toml"
          install -D -m 644 ${yznYaziConfig}/plugins/zoxide-editor.yazi/main.lua "$out/share/yazelix-next/yazi/plugins/zoxide-editor.yazi/main.lua"
          install -D -m 644 ${yznYaziConfig}/yazi.toml "$out/share/yazelix-next/yazi/yazi.toml"
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
      yzn_yazi_materialization = pkgs.runCommand "yzn-yazi-materialization-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 --test ${./runtime/yzn-yazi.rs} -o yzn-yazi-materialization-check
        ./yzn-yazi-materialization-check
        touch "$out"
      '';
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
