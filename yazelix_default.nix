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
  # Note: fish and zsh are only installed if set as default_shell or included in extra_shells
  default_shell = "nu";

  # Extra shells to install beyond nu/bash (e.g., ["fish", "zsh"]) (default: [])
  # Only install additional shells if you plan to use them
  extra_shells = [ ];

  # Enable verbose debug logging in the shellHook (default: false)
  debug_mode = false;

  # Skip the welcome screen on startup (default: false)
  # When true, welcome info is logged to the logs directory instead of displayed
  skip_welcome_screen = false;

  # User packages - add your custom Nix packages here
  user_packages = with pkgs; [
    # discord
    # vlc
    # inkscape
  ];
}
