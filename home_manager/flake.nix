{
  description = "Yazelix Home Manager Module";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    yazelix-src = {
      url = "path:../.";
      flake = false;
    };
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      yazelix-src,
      home-manager,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      # Home Manager module
      homeManagerModules.default = import ./module.nix;
      homeManagerModules.yazelix = import ./module.nix;

      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          hmConfig =
            if pkgs.stdenv.isLinux then
              home-manager.lib.homeManagerConfiguration {
                inherit pkgs;
                modules = [
                  self.homeManagerModules.default
                  {
                    _module.args.yazelixSrc = yazelix-src;
                  }
                  {
                    home.username = "test";
                    home.homeDirectory = "/home/test";
                    home.stateVersion = "24.11";
                    programs.yazelix.enable = true;
                    programs.yazelix.pack_names = [ "git" ];
                  }
                ];
              }
            else
              null;
        in
        if hmConfig == null then
          { }
        else
          let
            runtimeSource = hmConfig.config.xdg.dataFile."yazelix/runtime/current".source;
            yzxShim = hmConfig.config.home.file.".local/bin/yzx".text;
            desktopExec = hmConfig.config.xdg.desktopEntries.yazelix.exec;
            startupWMClass = hmConfig.config.xdg.desktopEntries.yazelix.settings.StartupWMClass;
            yazelixToml = hmConfig.config.xdg.configFile."yazelix/user_configs/yazelix.toml".text;
            yazelixPacksToml = hmConfig.config.xdg.configFile."yazelix/user_configs/yazelix_packs.toml".text;
            expectedRuntimePath = "/home/test/.local/share/yazelix/runtime/current";
            expectedYzxPath = "/home/test/.local/bin/yzx";
          in
          {
            desktop_entry_smoke = pkgs.runCommand "yazelix-home-manager-desktop-entry-smoke" {
              passthru.runtimeSource = runtimeSource;
              passthru.yzxShim = yzxShim;
              passthru.exec = desktopExec;
              passthru.startupWMClass = startupWMClass;
              passthru.yazelixToml = yazelixToml;
              passthru.yazelixPacksToml = yazelixPacksToml;
            } ''
              expected_runtime_path='${expectedRuntimePath}'
              expected_yzx_path='${expectedYzxPath}'
              desktop_exec='${desktopExec}'
              runtime_source='${runtimeSource}'
              yzx_shim='${yzxShim}'

              if [ "$desktop_exec" != "$expected_yzx_path desktop launch" ]; then
                echo "unexpected desktop exec: $desktop_exec" >&2
                exit 1
              fi

              if ! printf '%s\n' "$yzx_shim" | grep -F "exec \"$expected_runtime_path/bin/yzx\" \"\$@\"" >/dev/null; then
                echo "yzx shim does not target runtime/current" >&2
                exit 1
              fi

              cat > "$out" <<EOF
              StartupWMClass=${startupWMClass}
              DesktopExec=$desktop_exec
              RuntimeSource=$runtime_source
              EOF
            '';
          }
      );

      # Example configurations for testing
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          # Example Home Manager configuration
          example-config = pkgs.writeTextFile {
            name = "yazelix-example-config";
            text = ''
              # Basic Yazelix Home Manager configuration example
              programs.yazelix = {
                enable = true;
                
                # Dependency control
                recommended_deps = true;
                yazi_extensions = true;
                yazi_media = false;  # Disable heavy media tools
                
                # Shell configuration
                default_shell = "nu";
                extra_shells = [ "fish" ];
                
                # Terminal preference
                terminals = [ "ghostty" ];
                manage_terminals = true;
                
                # Editor configuration
                editor_command = "hx";
                
                # Display options
                welcome_style = "random";  # Random animated welcome style (never static)
                show_macchina_on_welcome = true;
                
                # Session configuration
                persistent_sessions = false;
                session_name = "yazelix";
              };
            '';
          };
        }
      );

      # Development shells for testing
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              nixpkgs-fmt
              statix
              deadnix
            ];

            shellHook = ''
              echo "Yazelix Home Manager Module Development Shell"
              echo "Available commands:"
              echo "  nixpkgs-fmt *.nix  # Format Nix files"
              echo "  statix check .     # Lint Nix files"
              echo "  deadnix .          # Find dead code"
            '';
          };
        }
      );
    };
}
