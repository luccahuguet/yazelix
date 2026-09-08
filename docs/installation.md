# Installation and packages

The README covers first launch and the shortest install paths. This guide
describes package variants, platform support, Home Manager ownership, and
updates

## Release channels

Use `github:Yazelix/nova/stable` for normal installs. Maintainers promote
an exact checked and dogfooded `main` revision at most once per week, with
earlier promotions reserved for urgent fixes. A Nix lock file keeps that
revision until its owner requests an update.

Use `github:Yazelix/nova/main#yazelix-main` for the development channel
and `github:Yazelix/nova/edge#yazelix-edge` for experimental dogfooding.
Immutable `nova-v*` tags identify exact releases.

The source reference and package output are both explicit because an immutable
Nix derivation cannot infer which Git branch selected its revision. On Linux,
the three outputs install `Yazelix Nova (Stable)`, `Yazelix Nova (Main)`, and
`Yazelix Nova (Edge)` entries with distinct desktop file IDs. Run the line for
the channel you want, or run all three in order to expose every launcher:

```sh
nix profile add --refresh github:Yazelix/nova/stable
nix profile add --refresh github:Yazelix/nova/main#yazelix-main --priority 6
nix profile add --refresh github:Yazelix/nova/edge#yazelix-edge --priority 7
```

The priorities resolve only shared profile paths. Each desktop entry keeps an
absolute package-owned launch command, so Stable, Main, and Edge still start
their exact immutable packages. Their running top bars identify that package as
`NOVA 1.2 STABLE`, `NOVA 1.2 MAIN`, or `NOVA 1.2 EDGE`, depending on the version installed.

## Package variants

Package names follow `yazelix[-no-rio][-no-helix][-no-yazi]`:

| Package | Rio | Managed Helix | Managed Yazi | Linux desktop channel |
| --- | --- | --- | --- | --- |
| `yazelix` | Yes | Yes | Yes | Stable |
| `yazelix-no-helix` | Yes | No | Yes | Stable |
| `yazelix-no-yazi` | Yes | Yes | No | Stable |
| `yazelix-no-helix-no-yazi` | Yes | No | No | Stable |
| `yazelix-no-rio` | No | Yes | Yes | None |
| `yazelix-no-rio-no-helix` | No | No | Yes | None |
| `yazelix-no-rio-no-yazi` | No | Yes | No | None |
| `yazelix-no-rio-no-helix-no-yazi` | No | No | No | None |

Rio-free packages omit Rio and its terminal-only closure, native config,
icon, desktop entry, and Ratconfig tab. Use `yzx enter` from the host terminal;
`yzx launch` exits with that instruction.
Helix-free packages replace `yzx-hx` with a clear unavailable command, so set
`editor.command` to an installed editor such as `nvim`. They neither evaluate
managed Helix nor retain Helix, Steel, Forest, its notify/glyph dependencies,
or the packaged grammar closure. Yazi-free
packages retain the managed launcher, configuration, Radar sidebar, popup, opener,
and reveal integration but require host-provided `yazi` and `ya` commands with
matching versions. A pair that differs from Nova's tested version warns and
continues. The host installation owns optional Yazi preview dependencies.
All eight capability variants exist for `x86_64-linux`, `aarch64-linux`,
`x86_64-darwin`, and `aarch64-darwin`. The full `yazelix-main` and
`yazelix-edge` channel outputs exist on the same systems and differ only in
their channel-qualified desktop and runtime identities

Install the external-editor variant with:

```sh
nix profile add --refresh github:Yazelix/nova/stable#yazelix-no-helix
```

Install the host-Yazi variant after providing `yazi` and `ya` on the launch
PATH:

```sh
nix profile add --refresh github:Yazelix/nova/stable#yazelix-no-yazi
```

The modifiers compose mechanically:

```sh
nix profile add --refresh github:Yazelix/nova/stable#yazelix-no-helix-no-yazi
```

Install the terminal-free full-TUI variant with:

```sh
nix profile add --refresh github:Yazelix/nova/stable#yazelix-no-rio
```

Add `-no-helix`, `-no-yazi`, or both to compose the reduced variants.

## Capability matrix

| Surface | Linux | `aarch64-darwin` |
| --- | --- | --- |
| All eight package variants | Build- and profile-tested on `x86_64-linux`, with flake outputs also covering `aarch64-linux` | Exposed; Rio packages are build-tested on a real GitHub macOS runner, while Rio-free variants await hosted evidence |
| Home Manager module | Activation closure build-tested on `x86_64-linux` | Activation closure build-tested on a real GitHub macOS runner |
| `enter` with managed Zellij and Yazi plus the selected editor | Contract-tested and used interactively with managed Helix; host-editor delegation is contract-tested | Packaged, with interactive workflow unverified |
| Full-package `launch` through Rio | Contract-tested and used interactively | Package build-tested, with Rio GUI unverified |
| Host editor delegation | Contract-tested with the selected host editor remaining host-owned | Packaged, with interactive delegation unverified |
| Desktop entry | Packages that include Rio | None, as asserted by the macOS package and Home Manager builds |

