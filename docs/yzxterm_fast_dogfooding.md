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
child fork's fast Cargo profile and skipping package checks. Further work should
be evaluated separately:

- Linkers: `mold` is designed as a faster Unix linker for large final links, so
  it is a plausible release-link experiment if the LTO link remains the dominant
  cost. See <https://github.com/rui314/mold>.
- Compiler caching: `sccache` supports Rust through `RUSTC_WRAPPER` and local or
  remote caches. It may help repeated Cargo builds, but it does not remove the
  final link step. See <https://github.com/mozilla/sccache>.
- Test execution: `cargo-nextest` can run Rust tests faster than `cargo test`
  and may help move package checks into a faster explicit validation lane. See
  <https://nexte.st/>.
- Binary cache: Cachix can push flake runtime closures so a published terminal
  revision is substituted instead of rebuilt locally. See
  <https://docs.cachix.org/pushing>.
