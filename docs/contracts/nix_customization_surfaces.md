# Nix Customization Surfaces

## Summary

Yazelix ships a batteries-included default runtime and exposes granular Nix customization through explicit APIs instead of a large matrix of public flake packages.

The default flake packages stay curated and reproducible. Users who want storage savings or host-managed tools use Home Manager or the advanced Nix builder.

## Supported Surfaces

### Default Flake Packages

The default flake package surface is for users who want Yazelix to work with minimal decisions.

Default flake packages:

- keep bundled runtime tools by default
- expose a small number of curated variants only when they are broadly useful
- do not expose every granular runtime-tool or component combination as a named package

### Home Manager Module

Home Manager is the friendly granular configuration surface.

It may expose typed options such as:

```nix
programs.yazelix.runtime_tool_sources.lazygit = "host";
programs.yazelix.runtime_tool_sources.bottom = "host";
```

Home Manager options should translate into the same package-builder arguments used by non-Home-Manager users.

### `lib.${system}.mkYazelix`

`lib.${system}.mkYazelix` is the advanced granular package-builder API for flake users who do not use Home Manager.

It may accept arguments such as:

```nix
inputs.yazelix.lib.${system}.mkYazelix {
  inherit pkgs;
  runtimeToolSources = {
    lazygit = "host";
    bottom = "host";
  };
  components = {
    screen = true;
    cursors = true;
  };
}
```

The default arguments must produce the same behavior as the default `.#yazelix` package.

### Overlay

`overlays.default` is a package-set integration surface.

It should expose the default bundled `yazelix` package. Granular customization remains explicit through `lib.${system}.mkYazelix` instead of hidden overlay magic.

## Runtime Tool Source Modes

Runtime tools may support these source modes:

- `bundled`: Yazelix includes the tool package and exports its commands in the runtime
- `host`: Yazelix omits the package/export and lets the inherited host `PATH` provide the commands
- `off`: Yazelix omits the package/export and treats dependent features as unavailable or warning-worthy

Only tools marked hostable may use `host`. Only tools marked disableable may use `off`. `mise` and `tombi` default to `host`; other omitted tools default to `bundled`.

Bootstrap-critical tools such as Nushell, Zellij, the selected terminal package, Nix, graphics wrappers, and core POSIX utilities remain bundled until a separate contract says otherwise.

## Component Toggle Policy

Component toggles are coarser than runtime tool source modes. They control Yazelix-owned subsystems, generated config, and child-repo-backed integrations. The default state must remain the complete integrated Yazelix experience.

Home Manager is the user-facing component-toggle surface. `lib.${system}.mkYazelix` receives the same `components` values for advanced package users. Default flake packages do not grow a named package for every component combination.

Disabling a component is supported only when all of these are true:

- the package can actually omit a dependency, asset set, generated config path, or runtime behavior
- generated configs no longer reference disabled assets or commands
- defaults still work when the component remains enabled
- disabled runtime behavior fails fast with a clear error instead of silently degrading
- the setting saves storage, closure size, startup work, or meaningful generated clutter

The current evaluated matrix is:

