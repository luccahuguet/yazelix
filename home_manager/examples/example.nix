# Yazelix Home Manager Configuration Example
# Shows all available options with sensible defaults
#
# For complete option reference, see:
#   - home_manager/module.nix (complete option definitions)
#   - yazelix_default.toml (TOML structure with detailed comments)

{ config, pkgs, ... }:

{
  # REQUIRED: Add nushell to your packages for terminal emulator compatibility
  home.packages = with pkgs; [
    nushell # Required for Yazelix terminal startup
    # Add your other packages here
  ];

  programs.yazelix = {
    enable = true;

    # Dependency control for specific use cases
    recommended_deps = true; # Productivity tools like lazygit, atuin
    yazi_extensions = true; # File preview support
    yazi_media = true; # Enable heavy media processing (~1GB)
    max_jobs = "half"; # Optional: "auto", "max", "max_minus_one", "half", "quarter", or "8"
    build_cores = "2"; # Optional: "max", "max_minus_one", "half", "quarter", or "2"

    # Build Helix from source for latest features
    helix_mode = "source";

    # Multi-shell environment
    default_shell = "nu";
    extra_shells = [
      "fish"
      "zsh"
    ]; # Optional: install additional shells

    # Terminal preference
    terminals = [
      "wezterm"
      "ghostty"
    ]; # Better for media previews
    manage_terminals = true;
    terminal_config_mode = "yazelix"; # Optional: "auto", "user", or "yazelix"
    ghostty_trail_color = "random"; # Optional: Ghostty color palette + Kitty fallback
    ghostty_trail_effect = "random"; # Optional: "tail", "warp", "sweep", "random", or null
    ghostty_mode_effect = "random"; # Optional: "ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle", "random", or null
    ghostty_trail_glow = "medium"; # Optional: "none", "low", "medium", or "high"
    transparency = "medium"; # Optional: "none".."super_high"

    # Editor configuration
    # editor_command = null;       # Optional: Use Yazelix's Helix (recommended)
    editor_command = "hx"; # Optional: Use system Helix from PATH
    # editor_command = "nvim";     # Optional: Use other editor (loses Helix features)
    # helix_runtime_path = "/home/user/helix/runtime";  # Optional: only for custom/nonstandard Helix runtimes

    # Development-friendly settings
    debug_mode = true; # Enable verbose logging
    skip_welcome_screen = false; # Show welcome screen
    ascii_art_mode = "static"; # Static ASCII art for faster startup
    show_macchina_on_welcome = true;

    # Zellij customization
    enable_sidebar = true;
    disable_zellij_tips = true;
    zellij_rounded_corners = true;
    zellij_theme = "default"; # Optional: any built-in theme name

    # Yazi customization
    yazi_plugins = [
      "git"
      "starship"
    ];
    yazi_theme = "default"; # Optional: flavor name or "random-dark"
    yazi_sort_by = "alphabetical";

    # Persistent sessions for long-running work
    persistent_sessions = true;
    session_name = "main-dev";

    # Packs (optional bundles defined in pack_declarations)
    pack_names = [
      "python"
      "git"
    ];
    pack_declarations = {
      python = [
        "ruff"
        "uv"
        "ty"
        "python3Packages.ipython"
      ];
      git = [
        "onefetch"
        "gh"
        "delta"
        "gitleaks"
        "jujutsu"
        "prek"
      ];
    };
    enable_atuin = true;

    # Additional tools for development workflow
    user_packages = with pkgs; [
      # Package management
      cargo-update
      mise

      # Development tools
      ruff # Python linting/formatting
      biome # JS/TS formatting and linting

      # File management
      ouch # Archive handling
      erdtree # Modern tree command
    ];
  };
}
