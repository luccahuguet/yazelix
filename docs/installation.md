# Installation and packages

The README covers first launch and the shortest install paths. This guide
describes package variants, platform support, Home Manager ownership, updates,
and measured closure sizes

## Release channels

Use `github:luccahuguet/yazelix/stable` for normal installs. Maintainers promote
an exact checked and dogfooded `main` revision at most once per week, with
earlier promotions reserved for urgent fixes. A Nix lock file keeps that
revision until its owner requests an update.

Use `github:luccahuguet/yazelix/main` for the development channel. Immutable
`nova-v*` tags identify exact releases.

## Package variants

Package names follow `yazelix[-no-mars][-no-helix][-no-yazi]`:

| Package | Mars | Managed Helix | Managed Yazi | Linux desktop entry |
| --- | --- | --- | --- | --- |
| `yazelix` | Yes | Yes | Yes | Yes |
| `yazelix-no-helix` | Yes | No | Yes | Yes |
| `yazelix-no-yazi` | Yes | Yes | No | Yes |
| `yazelix-no-helix-no-yazi` | Yes | No | No | Yes |
| `yazelix-no-mars` | No | Yes | Yes | No |
| `yazelix-no-mars-no-helix` | No | No | Yes | No |
| `yazelix-no-mars-no-yazi` | No | Yes | No | No |
| `yazelix-no-mars-no-helix-no-yazi` | No | No | No | No |

Helix-free packages replace `yzx-hx` with a clear unavailable command, so set
`editor.command` to an installed editor such as `nvim`. They neither evaluate
managed Helix nor retain Helix, Steel, or the packaged grammar closure.
Mars-free packages keep `bin/yzx`, the managed workspace, and configuration
without Mars, Rio, or desktop assets. Their `launch` command explains that Mars
is absent, so use `enter` in the current terminal or over SSH. Yazi-free
packages retain the managed launcher, configuration, sidebar, popup, opener,
and reveal integration but require host-provided `yazi` and `ya` commands with
matching versions. A pair that differs from Nova's tested version warns and
continues. The host installation owns optional Yazi preview dependencies. All
eight package and app outputs exist for `x86_64-linux`, `aarch64-linux`,
`x86_64-darwin`, and `aarch64-darwin`

Install the external-editor variant with:

```sh
nix profile add --refresh github:luccahuguet/yazelix/stable#yazelix-no-helix
```

Install the Mars-free variant with:

```sh
nix profile add --refresh github:luccahuguet/yazelix/stable#yazelix-no-mars
```

Install the host-Yazi variant after providing `yazi` and `ya` on the launch
PATH:

```sh
nix profile add --refresh github:luccahuguet/yazelix/stable#yazelix-no-yazi
```

The modifiers compose mechanically:

```sh
nix profile add --refresh github:luccahuguet/yazelix/stable#yazelix-no-mars-no-helix-no-yazi
```

## Capability matrix

| Surface | Linux | `aarch64-darwin` |
| --- | --- | --- |
| All eight package variants | Build- and profile-tested on `x86_64-linux`, with flake outputs also covering `aarch64-linux` | Build-tested on a real GitHub macOS runner |
| Home Manager module | Activation closure build-tested on `x86_64-linux` | Activation closure build-tested on a real GitHub macOS runner |
| `enter` with managed Zellij and Yazi plus the selected editor | Contract-tested and used interactively with managed Helix; host-editor delegation is contract-tested | Packaged, with interactive workflow unverified |
| Full-package `launch` through Mars | Contract-tested and used interactively | Package build-tested, with Mars GUI unverified |
| Host editor delegation | Contract-tested with the selected host editor remaining host-owned | Packaged, with interactive delegation unverified |
| Desktop entry | Every Mars package, with none in any `no-mars` package | None, as asserted by the macOS package and Home Manager builds |

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

The eight package closures measured on `x86_64-linux` with the 2026-07-21 lock
are:

| Package | Closure | Store paths |
| --- | ---: | ---: |
| `yazelix` | 2.28 GiB | 619 |
| `yazelix-no-helix` | 2.00 GiB | 321 |
| `yazelix-no-yazi` | 1.90 GiB | 500 |
| `yazelix-no-helix-no-yazi` | 1.63 GiB | 202 |
| `yazelix-no-mars` | 1.37 GiB | 591 |
| `yazelix-no-mars-no-helix` | 1.10 GiB | 293 |
| `yazelix-no-mars-no-yazi` | 0.98 GiB | 460 |
| `yazelix-no-mars-no-helix-no-yazi` | 0.70 GiB | 162 |

