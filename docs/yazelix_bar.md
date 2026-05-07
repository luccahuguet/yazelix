# Yazelix Bar

`yazelix_bar` is a standalone Zellij/zjstatus preset for users who want the Yazelix top-bar style without adopting the full Yazelix workspace.

## Install Shape

The flake package is:

```bash
nix build .#yazelix_bar
```

The package installs:

- `bin/yazelix_bar_generate`
- `share/yazelix_bar/zjstatus.wasm`
- `share/yazelix_bar/yazelix_bar.kdl`
- `share/yazelix_bar/yazelix_bar.template.kdl`
- `share/yazelix_bar/generated/yazelix_bar.kdl`
- `share/yazelix_bar/examples/custom_command_widgets.kdl`
- `share/yazelix_bar/examples/yazelix_runtime_widgets.kdl`
- `share/doc/yazelix_bar/README.md`

Use `yazelix_bar.kdl` as a Zellij layout plugin block. The template keeps `__YAZELIX_BAR_ZJSTATUS_WASM__` for users who want to substitute a different pinned `zjstatus.wasm`. The generated preset is emitted by `yazelix_bar_generate` with package-local paths. The example snippets are small blocks to copy into the plugin body rather than alternate full presets.

## Minimal Zellij Layout Snippet

```kdl
layout {
    pane size=1 borderless=true {
        // Paste the contents of share/yazelix_bar/yazelix_bar.kdl here
    }
    pane
}
```

The packaged `yazelix_bar.kdl` already points at the package's `zjstatus.wasm` with a `file:` URL.

## Generic Boundary

The standalone default is intentionally generic:

- mode
- tabs
- session
- datetime
- Yazelix-branded colors and tab overflow behavior

It does not require:

- `yzx`
- `yzx_control`
- Yazelix runtime paths
- the Yazelix pane orchestrator
- Nushell
- tokenusage
- full Yazelix installation

## Optional Command Widgets

Standalone users can add zjstatus command widgets directly in their own copied preset. Command stdout should be short plain text because the KDL format owns the style.

The main customization knobs are:

- `format_left`, `format_center`, and `format_right` for widget order
- inline `#[fg=...]` and `#[bg=...]` style tags for color
- mode and tab format keys for labels and tab display
- `command_*_command`, `command_*_format`, and `command_*_interval` for custom command widgets

Generic zjstatus placeholders such as `{mode}`, `{tabs}`, `{session}`, and `{datetime}` work without Yazelix. To add a host/status command widget, start from:

```kdl
format_right "#[fg=#ff0088,bold]{session} {datetime} {command_host} #[fg=#00ccff,bold]YAZELIX BAR "

command_host_command "hostname -s"
command_host_format " #[fg=#00ff88,bold][{stdout}]"
command_host_interval "30"
```

The packaged `share/yazelix_bar/examples/custom_command_widgets.kdl` contains a slightly larger version of this pattern.

## Preset Generator

Use `yazelix_bar_generate` when brand text, colors, widget order, or generic command widgets should come from structured options instead of manual KDL edits.

```bash
yazelix_bar_generate \
  --wasm-url "file:/path/to/zjstatus.wasm" \
  --brand-label "DEV BAR" \
  --right "session,datetime,command:host,brand" \
  --command "host=hostname -s"
```

The `--left`, `--center`, and `--right` flags accept comma-separated tokens:

- `mode`
- `tabs`
- `session`
- `datetime`
- `brand`
- `command:name`

Command widgets use `--command name=command`, with optional `--command-format name=format` and `--command-interval name=seconds`. This covers generic provider/status widgets without making provider tools part of the default package.

AI usage widgets are first-class Yazelix value, but they are provider-driven:

- generic standalone users should point zjstatus command widgets at their own provider commands
- Yazelix users can use existing cached provider commands from `yzx_control zellij status-cache-widget ...`
- expensive provider polling should stay outside zjstatus hot loops or behind a cache

## Yazelix-Specific Widgets

Workspace, cursor, Claude, Codex, OpenCode Go, CPU, and RAM widgets remain Yazelix integration widgets when they rely on Yazelix runtime helpers or launch-scoped cache files.

The full Yazelix runtime consumes the shared `yazelix_bar` Rust renderer for widget tray and tab label formatting. The standalone package consumes the same vendored `configs/zellij/plugins/zjstatus.wasm` source at build time, so the package does not require manual artifact copying.

Use `share/yazelix_bar/examples/yazelix_runtime_widgets.kdl` only inside a full Yazelix runtime or after replacing the helper commands with your own paths. The generic standalone preset does not assume `yzx_control`, Nushell, Yazelix cache files, or provider usage tools exist.

## Current Limit

Zellij/zjstatus presets do not currently have a native include or variables layer. Use the generator for structured brand, color, order, and command-widget changes; copy `yazelix_bar.template.kdl` only when editing lower-level zjstatus keys that the generator does not expose.

## Release Process

Maintainers update the vendored zjstatus wasm through the normal repo update flow, then validate:

```bash
nix build .#yazelix_bar
cargo test --manifest-path rust_core/Cargo.toml -p yazelix_bar
```

If the standalone preset grows beyond zjstatus configuration, the next step is a real plugin decision rather than forking zjstatus by default.
