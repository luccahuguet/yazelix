# Nushell Scripts Organization

This directory contains the remaining irreducible Nushell support files in Yazelix

## Current Shape

### `dev/`

Maintainer fixtures and inventory metadata for Rust-owned test runners and validators

### `maintainer/`

Maintainer test-suite inventory metadata

## Runtime Ownership

Normal startup, launch, restart, desktop, popup, generated-state repair, welcome
sequencing, and Rust-helper JSON transport are Rust-owned

Release metadata lives in root `release_metadata.toml` during maintainer work and
is packaged into `runtime_identity.json.version` for runtime version reporting

Use the shipped CLI and Rust-owned public commands

```bash
yzx launch
yzx enter
yzx run <command>
~/.config/yazelix/shells/posix/yzx_cli.sh help
```

For maintainer workflows, prefer the Rust-owned runner surfaces

```bash
yzx dev rust fmt --check
yzx dev rust check
yzx dev rust test <filter>
yzx dev test --lint-only
yzx dev test
```

## File Naming Convention

All files use underscores
