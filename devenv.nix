# devenv.nix - Production configuration for Yazelix
# This provides 13x faster shell startup through evaluation caching
{ pkgs, lib, config, inputs, ... }:

let
  # Import nixgl for GPU acceleration on non-NixOS systems
  nixgl = inputs.nixgl.packages.${pkgs.system};

  # Read yazelix.nix configuration (relative to this file)
  configFile = ./yazelix.nix;
  defaultConfigFile = ./yazelix_default.nix;
  
  userConfig = if builtins.pathExists configFile
    then import configFile { inherit pkgs; }
    else if builtins.pathExists defaultConfigFile
    then import defaultConfigFile { inherit pkgs; }
    else {};
  
  # Configuration with defaults
  recommendedDepsEnabled = userConfig.recommended_deps or true;
  yaziExtensionsEnabled = userConfig.yazi_extensions or true;
  yaziMediaEnabled = userConfig.yazi_media or true;
  defaultShell = userConfig.default_shell or "nu";
  extraShells = userConfig.extra_shells or [];
  helixMode = userConfig.helix_mode or "release";
  preferredTerminal = userConfig.preferred_terminal or "ghostty";
  extraTerminals = userConfig.extra_terminals or [];

  # Determine which shells to include
  shellsToInclude = ["nu" "bash" defaultShell] ++ extraShells;
  includeFish = builtins.elem "fish" shellsToInclude;
  includeZsh = builtins.elem "zsh" shellsToInclude;

  # Determine which terminals to include
  # Ghostty is always included on Linux (like flake.nix)
  # Other terminals are included based on preferred_terminal and extra_terminals
  includeKitty = (preferredTerminal == "kitty") || (builtins.elem "kitty" extraTerminals);
  includeWezterm = (preferredTerminal == "wezterm") || (builtins.elem "wezterm" extraTerminals);
  includeAlacritty = (preferredTerminal == "alacritty") || (builtins.elem "alacritty" extraTerminals);
  includeFoot = (preferredTerminal == "foot") || (builtins.elem "foot" extraTerminals);
  
in {
  # Essential dependencies (always included)
  packages = with pkgs; [
    # Core Yazelix tools
    zellij          # Terminal multiplexer
    helix           # Text editor
    yazi            # File manager
    nushell         # Modern shell
    fzf             # Fuzzy finder
    zoxide          # Smart directory jumper
    starship        # Shell prompt
    bashInteractive # Interactive Bash
    macchina        # System info
    mise            # Tool version manager
  ]
  # Recommended dependencies
  ++ (if recommendedDepsEnabled then [
    ripgrep       # Fast grep
    fd            # Fast find
    bat           # Cat with syntax highlighting
    jq            # JSON processor
    git           # Version control
    curl          # HTTP client
    wget          # File downloader
    unzip         # Zip extractor
    gnused        # Stream editor
    findutils     # Find utilities
    coreutils     # Core utilities
    gnutar        # Tar archiver
    gzip          # Gzip compression
    lazygit       # Terminal-based Git TUI
    atuin         # Shell history manager
    carapace      # Command-line completion
    markdown-oxide # PKMS for text editors
  ] else [])
  # Yazi media support
  ++ (if yaziMediaEnabled then [
    ffmpeg        # Video/audio processing
    imagemagick   # Image processing
    p7zip         # 7z compression
    unar          # Universal archiver
    poppler_utils # PDF utilities
  ] else [])
  # Extra shells
  ++ (if includeFish then [fish] else [])
  ++ (if includeZsh then [zsh] else [])
  # GPU acceleration for terminals on non-NixOS systems
  ++ [nixgl.nixGLIntel]
  # Terminal emulators (Ghostty always on Linux, others conditional)
  ++ [ghostty]  # Always include Ghostty on Linux (default terminal)
  ++ (if includeKitty then [kitty] else [])
  ++ (if includeWezterm then [wezterm] else [])
  ++ (if includeAlacritty then [alacritty] else [])
  ++ (if includeFoot then [foot] else []);

  # Environment variables
  env.YAZELIX_DIR = "$HOME/.config/yazelix";
  env.IN_YAZELIX_SHELL = "true";
  env.NIX_CONFIG = "warn-dirty = false";

  # Shell hook - runs environment setup
  enterShell = ''
    # Ensure HOME is set (devenv may not pass it through in pure mode)
    # DEVENV_ROOT is ~/.config/yazelix, so we can derive HOME from it
    if [ -z "$HOME" ]; then
      # Extract home directory from DEVENV_ROOT (which is ~/.config/yazelix)
      export HOME="$(dirname "$(dirname "$DEVENV_ROOT")")"
    fi

    # Set EDITOR
    export EDITOR="hx"
    if [ "$YAZELIX_ENV_ONLY" != "true" ]; then
      echo "📝 Set EDITOR to: hx"
    fi

    # Auto-copy config file if it doesn't exist
    if [ ! -f "$YAZELIX_DIR/yazelix.nix" ] && [ -f "$YAZELIX_DIR/yazelix_default.nix" ]; then
      cp "$YAZELIX_DIR/yazelix_default.nix" "$YAZELIX_DIR/yazelix.nix"
      echo "Created yazelix.nix from template. Customize it for your needs!"
    fi

    # Run main environment setup script
    nu ~/.config/yazelix/nushell/scripts/setup/environment.nu \
      ~/.config/yazelix \
      "${if recommendedDepsEnabled then "true" else "false"}" \
      "false" \
      "false" \
      "${defaultShell}" \
      "false" \
      "${if extraShells == [] then "NONE" else builtins.concatStringsSep "," extraShells}" \
      "true" \
      "${helixMode}" \
      "static" \
      "false"
  '';

}
