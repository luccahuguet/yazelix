# Yazelix Terminal Fast Dogfooding

This is a maintainer workflow for local Yazelix Terminal iteration. It keeps the
normal release/runtime path intact and adds an explicit fast path for terminal
fork dogfooding.

## Slow Path

The release path remains:

```sh
nix build .#runtime_yzxterm --no-link --no-write-lock-file
home-manager switch --flake .#lucca@loqness
```

Use it for final runtime validation and shareable releases. When the
`yazelixTerminal` input changes, this path consumes the checked
`yazelix-terminal` child package. The child package builds `rioterm` with the
release Cargo profile, including LTO and one codegen unit, then runs the package
test phase. In the observed June 3, 2026 update, the final release-LTO link and
the separate test graph rebuild were the expensive steps.

## Fast Path

For local dogfooding of terminal-only changes, use the explicit fast outputs:

```sh
nix build .#runtime_yzxterm_fast --no-link --no-write-lock-file
nix run .#yzxterm_fast -- launch
```

`runtime_yzxterm_fast` and `yzxterm_fast` use the `yazelix-terminal-fast` child
package. That child package keeps the same wrapper/config shape as the regular
terminal package, but its unwrapped Rio build uses the Cargo `fast` profile and
skips package checks. It is not release evidence.

The public cache publish workflow builds `runtime_yzxterm_fast` on
`x86_64-linux`. After that workflow publishes a commit, Nix can substitute the
fast runtime closure and the `yazelix-terminal-fast` child package from the
Yazelix Cachix cache instead of rebuilding the WGPU terminal graph locally.
Before a commit is published, or when using unpublished local child-repo
changes, expect the fast runtime to build locally.

For Home Manager dogfooding, keep the yzxterm runtime settings but override only
the terminal child package. The example assumes a direct `yazelixTerminal` flake
input pointing at `github:luccahuguet/yazelix-terminal`:

```nix
{
  programs.yazelix = {
    terminal = "yzxterm";
    yzxterm_profile = "shaders";
    yzxterm_package = inputs.yazelixTerminal.packages.${pkgs.stdenv.hostPlatform.system}.yazelix-terminal-fast;
  };
}
```

Remove the `programs.yazelix.yzxterm_package` override before final
release/runtime validation. The default Home Manager module path still uses the
checked `yazelix-terminal` package.

## Cheap Validation

Before building, use eval-only checks to confirm the fast outputs are present:

```sh
system="$(nix eval --raw --impure --expr builtins.currentSystem)"
nix eval --raw ".#packages.${system}.runtime_yzxterm_fast.name"
nix eval --raw ".#packages.${system}.yzxterm_fast.name"
```

After the rebuild-speed gate is satisfied and a build is acceptable, validate the
fast runtime with:

```sh
nix build .#runtime_yzxterm_fast --no-link --no-write-lock-file
nix build .#yzxterm_fast --no-link --no-write-lock-file
nix run .#yzxterm_fast -- status --versions
YAZELIX_TERMINAL_PROFILE=shaders nix run .#yzxterm_fast -- launch
```

The `yzxterm_fast` package bypasses the stable-profile redirect that normally
protects stale `/nix/store/.../bin/yzx` invocations, so `nix run
.#yzxterm_fast -- status --versions` should report a `yazelix-yzxterm-fast`
runtime dir. When the fast package is installed through the Home Manager package
override, plain `yzx status --versions` should show the fast runtime. In both
cases, `runtime_identity.json` carries `"package_profile": "yzxterm-fast"` and
`"yzxterm_terminal_package": "yazelix-terminal-fast"`.

## Further Speed Work

The current fast path avoids the two observed Nix package costs by using the
child fork's fast Cargo profile and skipping package checks. The second-stage
main-repo improvement is binary-cache publication for `runtime_yzxterm_fast`.
Cachix documents flake runtime closure publishing through `nix build --no-link
--print-out-paths ... | cachix push`, which matches the existing Yazelix publish
workflow. This keeps release validation unchanged while letting published fast
runtime commits substitute locally. See <https://docs.cachix.org/pushing>.

Use these paths according to the work being done:

- Local terminal source iteration: use the child repo's Cargo/dev shell first;
  do not start with a main-repo runtime build.
- Main-repo yzxterm dogfooding: use `runtime_yzxterm_fast` and `yzxterm_fast`;
  after CI publishes the commit, this should substitute from Cachix on
  `x86_64-linux`.
- Home Manager yzxterm dogfooding: set only `programs.yazelix.yzxterm_package`
  to the child `yazelix-terminal-fast` package.
- Release validation: use `runtime_yzxterm` and the default Home Manager path;
  this consumes the checked release child package.

Other build-speed approaches stay separate from the main runtime cache lane:

- Linkers: `mold` is designed as a faster Unix linker for large final links, so
  it is a plausible child-package experiment if the terminal link remains the
  dominant cost after cache publication. See <https://github.com/rui314/mold>.
- Compiler caching: `sccache` supports Rust through `RUSTC_WRAPPER` and local or
  remote caches. It may help repeated Cargo builds, but wiring mutable compiler
  cache state into main-repo Nix runtime packaging would add more ownership than
  this bead needs. See <https://github.com/mozilla/sccache>.
- Test execution: `cargo-nextest` can run Rust tests faster than `cargo test`
  and may help the child repo's explicit checked lane. It does not help the
  existing fast package because that package already skips checks. See
  <https://nexte.st/>.
