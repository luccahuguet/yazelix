{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.programs.yazelix;

  boolToToml = value: if value then "true" else "false";

  escapeString =
    value:
    let
      safe = lib.replaceStrings [ "\"" "\\" ] [ "\\\"" "\\\\" ] value;
    in
    "\"${safe}\"";

  listToToml =
    values:
    if values == [ ] then "[]" else "[ " + (concatStringsSep ", " (map escapeString values)) + " ]";

  packagesToToml =
    packages:
    let
      names = map (pkg: pkg.pname or pkg.name or "unknown") packages;
    in
    listToToml names;

  packDeclarationsToToml =
    declarations:
    let
      names = sort lessThan (attrNames declarations);
    in
    map (name: "${escapeString name} = ${listToToml declarations.${name}}") names;

in
{
  options.programs.yazelix = {
    enable = mkEnableOption "Yazelix terminal environment";

    # Configuration options (mirrors yazelix_default.toml structure)
    recommended_deps = mkOption {
      type = types.bool;
      default = true;
      description = "Install recommended productivity tools (~350MB)";
    };

    yazi_extensions = mkOption {
      type = types.bool;
      default = true;
      description = "Install Yazi file preview extensions (~125MB)";
    };

    yazi_media = mkOption {
      type = types.bool;
      default = false;
      description = "Install Yazi media processing tools (~1GB)";
    };

    build_cores = mkOption {
      type = types.str;
      default = "2";
      description = ''
        CPU cores per Nix build: "max", "max_minus_one", "half", "quarter", or a custom number like "2"
      '';
    };

    max_jobs = mkOption {
      type = types.str;
      default = "half";
      description = ''
        Concurrent Nix build jobs: "auto", "max", "max_minus_one", "half", "quarter", or a custom number like "8"
      '';
    };

    refresh_output = mkOption {
      type = types.enum [
        "quiet"
        "normal"
        "full"
      ];
      default = "normal";
      description = ''
        Refresh output level for launch/restart/refresh flows.
        - "quiet": suppress routine rebuild output
        - "normal": show standard devenv build output
        - "full": show full verbose devenv logs
      '';
    };

    helix_mode = mkOption {
      type = types.enum [
        "release"
        "source"
      ];
      default = "release";
      description = "Helix build mode: release (nixpkgs) or source (flake)";
    };

    default_shell = mkOption {
      type = types.enum [
        "nu"
        "bash"
        "fish"
        "zsh"
      ];
      default = "nu";
      description = "Default shell for Zellij sessions";
    };

    extra_shells = mkOption {
      type = types.listOf (
        types.enum [
          "fish"
          "zsh"
        ]
      );
      default = [ ];
      description = "Additional shells to install beyond nu/bash";
    };

    terminals = mkOption {
      type = types.listOf (
        types.enum [
          "wezterm"
          "ghostty"
          "kitty"
          "alacritty"
          "foot"
        ]
      );
      default = [ "ghostty" ];
      description = "Ordered terminal emulator list (first is primary, rest are fallbacks)";
    };

    manage_terminals = mkOption {
      type = types.bool;
      default = true;
      description = "Manage terminal emulators via Nix (disable to use system-installed terminals only)";
    };

    terminal_config_mode = mkOption {
      type = types.enum [
        "auto"
        "user"
        "yazelix"
      ];
      default = "yazelix";
      description = ''
        How Yazelix selects terminal configs:
        - "yazelix": use Yazelix-managed configs in ~/.local/share/yazelix (default)
        - "auto": prefer user configs if present, otherwise Yazelix configs
        - "user": always use user configs (e.g., ~/.config/ghostty/config)
      '';
    };

    ghostty_trail_color = mkOption {
      type = types.enum [
        "none"
        "blaze"
        "snow"
        "cosmic"
        "ocean"
        "forest"
        "sunset"
        "neon"
        "party"
        "eclipse"
        "dusk"
        "orchid"
        "reef"
        "inferno"
        "random"
      ];
      default = "random";
      description = ''
        Ghostty cursor color palette and Kitty cursor-trail fallback preset.
        Disable the palette and fallback trail: "none"
        Supported by Ghostty: "none", "blaze", "snow", "cosmic", "ocean", "forest", "sunset", "neon", "party", "eclipse", "dusk", "orchid", "reef", "inferno", "random"
        Supported by Ghostty and Kitty: "snow"
        "random" chooses a different Ghostty color palette each generation (excluding "party")
      '';
    };

    ghostty_trail_effect = mkOption {
      type = types.nullOr (types.enum [
        "tail"
        "warp"
        "sweep"
        "random"
      ]);
      default = "random";
      description = ''
        Ghostty trail effect for cursor movement.
        Set to null to disable extra tail effects.
        Valid values: "tail", "warp", "sweep", "random"
      '';
    };

    ghostty_mode_effect = mkOption {
      type = types.nullOr (types.enum [
        "ripple"
        "sonic_boom"
        "rectangle_boom"
        "ripple_rectangle"
        "random"
      ]);
      default = "random";
      description = ''
        Ghostty mode-change effect, triggered when the editor changes cursor mode
        such as Neovim switching between normal and insert.
        Set to null to disable mode-change effects.
        Valid values: "ripple", "sonic_boom", "rectangle_boom", "ripple_rectangle", "random"
      '';
    };

    ghostty_trail_glow = mkOption {
      type = types.enum [
        "none"
        "low"
        "medium"
        "high"
      ];
      default = "medium";
      description = ''
        Glow level around Ghostty cursor trails and related cursor effects.

        - "none": keep the cursor/trail color effect but remove the extra spatial glow
        - "low": a tighter, subtler aura
        - "medium": the current Yazelix look (default)
        - "high": a larger, brighter aura
      '';
    };

    transparency = mkOption {
      type = types.enum [
        "none"
        "very_low"
        "low"
        "medium"
        "high"
        "very_high"
        "super_high"
      ];
      default = "medium";
      description = ''
        Terminal transparency level for all terminals.

        - "none": No transparency (opacity = 1.0)
        - "very_low": Minimal transparency (opacity = 0.95)
        - "low": Light transparency (opacity = 0.90)
        - "medium": Medium transparency (opacity = 0.85)
        - "high": High transparency (opacity = 0.80)
        - "very_high": Very high transparency (opacity = 0.70)
        - "super_high": Maximum transparency (opacity = 0.60)
      '';
    };

    # Editor configuration
    editor_command = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = ''
        Editor command - yazelix will always set this as EDITOR.

        - null (default): Use yazelix's Nix-provided Helix - full integration
        - "nvim": Use Neovim - first-class support with full integration
        - "hx": Use system Helix from PATH (set helix_runtime_path only when your runtime lives outside Helix's normal discovery paths)
        - Other editors: "vim", "nano", "emacs", etc. (basic integration only)
      '';
    };

    helix_runtime_path = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = ''
        Custom Helix runtime path - only set this if editor_command points to a custom Helix build.

        IMPORTANT: The runtime MUST match your Helix binary version to avoid startup errors.
        Example: "/home/user/helix/runtime" for a custom Helix build in ~/helix
      '';
    };

    enable_sidebar = mkOption {
      type = types.bool;
      default = true;
      description = "Enable or disable the Yazi sidebar";
    };

    disable_zellij_tips = mkOption {
      type = types.bool;
      default = true;
      description = "Disable Zellij tips popup on startup for cleaner launches";
    };

    zellij_rounded_corners = mkOption {
      type = types.bool;
      default = true;
      description = "Enable rounded corners for Zellij pane frames";
    };

    support_kitty_keyboard_protocol = mkOption {
      type = types.bool;
      default = false;
      description = "Enable Kitty keyboard protocol in Zellij (disable if dead keys stop working)";
    };

    zellij_theme = mkOption {
      type = types.str;
      default = "default";
      description = ''
        Zellij color theme (37 built-in themes available).

        Dark themes: ansi, ao, atelier-sulphurpool, ayu_mirage, ayu_dark, catppuccin-frappe,
        catppuccin-macchiato, cyber-noir, blade-runner, retro-wave, dracula, everforest-dark,
        gruvbox-dark, iceberg-dark, kanagawa, lucario, menace, molokai-dark, night-owl, nightfox,
        nord, one-half-dark, onedark, solarized-dark, tokyo-night-dark, tokyo-night-storm,
        tokyo-night, vesper

        Light themes: ayu_light, catppuccin-latte, everforest-light, gruvbox-light,
        iceberg-light, dayfox, pencil-light, solarized-light, tokyo-night-light
      '';
    };

    zellij_widget_tray = mkOption {
      type = types.listOf types.str;
      default = [
        "editor"
        "shell"
        "term"
        "cpu"
        "ram"
      ];
      description = "Zjstatus widget tray order (editor/shell/term/cpu/ram)";
    };

    zellij_custom_text = mkOption {
      type = types.str;
      default = "";
      description = "Optional short zjstatus badge shown before YAZELIX. Trimmed and capped at 8 characters.";
    };

    popup_program = mkOption {
      type = types.listOf types.str;
      default = [ "lazygit" ];
      description = ''
        Default transient popup command for `yzx popup` and the default popup keybinding.
        Use an argv-style list, eg. [ "lazygit" ] or [ "claude-code" "--continue" ].
      '';
    };

    popup_width_percent = mkOption {
      type = types.intBetween 1 100;
      default = 90;
      description = "Width of the managed popup as a percentage of the current tab.";
    };

    popup_height_percent = mkOption {
      type = types.intBetween 1 100;
      default = 90;
      description = "Height of the managed popup as a percentage of the current tab.";
    };

    yazi_plugins = mkOption {
      type = types.listOf types.str;
      default = [
        "git"
        "starship"
      ];
      description = "Yazi plugins to load (core plugins auto_layout and sidebar_status are always loaded)";
    };

    yazi_theme = mkOption {
      type = types.str;
      default = "default";
      description = ''
        Yazi color theme (flavor). 25 built-in flavors available (19 dark + 5 light + default).
        Use "random-dark" or "random-light" to pick a different theme on each yazelix restart.
        Browse flavors: https://github.com/yazi-rs/flavors
      '';
    };

    yazi_sort_by = mkOption {
      type = types.enum [
        "alphabetical"
        "natural"
        "modified"
        "created"
        "size"
      ];
      default = "alphabetical";
      description = "Default file sorting method";
    };

    debug_mode = mkOption {
      type = types.bool;
      default = false;
      description = "Enable verbose debug logging";
    };

    skip_welcome_screen = mkOption {
      type = types.bool;
      default = false;
      description = "Skip the welcome screen on startup";
    };

    ascii_art_mode = mkOption {
      type = types.enum [
        "static"
        "animated"
      ];
      default = "animated";
      description = "ASCII art display mode";
    };

    show_macchina_on_welcome = mkOption {
      type = types.bool;
      default = true;
      description = "Show macchina system info on welcome screen";
    };

    persistent_sessions = mkOption {
      type = types.bool;
      default = false;
      description = "Enable persistent Zellij sessions";
    };

    session_name = mkOption {
      type = types.str;
      default = "yazelix";
      description = "Session name for persistent sessions";
    };

    zellij_default_mode = mkOption {
      type = types.enum [
        "normal"
        "locked"
      ];
      default = "normal";
      description = ''
        Startup mode for new Zellij sessions.
        - "normal": Yazelix default, starts unlocked
        - "locked": start in Zellij locked mode for compatibility with other TUIs
      '';
    };

    pack_names = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Packs to enable (must match pack_declarations keys)";
    };

    pack_declarations = mkOption {
      type = types.attrsOf (types.listOf types.str);
      default = {
        # AI coding agents (from llm-agents.nix)
        ai_agents = [
          "claude-code"
          "codex"
          "justcode"
          "gemini-cli"
          "pi"
          "opencode"
          "amp"
          "cursor-agent"
          "goose-cli"
          "tru"
        ];
        # AI support tools (from llm-agents.nix)
        ai_tools = [
          "coderabbit-cli"
          "ccusage"
          "ccusage-amp"
          "ccusage-codex"
          "ccusage-opencode"
          "beads"
          "beads-rust"
          "beads-viewer"
          "openclaw"
          "picoclaw"
          "zeroclaw"
        ];
        # unfree = []; # For unfree nixpkgs packages
        config = [
          "mpls"
          "yaml-language-server"
        ];
        file-management = [
          "ouch"
          "erdtree"
          "serpl"
        ];
        git = [
          "onefetch"
          "gh"
          "prek"
        ];
        jj = [
          "jujutsu"
          "lazyjj"
          "jjui"
        ];
        maintainer = [
          "gh"
          "prek"
          "tru"
          "beads-rust"
          "beads-viewer"
          "rust_wasi_toolchain"
        ];
        python = [
          "ruff"
          "uv"
          "ty"
          "python3Packages.ipython"
        ];
        rust = [
          "rust_toolchain"
          "cargo-edit"
          "cargo-watch"
          "cargo-nextest"
          "cargo-audit"
        ];
        rust_maintainer = [
          "cargo-update"
          "cargo-binstall"
        ];
        rust_wasi = [
          "rust_wasi_toolchain"
        ];
        nix = [
          "nil"
          "nixd"
          "nixfmt"
        ];
        ts = [
          "nodePackages.typescript-language-server"
          "tailwindcss-language-server"
          "biome"
          "oxlint"
        ];
        modern_js = [
          "bun"
          "deno"
        ];
        go = [
          "gopls"
          "golangci-lint"
        ];
        go_extra = [
          "delve"
          "air"
          "govulncheck"
        ];
        kotlin = [
          "kotlin-language-server"
          "ktlint"
          "detekt"
          "gradle"
        ];
        writing = [
          "typst"
          "tinymist"
          "pandoc"
          "markdown-oxide"
        ];
      };
      description = "Pack declarations mapping names to nixpkgs package strings (supports dotted paths)";
    };

    user_packages = mkOption {
      type = types.listOf types.package;
      default = [ ];
      description = "Additional packages to install in Yazelix environment";
    };
  };

  config = mkIf cfg.enable {
    # Desktop integration - copy yazelix assets
    xdg.configFile."yazelix/assets/logo.png".source = ../assets/logo.png;
    xdg.configFile."yazelix/assets/icons/48x48/yazelix.png".source = ../assets/icons/48x48/yazelix.png;
    xdg.configFile."yazelix/assets/icons/64x64/yazelix.png".source = ../assets/icons/64x64/yazelix.png;
    xdg.configFile."yazelix/assets/icons/128x128/yazelix.png".source =
      ../assets/icons/128x128/yazelix.png;
    xdg.configFile."yazelix/assets/icons/256x256/yazelix.png".source =
      ../assets/icons/256x256/yazelix.png;
    xdg.configFile."yazelix/docs/desktop_icon_setup.md".source = ../docs/desktop_icon_setup.md;

    # Desktop entry for application launcher
    xdg.desktopEntries.yazelix = {
      name = "Yazelix";
      comment = "Yazi + Zellij + Helix integrated terminal environment";
      exec = "${config.xdg.configHome}/yazelix/shells/posix/desktop_launcher.sh";
      icon = "yazelix";
      categories = [ "Development" ];
      type = "Application";
      settings = {
        StartupWMClass = "com.yazelix.Yazelix";
      };
    };

    # Generate yazelix.toml configuration file
    xdg.configFile."yazelix/yazelix.toml" = {
      text =
        let
          editorCommand = if cfg.editor_command != null then cfg.editor_command else "";
          ghosttyTrailEffectLine =
            if cfg.ghostty_trail_effect != null then
              [ "ghostty_trail_effect = ${escapeString cfg.ghostty_trail_effect}" ]
            else
              [ ];
          ghosttyModeEffectLine =
            if cfg.ghostty_mode_effect != null then
              [ "ghostty_mode_effect = ${escapeString cfg.ghostty_mode_effect}" ]
            else
              [ ];
          helixRuntimeLine =
            if cfg.helix_runtime_path != null then
              [ "runtime_path = ${escapeString cfg.helix_runtime_path}" ]
            else
              [ ];
        in
        lib.concatStringsSep "\n" (
          [
            "# Generated by the Yazelix Home Manager module."
            "# Edit your Home Manager configuration instead of this file."
            ""
            "[core]"
            "recommended_deps = ${boolToToml cfg.recommended_deps}"
            "yazi_extensions = ${boolToToml cfg.yazi_extensions}"
            "yazi_media = ${boolToToml cfg.yazi_media}"
            "debug_mode = ${boolToToml cfg.debug_mode}"
            "skip_welcome_screen = ${boolToToml cfg.skip_welcome_screen}"
            "show_macchina_on_welcome = ${boolToToml cfg.show_macchina_on_welcome}"
            "refresh_output = ${escapeString cfg.refresh_output}"
            "max_jobs = ${escapeString cfg.max_jobs}"
            "build_cores = ${escapeString cfg.build_cores}"
            ""
            "[helix]"
            "mode = ${escapeString cfg.helix_mode}"
          ]
          ++ helixRuntimeLine
          ++ [
            ""
            "[editor]"
            "command = ${escapeString editorCommand}"
            "enable_sidebar = ${boolToToml cfg.enable_sidebar}"
            ""
            "[shell]"
            "default_shell = ${escapeString cfg.default_shell}"
            "extra_shells = ${listToToml cfg.extra_shells}"
            ""
            "[terminal]"
            "terminals = ${listToToml cfg.terminals}"
            "manage_terminals = ${boolToToml cfg.manage_terminals}"
            "config_mode = ${escapeString cfg.terminal_config_mode}"
            "ghostty_trail_color = ${escapeString cfg.ghostty_trail_color}"
          ]
          ++ ghosttyTrailEffectLine
          ++ ghosttyModeEffectLine
          ++ [
            "ghostty_trail_glow = ${escapeString cfg.ghostty_trail_glow}"
            "transparency = ${escapeString cfg.transparency}"
            ""
            "[zellij]"
            "disable_tips = ${boolToToml cfg.disable_zellij_tips}"
            "rounded_corners = ${boolToToml cfg.zellij_rounded_corners}"
            "support_kitty_keyboard_protocol = ${boolToToml cfg.support_kitty_keyboard_protocol}"
            "theme = ${escapeString cfg.zellij_theme}"
            "widget_tray = ${listToToml cfg.zellij_widget_tray}"
            "custom_text = ${escapeString cfg.zellij_custom_text}"
            "popup_program = ${listToToml cfg.popup_program}"
            "popup_width_percent = ${toString cfg.popup_width_percent}"
            "popup_height_percent = ${toString cfg.popup_height_percent}"
            "persistent_sessions = ${boolToToml cfg.persistent_sessions}"
            "session_name = ${escapeString cfg.session_name}"
            "default_mode = ${escapeString cfg.zellij_default_mode}"
            ""
            "[yazi]"
            "plugins = ${listToToml cfg.yazi_plugins}"
            "theme = ${escapeString cfg.yazi_theme}"
            "sort_by = ${escapeString cfg.yazi_sort_by}"
            ""
            "[ascii]"
            "mode = ${escapeString cfg.ascii_art_mode}"
            ""
            "[packs]"
            "enabled = ${listToToml cfg.pack_names}"
            "user_packages = ${packagesToToml cfg.user_packages}"
            ""
            "[packs.declarations]"
            ""
          ]
          ++ packDeclarationsToToml cfg.pack_declarations
          ++ [
            ""
          ]
        )
        + "\n";
    };
  };
}
