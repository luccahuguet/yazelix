# Child-Repo Beads Ownership Policy

Yazelix uses Beads in the repository that owns the work. The main repository
owns integrated product planning, public Yazelix issue mapping, and release
transactions. Child repositories own their local source, package behavior,
standalone behavior, and implementation slices.

Cross-repo work may have Beads in more than one repository, but only one Bead is
the canonical executor for a given source edit. Other Beads should point to that
executor as parent, blocker, or release context instead of copying its full
acceptance criteria.

## Ownership Rules

| Work | Canonical Beads owner | Notes |
| --- | --- | --- |
| Child-only source, tests, docs, package metadata, or standalone behavior | Child repo | Use the child repo Beads database and close it with child-local evidence |
| Main runtime behavior, config, Home Manager, docs, package composition, or user contract | Main repo | Use the main repo Beads database |
| Main `flake.lock` update consuming a child change | Main repo | Treat as a coupled release transaction, even when the source edit was child-owned |
| Product feature whose implementation spans main and child repos | Main repo parent plus child repo implementation Beads | Main parent owns user outcome and release evidence; child Beads own source edits |
| Child standalone feature with no current Yazelix integration change | Child repo | Do not create a main Bead unless product planning or release integration is needed |
| Public GitHub issue about integrated Yazelix behavior | Main repo Bead with `external_ref` | The GitHub/Beads shared-subset contract in `AGENTS.md` applies |
| Public GitHub issue about a child standalone package or fork | Child repo Bead with that repo's GitHub issue reference | Do not mirror into main unless Yazelix integration also changes |

## Invariants

- A repository's Beads database owns work only for behavior, source, packaging,
  or release state owned by that repository
- Main-repo Beads should not become detailed implementation trackers for
  child-local edits
- Child-repo Beads should not own main-runtime release decisions
- Cross-repo work may use a main parent Bead plus child implementation Beads,
  but the child Bead is the canonical executor for child source edits
- A public GitHub issue maps to exactly one Bead in the repository that owns the
  public issue
- Cross-links are allowed; duplicate lifecycle ownership is not
- A main `flake.lock` update that consumes a child commit is a main-repo release
  transaction

## Local Workflow

For a child-owned source fix consumed by Yazelix:

1. Claim or create the child implementation Bead in the child repo
2. Implement and verify in the child repo
3. Commit and push the child repo
4. Update the main flake input to the published child revision
5. Run `yzx_repo_validator validate-child-release-transaction`
6. Run the relevant no-override main runtime or package validation
7. Commit and push the main release transaction after required manual approval

For a product feature that spans repos, keep the main Bead as the parent user
outcome and create child Beads only for concrete child-owned edits. Close child
Beads with child evidence. Close the main parent only after the integrated
runtime consumes the published child state and the main acceptance criteria are
verified.

## Agent Discovery

Agents should start from the repository named by the user or current working
directory, read its local `AGENTS.md`, and use that repo's `br` for local work.
When a task names both a main Bead and a child source edit, agents should
inspect both Beads databases and preserve the source-owner/release-owner split.

## Verification

- `yzx_repo_validator validate-child-release-transaction`
- `yzx dev validate_issue_contract`
- manual review of `br show <id>` in the repository being edited

## Related Contracts

- [Fork And Child-Repo Maintenance](./contracts/fork_child_repo_maintenance.md)
- [Artifact-First Child Integration](./contracts/artifact_first_child_integration.md)
