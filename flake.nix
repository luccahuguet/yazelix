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
    rio = {
      url = "github:Yazelix/nova-rio/2ad2987d0580855393667651ce1d22f094901a23";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixZellij = {
      url = "github:Yazelix/nova-zellij/39d174558ca940013817c424d3ace45a3ac400d2";
      flake = false;
    };
    yazelixHelix = {
      url = "github:Yazelix/nova-helix/c664557d933edcddac73e3d1a0d80730ea93efa1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixForest = {
      url = "github:luccahuguet/yazelix-forest/4c7826af01efe8b7328baa1c7e480e7a539e4eda";
      flake = false;
    };
    notifyHx = {
      url = "github:chuwy/notify.hx/0a328073e6d3e5041346374ae747c275ab8ce746";
      flake = false;
    };
    glyphHx = {
      url = "github:Ra77a3l3-jar/glyph.hx/1e63ccbc8f17511543412c955879ba672f3f8ec1";
      flake = false;
    };
    yazelixZellijPopup = {
      url = "github:Yazelix/zellij-popup";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "novaBar/fenix";
      inputs.flake-utils.follows = "yazelixZellijPaneOrchestrator/flake-utils";
    };
    novaBar = {
      url = "github:Yazelix/nova-bar";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.zjstatus.follows = "zjstatus";
    };
    yazelixZellijPaneOrchestrator = {
      url = "github:Yazelix/zellij-pane-orchestrator";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "novaBar/fenix";
    };
    zjRadar = {
      url = "github:Yazelix/zj-radar/166626879b2067561915d7c60781932c809939d2";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "yazelixZellijPaneOrchestrator/flake-utils";
    };
    yazelixScreen = {
      url = "github:Yazelix/anima";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.fenix.follows = "novaBar/fenix";
    };
    autoLayoutYazi = {
      url = "github:Yazelix/auto-layout.yazi/6c4be74524e821e7a06aeb2f4d85a031c468def0";
      flake = false;
    };
    starshipYazi = {
      url = "github:Rolv-Apneseth/starship.yazi/ea92cf49380466f07231c952b409831e6afd2156";
      flake = false;
    };
    gitYazi = {
      url = "github:yazi-rs/plugins/72f9e3c007956c122d8657f6d39c78e7585a4718";
      flake = false;
    };
    yaziBistro = {
      url = "github:Yazelix/yazi-bistro";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yaziSchemas = {
      url = "github:yazi-rs/schemas/0a86623a771139104eba47c87ef669dfb280c61c";
      flake = false;
    };
    zjstatus = {
      url = "github:Yazelix/zjstatus/yazelix-tab-activity-pipe";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "yazelixZellijPaneOrchestrator/flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    home-manager,
    rio,
    yazelixZellij,
    yazelixHelix,
    yazelixForest,
    notifyHx,
    glyphHx,
    yazelixZellijPopup,
    novaBar,
    yazelixZellijPaneOrchestrator,
    zjRadar,
    yazelixScreen,
    autoLayoutYazi,
    starshipYazi,
    gitYazi,
    yaziBistro,
    yaziSchemas,
    zjstatus,
  }: let
    novaVersion = "1.2.0";
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
    pkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [
          (final: prev: {
            yazi-unwrapped = prev.yazi-unwrapped.overrideAttrs (finalAttrs: previousAttrs: {
              version = "26.9.1";
              env = previousAttrs.env // {VERGEN_BUILD_DATE = "2026-09-01";};
              passthru = previousAttrs.passthru // {
                srcs = previousAttrs.passthru.srcs // {
                  code_src = final.fetchFromGitHub {
                    owner = "sxyazi";
                    repo = "yazi";
                    tag = "v${finalAttrs.version}";
                    hash = "sha256-/8j4bEbT8DR/xlWtt62FXVyeHyWtBlvV8Rq0VbtY6ms=";
                  };
                };
              };
              cargoDeps = final.rustPlatform.fetchCargoVendor {
                inherit (finalAttrs) pname version srcs sourceRoot;
                hash = "sha256-V69VxhMiTY1Tgo4aW06AjwBIoXjK0Ov6oIahxk0NzGg=";
              };
            });
            yazi = prev.yazi.override {yazi-unwrapped = final.yazi-unwrapped;};
          })
        ];
      };
    rioPackageFor = pkgs: let
      rioPackage = rio.packages.${pkgs.stdenv.hostPlatform.system}.rio.overrideAttrs (_: {
        CARGO_BUILD_JOBS = "1";
        CARGO_PROFILE_RELEASE_LTO = "false";
        CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "16";
        CARGO_PROFILE_RELEASE_DEBUG = "0";
        CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO = "off";
      });
    in
      if !pkgs.stdenv.hostPlatform.isLinux
      then rioPackage
      else
        pkgs.symlinkJoin {
          name = "nova-rio";
          paths = [rioPackage];
          nativeBuildInputs = [pkgs.makeWrapper];
          postBuild = ''
            rm "$out/bin/rio"
            makeWrapper "${rioPackage}/bin/rio" "$out/bin/rio" \
              --set-default VK_ADD_DRIVER_FILES "${pkgs.mesa}/share/vulkan/icd.d"
          '';
        };
  in {
    homeManagerModules.default = homeManagerModule;

    packages = eachSystem (system: let
      pkgs = pkgsFor system;
      rustBin = rustBinFor pkgs;
      rioPackage = rioPackageFor pkgs;
      yzxRioToml = pkgs.replaceVars ./defaults/rio/config.toml {
        jetbrainsMonoDir = "${pkgs.jetbrains-mono}/share/fonts/truetype";
        symbolsNerdDir = "${pkgs.nerd-fonts.symbols-only}/share/fonts/truetype/NerdFonts/Symbols";
        notoSymbolsDir = "${pkgs.noto-fonts}/share/fonts/noto";
        notoEmojiDir = "${pkgs.noto-fonts-color-emoji}/share/fonts/noto";
      };
      yzxRioConfig = pkgs.runCommand "yzx-rio-config" {} ''
        install -D -m 644 ${yzxRioToml} "$out/config.toml"
        install -D -m 644 ${./defaults/rio/themes/nova-dark.toml} "$out/themes/nova-dark.toml"
        install -D -m 644 ${./defaults/rio/themes/nova-light.toml} "$out/themes/nova-light.toml"
      '';
      yzxCarapaceInit = pkgs.runCommand "yzx-carapace-init" {} ''
        ${pkgs.carapace}/bin/carapace _carapace nushell > "$out"
      '';
      yzxAtuinInit = pkgs.runCommand "yzx-atuin-init" {} ''
        mkdir "$out"
        export HOME="$TMPDIR"
        export XDG_CONFIG_HOME="$TMPDIR/config"
        for shell in nu bash zsh fish; do
          unset ATUIN_NOBIND
          ${pkgs.atuin}/bin/atuin init "$shell" --disable-up-arrow --disable-ai > "$out/$shell"
          ATUIN_NOBIND=1 ${pkgs.atuin}/bin/atuin init "$shell" --disable-up-arrow --disable-ai > "$out/$shell-nobind"
        done
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
        atuinInit = "${yzxAtuinInit}/nu";
        atuinNoBindInit = "${yzxAtuinInit}/nu-nobind";
        nu = "${pkgs.nushell}/bin/nu";
        packagedNu = "${yzxNuConfig}";
        pathPrefix = pkgs.lib.makeBinPath [pkgs.nushell pkgs.starship pkgs.carapace pkgs.atuin pkgs.zoxide];
        yzxConfig = "${yzxConfig}/bin/yzx-config";
      };
      yzxNuShell = rustBin "yzx-nu" yzxNuRs;
      yzxBashAtuinRc = pkgs.writeText "yzx-bashrc" ''
        if [ -r "$HOME/.bashrc" ]; then
          . "$HOME/.bashrc"
        fi
        if ! declare -F __atuin_history >/dev/null; then
          yzx_atuin_source=${yzxAtuinInit}/bash
          if [ -n "''${ATUIN_NOBIND+x}" ]; then
            yzx_atuin_source="$yzx_atuin_source-nobind"
          fi
          . "$yzx_atuin_source" || printf '%s\n' "yzx-shell: managed Atuin init failed" >&2
          unset yzx_atuin_source
        fi
      '';
      yzxFishAtuinInit = pkgs.writeText "yzx-fish-atuin.fish" ''
        if not functions -q _atuin_search
          set -l yzx_atuin_source ${yzxAtuinInit}/fish
          if set -q ATUIN_NOBIND
            set yzx_atuin_source "$yzx_atuin_source-nobind"
          end
          source "$yzx_atuin_source"; or echo "yzx-shell: managed Atuin init failed" >&2
        end
      '';
      yzxZshEnv = pkgs.writeText "yzx-zshenv" ''
        ZDOTDIR="$YZX_USER_ZDOTDIR"
        if [[ -r "$ZDOTDIR/.zshenv" ]]; then
          source "$ZDOTDIR/.zshenv"
          YZX_USER_ZDOTDIR="''${ZDOTDIR:-$HOME}"
        fi
        ZDOTDIR="$YZX_MANAGED_ZDOTDIR"
      '';
      yzxZshRc = pkgs.writeText "yzx-zshrc" ''
        ZDOTDIR="$YZX_USER_ZDOTDIR"
        if [[ -r "$ZDOTDIR/.zshrc" ]]; then
          source "$ZDOTDIR/.zshrc"
        fi
        if (( ! $+functions[_atuin_search] )); then
          yzx_atuin_source=${yzxAtuinInit}/zsh
          if [[ -v ATUIN_NOBIND ]]; then
            yzx_atuin_source="$yzx_atuin_source-nobind"
          fi
          source "$yzx_atuin_source" || print -u2 -- "yzx-shell: managed Atuin init failed"
          unset yzx_atuin_source
        fi
        unset YZX_USER_ZDOTDIR YZX_MANAGED_ZDOTDIR
      '';
      yzxZshAtuinConfig = pkgs.linkFarm "yzx-zsh-atuin" [
        {
          name = ".zshenv";
          path = yzxZshEnv;
        }
        {
          name = ".zshrc";
          path = yzxZshRc;
        }
      ];
      yzxAgent = rustBin "yzx-agent" ./runtime/yzx-agent.rs;
      yzxStarshipDefaults = pkgs.runCommand "yzx-starship-defaults.toml" {} ''
        export HOME="$TMPDIR"
        STARSHIP_CONFIG=/dev/null ${pkgs.starship}/bin/starship print-config --default > "$out"
      '';
      yzxConfigSrc = pkgs.runCommand "yzx-config-src" {} ''
        mkdir -p "$out"
        cp -R ${pkgs.lib.cleanSource ./crates/yzx-config}/. "$out/"
        chmod -R u+w "$out"
        cp ${./defaults/config.toml} "$out/config.toml"
        cp ${yzxRioToml} "$out/rio.toml"
        cp ${./defaults/rio/themes/nova-dark.toml} "$out/rio-dark.toml"
        cp ${./defaults/rio/themes/nova-light.toml} "$out/rio-light.toml"
        cp ${./defaults/helix/config.toml} "$out/helix.toml"
        substituteInPlace "$out/src/catalog.rs" \
          --replace-fail '../../../defaults/config.toml' '../config.toml' \
          --replace-fail '../../../defaults/rio/config.toml' '../rio.toml' \
          --replace-fail '../../../defaults/rio/themes/nova-dark.toml' '../rio-dark.toml' \
          --replace-fail '../../../defaults/rio/themes/nova-light.toml' '../rio-light.toml' \
          --replace-fail '../../../defaults/helix/config.toml' '../helix.toml'
      '';
      yzxConfig = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-config";
        version = "0.1.0";
        src = yzxConfigSrc;
        cargoLock = {
          lockFile = ./crates/yzx-config/Cargo.lock;
          outputHashes."ratconfig-6.0.0" = "sha256-z8vsrmVhac5mCfKouc6yvK0xpUjWFmhi6z7SixrmT7I=";
        };
        YAZELIX_NIX_STORE_ROOT = builtins.storeDir;
        YAZELIX_PACKAGED_YAZI = yzxYaziConfig;
        YAZELIX_AGENT_LAUNCHER = "${yzxAgent}/bin/yzx-agent";
        YAZELIX_STARSHIP_DEFAULT_CONFIG = yzxStarshipDefaults;
        YAZELIX_STARSHIP_CONFIG_SCHEMA = "${pkgs.starship.src}/docs/public/config-schema.json";
      };
      yzxShellSrc = pkgs.replaceVars ./runtime/yzx-shell.sh {
        atuinPath = pkgs.lib.makeBinPath [pkgs.atuin pkgs.bash pkgs.coreutils pkgs.gawk pkgs.gnused pkgs.ncurses];
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        yzxNu = "${yzxNuShell}/bin/yzx-nu";
        bash = "${pkgs.bashInteractive}/bin/bash";
        bashAtuinRc = yzxBashAtuinRc;
        zsh = "${pkgs.zsh}/bin/zsh";
        zshAtuinConfig = yzxZshAtuinConfig;
        fish = "${pkgs.fish}/bin/fish";
        fishAtuinInit = yzxFishAtuinInit;
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
      novaBarPackage = novaBar.packages.${system}.nova_bar;
      zjRadarPackage = zjRadar.packages.${system}.zj-radar;
      zjRadarCliPackage = zjRadar.packages.${system}.zj-radar-cli;
      yazelixZellijPaneOrchestratorPackage =
        yazelixZellijPaneOrchestrator.packages.${system}.yazelix_zellij_pane_orchestrator;
      tokenusage = import ./packaging/tokenusage.nix {inherit pkgs;};
      yazelixScreenPackage = yazelixScreen.packages.${system}.anima;
      yzxWelcome = pkgs.writeShellApplication {
        name = "yzx-welcome";
        text = ''
          if [ "''${YZX_WELCOME_ENABLED:-true}" != false ]; then
            if ! YAZELIX_SCREEN_COMMAND_NAME='yzx anima' ${yazelixScreenPackage}/bin/anima "''${YZX_WELCOME_STYLE:-random}" --duration-seconds "''${YZX_WELCOME_DURATION_SECONDS:-3}"; then
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
      yazelixHelixSteelPackage = yazelixHelix.packages.${system}.yazelix_helix_steel;
      yzxForestCogs = pkgs.runCommand "yzx-forest-cogs" {} ''
        install -D -m 444 ${yazelixForest}/forest.scm "$out/forest/forest.scm"
        install -D -m 444 ${yazelixForest}/forest/core.scm "$out/forest/core.scm"
        install -D -m 444 ${notifyHx}/notify.scm "$out/notify/notify.scm"
        install -D -m 444 ${glyphHx}/glyph.scm "$out/glyph/glyph.scm"
      '';
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
      yzxHelixBridgeRegister = pkgs.writeShellApplication {
        name = "yzx-helix-register";
        runtimeInputs = [pkgs.coreutils pkgs.jq];
        text = ''
          if [ "$#" -ne 1 ]; then
            printf '%s\n' 'usage: yzx-helix-register <loopback-address>' >&2
            exit 64
          fi

          state_dir="''${YAZELIX_STATE_DIR:?YAZELIX_STATE_DIR is required}"
          session_id="''${YAZELIX_HELIX_BRIDGE_SESSION_ID:?YAZELIX_HELIX_BRIDGE_SESSION_ID is required}"
          instance_id="''${YAZELIX_HELIX_BRIDGE_INSTANCE_ID:?YAZELIX_HELIX_BRIDGE_INSTANCE_ID is required}"
          auth_token="''${YAZELIX_HELIX_BRIDGE_AUTH_TOKEN:?YAZELIX_HELIX_BRIDGE_AUTH_TOKEN is required}"
          endpoint="$1"

          if [[ ! "$endpoint" =~ ^127\.0\.0\.1:([0-9]+)$ ]] ||
            (( 10#''${BASH_REMATCH[1]} < 1 || 10#''${BASH_REMATCH[1]} > 65535 )); then
            printf '%s\n' 'bridge address must be an IPv4 loopback endpoint with a valid port' >&2
            exit 64
          fi

          validate_id() {
            case "$2" in
              ""|"."|".."|*[!A-Za-z0-9._-]*)
                printf '%s must be a safe path component using only ASCII letters, numbers, dots, hyphens, and underscores\n' "$1" >&2
                exit 64
                ;;
            esac
          }
          validate_id YAZELIX_HELIX_BRIDGE_SESSION_ID "$session_id"
          validate_id YAZELIX_HELIX_BRIDGE_INSTANCE_ID "$instance_id"

          umask 077
          bridge_dir="$state_dir/helix_bridge/$session_id"
          token_path="$bridge_dir/$instance_id.token"
          registry_path="$bridge_dir/$instance_id.json"
          token_tmp="$token_path.tmp.$$"
          registry_tmp="$registry_path.tmp.$$"
          trap 'rm -f "$token_tmp" "$registry_tmp"' EXIT

          mkdir -p "$bridge_dir"
          chmod 700 "$bridge_dir"
          printf %s "$auth_token" > "$token_tmp"
          chmod 600 "$token_tmp"
          mv -f "$token_tmp" "$token_path"

          jq -n \
            --arg session_id "$session_id" \
            --arg instance_id "$instance_id" \
            --arg addr "$endpoint" \
            --arg auth_token_path "$token_path" \
            --argjson pid "$PPID" \
            --arg zellij_session_name "''${ZELLIJ_SESSION_NAME:-}" \
            --arg zellij_tab_position "''${ZELLIJ_TAB_POSITION:-}" \
            --arg zellij_pane_id "''${ZELLIJ_PANE_ID:-}" \
            --argjson started_at_unix_ms "$(date +%s%3N)" \
            --arg managed_config_path "''${YAZELIX_HELIX_MANAGED_CONFIG_PATH:-}" \
            'def optional: if . == "" then null else . end;
             {
               schema_version: 2,
               session_id: $session_id,
               instance_id: $instance_id,
               transport: {kind: "tcp", addr: $addr},
               auth_token_path: $auth_token_path,
               pid: $pid,
               zellij_session_name: ($zellij_session_name | optional),
               zellij_tab_position: ($zellij_tab_position | optional),
               zellij_pane_id: ($zellij_pane_id | optional),
               started_at_unix_ms: $started_at_unix_ms,
               managed_config_path: ($managed_config_path | optional)
             }' > "$registry_tmp"
          chmod 600 "$registry_tmp"
          mv -f "$registry_tmp" "$registry_path"
          trap - EXIT
        '';
      };
      yzxHelixInit = pkgs.replaceVars ./runtime/yzx-helix-init.scm {
        bridgeModule = "${yazelixHelixSteelPackage}/share/yazelix-helix/steel/yazelix/bridge.scm";
        bridgeRegister = "${yzxHelixBridgeRegister}/bin/yzx-helix-register";
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
        install -m 0444 ${yzxHelixInit} "$out/init.scm"
      '';
      yzxHelixSrc = pkgs.replaceVars ./runtime/yzx-helix.sh {
        date = "${pkgs.coreutils}/bin/date";
        hx = "${yazelixHelixPackage}/bin/hx";
        ln = "${pkgs.coreutils}/bin/ln";
        mkdir = "${pkgs.coreutils}/bin/mkdir";
        od = "${pkgs.coreutils}/bin/od";
        tr = "${pkgs.coreutils}/bin/tr";
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        yzxHelixConfig = "${yzxHelixConfig}";
        yzxHelixSteelConfig = "${yzxHelixSteelConfig}";
        yzxForestCogs = "${yzxForestCogs}";
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
      yaziBistroPackage = yaziBistro.packages.${system}.default;
      yzxOpenCore = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-open";
        version = "0.1.0";
        src = ./crates/yzx-open;
        cargoLock.lockFile = ./crates/yzx-open/Cargo.lock;
      };
      yzxYaziToml = pkgs.replaceVars ./defaults/yazi/yazi.toml {
        opener = "YZX_ZELLIJ=${yazelixZellijPackage}/bin/zellij ${yzxOpenCore}/bin/yzx-open";
      };
      yzxYaziConfig =
        assert pkgs.yazi-unwrapped.version == "26.9.1";
          pkgs.runCommand "yzx-yazi-config" {} ''
        install -D -m 644 ${./defaults/yazi/init.lua} "$out/init.lua"
        install -D -m 644 ${./defaults/yazi/keymap.toml} "$out/keymap.toml"
        install -D -m 644 ${yzxYaziToml} "$out/yazi.toml"
        install -D -m 644 ${./defaults/yazi/yazelix_starship.toml} "$out/yazelix_starship.toml"
        install -D -m 644 ${pkgs.yazi-unwrapped.srcs.code_src}/yazi-config/preset/yazi-default.toml "$out/yazi-default.toml"
        install -D -m 644 ${pkgs.yazi-unwrapped.srcs.code_src}/yazi-config/preset/theme-dark.toml "$out/theme-dark.toml"
        install -D -m 644 ${pkgs.yazi-unwrapped.srcs.code_src}/yazi-config/preset/theme-light.toml "$out/theme-light.toml"
        install -D -m 644 ${yaziSchemas}/schemas/yazi.json "$out/yazi-schema.json"
        install -D -m 644 ${yaziSchemas}/schemas/theme.json "$out/theme-schema.json"
        install -D -m 644 ${yaziSchemas}/LICENSE "$out/share/licenses/yazi-schemas/LICENSE"
        mkdir -p "$out/plugins"
        install -D -m 644 ${./defaults/yazi/plugins/zoxide-editor.yazi/main.lua} "$out/plugins/zoxide-editor.yazi/main.lua"
        ln -s ${autoLayoutYazi} "$out/plugins/auto-layout.yazi"
        ln -s ${gitYazi}/git.yazi "$out/plugins/git.yazi"
        ln -s ${starshipYazi} "$out/plugins/starship.yazi"
        ln -s ${yaziBistroPackage}/share/yazi-flavors/catalog.toml "$out/catalog.toml"
        ln -s ${yaziBistroPackage}/share/yazi-flavors/flavors "$out/flavors"
      '';
      yzxYaziMaterializer = yzxYaziMaterializerFor pkgs;
      defaultConfig = builtins.fromTOML (builtins.readFile ./defaults/config.toml);
      defaultBarWidgets = defaultConfig.bar.widgets;
      defaultShellProgram = defaultConfig.shell.program;
      defaultPopupSideMargin = toString defaultConfig.popup.side_margin;
      defaultPopupVerticalMargin = toString defaultConfig.popup.vertical_margin;
      yzxBarRender = pkgs.writeShellApplication {
        name = "yzx-bar-render";
        runtimeInputs = [pkgs.jq];
        text = ''
          ${novaBarPackage}/${novaBarPackage.widgetPath} render-nova-runtime --json "$1" \
            | jq -er '.plugin_block'
        '';
      };
      yzxLayoutCheck = rustBin "yzx-layout-check" ./checks/zellij-layout.rs;
      zellijBuildBase =
        if pkgs ? "zellij-unwrapped"
        then pkgs."zellij-unwrapped"
        else if pkgs.zellij ? unwrapped
        then pkgs.zellij.unwrapped
        else throw "Yazelix Nova requires an unwrapped nixpkgs Zellij build recipe";
      yazelixZellijPackage = zellijBuildBase.overrideAttrs (_old: {
        pname = "zellij";
        version = "0.45.0";
        src = yazelixZellij;
        patches = [];
        prePatch = "";
        postPatch = "";
        postInstall = pkgs.lib.optionalString (pkgs.stdenv.buildPlatform.canExecute pkgs.stdenv.hostPlatform) ''
          installShellCompletion --cmd zellij \
            --bash <($out/bin/zellij setup --generate-completion bash) \
            --fish <($out/bin/zellij setup --generate-completion fish) \
            --zsh <($out/bin/zellij setup --generate-completion zsh)
        '';
        installCheckPhase = ''
          runHook preInstallCheck
          runHook postInstallCheck
        '';
        cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
          pname = "zellij";
          version = "0.45.0";
          src = yazelixZellij;
          hash = "sha256-ZwxoqdZ73/HvdkdNWOKW3Av6htI/vCFcJ0zVpSL1SuU=";
        };
        doCheck = false;
      });
      mkYzx = {
        channel ? "stable",
        withRio,
        withManagedHelix,
        withManagedYazi,
      }: let
        channelLabel =
          {
            stable = "Stable";
            main = "Main";
            edge = "Edge";
          }.${channel}
          or (throw "unsupported Yazelix channel: ${channel}");
        runtimeIdentity = pkgs.writeTextDir "runtime_identity.json" (builtins.toJSON {
          name = "Yazelix Nova";
          version = novaVersion;
          inherit channel;
          rio_revision = if withRio then rio.rev else null;
        });
        barRenderRequest = import ./packaging/bar-render-request.nix {
          inherit (pkgs) coreutils nushell;
          inherit runtimeIdentity;
          novaBar = novaBarPackage;
        };
        yzxBarRenderRequestTemplate =
          pkgs.writeText "yzx-bar-render-request-template.json" (builtins.toJSON (barRenderRequest {
            appearanceMode = "__YZX_APPEARANCE_MODE__";
            widgetTray = "__YZX_BAR_WIDGET_TRAY__";
            shellLabel = "__YZX_SHELL_LABEL__";
          }));
        variantSuffix = pkgs.lib.concatStringsSep "-" (
          pkgs.lib.optional (! withRio) "no-rio"
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
              --replace-fail 'false; // @managedHelix@' '${if withManagedHelix then "true;" else "false;"}' \
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
            export YZX_RIO_INCLUDED=${if withRio then "1" else "0"}
            export YZX_RIO=${if withRio then "${rioPackage}/bin/rio" else "''"}
            export YZX_HELIX_INCLUDED=${if withManagedHelix then "1" else "0"}
            export YZX_ZELLIJ=${yazelixZellijPackage}/bin/zellij
            exec ${yzxConfig}/bin/yzx-config "$@"
          '';
        };
        yazi = rustBin "yzx-yazi" (pkgs.replaceVars ./runtime/yzx-yazi.rs {
          yzxYaziConfig = "${yzxYaziConfig}";
          yzxYaziMaterializer = "${yzxYaziMaterializer}/bin/yzx-yazi-config";
          yzxOpen = "${yzxOpenCore}/bin/yzx-open";
          yzxYaziReturn = "${yzxOpenCore}/bin/yzx-yazi-return";
          zellij = "${yazelixZellijPackage}/bin/zellij";
          yzxHelix = "${managedEditor}/bin/yzx-hx";
          yzxEditor = "${editor}/bin/yzx-editor";
          yzxConfig = "${yzxConfig}/bin/yzx-config";
          pathPrefix = pkgs.lib.makeBinPath [pkgs.fzf pkgs.git pkgs.starship pkgs.zoxide];
        });
        layout = let
          main = pkgs.runCommand "layout.kdl" {} ''
            bar="$(${yzxBarRender}/bin/yzx-bar-render ${pkgs.lib.escapeShellArg (builtins.toJSON (barRenderRequest {
              appearanceMode = "dark";
              widgetTray = defaultBarWidgets;
              shellLabel = defaultShellProgram;
            }))})"
            substitute ${./defaults/zellij/layout.kdl} "$out" \
              --replace-fail '@yazi@' '${yazi}/bin/yzx-yazi' \
              --replace-fail '@sidebar@' '{
                plugin location="radar"
            }' \
              --replace-fail '@bar@' "$bar"
          '';
          swap = pkgs.replaceVars ./defaults/zellij/layout.swap.kdl {
            sidebar = ''
              {
                plugin location="radar"
              }
            '';
          };
        in
          pkgs.runCommand "yzx-zellij-layout" {} ''
            ${yzxLayoutCheck}/bin/yzx-layout-check ${main} ${swap}
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
          zjRadar = "file:${zjRadarPackage}/bin/zj_radar.wasm";
          yzxAgent = "${yzxAgent}/bin/yzx-agent";
          configKey = defaultConfig.keybindings.config;
          agentKey = defaultConfig.keybindings.agent;
          gitKey = defaultConfig.keybindings.git;
          menuKey = defaultConfig.keybindings.menu;
          screenKey = defaultConfig.keybindings.screen;
          sidebarKey = defaultConfig.keybindings.sidebar;
          inherit defaultPopupSideMargin defaultPopupVerticalMargin;
          yzxConfig = "${configUi}/bin/yzx-config-ui";
          yzxMenu = "${yzxMenu}/bin/yzx-menu";
          yzxScreen = "${yazelixScreenPackage}/bin/anima";
          yzxYazi = "${yazi}/bin/yzx-yazi";
          git = "${git}/bin/yzx-git";
          layout = "${layout}/layout.kdl";
          layoutDir = "${layout}";
        };
        main = pkgs.replaceVars ./runtime/yzx/main.rs {
          packageVariant = variant;
          managedHelix = if withManagedHelix then "included" else "omitted";
          yzxConfigUi = "${configUi}/bin/yzx-config-ui";
          yzxMenu = "${yzxMenu}/bin/yzx-menu";
          yzxTutor = "${tutor}/bin/yzx-tutor";
          yzxScreen = "${yazelixScreenPackage}/bin/anima";
          yzxWelcome = "${yzxWelcome}/bin/yzx-welcome";
          yzxShell = "${yzxShell}/bin/yzx-shell";
          yzxEnvSupervisor = "${yzxEnvSupervisor}/bin/yzx-env-supervisor";
          zellij = "${yazelixZellijPackage}/bin/zellij";
          rio = if withRio then "${rioPackage}/bin/rio" else "";
          layout = "${layout}/layout.kdl";
          layoutTemplate = "${./defaults/zellij/layout.kdl}";
          layoutSwapTemplate = "${./defaults/zellij/layout.swap.kdl}";
          yzxAgent = "${yzxAgent}/bin/yzx-agent";
          yzxYazi = "${yazi}/bin/yzx-yazi";
          yzxHelix = "${managedEditor}/bin/yzx-hx";
          yzxEditor = "${editor}/bin/yzx-editor";
          yzxConfig = "${yzxConfig}/bin/yzx-config";
          yzxZellijConfig = "${yzxZellijConfig}/bin/yzx-zellij-config";
          yzxConfigKdl = "${configKdl}";
          yzxYaziConfig = "${yzxYaziConfig}";
          yzxYaziMaterializer = "${yzxYaziMaterializer}/bin/yzx-yazi-config";
          yzxReveal = "${yzxOpenCore}/bin/yzx-reveal";
          yaziSource = yaziRuntime.source;
          yaziCommand = yaziRuntime.yaziCommand;
          yaCommand = yaziRuntime.yaCommand;
          yaziTestedVersion = pkgs.yazi.version;
          yzxBarRenderRequest = "${yzxBarRenderRequestTemplate}";
          yzxBarRender = "${yzxBarRender}/bin/yzx-bar-render";
          yazelixZellijPopupWasm = "${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
          novaBarWasm = "${novaBarPackage}/share/nova_bar/zjstatus.wasm";
          zjRadarWasm = "${zjRadarPackage}/bin/zj_radar.wasm";
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
            yazi
            zjRadarCliPackage
          ];
        };
        src = pkgs.runCommand "yzx-command-${variant}-src" {} ''
          mkdir -p "$out"
          cp -R ${pkgs.lib.cleanSource ./runtime/yzx}/. "$out/"
          chmod -R u+w "$out"
          cp ${main} "$out/main.rs"
        '';
        command = rustBin "yzx" "${src}/main.rs";
        withDesktop = withRio && pkgs.stdenv.hostPlatform.isLinux;
        desktop = pkgs.makeDesktopItem {
          name = "yzx-${channel}";
          desktopName = "Yazelix Nova (${channelLabel})";
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
          paths = [command yazi zjRadarCliPackage] ++ pkgs.lib.optional withDesktop desktop;
          postBuild =
            ''
              ${yazelixZellijPackage}/bin/zellij --config ${configKdl} setup --check >/dev/null
              install -d "$out/libexec/yazelix"
              ln -s ${yzxZellijConfig}/bin/yzx-zellij-config "$out/libexec/yazelix/yzx-zellij-config"
              ln -s ${yzxConfig}/bin/yzx-config "$out/libexec/yazelix/yzx-config"
              ln -s ${tutor}/bin/yzx-tutor "$out/libexec/yazelix/yzx-tutor"
              install -D -m 644 ${configKdl} "$out/share/yazelix/config.kdl"
              install -D -m 644 ${runtimeIdentity}/runtime_identity.json "$out/share/yazelix/runtime_identity.json"
              install -D -m 644 ${./defaults/config.toml} "$out/share/yazelix/config.toml"
              install -D -m 644 ${layout}/layout.kdl "$out/share/yazelix/layout.kdl"
              install -D -m 644 ${layout}/layout.swap.kdl "$out/share/yazelix/layout.swap.kdl"
              install -D -m 644 ${zjRadar}/LICENSE "$out/share/licenses/zj-radar/LICENSE"
              ln -s ${yzxYaziConfig} "$out/share/yazelix/yazi"
              install -D -m 644 ${yzxNuConfig}/config.nu "$out/share/yazelix/nu/config.nu"
              install -D -m 644 ${yzxNuConfig}/env.nu "$out/share/yazelix/nu/env.nu"
            ''
            + pkgs.lib.optionalString withManagedHelix ''
              install -D -m 644 ${yazelixForest}/LICENSE "$out/share/licenses/yazelix-forest/LICENSE"
              install -D -m 644 ${notifyHx}/LICENSE "$out/share/licenses/notify-hx/LICENSE"
              install -D -m 644 ${glyphHx}/LICENSE "$out/share/licenses/glyph-hx/LICENSE"
            ''
            + pkgs.lib.optionalString withRio ''
              cp -R ${yzxRioConfig}/. "$out/share/yazelix/rio/"
            ''
            + pkgs.lib.optionalString withDesktop ''
              install -d "$out/share/icons/hicolor/scalable/apps"
              ln -s ${rioPackage}/share/icons/hicolor/scalable/apps/rio.svg \
                "$out/share/icons/hicolor/scalable/apps/yzx.svg"
            '';
          meta.platforms = supportedSystems;
        };
      mkFullYzx = channel:
        mkYzx {
          inherit channel;
          withRio = true;
          withManagedHelix = true;
          withManagedYazi = true;
        };
    in rec {
      yazelix = mkFullYzx "stable";
      yazelix-main = mkFullYzx "main";
      yazelix-edge = mkFullYzx "edge";
      yazelix-no-helix = mkYzx {
        withRio = true;
        withManagedHelix = false;
        withManagedYazi = true;
      };
      yazelix-no-yazi = mkYzx {
        withRio = true;
        withManagedHelix = true;
        withManagedYazi = false;
      };
      yazelix-no-helix-no-yazi = mkYzx {
        withRio = true;
        withManagedHelix = false;
        withManagedYazi = false;
      };
      yazelix-no-rio = mkYzx {
        withRio = false;
        withManagedHelix = true;
        withManagedYazi = true;
      };
      yazelix-no-rio-no-helix = mkYzx {
        withRio = false;
        withManagedHelix = false;
        withManagedYazi = true;
      };
      yazelix-no-rio-no-yazi = mkYzx {
        withRio = false;
        withManagedHelix = true;
        withManagedYazi = false;
      };
      yazelix-no-rio-no-helix-no-yazi = mkYzx {
        withRio = false;
        withManagedHelix = false;
        withManagedYazi = false;
      };
      default = yazelix;
    });

    checks = eachSystem (system: let
      pkgs = pkgsFor system;
      yzx = self.packages.${system}.yazelix;
      yzxMain = self.packages.${system}.yazelix-main;
      yzxEdge = self.packages.${system}.yazelix-edge;
      yzxNoHelix = self.packages.${system}.yazelix-no-helix;
      yzxNoYazi = self.packages.${system}.yazelix-no-yazi;
      yzxNoHelixNoYazi = self.packages.${system}.yazelix-no-helix-no-yazi;
      yzxNoRio = self.packages.${system}.yazelix-no-rio;
      yzxNoRioNoHelix = self.packages.${system}.yazelix-no-rio-no-helix;
      yzxNoRioNoYazi = self.packages.${system}.yazelix-no-rio-no-yazi;
      yzxNoRioNoHelixNoYazi = self.packages.${system}.yazelix-no-rio-no-helix-no-yazi;
      rioPackage = rioPackageFor pkgs;
      yzxClosure = pkgs.closureInfo {rootPaths = [yzx];};
      noHelixClosure = pkgs.closureInfo {rootPaths = [yzxNoHelix];};
      noYaziClosure = pkgs.closureInfo {rootPaths = [yzxNoYazi];};
      noHelixNoYaziClosure = pkgs.closureInfo {rootPaths = [yzxNoHelixNoYazi];};
      noRioClosure = pkgs.closureInfo {rootPaths = [yzxNoRio];};
      noRioNoHelixClosure = pkgs.closureInfo {rootPaths = [yzxNoRioNoHelix];};
      noRioNoYaziClosure = pkgs.closureInfo {rootPaths = [yzxNoRioNoYazi];};
      noRioNoHelixNoYaziClosure = pkgs.closureInfo {rootPaths = [yzxNoRioNoHelixNoYazi];};
      novaBarPackage = novaBar.packages.${system}.default;
      yzxYaziMaterializer = yzxYaziMaterializerFor pkgs;
      yaziEnvPty =
        if pkgs.stdenv.hostPlatform.isDarwin
        then "${pkgs.unixtools.script}/bin/script -qe /dev/null ${pkgs.yazi}/bin/ya env"
        else "${pkgs.unixtools.script}/bin/script -qec '${pkgs.yazi}/bin/ya env' /dev/null";
      checksSrc = pkgs.lib.cleanSource ./checks;
      yzxContractsCheck = rustBinFor pkgs "yzx-contracts-check" "${checksSrc}/yzx-contracts.rs";
      helixContractsCheck = rustBinFor pkgs "helix-contracts-check" "${checksSrc}/helix-contracts.rs";
      noHelixContractsCheck =
        rustBinFor pkgs "no-helix-contracts-check" "${checksSrc}/no-helix-contracts.rs";
      mkFakeHostYazi = {
        multiline ? false,
        name,
        yaVersion ? pkgs.yazi.version,
        yaziVersion ? pkgs.yazi.version,
      }:
        pkgs.runCommand name {} ''
          mkdir -p "$out/bin"
          cat > "$out/bin/yazi" <<'EOF'
          #!${pkgs.runtimeShell}
          case "''${0##*/}" in
            yazi) label=Yazi version='${yaziVersion}' ;;
            ya) label=Ya version='${yaVersion}' ;;
            *) exit 64 ;;
          esac
          if [ "''${1:-}" = --version ]; then
            ${if multiline
            then ''printf '%s\n' "$label" "    Version: $version" "    Debug  : false"''
            else ''printf '%s %s\n' "$label" "$version"''}
          elif [ "$label" = Yazi ]; then
            printf 'fake Yazi config=%s starship=%s role=%s ya=%s args=' \
              "''${YAZI_CONFIG_HOME:-}" \
              "''${YZX_YAZI_STARSHIP_CONFIG:-}" \
              "''${YZX_YAZI_ROLE:-}" \
              "''${YZX_YA:-}"
            printf '%s ' "$@"
            printf '\n'
          else
            printf 'fake Ya args='
            printf '%s ' "$@"
            printf '\n'
          fi
          EOF
          cp "$out/bin/yazi" "$out/bin/ya"
          chmod 755 "$out/bin/yazi" "$out/bin/ya"
        '';
      fakeHostYazi = mkFakeHostYazi {name = "fake-host-yazi";};
      fakeNewerHostYazi = mkFakeHostYazi {
        multiline = true;
        name = "fake-newer-host-yazi";
        yaVersion = "99.0.0";
        yaziVersion = "99.0.0";
      };
      fakeMismatchedHostYazi = mkFakeHostYazi {
        name = "fake-mismatched-host-yazi";
        yaVersion = "98.0.0";
        yaziVersion = "99.0.0";
      };
      fakeShimHostYazi = pkgs.runCommand "fake-shim-host-yazi" {} ''
        mkdir -p "$out/bin"
        ln -s ${fakeHostYazi}/bin/yazi "$out/bin/yazi"
        ln -s ${fakeHostYazi}/bin/yazi "$out/bin/ya"
      '';
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
      fakeRio = pkgs.writeText "hm-rio.toml" ''
        [colors]
        cursor = "#00e6ff"
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
      homeManagerNoRio = homeManagerConfiguration {
        programs.yazelix.package = yzxNoRio;
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
            appearance.mode = "light";
            shell.program = "fish";
            welcome.enabled = false;
            keybindings.config = "Alt Shift C";
            keybindings.agent = "Alt Shift A";
            keybindings.git = "Alt Shift G";
            keybindings.menu = "Alt Shift U";
            keybindings.screen = false;
            keybindings.sidebar = "Ctrl Shift B";
            keybindings.sidebar_focus = "Ctrl Shift E";
            bar.widgets = ["editor" "shell"];
          };
          rio.source = fakeRio;
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
      zjstatus_native_tabs = pkgs.runCommand "yzx-zjstatus-native-tabs-check" {nativeBuildInputs = [pkgs.ripgrep];} ''
        rg -a -q 'host_theme_mode' ${novaBarPackage}/${novaBarPackage.wasmPath}
        if rg -a -q 'tab_activity_pipe_name' ${novaBarPackage}/${novaBarPackage.wasmPath}; then
          echo 'retired tab-activity overlay remains in the packaged renderer' >&2
          exit 1
        fi
        touch "$out"
      '';
      home_manager = pkgs.runCommand "yzx-home-manager-check" {} ''
        default_path="${homeManagerDefault.activationPackage}/home-path"
        override_path="${homeManagerOverride.activationPackage}/home-path"
        no_rio_path="${homeManagerNoRio.activationPackage}/home-path"
        no_yazi_path="${homeManagerNoYazi.activationPackage}/home-path"
        shared_config_files="${homeManagerSharedStarship.activationPackage}/home-files/.config/yazelix"
        hm_yzx="${homeManagerConfigFiles.activationPackage}/home-path/bin/yzx"
        config_files="${homeManagerConfigFiles.activationPackage}/home-files/.config/yazelix"

        test -x "$default_path/bin/yzx"
        ${if pkgs.stdenv.hostPlatform.isLinux then ''
          test -f "$default_path/share/applications/yzx-stable.desktop"
          grep -Fqx 'Name=Yazelix Nova (Stable)' "$default_path/share/applications/yzx-stable.desktop"
        '' else ''
          test ! -e "$default_path/share/applications/yzx-stable.desktop"
        ''}

        test -x "$override_path/bin/yzx"
        test "$("$override_path/bin/yzx")" = fake-yazelix
        grep -q 'Fake Yazelix' "$override_path/share/applications/yzx.desktop"

        test -x "$no_rio_path/bin/yzx"
        test ! -e "$no_rio_path/share/applications/yzx-stable.desktop"
        test -x "$no_yazi_path/bin/yzx"
        test -x "$no_yazi_path/bin/yazi"
        test -x "$no_yazi_path/bin/ya"

        if [ -e "${homeManagerDefault.activationPackage}/home-files/.config/yazelix" ]; then
          printf '%s\n' 'Home Manager v1 must not generate Yazelix runtime config files' >&2
          exit 1
        fi
        grep -q 'program = "fish"' "$config_files/config.toml"
        grep -q 'mode = "light"' "$config_files/config.toml"
        ! grep -q 'command = "yzx-hx"' "$config_files/config.toml"
        grep -q 'enabled = false' "$config_files/config.toml"
        ! grep -q 'style = "random"' "$config_files/config.toml"
        grep -q 'config = "Alt Shift C"' "$config_files/config.toml"
        grep -q 'agent = "Alt Shift A"' "$config_files/config.toml"
        grep -q 'git = "Alt Shift G"' "$config_files/config.toml"
        grep -q 'menu = "Alt Shift U"' "$config_files/config.toml"
        grep -q 'screen = false' "$config_files/config.toml"
        grep -q 'sidebar = "Ctrl Shift B"' "$config_files/config.toml"
        grep -q 'sidebar_focus = "Ctrl Shift E"' "$config_files/config.toml"
        ! grep -q 'ratconfig' "$config_files/config.toml"
        grep -q 'cursor = "#00e6ff"' "$config_files/rio/config.toml"
        test -L "$config_files/rio/config.toml"
        case "$(readlink "$config_files/rio/config.toml")" in
          /nix/store/*) ;;
          *) printf '%s\n' 'Home Manager Rio source is not store-backed' >&2; exit 1 ;;
        esac
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get shell.program)" = fish
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get appearance.mode)" = light
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get editor.command)" = yzx-hx
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get agent.command)" = auto
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get agent.args)" = "[]"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.config)" = "Alt Shift C"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.agent)" = "Alt Shift A"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.git)" = "Alt Shift G"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.menu)" = "Alt Shift U"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.screen)" = false
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.sidebar)" = "Ctrl Shift B"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.sidebar_focus)" = "Ctrl Shift E"
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
        hm_yazi_runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$config_files/yazi" "$TMPDIR/hm-yazi-state" dark)"
        grep -Fqx 'format = "$directory$git_branch"' "$hm_yazi_runtime/yazelix_starship.toml"
        YAZI_CONFIG_HOME="$hm_yazi_runtime" ${yaziEnvPty} > hm-yazi-debug
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
        grep -q 'layout: runtime (' status
        grep -q 'host_theme_mode "light"' "$YAZELIX_STATE_DIR/zellij/layout.kdl"
        grep -Fq 'host_theme_light_tab_normal "#[fg=#5c5f77] [{index}] {name} "' "$YAZELIX_STATE_DIR/zellij/layout.kdl"
        grep -q 'Yazelix Nova doctor' doctor
        grep -q 'ok    Configuration    settings valid' doctor
        grep -q 'ok    Commands         shell fish · editor yzx-hx · agent auto' doctor
        grep -q 'Yazelix Nova tutor lessons' tutor-list
        grep -qx 'Ya' ya-version
        touch "$out"
      '';
      yzx_yazi_materialization = pkgs.runCommand "yzx-yazi-materialization-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 --test ${./runtime/yzx-yazi.rs} -o yzx-yazi-materialization-check
        ./yzx-yazi-materialization-check

        yazi_env() {
          YZX_YAZI_STARSHIP_CONFIG="$1/yazelix_starship.toml" YAZI_CONFIG_HOME="$1" ${yaziEnvPty} > "$2"
        }

        user="$TMPDIR/yazi-user"
        state="$TMPDIR/yazi-state"
        install -D ${starshipYazi}/main.lua "$user/plugins/starship.yazi/main.lua"
        ln -s ${pkgs.yaziPlugins.smart-enter} "$user/plugins/smart-enter.yazi"
        touch "$user/plugins/starship.yazi/user-managed"
        printf '%s\n' 'require("smart-enter"):setup { open_multi = false }' > "$user/init.lua"
        printf '%s\n' '[[mgr.prepend_keymap]]' 'on = "l"' 'run = "plugin smart-enter"' > "$user/keymap.toml"

        runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$user" "$state" dark)"
        yazi_env "$runtime" yazi-debug
        test -f "$runtime/plugins/smart-enter.yazi/main.lua"
        test -f "$runtime/plugins/starship.yazi/user-managed"
        grep -q 'require("smart-enter")' "$runtime/init.lua"
        grep -q 'plugin smart-enter' "$runtime/keymap.toml"
        grep -q 'yzx-open' yazi-debug

        light_runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$TMPDIR/no-yazi-user" "$TMPDIR/light-state" light)"
        grep -Fqx 'dark = "${yaziBistro.lib.defaultLight}"' "$light_runtime/theme.toml"
        grep -Fqx 'light = "${yaziBistro.lib.defaultLight}"' "$light_runtime/theme.toml"
        yazi_env "$light_runtime" light-yazi-debug
        grep -q 'Dark/light flavor:.*${yaziBistro.lib.defaultLight}' light-yazi-debug

        for flavor_path in ${yzx}/share/yazelix/yazi/flavors/*.yazi; do
          flavor_dir="''${flavor_path##*/}"
          flavor="''${flavor_dir%.yazi}"
          flavor_user="$TMPDIR/flavor-$flavor"
          mkdir -p "$flavor_user"
          printf '[flavor]\ndark = "%s"\nlight = "%s"\n' "$flavor" "$flavor" > "$flavor_user/theme.toml"
          flavor_runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$flavor_user" "$TMPDIR/state-$flavor" dark)"
          yazi_env "$flavor_runtime" "debug-$flavor"
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
        rustc --edition=2024 --test ${./runtime/yzx-agent.rs} -o yzx-agent-unit-check
        ./yzx-agent-unit-check
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
        yzx_shell="$(${pkgs.gnused}/bin/sed -n 's/.*default_shell "\([^"]*\)".*/\1/p' ${yzx}/share/yazelix/config.kdl)"
        test -x "$yzx_shell"
        export HOME="$TMPDIR/home"
        export YAZELIX_CONFIG_HOME="$TMPDIR/config"
        export YAZELIX_STATE_DIR="$TMPDIR/state"
        mkdir -p "$HOME"
        "$yzx_shell" -c 'help clip copy | ignore'
        test "$("$yzx_shell" -c 'scope aliases | where name == "clc" | get expansion.0')" = "clip copy"
        test "$("$yzx_shell" -c 'scope aliases | where name == "clp" | get expansion.0')" = "clip paste"
      '';
      desktop_channels = pkgs.runCommand "yzx-desktop-channels" {} ''
        check_channel() {
          package="$1"
          channel="$2"
          identity="$package/share/yazelix/runtime_identity.json"

          ${pkgs.jq}/bin/jq -e --arg channel "$channel" --arg version '${novaVersion}' \
            '.channel == $channel and .version == $version and (has("revision") | not)' "$identity" >/dev/null
          badge="$(${novaBarPackage}/${novaBarPackage.widgetPath} version --runtime-dir "$package/share/yazelix")"
          test "''${badge#NOVA }" != "$badge"
          test "''${badge##* }" = "''${channel^^}"

          ${if pkgs.stdenv.hostPlatform.isLinux then ''
            desktop="$package/share/applications/yzx-$channel.desktop"
            executable="$(readlink -f "$package/bin/yzx")"

            test -f "$desktop"
            grep -Fqx "Name=Yazelix Nova (''${channel^})" "$desktop"
            grep -Fqx "Exec=$executable launch" "$desktop"
            grep -Fqx 'Icon=yzx' "$desktop"
            grep -Fqx 'StartupWMClass=yzx' "$desktop"
          '' else ''
            test ! -e "$package/share/applications"
          ''}
        }

        check_channel ${yzx} stable
        check_channel ${yzxMain} main
        check_channel ${yzxEdge} edge
        touch "$out"
      '';
      rio_contracts = pkgs.runCommand "yzx-rio-contracts" {} ''
        grep -Fx ${rioPackage} ${yzxClosure}/store-paths
        ${pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux "grep -Fx ${pkgs.mesa} ${yzxClosure}/store-paths && grep -Fq VK_ADD_DRIVER_FILES ${rioPackage}/bin/rio && ! grep -Fq VK_ICD_FILENAMES ${rioPackage}/bin/rio"}
        ! grep -E '/[0-9a-z]{32}-(mars|yazelix[-_]cursors)(-|$)' ${yzxClosure}/store-paths
        ${rioPackage}/bin/rio --help | grep -F -- '--theme-mode <THEME_MODE>'
        ${rioPackage}/bin/rio --config-editor < ${yzx}/share/yazelix/rio/config.toml > inventory.toml
        ${pkgs.python3}/bin/python3 - <<'PY'
        import tomllib
        with open("inventory.toml", "rb") as stream:
            inventory = tomllib.load(stream)
        assert inventory["version"] == 1
        blur = next(field for field in inventory["fields"] if field["path"] == "window.blur")
        assert True in blur["choices"]
        with open("${yzx}/share/yazelix/rio/config.toml", "rb") as stream:
            assert tomllib.load(stream)["window"]["blur"] is True
        PY
        test -x ${yzx}/bin/yzx
        test -f ${yzx}/share/yazelix/rio/config.toml
        grep -Fqx 'cursor = "#00e6ff"' ${yzx}/share/yazelix/rio/config.toml
        grep -Fqx 'trail-cursor = true' ${yzx}/share/yazelix/rio/config.toml
        grep -Fqx 'adaptive-theme = { dark = "nova-dark", light = "nova-light" }' ${yzx}/share/yazelix/rio/config.toml
        test -f ${yzx}/share/yazelix/rio/themes/nova-dark.toml
        test -f ${yzx}/share/yazelix/rio/themes/nova-light.toml
        grep -Fqx 'background = "#111416"' ${yzx}/share/yazelix/rio/themes/nova-dark.toml
        grep -Fqx 'background = "#f5f3ef"' ${yzx}/share/yazelix/rio/themes/nova-light.toml
        test "$(${pkgs.jq}/bin/jq -r .rio_revision ${yzx}/share/yazelix/runtime_identity.json)" = ${rio.rev}
        touch "$out"
      '';
      no_rio_contracts = pkgs.runCommand "yzx-no-rio-contracts" {} ''
        check_no_rio() {
          local package="$1"
          local variant="$2"
          local closure="$3"
          local root="$TMPDIR/$variant"

          test -x "$package/bin/yzx"
          test ! -e "$package/share/applications"
          test ! -e "$package/share/yazelix/rio"
          test ! -e "$package/share/icons/hicolor/scalable/apps/yzx.svg"
          test "$(${pkgs.jq}/bin/jq -r .rio_revision "$package/share/yazelix/runtime_identity.json")" = null
          ! grep -Fx ${rioPackage} "$closure"
          ${pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux "! grep -Fx ${pkgs.mesa} \"$closure\""}
          ! grep -E '/[0-9a-z]{32}-(nova-rio|rio-terminfo)(-|$)' "$closure"

          config_ui="$(${pkgs.gnused}/bin/sed -n 's/.*command "\([^"]*yzx-config-ui\)".*/\1/p' "$package/share/yazelix/config.kdl" | ${pkgs.coreutils}/bin/head -n 1)"
          grep -Fq 'export YZX_RIO_INCLUDED=0' "$config_ui"

          export HOME="$root/home"
          export YAZELIX_CONFIG_HOME="$root/config"
          export YAZELIX_STATE_DIR="$root/state"
          export XDG_DATA_HOME="$root/data"
          export PATH=${fakeHostYazi}/bin:${pkgs.gnugrep}/bin:${pkgs.coreutils}/bin
          mkdir -p "$HOME" "$YAZELIX_CONFIG_HOME" "$YAZELIX_STATE_DIR" "$XDG_DATA_HOME"
          printf '%s\n' '[welcome]' 'enabled = false' > "$YAZELIX_CONFIG_HOME/config.toml"

          "$package/bin/yzx" status --json > "$root/status.json"
          test "$(${pkgs.jq}/bin/jq -r .package "$root/status.json")" = "$variant"
          "$package/bin/yzx" status > "$root/status"
          grep -Fqx "package: $variant" "$root/status"
          grep -Fqx 'rio config: not included' "$root/status"
          "$package/bin/yzx" doctor > "$root/doctor"
          grep -Fq 'ok    Configs          Zellij · layout validated · Rio omitted' "$root/doctor"
          if "$package/bin/yzx" launch 2> "$root/launch-error"; then
            printf '%s\n' "$variant launch unexpectedly succeeded" >&2
            exit 1
          else
            status=$?
          fi
          test "$status" -eq 64
          grep -Fq 'this package omits Rio; use yzx enter' "$root/launch-error"
          "$package/bin/yzx" enter --version > "$root/enter-version"
          grep -q '^zellij ' "$root/enter-version"
          test ! -e "$YAZELIX_CONFIG_HOME/rio"
        }

        check_no_rio ${yzxNoRio} no-rio ${noRioClosure}/store-paths
        check_no_rio ${yzxNoRioNoHelix} no-rio-no-helix ${noRioNoHelixClosure}/store-paths
        check_no_rio ${yzxNoRioNoYazi} no-rio-no-yazi ${noRioNoYaziClosure}/store-paths
        check_no_rio ${yzxNoRioNoHelixNoYazi} no-rio-no-helix-no-yazi ${noRioNoHelixNoYaziClosure}/store-paths
        touch "$out"
      '';
      helix_contracts = pkgs.runCommand "yzx-helix-contracts" {} ''
        ${helixContractsCheck}/bin/helix-contracts-check ${yzx} "$out"
      '';
      helix_grammar_sources = yazelixHelix.checks.${system}.codeberg_grammars;
      no_helix_contracts = pkgs.runCommand "yzx-no-helix-contracts" {} ''
        ${noHelixContractsCheck}/bin/no-helix-contracts-check \
          ${yzxNoHelix} ${noHelixClosure}/store-paths no-helix
        ${noHelixContractsCheck}/bin/no-helix-contracts-check \
          ${yzxNoRioNoHelix} ${noRioNoHelixClosure}/store-paths no-rio-no-helix
        touch "$out"
      '';
      host_yazi_contracts = pkgs.runCommand "yzx-host-yazi-contracts" {} ''
        for closure in \
          ${noYaziClosure}/store-paths \
          ${noHelixNoYaziClosure}/store-paths \
          ${noRioNoYaziClosure}/store-paths \
          ${noRioNoHelixNoYaziClosure}/store-paths; do
          if grep -Fx ${pkgs.yazi} "$closure"; then
            printf '%s\n' "host-Yazi closure retained ${pkgs.yazi}" >&2
            exit 1
          fi
        done

        package=${yzxNoHelixNoYazi}
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
        grep -Fq 'ok    Yazi             ${pkgs.yazi.version} (host)' "$root/doctor"

        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run ya --version > "$root/ya-version"
        grep -Fqx 'Ya ${pkgs.yazi.version}' "$root/ya-version"
        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run yazi --version > "$root/yazi-version"
        grep -Fqx 'Yazi ${pkgs.yazi.version}' "$root/yazi-version"
        PATH=${fakeShimHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" status > "$root/shim-status"
        grep -Fqx 'yazi: ${fakeShimHostYazi}/bin/yazi' "$root/shim-status"
        grep -Fqx 'ya: ${fakeShimHostYazi}/bin/ya' "$root/shim-status"
        PATH=${fakeShimHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run ya --version > "$root/shim-ya-version"
        grep -Fqx 'Ya ${pkgs.yazi.version}' "$root/shim-ya-version"
        PATH=${fakeShimHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run yazi --version > "$root/shim-yazi-version"
        grep -Fqx 'Yazi ${pkgs.yazi.version}' "$root/shim-yazi-version"
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
        PATH=${fakeHostYazi}/bin:${pkgs.coreutils}/bin "$package/bin/yzx" run yazi \
          --yzx-startup-picker > "$root/yazi-picker"
        grep -F 'role=startup-picker' "$root/yazi-picker"
        grep -F 'args=' "$root/yazi-picker"

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
        grep -F 'warn  Yazi compatibility host yazi/ya 99.0.0 differs from Nova' "$root/newer-doctor"

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
