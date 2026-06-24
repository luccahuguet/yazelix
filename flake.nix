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
  };

  outputs = {
    self,
    nixpkgs,
    mars,
    yazelixZellij,
  }: let
    systems = [
      "x86_64-linux"
      "aarch64-linux"
    ];
    eachSystem = nixpkgs.lib.genAttrs systems;
    mkYazelixZellij = pkgs: let
      baseZellij =
        if pkgs.zellij ? unwrapped
        then pkgs.zellij.unwrapped
        else if builtins.hasAttr "zellij-unwrapped" pkgs
        then pkgs."zellij-unwrapped"
        else pkgs.zellij;
    in
      baseZellij.overrideAttrs (_old: {
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
  in {
    packages = eachSystem (system: let
      pkgs = import nixpkgs {inherit system;};
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
      yznNuShell = pkgs.writeShellApplication {
        name = "yzn-nu";
        runtimeInputs = [pkgs.nushell pkgs.starship pkgs.carapace pkgs.zoxide];
        text = ''
          exec nu --env-config ${yznNuConfig}/env.nu --config ${yznNuConfig}/config.nu "$@"
        '';
      };
      yznConfigKdl = pkgs.replaceVars ./config.kdl {
        nuShell = "${yznNuShell}/bin/yzn-nu";
      };
      yznZellijConfig = pkgs.runCommand "yzn-zellij-config" {} ''
        install -D -m 644 ${yznConfigKdl} "$out/config.kdl"
      '';
      yznZellijLayout = pkgs.runCommand "yzn-zellij-layout" {} ''
        install -D -m 644 ${./layout.kdl} "$out/layout.kdl"
      '';
      yazelixZellijPackage = mkYazelixZellij pkgs;
      yznCommand = pkgs.writeShellApplication {
        name = "yzn";
        runtimeInputs = [pkgs.nushell pkgs.starship pkgs.carapace pkgs.zoxide];
        text = ''
          export MARS_CONFIG_HOME=${yznMarsConfig}
          exec ${marsPackage}/bin/mars -e ${yazelixZellijPackage}/bin/zellij --config ${yznZellijConfig}/config.kdl --new-session-with-layout ${yznZellijLayout}/layout.kdl "$@"
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
          install -D -m 644 ${yznZellijConfig}/config.kdl "$out/share/yazelix-next/config.kdl"
          install -D -m 644 ${yznZellijLayout}/layout.kdl "$out/share/yazelix-next/layout.kdl"
          install -D -m 644 ${yznNuConfig}/config.nu "$out/share/yazelix-next/nu/config.nu"
          install -D -m 644 ${yznNuConfig}/env.nu "$out/share/yazelix-next/nu/env.nu"
          install -D -m 644 ${yznCarapaceInit} "$out/share/yazelix-next/nu/carapace.nu"
          install -D -m 644 ${yznZoxideInit} "$out/share/yazelix-next/nu/zoxide.nu"
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
      yazelix_zellij = yazelixZellijPackage;
      inherit yzn;
      default = yzn;
    });

    apps = eachSystem (system: let
      yzn = {
        type = "app";
        program = "${self.packages.${system}.yzn}/bin/yzn";
      };
    in {
      inherit yzn;
      default = yzn;
    });
  };
}
