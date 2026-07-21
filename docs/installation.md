# Installation and packages

The README covers first launch and the shortest install paths. This guide
describes package variants, platform support, Home Manager ownership, updates,
and measured closure sizes

## Package variants

The default `yzx` package includes Mars and a Linux desktop entry. The fixed
`runtime` package provides the same `bin/yzx`, workspace, and config without
Mars, Rio, or desktop assets. Its `launch` command explains that Mars is absent,
so use `enter` for the managed workspace. Both package and app outputs exist for
`x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, and `aarch64-darwin`

The `x86_64-linux`-only `lifeos_foundation_yzx` output composes canonical Nova
with the FlexNetOS toolchain. It deliberately owns one profile element, one
`bin/yzx` frontdoor, the agent desktop sources, and one default agent workspace.
The profile exposes one visible Yazelix Agent entry plus one hidden Claude URL
handler from `share/applications`; no post-install copy is created. The visible
entry executes
`/home/flexnetos/.nix-profile/bin/yzx launch` directly, with no parallel
regular/agent entry. Claude deep links execute
`/home/flexnetos/.nix-profile/bin/claude --handle-uri %u` through the same
profile-owned runtime boundary.

The FlexNetOS foundation uses `/home/flexnetos/.nix-profile` as an explicit
profile, including its generation links. A retired user XDG profile is a
legacy shadow, not an equivalent selector, and is archived during the checked
migration under `/home/flexnetos/.cache/flexnetos/archives/yazelix-nix-profile/`.
Generated runtime beneath the profile runtime link is evidence only and never
owns profile archives. Run
`/home/flexnetos/.nix-profile/bin/yazelix_profile_check` after
every foundation update; it fails when the retired selector exists even if both
paths resolve to identical bytes.

Evaluate the mutually exclusive Mars-free variant without adding it beside the
foundation element:

```nu
nix run github:FlexNetOS/yazelix#runtime -- enter
```

The default `#yazelix` and Mars-free `#runtime` outputs are supported evaluation
alternatives, not additional owners of the FlexNetOS foundation profile.

## Capability matrix

| Surface | Linux | `aarch64-darwin` |
| --- | --- | --- |
| Full and runtime packages | Build- and profile-tested on `x86_64-linux`, with flake outputs also covering `aarch64-linux` | Build-tested on a real GitHub macOS runner |
| Home Manager module | Activation closure build-tested on `x86_64-linux` | Activation closure build-tested on a real GitHub macOS runner |
| `enter` with managed Zellij, Yazi, and Helix | Contract-tested and used interactively | Packaged, with interactive workflow unverified |
| Full-package `launch` through Mars | Contract-tested and used interactively | Package build-tested, with Mars GUI unverified |
| Host editor delegation | Contract-tested with the selected host editor remaining host-owned | Packaged, with interactive delegation unverified |
| Desktop entry | Full package only, with none in the runtime package | None, as asserted by the macOS package and Home Manager builds |

`x86_64-darwin` remains an exposed, evaluated flake output rather than a
build-tested target. The current label is **build-tested on macOS, with
interactive workflow and Mars GUI unverified**

## Host terminals and SSH

`yzx enter` starts the managed Zellij, Yazi, and Helix workspace in the current
interactive terminal. It is the SSH/headless route and needs no Mars, desktop
entry, `DISPLAY`, or `WAYLAND_DISPLAY`

Nova guarantees the managed TUI workflow and configuration, not host clipboard,
image previews, cursor shaders, desktop notifications, or terminal graphics. It
does not provide SSH connectivity or remote file synchronization

## Installed size

The complete Nova package occupies a **2.28 GiB Nix store closure** across 619
store paths on `x86_64-linux`. The Mars-free runtime occupies **1.37 GiB** across
591 paths, saving **927 MiB**. Its evaluated source-build graph contains 5,664
derivations instead of 8,071, avoiding 2,407 derivations when nothing is cached.
These are locked-input measurements from 2026-07-12, and derivation counts
indicate potential work, not guaranteed compilations. Closure size is realized
and unpacked, not compressed download size, and an existing Nix store may
already contain shared paths

The module figures below are complete closures for the package roots Nova uses.
They overlap through common libraries and tools, so they do not add up to the
Nova total

