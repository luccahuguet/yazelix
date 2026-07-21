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
      url = "github:FlexNetOS/yazelix-helix/2657bf0f8e0f183c0e9bca7e6b1b42f75416be7c";
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
      url = "github:FlexNetOS/yazelix-yazi-assets/bd0deff7e83ecd7788b61f5c0cda122272826f74";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    yazelixTerminalSupport = {
      url = "github:FlexNetOS/yazelix-terminal-support/873f64b77eda3a39609d154bda192a2ad8405955";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    ratconfig = {
      url = "github:luccahuguet/ratconfig";
      flake = false;
    };
    beads_rust_source = {
      url = "github:FlexNetOS/beads_rust/2498339168b8e88d641e8ae1664843fc69740012";
      flake = false;
    };
    beads_viewer = {
      url = "github:FlexNetOS/beads_viewer/37d7c2a69797db37d373646ba50e5d0c62d9984a";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rtk_source = {
      url = "github:FlexNetOS/rtk-tokenkill/0dd13a48b81ac083d8a39351a6a72ca4e7b715c0";
      flake = false;
    };
    grit_source = {
      url = "github:FlexNetOS/grit/89d8addd170f408d1d82860c39096929375bd2ce";
      flake = false;
    };
    icm_source = {
      url = "github:FlexNetOS/icm/03d63a9102ce7f2c17cc7df66ac1aded46def88e";
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
    yazelixTerminalSupport,
    ratconfig,
    beads_rust_source,
    beads_viewer,
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
        install -D -m 644 ${flexnetosYaziAssetsRoot}/yazelix_starship.toml "$out/yazelix_starship.toml"
        mkdir -p "$out/plugins"
        install -D -m 644 ${./defaults/yazi/plugins/sidebar-state.yazi/main.lua} "$out/plugins/sidebar-state.yazi/main.lua"
        install -D -m 644 ${./defaults/yazi/plugins/sidebar-status.yazi/main.lua} "$out/plugins/sidebar-status.yazi/main.lua"
        install -D -m 644 ${./defaults/yazi/plugins/zoxide-editor.yazi/main.lua} "$out/plugins/zoxide-editor.yazi/main.lua"
        ln -s ${flexnetosYaziAssetsRoot}/plugins/auto-layout.yazi "$out/plugins/auto-layout.yazi"
        ln -s ${flexnetosYaziAssetsRoot}/plugins/git.yazi "$out/plugins/git.yazi"
        ln -s ${flexnetosYaziAssetsRoot}/plugins/starship.yazi "$out/plugins/starship.yazi"
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
      # The portable asset layer evaluates on every advertised platform.  The
      # mandatory ccboard/CodeDB tooling is a Linux-only Foundation concern:
      # CodeDB retains its upstream Bubblewrap sandbox rather than receiving a
      # fictional Darwin substitute.
      flexnetosYaziAssets = yazelixYaziAssets.packages.${system}.yazi_assets_only;
      flexnetosYaziAssetsRoot = "${flexnetosYaziAssets}/share/yazelix_yazi_assets";
      flexnetosLinuxYaziRuntimeTools =
        assert pkgs.stdenv.hostPlatform.isLinux;
        yazelixYaziAssets.packages.${system}.yazelix_yazi_assets;
      flexnetosTerminalSupport = yazelixTerminalSupport.packages.${system}.yazelix_terminal_support;
      flexnetosTerminalSupportMetadata = builtins.fromTOML (
        builtins.readFile "${yazelixTerminalSupport}/config_metadata/terminal_support.toml"
      );
      flexnetosTerminalSupportContract =
        assert flexnetosTerminalSupportMetadata.schema_version == 2;
        assert flexnetosTerminalSupportMetadata.default_terminal == "mars";
        assert flexnetosTerminalSupportMetadata.launch_order == ["mars"];
        assert flexnetosTerminalSupportMetadata.desktop_id_prefix == "com.flexnetos.Yazelix";
        assert flexnetosTerminalSupportMetadata.terminals.mars.desktop_suffix == "Agent";
        assert flexnetosTerminalSupportMetadata.terminals.mars.startup_wm_class == "mars";
        true;
      flexnetosCcboard = "${flexnetosLinuxYaziRuntimeTools}/share/yazelix_yazi_assets/runtime_tools/ccboard/bin/ccboard";
      flexnetosCodedb = "${flexnetosLinuxYaziRuntimeTools}/share/yazelix_yazi_assets/runtime_tools/codedb/bin/codedb";
      flexnetosNuPluginCodedb = "${flexnetosLinuxYaziRuntimeTools}/share/yazelix_yazi_assets/runtime_tools/codedb/bin/nu_plugin_codedb";
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
        desktopEntrySource ? "",
        desktopDatabaseUpdater ? "",
        defaultStateDir ? "",
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
          inherit desktopEntrySource desktopDatabaseUpdater defaultStateDir;
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
        desktopEntrySource ? "",
        desktopDatabaseUpdater ? "",
        defaultStateDir ? "",
      }: let
        command = mkYzxCommand {
          inherit withMars layoutPackage layoutTemplate configKdl shellPackage extraPathPrefix;
          inherit desktopEntrySource desktopDatabaseUpdater defaultStateDir;
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
              ln -s ${yzxYaziConfig} "$out/share/yazelix/yazi"
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
      flexnetosBeadsViewer = beads_viewer.packages.${system}.bv;
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
      flexnetosRust189Manifest = builtins.fetchurl {
        url = "https://static.rust-lang.org/dist/channel-rust-1.89.0.toml";
        sha256 = "sha256-+9FmLhAOezBZCOziO0Qct1NOrfpjNsXxc/8I0c7BdKE=";
      };
      flexnetosRust189 = fenixPkgs.fromManifestFile flexnetosRust189Manifest;
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
      flexnetosPython = pkgs.python3.withPackages (pythonPackages: [
        pythonPackages.pyyaml
      ]);
      profileEnvironmentFrontdoor = name: payload: nuApplication name ./nushell/system/profile_environment_frontdoor.nu {
        tool = name;
        inherit payload;
        realHome = "/home/flexnetos";
        dataHome = "/home/flexnetos/meta/var/lib";
        stateHome = "/home/flexnetos/meta/var/lib";
        cacheHome = "/run/user/1001/yazelix/volatile/cache";
        runtimeDir = "/run/user/1001";
        yazelixStateDir = "/run/user/1001/yazelix/profile-runtime/yazelix";
        profileNu = "/home/flexnetos/.nix-profile/toolbin/nu";
        chmod = "${pkgs.coreutils}/bin/chmod";
      };
      flexnetosNuFrontdoor = profileEnvironmentFrontdoor "nu" "${pkgs.nushell}/bin/nu";
      flexnetosRtkFrontdoor = profileEnvironmentFrontdoor "rtk" "${flexnetosRtk}/bin/rtk";
      flexnetosRtkNuFrontdoor = profileEnvironmentFrontdoor "rtk_nu" "${flexnetosRtk}/bin/rtk_nu";
      flexnetosCodexOwnedFrontdoor = profileEnvironmentFrontdoor "codex" "${flexnetosCodexFrontdoor}/bin/codex";
      flexnetosClaudeOwnedFrontdoor = profileEnvironmentFrontdoor "claude" "${flexnetosClaudeFrontdoor}/bin/claude";
      flexnetosIcmOwnedFrontdoor = profileEnvironmentFrontdoor "icm" "${flexnetosIcmFrontdoor}/bin/icm";
      flexnetosExecutables = {
        Xvfb = "${pkgs.xorg-server}/bin/Xvfb";
        actionlint = "${pkgs.actionlint}/bin/actionlint";
        ar = "${pkgs.binutils}/bin/ar";
        awk = "${pkgs.gawk}/bin/awk";
        bash = "${pkgs.bash}/bin/bash";
        basename = "${pkgs.coreutils}/bin/basename";
        br = "${flexnetosBeads}/bin/br";
        bv = "${flexnetosBeadsViewer}/bin/bv";
        bun = "${flexnetosBun}/bin/bun";
        bunx = "${flexnetosBun}/bin/bunx";
        bzip2 = "${pkgs.bzip2}/bin/bzip2";
        cargo = "${flexnetosRustToolchain}/bin/cargo";
        cargo-audit = "${pkgs.cargo-audit}/bin/cargo-audit";
        cargo-clippy = "${flexnetosRustToolchain}/bin/cargo-clippy";
        cargo-fmt = "${flexnetosRustToolchain}/bin/cargo-fmt";
        "cargo-msrv-1.89" = "${flexnetosRust189Lane}/bin/cargo-msrv-1.89";
        cargo-tauri = "${pkgs.cargo-tauri}/bin/cargo-tauri";
        cc = "${pkgs.stdenv.cc}/bin/cc";
        ccboard = flexnetosCcboard;
        cat = "${pkgs.coreutils}/bin/cat";
        clang = "${pkgs.clang}/bin/clang";
        "clang++" = "${pkgs.clang}/bin/clang++";
        chmod = "${pkgs.coreutils}/bin/chmod";
        claude = "${flexnetosClaudeOwnedFrontdoor}/bin/claude";
        clippy-driver = "${flexnetosRustToolchain}/bin/clippy-driver";
        cmake = "${pkgs.cmake}/bin/cmake";
        codedb = flexnetosCodedb;
        codex = "${flexnetosCodexOwnedFrontdoor}/bin/codex";
        cp = "${pkgs.coreutils}/bin/cp";
        curl = "${pkgs.curl}/bin/curl";
        cut = "${pkgs.coreutils}/bin/cut";
        date = "${pkgs.coreutils}/bin/date";
        dirname = "${pkgs.coreutils}/bin/dirname";
        env = "${pkgs.coreutils}/bin/env";
        file = "${pkgs.file}/bin/file";
        find = "${pkgs.findutils}/bin/find";
        fxrun = "${flexnetosRunner}/bin/fxrun";
        "fxrun-actions" = "${flexnetosRunner}/bin/fxrun-actions";
        "fxrun-dispatch" = "${flexnetosRunner}/bin/fxrun-dispatch";
        flexnetos_runner_policy = "${flexnetosRunnerPolicy}/bin/flexnetos_runner_policy";
        flexnetos_runner_service = "${flexnetosRunnerService}/bin/flexnetos_runner_service";
        yazelix_host_policy = "${flexnetosHostPolicy}/bin/yazelix_host_policy";
        yazelix_volatile_runtime = "${flexnetosVolatileRuntime}/bin/yazelix_volatile_runtime";
        gh = "${pkgs.gh}/bin/gh";
        git = "${pkgs.git}/bin/git";
        git-kb = "${flexnetosGitKb}/bin/git-kb";
        grep = "${pkgs.gnugrep}/bin/grep";
        grit = "${flexnetosGrit}/bin/grit";
        gzip = "${pkgs.gzip}/bin/gzip";
        head = "${pkgs.coreutils}/bin/head";
        home-manager = "${home-manager.packages.${system}.default}/bin/home-manager";
        icm = "${flexnetosIcmOwnedFrontdoor}/bin/icm";
        jq = "${pkgs.jq}/bin/jq";
        kache = "${flexnetosKache}/bin/kache";
        kache-rustc-wrapper = "${flexnetosKache}/bin/kache-rustc-wrapper";
        "ld.wild" = "${pkgs.wild}/bin/ld.wild";
        loop = "${flexnetosMeta}/bin/loop";
        meta = "${flexnetosMeta}/bin/meta";
        meta-git = "${flexnetosMeta}/bin/meta-git";
        meta-mcp = "${flexnetosMeta}/bin/meta-mcp";
        meta-project = "${flexnetosMeta}/bin/meta-project";
        mkdir = "${pkgs.coreutils}/bin/mkdir";
        mv = "${pkgs.coreutils}/bin/mv";
        ninja = "${pkgs.ninja}/bin/ninja";
        node = "${pkgs.nodejs_24}/bin/node";
        nix = "${pkgs.nix}/bin/nix";
        nix-build = "${pkgs.nix}/bin/nix-build";
        nix-daemon = "${pkgs.nix}/bin/nix-daemon";
        nix-env = "${pkgs.nix}/bin/nix-env";
        nix-instantiate = "${pkgs.nix}/bin/nix-instantiate";
        nix-shell = "${pkgs.nix}/bin/nix-shell";
        nix-store = "${pkgs.nix}/bin/nix-store";
        journalctl = "${pkgs.systemd}/bin/journalctl";
        ln = "${pkgs.coreutils}/bin/ln";
        notebooklm = "${flexnetosNotebooklm}/bin/notebooklm";
        nvim = "${pkgs.neovim}/bin/nvim";
        nu = "${flexnetosNuFrontdoor}/bin/nu";
        nu_plugin_codedb = flexnetosNuPluginCodedb;
        obscura = "${flexnetosObscura}/bin/obscura";
        openssl = "${pkgs.openssl}/bin/openssl";
        pkg-config = "${pkgs.pkg-config}/bin/pkg-config";
        python3 = "${flexnetosPython}/bin/python3";
        readlink = "${pkgs.coreutils}/bin/readlink";
        realpath = "${pkgs.coreutils}/bin/realpath";
        rg = "${pkgs.ripgrep}/bin/rg";
        rm = "${pkgs.coreutils}/bin/rm";
        rtk = "${flexnetosRtkFrontdoor}/bin/rtk";
        scp = "${pkgs.openssh}/bin/scp";
        sed = "${pkgs.gnused}/bin/sed";
        sh = "${pkgs.bash}/bin/sh";
        sha256sum = "${pkgs.coreutils}/bin/sha256sum";
        sort = "${pkgs.coreutils}/bin/sort";
        ssh = "${pkgs.openssh}/bin/ssh";
        stat = "${pkgs.coreutils}/bin/stat";
        rtk_nu = "${flexnetosRtkNuFrontdoor}/bin/rtk_nu";
        systemctl = "${pkgs.systemd}/bin/systemctl";
        rust-analyzer = "${flexnetosRustToolchain}/bin/rust-analyzer";
        rustc = "${flexnetosRustToolchain}/bin/rustc";
        "rustc-msrv-1.89" = "${flexnetosRust189Lane}/bin/rustc-msrv-1.89";
        rustdoc = "${flexnetosRustToolchain}/bin/rustdoc";
        "rustdoc-msrv-1.89" = "${flexnetosRust189Lane}/bin/rustdoc-msrv-1.89";
        rustfmt = "${flexnetosRustToolchain}/bin/rustfmt";
        tail = "${pkgs.coreutils}/bin/tail";
        tar = "${pkgs.gnutar}/bin/tar";
        tee = "${pkgs.coreutils}/bin/tee";
        timeout = "${pkgs.coreutils}/bin/timeout";
        touch = "${pkgs.coreutils}/bin/touch";
        tr = "${pkgs.coreutils}/bin/tr";
        sqld = "${pkgs.sqld}/bin/sqld";
        sqlite3 = "${pkgs.sqlite}/bin/sqlite3";
        tu = "${tokenusage}/bin/tu";
        uname = "${pkgs.coreutils}/bin/uname";
        usermod = "${pkgs.shadow}/bin/usermod";
        uv = "${pkgs.uv}/bin/uv";
        uvx = "${pkgs.uv}/bin/uvx";
        wasm-pack = "${pkgs.wasm-pack}/bin/wasm-pack";
        wc = "${pkgs.coreutils}/bin/wc";
        weave = "${flexnetosWeave}/bin/weave";
        wild = "${pkgs.wild}/bin/wild";
        xargs = "${pkgs.findutils}/bin/xargs";
        xz = "${pkgs.xz}/bin/xz";
        x86_64-linux-musl-ar = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-ar";
        "x86_64-linux-musl-g++" = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-g++";
        x86_64-linux-musl-gcc = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-gcc";
        x86_64-linux-musl-ranlib = "${flexnetosMuslToolchain}/bin/x86_64-linux-musl-ranlib";
        x86_64-unknown-linux-musl-ar = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-ar";
        "x86_64-unknown-linux-musl-g++" = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-g++";
        x86_64-unknown-linux-musl-gcc = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-gcc";
        x86_64-unknown-linux-musl-ranlib = "${flexnetosMuslToolchain}/bin/x86_64-unknown-linux-musl-ranlib";
      };
      flexnetosUtilityPackages = with pkgs; [
        bash
        bzip2
        coreutils
        curl
        debianutils
        diffutils
        findutils
        gawk
        gh
        git
        gnugrep
        gnused
        gnutar
        gzip
        jq
        openssh
        patch
        procps
        flexnetosPython
        ripgrep
        util-linux
        which
        xz
      ];
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
        + ''
          for package in ${pkgs.lib.escapeShellArgs flexnetosUtilityPackages}; do
            for executable in "$package"/bin/*; do
              test -x "$executable" || continue
              name="''${executable##*/}"
              if ! test -e "$out/bin/$name"; then
                ln -s "$executable" "$out/bin/$name"
              fi
              if ! test -e "$out/toolbin/$name"; then
                ln -s "$executable" "$out/toolbin/$name"
              fi
            done
          done
        ''
      );
      # YZXCONV-003: single-profile closure contract tools. The check verifies
      # that ~/.nix-profile is the sole foundation selector; the migration
      # performs the cutover (dry-run by default) with a rollback receipt.
      flexnetosProfileTools = pkgs.runCommand "flexnetos-profile-tools" {} ''
        mkdir -p "$out/bin" "$out/share/yazelix/packaging"
        install -m 644 ${./packaging/single_profile_check.nu} \
          "$out/share/yazelix/packaging/single_profile_check.nu"
        install -m 644 ${./packaging/profile_migration.nu} \
          "$out/share/yazelix/packaging/profile_migration.nu"
        cat > "$out/bin/yazelix_profile_check" <<EOF
        #!${pkgs.nushell}/bin/nu
        def --wrapped main [...args] {
          exec ${pkgs.nushell}/bin/nu "$out/share/yazelix/packaging/single_profile_check.nu" ...\$args
        }
        EOF
        cat > "$out/bin/yazelix_profile_migrate" <<EOF
        #!${pkgs.nushell}/bin/nu
        def --wrapped main [...args] {
          \$env.YZX_CHECK_SCRIPT = "$out/share/yazelix/packaging/single_profile_check.nu"
          \$env.YZX_NIX_BIN = (\$env.YZX_NIX_BIN? | default "${pkgs.nix}/bin/nix")
          \$env.YZX_NIX_STORE_BIN = (\$env.YZX_NIX_STORE_BIN? | default "${pkgs.nix}/bin/nix-store")
          \$env.YZX_NU_BIN = "${pkgs.nushell}/bin/nu"
          \$env.YZX_READLINK_BIN = "${pkgs.coreutils}/bin/readlink"
          \$env.YZX_MV_BIN = "${pkgs.coreutils}/bin/mv"
          \$env.YZX_DATE_BIN = "${pkgs.coreutils}/bin/date"
          exec ${pkgs.nushell}/bin/nu "$out/share/yazelix/packaging/profile_migration.nu" ...\$args
        }
        EOF
        chmod +x "$out/bin/yazelix_profile_check" "$out/bin/yazelix_profile_migrate"
      '';
      flexnetosCodexConfigOwner = pkgs.runCommand "yazelix-codex-config-owner" {} ''
        mkdir -p "$out/bin" "$out/share/yazelix/agent_configs/codex" \
          "$out/share/yazelix/nushell/scripts"
        install -m 644 ${./agent_configs/codex/config.toml.src} \
          "$out/share/yazelix/agent_configs/codex/config.toml.src"
        install -m 644 ${./agent_configs/codex/RULES.md.src} \
          "$out/share/yazelix/agent_configs/codex/RULES.md.src"
        install -m 644 ${./nushell/scripts/materialize_codex_config.nu} \
          "$out/share/yazelix/nushell/scripts/materialize_codex_config.nu"
        cat > "$out/bin/yazelix_codex_materialize" <<EOF
        #!${pkgs.nushell}/bin/nu
        def --wrapped main [...args] {
          if (\$args | is-empty) or \$args == ["--recover-only"] {
            let profile = "/home/flexnetos/.nix-profile"
            let codex_home = "/run/user/1001/yazelix/profile-runtime/codex"
            exec ${pkgs.nushell}/bin/nu \
              "$out/share/yazelix/nushell/scripts/materialize_codex_config.nu" \
              (\$profile | path join "share/yazelix/agent_configs/codex/config.toml.src") \
              (\$codex_home | path join "config.toml") \
              (\$profile | path join "share/yazelix/agent_configs/codex/RULES.md.src") \
              (\$codex_home | path join "RULES.md") ...\$args
          }
          exec ${pkgs.nushell}/bin/nu \
            "$out/share/yazelix/nushell/scripts/materialize_codex_config.nu" ...\$args
        }
        EOF
        chmod +x "$out/bin/yazelix_codex_materialize"
      '';
      flexnetosClaudeConfigOwner = pkgs.runCommand "yazelix-claude-config-owner" {} ''
        mkdir -p "$out/bin" "$out/share/yazelix/agent_configs/claude" \
          "$out/share/yazelix/nushell/scripts"
        install -m 644 ${./agent_configs/claude/settings.json.src} \
          "$out/share/yazelix/agent_configs/claude/settings.json.src"
        install -m 644 ${./agent_configs/claude/CLAUDE.md.src} \
          "$out/share/yazelix/agent_configs/claude/CLAUDE.md.src"
        install -m 644 ${./agent_configs/claude/RTK.md.src} \
          "$out/share/yazelix/agent_configs/claude/RTK.md.src"
        install -m 644 ${./nushell/scripts/materialize_claude_config.nu} \
          "$out/share/yazelix/nushell/scripts/materialize_claude_config.nu"
        cat > "$out/bin/yazelix_claude_materialize" <<EOF
        #!${pkgs.nushell}/bin/nu
        def --wrapped main [...args] {
          if (\$args | is-empty) {
            let profile = "/home/flexnetos/.nix-profile"
            let claude_home = "/home/flexnetos/meta/var/lib/claude"
            exec ${pkgs.nushell}/bin/nu \
              "$out/share/yazelix/nushell/scripts/materialize_claude_config.nu" \
              (\$profile | path join "share/yazelix/agent_configs/claude/settings.json.src") \
              (\$claude_home | path join "settings.json") \
              (\$profile | path join "share/yazelix/agent_configs/claude/CLAUDE.md.src") \
              (\$claude_home | path join "CLAUDE.md") \
              (\$profile | path join "share/yazelix/agent_configs/claude/RTK.md.src") \
              (\$claude_home | path join "RTK.md")
          }
          exec ${pkgs.nushell}/bin/nu \
            "$out/share/yazelix/nushell/scripts/materialize_claude_config.nu" ...\$args
        }
        EOF
        chmod +x "$out/bin/yazelix_claude_materialize"
      '';
      flexnetosCodexFrontdoor = nuApplication "codex" ./nushell/agent/profile_frontdoor.nu {
        agent = "codex";
        stateHome = "/run/user/1001/yazelix/profile-runtime/codex";
        payload = "${flexnetosCodex}/bin/codex";
        materializer = "/home/flexnetos/.nix-profile/bin/yazelix_codex_materialize";
        chmod = "${pkgs.coreutils}/bin/chmod";
      };
      flexnetosClaudeFrontdoor = nuApplication "claude" ./nushell/agent/profile_frontdoor.nu {
        agent = "claude";
        stateHome = "/home/flexnetos/meta/var/lib/claude";
        payload = "${flexnetosClaude}/bin/claude";
        materializer = "/home/flexnetos/.nix-profile/bin/yazelix_claude_materialize";
        chmod = "${pkgs.coreutils}/bin/chmod";
      };
      flexnetosIcmFrontdoor = nuApplication "icm" ./nushell/agent/icm_profile_frontdoor.nu {
        payload = "${flexnetosIcm}/bin/icm";
        defaultDb = "/home/flexnetos/meta/var/lib/icm/memories.db";
      };
      flexnetosDesktopSource = pkgs.makeDesktopItem {
        name = "com.flexnetos.Yazelix.Agent";
        destination = "/share/applications";
        desktopName = "FlexNetOS Yazelix Agent";
        genericName = "Terminal Emulator";
        comment = "Yazelix Nova with the profile-owned FlexNetOS agent workspace";
        exec = "/home/flexnetos/.nix-profile/bin/yzx launch";
        icon = "/home/flexnetos/.nix-profile/share/pixmaps/yazelix.png";
        terminal = false;
        categories = ["System" "TerminalEmulator"];
        startupNotify = true;
        startupWMClass = "mars";
        extraConfig = {
          "X-Yazelix-Managed" = "true";
          "X-FlexNetOS-Managed" = "true";
        };
      };
      flexnetosClaudeDesktopSource = pkgs.makeDesktopItem {
        name = "claude-code-url-handler";
        destination = "/share/applications";
        desktopName = "Claude Code URL Handler";
        comment = "Handle claude-cli deep links through the profile-owned Claude frontdoor";
        exec = "/home/flexnetos/.nix-profile/bin/claude --handle-uri %u";
        terminal = false;
        noDisplay = true;
        extraConfig = {
          "MimeType" = "x-scheme-handler/claude-cli;";
          "X-FlexNetOS-Managed" = "true";
        };
      };
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
        defaultStateDir = "/run/user/1001/yazelix/profile-runtime/yazelix";
      };
      lifeosFoundationYzx = assert flexnetosTerminalSupportContract; pkgs.symlinkJoin {
        name = "lifeos-foundation-yzx";
        paths = [flexnetosYzxBase flexnetosTools flexnetosProfileTools flexnetosCodexConfigOwner flexnetosClaudeConfigOwner flexnetosDesktopSource flexnetosClaudeDesktopSource flexnetosTerminalSupport flexnetosRunnerSystemd flexnetosHostPolicyBundle flexnetosVolatileRuntimeBundle];
        nativeBuildInputs = [pkgs.desktop-file-utils];
        postBuild = ''
          install -D -m 644 ${flexnetosZellijLayout}/layout.kdl \
            "$out/configs/zellij/layouts/flexnetos_agent_workspace.kdl"
          install -D -m 644 ${./nushell/config/config.nu} "$out/nushell/config/config.nu"
          install -D -m 644 ${./nushell/config/stack_prompt_guard.nu} "$out/nushell/config/stack_prompt_guard.nu"
          install -D -m 644 ${./nushell/config/rtk_wrappers.nu} "$out/nushell/config/rtk_wrappers.nu"
          install -D -m 644 ${./nushell/scripts/flexnetos_init.nu} "$out/nushell/scripts/flexnetos_init.nu"
          install -D -m 644 ${./nushell/system/profile_environment_frontdoor.nu} \
            "$out/nushell/system/profile_environment_frontdoor.nu"

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

    # The documented Teri Rust workflow enters this shell.  Keep its toolchain
    # profile-owned and provide the native OpenSSL/pkg-config boundary required
    # by Teri's existing dependency graph.
    devShells.x86_64-linux.ci = let
      pkgs = import nixpkgs {system = "x86_64-linux";};
    in pkgs.mkShell {
      packages = [self.packages.x86_64-linux.lifeos_foundation_yzx];
      nativeBuildInputs = [pkgs.pkg-config];
      buildInputs = [pkgs.openssl];
    };

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
      profile_agent_frontdoors = pkgs.runCommand "profile-agent-frontdoors" {
        nativeBuildInputs = [pkgs.nushell pkgs.coreutils];
      } ''
        ${pkgs.nushell}/bin/nu ${./tests/profile_agent_frontdoor.nu} \
          "$TMPDIR/profile-agent-frontdoors" \
          ${./nushell/agent/profile_frontdoor.nu} \
          ${pkgs.nushell}/bin/nu \
          ${pkgs.coreutils}/bin/chmod
        touch "$out"
      '';
      profile_environment_frontdoor = pkgs.runCommand "profile-environment-frontdoor" {
        nativeBuildInputs = [pkgs.nushell pkgs.coreutils];
      } ''
        ${pkgs.nushell}/bin/nu ${./tests/profile_environment_frontdoor.nu} \
          "$TMPDIR/profile-environment-frontdoor" \
          ${./nushell/system/profile_environment_frontdoor.nu} \
          ${pkgs.nushell}/bin/nu \
          ${pkgs.coreutils}/bin/chmod
        touch "$out"
      '';
      icm_profile_frontdoor = pkgs.runCommand "icm-profile-frontdoor" {
        nativeBuildInputs = [pkgs.nushell pkgs.coreutils];
      } ''
        ${pkgs.nushell}/bin/nu ${./tests/icm_profile_frontdoor.nu} \
          "$TMPDIR" \
          ${./nushell/agent/icm_profile_frontdoor.nu} \
          ${pkgs.nushell}/bin/nu \
          ${pkgs.coreutils}/bin/chmod
        touch "$out"
      '';
      strict_profile_sources = pkgs.runCommand "strict-profile-sources" {
        nativeBuildInputs = [pkgs.nushell];
      } ''
        ${pkgs.nushell}/bin/nu ${./tests/strict_profile_sources.nu} ${./.}
        touch "$out"
      '';
      inherit yzx;
      cache_shell_policy = pkgs.runCommand "cache-shell-policy-check" {} ''
        ${pkgs.nushell}/bin/nu ${./checks/cache_shell_policy.nu} ${./.}
        touch "$out"
      '';
      codex_config_materializer = pkgs.runCommand "codex-config-materializer-check" {} ''
        ${pkgs.nushell}/bin/nu ${./tests/codex_config_materializer.nu} ${./.}
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
        for plugin in auto-layout git sidebar-state sidebar-status starship zoxide-editor; do
          test -f "$runtime/plugins/$plugin.yazi/main.lua"
        done
        test -f "$runtime/yazelix_starship.toml"
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
        ${yzxContractsCheck}/bin/yzx-contracts-check \
          ${yzx} ${pkgs.git}/bin/git ${pkgs.jq}/bin/jq ${pkgs.nushell}/bin/nu "$out" \
          ${./README.md} ${./docs/installation.md} ${./docs/development.md} ${./AGENTS.md}
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
          stackPromptGuard = "${./nushell/config/stack_prompt_guard.nu}";
          flexnetosInit = "${./nushell/scripts/flexnetos_init.nu}";
          profileNu = "/home/flexnetos/.nix-profile/toolbin/nu";
        };
      in pkgs.runCommand "flexnetos-foundation-contracts" {} ''
        test -x ${foundation}/bin/yzx
        test -x ${foundation}/bin/br
        test -x ${foundation}/bin/bv
        ${foundation}/bin/bv --version | grep -Fx 'bv v0.16.1'
        test -x ${foundation}/bin/rtk
        test -x ${foundation}/bin/rtk_nu
        test -x ${foundation}/bin/nvim
        test -x ${foundation}/bin/bun
        test -x ${foundation}/bin/bunx
        test -x ${foundation}/bin/ar
        test -x ${foundation}/bin/python3
        ${foundation}/bin/python3 -c 'import yaml; assert yaml.safe_load("ready: true") == {"ready": True}'
        ${foundation}/bin/rtk --version | grep -F '0.43.0'
        ${foundation}/bin/rtk_nu --help | grep -F 'lossless Nushell ingestion envelope'
        PATH=${foundation}/bin:$PATH ${foundation}/bin/rtk_nu --format json -- \
          ${pkgs.coreutils}/bin/printf rtk-nu-proof > rtk-nu-proof.json
        grep -F '"schema_version": "flexnetos.rtk_nu.envelope.v1"' rtk-nu-proof.json
        grep -F '"payload_base64": "cnRrLW51LXByb29m"' rtk-nu-proof.json
        ${foundation}/bin/nvim --version | grep -F 'NVIM v'
        test "$(${foundation}/bin/bun --version)" = 1.3.14
        test ! -e ${foundation}/bin/npm
        test ! -e ${foundation}/bin/npx
        test ! -e ${foundation}/bin/pnpm
        test ! -e ${foundation}/bin/corepack
        test ! -e ${foundation}/bin/yarn
        test -x ${foundation}/bin/codex
        test -x ${foundation}/bin/claude
        test -x ${foundation}/bin/chmod
        readlink -f ${foundation}/bin/codex | grep -Eq '/nix/store/[a-z0-9]+-codex/bin/codex$'
        readlink -f ${foundation}/bin/claude | grep -Eq '/nix/store/[a-z0-9]+-claude/bin/claude$'
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
        test -x ${foundation}/bin/ln
        test -x ${foundation}/bin/systemctl
        test -x ${foundation}/bin/usermod
        test -x ${foundation}/toolbin/nu
        test -x ${foundation}/bin/yazelix_profile_check
        test -x ${foundation}/bin/yazelix_profile_migrate
        test -x ${foundation}/bin/yazelix_codex_materialize
        test -x ${foundation}/bin/yazelix_claude_materialize
        test ! -e ${foundation}/runtime
        test -f ${foundation}/share/yazelix/packaging/single_profile_check.nu
        test -f ${foundation}/share/yazelix/packaging/profile_migration.nu
        test -f ${foundation}/share/yazelix/agent_configs/codex/config.toml.src
        test -f ${foundation}/share/yazelix/agent_configs/codex/RULES.md.src
        test -f ${foundation}/share/yazelix/nushell/scripts/materialize_codex_config.nu
        codex_test_runtime="$TMPDIR/codex-owner-runtime"
        mkdir -p "$codex_test_runtime"
        ${foundation}/bin/yazelix_codex_materialize \
          ${foundation}/share/yazelix/agent_configs/codex/config.toml.src \
          "$codex_test_runtime/config.toml" \
          ${foundation}/share/yazelix/agent_configs/codex/RULES.md.src \
          "$codex_test_runtime/RULES.md"
        ${foundation}/bin/yazelix_codex_materialize \
          ${foundation}/share/yazelix/agent_configs/codex/config.toml.src \
          "$codex_test_runtime/config.toml" \
          ${foundation}/share/yazelix/agent_configs/codex/RULES.md.src \
          "$codex_test_runtime/RULES.md" --recover-only \
          | grep -F 'no pending Codex config/rules transaction'
        grep -F 'GENERATED by yazelix codex config materializer' "$codex_test_runtime/config.toml"
        grep -F 'GENERATED by yazelix codex rules materializer' "$codex_test_runtime/RULES.md"
        test "$(stat -c %a "$codex_test_runtime/config.toml")" = 644
        test "$(stat -c %a "$codex_test_runtime/RULES.md")" = 644
        test -f ${foundation}/share/yazelix/agent_configs/claude/settings.json.src
        test -f ${foundation}/share/yazelix/agent_configs/claude/CLAUDE.md.src
        test -f ${foundation}/share/yazelix/agent_configs/claude/RTK.md.src
        test -f ${foundation}/share/yazelix/nushell/scripts/materialize_claude_config.nu
        claude_test_runtime="$TMPDIR/claude-owner-runtime"
        mkdir -p "$claude_test_runtime"
        touch "$claude_test_runtime/.credentials.json"
        chmod 600 "$claude_test_runtime/.credentials.json"
        claude_credentials_before="$(${pkgs.coreutils}/bin/sha256sum "$claude_test_runtime/.credentials.json")"
        ${foundation}/bin/yazelix_claude_materialize \
          ${foundation}/share/yazelix/agent_configs/claude/settings.json.src \
          "$claude_test_runtime/settings.json" \
          ${foundation}/share/yazelix/agent_configs/claude/CLAUDE.md.src \
          "$claude_test_runtime/CLAUDE.md" \
          ${foundation}/share/yazelix/agent_configs/claude/RTK.md.src \
          "$claude_test_runtime/RTK.md"
        ${foundation}/bin/yazelix_claude_materialize \
          ${foundation}/share/yazelix/agent_configs/claude/settings.json.src \
          "$claude_test_runtime/settings.json" \
          ${foundation}/share/yazelix/agent_configs/claude/CLAUDE.md.src \
          "$claude_test_runtime/CLAUDE.md" \
          ${foundation}/share/yazelix/agent_configs/claude/RTK.md.src \
          "$claude_test_runtime/RTK.md"
        ${pkgs.jq}/bin/jq -e '.hooks.PreToolUse and .hooks.SessionEnd' "$claude_test_runtime/settings.json"
        grep -F '/home/flexnetos/.nix-profile/toolbin/rtk hook claude' "$claude_test_runtime/settings.json"
        grep -F '/home/flexnetos/.nix-profile/toolbin/icm hook pre' "$claude_test_runtime/settings.json"
        grep -F '/home/flexnetos/.nix-profile/toolbin/icm hook end' "$claude_test_runtime/settings.json"
        cmp ${foundation}/share/yazelix/agent_configs/claude/settings.json.src "$claude_test_runtime/settings.json"
        cmp ${foundation}/share/yazelix/agent_configs/claude/CLAUDE.md.src "$claude_test_runtime/CLAUDE.md"
        cmp ${foundation}/share/yazelix/agent_configs/claude/RTK.md.src "$claude_test_runtime/RTK.md"
        test "$(stat -c %a "$claude_test_runtime/settings.json")" = 600
        test "$(stat -c %a "$claude_test_runtime/CLAUDE.md")" = 644
        test "$(stat -c %a "$claude_test_runtime/RTK.md")" = 644
        test "$(stat -c %a "$claude_test_runtime/.yazelix-claude-generation.json")" = 600
        ${pkgs.jq}/bin/jq -e \
          '.schema == "yazelix.claude-config-generation.v1" and (.sources | length == 3)' \
          "$claude_test_runtime/.yazelix-claude-generation.json"
        test "$claude_credentials_before" = "$(${pkgs.coreutils}/bin/sha256sum "$claude_test_runtime/.credentials.json")"
        test ! -e ${foundation}/bin/yzx-desktop-launch
        test ! -e ${foundation}/bin/yzx-agent-workspace-launch

        desktop_count="$(find ${foundation}/share/applications -maxdepth 1 -name '*.desktop' | wc -l)"
        test "$desktop_count" = 2
        desktop=${foundation}/share/applications/com.flexnetos.Yazelix.Agent.desktop
        claude_desktop=${foundation}/share/applications/claude-code-url-handler.desktop
        test -f "$desktop"
        test -f "$claude_desktop"
        test ! -e ${foundation}/share/applications/com.flexnetos.Yazelix.desktop
        test ! -e ${foundation}/share/applications/com.yazelix.Yazelix.Kitty.desktop
        grep -Fx 'Name=FlexNetOS Yazelix Agent' "$desktop"
        grep -Fx 'GenericName=Terminal Emulator' "$desktop"
        grep -Fx 'Exec=/home/flexnetos/.nix-profile/bin/yzx launch' "$desktop"
        grep -Fx 'Icon=/home/flexnetos/.nix-profile/share/pixmaps/yazelix.png' "$desktop"
        grep -Fx 'StartupNotify=true' "$desktop"
        grep -Fx 'StartupWMClass=mars' "$desktop"
        grep -Fx 'Categories=System;TerminalEmulator' "$desktop"
        grep -Fx 'X-Yazelix-Managed=true' "$desktop"
        grep -Fx 'X-FlexNetOS-Managed=true' "$desktop"
        grep -Fx 'Name=Claude Code URL Handler' "$claude_desktop"
        grep -Fx 'Exec=/home/flexnetos/.nix-profile/bin/claude --handle-uri %u' "$claude_desktop"
        grep -Fx 'NoDisplay=true' "$claude_desktop"
        grep -Fx 'MimeType=x-scheme-handler/claude-cli;' "$claude_desktop"
        grep -Fx 'X-FlexNetOS-Managed=true' "$claude_desktop"
        test -f ${foundation}/share/pixmaps/yazelix.png
        test -s ${foundation}/share/pixmaps/yazelix.png
        terminal_metadata=${foundation}/share/yazelix_terminal_support/terminal_support.toml
        test -f "$terminal_metadata"
        ${pkgs.python3}/bin/python - "$terminal_metadata" "$desktop" <<'PY'
        import pathlib
        import sys
        import tomllib

        metadata_path = pathlib.Path(sys.argv[1])
        desktop_path = pathlib.Path(sys.argv[2])
        with metadata_path.open("rb") as metadata_file:
            metadata = tomllib.load(metadata_file)
        mars = metadata["terminals"][metadata["default_terminal"]]
        expected_name = f"{metadata['desktop_id_prefix']}.{mars['desktop_suffix']}.desktop"
        assert metadata["schema_version"] == 2
        assert metadata["launch_order"] == ["mars"]
        assert desktop_path.name == expected_name
        assert f"StartupWMClass={mars['startup_wm_class']}" in desktop_path.read_text()
        PY

        ! ${foundation}/bin/yzx desktop install --print-path

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
        test -f ${foundation}/nushell/config/stack_prompt_guard.nu
        test -f ${foundation}/nushell/config/rtk_wrappers.nu
        test -f ${foundation}/nushell/scripts/flexnetos_init.nu
        test -f ${foundation}/nushell/system/profile_environment_frontdoor.nu
        grep -F 'use rtk_wrappers.nu *' ${foundation}/nushell/config/config.nu
        grep -F 'XDG_DATA_HOME = $DATA_HOME' ${foundation}/nushell/system/profile_environment_frontdoor.nu
        grep -F 'source "${flexnetosNuConfig}"' ${foundation}/share/yazelix/nu/config.nu
        grep -F ${./nushell/scripts/flexnetos_init.nu} ${flexnetosNuConfig}
        ${pkgs.file}/bin/file -L ${foundation}/bin/kache-rustc-wrapper | grep -F ELF
        ${pkgs.file}/bin/file -L ${foundation}/libexec/kache/rustc | grep -F ELF
        runner_unit=${foundation}/lib/systemd/user/flexnetos_runner@.service
        test -f "$runner_unit"
        grep -Fx 'ExecStartPre=/home/flexnetos/.nix-profile/bin/flexnetos_runner_policy runtime %i' "$runner_unit"
        grep -Fx 'ExecStart=/home/flexnetos/.nix-profile/bin/flexnetos_runner_service %i' "$runner_unit"
        grep -Fx 'Environment=SHELL=/home/flexnetos/.nix-profile/toolbin/nu' "$runner_unit"
        grep -Fx 'Environment=KACHE_CACHE_DIR=/home/flexnetos/.cache/kache/runners/%i' "$runner_unit"
        grep -Fx 'Environment=CODEX_HOME=/run/user/1001/yazelix/profile-runtime/codex' "$runner_unit"
        grep -Fx 'Environment=CLAUDE_CONFIG_DIR=/home/flexnetos/meta/var/lib/claude' "$runner_unit"
        grep -Fx 'Environment=XDG_DATA_HOME=/home/flexnetos/meta/var/lib' "$runner_unit"
        grep -Fx 'Environment=XDG_STATE_HOME=/home/flexnetos/meta/var/lib' "$runner_unit"
        grep -Fx 'Environment=YAZELIX_STATE_DIR=/run/user/1001/yazelix/runners/%i/yazelix' "$runner_unit"
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
        test -f ${foundation}/lib/systemd/user/yazelix_volatile_runtime.service
        grep -Fx 'ExecStart=/home/flexnetos/.nix-profile/bin/yazelix_volatile_runtime ensure' ${foundation}/lib/systemd/user/yazelix_volatile_runtime.service
        volatile_env=${foundation}/share/yazelix/environment.d/10-yazelix-volatile.conf
        grep -Fx 'XDG_CACHE_HOME=/run/user/1001/yazelix/volatile/cache' "$volatile_env"
        grep -Fx 'XDG_DATA_HOME=/home/flexnetos/meta/var/lib' "$volatile_env"
        grep -Fx 'XDG_STATE_HOME=/home/flexnetos/meta/var/lib' "$volatile_env"
        grep -Fx 'YAZELIX_STATE_DIR=/run/user/1001/yazelix/profile-runtime/yazelix' "$volatile_env"
        grep -Fx 'TMPDIR=/run/user/1001/yazelix/volatile/tmp' "$volatile_env"
        grep -Fx 'KACHE_CACHE_DIR=/home/flexnetos/.cache/kache' "$volatile_env"
        grep -Fx 'RUSTC_WRAPPER=/home/flexnetos/.nix-profile/bin/kache-rustc-wrapper' "$volatile_env"
        grep -F 'legacy Kache root must not exist' ${./nushell/system/volatile_runtime.nu}
        grep -F 'legacy Kache delivery artifact must not exist' ${./nushell/system/volatile_runtime.nu}
        grep -F 'const PROFILE_RUNTIME_ROOT = "/run/user/1001/yazelix/profile-runtime"' ${./nushell/system/volatile_runtime.nu}

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
      # YZXCONV-003: the packaging must emit exactly one foundation element, the
      # profile-contract scripts must satisfy their fixture suite, and a staged
      # selector built from the real foundation closure must pass every clause.
      single_profile_contract = let
        foundation = self.packages.${system}.lifeos_foundation_yzx;
        foundationAttrCount =
          builtins.length
          (builtins.filter (pkgs.lib.hasPrefix "lifeos_foundation")
            (builtins.attrNames self.packages.${system}));
        stagedProfile = pkgs.runCommand "single-profile-staged-profile" {} ''
          mkdir -p "$out"
          ln -s ${foundation}/bin "$out/bin"
          ln -s ${foundation}/toolbin "$out/toolbin"
          cat > "$out/manifest.json" <<EOF
          {"version":3,"elements":{"lifeos_foundation_yzx":{"active":true,"attrPath":"packages.${system}.lifeos_foundation_yzx","originalUrl":"path:.","outputs":null,"priority":5,"storePaths":["${foundation}"],"url":"path:."}}}
          EOF
        '';
      in pkgs.runCommand "single-profile-contract-check" {nativeBuildInputs = [pkgs.nushell];} ''
        # source contract: exactly one foundation package attribute
        test ${toString foundationAttrCount} = 1

        # hermetic fixture suite for the check + migration scripts
        nu ${./packaging/tests/single_profile_contract_test.nu} ${./packaging}

        # staged selector pointing at the real foundation closure
        staging="$TMPDIR/staging"
        mkdir -p "$staging/state/nix" "$staging/home"
        ln -s ${stagedProfile} "$staging/home/.nix-profile-1-link"
        ln -s .nix-profile-1-link "$staging/home/.nix-profile"
        YZX_PROFILE_LINK="$staging/home/.nix-profile" \
          YZX_LEGACY_XDG_PROFILE="$staging/state/nix/profile" \
          YZX_LEGACY_NESTED_PROFILE="$staging/state/nix/profiles/profile" \
          YZX_EXPECTED_CLOSURE="${foundation}" \
          ${foundation}/bin/yazelix_profile_check > staged-check.json
        grep -F '"pass": true' staged-check.json
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
