{
  agentUsageProgramNames,
  defaultTerminal,
  lib,
  mkMainContractOption,
  runtimeToolSourceModes,
  terminalDescriptionBullets,
  terminalVariants,
}:

with lib;

{
  enable = mkEnableOption "Yazelix terminal environment";

  package = mkOption {
    type = types.nullOr types.package;
    default = null;
    description = ''
      Yazelix package to expose through the Home Manager profile.

      The default builds Yazelix from this module's runtime options. Set this
      only when selecting a specific prebuilt package output instead.
    '';
  };

  manage_config = mkOption {
    type = types.bool;
    default = false;
    description = ''
      Whether Home Manager generates ~/.config/yazelix/settings.jsonc.

      The default keeps Home Manager responsible for the Yazelix
      package/runtime/desktop integration while leaving settings.jsonc as a
      normal mutable user file managed through `yzx edit` or your editor.

      Set this to true only when you want Home Manager to generate and own
      settings.jsonc declaratively from programs.yazelix options.
    '';
  };

  manage_cursor_config = mkOption {
    type = types.bool;
    default = false;
    description = ''
      Whether Home Manager generates ~/.config/yazelix_cursors/settings.jsonc.

      Cursor settings are independent from the main Yazelix settings file so
      the standalone yzc command and full Yazelix can share one cursor source.
      Set this to true only when you want Home Manager to own the cursor
      registry declaratively.
    '';
  };

  terminal = mkOption {
    type = types.enum terminalVariants;
    default = defaultTerminal;
    description = ''
      Packaged Yazelix terminal. Yazelix packages Mars; configure other
      terminal emulators to start Yazelix with `yzx enter`.

${terminalDescriptionBullets}
    '';
  };

  mars_package = mkOption {
    type = types.nullOr types.package;
    default = null;
    description = ''
      Override package for the Mars terminal child runtime.

      Set this only when testing a local Mars build or pinning a custom Mars
      package. The package must expose passthru.marsPackageMetadata.
    '';
  };

  config.mars = mkOption {
    type = types.nullOr (types.submodule {
      options = {
        text = mkOption {
          type = types.nullOr types.lines;
          default = null;
          description = "Inline sparse Mars config.toml override contents.";
        };
        source = mkOption {
          type = types.nullOr types.path;
          default = null;
          description = "Sparse Mars config.toml override file to install.";
        };
      };
    });
    default = null;
    description = ''
      Sparse native Mars override at ~/.config/yazelix/mars/config.toml.
      Set exactly one of text or source.
    '';
  };

  runtime_tool_sources = mkOption {
    type = types.attrsOf (types.enum runtimeToolSourceModes);
    default = { };
    description = ''
      Per-tool runtime source modes. Omitted tools default to "bundled",
      except mise and tombi, which default to "host".

      Supported values:
      - "bundled": include the Yazelix-packaged tool and export its commands
      - "host": omit the package/export and rely on the inherited host PATH
      - "off": omit the package/export when the tool explicitly supports disabling

      Host mode is supported for leaf tools such as lazygit, zenith, helix, steel,
      neovim, yazi, fzf, zoxide, starship, carapace, macchina, mise, tombi, git, jq,
      fd, and ripgrep. Bootstrap tools such as the Mars terminal, Nushell, Zellij,
      Nix, POSIX utilities, and graphics wrappers remain bundled.

      Off mode is supported for optional helpers such as steel, macchina, p7zip,
      poppler, and resvg. Disabled helpers are intentionally omitted from the
      packaged runtime and reported as disabled instead of missing.
    '';
  };

  components = mkOption {
    type = types.attrsOf types.bool;
    default = { };
    example = {
      cursors = false;
      screen = false;
    };
    description = ''
      Optional Yazelix runtime components. Omitted components default to true.

      Supported components:
      - "cursors": Yazelix cursor shader assets and shared cursor config integration
      - "screen": startup welcome animation and `yzx screen` renderer integration
    '';
  };

  agent_usage_programs = mkOption {
    type = types.listOf (types.enum agentUsageProgramNames);
    default = [ "tokenusage" ];
    description = ''
      Usage binaries to include in the Yazelix runtime.

      These support zellij.widget_tray usage entries:
      - "tokenusage": claude_usage, codex_usage

      codex_usage is a combined 5h/week token and quota widget.
      claude_usage is a combined 5h/week token and quota widget.
      opencode_go_usage reads OpenCode's local SQLite database directly and does
      not require an extra usage binary. Configure rendered windows with
      zellij_codex_usage_periods, zellij_claude_usage_periods, and
      zellij_opencode_go_usage_periods.

      Set this to [] only if the Claude and Codex usage widgets are removed
      from zellij_widget_tray or intentionally host-provided.
    '';
  };

  default_shell = mkMainContractOption "shell.default_shell" {
    description = "Default shell for Zellij sessions";
  };

  appearance_mode = mkMainContractOption "appearance.mode" {
    description = ''
      Global appearance mode for generated Yazelix themes.

      - "dark": keep the default dark Yazelix palette
      - "light": use light defaults where Yazelix owns the generated theme
      - "auto": use terminal-supported system appearance switching where available
    '';
  };

  editor_command = mkMainContractOption "editor.command" {
    description = ''
      Editor command - yazelix will always set this as EDITOR.

      - null (default): Use yazelix's Nix-provided Helix - full integration
      - "nvim": Use Neovim - first-class support with full integration
      - "hx": Use the packaged Helix command from the Yazelix runtime
      - Other editors: "vim", "nano", "emacs", etc. (basic integration only)
    '';
  };

  helix_external = mkMainContractOption "helix.external" {
    description = ''
      Custom Helix binary/runtime pair.

      Set this only when running a user-owned fork based on Yazelix Helix.
      Both binary and runtime_path are required because the runtime MUST
      match the Helix binary version. Vanilla/upstream Helix does not support
      Yazelix's managed --config-dir or bridge-backed Yazi open behavior.

      Example:
        {
          binary = "/home/user/helix/target/release/hx";
          runtime_path = "/home/user/helix/runtime";
        }
    '';
  };

  helix_steel_plugins = mkMainContractOption "helix.steel_plugins" {
    description = ''
      Helix Steel plugin selection.

      enabled selects bundled plugin ids from Yazelix's packaged plugin
      repository. extra declares user-owned plugin manifests whose source
      files are resolved below ~/.config/yazelix/helix/steel_plugins and
      copied into the generated Yazelix Helix runtime config.
    '';
  };

  hide_sidebar_on_file_open = mkMainContractOption "editor.hide_sidebar_on_file_open" {
    description = ''
      Whether Yazelix should hide the managed sidebar after opening a file from
      the Yazi file-tree sidebar.
    '';
  };

  left_sidebar_command = mkMainContractOption "workspace.left_sidebar.command" {
    description = "Terminal command used for the managed left sidebar pane. Defaults to `yzx`.";
  };

  left_sidebar_args = mkMainContractOption "workspace.left_sidebar.args" {
    description = ''
      Arguments passed to the managed left sidebar command.

      The default launches Yazelix's managed Yazi file-tree adapter with `yzx sidebar yazi`.
    '';
  };

  left_sidebar_width_percent = mkMainContractOption "workspace.left_sidebar.width_percent" {
    description = "Width of the open left sidebar as a percentage of the tab.";
  };

  right_sidebar_command = mkMainContractOption "workspace.right_sidebar.command" {
    description = "Terminal command used for the managed right sidebar pane. Defaults to yzx agent.";
  };

  right_sidebar_args = mkMainContractOption "workspace.right_sidebar.args" {
    description = "Arguments passed to the managed right sidebar command. Defaults to [ \"agent\" ].";
  };

  right_sidebar_width_percent = mkMainContractOption "workspace.right_sidebar.width_percent" {
    description = "Width of the open right sidebar as a percentage of the tab.";
  };

  disable_zellij_tips = mkMainContractOption "zellij.disable_tips" {
    description = "Disable Zellij tips popup on startup for cleaner launches";
  };

  zellij_pane_frames = mkMainContractOption "zellij.pane_frames" {
    description = "Show Zellij pane frames";
  };

  zellij_rounded_corners = mkMainContractOption "zellij.rounded_corners" {
    description = "Enable rounded corners for Zellij pane frames";
  };

  support_kitty_keyboard_protocol = mkMainContractOption "zellij.support_kitty_keyboard_protocol" {
    description = "Enable Kitty keyboard protocol in Zellij (disable if dead keys stop working)";
  };

  zellij_theme = mkMainContractOption "zellij.theme" {
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

  zellij_widget_tray = mkMainContractOption "zellij.widget_tray" {
    description = "Zjstatus widget tray order (session/editor/shell/term/workspace/usage/cpu/ram); dynamic entries read from a window-local cache";
  };

  zellij_widget_frame = mkMainContractOption "zellij.widget_frame" {
    description = "Zjstatus widget frame style: none, square, or round";
  };

  zellij_widget_separator = mkMainContractOption "zellij.widget_separator" {
    description = "Zjstatus separator between adjacent widgets: dot, pipe, empty, or space";
  };

  zellij_tab_label_mode = mkMainContractOption "zellij.tab_label_mode" {
    description = ''
      Zjstatus tab-label mode.

      - "full": show tab index and tab name
      - "compact": show tab index and state indicators only
    '';
  };

  zellij_codex_usage_display = mkMainContractOption "zellij.codex_usage_display" {
    description = "Codex usage widget display mode: token, quota, or both";
  };

  zellij_codex_usage_periods = mkMainContractOption "zellij.codex_usage_periods" {
    description = "Periods shown by the codex_usage widget: 5h, week";
  };

  zellij_claude_usage_display = mkMainContractOption "zellij.claude_usage_display" {
    description = "Claude usage widget display mode: token, quota, or both";
  };

  zellij_opencode_go_usage_display = mkMainContractOption "zellij.opencode_go_usage_display" {
    description = "OpenCode Go usage widget display mode: token, quota, or both";
  };

  zellij_opencode_go_usage_periods = mkMainContractOption "zellij.opencode_go_usage_periods" {
    description = "Periods shown by the opencode_go_usage widget: 5h, week, month";
  };

  zellij_claude_usage_periods = mkMainContractOption "zellij.claude_usage_periods" {
    description = "Periods shown by the claude_usage widget: 5h, week";
  };

  zellij_custom_text = mkMainContractOption "zellij.custom_text" {
    description = "Optional short zjstatus badge shown before YAZELIX. Trimmed and capped at 8 characters.";
  };

  popup_commands = mkMainContractOption "zellij.popup_commands" {
    description = ''
      Commands for built-in Yazelix popup surfaces.
      Defaults: bottom_popup = [ "lazygit" ], top_popup = [ "yzx" "config" "ui" ],
      menu = [ "yzx" "menu" ].
    '';
  };

  custom_popups = mkMainContractOption "zellij.custom_popups" {
    description = ''
      User-defined Yazelix popup surfaces.
      Default: { id = "zenith"; command = [ "zenith" ]; keybindings = [ "Alt Shift I" ]; keep_alive = true; }.
    '';
  };

  popup_width_percent = mkMainContractOption "zellij.popup_width_percent" {
    description = "Width of the managed popup as a percentage of the current tab.";
  };

  popup_height_percent = mkMainContractOption "zellij.popup_height_percent" {
    description = "Height of the managed popup as a percentage of the current tab.";
  };

  screen_saver_enabled = mkMainContractOption "zellij.screen_saver_enabled" {
    description = "Enable the opt-in idle `yzx screen` pane-orchestrator screen saver.";
  };

  screen_saver_idle_seconds = mkMainContractOption "zellij.screen_saver_idle_seconds" {
    description = "Seconds of Zellij input inactivity before the screen saver opens.";
  };

  screen_saver_style = mkMainContractOption "zellij.screen_saver_style" {
    description = "Animated `yzx screen` style to run when the idle screen saver opens.";
  };

  yazi_plugins = mkMainContractOption "yazi.plugins" {
    description = "Yazi plugins to load (core plugins auto_layout and sidebar_status are always loaded)";
  };

  yazi_command = mkMainContractOption "yazi.command" {
    description = "Custom Yazi binary for Yazelix-managed Yazi launches. Null uses `yazi` from PATH.";
  };

  yazi_ya_command = mkMainContractOption "yazi.ya_command" {
    description = "Custom `ya` CLI for Yazelix-managed reveal and sidebar-sync actions. Null uses `ya` from PATH.";
  };

  yazi_theme = mkMainContractOption "yazi.theme" {
    description = ''
      Yazi color theme (flavor). 25 built-in flavors available (19 dark + 5 light + default).
      Use "default" to keep Yazi's upstream built-in theme.
      Use "random-dark" or "random-light" to pick a different theme on each yazelix restart.
      Browse bundled Yazelix flavors: https://github.com/luccahuguet/yazelix-yazi-assets/tree/main/flavors
    '';
  };

  yazi_sort_by = mkMainContractOption "yazi.sort_by" {
    description = "Default file sorting method";
  };

  yazi_keybindings = mkMainContractOption "yazi.keybindings" {
    description = ''
      Semantic remaps for Yazelix-owned Yazi integration actions.

      Keys are action ids such as "open_directory_as_workspace_pane" and
      "open_zoxide_in_editor"; values are lists of generated Yazi bindings
      such as "<A-p>". Use an empty list to disable the generated binding for
      one action.
    '';
  };

  debug_mode = mkMainContractOption "core.debug_mode" {
    description = "Enable verbose debug logging";
  };

  skip_welcome_screen = mkMainContractOption "core.skip_welcome_screen" {
    description = "Skip the welcome screen on startup";
  };

  welcome_style = mkMainContractOption "core.welcome_style" {
    description = ''
      Welcome screen style.
      - "static": show the resting Yazelix logo frame only
      - "logo": show the branded animated logo reveal
      - "boids": alias for "boids_predator"
      - "boids_predator": show boids with predator/prey motion
      - "boids_schools": show species-separated boids schools
      - "mandelbrot": show the Seahorse/Misiurewicz Mandelbrot zoom
      - "game_of_life_gliders": show the glider-swarm Game of Life style
      - "game_of_life_oscillators": show the oscillator-garden Game of Life style
      - "game_of_life_bloom": show the bloom-field Game of Life style
      - "random": choose evenly across Game of Life, boids, and Mandelbrot families (never "static" or "logo")
    '';
  };

  welcome_duration_seconds = mkMainContractOption "core.welcome_duration_seconds" {
    description = ''
      Welcome animation duration in seconds for animated styles.
      The logo style keeps its fixed timing and ignores this value.
      Default: 2.0.
      Valid range: 0.2 to 8.0.
    '';
  };

  game_of_life_cell_style = mkMainContractOption "core.game_of_life_cell_style" {
    description = ''
      Game of Life cell rendering style.
      - "full_block": solid cells matching the old Nushell renderer
      - "dotted": braille scale-4 texture with the same footprint
    '';
  };

  show_macchina_on_welcome = mkMainContractOption "core.show_macchina_on_welcome" {
    description = "Show macchina system info on welcome screen";
  };

  zellij_default_mode = mkMainContractOption "zellij.default_mode" {
    description = ''
      Startup mode for new Zellij sessions.
      - "normal": Yazelix default, starts unlocked
      - "locked": start in Zellij locked mode for compatibility with other TUIs
    '';
  };

  zellij_keybindings = mkMainContractOption "zellij.keybindings" {
    description = ''
      Semantic remaps for Yazelix-owned Zellij actions.

      Keys are action ids such as "bottom_popup", "top_popup", "menu",
      "toggle_left_sidebar", and "move_focus_left_or_tab"; values are lists of
      Zellij key strings. Use an empty list to disable the generated binding
      for one action.
    '';
  };

  zellij_native_keybindings = mkMainContractOption "zellij.native_keybindings" {
    description = ''
      Curated native Zellij key policy remaps and unbinds managed by Yazelix.

      Keys are policy ids such as "scroll_mode", "scroll_mode_unbind",
      "move_tab_left", "move_pane_down", and "move_tab_left_unbind"; values
      are lists of Zellij key strings. Use an empty list to disable one native
      policy entry.
    '';
  };
}
