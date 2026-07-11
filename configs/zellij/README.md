# Yazelix Zellij Configuration

Yazelix keeps semantic workspace behavior in `settings.jsonc` and native Zellij input in two guarded files:

```text
~/.config/yazelix/zellij/config.kdl
~/.config/yazelix/zellij/plugins.kdl
```

`config.kdl` accepts native preferences that Yazelix does not own. `plugins.kdl` accepts only additive `plugins` and `load_plugins` blocks. Runtime-owned nodes, keybindings, and first-party plugin ids fail before materialization.

Plain `~/.config/zellij/config.kdl` is never loaded implicitly. Use `yzx import zellij` to split a compatible native file into the managed pair.

See [Zellij Configuration](../../docs/zellij-configuration.md) for the full boundary.
