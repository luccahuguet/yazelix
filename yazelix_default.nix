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

  # Use patchy to build Helix with community PRs (default: false)
  # Note: This requires build_helix_from_source = true or will enable it automatically
  use_patchy_helix = false;

  # Patchy Helix configuration
  patchy_helix_config = {
    # Popular community PRs (curated for stability)
    pull_requests = [
      "12309" # syntax highlighting for nginx files
      "8908" # global status line
      "13197" # welcome screen
      "11700" # add per view search location and total matches to statusline
      "11497" # rounded-corners option to draw rounded borders
      "13133" # inline git blame
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
