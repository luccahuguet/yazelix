# Role Placement Config Decision

## Status

Accepted for the next role-placement implementation.

This is a planning decision, not a live config contract. Current Yazelix still
uses `editor.sidebar_*`, `editor.hide_sidebar_on_file_open`,
`zellij.popup_*`, and the current keybinding maps until the implementation
beads land.

## Core Decision

Add one workspace-owned config section for role placement:

```jsonc
{
  "workspace": {
    "roles": {
      "file_tree": {
        "placement": "left_sidebar",
        "launcher": "yazi_file_tree",
        "hide_on_file_open": false
      },
      "editor": {
        "placement": "main_stack"
      },
      "agent": {
        "placement": "right_sidebar",
        "command": [
          "codex"
        ]
      },
      "git_client": {
        "placement": "bottom_popup",
        "command": [
          "lazygit"
        ]
      },
      "config_ui": {
        "placement": "top_popup",
        "command": [
          "yzx",
          "config"
        ]
      }
    },
    "placements": {
      "left_sidebar": {
        "size_percent": 20
      },
      "right_sidebar": {
        "size_percent": 40
      },
      "top_popup": {
        "width_percent": 90,
        "height_percent": 90
      },
      "bottom_popup": {
        "width_percent": 90,
        "height_percent": 90
      }
    }
  }
}
```

This replaces the old shape where sidebars lived under `editor` and popups
lived under `zellij`. The new shape makes the workspace model explicit:

- `workspace.roles` answers what a surface is and which placement owns it
- `workspace.placements` answers the placement geometry
- `launcher` selects a Yazelix-owned built-in launcher without exposing runtime
  script paths
- `command` is an argv list for command-launched roles
- `zellij.keybindings` answers which keys invoke Yazelix-owned actions
- `zellij.native_keybindings` stays limited to curated native Zellij policy

Do not put role placement under `zellij`. Zellij is the backend, not the user
mental model. Do not put role placement under `editor`. The file tree, agent,
git client, and config UI are workspace surfaces, not editor settings.

## Minimal Option Set

The first implementation should add only these configurable fields:

- role placement
- built-in launcher tokens for Yazelix-owned integration roles
- role command argv for command-launched roles
- `file_tree.hide_on_file_open`
- sidebar placement width
- popup placement width and height

Do not add these knobs in the first pass:

- `allow_both_sidebars`
- per-role lifecycle policy
- per-placement focus policy
- editor-as-popup profile
- fallback agent provider
- command aliases for old popup/sidebar actions
- arbitrary native Zellij action DSL

The hardcoded first-pass lifecycle remains:

- sidebars are single-open
- opening one sidebar closes the other
- popups pull focus
- repeated popup actions use absent-open, existing-focus, focused-close
- `command_pane` stays on the command-palette path outside placement config

## Role Rules

Accepted built-in roles:

- `file_tree`
- `editor`
- `agent`
- `git_client`
- `config_ui`

Role ids use snake_case. Built-in role ids are reserved.

The first implementation should reject unknown enabled role ids unless a role
registry entry exists. This keeps validation honest while leaving the
`workspace.roles` map shape ready for later user-defined popup roles.

The default `editor` role stays commandless in `workspace.roles`. The editor
binary and integration behavior remain in `editor.command` and
`helix.runtime_path`. Workspace config decides where the editor role lives, not
which editor is installed.

Role entries may use `launcher` or `command`, not both. `launcher` names a
Yazelix-owned integration that can hide runtime paths, generated script
locations, and adapter details from user config. `command` is for roles that
only need Yazelix to launch, focus, and close an argv list.

The first accepted launcher is `yazi_file_tree`. It expands to the existing
managed Yazi file-tree sidebar adapter and preserves editor-routing behavior.
Do not ask users or Home Manager configs to spell runtime adapter script paths.

`agent.command` defaults to `["codex"]`. Codex is the first-class Yazelix agent,
but the default Yazelix flake package should not bundle Codex. Codex remains a
host-installed command by default so the normal runtime does not gain a roughly
1 GiB optional agent dependency.

Home Manager users can install and update Codex through their normal Home
Manager package set. Non-Home Manager users can install Codex with their
preferred package manager. Yazelix should launch the configured argv from the
active runtime PATH and should fail clearly if the command is missing or not
authenticated.

Yazelix must not silently fall back from Codex to OpenCode, Claude, Gemini, or
another agent. The missing-command error should say that Codex is the default
agent, explain that it is not bundled by default, and tell users to install
Codex or set `workspace.roles.agent.command` to another argv.

Users who want OpenCode can set:

```jsonc
{
  "workspace": {
    "roles": {
      "agent": {
        "placement": "right_sidebar",
        "command": [
          "opencode"
        ]
      }
    }
  }
}
```

`git_client.command` defaults to `["lazygit"]`. It remains a launch/focus/close
role, not a rich Git subsystem.

## Placement Rules

Accepted placements:

