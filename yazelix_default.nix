{ pkgs }:
{
  # Dependency groups - See docs/package_sizes.md for details
  recommended_deps = true; # Productivity tools (~350MB)
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

  # Atuin shell history integration (separate control; disabled by default)
  enable_atuin = false;

  # Preferred terminal emulator for launch_yazelix.nu (default: "ghostty")
  # Options: "ghostty", "wezterm", "kitty", "alacritty", "foot" (Linux-only)
  #
  # Ghostty installation:
  # - Linux: Provided by Yazelix via Nix
  # - macOS: Install via Homebrew: `brew install --cask ghostty`
  #   (Nix package doesn't support macOS due to app bundle limitations)
  #
  # Auto-detection fallback order: ghostty → wezterm → kitty → alacritty → foot
  # WezTerm is the recommended fallback (works on both platforms, best image preview)
  preferred_terminal = "ghostty";

  # Extra terminal emulators to install beyond Ghostty (default: [])
  # Options: ["wezterm", "kitty", "alacritty", "foot" (Linux-only)]
  # Only install additional terminals if you plan to use them
  extra_terminals = [ ];

  # Terminal config mode (how Yazelix handles terminal emulator configs)
  # Options:
  # - "auto"   : Prefer the user's config if present; otherwise use Yazelix's config
  # - "user"   : Always use the user's config paths
  # - "yazelix": Always use Yazelix-provided configs under ~/.config/yazelix/configs/terminal_emulators
  terminal_config_mode = "yazelix";

  # Cursor trail preset
  # None: "none"
  # Mono-color: "blaze", "snow", "cosmic", "ocean", "forest", "sunset"
  # Duo-color: "neon", "eclipse", "dusk", "orchid", "reef"
  # Extreme: "party"
  # Special: "random" (chooses any Ghostty preset except "none" and "party")
  # Supported by Ghostty and Kitty: "snow"
  # In short: pick ghostty for cool cursor trails
  cursor_trail = "random";

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

  # Disable Zellij tips popup on startup (default: false)
  # Set to true to suppress the tips dialog for cleaner launches
  disable_zellij_tips = false;


  # Enable verbose debug logging in the shellHook (default: false)
  debug_mode = false;

  # Skip the welcome screen on startup (default: true)
  # When true, welcome info is logged to the logs directory instead of displayed
  skip_welcome_screen = true;

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

  # Language packs - complete toolchains for programming languages
  language_packs = [
    # "python"         # ruff (formatter/linter), uv (package manager), ty (type checker), ipython (enhanced REPL)
    # "ts"             # typescript-language-server (LSP), biome (formatter/linter), oxlint (fast linter), bun (runtime/bundler)
    # "rust"           # cargo-update, cargo-binstall, cargo-edit (add/rm), cargo-watch, cargo-audit, cargo-nextest
    # "go"             # gopls (language server), golangci-lint (linter), delve (debugger), air (hot reload), govulncheck (vulnerability scanner)
    # "kotlin"         # kotlin-language-server (LSP), ktlint (linter/formatter), detekt (static analysis), gradle (build tool)
    # "gleam"          # gleam (compiler with built-in LSP, formatter, and build tool)
    # "nix"            # nil (language server), nixd (advanced language server), nixfmt-rfc-style (formatter)
  ];

  # Tool packs - general-purpose development tools
  tool_packs = [
    # "config"         # taplo (TOML formatter/LSP), mpls (Markdown preview LSP)
    # "file-management" # ouch (archives), erdtree (tree view), serpl (search/replace)
    # "git"            # onefetch (repo summary), gh (GitHub CLI), delta (diff viewer), gitleaks (secret scanner), jj (Jujutsu VCS), prek (commit log viewer)
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
