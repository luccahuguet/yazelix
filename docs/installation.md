# Installation and packages

The README covers first launch and the shortest install paths. This guide
describes package variants, platform support, Home Manager ownership, updates,
and measured closure sizes

## Release channels

Use `github:luccahuguet/yazelix/stable` for normal installs. Maintainers promote
an exact checked and dogfooded `main` revision at most once per week, with
earlier promotions reserved for urgent fixes. A Nix lock file keeps that
revision until its owner requests an update.

Use `github:luccahuguet/yazelix/main#yazelix-main` for the development channel
and `github:luccahuguet/yazelix/edge#yazelix-edge` for experimental dogfooding.
Immutable `nova-v*` tags identify exact releases.

The source reference and package output are both explicit because an immutable
Nix derivation cannot infer which Git branch selected its revision. On Linux,
the three outputs install `Yazelix Nova (Stable)`, `Yazelix Nova (Main)`, and
`Yazelix Nova (Edge)` entries with distinct desktop file IDs. Run the line for
the channel you want, or run all three in order to expose every launcher:

```sh
nix profile add --refresh github:luccahuguet/yazelix/stable
nix profile add --refresh github:luccahuguet/yazelix/main#yazelix-main --priority 6
nix profile add --refresh github:luccahuguet/yazelix/edge#yazelix-edge --priority 7
```

The priorities resolve only shared profile paths. Each desktop entry keeps an
absolute package-owned launch command, so Stable, Main, and Edge still start
their exact immutable packages. Their running top bars identify that package as
`NOVA β4 STABLE`, `NOVA β5 MAIN`, or `NOVA β5 EDGE`.

## Package variants

Package names follow `yazelix[-no-mars][-no-helix][-no-yazi]`:

| Package | Mars | Managed Helix | Managed Yazi | Linux desktop channel |
| --- | --- | --- | --- | --- |
| `yazelix` | Yes | Yes | Yes | Stable |
| `yazelix-no-helix` | Yes | No | Yes | Stable |
| `yazelix-no-yazi` | Yes | Yes | No | Stable |
| `yazelix-no-helix-no-yazi` | Yes | No | No | Stable |
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
eight capability variants exist for `x86_64-linux`, `aarch64-linux`,
`x86_64-darwin`, and `aarch64-darwin`. The full `yazelix-main` and
`yazelix-edge` channel outputs exist on the same systems and differ only in
their channel-qualified desktop and runtime identities

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
observational interactive beta use and no known regression; the individual
workflow checklist and Mars GUI remain unverified**

## Host terminals and SSH

`yzx enter` starts the managed Zellij and Yazi workspace with the selected
editor in the current interactive terminal. It is the SSH/headless route and
needs no Mars, desktop
entry, `DISPLAY`, or `WAYLAND_DISPLAY`

Nova guarantees the managed TUI workflow and configuration, not host clipboard,
image previews, cursor shaders, desktop notifications, or terminal graphics. It
does not provide SSH connectivity or remote file synchronization

## Installed size

The eight package closures measured on `x86_64-linux` with the 2026-07-26 lock
are:

| Package | Closure | Store paths |
| --- | ---: | ---: |
| `yazelix` | 2.28 GiB | 643 |
| `yazelix-no-helix` | 2.01 GiB | 345 |
| `yazelix-no-yazi` | 1.91 GiB | 524 |
| `yazelix-no-helix-no-yazi` | 1.63 GiB | 226 |
| `yazelix-no-mars` | 1.38 GiB | 615 |
| `yazelix-no-mars-no-helix` | 1.10 GiB | 317 |
| `yazelix-no-mars-no-yazi` | 0.98 GiB | 484 |
| `yazelix-no-mars-no-helix-no-yazi` | 0.70 GiB | 186 |

Removing managed Yazi saves 384.8 MiB when Mars is present and 406.4 MiB when
Mars is absent because some Yazi dependencies are already shared with Mars.
The Mars-free evaluated source-build graph contains 5,732 derivations instead
of 8,115, avoiding 2,383 derivations when nothing is cached. Derivation counts
are from 2026-07-26 and indicate potential work, not guaranteed compilations.
Closure size is realized and unpacked, not compressed download size, and an
existing Nix store may already contain shared paths

The component figures below are complete closures for the package roots Nova uses.
They overlap through common libraries and tools, so they do not add up to the
Nova total

| Runtime scope | Closure size | What the measurement includes |
| --- | ---: | --- |
| Mars | 1.13 GiB | Mars, Rio, graphics libraries, Python runtime, and packaged fonts/emoji |
| Yazi + preview tools | 503.2 MiB | Yazi plus Chafa, FFmpeg, ImageMagick, Poppler, resvg, 7-Zip, `fd`, `rg`, `jq`, `fzf`, and `zoxide` |
| Git | 373.8 MiB | Packaged Git CLI and its runtime dependencies |
| Yazelix Helix | 327.6 MiB | Managed Helix, runtime queries, and packaged tree-sitter grammars |
| Ratconfig / `yzx-config` | 108.9 MiB | Compiled configuration UI, validation, persistence, and runtime libraries |
| Carapace | 105.9 MiB | Shell completion engine |
| Nushell | 104.1 MiB | Managed shell executable and runtime libraries |
| Yazelix Zellij | 101.9 MiB | Managed Zellij fork and runtime libraries |
| tokenusage | 75.5 MiB | Codex/Claude usage widget helper |
| zoxide | 60.8 MiB | Directory-jump tool and runtime libraries |
| LazyGit | 59.4 MiB | Terminal Git client and runtime libraries |
| Starship | 58.9 MiB | Managed prompt executable and runtime libraries |
| fzf | 49.5 MiB | Fuzzy finder used by menus and Yazi |
| Yazelix Zellij bar | 43.1 MiB | Top-bar WebAssembly plugin closure |
| Yazelix Screen | 47.9 MiB | Welcome-screen renderer and separately packaged aquarium closure |
| Zellij pane orchestrator | 2.1 MiB | Pane-orchestration WebAssembly plugin |
| Zellij popup | 1.9 MiB | Popup WebAssembly plugin |

Nova's own top-level store output is only 48.9 KiB of NAR data. It is primarily
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
  package_path=$(nix build ".#$package" --no-link --print-out-paths)
  closure_size=$(nix path-info -Sh "$package_path" | awk '{print $2}')
  store_paths=$(nix path-info -r "$package_path" | wc -l)
  printf '%s\t%s\t%s paths\n' "$package" "$closure_size" "$store_paths"
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

Main and Edge inputs must select their matching package output to retain the
channel-qualified launcher:

```nix
programs.yazelix.package =
  inputs.yazelix.packages.${pkgs.system}.yazelix-main;
```

Use `yazelix-edge` in the same declaration for an `edge` input.

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
    appearance.mode = "light";
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

The update follows the input's configured `stable`, `main`, `edge`, or tag
reference. Your next launch uses the updated package. Each open Nova session
keeps its current immutable Nix store paths until you close and relaunch it
