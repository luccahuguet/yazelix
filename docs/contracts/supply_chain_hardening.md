# Supply-Chain Hardening

## Summary

Yazelix should define a lightweight supply-chain hardening policy for the tool surfaces it ships and documents. The policy should keep the default installed/runtime-declared surface conservative, keep higher-risk tool ecosystems explicitly opt-in, and give maintainers a repeatable workflow for triaging and communicating future supply-chain advisories.

## Why

Yazelix ships and documents many third-party tools, including fast-moving AI and agent tooling. Incidents like compromised package-manager releases are easier to handle when the project already knows:

- which tool surfaces are enabled by default
- which surfaces are opt-in
- which ecosystems are treated as higher risk
- which update paths Yazelix actually controls
- how maintainers should assess impact and communicate the result

Without that policy, every incident becomes ad-hoc reasoning under time pressure.

## Scope

- define risk-aware categories for Yazelix tool surfaces
- clarify what should remain opt-in by default
- define preferred runtime/update paths for shipped tools
- define a small review checklist for new default-enabled tools
- define a lightweight incident triage workflow for future advisories

## Behavior

- Default-installed or default-runtime-declared tool surfaces should stay conservative.
- Higher-risk ecosystems or fast-moving external tools should remain opt-in unless there is an explicit reason to make them default.
- Yazelix should prefer hermetic, Nix-managed runtime/update paths over user-global package-manager installs in its maintained surface.
- Adding a new default-enabled tool should require a small review that answers:
  - what ecosystem it comes from
  - whether it self-updates
  - whether it executes install-time or startup-time code from host/user paths
  - whether it is needed in the default surface or can stay opt-in
- For future supply-chain advisories, maintainers should classify impact using a short workflow:
  1. Is the affected package in the default surface, an opt-in pack, or nowhere in Yazelix?
  2. Is the affected path controlled by Yazelix or only by user-local host state?
  3. What concrete repo/runtime/package evidence supports the conclusion?
  4. What user-facing remediation or communication is needed?

## Non-goals

- exhaustive auditing of every transitive dependency of every upstream tool
- banning all non-Rust or non-Nix ecosystems
- implementing sandboxing or provenance enforcement immediately
- turning tool review into heavyweight process bureaucracy

## Acceptance Cases

1. When maintainers consider enabling a new default tool, they can classify whether it belongs in the default surface or an opt-in pack using a short documented checklist.
2. When a supply-chain incident affects an ecosystem Yazelix might touch, maintainers can quickly determine whether the default surface is affected, only opt-in surfaces are affected, or there is no direct Yazelix evidence.
3. The policy makes clear that hermetic Nix-managed runtime/update paths are preferred over user-global package-manager flows in Yazelix's maintained surface.
4. The policy is short enough to use during normal maintenance rather than being ignored as a giant security essay.

## Verification

- unit tests: n/a
- integration tests: n/a
- manual verification: review `maintainer_shell.nix`, `yazelix_default.toml`, `home_manager/module.nix`, and the documented packs against this policy
- manual verification: use a recent incident triage example to confirm the workflow leads to a specific default-surface / opt-in / not-present conclusion

## Traceability
- Defended by: `manual review of maintainer_shell.nix, yazelix_default.toml, and home_manager/module.nix against this policy`
- Defended by: `manual incident-triage review using a concrete advisory against the documented workflow`

## Open Questions

- Should Yazelix eventually add a validator for risky default package ecosystems, or stay policy-only for now?
- Should optional AI/agent packs get additional wrapper-level environment scrubbing or sandboxing later?
