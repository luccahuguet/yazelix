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
      inputs.zjstatus.follows = "zjstatus";
    };
    yazelixZellijPaneOrchestrator = {
      url = "github:luccahuguet/yazelix-zellij-pane-orchestrator";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixScreen = {
      url = "github:luccahuguet/yazelix-screen";
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
    zjstatus = {
      url = "github:luccahuguet/zjstatus/yazelix-tab-activity-pipe";
      inputs.nixpkgs.follows = "nixpkgs";
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
    autoLayoutYazi,
    starshipYazi,
    zjstatus,
  }: let
    novaVersion = "1.0.0-beta.2";
    compactNovaVersion = version:
      if version == "dev"
      then "NOVA DEV"
      else let
        parsed = builtins.match "([0-9]+)\\.([0-9]+)\\.[0-9]+(-beta\\.([0-9]+))?" version;
      in
        if parsed == null
        then throw "unsupported Nova version: ${version}"
        else if builtins.elemAt parsed 2 == null
        then "NOVA ${builtins.elemAt parsed 0}.${builtins.elemAt parsed 1}"
        else "NOVA β${builtins.elemAt parsed 3}";
    novaBarLabel =
      assert compactNovaVersion "dev" == "NOVA DEV";
      assert compactNovaVersion "1.0.0-beta.1" == "NOVA β1";
      assert compactNovaVersion "1.0.0-beta.12" == "NOVA β12";
      assert compactNovaVersion "1.0.0" == "NOVA 1.0";
      compactNovaVersion novaVersion;
    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];
    eachSystem = nixpkgs.lib.genAttrs supportedSystems;
    homeManagerModule = import ./home-manager/module.nix {
      defaultPackageFor = system: self.packages.${system}.yazelix;
    };
    rustBinFor = pkgs: name: src: pkgs.runCommand name {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
      mkdir -p "$out/bin"
      rustc --edition=2024 ${src} -o "$out/bin/${name}"
    '';
    yzxYaziMaterializerFor = pkgs:
      pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-yazi-config";
        version = "0.1.0";
        src = ./crates/yzx-yazi-config;
        cargoLock.lockFile = ./crates/yzx-yazi-config/Cargo.lock;
      };
  in {
    homeManagerModules.default = homeManagerModule;

    packages = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      rustBin = rustBinFor pkgs;
      marsPackage = mars.packages.${system}.mars;
      yzxMarsToml = pkgs.replaceVars ./defaults/mars/config.toml {
        jetbrainsMonoDir = "${pkgs.jetbrains-mono}/share/fonts/truetype";
        symbolsNerdDir = "${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols";
        notoSymbolsDir = "${pkgs.noto-fonts}/share/fonts/noto";
        notoEmojiDir = "${pkgs.noto-fonts-color-emoji}/share/fonts/noto";
      };
      yzxMarsConfig = pkgs.runCommand "yzx-mars-config" {} ''
        install -D -m 644 ${yzxMarsToml} "$out/config.toml"
      '';
      yzxCarapaceInit = pkgs.runCommand "yzx-carapace-init" {} ''
        ${pkgs.carapace}/bin/carapace _carapace nushell > "$out"
      '';
      yzxZoxideInit = pkgs.runCommand "yzx-zoxide-init" {} ''
        ${pkgs.zoxide}/bin/zoxide init nushell > "$out"
      '';
      yzxNuConfigNu = pkgs.replaceVars ./defaults/nu/config.nu {
        carapaceInit = "${yzxCarapaceInit}";
        starship = "${pkgs.starship}/bin/starship";
        zoxideInit = "${yzxZoxideInit}";
      };
      yzxNuConfig = pkgs.runCommand "yzx-nu-config" {} ''
        install -D -m 644 ${yzxNuConfigNu} "$out/config.nu"
        install -D -m 644 ${./defaults/nu/env.nu} "$out/env.nu"
      '';
      yzxNuRs = pkgs.replaceVars ./runtime/yzx-nu.rs {
        nu = "${pkgs.nushell}/bin/nu";
        packagedNu = "${yzxNuConfig}";
        pathPrefix = pkgs.lib.makeBinPath [pkgs.nushell pkgs.starship pkgs.carapace pkgs.zoxide];
        yzxConfig = "${yzxConfig}/bin/yzx-config";
      };
      yzxNuShell = rustBin "yzx-nu" yzxNuRs;
      yzxAgent = rustBin "yzx-agent" ./runtime/yzx-agent.rs;
      yzxConfigSrc = pkgs.runCommand "yzx-config-src" {} ''
        mkdir -p "$out"
        cp -R ${pkgs.lib.cleanSource ./crates/yzx-config}/. "$out/"
        chmod -R u+w "$out"
        ln -s ${yazelixCursors} "$out/yazelix-cursors"
        cp ${./defaults/config.toml} "$out/config.toml"
        cp ${./defaults/mars/config.toml} "$out/mars.toml"
        substituteInPlace "$out/Cargo.toml" \
          --replace-fail '../../../yazelix-cursors' './yazelix-cursors'
        substituteInPlace "$out/src/catalog.rs" \
          --replace-fail '../../../defaults/config.toml' '../config.toml' \
          --replace-fail '../../../defaults/mars/config.toml' '../mars.toml'
      '';
      yzxConfig = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-config";
        version = "0.1.0";
        src = yzxConfigSrc;
        cargoLock = {
          lockFile = ./crates/yzx-config/Cargo.lock;
          outputHashes."ratconfig-6.0.0" = "sha256-9pZHUUTO5+pcDGlS8Gpl++xzdAHXf+GzIlYMqOxzwz0=";
        };
        YAZELIX_NIX_STORE_ROOT = builtins.storeDir;
        YAZELIX_PACKAGED_YAZI = yzxYaziConfig;
        YAZELIX_AGENT_LAUNCHER = "${yzxAgent}/bin/yzx-agent";
      };
      yzxShellSrc = pkgs.replaceVars ./runtime/yzx-shell.sh {
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        yzxNu = "${yzxNuShell}/bin/yzx-nu";
        bash = "${pkgs.bashInteractive}/bin/bash";
        zsh = "${pkgs.zsh}/bin/zsh";
        fish = "${pkgs.fish}/bin/fish";
      };
      yzxShell = pkgs.runCommand "yzx-shell" {} ''
        install -D -m 755 ${yzxShellSrc} "$out/bin/yzx-shell"
        patchShebangs "$out/bin/yzx-shell"
      '';
      yzxEnvSupervisor = pkgs.runCommand "yzx-env-supervisor" {} ''
        install -D -m 755 ${./runtime/yzx-env-supervisor.sh} "$out/bin/yzx-env-supervisor"
        patchShebangs "$out/bin/yzx-env-supervisor"
      '';
      yzxMenuSrc = pkgs.replaceVars ./runtime/yzx-menu.rs {
        fzf = "${pkgs.fzf}/bin/fzf";
      };
      yzxMenu = rustBin "yzx-menu" yzxMenuSrc;
      yazelixZellijPopupPackage = yazelixZellijPopup.packages.${system}.yzpp;
      yazelixZellijBarPackage = yazelixZellijBar.packages.${system}.yazelix_zellij_bar;
      yazelixZellijPaneOrchestratorPackage =
        yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
      tokenusage = import ./packaging/tokenusage.nix {inherit pkgs;};
      yazelixScreenPackage = yazelixScreen.packages.${system}.yzs;
      yzxWelcome = pkgs.writeShellApplication {
        name = "yzx-welcome";
        text = ''
          if [ "''${YZX_WELCOME_ENABLED:-true}" != false ]; then
            if ! YAZELIX_SCREEN_COMMAND_NAME='yzx screen' ${yazelixScreenPackage}/bin/yzs "''${YZX_WELCOME_STYLE:-random}" --duration-seconds "''${YZX_WELCOME_DURATION_SECONDS:-3}"; then
              printf 'yzx welcome: failed to render welcome screen\n' >&2
            fi
          fi
          if [ "$#" -eq 0 ]; then
            exit 0
          fi
          exec "$@"
        '';
      };
      yzxZellijConfig = rustBin "yzx-zellij-config" ./runtime/yzx-zellij-config.rs;
      yazelixHelixPackage = yazelixHelix.packages.${system}.yazelix_helix;
      yzxHelixConfig = pkgs.writeTextDir "config.toml" (builtins.readFile ./defaults/helix/config.toml);
      yzxOpenTerminal = pkgs.writeShellApplication {
        name = "yzx-open-terminal";
        text = ''
          if [ "$#" -ne 1 ]; then
            printf '%s\n' 'usage: yzx-open-terminal <path>' >&2
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
      yzxHelixSteelConfig = pkgs.runCommand "yzx-helix-steel-config" {} ''
        mkdir -p "$out"
        cat > "$out/helix.scm" <<'EOF'
        ;; Yazelix Nova packaged Steel module.
        (provide yzx-new-shell)
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

        (define (yzx-new-shell-command target)
          (string-append "\"${yzxOpenTerminal}/bin/yzx-open-terminal\" " (yazelix-posix-quote target)))

        ;;@doc
        ;;Open a Yazelix terminal pane at the current Helix file or workspace.
        (define (yzx-new-shell)
          (let ([current-file (cx->current-file)]
                [current-workspace (get-helix-cwd)])
            (cond
              [(string? current-file)
               (run-shell-command (yzx-new-shell-command current-file))]
              [(string? current-workspace)
               (run-shell-command (yzx-new-shell-command current-workspace))]
              [else
               (set-error! "Yazelix could not resolve a target path for opening a shell")])))
        EOF
        cat > "$out/init.scm" <<'EOF'
        ;; Yazelix Nova packaged Steel init.
        EOF
      '';
      yzxHelixSrc = pkgs.replaceVars ./runtime/yzx-helix.sh {
        date = "${pkgs.coreutils}/bin/date";
        hx = "${yazelixHelixPackage}/bin/hx";
        mkdir = "${pkgs.coreutils}/bin/mkdir";
        od = "${pkgs.coreutils}/bin/od";
        tr = "${pkgs.coreutils}/bin/tr";
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        yzxHelixConfig = "${yzxHelixConfig}";
        yzxHelixSteelConfig = "${yzxHelixSteelConfig}";
      };
      yzxHelix = pkgs.runCommand "yzx-hx" {} ''
        install -D -m 755 ${yzxHelixSrc} "$out/bin/yzx-hx"
        ln -s yzx-hx "$out/bin/hx"
        patchShebangs "$out/bin/yzx-hx"
      '';
      yzxHelixUnavailable = pkgs.runCommand "yzx-hx-unavailable" {} ''
        mkdir -p "$out/bin"
        cat > "$out/bin/yzx-hx" <<'EOF'
        #!${pkgs.runtimeShell}
        printf '%s\n' 'yzx-hx: managed Helix is unavailable in this Yazelix package; set editor.command to an installed editor or select a package that includes managed Helix' >&2
        exit 69
        EOF
        chmod 755 "$out/bin/yzx-hx"
        ln -s yzx-hx "$out/bin/hx"
      '';
      yaziAssetsSelection = pkgs.fetchFromGitHub {
        owner = "luccahuguet";
        repo = "yazelix-yazi-assets";
        rev = "677c127bceca9b9de3aab1251f8b65fe81631309";
        sparseCheckout = ["plugins/git.yazi" "yazelix_starship.toml"];
        nonConeMode = true;
        hash = "sha256-E40pXHSUX75ig0ACcuinTSC4xiJu0r8fO/G9z+w+YuI=";
      };
      yaziFlavorNames = [
        "catppuccin-frappe.yazi"
        "catppuccin-latte.yazi"
        "catppuccin-macchiato.yazi"
        "catppuccin-mocha.yazi"
        "dracula.yazi"
      ];
      yaziFlavorsSelection = pkgs.fetchFromGitHub {
        owner = "yazi-rs";
        repo = "flavors";
        rev = "4770a3467169bfdb0a3b11601921aaf27c100630";
        sparseCheckout = yaziFlavorNames;
        hash = "sha256-TwYnWeRnclmHFwq6bisn7OTXqzWmGiEaEGIZFGAYhsw=";
      };
      yzxOpenCore = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-open";
        version = "0.1.0";
        src = ./crates/yzx-open;
        cargoLock.lockFile = ./crates/yzx-open/Cargo.lock;
      };
      yzxYaziToml = pkgs.replaceVars ./defaults/yazi/yazi.toml {
        opener = "YZX_ZELLIJ=${yazelixZellijPackage}/bin/zellij ${yzxOpenCore}/bin/yzx-open";
      };
      yzxYaziConfig = pkgs.runCommand "yzx-yazi-config" {} ''
        install -D -m 644 ${./defaults/yazi/init.lua} "$out/init.lua"
        install -D -m 644 ${./defaults/yazi/keymap.toml} "$out/keymap.toml"
        install -D -m 644 ${yzxYaziToml} "$out/yazi.toml"
        install -D -m 644 ${yaziAssetsSelection}/yazelix_starship.toml "$out/yazelix_starship.toml"
        mkdir -p "$out/plugins"
        install -D -m 644 ${./defaults/yazi/plugins/sidebar-state.yazi/main.lua} "$out/plugins/sidebar-state.yazi/main.lua"
        install -D -m 644 ${./defaults/yazi/plugins/sidebar-status.yazi/main.lua} "$out/plugins/sidebar-status.yazi/main.lua"
        install -D -m 644 ${./defaults/yazi/plugins/zoxide-editor.yazi/main.lua} "$out/plugins/zoxide-editor.yazi/main.lua"
        ln -s ${autoLayoutYazi} "$out/plugins/auto-layout.yazi"
        ln -s ${yaziAssetsSelection}/plugins/git.yazi "$out/plugins/git.yazi"
        ln -s ${starshipYazi} "$out/plugins/starship.yazi"
        for flavor in ${pkgs.lib.concatStringsSep " " yaziFlavorNames}; do
          for file in flavor.toml tmtheme.xml LICENSE LICENSE-tmtheme; do
            install -D -m 644 ${yaziFlavorsSelection}/"$flavor/$file" "$out/flavors/$flavor/$file"
          done
        done
      '';
      yzxYaziMaterializer = yzxYaziMaterializerFor pkgs;
      yzxRuntimeIdentity = pkgs.writeTextDir "runtime_identity.json" (builtins.toJSON {
        name = "Yazelix Nova";
        version = novaVersion;
      });
      defaultConfig = builtins.fromTOML (builtins.readFile ./defaults/config.toml);
      defaultBarWidgets = defaultConfig.bar.widgets;
      defaultShellProgram = defaultConfig.shell.program;
      defaultPopupSideMargin = toString defaultConfig.popup.side_margin;
      defaultPopupVerticalMargin = toString defaultConfig.popup.vertical_margin;
      barRenderRequest = import ./packaging/bar-render-request.nix {
        inherit (pkgs) coreutils nushell;
        runtimeIdentity = yzxRuntimeIdentity;
        zellijBar = yazelixZellijBarPackage;
      };
      yzxBarRenderRequest =
        pkgs.writeText "yzx-bar-render-request.json" (builtins.toJSON (barRenderRequest {
          widgetTray = defaultBarWidgets;
          shellLabel = defaultShellProgram;
        }));
      yzxBarRenderRequestTemplate =
        pkgs.writeText "yzx-bar-render-request-template.json" (builtins.toJSON (barRenderRequest {
          widgetTray = "__YZX_BAR_WIDGET_TRAY__";
          shellLabel = "__YZX_SHELL_LABEL__";
        }));
      yzxBarRender = pkgs.writeShellApplication {
        name = "yzx-bar-render";
        runtimeInputs = [pkgs.jq];
        text = ''
          ${yazelixZellijBarPackage}/${yazelixZellijBarPackage.widgetPath} render-yazelix-runtime --json "$1" \
            | jq -er '.plugin_block' \
            | ${pkgs.gnused}/bin/sed 's/YZX {command_version}/${novaBarLabel}/g'
        '';
      };
      yzxBarKdl = pkgs.runCommand "yzx-zellij-bar.kdl" {} ''
        ${yzxBarRender}/bin/yzx-bar-render "$(<${yzxBarRenderRequest})" > "$out"
      '';
      yzxLayoutCheck = rustBin "yzx-layout-check" ./checks/zellij-layout.rs;
      zellijBuildBase =
        if pkgs ? "zellij-unwrapped"
        then pkgs."zellij-unwrapped"
        else if pkgs.zellij ? unwrapped
        then pkgs.zellij.unwrapped
        else throw "Yazelix Nova requires the nixpkgs Zellij 0.44.3 unwrapped package contract";
      yazelixZellijPackage =
        assert zellijBuildBase.version == "0.44.3";
        zellijBuildBase.overrideAttrs (_old: {
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
      mkYzx = {
        withManagedHelix,
        withManagedYazi,
        withMars,
      }: let
        variantSuffix = pkgs.lib.concatStringsSep "-" (
          pkgs.lib.optional (! withMars) "no-mars"
          ++ pkgs.lib.optional (! withManagedHelix) "no-helix"
          ++ pkgs.lib.optional (! withManagedYazi) "no-yazi"
        );
        variant = if variantSuffix == "" then "full" else variantSuffix;
        name = "yazelix" + pkgs.lib.optionalString (variantSuffix != "") "-${variantSuffix}";
        yaziRuntime =
          if withManagedYazi
          then {
            source = "bundled";
            yaziCommand = "${pkgs.yazi}/bin/yazi";
            yaCommand = "${pkgs.yazi}/bin/ya";
          }
          else {
            source = "host";
            yaziCommand = "yazi";
            yaCommand = "ya";
          };
        managedEditor =
          if withManagedHelix
          then yzxHelix
          else yzxHelixUnavailable;
        tutor = let
          src = pkgs.runCommand "yzx-tutor-src" {} ''
            mkdir -p "$out"
            cp -R ${pkgs.lib.cleanSource ./crates/yzx-tutor}/. "$out/"
            chmod -R u+w "$out"
            substituteInPlace "$out/src/main.rs" \
              --replace-fail '@yzxHelix@' '${managedEditor}/bin/yzx-hx' \
              --replace-fail '@nu@' '${pkgs.nushell}/bin/nu'
          '';
        in
          pkgs.rustPlatform.buildRustPackage {
            pname = "yzx-tutor";
            version = "0.1.0";
            inherit src;
            cargoLock.lockFile = ./crates/yzx-tutor/Cargo.lock;
          };
        editor = pkgs.writeShellApplication {
          name = "yzx-editor";
          text = ''
            fallback="''${YAZELIX_EDITOR:-${managedEditor}/bin/yzx-hx}"
            editor="$(${yzxConfig}/bin/yzx-config --get editor.command 2>/dev/null || printf %s "$fallback")"
            case "$editor" in
              yzx-hx|hx) editor=${managedEditor}/bin/yzx-hx ;;
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
        editorEnv = ''
          export EDITOR=${editor}/bin/yzx-editor
          export VISUAL=${editor}/bin/yzx-editor
          export GIT_EDITOR=${editor}/bin/yzx-editor
        '';
        configUi = pkgs.writeShellApplication {
          name = "yzx-config-ui";
          text = ''
            unset YAZELIX_EDITOR
            ${editorEnv}
            exec ${yzxConfig}/bin/yzx-config "$@"
          '';
        };
        yazi = rustBin "yzx-yazi" (pkgs.replaceVars ./runtime/yzx-yazi.rs {
          yzxYaziConfig = "${yzxYaziConfig}";
          yzxYaziMaterializer = "${yzxYaziMaterializer}/bin/yzx-yazi-config";
          yzxOpen = "${yzxOpenCore}/bin/yzx-open";
          zellij = "${yazelixZellijPackage}/bin/zellij";
          yzxHelix = "${managedEditor}/bin/yzx-hx";
          yzxEditor = "${editor}/bin/yzx-editor";
          yzxConfig = "${yzxConfig}/bin/yzx-config";
          pathPrefix = pkgs.lib.makeBinPath [pkgs.fzf pkgs.git pkgs.starship pkgs.zoxide];
        });
        layout = let
          main = pkgs.runCommand "layout.kdl" {} ''
            substitute ${./defaults/zellij/layout.kdl} "$out" \
              --replace-fail '@yazi@' '${yazi}/bin/yzx-yazi' \
              --replace-fail '@bar@' "$(<${yzxBarKdl})"
          '';
          swap = pkgs.replaceVars ./defaults/zellij/layout.swap.kdl {
            yazi = "${yazi}/bin/yzx-yazi";
          };
        in
          pkgs.runCommand "yzx-zellij-layout" {} ''
            ${yzxLayoutCheck}/bin/yzx-layout-check ${main} ${swap} ${pkgs.lib.escapeShellArg novaBarLabel}
            install -D -m 644 ${main} "$out/layout.kdl"
            install -D -m 644 ${swap} "$out/layout.swap.kdl"
          '';
        git = let
          config = pkgs.writeText "yzx-lazygit.yml" ''
            os:
              edit: '${editor}/bin/yzx-editor {{filename}}'
              editAtLine: '${editor}/bin/yzx-editor {{filename}}'
              editAtLineAndWait: '${editor}/bin/yzx-editor {{filename}}'
              editInTerminal: true
              openDirInEditor: '${editor}/bin/yzx-editor {{dir}}'
          '';
        in
          pkgs.writeShellApplication {
            name = "yzx-git";
            text = ''
              ${editorEnv}
              if [ -z "''${LG_CONFIG_FILE:-}" ]; then
                config_file="$(${pkgs.lazygit}/bin/lazygit --print-config-dir)/config.yml"
                [ ! -f "$config_file" ] || LG_CONFIG_FILE="$config_file"
              fi
              export LG_CONFIG_FILE="''${LG_CONFIG_FILE:+$LG_CONFIG_FILE,}${config}"
              exec ${pkgs.lazygit}/bin/lazygit "$@"
            '';
          };
        configKdl = pkgs.replaceVars ./defaults/zellij/config.kdl {
          yzxShell = "${yzxShell}/bin/yzx-shell";
          yzpp = "file:${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
          yzxPaneOrchestrator = "file:${yazelixZellijPaneOrchestratorPackage}/${yazelixZellijPaneOrchestratorPackage.wasmPath}";
          yzxAgent = "${yzxAgent}/bin/yzx-agent";
          configKey = defaultConfig.keybindings.config;
          agentKey = defaultConfig.keybindings.agent;
          gitKey = defaultConfig.keybindings.git;
          menuKey = defaultConfig.keybindings.menu;
          screenKey = defaultConfig.keybindings.screen;
          sidebarKey = defaultConfig.keybindings.sidebar;
          sidebarFocusKey = defaultConfig.keybindings.sidebar_focus;
          inherit defaultPopupSideMargin defaultPopupVerticalMargin;
          yzxConfig = "${configUi}/bin/yzx-config-ui";
          yzxMenu = "${yzxMenu}/bin/yzx-menu";
          yzxScreen = "${yazelixScreenPackage}/bin/yzs";
          yzxYazi = "${yazi}/bin/yzx-yazi";
          yzxSidebarRefresh = "${yzxOpenCore}/bin/yzx-sidebar-refresh";
          git = "${git}/bin/yzx-git";
          layout = "${layout}/layout.kdl";
        };
        main = pkgs.replaceVars ./runtime/yzx/main.rs {
          packageVariant = variant;
          managedHelix = if withManagedHelix then "included" else "omitted";
          yzxConfigUi = "${configUi}/bin/yzx-config-ui";
          yzxMenu = "${yzxMenu}/bin/yzx-menu";
          yzxTutor = "${tutor}/bin/yzx-tutor";
          yzxScreen = "${yazelixScreenPackage}/bin/yzs";
          yzxWelcome = "${yzxWelcome}/bin/yzx-welcome";
          yzxShell = "${yzxShell}/bin/yzx-shell";
          yzxEnvSupervisor = "${yzxEnvSupervisor}/bin/yzx-env-supervisor";
          zellij = "${yazelixZellijPackage}/bin/zellij";
          mars = if withMars then "${marsPackage}/bin/mars" else "";
          layout = "${layout}/layout.kdl";
          layoutTemplate = "${./defaults/zellij/layout.kdl}";
          layoutSwapTemplate = "${./defaults/zellij/layout.swap.kdl}";
          yzxAgent = "${yzxAgent}/bin/yzx-agent";
          yzxYazi = "${yazi}/bin/yzx-yazi";
          yzxHelix = "${managedEditor}/bin/yzx-hx";
          yzxEditor = "${editor}/bin/yzx-editor";
          yzxConfig = "${yzxConfig}/bin/yzx-config";
          yzxMarsConfig = if withMars then "${yzxMarsConfig}" else "";
          yzxZellijConfig = "${yzxZellijConfig}/bin/yzx-zellij-config";
          yzxConfigKdl = "${configKdl}";
          yzxYaziConfig = "${yzxYaziConfig}";
          yzxYaziMaterializer = "${yzxYaziMaterializer}/bin/yzx-yazi-config";
          yzxReveal = "${yzxOpenCore}/bin/yzx-reveal";
          yzxSidebarRefresh = "${yzxOpenCore}/bin/yzx-sidebar-refresh";
          yaziSource = yaziRuntime.source;
          yaziCommand = yaziRuntime.yaziCommand;
          yaCommand = yaziRuntime.yaCommand;
          yaziTestedVersion = pkgs.yazi.version;
          yzxBarRenderRequest = "${yzxBarRenderRequestTemplate}";
          yzxBarRender = "${yzxBarRender}/bin/yzx-bar-render";
          yazelixZellijPopupWasm = "${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
          yazelixZellijBarWasm = "${yazelixZellijBarPackage}/share/yazelix_zellij_bar/zjstatus.wasm";
          yazelixZellijPaneOrchestratorWasm = "${yazelixZellijPaneOrchestratorPackage}/${yazelixZellijPaneOrchestratorPackage.wasmPath}";
          defaultBarWidgetsJson = builtins.toJSON defaultBarWidgets;
          inherit defaultShellProgram;
          defaultConfigKeybinding = defaultConfig.keybindings.config;
          defaultAgentKeybinding = defaultConfig.keybindings.agent;
          defaultGitKeybinding = defaultConfig.keybindings.git;
          defaultMenuKeybinding = defaultConfig.keybindings.menu;
          defaultScreenKeybinding = defaultConfig.keybindings.screen;
          defaultSidebarKeybinding = defaultConfig.keybindings.sidebar;
          defaultSidebarFocusKeybinding = defaultConfig.keybindings.sidebar_focus;
          inherit defaultPopupSideMargin defaultPopupVerticalMargin;
          version = novaVersion;
          pathPrefix = pkgs.lib.makeBinPath [
            pkgs.coreutils
            pkgs.git
            pkgs.lazygit
            tokenusage
            managedEditor
          ];
        };
        src = pkgs.runCommand "yzx-command-${variant}-src" {} ''
          mkdir -p "$out"
          cp -R ${pkgs.lib.cleanSource ./runtime/yzx}/. "$out/"
          chmod -R u+w "$out"
          cp ${main} "$out/main.rs"
        '';
        command = rustBin "yzx" "${src}/main.rs";
        withDesktop = withMars && pkgs.stdenv.hostPlatform.isLinux;
        desktop = pkgs.makeDesktopItem {
          name = "yzx";
          desktopName = "Yazelix Nova";
          genericName = "Terminal Emulator";
          comment = "Open the Yazelix integrated terminal workspace";
          exec = "${command}/bin/yzx launch";
          icon = "yzx";
          terminal = false;
          categories = ["System" "TerminalEmulator"];
          startupNotify = true;
          startupWMClass = "yzx";
        };
      in
        pkgs.symlinkJoin {
          inherit name;
          paths = [command] ++ pkgs.lib.optional withDesktop desktop;
          postBuild =
            ''
              ${yazelixZellijPackage}/bin/zellij --config ${configKdl} setup --check >/dev/null
              install -d "$out/libexec/yazelix"
              ln -s ${yzxZellijConfig}/bin/yzx-zellij-config "$out/libexec/yazelix/yzx-zellij-config"
              ln -s ${yzxConfig}/bin/yzx-config "$out/libexec/yazelix/yzx-config"
              ln -s ${tutor}/bin/yzx-tutor "$out/libexec/yazelix/yzx-tutor"
              install -D -m 644 ${configKdl} "$out/share/yazelix/config.kdl"
              install -D -m 644 ${yzxRuntimeIdentity}/runtime_identity.json "$out/share/yazelix/runtime_identity.json"
              install -D -m 644 ${yazelixCursors}/yazelix_cursors_default.toml "$out/share/yazelix/cursors.toml"
              install -D -m 644 ${./defaults/config.toml} "$out/share/yazelix/config.toml"
              install -D -m 644 ${layout}/layout.kdl "$out/share/yazelix/layout.kdl"
              install -D -m 644 ${layout}/layout.swap.kdl "$out/share/yazelix/layout.swap.kdl"
              ln -s ${yzxYaziConfig} "$out/share/yazelix/yazi"
              install -D -m 644 ${yzxNuConfig}/config.nu "$out/share/yazelix/nu/config.nu"
              install -D -m 644 ${yzxNuConfig}/env.nu "$out/share/yazelix/nu/env.nu"
            ''
            + pkgs.lib.optionalString withMars ''
              install -D -m 644 ${yzxMarsConfig}/config.toml "$out/share/yazelix/mars/config.toml"
            ''
            + pkgs.lib.optionalString withDesktop ''
              for icon in ${marsPackage}/share/icons/hicolor/*/apps/mars.png; do
                size="$(basename "$(dirname "$(dirname "$icon")")")"
                install -d "$out/share/icons/hicolor/$size/apps"
                ln -s "$icon" "$out/share/icons/hicolor/$size/apps/yzx.png"
              done
              install -d "$out/share/pixmaps"
              ln -s ${marsPackage}/share/pixmaps/mars.png "$out/share/pixmaps/yzx.png"
            '';
          meta.platforms = supportedSystems;
        };
    in rec {
      yazelix = mkYzx {
        withManagedHelix = true;
        withManagedYazi = true;
        withMars = true;
      };
      yazelix-no-helix = mkYzx {
        withManagedHelix = false;
        withManagedYazi = true;
        withMars = true;
      };
      yazelix-no-yazi = mkYzx {
        withManagedHelix = true;
        withManagedYazi = false;
        withMars = true;
      };
      yazelix-no-helix-no-yazi = mkYzx {
        withManagedHelix = false;
        withManagedYazi = false;
        withMars = true;
      };
      yazelix-no-mars = mkYzx {
        withManagedHelix = true;
        withManagedYazi = true;
        withMars = false;
      };
      yazelix-no-mars-no-helix = mkYzx {
        withManagedHelix = false;
        withManagedYazi = true;
        withMars = false;
      };
      yazelix-no-mars-no-yazi = mkYzx {
        withManagedHelix = true;
        withManagedYazi = false;
        withMars = false;
      };
      yazelix-no-mars-no-helix-no-yazi = mkYzx {
        withManagedHelix = false;
        withManagedYazi = false;
        withMars = false;
      };
      default = yazelix;
    });

    checks = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      yzx = self.packages.${system}.yazelix;
      yzxNoHelix = self.packages.${system}.yazelix-no-helix;
      yzxNoYazi = self.packages.${system}.yazelix-no-yazi;
      yzxNoHelixNoYazi = self.packages.${system}.yazelix-no-helix-no-yazi;
      yzxNoMars = self.packages.${system}.yazelix-no-mars;
      yzxNoMarsNoHelix = self.packages.${system}.yazelix-no-mars-no-helix;
      yzxNoMarsNoYazi = self.packages.${system}.yazelix-no-mars-no-yazi;
      yzxNoMarsNoHelixNoYazi =
        self.packages.${system}.yazelix-no-mars-no-helix-no-yazi;
      marsPackage = mars.packages.${system}.mars;
      noHelixClosure = pkgs.closureInfo {rootPaths = [yzxNoHelix];};
      noYaziClosure = pkgs.closureInfo {rootPaths = [yzxNoYazi];};
      noHelixNoYaziClosure = pkgs.closureInfo {rootPaths = [yzxNoHelixNoYazi];};
      noMarsClosure = pkgs.closureInfo {rootPaths = [yzxNoMars];};
      noMarsNoHelixClosure = pkgs.closureInfo {rootPaths = [yzxNoMarsNoHelix];};
      noMarsNoYaziClosure = pkgs.closureInfo {rootPaths = [yzxNoMarsNoYazi];};
      noMarsNoHelixNoYaziClosure =
        pkgs.closureInfo {rootPaths = [yzxNoMarsNoHelixNoYazi];};
      zellijBarPackage = yazelixZellijBar.packages.${system}.default;
      yzxYaziMaterializer = yzxYaziMaterializerFor pkgs;
      checksSrc = pkgs.lib.cleanSource ./checks;
      yzxContractsCheck = rustBinFor pkgs "yzx-contracts-check" "${checksSrc}/yzx-contracts.rs";
      helixContractsCheck = rustBinFor pkgs "helix-contracts-check" "${checksSrc}/helix-contracts.rs";
      noHelixContractsCheck =
        rustBinFor pkgs "no-helix-contracts-check" "${checksSrc}/no-helix-contracts.rs";
      mkFakeHostYazi = {
        name,
        yaVersion ? pkgs.yazi.version,
        yaziVersion ? pkgs.yazi.version,
      }:
        pkgs.runCommand name {} ''
          mkdir -p "$out/bin"
          cat > "$out/bin/yazi" <<'EOF'
          #!${pkgs.runtimeShell}
          if [ "''${1:-}" = --version ]; then
            printf '%s\n' 'Yazi ${yaziVersion}'
          else
            printf 'fake Yazi config=%s starship=%s role=%s ya=%s args=' \
              "''${YAZI_CONFIG_HOME:-}" \
              "''${YZX_YAZI_STARSHIP_CONFIG:-}" \
              "''${YZX_YAZI_ROLE:-}" \
              "''${YZX_YA:-}"
            printf '%s ' "$@"
            printf '\n'
          fi
          EOF
          cat > "$out/bin/ya" <<'EOF'
          #!${pkgs.runtimeShell}
          if [ "''${1:-}" = --version ]; then
            printf '%s\n' 'Ya ${yaVersion}'
          else
            printf 'fake Ya args='
            printf '%s ' "$@"
            printf '\n'
          fi
          EOF
          chmod 755 "$out/bin/yazi" "$out/bin/ya"
        '';
      fakeHostYazi = mkFakeHostYazi {name = "fake-host-yazi";};
      fakeNewerHostYazi = mkFakeHostYazi {
        name = "fake-newer-host-yazi";
        yaVersion = "99.0.0";
        yaziVersion = "99.0.0";
      };
      fakeMismatchedHostYazi = mkFakeHostYazi {
        name = "fake-mismatched-host-yazi";
        yaVersion = "98.0.0";
        yaziVersion = "99.0.0";
      };
      fakeYazelix = pkgs.runCommand "fake-yazelix-hm-package" {} ''
        mkdir -p "$out/bin" "$out/share/applications"
        cat > "$out/bin/yzx" <<'EOF'
        #!${pkgs.runtimeShell}
        printf '%s\n' fake-yazelix
        EOF
        chmod 755 "$out/bin/yzx"
        cat > "$out/share/applications/yzx.desktop" <<'EOF'
        [Desktop Entry]
        Type=Application
        Name=Fake Yazelix
        Exec=yzx
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
      fakeStarship = pkgs.writeText "hm-starship.toml" ''
        format = "$directory$git_branch"
      '';
      fakeYaziFlavor = pkgs.writeTextDir "flavor.toml" ''
        [mgr]
        cwd = { fg = "#c0ffee" }
      '';
      homeManagerConfiguration = module:
        home-manager.lib.homeManagerConfiguration {
          inherit pkgs;
          modules = [
            self.homeManagerModules.default
            {
              home.username = "yzx-test";
              home.homeDirectory = "/tmp/yzx-test-home";
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
      homeManagerNoMars = homeManagerConfiguration {
        programs.yazelix.package = yzxNoMars;
      };
      homeManagerNoYazi = homeManagerConfiguration {
        home.packages = [pkgs.yazi];
        programs.yazelix.package = yzxNoYazi;
      };
      homeManagerSharedStarship = homeManagerConfiguration {
        programs.yazelix.config = {
          starship.source = fakeStarship;
          yazi.starship.source = fakeStarship;
        };
      };
      homeManagerConfigFiles = homeManagerConfiguration {
        xdg.configFile."yazelix/yazi/flavors/example.yazi".source = fakeYaziFlavor;
        programs.yazelix.config = {
          settings = {
            shell.program = "fish";
            welcome.enabled = false;
            keybindings.config = "Alt Shift C";
            keybindings.agent = "Alt Shift A";
            keybindings.git = "Alt Shift G";
            keybindings.menu = "Alt Shift U";
            keybindings.screen = "Ctrl Shift S";
            keybindings.sidebar = "Ctrl Shift B";
            keybindings.sidebar_focus = "Ctrl Shift E";
            bar.widgets = ["editor" "shell"];
          };
          cursors.source = fakeCursors;
          mars.text = "[window]\nwidth = 1200\n";
          zellij.text = "pane_frames false\n";
          starship.text = "[character]\nformat = \"::\"\n";
          helix.config.text = "[editor]\nline-number = \"relative\"\n";
          helix.languages.source = fakeHelixLanguages;
          helix.module.text = "(provide yzx-test)\n";
          helix.init.text = ";; init\n";
          yazi.config.text = "[mgr]\nshow_hidden = true\n";
          yazi.init.text = "-- init\n";
          yazi.keymap.text = "[manager]\n";
          yazi.package.text = "[plugin]\ndeps = []\n";
          yazi.starship.source = fakeStarship;
          yazi.theme.text = "[flavor]\ndark = \"example\"\n";
          nu.env.text = "# env\n";
          nu.config.text = "# config\n";
        };
      };
    in {
      inherit yzx;
      zjstatus_activity_pipe = pkgs.runCommand "yzx-zjstatus-activity-pipe-check" {nativeBuildInputs = [pkgs.ripgrep];} ''
        rg -a -q 'tab_activity_pipe_name' ${zellijBarPackage}/${zellijBarPackage.wasmPath}
        touch "$out"
      '';
      home_manager = pkgs.runCommand "yzx-home-manager-check" {} ''
        default_path="${homeManagerDefault.activationPackage}/home-path"
        override_path="${homeManagerOverride.activationPackage}/home-path"
        no_mars_path="${homeManagerNoMars.activationPackage}/home-path"
        no_yazi_path="${homeManagerNoYazi.activationPackage}/home-path"
        shared_config_files="${homeManagerSharedStarship.activationPackage}/home-files/.config/yazelix"
        hm_yzx="${homeManagerConfigFiles.activationPackage}/home-path/bin/yzx"
        config_files="${homeManagerConfigFiles.activationPackage}/home-files/.config/yazelix"

        test -x "$default_path/bin/yzx"
        ${if pkgs.stdenv.hostPlatform.isLinux then ''
          test -f "$default_path/share/applications/yzx.desktop"
          grep -q 'Yazelix Nova' "$default_path/share/applications/yzx.desktop"
        '' else ''
          test ! -e "$default_path/share/applications/yzx.desktop"
        ''}

        test -x "$override_path/bin/yzx"
        test "$("$override_path/bin/yzx")" = fake-yazelix
        grep -q 'Fake Yazelix' "$override_path/share/applications/yzx.desktop"

        test -x "$no_mars_path/bin/yzx"
        test ! -e "$no_mars_path/share/applications/yzx.desktop"
        test -x "$no_yazi_path/bin/yzx"
        test -x "$no_yazi_path/bin/yazi"
        test -x "$no_yazi_path/bin/ya"

        if [ -e "${homeManagerDefault.activationPackage}/home-files/.config/yazelix" ]; then
          printf '%s\n' 'Home Manager v1 must not generate Yazelix runtime config files' >&2
          exit 1
        fi
        grep -q 'program = "fish"' "$config_files/config.toml"
        ! grep -q 'command = "yzx-hx"' "$config_files/config.toml"
        grep -q 'enabled = false' "$config_files/config.toml"
        ! grep -q 'style = "random"' "$config_files/config.toml"
        grep -q 'config = "Alt Shift C"' "$config_files/config.toml"
        grep -q 'agent = "Alt Shift A"' "$config_files/config.toml"
        grep -q 'git = "Alt Shift G"' "$config_files/config.toml"
        grep -q 'menu = "Alt Shift U"' "$config_files/config.toml"
        grep -q 'screen = "Ctrl Shift S"' "$config_files/config.toml"
        grep -q 'sidebar = "Ctrl Shift B"' "$config_files/config.toml"
        grep -q 'sidebar_focus = "Ctrl Shift E"' "$config_files/config.toml"
        ! grep -q 'ratconfig' "$config_files/config.toml"
        grep -q 'trail = "reef"' "$config_files/cursors.toml"
        test -L "$config_files/cursors.toml"
        case "$(readlink "$config_files/cursors.toml")" in
          /nix/store/*) ;;
          *) printf '%s\n' 'Home Manager cursor source is not store-backed' >&2; exit 1 ;;
        esac
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get shell.program)" = fish
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get editor.command)" = yzx-hx
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get agent.command)" = auto
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get agent.args)" = "[]"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.config)" = "Alt Shift C"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.agent)" = "Alt Shift A"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.git)" = "Alt Shift G"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.menu)" = "Alt Shift U"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.screen)" = "Ctrl Shift S"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.sidebar)" = "Ctrl Shift B"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.sidebar_focus)" = "Ctrl Shift E"
        grep -q 'width = 1200' "$config_files/mars/config.toml"
        grep -q 'pane_frames false' "$config_files/zellij/config.kdl"
        grep -q '^\[character\]$' "$config_files/starship.toml"
        grep -q 'format = "::"' "$config_files/starship.toml"
        grep -q 'line-number = "relative"' "$config_files/helix/config.toml"
        grep -q 'name = "nix"' "$config_files/helix/languages.toml"
        grep -q '(provide yzx-test)' "$config_files/helix/helix.scm"
        grep -q 'show_hidden = true' "$config_files/yazi/yazi.toml"
        grep -q -- '-- init' "$config_files/yazi/init.lua"
        grep -q 'deps = \[\]' "$config_files/yazi/package.toml"
        grep -Fqx 'format = "$directory$git_branch"' "$config_files/yazi/starship.toml"
        test "$(readlink -f "$config_files/starship.toml")" != \
          "$(readlink -f "$config_files/yazi/starship.toml")"
        grep -q 'dark = "example"' "$config_files/yazi/theme.toml"
        test -L "$config_files/yazi/flavors/example.yazi"
        case "$(readlink "$config_files/yazi/flavors/example.yazi")" in
          /nix/store/*) ;;
          *) printf '%s\n' 'Home Manager Yazi flavor is not store-backed' >&2; exit 1 ;;
        esac
        hm_yazi_runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$config_files/yazi" "$TMPDIR/hm-yazi-state")"
        grep -Fqx 'format = "$directory$git_branch"' "$hm_yazi_runtime/yazelix_starship.toml"
        YAZI_CONFIG_HOME="$hm_yazi_runtime" ${pkgs.yazi}/bin/yazi --debug > hm-yazi-debug
        grep -q 'Dark/light flavor:.*example' hm-yazi-debug

        test "$(readlink -f "$shared_config_files/starship.toml")" = \
          "$(readlink -f "$shared_config_files/yazi/starship.toml")"
        grep -q '# config' "$config_files/nu/config.nu"

        export HOME="$TMPDIR/hm-yzx-home"
        runtime_config="$TMPDIR/hm-yzx-config"
        cp -R "$config_files" "$runtime_config"
        chmod -R u+w "$runtime_config"
        export YAZELIX_CONFIG_HOME="$runtime_config"
        export YAZELIX_STATE_DIR="$TMPDIR/hm-yzx-state"
        export XDG_DATA_HOME="$TMPDIR/hm-yzx-data"
        mkdir -p "$HOME" "$YAZELIX_STATE_DIR" "$XDG_DATA_HOME"

        "$hm_yzx" help > help
        "$hm_yzx" status > status
        "$hm_yzx" doctor > doctor
        "$hm_yzx" tutor list > tutor-list
        "$hm_yzx" run ya --version > ya-version
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
        grep -q '^Ya ' ya-version
        touch "$out"
      '';
      yzx_yazi_materialization = pkgs.runCommand "yzx-yazi-materialization-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 --test ${./runtime/yzx-yazi.rs} -o yzx-yazi-materialization-check
        ./yzx-yazi-materialization-check

        user="$TMPDIR/yazi-user"
        state="$TMPDIR/yazi-state"
        install -D ${starshipYazi}/main.lua "$user/plugins/starship.yazi/main.lua"
        ln -s ${pkgs.yaziPlugins.smart-enter} "$user/plugins/smart-enter.yazi"
        touch "$user/plugins/starship.yazi/user-managed"
        printf '%s\n' 'require("smart-enter"):setup { open_multi = false }' > "$user/init.lua"
        printf '%s\n' '[[mgr.prepend_keymap]]' 'on = "l"' 'run = "plugin smart-enter"' > "$user/keymap.toml"

        runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$user" "$state")"
        YZX_YAZI_STARSHIP_CONFIG="$runtime/yazelix_starship.toml" YAZI_CONFIG_HOME="$runtime" ${pkgs.yazi}/bin/yazi --debug > yazi-debug
        test -f "$runtime/plugins/smart-enter.yazi/main.lua"
        test -f "$runtime/plugins/starship.yazi/user-managed"
        grep -q 'require("smart-enter")' "$runtime/init.lua"
        grep -q 'plugin smart-enter' "$runtime/keymap.toml"
        grep -q 'yzx-open' yazi-debug

        for flavor_path in ${yzx}/share/yazelix/yazi/flavors/*.yazi; do
          flavor_dir="''${flavor_path##*/}"
          flavor="''${flavor_dir%.yazi}"
          flavor_user="$TMPDIR/flavor-$flavor"
          mkdir -p "$flavor_user"
          printf '[flavor]\ndark = "%s"\nlight = "%s"\n' "$flavor" "$flavor" > "$flavor_user/theme.toml"
          flavor_runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$flavor_user" "$TMPDIR/state-$flavor")"
          YAZI_CONFIG_HOME="$flavor_runtime" ${pkgs.yazi}/bin/yazi --debug > "debug-$flavor"
          grep -q "Dark/light flavor:.*$flavor" "debug-$flavor"
          test -f "$flavor_runtime/flavors/$flavor_dir/flavor.toml"
          test -f "$flavor_runtime/flavors/$flavor_dir/tmtheme.xml"
          test ! -e "$flavor_runtime/flavors/$flavor_dir/preview.png"
        done
        touch "$out"
      '';
      yzx_launcher_unit = pkgs.runCommand "yzx-launcher-unit-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 --test ${pkgs.lib.cleanSource ./runtime/yzx}/main.rs -o yzx-launcher-unit-check
        ./yzx-launcher-unit-check
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

        extract_array ${./runtime/yzx-zellij-config.rs} FORBIDDEN > runtime
        extract_array ${./crates/yzx-config/src/catalog.rs} ZELLIJ_FORBIDDEN_TOP_LEVEL > config_ui
        diff -u runtime config_ui
        grep -qx default_shell runtime
        grep -qx env runtime
        touch "$out"
      '';
      zellij_theme_inventory_parity = pkgs.runCommand "zellij-theme-inventory-parity-check" {} ''
        for file in ${yazelixZellij}/zellij-utils/assets/themes/*.kdl; do
          awk '
            /^[[:space:]]*themes[[:space:]]*\{/ {
              in_themes = 1
              depth = 1
              next
            }
            in_themes {
              line = $0
              sub(/\/\/.*/, "", line)
              if (depth == 1 && line ~ /^[[:space:]]*("[^"]+"|[A-Za-z0-9_-]+)[[:space:]]*\{/) {
                name = line
                sub(/^[[:space:]]*/, "", name)
                if (name ~ /^"/) {
                  sub(/^"/, "", name)
                  sub(/".*/, "", name)
                } else {
                  sub(/[[:space:]]*\{.*/, "", name)
                }
                print name
              }
              opens = line
              closes = line
              depth += gsub(/\{/, "", opens) - gsub(/\}/, "", closes)
              if (depth <= 0) exit
            }
          ' "$file"
        done > actual-unsorted
        sort actual-unsorted > actual
        test "$(wc -l < actual)" -eq "$(sort -u actual | wc -l)"
        diff -u ${./crates/yzx-config/zellij-themes.txt} actual
        touch "$out"
      '';
      key_reference_parity = pkgs.runCommand "key-reference-parity-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 ${./checks/key-reference-parity.rs} -o key-reference-parity-check
        ./key-reference-parity-check ${./crates/yzx-config/src/catalog.rs} ${yzx}/share/yazelix/config.kdl ${./crates/yzx-tutor/src/main.rs}
        touch "$out"
      '';
      contracts = pkgs.runCommand "yzx-contracts" {} ''
        ${yzxContractsCheck}/bin/yzx-contracts-check ${yzx} ${pkgs.git}/bin/git ${pkgs.jq}/bin/jq "$out"
      '';
      no_mars_contracts = pkgs.runCommand "yzx-no-mars-contracts" {} ''
        check_no_mars() {
          local package="$1"
          local variant="$2"
          local closure="$3"
          local root="$TMPDIR/$variant"

          test -x "$package/bin/yzx"
          test ! -e "$package/share/applications/yzx.desktop"
          ! grep -Fx ${marsPackage} "$closure"
          ! grep -E '/[^/]*-rio-[^/]*$' "$closure"

          export HOME="$root/home"
          export YAZELIX_CONFIG_HOME="$root/config"
          export YAZELIX_STATE_DIR="$root/state"
          export XDG_DATA_HOME="$root/data"
          mkdir -p "$HOME" "$YAZELIX_CONFIG_HOME" "$YAZELIX_STATE_DIR" "$XDG_DATA_HOME"
          printf '%s\n' '[welcome]' 'enabled = false' > "$YAZELIX_CONFIG_HOME/config.toml"

          "$package/bin/yzx" status --json > "$root/status.json"
          test "$(${pkgs.jq}/bin/jq -r .package "$root/status.json")" = "$variant"
          "$package/bin/yzx" status > "$root/status"
          grep -Fqx "package: $variant" "$root/status"
          grep -Fqx 'mars config: not included' "$root/status"
          "$package/bin/yzx" doctor > "$root/doctor"
          grep -Fqx 'ok mars: not included' "$root/doctor"
          if "$package/bin/yzx" launch 2> "$root/launch-error"; then
            printf '%s\n' "$variant launch unexpectedly succeeded" >&2
            exit 1
          fi
          grep -q 'this package omits Mars' "$root/launch-error"
          "$package/bin/yzx" enter --version > "$root/enter-version"
          grep -q '^zellij ' "$root/enter-version"
        }

        check_no_mars ${yzxNoMars} no-mars ${noMarsClosure}/store-paths
        check_no_mars ${yzxNoMarsNoHelix} no-mars-no-helix ${noMarsNoHelixClosure}/store-paths
        touch "$out"
      '';
      helix_contracts = pkgs.runCommand "yzx-helix-contracts" {} ''
        ${helixContractsCheck}/bin/helix-contracts-check ${yzx} "$out"
      '';
      no_helix_contracts = pkgs.runCommand "yzx-no-helix-contracts" {} ''
        ${noHelixContractsCheck}/bin/no-helix-contracts-check \
          ${yzxNoHelix} ${noHelixClosure}/store-paths no-helix
        ${noHelixContractsCheck}/bin/no-helix-contracts-check \
          ${yzxNoMarsNoHelix} ${noMarsNoHelixClosure}/store-paths no-mars-no-helix
        touch "$out"
      '';
      host_yazi_contracts = pkgs.runCommand "yzx-host-yazi-contracts" {} ''
        for closure in \
          ${noYaziClosure}/store-paths \
          ${noHelixNoYaziClosure}/store-paths \
          ${noMarsNoYaziClosure}/store-paths \
          ${noMarsNoHelixNoYaziClosure}/store-paths; do
          if grep -Fx ${pkgs.yazi} "$closure"; then
            printf '%s\n' "host-Yazi closure retained ${pkgs.yazi}" >&2
            exit 1
          fi
        done

        package=${yzxNoMarsNoHelixNoYazi}
        root="$TMPDIR/host-yazi"
        export HOME="$root/home"
        export YAZELIX_CONFIG_HOME="$root/config"
        export YAZELIX_STATE_DIR="$root/state"
        export XDG_DATA_HOME="$root/data"
        mkdir -p "$HOME" "$YAZELIX_CONFIG_HOME" "$YAZELIX_STATE_DIR" "$XDG_DATA_HOME"
        printf '%s\n' '[welcome]' 'enabled = false' > "$YAZELIX_CONFIG_HOME/config.toml"

        user_yazi="$root/user-yazi"
        materialized_state="$root/materialized-state"
        mkdir -p "$user_yazi"
        printf '%s\n' '[mgr]' 'show_hidden = true' > "$user_yazi/yazi.toml"
        effective="$({
          PATH=${pkgs.coreutils}/bin "$package/bin/yzx" yazi-config materialize \
            --state-dir "$materialized_state" \
            --user-config-dir "$user_yazi"
        } 2> "$root/materialize-error")"
        test ! -s "$root/materialize-error"
        test "$effective" = "$(${pkgs.coreutils}/bin/readlink -f "$materialized_state")/yazi"
        grep -Fqx 'show_hidden = true' "$effective/yazi.toml"
        grep -F 'yzx-open' "$effective/yazi.toml"

        empty_user_yazi="$root/empty-user-yazi"
        empty_state="$root/empty-state"
        mkdir -p "$empty_user_yazi"
        empty_effective="$(PATH=${pkgs.coreutils}/bin "$package/bin/yzx" yazi-config materialize \
          --user-config-dir "$empty_user_yazi" \
          --state-dir "$empty_state")"
        test "$(${pkgs.coreutils}/bin/readlink -f "$empty_effective")" = \
          "$(${pkgs.coreutils}/bin/readlink -f "$package/share/yazelix/yazi")"
        test ! -e "$empty_state"

        invalid_user_yazi="$root/invalid-user-yazi"
        invalid_state="$root/invalid-state"
        mkdir -p "$invalid_user_yazi" "$invalid_state/yazi"
        printf '%s\n' '[mgr' > "$invalid_user_yazi/yazi.toml"
        printf '%s\n' keep > "$invalid_state/yazi/sentinel"
        set +e
        PATH=${pkgs.coreutils}/bin "$package/bin/yzx" yazi-config materialize \
          --user-config-dir "$invalid_user_yazi" \
          --state-dir "$invalid_state" \
          > "$root/invalid-output" 2> "$root/invalid-error"
        invalid_status=$?
        set -e
        test "$invalid_status" -eq 1
        test ! -s "$root/invalid-output"
        grep -F 'invalid user Yazi TOML' "$root/invalid-error"
        grep -Fqx keep "$invalid_state/yazi/sentinel"

        PATH=${pkgs.coreutils}/bin "$package/bin/yzx" yazi-config --help > "$root/yazi-config-help"
        grep -F 'yzx yazi-config materialize' "$root/yazi-config-help"
        PATH=${pkgs.coreutils}/bin "$package/bin/yzx" yazi-config materialize --help > "$root/materialize-help"
        grep -F -- '--user-config-dir <path>' "$root/materialize-help"
        set +e
        PATH=${pkgs.coreutils}/bin "$package/bin/yzx" yazi-config materialize \
          --user-config-dir "$user_yazi" > /dev/null 2> "$root/materialize-usage"
        usage_status=$?
        set -e
        test "$usage_status" -eq 64
        grep -F 'Usage: yzx yazi-config materialize' "$root/materialize-usage"

        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" doctor > "$root/doctor"
        grep -Fqx 'ok yazi source: host' "$root/doctor"
        grep -Fqx 'ok yazi: ${fakeHostYazi}/bin/yazi' "$root/doctor"
        grep -Fqx 'ok ya: ${fakeHostYazi}/bin/ya' "$root/doctor"
        grep -Fqx 'ok yazi version: ${pkgs.yazi.version}' "$root/doctor"
        grep -Fqx 'ok yazi tested version: ${pkgs.yazi.version}' "$root/doctor"

        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run ya --version > "$root/ya-version"
        grep -Fqx 'Ya ${pkgs.yazi.version}' "$root/ya-version"
        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run yazi --version > "$root/yazi-version"
        grep -Fqx 'Yazi ${pkgs.yazi.version}' "$root/yazi-version"
        mkdir -p "$YAZELIX_CONFIG_HOME/yazi"
        printf '%s\n' 'format = "$directory$git_branch"' > "$YAZELIX_CONFIG_HOME/yazi/starship.toml"
        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run yazi managed > "$root/yazi-managed"
        grep -F 'fake Yazi config=' "$root/yazi-managed"
        grep -F "starship=$YAZELIX_STATE_DIR/yazi/yazelix_starship.toml" "$root/yazi-managed"
        grep -F 'role= ya=' "$root/yazi-managed"
        grep -F 'ya=${fakeHostYazi}/bin/ya' "$root/yazi-managed"
        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run yazi \
          --yzx-workspace-popup popup > "$root/yazi-popup"
        grep -F "starship=$YAZELIX_STATE_DIR/yazi/yazelix_starship.toml" "$root/yazi-popup"
        grep -F 'role=workspace-popup' "$root/yazi-popup"
        grep -F 'args=popup ' "$root/yazi-popup"

        YZX_YAZI_BIN=${fakeHostYazi}/bin/yazi \
          YZX_YA=${fakeHostYazi}/bin/ya \
          PATH=${fakeMismatchedHostYazi}/bin:${pkgs.coreutils}/bin \
          "$package/bin/yzx" run yazi inherited > "$root/yazi-inherited"
        grep -F 'args=inherited' "$root/yazi-inherited"
        grep -F 'ya=${fakeHostYazi}/bin/ya' "$root/yazi-inherited"

        YZX_YAZI_BIN=${fakeMismatchedHostYazi}/bin/yazi \
          PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin \
          "$package/bin/yzx" status > "$root/partial-inherited"
        grep -Fqx 'yazi: ${fakeHostYazi}/bin/yazi' "$root/partial-inherited"
        grep -Fqx 'ya: ${fakeHostYazi}/bin/ya' "$root/partial-inherited"

        PATH=${fakeNewerHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" status > "$root/newer-status" 2> "$root/newer-warning"
        grep -F 'host yazi/ya 99.0.0 differs from Nova' "$root/newer-warning"
        PATH=${fakeNewerHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" doctor > "$root/newer-doctor"
        grep -F 'warn yazi compatibility: host yazi/ya 99.0.0 differs from Nova' "$root/newer-doctor"

        if PATH=${fakeMismatchedHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" status > /dev/null 2> "$root/mismatch"; then
          printf '%s\n' 'mismatched host Yazi pair unexpectedly succeeded' >&2
          exit 1
        fi
        grep -F 'yazi 99.0.0 and ya 98.0.0 differ' "$root/mismatch"

        if PATH=${pkgs.coreutils}/bin "$package/bin/yzx" status > /dev/null 2> "$root/missing"; then
          printf '%s\n' 'missing host Yazi pair unexpectedly succeeded' >&2
          exit 1
        fi
        grep -F 'yazi: command not found in PATH' "$root/missing"
        grep -F 'ya: command not found in PATH' "$root/missing"
        test "$(PATH=${pkgs.coreutils}/bin "$package/bin/yzx" run printf unrelated)" = unrelated
        touch "$out"
      '';
    });

    apps = eachSystem (system:
      builtins.mapAttrs (_name: package: {
        type = "app";
        program = "${package}/bin/yzx";
      })
      self.packages.${system});
  };
}
