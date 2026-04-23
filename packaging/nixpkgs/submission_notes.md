# Nixpkgs Submission Notes

## Scope

This directory prepares the Yazelix nixpkgs submission locally without opening the upstream PR yet.

The draft package here is intentionally narrower than the first-party flake installer:

- direct store-backed `yazelix` package
- canonical entrypoint: `bin/yzx`
- runtime-local `nu` plus the fixed runtime toolset
- no install-time mutation of the user's home directory
- desktop integration stays explicit via `yzx desktop install`
- Linux-only for the first submission

## Upstream Target

When opening the real nixpkgs PR, the package body in [yazelix_package.nix](./yazelix_package.nix) should be translated into:

```text
pkgs/by-name/ya/yazelix/package.nix
```

This local directory keeps the upstream-facing package content and notes together without forcing the repository itself to adopt nixpkgs path conventions.

The package body is intentionally parameterized by `src` in this repo-local draft so it can be smoke-tested directly against the current checkout. In the real nixpkgs PR, that `src` input should be replaced with the normal upstream fetcher stanza for the chosen Yazelix release or revision.

## Reviewer-Relevant Notes

1. The package is not the flake installer.
   It runs directly from the store path and does not create `~/.local/bin/yzx`, manual-install runtime symlinks, shell hooks, or desktop entries during installation.

2. The package owns the bootstrap tools it needs.
   In particular, the direct package includes runtime-local `nu` and the fixed Yazelix runtime toolset; it does not ship or prefer a second `devenv` binary.

3. Desktop integration remains explicit.
   Users install a desktop entry later with `yzx desktop install`. The first nixpkgs submission deliberately avoids package-install desktop side effects.

4. Linux is the supported scope for the first submission.
    Darwin is intentionally not claimed in `meta.platforms` yet. This is narrower than the first-party flake package, which claims all four exported flake systems. The two surfaces are intentionally independent: the first-party flake package must not inherit the narrower submission scope, and the submission draft must not accidentally inherit the broader flake scope.

## Local Validation

Current local validation for the draft submission path:

```bash
cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- validate-nixpkgs-package
cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_validator -- validate-nixpkgs-submission
```

The first validator defends the direct repo-local package surface. The second validator defends the upstream-facing draft package body in this directory.

## Remaining Mechanical Work Before PR

1. Copy the draft package body into a nixpkgs checkout at `pkgs/by-name/ya/yazelix/package.nix`.
2. Replace the local `src` argument with the final `fetchFromGitHub` release/revision stanza and hash.
3. Add or confirm the maintainer entry expected for the upstream PR.
4. Run the normal nixpkgs formatting and package build checks in that checkout.
5. Open the PR with these notes adapted into the PR body or reviewer context.