`x86_64-darwin` remains an exposed, evaluated flake output rather than a
build-tested target. The current label is **build-tested on macOS, with
observational interactive beta use and no known regression; the individual
workflow checklist and Rio GUI remain unverified**

## Host terminals and SSH

`yzx enter` starts the managed Zellij and Yazi workspace with the selected
editor in the current interactive terminal. It is the SSH/headless route and
does not start Rio or require a desktop
entry, `DISPLAY`, or `WAYLAND_DISPLAY`

Nova guarantees the managed TUI workflow and configuration, not host clipboard,
image previews, cursor shaders, desktop notifications, or terminal graphics. It
does not provide SSH connectivity or remote file synchronization

## Home Manager

Declare the stable input in the consumer flake:

```nix
inputs.yazelix.url = "github:Yazelix/nova/stable";
```

This preserves Nova's locked dependencies so its packages can match published
cache artifacts. Your host and Home Manager keep their own nixpkgs choices.
Configure the [binary cache](#binary-cache) before the first Nova build.

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

The two optional managed components compose through package selection without
additional Home Manager options:

```nix
{
  home.packages = [ pkgs.neovim pkgs.yazi ];
  programs.yazelix = {
    package = inputs.yazelix.packages.${pkgs.system}.yazelix-no-helix-no-yazi;
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
  rio.source = ./rio.toml;
  yazi.config.source = ./yazi.toml;
  yazi.starship.source = ./yazi-starship.toml;
};
```

`rio.source` replaces Nova's complete native Rio config. If that file uses
`adaptive-theme`, install its referenced theme files under
`~/.config/yazelix/rio/themes/` through Home Manager as well, for example with
`xdg.configFile."yazelix/rio/themes/<name>.toml".source`.
Because the store-backed file is read-only, Nova cannot project its reserved
`force-theme` field. Rio receives the root appearance at launch instead, and
all managed components keep that captured mode until the next session.

`settings` renders only the declared values to
`~/.config/yazelix/config.toml`, while undeclared values inherit packaged Nova
defaults. Native files are `text` or `source` passthroughs. Store-backed files
show as `home-manager` and read-only in `yzx config`. Save, reset, and file-open
attempts name the exact `programs.yazelix.config.*` option to edit before the
normal Home Manager switch, while permission-only read-only files remain
user-owned

### Binary cache

Nova's Home Manager module installs the selected package and configuration;
the host's Nix configuration owns cache access. Cache reuse requires both a
matching published store path and a configured, trusted cache.

On NixOS or nix-darwin, add these settings to the system configuration:

```nix
nix.settings = {
  extra-substituters = [ "https://yazelix.cachix.org" ];
  extra-trusted-public-keys = [
    "yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM="
  ];
};
```

Apply that system configuration before building Nova through Home Manager.
For other Nix installations, add the equivalent settings to the Nix
configuration that controls builds (`/etc/nix/nix.conf` for daemon installs):

```ini
extra-substituters = https://yazelix.cachix.org
extra-trusted-public-keys = yazelix.cachix.org-1:ZgxIjQvaP0VTWL8Racx27mpUNzDJ97xC2y7QWYjmGNM=
```

The `extra-` settings retain existing caches and keys. Restart the Nix daemon
after editing its configuration. See [Nix's cache configuration guide](https://nix.dev/guides/recipes/add-binary-cache.html)
for configuration ownership and trust details.

To inspect the planned downloads and builds without compiling, run this from
your consumer flake, replacing `username` with your Home Manager output name:

```sh
nix build --dry-run '.#homeConfigurations.username.activationPackage'
```

If major Nova components would build, check the selected Nova revision and
dependency overrides, cache settings, and warnings about ignored substituters
or signatures. A cache miss does not by itself show that the cache was bypassed;
the requested output may not have been published. See [Cachix troubleshooting](https://docs.cachix.org/faq#why-is-nix-not-picking-up-on-any-of-the-pre-built-artifacts).

### Optional dependency overrides

To deliberately build Nova using a nixpkgs input declared by your consumer
flake, add a `follows` declaration:

```nix
inputs.yazelix.inputs.nixpkgs.follows = "nixpkgs-unstable";
```

Use your actual input name, such as `nixpkgs` or `nixpkgs-unstable`; a local
variable such as `pkgs-unstable = import nixpkgs-unstable { ... };` is not an
input name. Renaming an input alone does not change its packages. Two inputs
with the same branch URL can have different locked commits; `follows` selects
the referenced input's locked revision. See [Nix's input reference](https://nix.dev/manual/nix/2.35/command-ref/new-cli/nix3-flake.html#flake-inputs).

Nova still pins its own Zellij fork, but changing nixpkgs can change build
dependencies and require source builds. Overriding child inputs such as
`fenix` or `rust-overlay` can also change toolchains and cached output paths,
even when nixpkgs stays fixed. Leave Nova's internal inputs at their locked
revisions unless you intend to change those dependencies.

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
