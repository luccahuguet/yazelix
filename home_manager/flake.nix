{
  description = "Yazelix Home Manager Module";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
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
            desktopExec = hmConfig.config.xdg.desktopEntries.yazelix.exec;
            startupWMClass = hmConfig.config.xdg.desktopEntries.yazelix.settings.StartupWMClass;
            yazelixToml = hmConfig.config.xdg.configFile."yazelix/yazelix.toml".text;
            yazelixPacksToml = hmConfig.config.xdg.configFile."yazelix/yazelix_packs.toml".text;
          in
          {
            desktop_entry_smoke = pkgs.runCommand "yazelix-home-manager-desktop-entry-smoke" {
              passthru.exec = desktopExec;
              passthru.startupWMClass = startupWMClass;
              passthru.yazelixToml = yazelixToml;
              passthru.yazelixPacksToml = yazelixPacksToml;
            } ''
              printf '%s' '${startupWMClass}' > "$out"
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
                
                # Editor configuration (flat structure)
                set_editor = true;
                override_existing = true;
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
