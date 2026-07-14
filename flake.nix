{
  description = "Yazelix Nova";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix/96e0fc9f1a9b37f6477fa11c3fd48575354773ed";
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
    yazelixYaziAssets = {
      url = "github:FlexNetOS/yazelix-yazi-assets/0935209c3c7d8407c12c9a1a61bd0df6e8fd6a58";
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
    beads_rust_source = {
      url = "github:FlexNetOS/beads_rust/2498339168b8e88d641e8ae1664843fc69740012";
      flake = false;
    };
    rtk_source = {
      url = "github:rtk-ai/rtk/v0.43.0";
      flake = false;
    };
    grit_source = {
      url = "github:FlexNetOS/grit/89d8addd170f408d1d82860c39096929375bd2ce";
      flake = false;
    };
    icm_source = {
      url = "github:FlexNetOS/icm/ae4ed52c6bbf806e45f9c5b425e15b44398de4b7";
      flake = false;
    };
    weave_source = {
      url = "github:FlexNetOS/weave/9eae5c4d9cc9acb520e3d45dad25ea60ea22e63d";
      flake = false;
    };
    obscura_source = {
      url = "github:FlexNetOS/obscura/4f5b6e52d358b0e7a6a021a24bd12ff77b3f3989";
      flake = false;
    };
    flexnetos_runner_source = {
      url = "github:FlexNetOS/flexnetos_runner/be0960c138d9f293aa6272e6ef154c728b37f73a";
      flake = false;
    };
  };

  outputs = {
    self,
    nixpkgs,
    home-manager,
    fenix,
    mars,
    yazelixCursors,
    yazelixZellij,
    yazelixHelix,
    yazelixZellijPopup,
    yazelixZellijBar,
    yazelixZellijPaneOrchestrator,
    yazelixScreen,
    yazelixYaziAssets,
    ratconfig,
    autoLayoutYazi,
    starshipYazi,
    beads_rust_source,
    rtk_source,
    grit_source,
    icm_source,
    weave_source,
    obscura_source,
    flexnetos_runner_source,
  }: let
    novaVersion = "1.0.0-beta.1";
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
    nuApplicationFor = pkgs: name: source: replacements:
      pkgs.writeTextFile {
        inherit name;
        destination = "/bin/${name}";
        executable = true;
        text =
          "#!${pkgs.nushell}/bin/nu\n"
          + builtins.replaceStrings
          (map (key: "@${key}@") (builtins.attrNames replacements))
          (builtins.attrValues replacements)
          (builtins.readFile source);
      };
  in {
    homeManagerModules.default = homeManagerModule;

    packages = eachSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfreePredicate = package:
          nixpkgs.lib.getName package == "claude-code";
      };
      rustBin = rustBinFor pkgs;
      nuApplication = nuApplicationFor pkgs;
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
      flexnetosNuConfig = pkgs.replaceVars ./nushell/config/config.nu {
        rtkWrappers = "${./nushell/config/rtk_wrappers.nu}";
        stackPromptGuard = "${./nushell/config/stack_prompt_guard.nu}";
        flexnetosInit = "${./nushell/scripts/flexnetos_init.nu}";
        profileNu = "/home/flexnetos/.nix-profile/toolbin/nu";
      };
      yzxNuConfig = pkgs.runCommand "yzx-nu-config" {} ''
        install -D -m 644 ${yzxNuConfigNu} "$out/config.nu"
        install -D -m 644 ${./defaults/nu/env.nu} "$out/env.nu"
      '';
      flexnetosYzxNuConfig = pkgs.runCommand "flexnetos-yzx-nu-config" {} ''
        install -D -m 644 ${yzxNuConfigNu} "$out/config.nu"
        printf '\nsource "%s"\n' ${flexnetosNuConfig} >> "$out/config.nu"
        install -D -m 644 ${./defaults/nu/env.nu} "$out/env.nu"
      '';
      yzxConfigSrc = pkgs.runCommand "yzx-config-src" {} ''
        mkdir -p "$out"
        cp -R ${pkgs.lib.cleanSource ./crates/yzx-config}/. "$out/"
        chmod -R u+w "$out"
        ln -s ${ratconfig} "$out/ratconfig"
        ln -s ${yazelixCursors} "$out/yazelix-cursors"
        cp ${./defaults/config.toml} "$out/config.toml"
        cp ${./defaults/mars/config.toml} "$out/mars.toml"
        substituteInPlace "$out/Cargo.toml" \
          --replace-fail '../../../ratconfig' './ratconfig' \
          --replace-fail '../../../yazelix-cursors' './yazelix-cursors'
        substituteInPlace "$out/src/catalog.rs" \
          --replace-fail '../../../defaults/config.toml' '../config.toml' \
          --replace-fail '../../../defaults/mars/config.toml' '../mars.toml'
      '';
      yzxConfig = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-config";
        version = "0.1.0";
        src = yzxConfigSrc;
        cargoLock.lockFile = ./crates/yzx-config/Cargo.lock;
        YAZELIX_NIX_STORE_ROOT = builtins.storeDir;
        YZX_TEST_NU = "${pkgs.nushell}/bin/nu";
      };
      mkYzxNuShell = name: nuConfig: let
        source = pkgs.replaceVars ./runtime/yzx-nu.rs {
          nu = "${pkgs.nushell}/bin/nu";
          packagedNu = "${nuConfig}";
          pathPrefix = pkgs.lib.makeBinPath [pkgs.nushell pkgs.starship pkgs.carapace pkgs.zoxide];
          yzxConfig = "${yzxConfig}/bin/yzx-config";
        };
      in rustBin name source;
      yzxNuShell = mkYzxNuShell "yzx-nu" yzxNuConfig;
      flexnetosYzxNuShell = mkYzxNuShell "flexnetos-yzx-nu" flexnetosYzxNuConfig;
      mkYzxShell = name: nuShell:
        pkgs.linkFarm name [
          {
            name = "bin/yzx-shell";
            path = "${nuShell}/bin/${nuShell.name}";
          }
        ];
      yzxShell = mkYzxShell "yzx-shell" yzxNuShell;
      flexnetosYzxShell = mkYzxShell "flexnetos-yzx-shell" flexnetosYzxNuShell;
      yzxEnvSupervisor = rustBin "yzx-env-supervisor" ./runtime/yzx_env_supervisor.rs;
      yzxAgent = rustBin "yzx-agent" ./runtime/yzx-agent.rs;
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
      yzxWelcome = nuApplication "yzx-welcome" ./runtime/yzx_welcome.nu {
        yzs = "${yazelixScreenPackage}/bin/yzs";
      };
      yzxZellijConfig = rustBin "yzx-zellij-config" ./runtime/yzx-zellij-config.rs;
      yazelixHelixPackage = yazelixHelix.packages.${system}.yazelix_helix;
      yzxHelixConfig = pkgs.writeTextDir "config.toml" (builtins.readFile ./defaults/helix/config.toml);
      yzxOpenTerminal = nuApplication "yzx-open-terminal" ./runtime/yzx_open_terminal.nu {
        zellij = "${yazelixZellijPackage}/bin/zellij";
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
      yzxHelixBase = nuApplication "yzx-hx" ./runtime/yzx_helix.nu {
        hx = "${yazelixHelixPackage}/bin/hx";
        od = "${pkgs.coreutils}/bin/od";
        tr = "${pkgs.coreutils}/bin/tr";
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        yzxHelixConfig = "${yzxHelixConfig}";
        yzxHelixSteelConfig = "${yzxHelixSteelConfig}";
      };
      yzxHelix = pkgs.linkFarm "yzx-hx" [
        {
          name = "bin/yzx-hx";
          path = "${yzxHelixBase}/bin/yzx-hx";
        }
        {
          name = "bin/hx";
          path = "${yzxHelixBase}/bin/yzx-hx";
        }
      ];
      yzxTutorSrc = pkgs.runCommand "yzx-tutor-src" {} ''
        mkdir -p "$out"
        cp -R ${pkgs.lib.cleanSource ./crates/yzx-tutor}/. "$out/"
        chmod -R u+w "$out"
        substituteInPlace "$out/src/main.rs" \
          --replace-fail '@yzxHelix@' '${yzxHelix}/bin/yzx-hx' \
          --replace-fail '@nu@' '${pkgs.nushell}/bin/nu'
      '';
      yzxTutor = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-tutor";
        version = "0.1.0";
        src = yzxTutorSrc;
        cargoLock.lockFile = ./crates/yzx-tutor/Cargo.lock;
      };
      yzxEditor = nuApplication "yzx-editor" ./runtime/yzx_editor.nu {
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        yzxHelix = "${yzxHelix}/bin/yzx-hx";
      };
      yzxConfigUi = nuApplication "yzx-config-ui" ./runtime/yzx_config_ui.nu {
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        yzxEditor = "${yzxEditor}/bin/yzx-editor";
        yzxHelix = "${yzxHelix}/bin/yzx-hx";
      };
      yaziAssetsSelection = pkgs.fetchFromGitHub {
        owner = "luccahuguet";
        repo = "yazelix-yazi-assets";
        rev = "aea0703247479e1fa373be6b305e24e568cb30c7";
        sparseCheckout = ["plugins/git.yazi" "yazelix_starship.toml"];
        nonConeMode = true;
        hash = "sha256-eHt6kRaLcXgjhdnmhI2QY2O1tF9wGFXbIjXc4pObF4U=";
      };
      yzxOpenCore = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-open";
        version = "0.1.0";
        src = ./crates/yzx-open;
        cargoLock.lockFile = ./crates/yzx-open/Cargo.lock;
        YZX_TEST_NU = "${pkgs.nushell}/bin/nu";
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
      '';
      yzxYaziMaterializer = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-yazi-config";
        version = "0.1.0";
        src = ./crates/yzx-yazi-config;
        cargoLock.lockFile = ./crates/yzx-yazi-config/Cargo.lock;
      };
      yzxYaziSrc = pkgs.replaceVars ./runtime/yzx-yazi.rs {
        yazi = "${pkgs.yazi}/bin/yazi";
        yzxYaziConfig = "${yzxYaziConfig}";
        yzxYaziMaterializer = "${yzxYaziMaterializer}/bin/yzx-yazi-config";
        yzxOpen = "${yzxOpenCore}/bin/yzx-open";
        zellij = "${yazelixZellijPackage}/bin/zellij";
        yzxHelix = "${yzxHelix}/bin/yzx-hx";
        yzxEditor = "${yzxEditor}/bin/yzx-editor";
        yzxConfig = "${yzxConfig}/bin/yzx-config";
        pathPrefix = pkgs.lib.makeBinPath [pkgs.fzf pkgs.git pkgs.starship pkgs.zoxide];
      };
      yzxYazi = rustBin "yzx-yazi" yzxYaziSrc;
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
      yzxBarRender = nuApplication "yzx-bar-render" ./runtime/yzx_bar_render.nu {
        bar = "${yazelixZellijBarPackage}/${yazelixZellijBarPackage.widgetPath}";
        inherit novaBarLabel;
      };
      yzxBarKdl = pkgs.runCommand "yzx-zellij-bar.kdl" {} ''
        ${yzxBarRender}/bin/yzx-bar-render "$(<${yzxBarRenderRequest})" > "$out"
      '';
      yzxLayoutKdl = pkgs.runCommand "layout.kdl" {} ''
        substitute ${./defaults/zellij/layout.kdl} "$out" \
          --replace-fail '@yazi@' '${yzxYazi}/bin/yzx-yazi' \
          --replace-fail '@bar@' "$(<${yzxBarKdl})"
      '';
      yzxLayoutSwapKdl = pkgs.replaceVars ./defaults/zellij/layout.swap.kdl {
        yazi = "${yzxYazi}/bin/yzx-yazi";
      };
      yzxLayoutCheck = rustBin "yzx-layout-check" ./checks/zellij-layout.rs;
      yzxZellijLayout = pkgs.runCommand "yzx-zellij-layout" {} ''
        ${yzxLayoutCheck}/bin/yzx-layout-check ${yzxLayoutKdl} ${yzxLayoutSwapKdl} ${pkgs.lib.escapeShellArg novaBarLabel}
        install -D -m 644 ${yzxLayoutKdl} "$out/layout.kdl"
        install -D -m 644 ${yzxLayoutSwapKdl} "$out/layout.swap.kdl"
      '';
      flexnetosYaziAssets = yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
      flexnetosCcboard = "${flexnetosYaziAssets}/share/yazelix_yazi_assets/runtime_tools/ccboard/bin/ccboard";
      flexnetosCodedb = "${flexnetosYaziAssets}/share/yazelix_yazi_assets/runtime_tools/codedb/bin/codedb";
      flexnetosNuPluginCodedb = "${flexnetosYaziAssets}/share/yazelix_yazi_assets/runtime_tools/codedb/bin/nu_plugin_codedb";
      flexnetosLayoutTemplate = pkgs.runCommand "flexnetos-agent-workspace-template.kdl" {} ''
        substitute ${./defaults/zellij/flexnetos_agent_workspace.kdl} "$out" \
          --replace-fail '@yazi@' '${yzxYazi}/bin/yzx-yazi' \
          --replace-fail '@shell@' '${flexnetosYzxShell}/bin/yzx-shell' \
          --replace-fail '@agent@' '${yzxAgent}/bin/yzx-agent' \
          --replace-fail '@ccboard@' '${flexnetosCcboard}'
      '';
      flexnetosLayoutKdl = pkgs.runCommand "flexnetos-agent-workspace.kdl" {} ''
        substitute ${flexnetosLayoutTemplate} "$out" \
          --replace-fail '@bar@' "$(<${yzxBarKdl})"
      '';
      flexnetosZellijLayout = pkgs.runCommand "flexnetos-zellij-layout" {} ''
        ${yzxLayoutCheck}/bin/yzx-layout-check ${flexnetosLayoutKdl} ${yzxLayoutSwapKdl} ${pkgs.lib.escapeShellArg novaBarLabel} workspace
        install -D -m 644 ${flexnetosLayoutKdl} "$out/layout.kdl"
        install -D -m 644 ${yzxLayoutSwapKdl} "$out/layout.swap.kdl"
      '';
      yzxLazyGitConfig = pkgs.writeText "yzx-lazygit.yml" ''
        os:
          edit: '${yzxEditor}/bin/yzx-editor {{filename}}'
          editAtLine: '${yzxEditor}/bin/yzx-editor {{filename}}'
          editAtLineAndWait: '${yzxEditor}/bin/yzx-editor {{filename}}'
          editInTerminal: true
          openDirInEditor: '${yzxEditor}/bin/yzx-editor {{dir}}'
      '';
      yzxGit = nuApplication "yzx-git" ./runtime/yzx_git.nu {
        lazygit = "${pkgs.lazygit}/bin/lazygit";
        yzxEditor = "${yzxEditor}/bin/yzx-editor";
        yzxLazyGitConfig = "${yzxLazyGitConfig}";
      };
      mkYzxConfigKdl = shellPackage: pkgs.replaceVars ./defaults/zellij/config.kdl {
        yzxShell = "${shellPackage}/bin/yzx-shell";
        yzpp = "file:${yazelixZellijPopupPackage}/${yazelixZellijPopupPackage.wasmPath}";
        yzxPaneOrchestrator = "file:${yazelixZellijPaneOrchestratorPackage}/${yazelixZellijPaneOrchestratorPackage.wasmPath}";
        yzxAgent = "${yzxAgent}/bin/yzx-agent";
        configKey = defaultConfig.keybindings.config;
        agentKey = defaultConfig.keybindings.agent;
        gitKey = defaultConfig.keybindings.git;
        menuKey = defaultConfig.keybindings.menu;
        inherit defaultPopupSideMargin defaultPopupVerticalMargin;
        yzxConfig = "${yzxConfigUi}/bin/yzx-config-ui";
        yzxMenu = "${yzxMenu}/bin/yzx-menu";
        yzxSidebarRefresh = "${yzxOpenCore}/bin/yzx-sidebar-refresh";
        git = "${yzxGit}/bin/yzx-git";
        layout = "${yzxZellijLayout}/layout.kdl";
      };
      yzxConfigKdl = mkYzxConfigKdl yzxShell;
      flexnetosYzxConfigKdl = mkYzxConfigKdl flexnetosYzxShell;
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
      mkYzxCommand = {
        withMars,
        layoutPackage ? yzxZellijLayout,
        layoutTemplate ? ./defaults/zellij/layout.kdl,
        configKdl ? yzxConfigKdl,
        shellPackage ? yzxShell,
        extraPathPrefix ? [],
      }: let
        packageVariant = if withMars then "full" else "runtime";
        marsPath = if withMars then "${marsPackage}/bin/mars" else "";
        main = pkgs.replaceVars ./runtime/yzx/main.rs {
          yzxConfigUi = "${yzxConfigUi}/bin/yzx-config-ui";
          yzxMenu = "${yzxMenu}/bin/yzx-menu";
          yzxTutor = "${yzxTutor}/bin/yzx-tutor";
          yzxScreen = "${yazelixScreenPackage}/bin/yzs";
          yzxWelcome = "${yzxWelcome}/bin/yzx-welcome";
          yzxShell = "${shellPackage}/bin/yzx-shell";
          yzxEnvSupervisor = "${yzxEnvSupervisor}/bin/yzx-env-supervisor";
          zellij = "${yazelixZellijPackage}/bin/zellij";
          mars = marsPath;
          layout = "${layoutPackage}/layout.kdl";
          layoutTemplate = "${layoutTemplate}";
          layoutSwapTemplate = "${./defaults/zellij/layout.swap.kdl}";
          yzxAgent = "${yzxAgent}/bin/yzx-agent";
          yzxYazi = "${yzxYazi}/bin/yzx-yazi";
          yzxHelix = "${yzxHelix}/bin/yzx-hx";
          yzxEditor = "${yzxEditor}/bin/yzx-editor";
          yzxConfig = "${yzxConfig}/bin/yzx-config";
          yzxMarsConfig = if withMars then "${yzxMarsConfig}" else "";
          yzxZellijConfig = "${yzxZellijConfig}/bin/yzx-zellij-config";
          yzxConfigKdl = "${configKdl}";
          yzxRuntimeIdentity = "${yzxRuntimeIdentity}/runtime_identity.json";
          yzxReveal = "${yzxOpenCore}/bin/yzx-reveal";
          yzxSidebarRefresh = "${yzxOpenCore}/bin/yzx-sidebar-refresh";
          yzxYa = "${pkgs.yazi}/bin/ya";
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
          inherit defaultPopupSideMargin defaultPopupVerticalMargin;
          version = novaVersion;
          pathPrefix =
            pkgs.lib.makeBinPath [
              pkgs.coreutils
              pkgs.git
              pkgs.lazygit
              tokenusage
              yzxHelix
            ]
            + pkgs.lib.optionalString (extraPathPrefix != []) (
              ":" + pkgs.lib.makeBinPath extraPathPrefix
            );
        };
        src = pkgs.runCommand "yzx-command-${packageVariant}-src" {} ''
          mkdir -p "$out"
          cp -R ${pkgs.lib.cleanSource ./runtime/yzx}/. "$out/"
          chmod -R u+w "$out"
          cp ${main} "$out/main.rs"
        '';
      in
        rustBin "yzx" "${src}/main.rs";
      mkYzx = {
        name,
        withMars ? false,
        withDesktop ? withMars && pkgs.stdenv.hostPlatform.isLinux,
        layoutPackage ? yzxZellijLayout,
        layoutTemplate ? ./defaults/zellij/layout.kdl,
        configKdl ? yzxConfigKdl,
        nuConfig ? yzxNuConfig,
        shellPackage ? yzxShell,
        extraPathPrefix ? [],
      }: let
        command = mkYzxCommand {
          inherit withMars layoutPackage layoutTemplate configKdl shellPackage extraPathPrefix;
        };
        desktop = pkgs.makeDesktopItem {
          name = "yzx";
          desktopName = "Yazelix Nova";
          genericName = "Terminal Emulator";
          comment = "Open Yazelix Nova";
          exec = "${command}/bin/yzx launch";
          icon = "yzx";
          terminal = false;
          categories = ["System" "TerminalEmulator"];
          startupNotify = true;
          startupWMClass = "mars";
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
              ln -s ${yzxTutor}/bin/yzx-tutor "$out/libexec/yazelix/yzx-tutor"
              install -D -m 644 ${configKdl} "$out/share/yazelix/config.kdl"
              install -D -m 644 ${yzxRuntimeIdentity}/runtime_identity.json "$out/share/yazelix/runtime_identity.json"
              install -D -m 644 ${yazelixCursors}/yazelix_cursors_default.toml "$out/share/yazelix/cursors.toml"
              install -D -m 644 ${./defaults/config.toml} "$out/share/yazelix/config.toml"
              install -D -m 644 ${layoutPackage}/layout.kdl "$out/share/yazelix/layout.kdl"
              install -D -m 644 ${layoutPackage}/layout.swap.kdl "$out/share/yazelix/layout.swap.kdl"
              install -D -m 644 ${yzxYaziConfig}/init.lua "$out/share/yazelix/yazi/init.lua"
              install -D -m 644 ${yzxYaziConfig}/keymap.toml "$out/share/yazelix/yazi/keymap.toml"
              install -D -m 644 ${yzxYaziConfig}/plugins/sidebar-state.yazi/main.lua "$out/share/yazelix/yazi/plugins/sidebar-state.yazi/main.lua"
              install -D -m 644 ${yzxYaziConfig}/plugins/zoxide-editor.yazi/main.lua "$out/share/yazelix/yazi/plugins/zoxide-editor.yazi/main.lua"
              ln -s ${yzxYaziConfig}/plugins/git.yazi "$out/share/yazelix/yazi/plugins/git.yazi"
              install -D -m 644 ${yzxYaziConfig}/yazi.toml "$out/share/yazelix/yazi/yazi.toml"
              install -D -m 644 ${nuConfig}/config.nu "$out/share/yazelix/nu/config.nu"
              install -D -m 644 ${nuConfig}/env.nu "$out/share/yazelix/nu/env.nu"
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
      yazelix = mkYzx {
        name = "yazelix";
        withMars = true;
      };
      yzxRuntime = mkYzx {name = "yazelix-runtime";};
      fenixPkgs = fenix.packages.${system};
      flexnetosRustPlatform = pkgs.makeRustPlatform {
        cargo = fenixPkgs.latest.cargo;
        rustc = fenixPkgs.latest.rustc;
      };
      flexnetosBeads = import ./packaging/beads_rust.nix {
        inherit pkgs;
        beadsSource = beads_rust_source;
        rustPlatform = flexnetosRustPlatform;
      };
      flexnetosClaude = import ./packaging/claude_code_release.nix {
        inherit pkgs;
        version = "2.1.207";
      };
      flexnetosCodex = import ./packaging/codex_cli_release.nix {
        inherit pkgs system;
        version = "0.144.0";
      };
      flexnetosGitKb = import ./packaging/git_kb_release.nix {
        inherit pkgs;
        version = "0.2.12";
      };
      flexnetosRtk = import ./packaging/rtk_release.nix {
        inherit pkgs;
        rtkSource = rtk_source;
        rustPlatform = flexnetosRustPlatform;
      };
      flexnetosGrit = import ./packaging/grit_release.nix {
        inherit pkgs;
        gritSource = grit_source;
      };
      flexnetosIcm = import ./packaging/icm_release.nix {
        inherit pkgs;
        icmSource = icm_source;
      };
      flexnetosWeave = import ./packaging/weave_release.nix {
        inherit pkgs;
        weaveSource = weave_source;
      };
      flexnetosObscura = import ./packaging/obscura_release.nix {
        inherit pkgs;
        obscuraSource = obscura_source;
      };
      flexnetosMeta = import ./packaging/meta_release.nix {inherit pkgs;};
      flexnetosKacheBase = import ./packaging/kache_release.nix {inherit pkgs;};
      flexnetosRunner = import ./packaging/flexnetos_runner_release.nix {
        inherit pkgs;
        runnerSource = flexnetos_runner_source;
      };
      flexnetosNotebooklm = import ./packaging/notebooklm_release.nix {
        inherit pkgs;
        version = "0.8.0a3";
      };
      flexnetosKacheWrapperSource = pkgs.replaceVars ./packaging/kache_rustc_wrapper.rs {
        kache = "${flexnetosKacheBase}/bin/kache";
      };
      flexnetosKacheWrapper = rustBin "kache-rustc-wrapper" flexnetosKacheWrapperSource;
      flexnetosKacheWrappers = pkgs.linkFarm "kache-rustc-wrappers" [
        {
          name = "bin/kache-rustc-wrapper";
          path = "${flexnetosKacheWrapper}/bin/kache-rustc-wrapper";
        }
        {
          name = "libexec/kache/rustc";
          path = "${flexnetosKacheWrapper}/bin/kache-rustc-wrapper";
        }
      ];
      flexnetosKache = pkgs.symlinkJoin {
        name = "kache-with-rustc-wrapper-${flexnetosKacheBase.version}";
        paths = [flexnetosKacheBase flexnetosKacheWrappers];
      };
      flexnetosRunnerPolicy = nuApplication "flexnetos_runner_policy" ./nushell/runner/runner_policy.nu {};
      flexnetosRunnerService = nuApplication "flexnetos_runner_service" ./nushell/runner/runner_service.nu {};
      flexnetosHostPolicy = nuApplication "yazelix_host_policy" ./nushell/system/host_policy.nu {};
      flexnetosVolatileRuntime = nuApplication "yazelix_volatile_runtime" ./nushell/system/volatile_runtime.nu {};
      flexnetosRunnerSystemd = pkgs.writeTextDir
        "lib/systemd/user/flexnetos_runner@.service"
        (builtins.readFile (./systemd/user + "/flexnetos_runner@.service"));
      flexnetosHostPolicyBundle = pkgs.symlinkJoin {
        name = "yazelix-host-policy";
        paths = [
          (pkgs.writeTextDir "share/yazelix/host-policy/nix.conf" (builtins.readFile ./host-policy/nix.conf))
          (pkgs.writeTextDir "share/yazelix/host-policy/nix.custom.conf" (builtins.readFile ./host-policy/nix.custom.conf))
          (pkgs.writeTextDir "share/yazelix/host-policy/determinate-config.json" (builtins.readFile ./host-policy/determinate-config.json))
          (pkgs.writeTextDir "share/yazelix/host-policy/shells" (builtins.readFile ./host-policy/shells))
          (pkgs.writeTextDir "share/yazelix/host-policy/nix-daemon.service" (builtins.readFile ./host-policy/nix-daemon.service))
          (pkgs.writeTextDir "share/yazelix/host-policy/nix-daemon.socket" (builtins.readFile ./host-policy/nix-daemon.socket))
          (pkgs.writeTextDir "share/yazelix/host-policy/journald-no-storage.conf" (builtins.readFile ./host-policy/journald-no-storage.conf))
          (pkgs.writeTextDir "share/yazelix/host-policy/docker-daemon.json" (builtins.readFile ./host-policy/docker-daemon.json))
          (pkgs.writeTextDir "share/yazelix/host-policy/chrome-storage.json" (builtins.readFile ./host-policy/chrome-storage.json))
          (pkgs.writeTextDir "lib/systemd/system/yazelix_host_policy.service" (builtins.readFile ./systemd/system/yazelix_host_policy.service))
          (pkgs.writeTextDir "lib/systemd/system/yazelix_host_policy.path" (builtins.readFile ./systemd/system/yazelix_host_policy.path))
          (pkgs.writeTextDir "lib/systemd/system/nix-daemon.service.d/10-yazelix-host-policy.conf" (builtins.readFile ./systemd/system/nix-daemon.service.d/10-yazelix-host-policy.conf))
        ];
      };
      flexnetosVolatileRuntimeBundle = pkgs.symlinkJoin {
        name = "yazelix-volatile-runtime";
        paths = [
          (pkgs.writeTextDir "share/yazelix/environment.d/10-yazelix-volatile.conf" (builtins.readFile ./host-policy/10-yazelix-volatile.conf))
          (pkgs.writeTextDir "lib/systemd/user/yazelix_volatile_runtime.service" (builtins.readFile ./systemd/user/yazelix_volatile_runtime.service))
        ];
      };
      flexnetosRustToolchain = fenixPkgs.combine (
        [
          fenixPkgs.latest.cargo
          fenixPkgs.latest.rustc
          fenixPkgs.latest.rustfmt
          fenixPkgs.latest.clippy
          fenixPkgs.latest.rust-analyzer
        ]
        ++ pkgs.lib.optionals (system == "x86_64-linux") [
          fenixPkgs.targets.x86_64-unknown-linux-musl.latest.rust-std
        ]
      );
      flexnetosRust189 = fenixPkgs.fromToolchainName {
        name = "1.89.0";
        sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
      };
      flexnetosRust189Lane = pkgs.runCommand
        "flexnetos-foundation-rust-1.89-lane"
        {nativeBuildInputs = [pkgs.makeWrapper];}
        ''
          mkdir -p "$out/bin"
          makeWrapper "${flexnetosRust189.cargo}/bin/cargo" \
            "$out/bin/cargo-msrv-1.89" \
            --unset CARGO_BUILD_RUSTC_WRAPPER \
            --unset RUSTC_WRAPPER \
            --unset RUSTUP_TOOLCHAIN \
            --set RUSTC "${flexnetosRust189.rustc}/bin/rustc" \
            --set RUSTDOC "${flexnetosRust189.rustc}/bin/rustdoc"
          ln -s "${flexnetosRust189.rustc}/bin/rustc" "$out/bin/rustc-msrv-1.89"
          ln -s "${flexnetosRust189.rustc}/bin/rustdoc" "$out/bin/rustdoc-msrv-1.89"
        '';
      flexnetosMuslToolchain = pkgs.symlinkJoin {
        name = "flexnetos-foundation-musl-toolchain";
        paths = [pkgs.pkgsCross.musl64.stdenv.cc];
        postBuild = ''
          ln -s "$out/bin/x86_64-unknown-linux-musl-gcc" "$out/bin/x86_64-linux-musl-gcc"
          ln -s "$out/bin/x86_64-unknown-linux-musl-g++" "$out/bin/x86_64-linux-musl-g++"
          ln -s "$out/bin/x86_64-unknown-linux-musl-ar" "$out/bin/x86_64-linux-musl-ar"
          ln -s "$out/bin/x86_64-unknown-linux-musl-ranlib" "$out/bin/x86_64-linux-musl-ranlib"
        '';
      };
      flexnetosBun = pkgs.bun.overrideAttrs (_old: {
        version = "1.3.14";
        src = pkgs.fetchurl {
          url = "https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-linux-x64.zip";
          hash = "sha256-lR7iruhV8IWVruxiJSJqKY0/6oOj3NZGXAnLzN9+hI8=";
        };
      });
      flexnetosExecutables = {
        Xvfb = "${pkgs.xorg-server}/bin/Xvfb";
        actionlint = "${pkgs.actionlint}/bin/actionlint";
        br = "${flexnetosBeads}/bin/br";
        bun = "${flexnetosBun}/bin/bun";
        bunx = "${flexnetosBun}/bin/bunx";
        cargo = "${flexnetosRustToolchain}/bin/cargo";
        cargo-audit = "${pkgs.cargo-audit}/bin/cargo-audit";
        cargo-clippy = "${flexnetosRustToolchain}/bin/cargo-clippy";
        cargo-fmt = "${flexnetosRustToolchain}/bin/cargo-fmt";
        "cargo-msrv-1.89" = "${flexnetosRust189Lane}/bin/cargo-msrv-1.89";
        cargo-tauri = "${pkgs.cargo-tauri}/bin/cargo-tauri";
        cc = "${pkgs.stdenv.cc}/bin/cc";
        ccboard = flexnetosCcboard;
        clang = "${pkgs.clang}/bin/clang";
        "clang++" = "${pkgs.clang}/bin/clang++";
        claude = "${flexnetosClaude}/bin/claude";
        clippy-driver = "${flexnetosRustToolchain}/bin/clippy-driver";
        codedb = flexnetosCodedb;
        codex = "${flexnetosCodex}/bin/codex";
        corepack = "${pkgs.corepack}/bin/corepack";
        file = "${pkgs.file}/bin/file";
        fxrun = "${flexnetosRunner}/bin/fxrun";
        "fxrun-actions" = "${flexnetosRunner}/bin/fxrun-actions";
        "fxrun-dispatch" = "${flexnetosRunner}/bin/fxrun-dispatch";
        flexnetos_runner_policy = "${flexnetosRunnerPolicy}/bin/flexnetos_runner_policy";
        flexnetos_runner_service = "${flexnetosRunnerService}/bin/flexnetos_runner_service";
        yazelix_host_policy = "${flexnetosHostPolicy}/bin/yazelix_host_policy";
        yazelix_volatile_runtime = "${flexnetosVolatileRuntime}/bin/yazelix_volatile_runtime";
        git-kb = "${flexnetosGitKb}/bin/git-kb";
        grit = "${flexnetosGrit}/bin/grit";
        home-manager = "${home-manager.packages.${system}.default}/bin/home-manager";
        icm = "${flexnetosIcm}/bin/icm";
        kache = "${flexnetosKache}/bin/kache";
        kache-rustc-wrapper = "${flexnetosKache}/bin/kache-rustc-wrapper";
        "ld.wild" = "${pkgs.wild}/bin/ld.wild";
        loop = "${flexnetosMeta}/bin/loop";
        meta = "${flexnetosMeta}/bin/meta";
        meta-git = "${flexnetosMeta}/bin/meta-git";
        meta-mcp = "${flexnetosMeta}/bin/meta-mcp";
        meta-project = "${flexnetosMeta}/bin/meta-project";
        node = "${pkgs.nodejs_24}/bin/node";
        nix = "${pkgs.nix}/bin/nix";
        nix-build = "${pkgs.nix}/bin/nix-build";
        nix-daemon = "${pkgs.nix}/bin/nix-daemon";
        nix-env = "${pkgs.nix}/bin/nix-env";
        nix-instantiate = "${pkgs.nix}/bin/nix-instantiate";
        nix-shell = "${pkgs.nix}/bin/nix-shell";
        nix-store = "${pkgs.nix}/bin/nix-store";
        journalctl = "${pkgs.systemd}/bin/journalctl";
        notebooklm = "${flexnetosNotebooklm}/bin/notebooklm";
        npm = "${pkgs.nodejs_24}/bin/npm";
        nu = "${pkgs.nushell}/bin/nu";
        nu_plugin_codedb = flexnetosNuPluginCodedb;
        obscura = "${flexnetosObscura}/bin/obscura";
        pkg-config = "${pkgs.pkg-config}/bin/pkg-config";
        pnpm = "${pkgs.corepack}/bin/pnpm";
        rtk = "${flexnetosRtk}/bin/rtk";
        systemctl = "${pkgs.systemd}/bin/systemctl";
        rust-analyzer = "${flexnetosRustToolchain}/bin/rust-analyzer";
        rustc = "${flexnetosRustToolchain}/bin/rustc";
        "rustc-msrv-1.89" = "${flexnetosRust189Lane}/bin/rustc-msrv-1.89";
        rustdoc = "${flexnetosRustToolchain}/bin/rustdoc";
        "rustdoc-msrv-1.89" = "${flexnetosRust189Lane}/bin/rustdoc-msrv-1.89";
        rustfmt = "${flexnetosRustToolchain}/bin/rustfmt";
        sqld = "${pkgs.sqld}/bin/sqld";
        sqlite3 = "${pkgs.sqlite}/bin/sqlite3";
        tu = "${tokenusage}/bin/tu";
        usermod = "${pkgs.shadow}/bin/usermod";
        uv = "${pkgs.uv}/bin/uv";
        uvx = "${pkgs.uv}/bin/uvx";
        wasm-pack = "${pkgs.wasm-pack}/bin/wasm-pack";
        weave = "${flexnetosWeave}/bin/weave";
        wild = "${pkgs.wild}/bin/wild";
        yarn = "${pkgs.corepack}/bin/yarn";
        x86_64-linux-musl-ar = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-ar";
        "x86_64-linux-musl-g++" = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-g++";
        x86_64-linux-musl-gcc = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-gcc";
        x86_64-linux-musl-ranlib = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-ranlib";
        x86_64-unknown-linux-musl-ar = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-ar";
        "x86_64-unknown-linux-musl-g++" = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-g++";
        x86_64-unknown-linux-musl-gcc = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-gcc";
        x86_64-unknown-linux-musl-ranlib = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-ranlib";
      };
      flexnetosTools = pkgs.runCommand "flexnetos-foundation-tools" {} (
        ''
          mkdir -p "$out/bin" "$out/toolbin" "$out/libexec/kache"
          ln -s ${flexnetosKache}/libexec/kache/rustc "$out/libexec/kache/rustc"
        ''
        + pkgs.lib.concatStringsSep "\n" (
          pkgs.lib.mapAttrsToList (name: executable: ''
            test -x ${pkgs.lib.escapeShellArg executable}
            ln -s ${pkgs.lib.escapeShellArg executable} "$out/bin/${name}"
            ln -s ${pkgs.lib.escapeShellArg executable} "$out/toolbin/${name}"
          '') flexnetosExecutables
        )
      );
      flexnetosYzxBase = mkYzx {
        name = "lifeos-foundation-yzx-base";
        withMars = true;
        withDesktop = false;
        layoutPackage = flexnetosZellijLayout;
        layoutTemplate = flexnetosLayoutTemplate;
        configKdl = flexnetosYzxConfigKdl;
        nuConfig = flexnetosYzxNuConfig;
        shellPackage = flexnetosYzxShell;
        extraPathPrefix = [flexnetosTools];
      };
      lifeosFoundationYzx = pkgs.symlinkJoin {
        name = "lifeos-foundation-yzx";
        paths = [flexnetosYzxBase flexnetosTools flexnetosRunnerSystemd flexnetosHostPolicyBundle flexnetosVolatileRuntimeBundle];
        nativeBuildInputs = [pkgs.desktop-file-utils];
        postBuild = ''
          install -D -m 644 ${flexnetosZellijLayout}/layout.kdl \
            "$out/configs/zellij/layouts/flexnetos_agent_workspace.kdl"
          install -D -m 644 ${./nushell/config/config.nu} "$out/nushell/config/config.nu"
          install -D -m 644 ${./nushell/config/rtk_wrappers.nu} "$out/nushell/config/rtk_wrappers.nu"
          install -D -m 644 ${./nushell/config/stack_prompt_guard.nu} "$out/nushell/config/stack_prompt_guard.nu"
          install -D -m 644 ${./nushell/scripts/flexnetos_init.nu} "$out/nushell/scripts/flexnetos_init.nu"

          install -D -m 644 /dev/stdin "$out/share/applications/com.flexnetos.Yazelix.Agent.desktop" <<'EOF'
          [Desktop Entry]
          Version=1.4
          Type=Application
          Name=FlexNetOS Yazelix Agent
          Comment=Yazelix Nova with the profile-owned FlexNetOS agent workspace
          Icon=/home/flexnetos/.nix-profile/share/pixmaps/yazelix.png
          StartupWMClass=mars
          Terminal=false
          X-Yazelix-Managed=true
          X-FlexNetOS-Managed=true
          Exec=/home/flexnetos/.nix-profile/bin/yzx launch
          Categories=System;TerminalEmulator;
          EOF
          desktop-file-validate "$out/share/applications/com.flexnetos.Yazelix.Agent.desktop"

          for icon in ${marsPackage}/share/icons/hicolor/*/apps/mars.png; do
            size="$(basename "$(dirname "$(dirname "$icon")")")"
            install -d "$out/share/icons/hicolor/$size/apps"
            ln -s "$icon" "$out/share/icons/hicolor/$size/apps/yzx.png"
          done
          install -D -m 644 ${marsPackage}/share/pixmaps/mars.png \
            "$out/share/pixmaps/yazelix.png"
        '';
        meta = flexnetosYzxBase.meta;
      };
    in {
      inherit yazelix;
      runtime = yzxRuntime;
      default = yazelix;
    } // pkgs.lib.optionalAttrs (system == "x86_64-linux") {
      lifeos_foundation_yzx = lifeosFoundationYzx;
    });

    checks = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      yzx = self.packages.${system}.yazelix;
      yzxRuntime = self.packages.${system}.runtime;
      marsPackage = mars.packages.${system}.mars;
      runtimeClosure = pkgs.closureInfo {rootPaths = [yzxRuntime];};
      yzxYaziMaterializer = pkgs.rustPlatform.buildRustPackage {
        pname = "yzx-yazi-config";
        version = "0.1.0";
        src = ./crates/yzx-yazi-config;
        cargoLock.lockFile = ./crates/yzx-yazi-config/Cargo.lock;
      };
      checksSrc = pkgs.lib.cleanSource ./checks;
      yzxContractsCheck = rustBinFor pkgs "yzx-contracts-check" "${checksSrc}/yzx-contracts.rs";
      helixContractsCheck = rustBinFor pkgs "helix-contracts-check" "${checksSrc}/helix-contracts.rs";
      fakeYazelixBinary = pkgs.writeTextFile {
        name = "fake-yazelix-binary";
        destination = "/bin/yzx";
        executable = true;
        text = ''
          #!${pkgs.nushell}/bin/nu
          print fake-yazelix
        '';
      };
      fakeYazelixDesktop = pkgs.writeTextDir "share/applications/yzx.desktop" ''
        [Desktop Entry]
        Type=Application
        Name=Fake Yazelix
        Exec=yzx
      '';
      fakeYazelix = pkgs.symlinkJoin {
        name = "fake-yazelix-hm-package";
        paths = [fakeYazelixBinary fakeYazelixDesktop];
      };
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
      homeManagerRuntime = homeManagerConfiguration {
        programs.yazelix.package = yzxRuntime;
      };
      homeManagerConfigFiles = homeManagerConfiguration {
        programs.yazelix.config = {
          settings = {
            shell.program = "nu";
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
          helix.module.text = "(provide yzx-test)\n";
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
      inherit yzx;
      cache_shell_policy = pkgs.runCommand "cache-shell-policy-check" {} ''
        ${pkgs.nushell}/bin/nu ${./checks/cache_shell_policy.nu} ${./.}
        touch "$out"
      '';
      home_manager = pkgs.runCommand "yzx-home-manager-check" {} ''
        default_path="${homeManagerDefault.activationPackage}/home-path"
        override_path="${homeManagerOverride.activationPackage}/home-path"
        runtime_path="${homeManagerRuntime.activationPackage}/home-path"
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

        test -x "$runtime_path/bin/yzx"
        test ! -e "$runtime_path/share/applications/yzx.desktop"

        if [ -e "${homeManagerDefault.activationPackage}/home-files/.config/yazelix" ]; then
          printf '%s\n' 'Home Manager v1 must not generate Yazelix runtime config files' >&2
          exit 1
        fi
        grep -q 'program = "nu"' "$config_files/config.toml"
        ! grep -q 'command = "yzx-hx"' "$config_files/config.toml"
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
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get shell.program)" = nu
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get editor.command)" = yzx-hx
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get agent.command)" = auto
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get agent.args)" = "[]"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.config)" = "Alt Shift C"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.agent)" = "Alt Shift A"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.git)" = "Alt Shift G"
        test "$(YAZELIX_CONFIG_HOME="$config_files" ${yzx}/libexec/yazelix/yzx-config --get keybindings.menu)" = "Alt Shift U"
        grep -q 'width = 1200' "$config_files/mars/config.toml"
        grep -q 'pane_frames false' "$config_files/zellij/config.kdl"
        grep -q 'format = "::"' "$config_files/starship.toml"
        grep -q 'line-number = "relative"' "$config_files/helix/config.toml"
        grep -q 'name = "nix"' "$config_files/helix/languages.toml"
        grep -q '(provide yzx-test)' "$config_files/helix/helix.scm"
        grep -q 'show_hidden = true' "$config_files/yazi/yazi.toml"
        grep -q -- '-- init' "$config_files/yazi/init.lua"
        grep -q 'deps = \[\]' "$config_files/yazi/package.toml"
        grep -q 'dark = "example"' "$config_files/yazi/theme.toml"
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
        grep -q 'Usage:' help
        grep -q 'Yazelix Nova status' status
        grep -q "config home: $runtime_config" status
        grep -q "state dir: $YAZELIX_STATE_DIR" status
        grep -q 'shell: nu' status
        grep -q 'welcome enabled: false' status
        grep -q 'Yazelix Nova doctor' doctor
        grep -q "ok config home: $runtime_config" doctor
        grep -q 'ok shell.program: nu' doctor
        grep -q 'Yazelix Nova tutor lessons' tutor-list
        touch "$out"
      '';
      yzx_yazi_materialization = pkgs.runCommand "yzx-yazi-materialization-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 --test ${./runtime/yzx-yazi.rs} -o yzx-yazi-materialization-check
        ./yzx-yazi-materialization-check

        user="$TMPDIR/yazi-user"
        state="$TMPDIR/yazi-state"
        mkdir -p "$user/plugins"
        ln -s ${pkgs.yaziPlugins.smart-enter} "$user/plugins/smart-enter.yazi"
        printf '%s\n' 'require("smart-enter"):setup { open_multi = false }' > "$user/init.lua"
        printf '%s\n' '[[mgr.prepend_keymap]]' 'on = "l"' 'run = "plugin smart-enter"' > "$user/keymap.toml"

        runtime="$(${yzxYaziMaterializer}/bin/yzx-yazi-config ${yzx}/share/yazelix/yazi "$user" "$state")"
        YAZI_CONFIG_HOME="$runtime" ${pkgs.yazi}/bin/yazi --debug > yazi-debug
        test -f "$runtime/plugins/smart-enter.yazi/main.lua"
        grep -q 'require("smart-enter")' "$runtime/init.lua"
        grep -q 'plugin smart-enter' "$runtime/keymap.toml"
        grep -q 'yzx-open' yazi-debug
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
      key_reference_parity = pkgs.runCommand "key-reference-parity-check" {nativeBuildInputs = [pkgs.rustc pkgs.stdenv.cc];} ''
        rustc --edition=2024 ${./checks/key-reference-parity.rs} -o key-reference-parity-check
        ./key-reference-parity-check ${./crates/yzx-config/src/catalog.rs} ${yzx}/share/yazelix/config.kdl ${./crates/yzx-tutor/src/main.rs}
        touch "$out"
      '';
      contracts = pkgs.runCommand "yzx-contracts" {} ''
        ${yzxContractsCheck}/bin/yzx-contracts-check ${yzx} ${pkgs.git}/bin/git ${pkgs.jq}/bin/jq ${pkgs.nushell}/bin/nu "$out"
      '';
      runtime_contracts = pkgs.runCommand "yzx-runtime-contracts" {} ''
        test -x ${yzxRuntime}/bin/yzx
        test ! -e ${yzxRuntime}/share/applications/yzx.desktop
        ! grep -Fx ${marsPackage} ${runtimeClosure}/store-paths
        ! grep -E '/[^/]*-rio-[^/]*$' ${runtimeClosure}/store-paths

        export HOME="$TMPDIR/home"
        export YAZELIX_CONFIG_HOME="$TMPDIR/config"
        export YAZELIX_STATE_DIR="$TMPDIR/state"
        export XDG_DATA_HOME="$TMPDIR/data"
        mkdir -p "$HOME" "$YAZELIX_CONFIG_HOME" "$YAZELIX_STATE_DIR" "$XDG_DATA_HOME"
        printf '%s\n' '[welcome]' 'enabled = false' > "$YAZELIX_CONFIG_HOME/config.toml"

        ${yzxRuntime}/bin/yzx status --json > status.json
        test "$(${pkgs.jq}/bin/jq -r .package status.json)" = runtime
        ${yzxRuntime}/bin/yzx status > status
        grep -q '^package: runtime$' status
        grep -q '^mars config: not included$' status
        ${yzxRuntime}/bin/yzx doctor > doctor
        grep -q '^ok mars: not included$' doctor
        if ${yzxRuntime}/bin/yzx launch 2> launch-error; then
          printf '%s\n' 'Mars-free runtime launch unexpectedly succeeded' >&2
          exit 1
        fi
        grep -q 'launch is unavailable in the Mars-free runtime package' launch-error
        ${yzxRuntime}/bin/yzx enter --version > enter-version
        grep -q '^zellij ' enter-version
        touch "$out"
      '';
      helix_contracts = pkgs.runCommand "yzx-helix-contracts" {} ''
        ${helixContractsCheck}/bin/helix-contracts-check ${yzx} ${pkgs.nushell}/bin/nu "$out"
      '';
    } // pkgs.lib.optionalAttrs (system == "x86_64-linux") {
      flexnetos_foundation_contracts = let
        foundation = self.packages.${system}.lifeos_foundation_yzx;
        flexnetosNuConfig = pkgs.replaceVars ./nushell/config/config.nu {
          rtkWrappers = "${./nushell/config/rtk_wrappers.nu}";
          stackPromptGuard = "${./nushell/config/stack_prompt_guard.nu}";
          flexnetosInit = "${./nushell/scripts/flexnetos_init.nu}";
          profileNu = "/home/flexnetos/.nix-profile/toolbin/nu";
        };
      in pkgs.runCommand "flexnetos-foundation-contracts" {} ''
        test -x ${foundation}/bin/yzx
        test -x ${foundation}/bin/rtk
        test -x ${foundation}/bin/codex
        test -x ${foundation}/bin/claude
        test -x ${foundation}/bin/ccboard
        test -x ${foundation}/bin/codedb
        test -x ${foundation}/bin/nu_plugin_codedb
        test -x ${foundation}/bin/fxrun
        test -x ${foundation}/bin/fxrun-actions
        test -x ${foundation}/bin/fxrun-dispatch
        test -x ${foundation}/bin/flexnetos_runner_policy
        test -x ${foundation}/bin/flexnetos_runner_service
        test -x ${foundation}/bin/yazelix_host_policy
        test -x ${foundation}/bin/yazelix_volatile_runtime
        test -x ${foundation}/bin/kache
        test -x ${foundation}/bin/kache-rustc-wrapper
        test -x ${foundation}/bin/nix
        test -x ${foundation}/bin/nix-daemon
        test -x ${foundation}/bin/nix-store
        test -x ${foundation}/bin/journalctl
        test -x ${foundation}/bin/systemctl
        test -x ${foundation}/bin/usermod
        test -x ${foundation}/toolbin/nu
        test ! -e ${foundation}/bin/yzx-desktop-launch
        test ! -e ${foundation}/bin/yzx-agent-workspace-launch

        desktop_count="$(find ${foundation}/share/applications -maxdepth 1 -name '*.desktop' | wc -l)"
        test "$desktop_count" = 1
        desktop=${foundation}/share/applications/com.flexnetos.Yazelix.Agent.desktop
        test -f "$desktop"
        test ! -e ${foundation}/share/applications/com.flexnetos.Yazelix.desktop
        test ! -e ${foundation}/share/applications/com.yazelix.Yazelix.Kitty.desktop
        grep -Fx 'Name=FlexNetOS Yazelix Agent' "$desktop"
        grep -Fx 'Exec=/home/flexnetos/.nix-profile/bin/yzx launch' "$desktop"
        grep -Fx 'Icon=/home/flexnetos/.nix-profile/share/pixmaps/yazelix.png' "$desktop"
        grep -Fx 'StartupWMClass=mars' "$desktop"
        grep -Fx 'Categories=System;TerminalEmulator;' "$desktop"
        test -f ${foundation}/share/pixmaps/yazelix.png
        test -s ${foundation}/share/pixmaps/yazelix.png

        layout=${foundation}/configs/zellij/layouts/flexnetos_agent_workspace.kdl
        test -f "$layout"
        grep -F 'tab name="FlexNetOS" focus=true' "$layout"
        grep -F 'tab name="Mission Control"' "$layout"
        ! grep -F '@bar@' "$layout"
        ! grep -F '@yazi@' "$layout"
        test "$(grep -cE 'command="/nix/store/[^/]+-flexnetos-yzx-shell/bin/yzx-shell"' "$layout")" = 2

        config=${foundation}/share/yazelix/config.kdl
        grep -Eq '^default_shell "/nix/store/[^/]+-flexnetos-yzx-shell/bin/yzx-shell"$' "$config"

        test -f ${foundation}/nushell/config/config.nu
        test -f ${foundation}/nushell/config/rtk_wrappers.nu
        test -f ${foundation}/nushell/config/stack_prompt_guard.nu
        test -f ${foundation}/nushell/scripts/flexnetos_init.nu
        grep -F 'source "${flexnetosNuConfig}"' ${foundation}/share/yazelix/nu/config.nu
        grep -F ${./nushell/config/rtk_wrappers.nu} ${flexnetosNuConfig}
        grep -F ${./nushell/scripts/flexnetos_init.nu} ${flexnetosNuConfig}
        ${pkgs.file}/bin/file -L ${foundation}/bin/kache-rustc-wrapper | grep -F ELF
        ${pkgs.file}/bin/file -L ${foundation}/libexec/kache/rustc | grep -F ELF
        runner_unit=${foundation}/lib/systemd/user/flexnetos_runner@.service
        test -f "$runner_unit"
        grep -Fx 'ExecStartPre=/home/flexnetos/.nix-profile/bin/flexnetos_runner_policy runtime %i' "$runner_unit"
        grep -Fx 'ExecStart=/home/flexnetos/.nix-profile/bin/flexnetos_runner_service %i' "$runner_unit"
        grep -Fx 'Environment=SHELL=/home/flexnetos/.nix-profile/toolbin/nu' "$runner_unit"
        grep -Fx 'Environment=KACHE_CACHE_DIR=/home/flexnetos/.cache/kache/runners/%i' "$runner_unit"
        YAZELIX_HOST_POLICY_ROOT=${foundation}/share/yazelix/host-policy \
          ${foundation}/bin/yazelix_host_policy check-bundle
        host_policy_test_root="$TMPDIR/host-policy-root"
        YAZELIX_HOST_POLICY_ROOT=${foundation}/share/yazelix/host-policy \
          YAZELIX_HOST_POLICY_TARGET_ROOT="$host_policy_test_root" \
          ${foundation}/bin/yazelix_host_policy apply-nix
        YAZELIX_HOST_POLICY_ROOT=${foundation}/share/yazelix/host-policy \
          YAZELIX_HOST_POLICY_TARGET_ROOT="$host_policy_test_root" \
          ${foundation}/bin/yazelix_host_policy check-files
        YAZELIX_HOST_POLICY_ROOT=${foundation}/share/yazelix/host-policy \
          YAZELIX_HOST_POLICY_TARGET_ROOT="$host_policy_test_root" \
          ${foundation}/bin/yazelix_host_policy apply-logs
        YAZELIX_HOST_POLICY_ROOT=${foundation}/share/yazelix/host-policy \
          YAZELIX_HOST_POLICY_TARGET_ROOT="$host_policy_test_root" \
          ${foundation}/bin/yazelix_host_policy check-log-files
        grep -Fx 'substitute = false' ${foundation}/share/yazelix/host-policy/nix.conf
        grep -Fx 'substituters =' ${foundation}/share/yazelix/host-policy/nix.conf
        grep -Fx 'trusted-substituters =' ${foundation}/share/yazelix/host-policy/nix.conf
        grep -Fx 'keep-build-log = false' ${foundation}/share/yazelix/host-policy/nix.conf
        grep -Fx 'compress-build-log = false' ${foundation}/share/yazelix/host-policy/nix.conf
        grep -F '"endpoint": null' ${foundation}/share/yazelix/host-policy/determinate-config.json
        grep -Fx '/home/flexnetos/.nix-profile/toolbin/nu' ${foundation}/share/yazelix/host-policy/shells
        grep -Fx 'Storage=none' ${foundation}/share/yazelix/host-policy/journald-no-storage.conf
        grep -F '"log-driver": "none"' ${foundation}/share/yazelix/host-policy/docker-daemon.json
        grep -F '"GenAILocalFoundationalModelSettings": 1' ${foundation}/share/yazelix/host-policy/chrome-storage.json
        grep -F '"DiskCacheDir": "/run/user/1001/yazelix/volatile/cache/google-chrome"' ${foundation}/share/yazelix/host-policy/chrome-storage.json
        grep -Fx 'ExecStart=/home/flexnetos/.nix-profile/bin/yazelix_host_policy apply-nix' ${foundation}/lib/systemd/system/yazelix_host_policy.service
        grep -Fx 'ExecStart=/home/flexnetos/.nix-profile/bin/yazelix_host_policy apply-logs' ${foundation}/lib/systemd/system/yazelix_host_policy.service
        test -f ${foundation}/lib/systemd/system/yazelix_host_policy.path
        test -f ${foundation}/lib/systemd/system/nix-daemon.service.d/10-yazelix-host-policy.conf
        grep -Fx 'ExecStart=@/home/flexnetos/.nix-profile/bin/nix-daemon nix-daemon --daemon' ${foundation}/lib/systemd/system/nix-daemon.service.d/10-yazelix-host-policy.conf
        test -f ${foundation}/lib/systemd/user/yazelix_volatile_runtime.service
        grep -Fx 'ExecStart=/home/flexnetos/.nix-profile/bin/yazelix_volatile_runtime ensure' ${foundation}/lib/systemd/user/yazelix_volatile_runtime.service
        volatile_env=${foundation}/share/yazelix/environment.d/10-yazelix-volatile.conf
        grep -Fx 'XDG_CACHE_HOME=/run/user/1001/yazelix/volatile/cache' "$volatile_env"
        grep -Fx 'TMPDIR=/run/user/1001/yazelix/volatile/tmp' "$volatile_env"
        grep -Fx 'KACHE_CACHE_DIR=/home/flexnetos/.cache/kache' "$volatile_env"
        grep -Fx 'RUSTC_WRAPPER=/home/flexnetos/.nix-profile/bin/kache-rustc-wrapper' "$volatile_env"
        grep -F 'legacy Kache root must not exist' ${./nushell/system/volatile_runtime.nu}
        grep -F 'legacy Kache delivery artifact must not exist' ${./nushell/system/volatile_runtime.nu}

        export HOME="$TMPDIR/home"
        export YAZELIX_CONFIG_HOME="$TMPDIR/config"
        export YAZELIX_STATE_DIR="$TMPDIR/state"
        mkdir -p "$HOME" "$YAZELIX_CONFIG_HOME" "$YAZELIX_STATE_DIR"
        ${foundation}/bin/yzx status > status
        ${foundation}/bin/yzx doctor > doctor
        grep -Fx 'package: full' status
        grep -Fx 'shell: nu' status
        grep -F "runtime identity: $YAZELIX_STATE_DIR/runtime_identity.json" status
        grep -Fx 'ok shell.program: nu' doctor
        grep -F 'ok mars: /nix/store/' doctor
        cmp ${foundation}/share/yazelix/runtime_identity.json "$YAZELIX_STATE_DIR/runtime_identity.json"
        touch "$out"
      '';
    });

    apps = eachSystem (system:
      rec {
        yazelix = {
          type = "app";
          program = "${self.packages.${system}.yazelix}/bin/yzx";
        };
        runtime = {
          type = "app";
          program = "${self.packages.${system}.runtime}/bin/yzx";
        };
        default = yazelix;
      }
      // nixpkgs.lib.optionalAttrs (system == "x86_64-linux") {
        lifeos_foundation_yzx = {
          type = "app";
          program = "${self.packages.${system}.lifeos_foundation_yzx}/bin/yzx";
        };
      });
  };
}
