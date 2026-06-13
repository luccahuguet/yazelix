# Zellij bar public surface

`yazelix-zellij-bar` is a Zellij bar plugin package, not a configuration framework.

The public standalone surface is:

```text
bin/yazelix_zellij_bar_widget
share/yazelix_zellij_bar/zjstatus.wasm
share/yazelix_zellij_bar/yazelix_zellij_bar.kdl
share/yazelix_zellij_bar/yazelix_zellij_bar.template.kdl
share/yazelix_zellij_bar/examples/standalone_zellij_layout.kdl
share/yazelix_zellij_bar/examples/yazelix_runtime_widgets.kdl
share/doc/yazelix_zellij_bar/README.md
```

There is no standalone configuration generator binary and no central `~/.config/yazelix_zellij_bar/config.toml`.

## Configuration model

KDL is the user configuration surface. Users enable, remove, reorder, and style widgets by editing the `zjstatus` plugin block directly.

Standalone KDL should use clean PATH-based commands:

```kdl
command_cpu_command "yazelix_zellij_bar_widget cpu"
command_codex_usage_command "yazelix_zellij_bar_widget codex"
```

This assumes the package is installed in the user's profile or another environment that places `yazelix_zellij_bar_widget` on `PATH`.

## Widget helper

`yazelix_zellij_bar_widget` is the only public helper binary. It prints short stdout segments for zjstatus command widgets:

```text
yazelix_zellij_bar_widget cursor
yazelix_zellij_bar_widget codex
yazelix_zellij_bar_widget claude
yazelix_zellij_bar_widget opencode_go
yazelix_zellij_bar_widget cpu
yazelix_zellij_bar_widget ram
```

Long flags are escape hatches, not the common standalone configuration.

## State files

Provider caches live under the user's cache directory:

```text
$XDG_CACHE_HOME/yazelix_zellij_bar
$HOME/.cache/yazelix_zellij_bar
```

Do not add a central TOML config file unless a future contract replaces KDL as the public configuration model.

## Yazelix runtime integration

The full Yazelix runtime may generate KDL with internal absolute helper paths because users do not author that generated runtime surface. This does not change the standalone package contract: standalone examples and package docs use readable PATH commands.

## Verification

- `cargo test` in `luccahuguet/yazelix-zellij-bar`
- `nix build .#yazelix_zellij_bar`
- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
