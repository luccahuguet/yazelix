# Editor Configuration

Yazelix uses one semantic editor setting:

```toml
[editor]
command = "hx"
```

Omit `editor.command` to inherit the packaged default. The default `hx` and the
alias `helix` select Yazelix's bundled Helix binary and matching runtime.

## Supported Editors

### Yazelix Helix

The bundled `luccahuguet/yazelix-helix` fork is the first-class editor path. It
ships the matching runtime, managed config loading, Steel support, and the
editor bridge used by existing-pane Yazi opens.

Managed source files live under `~/.config/yazelix/helix/`, including
`config.toml`, `languages.toml`, `themes/`, and Steel plugin files. Native
`~/.config/helix/` remains user-owned. Run `yzx import helix` when you explicitly
want to adopt an existing native config.

The semantic root does not support an external Helix binary/runtime pair. Use
the bundled `hx` or `helix` command so the binary, runtime, and Yazelix bridge
stay aligned.

### Neovim

```toml
[editor]
command = "nvim"
```

Yazelix reuses the managed Neovim pane when possible. Add an editor-local
binding for `yzx reveal` if you want to reveal the current file in Yazi; see
[Neovim keybindings](./neovim_keybindings.md).

### Other Terminal Editors

```toml
[editor]
command = "vim"
```

Other executable tokens receive basic pane launch, cwd, and tab behavior. They
do not receive Helix- or Neovim-specific same-instance and reveal integration.

## Managed Editor Behavior

- Yazi file opens target the managed `editor` pane
- Helix and Neovim reuse the existing managed instance when supported
- Editors launched manually from a shell pane remain ordinary panes
- `EDITOR` uses the configured command inside the Yazelix environment
- The managed Helix reveal binding is `Alt+r`

## Home Manager

```nix
programs.yazelix = {
  enable = true;
  config.settings.editor.command = "nvim";
};
```

Leave `config.settings.editor.command` unset to inherit bundled Helix. Declaring
`config.settings` makes Home Manager own `config.toml`, so change the
declaration and run `home-manager switch` instead of editing the generated file.

## Troubleshooting

- Run `yzx doctor` for editor and managed-runtime diagnostics
- Use `yzx reset config --yes` only when you intentionally want to discard the
  semantic override and return to packaged defaults
- A custom path ending in `/hx` or `/helix` is rejected because Yazelix cannot
  prove that its runtime and bridge match that binary

See the [troubleshooting guide](./troubleshooting.md) for runtime recovery.