| Component or surface | Default | Current package impact | Generated-config impact when disabled | Decision |
| --- | --- | --- | --- | --- |
| `runtime_tool_sources.<tool> = "host"` for leaf tools | bundled | Implemented: omits supported leaf tool packages and exports, then relies on host `PATH` | Runtime manifest records host source and doctor checks required commands | Keep implemented |
| `runtime_tool_sources.mise` and `runtime_tool_sources.tombi` | host | Implemented default omission: these host/maintainer-adjacent tools are not bundled unless explicitly set to `bundled` | Runtime manifest records host source; generated shell initializers omit `mise` cleanly when absent, and doctor reports missing default optional integrations as informational | Keep implemented |
| `agent_usage_programs = [ "tokenusage" ]` | on | Implemented opt-out: includes `tokenusage` for the default Codex/Claude status widgets, and omits it only when the list is set to `[]` | Agent usage widgets in the default tray have their helper available; users who remove those widgets can omit the helper explicitly | Keep implemented |
| `terminal = "ghostty"`, `"kitty"`, `"rio"`, `"yzxterm"`, `"wezterm"`, or Linux-only `"foot"`/`"ratty"` | `ghostty` | Implemented: selects one packaged terminal instead of bundling every terminal or selecting terminals through config | Generated terminal config follows the selected terminal; Yazelix Terminal additionally reuses the child package profile config, injects `terminal.transparency`, and keeps default Rio trail cursor without packaged `custom-shader` | Keep implemented |
| `extra_terminal_launchers = [ "ghostty" "wezterm" ... ]` | `[]` | Implemented in Home Manager: installs additional Linux desktop launchers without changing the active runtime identity | Each extra entry points directly at that terminal variant package in the Nix store, so dependencies remain available without adding duplicate profile `bin/yzx` commands | Keep implemented |
| `yzxterm_profile = "full"`, `"baseline"`, or `"shaders"` | `full` | Implemented in Home Manager: selects the generated Yazelix Terminal profile for activation, Linux desktop launches, and new shell sessions | `full` keeps Rio trail cursor without custom shaders, `baseline` disables effects, and `shaders` passes `YAZELIX_TERMINAL_PROFILE=shaders` directly into activation, the desktop entry, and the Home Manager session so app-launcher and shell environments do not decide the profile | Keep implemented |
| `yzxterm_emoji_font = "noto"`, `"twitter"`, or `"serenityos"` | `noto` | Implemented in Home Manager: selects a child-owned Yazelix Terminal emoji fallback preset for activation, Linux desktop launches, and new shell sessions | Main Yazelix passes `YAZELIX_TERMINAL_EMOJI_FONT` and reads the matching yzxterm package config root; yzxterm owns the bundled font paths, font family names, and Rio font fallback semantics | Keep implemented |
| `components.cursors` | enabled | Implemented partial package omission: cursor shader assets and `yazelix_ghostty_cursors_default.toml` are removed from the runtime tree; cursor registry code remains linked into `yazelix_core` until crate-level feature gates exist | Ghostty config generation omits Yazelix cursor shaders, cursor sidecar bootstrap is skipped, config UI hides cursor fields, Home Manager rejects cursor config ownership, and launch facts report `n/a` | Keep implemented |
| `components.screen` | enabled | Implemented behavior toggle: welcome/screen rendering remains linked into `yazelix_core` until crate-level feature gates exist | Home Manager requires `core.skip_welcome_screen = true` and rejects enabled screen saver settings; Zellij materialization rejects screen saver when disabled; `yzx screen` returns a disabled-component error | Keep implemented |
| Helix Steel authoring tools | bundled | Implemented `off`: omits `steel`, `steel-language-server`, `forge`, `cargo-steel-lib`, and `repl-connect`; implemented `host`: relies on host `steel` and `steel-language-server` | Managed Helix Steel plugin execution still uses the bundled Helix fork and generated config, so disabling these commands affects authoring/debugging only | Keep implemented |
| `components.status_bar` / integrated zjstatus | enabled | Not accepted yet: `zjstatus.wasm` is a real runtime asset, but the top/status bar is part of the current Zellij layout contract | Defer until layout ownership and barless/native Zellij layout behavior are designed; hiding widgets through `zellij.widget_tray` is not a package-saving toggle | Defer |
| `yazelix_zellij_bar` standalone package forwarding | available on demand | No Home Manager toggle needed: forwarded flake output is not installed unless the user asks for it | Integrated Yazelix consumes only the crate/API it needs for generated layouts | Reject toggle |
| `yazelix-zellij-popup` / `yzpp` | enabled | Not accepted yet: Yazelix packages `yzpp.wasm` because popup, menu, and config UI panes all use the integrated popup path | Defer until popup/menu/config UI can be disabled or replaced as a coherent component | Defer |
| `components.yazi_assets` / `yazelix-yazi-assets` | enabled | Not implemented yet: reusable flavors, reusable plugins, and the bundled Starship Yazi config are linked from the child asset pack | Candidate only after the Yazi writer can render a first-party-only profile that does not reference missing child assets, themes, or plugin commands | Evaluate next |
| Yazi preview/helper tools such as `p7zip`, `poppler`, and `resvg` | bundled | Implemented `off`: omits helper packages and exports; generated Yazelix config does not directly call these helpers | Doctor reports intentional disabled helper state instead of missing-host warnings | Keep implemented |
| `macchina` welcome summary helper | bundled unless host-sourced | Implemented `off`: omits `macchina` from runtime packages and exports | Home Manager requires `core.show_macchina_on_welcome = false`; doctor reports intentional disabled helper state | Keep implemented |

