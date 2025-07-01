{ pkgs }:
{
  # Include optional tools like lazygit, mise, etc. (default: true)
  include_optional_deps = true;

  # Include Yazi extensions for previews, archives, etc. (default: true)
  include_yazi_extensions = true;

  # Include heavy media packages for Yazi (WARNING: ~800MB-1.2GB) (default: true)
  include_yazi_media = true;

  # Helix build mode (choose ONE):
  # "release" - Use latest Helix release from nixpkgs (fast, recommended for first-time users)
  # "source"  - Use Helix flake from repository (bleeding edge, recommended for most users)
  # "patchy"  - Build Helix with community PRs (experimental)
  # "steel"   - Build Helix with steel plugin system (experimental)
  #              Includes auto-generated example plugin with basic commands for testing
  helix_mode = "release";

  # Patchy Helix configuration (only used if helix_mode = "patchy")
  patchy_helix_config = {
    # Popular community PRs (curated for stability)
    pull_requests = [
      # "13197" # welcome screen: no conflicts on its own
      # "13133" # inline git blame: has merge conflicts with main
      # "11497" # rounded-corners option to draw rounded borders: has merge conflicts with main
      # "8908" # global status line: unknown
      # "11700" # add per view search location and total matches to statusline: unknown
    ];

    # Custom patches (empty by default)
    patches = [ ];

    # Pin commits for stability (recommended: true)
    pin_commits = true;

    # Custom repository/branch (optional, defaults to helix-editor/helix@master)
    # repo = "helix-editor/helix";
    # remote_branch = "master";
  };

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
