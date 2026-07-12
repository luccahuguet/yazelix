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
      Whether Home Manager generates ~/.config/yazelix/config.toml.

      The default keeps Home Manager responsible for the Yazelix
      package/runtime/desktop integration while leaving config.toml as a
      normal mutable user file managed through `yzx edit` or your editor.

      Set this to true only when you want Home Manager to generate and own
      config.toml declaratively from programs.yazelix options.
    '';
  };

  manage_cursor_config = mkOption {
    type = types.bool;
    default = false;
    description = ''
      Whether Home Manager generates ~/.config/yazelix/cursors.toml.

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

  config.zellij = mkOption {
    type = types.nullOr (types.submodule {
      options = {
        text = mkOption {
          type = types.nullOr types.lines;
          default = null;
          description = "Inline zellij/config.kdl contents.";
        };
        source = mkOption {
          type = types.nullOr types.path;
          default = null;
          description = "zellij/config.kdl file to install.";
        };
      };
    });
    default = null;
    description = ''
      Guarded native Zellij preferences at ~/.config/yazelix/zellij/config.kdl.
      Set exactly one of text or source. Plugin declarations remain a normal
      user-owned zellij/plugins.kdl file.
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

      These support bar.widgets usage entries:
      - "tokenusage": claude_usage, codex_usage

      codex_usage is a combined 5h/week token and quota widget.
      claude_usage is a combined 5h/week token and quota widget.
      opencode_go_usage reads OpenCode's local SQLite database directly and does
      not require an extra usage binary.

      Set this to [] only if the Claude and Codex usage widgets are removed
      from bar_widgets or intentionally host-provided.
    '';
  };

  open_log_level = mkMainContractOption "open.log_level" {
    description = "Diagnostics written for managed Yazi open requests";
  };

  shell_program = mkMainContractOption "shell.program" {
    description = "Packaged shell launched in new Yazelix panes";
  };

  editor_command = mkMainContractOption "editor.command" {
    description = "Editor executable used by managed Yazi opens";
  };

  agent_command = mkMainContractOption "agent.command" {
    description = "Auto provider discovery or one executable for the managed agent surface";
  };

  agent_args = mkMainContractOption "agent.args" {
    description = "Arguments passed to a custom agent command";
  };

  welcome_enabled = mkMainContractOption "welcome.enabled" {
    description = "Show the startup welcome splash";
  };

  welcome_style = mkMainContractOption "welcome.style" {
    description = "Welcome splash style";
  };

  welcome_duration_seconds = mkMainContractOption "welcome.duration_seconds" {
    description = "Welcome splash duration in whole seconds";
  };

  popup_side_margin = mkMainContractOption "popup.side_margin" {
    description = "Left and right cell margin for managed popups";
  };

  popup_vertical_margin = mkMainContractOption "popup.vertical_margin" {
    description = "Top and bottom cell margin for managed popups";
  };

  keybinding_config = mkMainContractOption "keybindings.config" {
    description = "Key chord for the managed config popup";
  };

  keybinding_agent = mkMainContractOption "keybindings.agent" {
    description = "Key chord for the managed agent surface";
  };

  keybinding_git = mkMainContractOption "keybindings.git" {
    description = "Key chord for the managed Git popup";
  };

  keybinding_menu = mkMainContractOption "keybindings.menu" {
    description = "Key chord for the managed command palette popup";
  };

  bar_widgets = mkMainContractOption "bar.widgets" {
    description = "Top-bar widget order";
  };

  popups = mkOption {
    type = types.nullOr (types.attrsOf (types.submodule {
      options = {
        command = mkOption {
          type = types.str;
          description = "Popup executable";
        };
        args = mkOption {
          type = types.nullOr (types.listOf types.str);
          default = null;
          description = "Popup arguments";
        };
        title = mkOption {
          type = types.nullOr types.str;
          default = null;
          description = "Optional popup pane title";
        };
        keybinding = mkOption {
          type = types.str;
          description = "Popup key chord";
        };
        keep_alive = mkOption {
          type = types.nullOr types.bool;
          default = null;
          description = "Whether a focused toggle hides instead of closes the popup";
        };
      };
    }));
    default = null;
    description = "Custom popup definitions keyed by stable id";
  };
}
