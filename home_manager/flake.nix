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
                ascii_art_mode = "animated";  # Opt-in for animated welcome (default: "static")
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