Do not add a toggle whose only effect is hiding Home Manager options or removing a forwarded flake output. A toggle must change package contents, generated runtime behavior, or validation in a way users can feel.

## Terminal Launcher Naming Decision

The current Home Manager surface uses `programs.yazelix.terminal` for the single profile-owned runtime and `programs.yazelix.extra_terminal_launchers` for additional launcher entries. If this surface is reshaped, use this structure:

```nix
programs.yazelix.terminals = {
  active_runtime = "ratty";
  launchers = [ "ghostty" "rio" "foot" "wezterm" ];
};
```

`active_runtime` names the one packaged runtime that owns the profile `yzx`, runtime identity, generated runtime state, and primary launcher. `launchers` names additional app-launch surfaces for terminal-specific runtime packages without changing the active runtime identity and without installing duplicate profile `bin/yzx` commands.

Keep the implementation invariant: each launcher points at the selected terminal variant package in the Nix store. The launcher must not call the profile-owned `yzx` unless it intentionally wants the active runtime. The profile `yzx` remains singular.

Avoid replacement names that imply the wrong contract:
- `alternates`: sounds like fallback behavior
- `enabled`, `available`, or `installed`: implies every listed terminal is equally available through profile `yzx`
- `desktop_entries`: names the Linux implementation instead of the cross-platform app-launch concept
- `desktop_launchers`: clearer than `desktop_entries`, but still too Linux-desktop-specific for future macOS app-bundle or Dock/Launchpad integration

## Component Audit Outcome

The 2026-05-08 optional child-component audit keeps the current defaults fully integrated and does not add a hot-path toggle immediately.

The next plausible component toggle is the `yazelix-yazi-assets` asset pack because it is a real child repository with package contents that can be omitted. It is not ready as a direct boolean until the generated Yazi profile has a documented reduced mode that avoids child-provided flavors, reusable plugins, and `yazelix_starship.toml`.

The integrated status bar remains deferred behind layout ownership and barless/native Zellij behavior. A widget-tray setting is not a storage-saving component toggle.

`yzpp` remains deferred until popup, command-menu, and config-UI panes have a single coherent off or replacement mode. A toggle that only removes `yzpp.wasm` while leaving those commands and keybindings active is invalid.

## Doctor Behavior

When a runtime tool is configured as `host`, `yzx doctor` must check the active `PATH` for the required commands and report actionable findings if they are missing.

Default bundled installs must not gain new warnings from runtime-tool source diagnostics.

When a component is disabled, doctor reports the disabled state as intentional and should report only invalid references to disabled commands, assets, widgets, or config ownership surfaces.

## Public Flake Presets

Curated granular flake presets are optional and demand-driven.

Before adding a named preset, the project must decide:

- the exact tool/component modes it represents
- why Home Manager and `lib.${system}.mkYazelix` are not enough
- how many named presets remain supportable

## Verification

- `nix flake show`
- `nix build .#yazelix`
- Nix eval/build checks for `lib.${system}.mkYazelix`
- Home Manager module checks for granular options
- `yzx doctor` checks for host-sourced tool diagnostics
