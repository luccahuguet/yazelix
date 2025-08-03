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

  # Enable or disable the Yazi sidebar (default: false)
  # When false, Yazelix uses clean, full-screen layouts with on-demand file picking
  # When true, Yazelix uses persistent sidebar layouts for IDE-like workflow
  # You can access Yazi manually with `yazi` command or `Ctrl+y` in Helix
  enable_sidebar = false;

  # Smart directory start using zoxide database (default: true)
  # When true, editor opens in the most frequently accessed directory from zoxide
  # When false, editor opens in the current working directory (~/.config/yazelix)
  # Helps avoid always starting in the config directory and instead opens your most-used project
  smart_directory_start = true;

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

  # Package packs - enable entire technology stacks
  packs = [
    # "python"         # ruff (formatter/linter), uv (package manager), ty (type checker)
    # "js_ts"          # biome (formatter/linter), bun (runtime/bundler)  
    # "rust"           # cargo-update (crate updater), cargo-binstall (binary installer)
    # "config"         # taplo (TOML), nixfmt-rfc-style (Nix), mpls (Markdown preview)
    # "file-management" # ouch (archives), erdtree (tree view), serpl (search/replace)
  ];

  # User packages - add individual packages here
  # Tip: if you don't want an entire pack from above, place the individual deps below. (example: taplo)
  user_packages = with pkgs; [
    # Add custom packages here
    # gh # GitHub CLI for repository management
    # docker # Container platform for development  
    # kubectl # Kubernetes command-line tool
  ];
}
