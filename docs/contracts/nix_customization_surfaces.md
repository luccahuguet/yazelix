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
programs.yazelix.runtime_tool_sources.zenith = "host";
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
    zenith = "host";
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

### Bootstrap Install Check

`shells/posix/install_check.sh` is the primary pre-install diagnostic surface. It must be usable as a standalone POSIX `sh` script so users can run it before installing Yazelix and before Nix is present on `PATH`.

`apps.${system}.install_check` and `packages.${system}.install_check` expose the same script through Nix for users who already have a working Nix installation.

The install check must remain small and read-only:

- it must not depend on the default Yazelix runtime package, selected terminal package, Helix, Zellij, child plugin artifacts, or generated runtime tree
- it must not run `sudo`, edit Nix configuration, install packages, or mutate user state
- it may inspect local Nix commands, platform identity, active Nix configuration, and Yazelix cache trust state
- it should treat missing Yazelix Cachix trust as a speed warning, not an install blocker
- it should print numbered next steps with the recommended `nix profile add --refresh --accept-flake-config github:luccahuguet/yazelix#yazelix` command, `yzx launch`, and optional cache setup guidance when Nix is available
- it should print numbered next steps with a Nix installer command and rerun command when Nix is missing

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
| `terminal = "ghostty"`, `"kitty"`, `"rio"`, `"wezterm"`, or Linux-only `"foot"`/`"ratty"` | `ghostty` | Implemented: selects one packaged terminal instead of bundling every terminal or selecting terminals through config | Generated terminal config follows the selected terminal | Keep implemented |
| `appearance_mode = "dark"`, `"light"`, or `"auto"` | `dark` | Implemented in Home Manager: declarative value for `appearance.mode` when `manage_config = true`; default ratconfig-owned setups keep `settings.jsonc` as the semantic owner | Ghostty and WezTerm use native static/auto theme selection; Rio and Foot use generated static palettes; Zellij and Yazi change only their implicit default themes | Keep implemented |
| `extra_terminal_launchers = [ "ghostty" "wezterm" ... ]` | `[]` | Implemented in Home Manager: installs additional Linux desktop launchers without changing the active runtime identity | Each extra entry points directly at that terminal variant package in the Nix store, so dependencies remain available without adding duplicate profile `bin/yzx` commands | Keep implemented |
| `components.cursors` | enabled | Implemented partial package omission: cursor shader assets and `yazelix_cursors_default.toml` are removed from the runtime tree; cursor registry code remains linked into `yazelix_core` until crate-level feature gates exist | Ghostty config generation omits Yazelix cursor shaders, cursor sidecar bootstrap is skipped, config UI hides cursor fields, Home Manager rejects cursor config ownership, and launch facts report `n/a` | Keep implemented |
| `components.screen` | enabled | Implemented behavior toggle: welcome/screen rendering remains linked into `yazelix_core` until crate-level feature gates exist | Home Manager requires `core.skip_welcome_screen = true` and rejects enabled screen saver settings; Zellij materialization rejects screen saver when disabled; `yzx screen` returns a disabled-component error | Keep implemented |
| Helix Steel authoring tools | bundled | Implemented `off`: omits `steel`, `steel-language-server`, `forge`, `cargo-steel-lib`, and `repl-connect`; implemented `host`: relies on host `steel` and `steel-language-server` | Managed Helix Steel plugin execution still uses the bundled Helix fork and generated config, so disabling these commands affects authoring/debugging only | Keep implemented |
| `components.status_bar` / integrated zjstatus | enabled | Not accepted yet: `zjstatus.wasm` is a real runtime asset, but the top/status bar is part of the current Zellij layout contract | Defer until layout ownership and barless/native Zellij layout behavior are designed; hiding widgets through `zellij.widget_tray` is not a package-saving toggle | Defer |
| `yazelix_zellij_bar` standalone package forwarding | available on demand | No Home Manager toggle needed: forwarded flake output is not installed unless the user asks for it | Integrated Yazelix consumes only the crate/API it needs for generated layouts | Reject toggle |
| `yazelix-zellij-popup` / `yzpp` | enabled | Rejected for now: Yazelix packages `yzpp.wasm` because popup, menu, and config UI panes all use the integrated popup path | A disabled mode would split one coherent popup surface across command failure, fallback behavior, default keybindings, generated Zellij plugin aliases, and doctor diagnostics; that extra component boundary is not worth the complexity | Reject for simplicity |
| `components.yazi_assets` / `yazelix-yazi-assets` | enabled | Rejected for now: reusable flavors, reusable plugins, and the bundled Starship Yazi config stay integrated through the child asset pack | A disabled mode would require a second first-party-only Yazi profile that removes child-provided flavors, `auto-layout.yazi`, `git.yazi`, `lazygit.yazi`, `starship.yazi`, and `yazelix_starship.toml`; the modest package/clutter savings do not justify that extra profile surface | Reject for simplicity |
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

The `yazelix-yazi-assets` asset pack remains integrated. It is a real child repository with package contents that could be omitted, but the gain is modest and the cost is a second Yazi profile surface. The current default Yazi runtime links child flavors, `auto-layout.yazi`, `git.yazi`, `lazygit.yazi`, `starship.yazi`, and `yazelix_starship.toml`; generated keymaps and sidebar behavior can still call those plugins. Keeping one coherent integrated profile is simpler than maintaining a reduced first-party-only mode.

The integrated status bar remains deferred behind layout ownership and barless/native Zellij behavior. A widget-tray setting is not a storage-saving component toggle.

`yzpp` remains integrated. Popup, command-menu, and config-UI panes share the same plugin path, so an optional component would either leave broken commands behind or require a parallel fallback surface. Keeping one popup contract is simpler than supporting a disableable `yzpp` mode.

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
