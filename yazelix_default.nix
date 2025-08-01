{ pkgs }:
{
  # Dependency groups - See docs/package_sizes.md for details
  recommended_deps = true; # Productivity tools (~350MB)
  yazi_extensions = true; # File preview support (~125MB)
  yazi_media = false; # Media processing (~1GB)

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

  # Preferred terminal emulator for launch_yazelix.nu (default: "ghostty")
  # Options: "wezterm", "ghostty", "kitty", "alacritty"
  # Ghostty is the default for great performance; use WezTerm if you need better image preview support in Yazi
  preferred_terminal = "ghostty";

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

  # Zellij persistent session configuration
  # Enable persistent sessions (default: false)
  # When true, Yazelix will use zellij attach with the specified session name
  # When false, Yazelix will create a new session each time
  persistent_sessions = false;
  # Session name for persistent sessions (default: "yazelix")
  # This name will be used when creating or attaching to persistent sessions
  session_name = "yazelix";

  # User packages - add your custom Nix packages here
  user_packages = with pkgs; [
    # Package Management Pack
    # cargo-update # Updates Rust crates for project maintenance
    # cargo-binstall # Faster installation of Rust tools
    # mise # Tool version manager for consistent environments

    # JavaScript/TypeScript Pack
    # biome # formats JS, TS, JSON, CSS, and lints js/ts

    # Python Pack
    # ruff # Fast Python linter and code formatter
    # uv # Ultra-fast Python package installer and resolver
    # ty # Extremely fast Python type checker from Astral

    # File Management Pack
    # ouch # Compression tool for handling archives
    # erdtree # Modern tree command with file size display
    # serpl # Command-line tool for search and replace operations
  ];
}
