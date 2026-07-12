# Yazi Configuration

Yazelix packages a managed Yazi base and overlays native user files from:

```text
~/.config/yazelix/yazi/yazi.toml
~/.config/yazelix/yazi/keymap.toml
~/.config/yazelix/yazi/init.lua
~/.config/yazelix/yazi/plugins/
~/.config/yazelix/yazi/flavors/
```

Classic can preserve `yazi/package.toml` for the Nova source swap, but does not
copy it into generated Yazi state or apply it. Home Manager may also stage
`yazi/theme.toml`; Classic continues generating its own runtime theme until the
swap.

The semantic root does not own Yazi binaries, plugins, themes, sorting, or
keybindings. Do not add a `[yazi]` table to `config.toml`.

## Packaged Integration

The managed base provides:

- the `git` status plugin
- the `sidebar-state` bridge for tab-local reveal and cwd synchronization
- the `zoxide-editor` bridge on `Alt+z`
- a managed edit opener that targets the Yazelix `editor` pane
- packaged plugins and flavors from `yazelix-yazi-assets`

Generated output lives under `~/.local/share/yazelix/configs/yazi/` and is not a
user-editable source.

## Native Settings

Put normal Yazi settings in `yazi.toml`. For example:

```toml
[mgr]
sort_by = "mtime"
ratio = [1, 4, 0]
```

The user overlay merges with packaged defaults while preserving Yazelix's
managed edit opener. Repeated native arrays replace the packaged value instead
of being appended.

## Native Keybindings

Put additions and overrides in `keymap.toml`:

```toml
[[mgr.prepend_keymap]]
on = ["e"]
run = "open"
desc = "Open selected files"
```

The packaged open flow remains native Yazi behavior:

- `Enter` and `o` open selected files through the managed editor opener
- `O` opens Yazi's native “Open with” menu
- `Z` keeps Yazi's native Zoxide jump inside Yazi
- `Alt+z` retargets the managed editor and workspace through Yazelix
- `Alt+p` opens the selected directory in a new workspace pane

Press `~` inside Yazi for its complete live keymap.

## Plugins And Lua

Use `plugins/` and `init.lua` as active Classic Yazi inputs. After
installing a plugin into plain `~/.config/yazi/` with `ya pkg`, import it before
expecting the managed runtime to see it:

```bash
yzx import yazi
```

Custom plugin setup belongs in `init.lua`, for example:

```lua
require("yamb"):setup({
    jump_notify = true,
})
```

`yzx import yazi` copies `package.toml` for preservation, but package metadata
remains dormant in Classic. Nova validates and consumes that file after the
source swap.

## Importing Existing Yazi State

`yzx import yazi` copies the supported native files, plugin directories, and
flavor directories from `~/.config/yazi/` into the Yazelix-managed Yazi home.
Use `--force` only when you want backup-first replacement of existing managed
inputs:

```bash
yzx import yazi --force
```

Yazelix never loads plain `~/.config/yazi/` implicitly.

## Applying Changes

Open a fresh Yazelix window after editing native Yazi inputs. If generated state
is stale, run `yzx doctor --fix`, then open another fresh window. Do not edit the
generated files under `~/.local/share/yazelix/configs/yazi/` directly.
