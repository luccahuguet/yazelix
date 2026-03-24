## Contributing to Yazelix

We welcome contributions to Yazelix! Here are some guidelines to help you get started:

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

### Workflow

1. **Choose an issue** to work on or create a new one if needed.
2. **Create a new branch** following the naming convention above.
3. **Check whether the change needs a spec**. User-visible behavior changes, cross-subsystem boundary changes, and supported integration behavior should usually get a short spec under [`docs/specs/`](/home/lucca/.config/yazelix/docs/specs). See [Spec-Driven Workflow](./spec_driven_workflow.md).
4. **For test work, use the suite policy**. Before adding or moving tests, read [Test Suite Governance](./specs/test_suite_governance.md) so the lane choice and justification stay consistent.
5. **Make your changes** in your branch, adhering to the existing code style.
6. **Commit your changes** using the commit message format described above.
7. **Push your branch** to your fork on GitHub.
8. **Open a pull request** against the `main` branch of the Yazelix repository.
9. **Describe your changes** in the PR description, linking to the relevant issue(s).

Thank you for contributing to Yazelix!