Removing managed Yazi saves 384.8 MiB when Mars is present and 406.4 MiB when
Mars is absent because some Yazi dependencies are already shared with Mars.
The Mars-free evaluated source-build graph contains 5,664 derivations instead
of 8,071, avoiding 2,407 derivations when nothing is cached. Derivation counts
are from 2026-07-12 and indicate potential work, not guaranteed compilations.
Closure size is realized and unpacked, not compressed download size, and an
existing Nix store may already contain shared paths

The module figures below are complete closures for the package roots Nova uses.
They overlap through common libraries and tools, so they do not add up to the
Nova total

| Runtime scope | Closure size | What the measurement includes |
| --- | ---: | --- |
| **Nova (`yzx`)** | **2.28 GiB** | Entire launcher, terminal, workspace, editor, file manager, shell, Git tools, plugins, fonts, and configuration assets |
| **Nova without managed Helix** | **2.00 GiB** | Full Mars workspace and integrations, with editing delegated to a host-installed command |
| **Nova without Mars** | **1.37 GiB** | Same command, workspace, tools, config, and cursor schema without Mars, Rio, desktop entry, or Mars-only assets |
| **Nova without Mars or managed Helix** | **1.10 GiB** | Managed TUI workspace and host-editor delegation without Mars, Rio, or managed Helix |
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
| Yazelix Screen | 47.9 MiB | Welcome-screen renderer and separately packaged aquarium closure |
| Zellij pane orchestrator | 2.1 MiB | Pane-orchestration WebAssembly plugin |
| Zellij popup | 1.9 MiB | Popup WebAssembly plugin |

Nova's own top-level store output is only 39.1 KiB of NAR data. It is primarily
a thin command and desktop-entry join that points at the modules above. The
Yazi Lua plugin inputs are each 17 KiB or less, and the installed cursor
template is 3.8 KiB

Reproduce the total for the current system and lock file with:

```sh
for package in \
  yazelix \
  yazelix-no-helix \
  yazelix-no-yazi \
  yazelix-no-helix-no-yazi \
  yazelix-no-mars \
  yazelix-no-mars-no-helix \
  yazelix-no-mars-no-yazi \
  yazelix-no-mars-no-helix-no-yazi; do
  path=$(nix build ".#$package" --no-link --print-out-paths)
  nix path-info -Sh "$path"
done
```

## Home Manager

Declare the stable input in the consumer flake:

```nix
inputs.yazelix = {
  url = "github:luccahuguet/yazelix/stable";
  inputs.nixpkgs.follows = "nixpkgs";
};
```

Import the module from that input:

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
programs.yazelix.package =
  inputs.yazelix.packages.${pkgs.system}.yazelix-no-mars;
```

Select the Helix-free package and an installed editor through the same two
existing owners:

```nix
programs.yazelix = {
  package = inputs.yazelix.packages.${pkgs.system}.yazelix-no-helix;
  config.settings.editor.command = "nvim";
};
```

Select host-owned Yazi through the same package owner and provide both `yazi`
and `ya` through the Home Manager profile:

```nix
{
  home.packages = [ pkgs.yazi ];
  programs.yazelix.package =
    inputs.yazelix.packages.${pkgs.system}.yazelix-no-yazi;
}
```

All three omissions compose through package selection without additional Home
Manager options:

```nix
{
  home.packages = [ pkgs.neovim pkgs.yazi ];
  programs.yazelix = {
    package = inputs.yazelix.packages.${pkgs.system}.yazelix-no-mars-no-helix-no-yazi;
    config.settings.editor.command = "nvim";
  };
}
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
  yazi.starship.source = ./yazi-starship.toml;
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

Pass the installed package name to `nix profile upgrade --refresh`. Run
`nix profile list` when you need to confirm an entry name

For a Home Manager or nix-darwin install, run this from the configuration that
declares the Yazelix input:

```sh
nix flake update yazelix
```

Then run that configuration's normal Home Manager or nix-darwin switch command
Replace `yazelix` with your chosen input name when it differs. Do not run
`nix profile upgrade` for a package installed by Home Manager

The update follows the input's configured `stable`, `main`, or tag reference.
Your next launch uses the updated package. Each open Nova session keeps its
current immutable Nix store paths until you close and relaunch it
