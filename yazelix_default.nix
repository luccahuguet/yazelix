{ pkgs }:
{
  # Recommended tools like lazygit, mise, etc. (default: true)
  recommended_deps = true;

  # Yazi extensions for previews, archives, etc. (default: true)
  yazi_extensions = true;

  # Heavy media packages for Yazi (WARNING: ~800MB-1.2GB) (default: true)
  yazi_media = true;

  # Helix build mode (choose ONE):
  # "release" - Use latest Helix release from nixpkgs (recommended for first-time users)
  # "source"  - Use Helix flake from repository (bleeding edge, recommended for most users)
  helix_mode = "release";

  # Default shell for Zellij: "nu", "bash", "fish", or "zsh". (default: "nu")
  # Note: fish and zsh are only installed if set as default_shell or included in extra_shells
  default_shell = "nu";

  # Extra shells to install beyond nu/bash (e.g., ["fish", "zsh"]) (default: [])
  # Only install additional shells if you plan to use them
  extra_shells = [ ];

  # Preferred terminal emulator for launch_yazelix.nu (default: "wezterm")
  # Options: "wezterm", "ghostty", "kitty", "alacritty"
  # WezTerm is the default because it currently has better image preview support in Yazi but all four are great
  preferred_terminal = "wezterm";

  # Whether to set EDITOR environment variable (default: true)
  set_editor = true;
  # Whether to override existing EDITOR if already set (default: true)
  # Set to false if you want to keep your existing EDITOR
  override_existing = true;
  # Custom editor command (default: "hx" for Helix)
  # You can change this to "vim", "nvim", "kak", etc. if you prefer
  editor_command = "hx";

  # Enable verbose debug logging in the shellHook (default: false)
  debug_mode = false;

  # Skip the welcome screen on startup (default: false)
  # When true, welcome info is logged to the logs directory instead of displayed
  skip_welcome_screen = false;

  # ASCII art display mode (default: "animated")
  # Options: "static" - Show static ASCII art, "animated" - Show animated ASCII art
  ascii_art_mode = "animated";

  # Show macchina system info on the welcome screen if enabled (uses macchina, always available in Yazelix)
  show_macchina_on_welcome = true;

  # User packages - add your custom Nix packages here
  user_packages = with pkgs; [
    # discord
    # vlc
    # inkscape
  ];
}
