# Documentation Architecture

Yazelix documentation has one durable source-of-truth hierarchy. Delete or demote ambiguous categories before adding a new document type.

## Canonical Surfaces

| Surface | Owns | Does not own |
| --- | --- | --- |
| `README.md` | The public front door, install hint, and high-level product promise | Detailed reference material or maintainer process |
| `docs/contracts/` | Current Yazelix contracts: product behavior, runtime boundaries, subsystem ownership, supported failure semantics, and validation policy | Research, implementation diaries, historical audits, migration plans, release archaeology, or Bead execution history |
| User guides under `docs/` | Task-oriented user explanation derived from contracts | New source-of-truth semantics that contradict contracts |
| Maintainer docs under `docs/` | How maintainers operate the repo, release surface, validators, and tooling | Planning status or issue sequencing |
| Architecture maps and inventories | Current ownership maps, measured code inventories, and deletion targets | Historical rationale when the live owner is already clear |
| `docs/history.md` and explicit historical notes | Human-readable past context that is not normative | Current implementation requirements |
| Beads | Decisions, investigations, rejected alternatives, implementation sequencing, and closure evidence | Durable behavior contracts |

## Rules

1. If a file says what Yazelix currently promises or rejects, it belongs in `docs/contracts/`
2. If a file says why a choice was made, what was investigated, or what was tried, it belongs in Beads or a non-canonical historical note
3. If two documents claim source-of-truth status for the same behavior, delete or demote one before editing both
4. Contracts should be current-tense and normative
5. Beads should point to contracts; contracts should not point back to Beads unless the contract is about the Beads/GitHub planning architecture itself
6. Historical content can survive only when it is explicitly non-canonical and still useful to readers

## Deletion Preference

When cleaning docs, prefer this order:

1. Delete obsolete planning or duplicated prose
2. Move useful historical context to Beads or `docs/history.md`
3. Rewrite a surviving file as a smaller current contract
4. Rename only after the surviving purpose is clear
