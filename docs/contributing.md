## Contributing to Yazelix

We welcome contributions to Yazelix! Here are some guidelines to help you get started:

### Repository Root

Treat the repo root as a high-signal surface.

Keep files at the root only when they are one of:
- repository entrypoints such as [`README.md`](../README.md), [`CHANGELOG.md`](../CHANGELOG.md), and [`AGENTS.md`](../AGENTS.md)
- build and packaging entrypoints such as [`flake.nix`](../flake.nix), [`maintainer_shell.nix`](../maintainer_shell.nix), and the top-level package front doors
- maintainer workflow files such as [`.pre-commit-config.yaml`](../.pre-commit-config.yaml) and [`.taplo.toml`](../.taplo.toml)
- source-of-truth templates or contracts that are intentionally top-level, such as [`yazelix_default.toml`](../yazelix_default.toml)

If a file is exploratory, subsystem-specific, or only useful as supporting documentation, prefer a home under [`docs/`](./), [`packaging/`](../packaging), or the owning subsystem directory instead of leaving it at the root.
### Branch Naming Convention

When creating a new branch to work on an issue, please use the following naming convention:

```
issue_{number-of-issue}
```

For example, if you're working on issue #42, your branch should be named `issue_42`.

We follow a "one branch per issue" approach to keep changes focused and manageable.

### Commit Messages

For commit messages, please use the following format:

```
#{issue-number} {commit description}
```

For example, a commit addressing issue #42 might look like:

```
#42 Add rounded corners to sidebar
```

This helps in easily tracking which commits are related to specific issues.


Thank you for contributing to Yazelix!
