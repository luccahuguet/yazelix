# Zellij plugin naming

Yazelix-owned Zellij plugin packages use this public name shape:

```text
yazelix-zellij-<plugin-name>
```

Use this name shape when the artifact is meant to be installed or understood as a Zellij plugin package, even if the package also ships helper binaries, presets, examples, or docs.

## Package and repository names

Repository names and public package names use kebab case:

```text
yazelix-zellij-bar
yazelix-zellij-pane-orchestrator
```

The middle `zellij` segment is part of the product contract. It tells standalone users the artifact runs in Zellij without requiring the full Yazelix runtime.

## Rust and filesystem names

Rust crate names, Rust module names, binaries, and installed data paths use underscores:

```text
yazelix_zellij_bar
yazelix_zellij_bar_widget
share/yazelix_zellij_bar
share/doc/yazelix_zellij_bar
```

This follows the Yazelix repository rule that file and directory names use underscores, not hyphens.

## Command names

Commands shipped by a Zellij plugin package use the same underscore prefix:

```text
yazelix_zellij_<plugin>_<command>
```

Prefer short subcommands inside the binary over long binary names or repeated suffixes. For example:

```text
yazelix_zellij_bar_widget cursor
yazelix_zellij_bar_widget codex
yazelix_zellij_bar_widget claude
yazelix_zellij_bar_widget opencode_go
yazelix_zellij_bar_widget cpu
yazelix_zellij_bar_widget ram
```

Long options are escape hatches, not the default standalone configuration. Common standalone usage should work with XDG defaults and short subcommands.

## Exceptions

Do not use `zellij` in the name when the artifact is not primarily a Zellij plugin package.

Examples:

- `yazelix-cursors` stays broader than a Zellij plugin when it owns cursor facts or terminal integration outside Zellij
- maintainer-only helper scripts stay under Yazelix-owned script paths
- libraries that are not installable Zellij plugin packages should use their domain name rather than implying Zellij runtime ownership

## Compatibility

Renames should not preserve stale command surfaces by default. Keep old names only when there is a documented compatibility decision with an owner, removal condition, and verification path.

## Verification

- `cargo test --manifest-path rust_core/Cargo.toml -p yazelix_core zellij_materialization`
- `nix build github:luccahuguet/yazelix-zellij-bar#yazelix_zellij_bar`
