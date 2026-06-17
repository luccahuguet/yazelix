# Mars Fast Dogfooding

This is a maintainer workflow for local Mars iteration. It keeps the normal
release/runtime path intact and adds an explicit fast path for terminal fork
dogfooding.

## Slow Path

The release path remains:

```sh
nix build .#runtime_yzxterm --no-link --no-write-lock-file
home-manager switch --flake .#lucca@loqness
```

Use it for final runtime validation and shareable releases. When the
`marsTerminal` input changes, this path consumes the checked
`mars` child package. The child package builds Mars with the release Cargo
profile, including LTO and one codegen unit, then runs the package test phase.
In the observed June 3, 2026 update, the final release-LTO link and the
separate test graph rebuild were the expensive steps.

## Fast Path

For local dogfooding of terminal-only changes, use the explicit fast outputs:

```sh
nix build .#runtime_yzxterm_fast --no-link --no-write-lock-file
nix run .#yzxterm_fast -- launch
```

`runtime_yzxterm_fast` and `yzxterm_fast` use the `mars-fast` child package.
That child package keeps the same wrapper/config shape as the regular terminal
package, but its unwrapped Mars build uses the Cargo `fast` profile and skips
package checks. It is not release evidence.

The public cache publish workflow builds `runtime_yzxterm_fast` on
`x86_64-linux`. After that workflow publishes a commit, Nix can substitute the
fast runtime closure and the `mars-fast` child package from the Yazelix Cachix
cache instead of rebuilding the WGPU terminal graph locally.
Before a commit is published, or when using unpublished local child-repo
changes, expect the fast runtime to build locally.

For Home Manager dogfooding, keep the yzxterm runtime settings but override only
the terminal child package. The example assumes a direct `marsTerminal` flake
input pointing at `github:luccahuguet/mars`:

```nix
{
  programs.yazelix = {
    terminal = "yzxterm";
    yzxterm_profile = "shaders";
    yzxterm_package = inputs.marsTerminal.packages.${pkgs.stdenv.hostPlatform.system}.mars-fast;
  };
}
```

Remove the `programs.yazelix.yzxterm_package` override before final
release/runtime validation. The default Home Manager module path still uses the
checked `mars` package.

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
`"yzxterm_terminal_package": "mars-fast"`.

## Further Speed Work

The current fast path avoids the two observed Nix package costs by using the
child fork's fast Cargo profile and skipping package checks. The second-stage
main-repo improvement is binary-cache publication for `runtime_yzxterm_fast`.
Cachix documents flake runtime closure publishing through `nix build --no-link
--print-out-paths ... | cachix push`, which matches the existing Yazelix publish
workflow. This keeps release validation unchanged while letting published fast
runtime commits substitute locally. See <https://docs.cachix.org/pushing>.

Post-cache dry-run evidence from June 6, 2026:

- main repo dirty `nix build .#runtime_yzxterm_fast --dry-run` reported only 3
  derivations to build and 55 fetched paths; the `mars-fast` and
  `mars-fast-unwrapped` child paths were fetched/substituted
- child repo `nix build .#mars-fast --dry-run` from an unpublished
  local checkout still reported 60 derivations to build and 141 fetched paths

That means the normal main-repo dogfooding bottleneck is already handled by the
cache lane once CI has published the commit. The remaining expensive case is
unpublished child source iteration, and that belongs to `mars`, not to main
Yazelix runtime packaging.

Use these paths according to the work being done:

- Local terminal source iteration: use the child repo's Cargo/dev shell first;
  do not start with a main-repo runtime build.
- Main-repo yzxterm dogfooding: use `runtime_yzxterm_fast` and `yzxterm_fast`;
  after CI publishes the commit, this should substitute from Cachix on
  `x86_64-linux`.
- Home Manager yzxterm dogfooding: set only `programs.yazelix.yzxterm_package`
  to the child `mars-fast` package.
- Release validation: use `runtime_yzxterm` and the default Home Manager path;
  this consumes the checked release child package.

Other build-speed approaches stay separate from the main runtime cache lane:

- Compiler caching: `sccache` supports Rust through Cargo's `rustc-wrapper` /
  `RUSTC_WRAPPER` setting and local or remote caches. It is the best first child
  experiment for repeated local Cargo builds, but it should be opt-in and
  child-dev-only because it introduces mutable cache state. See
  <https://doc.rust-lang.org/cargo/reference/config.html#buildrustc-wrapper>
  and <https://github.com/mozilla/sccache>.
- Linkers: `mold` is designed as a faster Unix linker for large final links and
  documents a Rust/Linux `target.'cfg(target_os = "linux")'` configuration. It
  is plausible only if timing proves final link dominates the child fast build.
  Keep it Linux-gated and child-owned. See <https://github.com/rui314/mold>.
- Linkers: LLVM `lld` is a broader portable linker candidate and Cargo supports
  per-target or per-`cfg` linker configuration. Prefer `lld` over `mold` only if
  the child needs a less Linux-specific experiment. See <https://lld.llvm.org/>
  and <https://doc.rust-lang.org/cargo/reference/config.html#targettriplelinker>.
- Test execution: `cargo-nextest` can help the checked child lane, but it does
  not help `mars-fast` because that package already sets
  `doCheck = false`. Nixpkgs also documents that `checkType = "debug"` compiles
  once for build and again for checks, which matches the release-path cost we
  are intentionally avoiding in fast packages. See
  <https://nixos.org/manual/nixpkgs/unstable/#rust>.

Do not wire `sccache`, `mold`, or `lld` into the main `runtime_yzxterm_fast`
package by default. A child experiment can expose a separate explicitly named
package or dev shell after it records before/after timings for the child build
phase it claims to improve.
