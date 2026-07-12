{
  description = "Yazelix Nova";

  nixConfig = {
    extra-substituters = ["https://yazelix.cachix.org"];
    extra-trusted-public-keys = [
      "yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    mars = {
      url = "github:luccahuguet/mars";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixCursors = {
      url = "github:luccahuguet/yazelix-cursors";
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
    yazelixScreen = {
      url = "github:luccahuguet/yazelix-screen";
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
    home-manager,
    mars,
    yazelixCursors,
    yazelixZellij,
    yazelixHelix,
    yazelixZellijPopup,
    yazelixZellijBar,
    yazelixZellijPaneOrchestrator,
    yazelixScreen,
    ratconfig,
    autoLayoutYazi,
    starshipYazi,
  }: let
    novaVersion = "dev";
    compactNovaVersion = version:
      if version == "dev"
      then "NOVA DEV"
      else let
        parsed = builtins.match "([0-9]+)\\.([0-9]+)\\.[0-9]+(-beta\\.[0-9]+)?" version;
      in
        if parsed == null
        then throw "unsupported Nova version: ${version}"
        else "NOVA ${builtins.elemAt parsed 0}${if builtins.elemAt parsed 2 == null then ".${builtins.elemAt parsed 1}" else "β"}";
    novaBarLabel =
      assert compactNovaVersion "dev" == "NOVA DEV";
      assert compactNovaVersion "1.0.0-beta.1" == "NOVA 1β";
      assert compactNovaVersion "1.0.0" == "NOVA 1.0";
      compactNovaVersion novaVersion;
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    eachSystem = nixpkgs.lib.genAttrs supportedSystems;
    homeManagerModule = import ./home-manager/module.nix {
      defaultPackageFor = system: self.packages.${system}.yzn;
    };
    rustBinFor = pkgs: name: src: pkgs.runCommand name {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
      mkdir -p "$out/bin"
      rustc --edition=2024 ${src} -o "$out/bin/${name}"
    '';
  in {
    homeManagerModules.default = homeManagerModule;

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
        yznConfig = "${yznConfig}/bin/yzn-config";
      };
      yznNuShell = rustBin "yzn-nu" yznNuRs;
      yznConfigSrc = pkgs.runCommand "yzn-config-src" {} ''
        mkdir -p "$out"
        cp -R ${pkgs.lib.cleanSource ./crates/yzn-config}/. "$out/"
        chmod -R u+w "$out"
        ln -s ${ratconfig} "$out/ratconfig"
        ln -s ${yazelixCursors} "$out/yazelix-cursors"
        cp ${./config.toml} "$out/config.toml"
        cp ${./mars.toml} "$out/mars.toml"
        substituteInPlace "$out/Cargo.toml" \
          --replace-fail '../../../ratconfig' './ratconfig' \
          --replace-fail '../../../yazelix-cursors' './yazelix-cursors'
        substituteInPlace "$out/src/catalog.rs" \
          --replace-fail '../../../config.toml' '../config.toml' \
          --replace-fail '../../../mars.toml' '../mars.toml'
      '';
      yznConfig = pkgs.rustPlatform.buildRustPackage {
        pname = "yzn-config";
        version = "0.1.0";
        src = yznConfigSrc;
        cargoLock.lockFile = ./crates/yzn-config/Cargo.lock;
        YAZELIX_NIX_STORE_ROOT = builtins.storeDir;
      };
      yznShellSrc = pkgs.replaceVars ./shell/sh/yzn-shell.sh {
        yznConfig = "${yznConfig}/bin/yzn-config";
        yznNu = "${yznNuShell}/bin/yzn-nu";
        bash = "${pkgs.bashInteractive}/bin/bash";
        zsh = "${pkgs.zsh}/bin/zsh";
        fish = "${pkgs.fish}/bin/fish";
      };
      yznShell = pkgs.runCommand "yzn-shell" {} ''
        install -D -m 755 ${yznShellSrc} "$out/bin/yzn-shell"
        patchShebangs "$out/bin/yzn-shell"
      '';
      yznEnvSupervisor = pkgs.runCommand "yzn-env-supervisor" {} ''
        install -D -m 755 ${./shell/sh/yzn-env-supervisor.sh} "$out/bin/yzn-env-supervisor"
        patchShebangs "$out/bin/yzn-env-supervisor"
      '';
      yznAgent = rustBin "yzn-agent" ./runtime/yzn-agent.rs;
      yznMenuSrc = pkgs.replaceVars ./runtime/yzn-menu.rs {
        fzf = "${pkgs.fzf}/bin/fzf";
      };
      yznMenu = rustBin "yzn-menu" yznMenuSrc;
      yazelixZellijPopupPackage = yazelixZellijPopup.packages.${system}.yzpp;
      yazelixZellijBarPackage = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
      yazelixZellijPaneOrchestratorPackage =
        yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
      tokenusage = import ./packaging/tokenusage.nix {inherit pkgs;};
      yazelixScreenPackage = yazelixScreen.packages.${system}.yzs;
      yznWelcome = pkgs.writeShellApplication {
        name = "yzn-welcome";
        text = ''
          if [ "''${YZN_WELCOME_ENABLED:-true}" != false ]; then
            if ! YAZELIX_SCREEN_COMMAND_NAME='yzn screen' ${yazelixScreenPackage}/bin/yzs "''${YZN_WELCOME_STYLE:-random}" --duration-seconds "''${YZN_WELCOME_DURATION_SECONDS:-3}"; then
              printf 'yzn welcome: failed to render welcome screen\n' >&2
            fi
          fi
          if [ "$#" -eq 0 ]; then
            exit 0
          fi
          exec "$@"
        '';
      };
      yznZellijConfig = rustBin "yzn-zellij-config" ./runtime/yzn-zellij-config.rs;
      yazelixHelixPackage = yazelixHelix.packages.${system}.yazelix_helix;
      yznHelixConfig = pkgs.writeTextDir "config.toml" (builtins.readFile ./helix/config.toml);
      yznOpenTerminal = pkgs.writeShellApplication {
        name = "yzn-open-terminal";
        text = ''
          if [ "$#" -ne 1 ]; then
            printf '%s\n' 'usage: yzn-open-terminal <path>' >&2
            exit 64
          fi
          target="$1"
          if [ -d "$target" ]; then
            cwd="$target"
          else
            cwd="$(${pkgs.coreutils}/bin/dirname -- "$target")"
          fi
          exec ${yazelixZellijPackage}/bin/zellij action new-pane --cwd "$cwd"
        '';
      };
      yznHelixSteelConfig = pkgs.runCommand "yzn-helix-steel-config" {} ''
        mkdir -p "$out"
        cat > "$out/helix.scm" <<'EOF'
        ;; Yazelix Nova packaged Steel module.
        (provide yzn-new-shell)
        (require (only-in "helix/static.scm" cx->current-file get-helix-cwd))
        (require (only-in "helix/commands.scm" run-shell-command))
        (require (only-in "helix/misc.scm" set-error!))

        (define yazelix-single-quote "'")
        (define (yazelix-posix-quote value)
          (string-append
            yazelix-single-quote
            (string-replace
              value
              yazelix-single-quote
              (string-append yazelix-single-quote "\\" yazelix-single-quote yazelix-single-quote))
            yazelix-single-quote))

        (define (yzn-new-shell-command target)
          (string-append "\"${yznOpenTerminal}/bin/yzn-open-terminal\" " (yazelix-posix-quote target)))

        ;;@doc
        ;;Open a Yazelix terminal pane at the current Helix file or workspace.
        (define (yzn-new-shell)
          (let ([current-file (cx->current-file)]
                [current-workspace (get-helix-cwd)])
            (cond
              [(string? current-file)
               (run-shell-command (yzn-new-shell-command current-file))]
              [(string? current-workspace)
               (run-shell-command (yzn-new-shell-command current-workspace))]
              [else
               (set-error! "Yazelix could not resolve a target path for opening a shell")])))
        EOF
        cat > "$out/init.scm" <<'EOF'
        ;; Yazelix Nova packaged Steel init.
        EOF
      '';
      yznHelixSrc = pkgs.replaceVars ./shell/sh/yzn-helix.sh {
        date = "${pkgs.coreutils}/bin/date";
        hx = "${yazelixHelixPackage}/bin/hx";
        mkdir = "${pkgs.coreutils}/bin/mkdir";
        od = "${pkgs.coreutils}/bin/od";
        tr = "${pkgs.coreutils}/bin/tr";
        yznConfig = "${yznConfig}/bin/yzn-config";
        yznHelixConfig = "${yznHelixConfig}";
        yznHelixSteelConfig = "${yznHelixSteelConfig}";
      };
      yznHelix = pkgs.runCommand "yzn-hx" {} ''
        install -D -m 755 ${yznHelixSrc} "$out/bin/yzn-hx"
        ln -s yzn-hx "$out/bin/hx"
        patchShebangs "$out/bin/yzn-hx"
      '';
      yznTutorSrc = pkgs.runCommand "yzn-tutor-src" {} ''
        mkdir -p "$out"
        cp -R ${pkgs.lib.cleanSource ./crates/yzn-tutor}/. "$out/"
        chmod -R u+w "$out"
        substituteInPlace "$out/src/main.rs" \
          --replace-fail '@yznHelix@' '${yznHelix}/bin/yzn-hx' \
          --replace-fail '@nu@' '${pkgs.nushell}/bin/nu'
      '';
      yznTutor = pkgs.rustPlatform.buildRustPackage {
        pname = "yzn-tutor";
        version = "0.1.0";
        src = yznTutorSrc;
        cargoLock.lockFile = ./crates/yzn-tutor/Cargo.lock;
      };
      yznEditor = pkgs.writeShellApplication {
        name = "yzn-editor";
        text = ''
          fallback="''${YAZELIX_NEXT_EDITOR:-}"
          if [ -n "$fallback" ]; then
            editor="$(${yznConfig}/bin/yzn-config --get editor.command 2>/dev/null || printf %s "$fallback")"
          else
            editor="$(${yznConfig}/bin/yzn-config --get editor.command)"
          fi
          case "$editor" in
            yzn-hx|hx) editor=${yznHelix}/bin/yzn-hx ;;
          esac
          if ! command -v -- "$editor" >/dev/null 2>&1; then
            printf 'Yazelix editor command not found: %s. Set editor.command to one executable name or path without arguments.\n' "$editor" >&2
            exit 127
          fi
          export YAZELIX_HELIX_BRIDGE=0
          trap '[ -z "''${ZELLIJ:-}" ] || printf "\033]111\a"' EXIT
          command -- "$editor" "$@"
        '';
      };
      yznEditorEnv = ''
        export EDITOR=${yznEditor}/bin/yzn-editor
        export VISUAL=${yznEditor}/bin/yzn-editor
        export GIT_EDITOR=${yznEditor}/bin/yzn-editor
      '';
      yznConfigUi = pkgs.writeShellApplication {
        name = "yzn-config-ui";
        text = ''
          export YAZELIX_NEXT_EDITOR="''${YAZELIX_NEXT_EDITOR:-${yznHelix}/bin/yzn-hx}"
          ${yznEditorEnv}
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
        opener = "YZN_ZELLIJ=${yazelixZellijPackage}/bin/zellij ${yznOpenCore}/bin/yzn-open";
      };
      yznYaziConfig = pkgs.runCommand "yzn-yazi-config" {} ''
        install -D -m 644 ${./yazi/init.lua} "$out/init.lua"
        install -D -m 644 ${./yazi/keymap.toml} "$out/keymap.toml"
        install -D -m 644 ${yznYaziToml} "$out/yazi.toml"
        install -D -m 644 ${yaziAssetsSelection}/yazelix_starship.toml "$out/yazelix_starship.toml"
        mkdir -p "$out/plugins"
        install -D -m 644 ${./yazi/plugins/sidebar-state.yazi/main.lua} "$out/plugins/sidebar-state.yazi/main.lua"
        install -D -m 644 ${./yazi/plugins/sidebar-status.yazi/main.lua} "$out/plugins/sidebar-status.yazi/main.lua"
        install -D -m 644 ${./yazi/plugins/zoxide-editor.yazi/main.lua} "$out/plugins/zoxide-editor.yazi/main.lua"
        ln -s ${autoLayoutYazi} "$out/plugins/auto-layout.yazi"
        ln -s ${yaziAssetsSelection}/plugins/git.yazi "$out/plugins/git.yazi"
        ln -s ${starshipYazi} "$out/plugins/starship.yazi"
      '';
      yznYaziMaterializer = pkgs.rustPlatform.buildRustPackage {
        pname = "yzn-yazi-config";
        version = "0.1.0";
        src = ./crates/yzn-yazi-config;
        cargoLock.lockFile = ./crates/yzn-yazi-config/Cargo.lock;
      };
      yznYaziSrc = pkgs.replaceVars ./runtime/yzn-yazi.rs {
        yazi = "${pkgs.yazi}/bin/yazi";
        yznYaziConfig = "${yznYaziConfig}";
        yznYaziMaterializer = "${yznYaziMaterializer}/bin/yzn-yazi-config";
        yznOpen = "${yznOpenCore}/bin/yzn-open";
        zellij = "${yazelixZellijPackage}/bin/zellij";
        yznHelix = "${yznHelix}/bin/yzn-hx";
        yznEditor = "${yznEditor}/bin/yzn-editor";
        yznConfig = "${yznConfig}/bin/yzn-config";
        pathPrefix = pkgs.lib.makeBinPath [pkgs.fzf pkgs.git pkgs.starship pkgs.zoxide];
      };
      yznYazi = rustBin "yzn-yazi" yznYaziSrc;
      yznRuntimeIdentity = pkgs.writeTextDir "runtime_identity.json" (builtins.toJSON {
        name = "Yazelix Nova";
        version = novaVersion;
      });
      defaultConfig = builtins.fromTOML (builtins.readFile ./config.toml);
      defaultBarWidgets = defaultConfig.bar.widgets;
      defaultShellProgram = defaultConfig.shell.program;
      defaultPopupSideMargin = toString defaultConfig.popup.side_margin;
      defaultPopupVerticalMargin = toString defaultConfig.popup.vertical_margin;
      barRenderRequest = import ./packaging/bar-render-request.nix {
        inherit (pkgs) coreutils nushell;
        runtimeIdentity = yznRuntimeIdentity;
        zellijBar = yazelixZellijBarPackage;
      };
      yznBarRenderRequest =
        pkgs.writeText "yzn-bar-render-request.json" (builtins.toJSON (barRenderRequest {
          widgetTray = defaultBarWidgets;
          shellLabel = defaultShellProgram;
        }));
      yznBarRenderRequestTemplate =
        pkgs.writeText "yzn-bar-render-request-template.json" (builtins.toJSON (barRenderRequest {
          widgetTray = "__YZN_BAR_WIDGET_TRAY__";
          shellLabel = "__YZN_SHELL_LABEL__";
        }));
      yznBarRender = pkgs.writeShellApplication {
        name = "yzn-bar-render";
        runtimeInputs = [pkgs.jq];
        text = ''
          ${yazelixZellijBarPackage}/${yazelixZellijBarPackage.widgetPath} render-yazelix-runtime --json "$1" \
            | jq -er '.plugin_block' \
            | ${pkgs.gnused}/bin/sed 's/YZX {command_version}/${novaBarLabel}/g'
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
        ${yznLayoutCheck}/bin/yzn-layout-check ${yznLayoutKdl} ${yznLayoutSwapKdl} ${pkgs.lib.escapeShellArg novaBarLabel}
        install -D -m 644 ${yznLayoutKdl} "$out/layout.kdl"
        install -D -m 644 ${yznLayoutSwapKdl} "$out/layout.swap.kdl"
      '';
      yznLazyGitConfig = pkgs.writeText "yzn-lazygit.yml" ''
        os:
          edit: '${yznEditor}/bin/yzn-editor {{filename}}'
          editAtLine: '${yznEditor}/bin/yzn-editor {{filename}}'
          editAtLineAndWait: '${yznEditor}/bin/yzn-editor {{filename}}'
          editInTerminal: true
          openDirInEditor: '${yznEditor}/bin/yzn-editor {{dir}}'
      '';
      yznGit = pkgs.writeShellApplication {
        name = "yzn-git";
        text = ''
          ${yznEditorEnv}
          if [ -z "''${LG_CONFIG_FILE:-}" ]; then
            config_file="$(${pkgs.lazygit}/bin/lazygit --print-config-dir)/config.yml"
            [ ! -f "$config_file" ] || LG_CONFIG_FILE="$config_file"
          fi
          export LG_CONFIG_FILE="''${LG_CONFIG_FILE:+$LG_CONFIG_FILE,}${yznLazyGitConfig}"
          exec ${pkgs.lazygit}/bin/lazygit "$@"
        '';
      };
      yznConfigKdl = pkgs.replaceVars ./config.kdl {
        yznShell = "${yznShell}/bin/yzn-shell";
        yzpp = "file:${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
        yznPaneOrchestrator = "file:${yazelixZellijPaneOrchestratorPackage}/${yazelixZellijPaneOrchestratorPackage.wasmPath}";
        yznAgent = "${yznAgent}/bin/yzn-agent";
        configKey = defaultConfig.keybindings.config;
        agentKey = defaultConfig.keybindings.agent;
        gitKey = defaultConfig.keybindings.git;
        menuKey = defaultConfig.keybindings.menu;
        inherit defaultPopupSideMargin defaultPopupVerticalMargin;
        yznConfig = "${yznConfigUi}/bin/yzn-config-ui";
        yznMenu = "${yznMenu}/bin/yzn-menu";
        yznSidebarRefresh = "${yznOpenCore}/bin/yzn-sidebar-refresh";
        git = "${yznGit}/bin/yzn-git";
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
      mkYznCommand = withMars: let
        packageVariant = if withMars then "full" else "runtime";
        marsPath = if withMars then "${marsPackage}/bin/mars" else "";
        main = pkgs.replaceVars ./runtime/yzn/main.rs {
          yznConfigUi = "${yznConfigUi}/bin/yzn-config-ui";
          yznMenu = "${yznMenu}/bin/yzn-menu";
          yznTutor = "${yznTutor}/bin/yzn-tutor";
          yznScreen = "${yazelixScreenPackage}/bin/yzs";
          yznWelcome = "${yznWelcome}/bin/yzn-welcome";
          yznShell = "${yznShell}/bin/yzn-shell";
          yznEnvSupervisor = "${yznEnvSupervisor}/bin/yzn-env-supervisor";
          zellij = "${yazelixZellijPackage}/bin/zellij";
          mars = marsPath;
          layout = "${yznZellijLayout}/layout.kdl";
          layoutTemplate = "${./layout.kdl}";
          layoutSwapTemplate = "${./layout.swap.kdl}";
          yznAgent = "${yznAgent}/bin/yzn-agent";
          yznYazi = "${yznYazi}/bin/yzn-yazi";
          yznHelix = "${yznHelix}/bin/yzn-hx";
          yznEditor = "${yznEditor}/bin/yzn-editor";
          yznConfig = "${yznConfig}/bin/yzn-config";
          yznMarsConfig = if withMars then "${yznMarsConfig}" else "";
          yznZellijConfig = "${yznZellijConfig}/bin/yzn-zellij-config";
          yznConfigKdl = "${yznConfigKdl}";
          yznReveal = "${yznOpenCore}/bin/yzn-reveal";
          yznSidebarRefresh = "${yznOpenCore}/bin/yzn-sidebar-refresh";
          yznYa = "${pkgs.yazi}/bin/ya";
          yznBarRenderRequest = "${yznBarRenderRequestTemplate}";
          yznBarRender = "${yznBarRender}/bin/yzn-bar-render";
          yazelixZellijPopupWasm = "${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
          yazelixZellijBarWasm = "${yazelixZellijBarPackage}/share/yazelix_zellij_bar/zjstatus.wasm";
          yazelixZellijPaneOrchestratorWasm = "${yazelixZellijPaneOrchestratorPackage}/${yazelixZellijPaneOrchestratorPackage.wasmPath}";
          defaultBarWidgetsJson = builtins.toJSON defaultBarWidgets;
          inherit defaultShellProgram;
          defaultConfigKeybinding = defaultConfig.keybindings.config;
          defaultAgentKeybinding = defaultConfig.keybindings.agent;
          defaultGitKeybinding = defaultConfig.keybindings.git;
          defaultMenuKeybinding = defaultConfig.keybindings.menu;
          inherit defaultPopupSideMargin defaultPopupVerticalMargin;
          version = novaVersion;
          pathPrefix = pkgs.lib.makeBinPath [
            pkgs.coreutils
            pkgs.git
            pkgs.lazygit
            tokenusage
            yznHelix
          ];
        };
        src = pkgs.runCommand "yzn-command-${packageVariant}-src" {} ''
          mkdir -p "$out"
          cp -R ${pkgs.lib.cleanSource ./runtime/yzn}/. "$out/"
          chmod -R u+w "$out"
          cp ${main} "$out/main.rs"
        '';
      in
        rustBin "yzn" "${src}/main.rs";
      mkYzn = {
        name,
        withMars ? false,
      }: let
        command = mkYznCommand withMars;
        desktop = pkgs.makeDesktopItem {
          name = "yzn";
          desktopName = "Yazelix Nova";
          genericName = "Terminal Emulator";
          comment = "Open Yazelix Nova";
          exec = "${command}/bin/yzn launch";
          icon = "yzn";
          terminal = false;
          categories = ["System" "TerminalEmulator"];
          startupNotify = true;
          startupWMClass = "mars";
        };
      in
        pkgs.symlinkJoin {
          inherit name;
          paths = [command] ++ pkgs.lib.optional withMars desktop;
          postBuild =
            ''
              ${yazelixZellijPackage}/bin/zellij --config ${yznConfigKdl} setup --check >/dev/null
              install -d "$out/libexec/yazelix-next"
              ln -s ${yznZellijConfig}/bin/yzn-zellij-config "$out/libexec/yazelix-next/yzn-zellij-config"
              ln -s ${yznConfig}/bin/yzn-config "$out/libexec/yazelix-next/yzn-config"
              ln -s ${yznTutor}/bin/yzn-tutor "$out/libexec/yazelix-next/yzn-tutor"
              install -D -m 644 ${yznConfigKdl} "$out/share/yazelix-next/config.kdl"
              install -D -m 644 ${yznRuntimeIdentity}/runtime_identity.json "$out/share/yazelix-next/runtime_identity.json"
              install -D -m 644 ${yazelixCursors}/yazelix_cursors_default.toml "$out/share/yazelix-next/cursors.toml"
              install -D -m 644 ${./config.toml} "$out/share/yazelix-next/config.toml"
              install -D -m 644 ${yznZellijLayout}/layout.kdl "$out/share/yazelix-next/layout.kdl"
              install -D -m 644 ${yznZellijLayout}/layout.swap.kdl "$out/share/yazelix-next/layout.swap.kdl"
              install -D -m 644 ${yznYaziConfig}/init.lua "$out/share/yazelix-next/yazi/init.lua"
              install -D -m 644 ${yznYaziConfig}/keymap.toml "$out/share/yazelix-next/yazi/keymap.toml"
              install -D -m 644 ${yznYaziConfig}/plugins/sidebar-state.yazi/main.lua "$out/share/yazelix-next/yazi/plugins/sidebar-state.yazi/main.lua"
              install -D -m 644 ${yznYaziConfig}/plugins/zoxide-editor.yazi/main.lua "$out/share/yazelix-next/yazi/plugins/zoxide-editor.yazi/main.lua"
              ln -s ${yznYaziConfig}/plugins/git.yazi "$out/share/yazelix-next/yazi/plugins/git.yazi"
              install -D -m 644 ${yznYaziConfig}/yazi.toml "$out/share/yazelix-next/yazi/yazi.toml"
              install -D -m 644 ${yznNuConfig}/config.nu "$out/share/yazelix-next/nu/config.nu"
              install -D -m 644 ${yznNuConfig}/env.nu "$out/share/yazelix-next/nu/env.nu"
            ''
            + pkgs.lib.optionalString withMars ''
              install -D -m 644 ${yznMarsConfig}/config.toml "$out/share/yazelix-next/mars/config.toml"
              for icon in ${marsPackage}/share/icons/hicolor/*/apps/mars.png; do
                size="$(basename "$(dirname "$(dirname "$icon")")")"
                install -d "$out/share/icons/hicolor/$size/apps"
                ln -s "$icon" "$out/share/icons/hicolor/$size/apps/yzn.png"
              done
              install -d "$out/share/pixmaps"
              ln -s ${marsPackage}/share/pixmaps/mars.png "$out/share/pixmaps/yzn.png"
            '';
          meta.platforms = supportedSystems;
        };
      yzn = mkYzn {
        name = "yzn";
        withMars = true;
      };
      yznRuntime = mkYzn {name = "yzn-runtime";};
    in {
      yazelix_helix = yazelixHelixPackage;
      yazelix_zellij = yazelixZellijPackage;
      inherit yzn;
      runtime = yznRuntime;
      default = yzn;
    });

    checks = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      yzn = self.packages.${system}.yzn;
      yznRuntime = self.packages.${system}.runtime;
      marsPackage = mars.packages.${system}.mars;
      runtimeClosure = pkgs.closureInfo {rootPaths = [yznRuntime];};
      yznYaziMaterializer = pkgs.rustPlatform.buildRustPackage {
        pname = "yzn-yazi-config";
        version = "0.1.0";
        src = ./crates/yzn-yazi-config;
        cargoLock.lockFile = ./crates/yzn-yazi-config/Cargo.lock;
      };
      checksSrc = pkgs.lib.cleanSource ./checks;
      yznContractsCheck = rustBinFor pkgs "yzn-contracts-check" "${checksSrc}/yzn-contracts.rs";
      helixContractsCheck = rustBinFor pkgs "helix-contracts-check" "${checksSrc}/helix-contracts.rs";
      fakeYazelix = pkgs.runCommand "fake-yazelix-hm-package" {} ''
        mkdir -p "$out/bin" "$out/share/applications"
        cat > "$out/bin/yzn" <<'EOF'
        #!${pkgs.runtimeShell}
        printf '%s\n' fake-yazelix
        EOF
        chmod 755 "$out/bin/yzn"
        cat > "$out/share/applications/yzn.desktop" <<'EOF'
        [Desktop Entry]
        Type=Application
        Name=Fake Yazelix
        Exec=yzn
        EOF
      '';
      fakeHelixLanguages = pkgs.writeText "hm-helix-languages.toml" ''
        [[language]]
        name = "nix"
      '';
      fakeCursors = pkgs.writeText "hm-cursors.toml" ''
        enabled_cursors = ["reef"]
        [settings]
        trail = "reef"
      '';
      homeManagerConfiguration = module:
        home-manager.lib.homeManagerConfiguration {
          inherit pkgs;
          modules = [
            self.homeManagerModules.default
            {
              home.username = "yzn-test";
              home.homeDirectory = "/tmp/yzn-test-home";
              home.stateVersion = "25.05";
              manual.manpages.enable = false;
              programs.yazelix.enable = true;
            }
            module
          ];
        };
      homeManagerDefault = homeManagerConfiguration {};
      homeManagerOverride = homeManagerConfiguration {
        programs.yazelix.package = fakeYazelix;
      };
      homeManagerRuntime = homeManagerConfiguration {
        programs.yazelix.package = yznRuntime;
      };
      homeManagerConfigFiles = homeManagerConfiguration {
        programs.yazelix.config = {
          settings = {
            shell.program = "fish";
            welcome.enabled = false;
            keybindings.config = "Alt Shift C";
            keybindings.agent = "Alt Shift A";
            keybindings.git = "Alt Shift G";
            keybindings.menu = "Alt Shift U";
            bar.widgets = ["editor" "shell"];
          };
          cursors.source = fakeCursors;
          mars.text = "[window]\nwidth = 1200\n";
          zellij.text = "pane_frames false\n";
          starship.text = "format = \"::\"\n";
          helix.config.text = "[editor]\nline-number = \"relative\"\n";
          helix.languages.source = fakeHelixLanguages;
          helix.module.text = "(provide yzn-test)\n";
          helix.init.text = ";; init\n";
          yazi.config.text = "[mgr]\nshow_hidden = true\n";
          yazi.init.text = "-- init\n";
          yazi.keymap.text = "[manager]\n";
          yazi.package.text = "[plugin]\ndeps = []\n";
          yazi.theme.text = "[flavor]\ndark = \"example\"\n";
          nu.env.text = "# env\n";
          nu.config.text = "# config\n";
        };
      };
    in {
      inherit yzn;
      home_manager = pkgs.runCommand "yzn-home-manager-check" {} ''
        default_path="${homeManagerDefault.activationPackage}/home-path"
        override_path="${homeManagerOverride.activationPackage}/home-path"
        runtime_path="${homeManagerRuntime.activationPackage}/home-path"
        hm_yzn="${homeManagerConfigFiles.activationPackage}/home-path/bin/yzn"
        config_files="${homeManagerConfigFiles.activationPackage}/home-files/.config/yazelix-next"

        test -x "$default_path/bin/yzn"
        test -f "$default_path/share/applications/yzn.desktop"
        grep -q 'Yazelix Nova' "$default_path/share/applications/yzn.desktop"

        test -x "$override_path/bin/yzn"
        test "$("$override_path/bin/yzn")" = fake-yazelix
        grep -q 'Fake Yazelix' "$override_path/share/applications/yzn.desktop"

        test -x "$runtime_path/bin/yzn"
        test ! -e "$runtime_path/share/applications/yzn.desktop"

        if [ -e "${homeManagerDefault.activationPackage}/home-files/.config/yazelix-next" ]; then
          printf '%s\n' 'Home Manager v1 must not generate Yazelix runtime config files' >&2
          exit 1
        fi
        grep -q 'program = "fish"' "$config_files/config.toml"
        ! grep -q 'command = "yzn-hx"' "$config_files/config.toml"
        grep -q 'enabled = false' "$config_files/config.toml"
        ! grep -q 'style = "random"' "$config_files/config.toml"
        grep -q 'config = "Alt Shift C"' "$config_files/config.toml"
        grep -q 'agent = "Alt Shift A"' "$config_files/config.toml"
        grep -q 'git = "Alt Shift G"' "$config_files/config.toml"
        grep -q 'menu = "Alt Shift U"' "$config_files/config.toml"
        ! grep -q 'ratconfig' "$config_files/config.toml"
        grep -q 'trail = "reef"' "$config_files/cursors.toml"
        test -L "$config_files/cursors.toml"
        case "$(readlink "$config_files/cursors.toml")" in
          /nix/store/*) ;;
          *) printf '%s\n' 'Home Manager cursor source is not store-backed' >&2; exit 1 ;;
        esac
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get shell.program)" = fish
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get editor.command)" = yzn-hx
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get agent.command)" = auto
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get agent.args)" = "[]"
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get keybindings.config)" = "Alt Shift C"
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get keybindings.agent)" = "Alt Shift A"
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get keybindings.git)" = "Alt Shift G"
        test "$(YAZELIX_NEXT_CONFIG_HOME="$config_files" ${yzn}/libexec/yazelix-next/yzn-config --get keybindings.menu)" = "Alt Shift U"
        grep -q 'width = 1200' "$config_files/mars/config.toml"
        grep -q 'pane_frames false' "$config_files/zellij/config.kdl"
        grep -q 'format = "::"' "$config_files/starship.toml"
        grep -q 'line-number = "relative"' "$config_files/helix/config.toml"
        grep -q 'name = "nix"' "$config_files/helix/languages.toml"
        grep -q '(provide yzn-test)' "$config_files/helix/helix.scm"
        grep -q 'show_hidden = true' "$config_files/yazi/yazi.toml"
        grep -q -- '-- init' "$config_files/yazi/init.lua"
        grep -q 'deps = \[\]' "$config_files/yazi/package.toml"
        grep -q 'dark = "example"' "$config_files/yazi/theme.toml"
        grep -q '# config' "$config_files/nu/config.nu"

        export HOME="$TMPDIR/hm-yzn-home"
        runtime_config="$TMPDIR/hm-yzn-config"
        cp -R "$config_files" "$runtime_config"
        chmod -R u+w "$runtime_config"
        export YAZELIX_NEXT_CONFIG_HOME="$runtime_config"
        export YAZELIX_STATE_DIR="$TMPDIR/hm-yzn-state"
        export XDG_DATA_HOME="$TMPDIR/hm-yzn-data"
        mkdir -p "$HOME" "$YAZELIX_STATE_DIR" "$XDG_DATA_HOME"

        "$hm_yzn" help > help
        "$hm_yzn" status > status
        "$hm_yzn" doctor > doctor
        "$hm_yzn" tutor list > tutor-list
        grep -q 'Usage:' help
        grep -q 'Yazelix Nova status' status
        grep -q "config home: $runtime_config" status
        grep -q "state dir: $YAZELIX_STATE_DIR" status
        grep -q 'shell: fish' status
        grep -q 'welcome enabled: false' status
        grep -q 'Yazelix Nova doctor' doctor
        grep -q "ok config home: $runtime_config" doctor
        grep -q 'ok shell.program: fish' doctor
        grep -q 'Yazelix Nova tutor lessons' tutor-list
        touch "$out"
      '';
      yzn_yazi_materialization = pkgs.runCommand "yzn-yazi-materialization-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 --test ${./runtime/yzn-yazi.rs} -o yzn-yazi-materialization-check
        ./yzn-yazi-materialization-check

        user="$TMPDIR/yazi-user"
        state="$TMPDIR/yazi-state"
        mkdir -p "$user/plugins"
        ln -s ${pkgs.yaziPlugins.smart-enter} "$user/plugins/smart-enter.yazi"
        printf '%s\n' 'require("smart-enter"):setup { open_multi = false }' > "$user/init.lua"
        printf '%s\n' '[[mgr.prepend_keymap]]' 'on = "l"' 'run = "plugin smart-enter"' > "$user/keymap.toml"

        runtime="$(${yznYaziMaterializer}/bin/yzn-yazi-config ${yzn}/share/yazelix-next/yazi "$user" "$state")"
        YAZI_CONFIG_HOME="$runtime" ${pkgs.yazi}/bin/yazi --debug > yazi-debug
        test -f "$runtime/plugins/smart-enter.yazi/main.lua"
        grep -q 'require("smart-enter")' "$runtime/init.lua"
        grep -q 'plugin smart-enter' "$runtime/keymap.toml"
        grep -q 'yzn-open' yazi-debug
        touch "$out"
      '';
      yzn_launcher_unit = pkgs.runCommand "yzn-launcher-unit-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 --test ${pkgs.lib.cleanSource ./runtime/yzn}/main.rs -o yzn-launcher-unit-check
        ./yzn-launcher-unit-check
        touch "$out"
      '';
      zellij_sidecar_guard_parity = pkgs.runCommand "zellij-sidecar-guard-parity-check" {} ''
        extract_array() {
          file="$1"
          name="$2"
          awk -v name="$name" '
            index($0, name) { in_array = 1; next }
            in_array && /\];/ { exit }
            in_array {
              line = $0
              if (sub(/^[[:space:]]*"/, "", line)) {
                sub(/".*$/, "", line)
                print line
              }
            }
          ' "$file" | sort
        }

        extract_array ${./runtime/yzn-zellij-config.rs} FORBIDDEN > runtime
        extract_array ${./crates/yzn-config/src/catalog.rs} ZELLIJ_FORBIDDEN_TOP_LEVEL > config_ui
        diff -u runtime config_ui
        grep -qx default_shell runtime
        grep -qx env runtime
        touch "$out"
      '';
      key_reference_parity = pkgs.runCommand "key-reference-parity-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 ${./checks/key-reference-parity.rs} -o key-reference-parity-check
        ./key-reference-parity-check ${./crates/yzn-config/src/catalog.rs} ${yzn}/share/yazelix-next/config.kdl ${./crates/yzn-tutor/src/main.rs}
        touch "$out"
      '';
      contracts = pkgs.runCommand "yzn-contracts" {} ''
        ${yznContractsCheck}/bin/yzn-contracts-check ${yzn} ${pkgs.git}/bin/git ${pkgs.jq}/bin/jq "$out"
      '';
      runtime_contracts = pkgs.runCommand "yzn-runtime-contracts" {} ''
        test -x ${yznRuntime}/bin/yzn
        test ! -e ${yznRuntime}/share/applications/yzn.desktop
        ! grep -Fx ${marsPackage} ${runtimeClosure}/store-paths
        ! grep -E '/[^/]*-rio-[^/]*$' ${runtimeClosure}/store-paths

        export HOME="$TMPDIR/home"
        export YAZELIX_NEXT_CONFIG_HOME="$TMPDIR/config"
        export YAZELIX_STATE_DIR="$TMPDIR/state"
        export XDG_DATA_HOME="$TMPDIR/data"
        mkdir -p "$HOME" "$YAZELIX_NEXT_CONFIG_HOME" "$YAZELIX_STATE_DIR" "$XDG_DATA_HOME"
        printf '%s\n' '[welcome]' 'enabled = false' > "$YAZELIX_NEXT_CONFIG_HOME/config.toml"

        ${yznRuntime}/bin/yzn status --json > status.json
        test "$(${pkgs.jq}/bin/jq -r .package status.json)" = runtime
        ${yznRuntime}/bin/yzn status > status
        grep -q '^package: runtime$' status
        grep -q '^mars config: not included$' status
        ${yznRuntime}/bin/yzn doctor > doctor
        grep -q '^ok mars: not included$' doctor
        if ${yznRuntime}/bin/yzn launch 2> launch-error; then
          printf '%s\n' 'Mars-free runtime launch unexpectedly succeeded' >&2
          exit 1
        fi
        grep -q 'launch is unavailable in the Mars-free runtime package' launch-error
        ${yznRuntime}/bin/yzn enter --version > enter-version
        grep -q '^zellij ' enter-version
        touch "$out"
      '';
      helix_contracts = pkgs.runCommand "yzn-helix-contracts" {} ''
        ${helixContractsCheck}/bin/helix-contracts-check ${yzn} "$out"
      '';
    });

    apps = eachSystem (system: rec {
      yzn = {
        type = "app";
        program = "${self.packages.${system}.yzn}/bin/yzn";
      };
      runtime = {
        type = "app";
        program = "${self.packages.${system}.runtime}/bin/yzn";
      };
      default = yzn;
    });
  };
}