| Runtime scope | Closure size | What the measurement includes |
| --- | ---: | --- |
| **Nova (`yzx`)** | **2.28 GiB** | Entire launcher, terminal, workspace, editor, file manager, shell, Git tools, plugins, fonts, and configuration assets |
| **Nova runtime** | **1.37 GiB** | Same command, workspace, tools, config, and cursor schema without Mars, Rio, desktop entry, or Mars-only assets |
| Mars | 1.13 GiB | Mars, Rio, graphics libraries, Python runtime, and packaged fonts/emoji |
| Yazi + preview tools | 503.2 MiB | Yazi plus Chafa, FFmpeg, ImageMagick, Poppler, resvg, 7-Zip, `fd`, `rg`, `jq`, `fzf`, and `zoxide` |
| Git | 373.8 MiB | Packaged Git CLI and its runtime dependencies |
| Yazelix Helix | 327.6 MiB | Managed Helix, runtime queries, and packaged tree-sitter grammars |
| Ratconfig / `yzx-config` | 124.4 MiB | Compiled configuration UI, validation, persistence, and runtime libraries |
| Carapace | 105.9 MiB | Shell completion engine |
| Nushell | 104.1 MiB | Managed shell executable and runtime libraries |
| Yazelix Zellij | 101.9 MiB | Managed Zellij fork and runtime libraries |
| tokenusage | 75.5 MiB | Codex/Claude usage widget helper |
| zoxide | 60.8 MiB | Directory-jump tool and runtime libraries |
| LazyGit | 59.4 MiB | Terminal Git client and runtime libraries |
| Starship | 58.9 MiB | Managed prompt executable and runtime libraries |
| fzf | 49.5 MiB | Fuzzy finder used by menus and Yazi |
| Yazelix Zellij bar | 43.0 MiB | Top-bar WebAssembly plugin closure |
| Yazelix Screen | 36.7 MiB | Welcome-screen renderer closure |
| Zellij pane orchestrator | 2.1 MiB | Pane-orchestration WebAssembly plugin |
| Zellij popup | 1.9 MiB | Popup WebAssembly plugin |

Nova's own top-level store output is only 46.1 KiB of NAR data. It is primarily
a thin command and desktop-entry join that points at the modules above. The
Yazi Lua plugin inputs are each 17 KiB or less, and the installed cursor
template is 3.8 KiB

Reproduce the total for the current system and lock file with:

```nu
let full = (nix build .#yazelix --no-link --print-out-paths | str trim)
let runtime = (nix build .#runtime --no-link --print-out-paths | str trim)
nix path-info -Sh $full $runtime
nix path-info --json --json-format 1 -S "$full" "$runtime"
```

## Home Manager

```nix
{ inputs, ... }: {
  imports = [ inputs.yazelix.homeManagerModules.default ];
  programs.yazelix.enable = true;
}
```

The optional `programs.yazelix.package` setting overrides the installed package
The module writes no runtime config files unless you configure them

Select the Mars-free package without another module option:

```nix
programs.yazelix.package = inputs.yazelix.packages.${pkgs.system}.runtime;
```

Example:

```nix
programs.yazelix.config = {
  settings = {
    shell.program = "nu";
    editor.command = "nvim";
    welcome.enabled = false;
  };

  starship.text = ''
    format = ":: "
  '';

  helix.languages.source = ./languages.toml;
  yazi.config.source = ./yazi.toml;
};
```

`settings` renders only the declared values to
`~/.config/yazelix/config.toml`, while undeclared values inherit packaged Nova
defaults. Native files are `text` or `source` passthroughs. Store-backed files
show as `home-manager` and read-only in `yzx config`. Save, reset, and file-open
attempts name the exact `programs.yazelix.config.*` option to edit before the
normal Home Manager switch, while permission-only read-only files remain
user-owned

## Updates

Choose one update owner for each installation. Profile installs belong to the
Nix profile. Home Manager and nix-darwin installs belong to the declarative
configuration. Do not mix both update paths for the same installation

Build the exact replacement foundation closure, then let the checked migration
archive both legacy profile namespaces, protect the prior closure, install the
single replacement element, and verify it:

```nu
let closure = (nix build github:FlexNetOS/yazelix#lifeos_foundation_yzx --no-link --print-out-paths | str trim)
let migrator = ($closure | path join "bin" "yazelix_profile_migrate")
^$migrator --closure $closure --flake-ref github:FlexNetOS/yazelix --execute
```

Confirm an entry name with:

```nu
nix profile list --profile /home/flexnetos/.nix-profile
```

For a Home Manager or nix-darwin install, run this from the configuration that
declares the Yazelix input:

```nu
nix flake update yazelix
```

Then run that configuration's normal Home Manager or nix-darwin switch command.
Replace `yazelix` with your chosen input name when it differs. Do not run an
imperative profile migration for a package installed by Home Manager.

Your next launch uses the updated package. Each open Nova session keeps its
current immutable Nix store paths until you close and relaunch it
