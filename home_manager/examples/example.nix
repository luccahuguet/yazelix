# Yazelix Home Manager Configuration Example
# Shows all available options with sensible defaults

{ config, pkgs, ... }:

{
  # REQUIRED: Add nushell to your packages for terminal emulator compatibility
  home.packages = with pkgs; [
    nushell  # Required for Yazelix terminal startup
    # Add your other packages here
  ];

  programs.yazelix = {
    enable = true;
    
    # Dependency control for specific use cases
    recommended_deps = true;      # Productivity tools like lazygit, atuin
    yazi_extensions = true;       # File preview support
    yazi_media = true;           # Enable heavy media processing (~1GB)
    
    # Build Helix from source for latest features
    helix_mode = "source";
    
    # Multi-shell environment
    default_shell = "nu";
    extra_shells = [ "fish" "zsh" ];  # Install additional shells
    
    # Terminal preference
    preferred_terminal = "wezterm";  # Better for media previews
    
    # Custom editor configuration
    editor_config = {
      set_editor = true;
      override_existing = false;     # Keep existing EDITOR if set
      editor_command = "hx";         # or "vim", "nvim", etc.
    };
    
    # Development-friendly settings
    debug_mode = true;             # Enable verbose logging
    skip_welcome_screen = false;   # Show welcome screen
    ascii_art_mode = "static";     # Static ASCII art for faster startup
    show_macchina_on_welcome = true;
    
    # Persistent sessions for long-running work
    persistent_sessions = true;
    session_name = "main-dev";
    
    # Additional tools for development workflow
    user_packages = with pkgs; [
      # Package management
      cargo-update
      mise
      
      # Development tools
      ruff          # Python linting/formatting
      biome         # JS/TS formatting and linting
      
      # File management
      ouch          # Archive handling
      erdtree       # Modern tree command
    ];
  };
}