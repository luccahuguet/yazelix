# Yazelix Home Manager module

The module installs one complete Yazelix package and optionally owns sparse files under `~/.config/yazelix`

## Quick start

Add the main Yazelix flake and its module to your Home Manager configuration

```nix
{
  inputs.yazelix = {
    url = "github:luccahuguet/yazelix";
    inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { home-manager, nixpkgs, yazelix, ... }: {
    homeConfigurations.your-user = home-manager.lib.homeManagerConfiguration {
      pkgs = import nixpkgs { system = "x86_64-linux"; };
      modules = [
        yazelix.homeManagerModules.default
        {
          home.username = "your-user";
          home.homeDirectory = "/home/your-user";
          home.stateVersion = "24.11";
          programs.yazelix.enable = true;
        }
      ];
    };
  };
}
```

Run `home-manager switch`, open a fresh shell, then run `yzx launch`

The minimal declaration installs the complete Yazelix package and creates no user config files, so all behavior follows packaged defaults

## Public options

```nix
programs.yazelix = {
  enable = true;

  # Optional whole-package replacement
  package = inputs.yazelix.packages.${pkgs.system}.yazelix;

  # Optional sparse semantic config
  config.settings = {
    editor.command = "hx";
    welcome.enabled = false;
  };

  # Optional native files use exactly one of text or source
  config.cursors.source = ./cursors.toml;
  config.mars.text = ''
    [window]
    opacity = 0.9
  '';
};
```

`package` replaces the complete package and is not a granular package builder or host-tool selector

`config.settings = null` is the default and leaves `~/.config/yazelix/config.toml` absent

`config.settings = {}` explicitly creates an empty Home Manager-owned `config.toml`

Any nonempty `config.settings` value renders exactly that TOML tree without copying packaged defaults, so removing a field returns it to packaged inheritance after the next switch

Home Manager-owned files are store-backed and read-only, so edit the declaration and rerun `home-manager switch` instead of editing the generated path

## Native files

Every native option is nullable and accepts exactly one of these shapes

```nix
config.mars.text = ''
  [window]
  opacity = 0.9
'';
```

```nix
config.mars.source = ./mars.toml;
```

The supported paths are

| Home Manager option | Installed path | Classic behavior |
| --- | --- | --- |
| `config.cursors` | `yazelix/cursors.toml` | consumed |
| `config.mars` | `yazelix/mars/config.toml` | consumed |
| `config.zellij` | `yazelix/zellij/config.kdl` | consumed |
| `config.helix.config` | `yazelix/helix/config.toml` | consumed |
| `config.helix.languages` | `yazelix/helix/languages.toml` | consumed |
| `config.yazi.config` | `yazelix/yazi/yazi.toml` | consumed |
| `config.yazi.init` | `yazelix/yazi/init.lua` | consumed |
| `config.yazi.keymap` | `yazelix/yazi/keymap.toml` | consumed when compatible with the packaged Yazi version |
| `config.nu.env` | `yazelix/nu/env.nu` | staged for Yazelix Nova v1 |
| `config.nu.config` | `yazelix/nu/config.nu` | staged for Yazelix Nova v1 |
| `config.starship` | `yazelix/starship.toml` | staged for Yazelix Nova v1 |
| `config.helix.module` | `yazelix/helix/helix.scm` | staged for Yazelix Nova v1 |
| `config.helix.init` | `yazelix/helix/init.scm` | staged for Yazelix Nova v1 |
| `config.yazi.package` | `yazelix/yazi/package.toml` | staged for Yazelix Nova v1 |
| `config.yazi.theme` | `yazelix/yazi/theme.toml` | staged for Yazelix Nova v1 |

Staged files are installed so declarations can be prepared before the Nova source swap, but Classic does not claim to consume them

The Helix Steel extension activates in Nova only when both `config.helix.module` and `config.helix.init` are present

Home Manager does not rewrite Yazi declarations for version compatibility or resolve packaged-name collisions

The module does not own `zellij/plugins.kdl`, plugin or flavor directories, ambient tool configuration, or generated runtime state

## Migrating an older declaration

The final Classic bridge intentionally removes the broad package-shaping and per-setting option languages without aliases

After Nova replaces the default branch, pin `yazelix.url = "github:luccahuguet/yazelix/v17.11";` while running this migration, launch the bridge once, then update the input to Nova

Before

```nix
programs.yazelix = {
  enable = true;
  manage_config = true;
  terminal = "mars";
  editor_command = "hx";
  welcome_enabled = false;
  runtime_tool_sources.helix = "host";
  components.cursors = false;
  agent_usage_programs = [ ];
};
```

After

```nix
programs.yazelix = {
  enable = true;
  config.settings = {
    editor.command = "hx";
    welcome.enabled = false;
  };
};
```

Migration mapping

| Removed surface | Replacement |
| --- | --- |
| `manage_config` | Declare `config.settings` when Home Manager should own `config.toml`, otherwise omit it |
| `manage_cursor_config` | Declare `config.cursors.text` or `config.cursors.source` |
| Explicit `package = null` | Remove the assignment to use the default complete package |
| `terminal = "mars"` | Remove the assignment because the default complete package owns Mars |
| `mars_package` | Build a complete compatible Yazelix package and pass it through `package` |
| `open_log_level` | `config.settings.open.log_level` |
| `shell_program` | `config.settings.shell.program` |
| `editor_command` | `config.settings.editor.command` |
| `agent_command` | `config.settings.agent.command` |
| `agent_args` | `config.settings.agent.args` |
| `welcome_enabled` | `config.settings.welcome.enabled` |
| `welcome_style` | `config.settings.welcome.style` |
| `welcome_duration_seconds` | `config.settings.welcome.duration_seconds` |
| `popup_side_margin` | `config.settings.popup.side_margin` |
| `popup_vertical_margin` | `config.settings.popup.vertical_margin` |
| `keybinding_config` | `config.settings.keybindings.config` |
| `keybinding_agent` | `config.settings.keybindings.agent` |
| `keybinding_git` | `config.settings.keybindings.git` |
| `keybinding_menu` | `config.settings.keybindings.menu` |
| `bar_widgets` | `config.settings.bar.widgets` |
| `popups` | `config.settings.popups` |
| `runtime_tool_sources` | No Home Manager replacement |
| `components` | No Home Manager replacement |
| `agent_usage_programs` | No Home Manager replacement |

Users who need a custom package shape can build one outside the module and pass the result through `programs.yazelix.package`

## Existing mutable files

Home Manager stops on collisions instead of silently replacing user-managed files

Before the first declarative takeover, inspect the paths you plan to own and move or import them explicitly

```bash
yzx home_manager prepare
yzx home_manager prepare --apply
home-manager switch
```

Omitting an option leaves the corresponding file user-owned

## Updating

Run `yzx update home_manager` from your Home Manager flake directory to print the input update and switch commands

Already-open Yazelix windows keep their current runtime, so open a fresh window after a successful switch

## Examples

- [Minimal flake](./examples/minimal_flake)
- [Package and config example](./examples/example.nix)
