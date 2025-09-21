{ pkgs }:
{
  # Dependency groups - See docs/package_sizes.md for details
  recommended_deps = true; # Productivity tools (~350MB)
  # Atuin shell history integration (separate control; disabled by default)
  enable_atuin = false;
  yazi_extensions = true; # File preview support (~125MB)
  yazi_media = false; # Media processing: ffmpeg + imagemagick (~1GB)

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
  # Ghostty is always included by default with nixGL acceleration
  preferred_terminal = "ghostty";

  # Extra terminal emulators to install beyond Ghostty (default: [])
  # Options: ["wezterm", "kitty", "alacritty"]
  # Only install additional terminals if you plan to use them
  extra_terminals = [ ];

  # Terminal config mode (how Yazelix handles terminal emulator configs)
  # Options:
  # - "auto"   : Prefer the user's config if present; otherwise use Yazelix's config
  # - "user"   : Always use the user's config paths
  # - "yazelix": Always use Yazelix-provided configs under ~/.config/yazelix/configs/terminal_emulators
  terminal_config_mode = "yazelix";

  # Cursor trail preset
  # Supported by all terminal emulators: "none"
  # Supported by Ghostty: "blaze", "snow", "cosmic", "ocean", "forest", "sunset", "neon", "party"
  # Supported by Ghostty and Kitty: "snow"
  # In Short: only ghostty supports all the cool cursor trails
  cursor_trail = "blaze";

  # Terminal transparency level (default: "low")
  # Options: "none", "low", "medium", "high"
  # - "none": No transparency (opacity = 1.0)
  # - "low": Light transparency (opacity = 0.95)
  # - "medium": Medium transparency (opacity = 0.9)
  # - "high": High transparency (opacity = 0.8)
  transparency = "low";


  # ==================== Editor Configuration ====================
  # Yazelix always sets this as your EDITOR environment variable

  # editor_command options:
  # • null (recommended): Use yazelix's Nix-provided Helix
  #   - Eliminates runtime conflicts with existing Helix installations
  #   - Binary and runtime are perfectly matched
  #   - Full yazelix integration features (reveal in sidebar, same-instance opening, etc.)
  #
  # • "hx": Use your system Helix from PATH
  #   - Requires setting helix_runtime_path to match your Helix version
  #   - Full yazelix integration if runtime matches
  #   - Use this if you have a custom Helix build you prefer
  #
  # • Other editors: "vim", "nvim", "nano", "emacs", etc.
  #   - Basic integration only (new panes, tab naming)
  #   - Loses advanced features (reveal in sidebar, same-instance opening)
  #   - Works reliably but with limited yazelix-specific functionality
  editor_command = null;

  # Helix runtime path (advanced users only)
  # ONLY set this if editor_command points to a custom Helix build
  # The runtime MUST exactly match your Helix binary version to avoid errors
  #
  # Common scenarios:
  # • Development build: "/home/user/helix/runtime"
  # • Custom install: "/opt/helix/share/helix/runtime"
  # • System package: "/usr/share/helix/runtime"
  #
  # To find your runtime: ls $(dirname $(which hx))/../share/helix/runtime
  # If this path doesn't exist, your Helix may be incompatible
  helix_runtime_path = null;

  # Enable or disable the Yazi sidebar (default: true)
  # When false, Yazelix uses clean, full-screen layouts with on-demand file picking
  # When true, Yazelix uses persistent sidebar layouts for IDE-like workflow
  # You can access Yazi manually with `yazi` command or `Ctrl+y` in Helix
  enable_sidebar = true;


  # Enable verbose debug logging in the shellHook (default: false)
  debug_mode = false;

  # Skip the welcome screen on startup (default: false)
  # When true, welcome info is logged to the logs directory instead of displayed
  skip_welcome_screen = false;

  # ASCII art display mode (default: "static")
  # Options: "static" - Show static ASCII art, "animated" - Show animated ASCII art (opt-in for faster startup)
  ascii_art_mode = "static";

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
