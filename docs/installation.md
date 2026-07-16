# Installation and packages

The README covers first launch and the shortest install paths. This guide
describes package variants, platform support, Home Manager ownership, updates,
and measured closure sizes

## Package variants

The default `yzx` package includes Mars, managed Helix, and a Linux desktop
entry. `yazelix-no-helix` keeps Mars and the desktop entry but replaces
`yzx-hx` with a clear unavailable command, so set `editor.command` to an
installed editor such as `nvim`. It neither evaluates managed Helix nor retains
Helix or Steel in its closure. The fixed `runtime` package provides the same
`bin/yzx`, workspace, and config without Mars, Rio, or desktop assets, but it
still includes managed Helix. Its `launch` command explains that Mars is absent,
so use `enter` for the managed workspace. All three package and app outputs
exist for `x86_64-linux`, `aarch64-linux`, `x86_64-darwin`, and
`aarch64-darwin`

Install the external-editor variant with:

```sh
nix profile add --refresh github:luccahuguet/yazelix#yazelix-no-helix
```

Install the Mars-free variant with:

```sh
nix profile add --refresh github:luccahuguet/yazelix#runtime
```

## Capability matrix

| Surface | Linux | `aarch64-darwin` |
| --- | --- | --- |
| Full, no-Helix, and runtime packages | Build- and profile-tested on `x86_64-linux`, with flake outputs also covering `aarch64-linux` | Build-tested on a real GitHub macOS runner |
| Home Manager module | Activation closure build-tested on `x86_64-linux` | Activation closure build-tested on a real GitHub macOS runner |
| `enter` with managed Zellij and Yazi plus the selected editor | Contract-tested and used interactively with managed Helix; host-editor delegation is contract-tested | Packaged, with interactive workflow unverified |
| Full-package `launch` through Mars | Contract-tested and used interactively | Package build-tested, with Mars GUI unverified |
| Host editor delegation | Contract-tested with the selected host editor remaining host-owned | Packaged, with interactive delegation unverified |
| Desktop entry | Full and no-Helix packages, with none in the runtime package | None, as asserted by the macOS package and Home Manager builds |

`x86_64-darwin` remains an exposed, evaluated flake output rather than a
build-tested target. The current label is **build-tested on macOS, with
interactive workflow and Mars GUI unverified**

## Host terminals and SSH

`yzx enter` starts the managed Zellij and Yazi workspace with the selected
editor in the current interactive terminal. It is the SSH/headless route and
needs no Mars, desktop
entry, `DISPLAY`, or `WAYLAND_DISPLAY`

Nova guarantees the managed TUI workflow and configuration, not host clipboard,
image previews, cursor shaders, desktop notifications, or terminal graphics. It
does not provide SSH connectivity or remote file synchronization

## Installed size

The complete Nova package occupies a **2.28 GiB Nix store closure** across 619
store paths on `x86_64-linux`. The external-editor package occupies **2.00
GiB** across 321 paths, saving **281.6 MiB** and 298 paths by excluding managed
Helix and its grammar closure. The Mars-free runtime occupies **1.37 GiB**
across 591 paths, saving **927 MiB**. Its evaluated source-build graph contains
5,664 derivations instead of 8,071, avoiding 2,407 derivations when nothing is
cached. Closure measurements use the 2026-07-16 lock; derivation counts are from
2026-07-12 and indicate potential work, not guaranteed compilations. Closure
size is realized and unpacked, not compressed download size, and an existing
Nix store may already contain shared paths

The module figures below are complete closures for the package roots Nova uses.
They overlap through common libraries and tools, so they do not add up to the
Nova total

| Runtime scope | Closure size | What the measurement includes |
| --- | ---: | --- |
| **Nova (`yzx`)** | **2.28 GiB** | Entire launcher, terminal, workspace, editor, file manager, shell, Git tools, plugins, fonts, and configuration assets |
| **Nova without managed Helix** | **2.00 GiB** | Full Mars workspace and integrations, with editing delegated to a host-installed command |
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

```sh
full=$(nix build .#yazelix --no-link --print-out-paths)
no_helix=$(nix build .#yazelix-no-helix --no-link --print-out-paths)
runtime=$(nix build .#runtime --no-link --print-out-paths)
nix path-info -Sh "$full" "$no_helix" "$runtime"
nix path-info --json --json-format 1 -S "$full" "$no_helix" "$runtime"
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

Select the Helix-free package and an installed editor through the same two
existing owners:

```nix
programs.yazelix = {
  package = inputs.yazelix.packages.${pkgs.system}.yazelix-no-helix;
  config.settings.editor.command = "nvim";
};
```

Example:

```nix
programs.yazelix.config = {
  settings = {
    shell.program = "fish";
    editor.command = "nvim";
    welcome.enabled = false;
  };

  starship.text = ''
    [character]
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

Update a profile install with:

```sh
nix profile upgrade --refresh yazelix
```

The `#yazelix-no-helix` and `#runtime` installs use
`nix profile upgrade --refresh yazelix-no-helix` and
`nix profile upgrade --refresh runtime`, respectively. Run `nix profile list`
when you need to confirm an entry name

For a Home Manager or nix-darwin install, run this from the configuration that
declares the Yazelix input:

```sh
nix flake update yazelix
```

Then run that configuration's normal Home Manager or nix-darwin switch command
Replace `yazelix` with your chosen input name when it differs. Do not run
`nix profile upgrade` for a package installed by Home Manager

Your next launch uses the updated package. Each open Nova session keeps its
current immutable Nix store paths until you close and relaunch it
