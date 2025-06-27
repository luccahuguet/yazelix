{ pkgs }:
{
  # Include optional tools like lazygit, mise, etc. (default: true)
  include_optional_deps = true;

  # Include Yazi extensions for previews, archives, etc. (default: true)
  include_yazi_extensions = true;

  # Include heavy media packages for Yazi (WARNING: ~800MB-1.2GB) (default: true)
  include_yazi_media = true;

  # Build Helix from source (true) or use nixpkgs version (false). (default: false)
  build_helix_from_source = false;

  # Default shell for Zellij: "nu", "bash", "fish", or "zsh". (default: "nu")
  default_shell = "nu";

  # Enable verbose debug logging in the shellHook (default: false)
  debug_mode = false;

  # User packages - add your custom Nix packages here
  user_packages = with pkgs; [
    # discord
    # vlc
    # inkscape
  ];
}
