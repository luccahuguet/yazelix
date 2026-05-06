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

Only tools marked hostable may use `host`. Only tools marked disableable may use `off`.

Bootstrap-critical tools such as Nushell, Zellij, the selected terminal package, Nix, graphics wrappers, and core POSIX utilities remain bundled until a separate contract says otherwise.

## Doctor Behavior

When a runtime tool is configured as `host`, `yzx doctor` must check the active `PATH` for the required commands and report actionable findings if they are missing.

Default bundled installs must not gain new warnings from runtime-tool source diagnostics.

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