- `main_stack`
- `left_sidebar`
- `right_sidebar`
- `top_popup`
- `bottom_popup`

First-pass placement constraints:

- `editor` must stay in `main_stack`
- `file_tree` may use `left_sidebar` or `right_sidebar`
- `agent` may use `left_sidebar` or `right_sidebar`
- `git_client` may use `top_popup` or `bottom_popup`
- `config_ui` may use `top_popup` or `bottom_popup`
- only one enabled role may claim a singleton placement
- `main_stack` is not a general command-launch placement yet

Invalid role or placement names should fail fast before launch with the setting
path, accepted values, and the nearest remediation. Do not silently move a role
to a default placement.

## Keybinding Policy

Role placement config does not own keys.

The accepted directional actions should be added to `zellij.keybindings` as
semantic Yazelix actions:

- `toggle_left_sidebar`
- `toggle_bottom_popup`
- `toggle_top_popup`
- `toggle_right_sidebar`

Those action names intentionally target placements, not default role names.
If a user moves `agent` from `right_sidebar` to `left_sidebar`, the left-sidebar
action opens the agent because the placement is what the key invokes.

Existing semantic action ids that name old implementation concepts should be
removed or renamed when the implementation lands:

- replace `popup` with `toggle_bottom_popup`
- replace `config` with `toggle_top_popup`
- replace `toggle_sidebar` with `toggle_left_sidebar`
- keep `menu` for the command palette
- keep `toggle_editor_sidebar_focus` only for the existing editor/file-tree
  focus behavior until a better focus-action model is designed

Do not keep legacy keybinding aliases by default. Users can remap the new
semantic actions through `zellij.keybindings`.

## Compatibility Policy

The role-placement implementation should replace these old settings:

- `editor.sidebar_width_percent`
- `editor.sidebar_command`
- `editor.sidebar_args`
- `editor.hide_sidebar_on_file_open`
- `zellij.popup_program`
- `zellij.popup_width_percent`
- `zellij.popup_height_percent`

Do not keep old runtime aliases. If an existing user config still contains old
fields after the implementation, strict config validation should fail with a
message that names the replacement path.

The implementation should update `settings_default.jsonc`, the schema,
`main_config_contract.toml`, Config UI metadata, Home Manager options, docs, and
upgrade notes in the same slice. Do not create a deferred cleanup bead for
removing the old settings.

## Home Manager Shape

Home Manager should mirror the JSON shape instead of adding many flat options:

```nix
programs.yazelix = {
  enable = true;
  manage_config = true;

  workspace.roles = {
    file_tree = {
      placement = "left_sidebar";
      launcher = "yazi_file_tree";
      hide_on_file_open = false;
    };
    editor.placement = "main_stack";
    agent = {
      placement = "right_sidebar";
      command = [ "codex" ];
    };
    git_client = {
      placement = "bottom_popup";
      command = [ "lazygit" ];
    };
    config_ui = {
      placement = "top_popup";
      command = [
        "yzx"
        "config"
      ];
    };
  };

  workspace.placements = {
    left_sidebar.size_percent = 20;
    right_sidebar.size_percent = 40;
    top_popup = {
      width_percent = 90;
      height_percent = 90;
    };
    bottom_popup = {
      width_percent = 90;
      height_percent = 90;
    };
  };
};
```

This will require the Home Manager module and main config contract to support a
nested object field or a small typed submodule for `workspace`. That is better
than flattening role placement into many one-off options.

When `manage_config = true`, Home Manager owns the generated complete
`settings.jsonc` snapshot just as it does today. The config UI stays read-only
for Home Manager-owned fields and should report "Takes effect after Home Manager
switch".

## Apply Mode

All `workspace.roles` and `workspace.placements` fields should use
`tab_session_restart`.

Role placement changes affect pane startup, Zellij layout generation, plugin
specs, and singleton runtime identity. They should not be presented as live
settings until the pane orchestrator and generated Zellij runtime have a narrow
reload protocol that can prove the active session matches the saved settings.

## Future Extension

The object shape leaves room for later user-defined command roles without
redesign:

```jsonc
{
  "workspace": {
    "roles": {
      // "notes": {
      //   "placement": "bottom_popup",
      //   "command": [
      //     "nvim",
      //     "NOTES.md"
      //   ],
      //   "enabled": false
      // }
    }
  }
}
```

The first implementation should not enable this example. A later user-defined
role pass must define role-id validation, placement sharing, action naming, and
duplicate lifecycle behavior before accepting enabled custom roles.

## Evidence Checked

- `settings_default.jsonc`
- `config_metadata/main_config_contract.toml`
- `config_metadata/yazelix_settings.schema.json`
- `config_metadata/config_ui_metadata.toml`
- `home_manager/module.nix`
- `home_manager/examples/example.nix`
- `docs/contracts/runtime_applied_settings.md`
- `docs/contracts/keybinding_action_ownership.md`
- `docs/contracts/floating_tui_panes.md`
- `docs/layouts.md`
- `docs/zellij-configuration.md`
