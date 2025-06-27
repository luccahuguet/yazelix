{ pkgs }:
{
  # Include optional tools like lazygit, mise, etc. (default: true)
  include_optional_deps = true;

  # Include Yazi extensions for previews, archives, etc. (default: true)
  include_yazi_extensions = true;

  # Build Helix from source (true) or use nixpkgs version (false). (default: true)
  build_helix_from_source = true;

  # Default shell for Zellij: "nu" or "bash". (default: "nu")
  default_shell = "nu";

  # Enable verbose debug logging in the shellHook (default: false)
  debug_mode = false;

  # User packages - add your custom Nix packages here
  user_packages = with pkgs; [
    # discord
    # vlc
    inkscape
  ];
}
