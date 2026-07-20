# FlexNetOS foundation contract

The FlexNetOS package is a composition around canonical Yazelix Nova, not a
second runtime implementation.

## Lineage

The recovery merge has canonical Nova commit
`1bf5aa3253ae5a99a679023b75fb8ac0433efa59` as first parent and FlexNetOS commit
`4112c7ee6952c5d813ec11e10be8db19aa48e8ba` as second parent. The merge keeps
the complete ancestry of both repositories. Recovery must not use a discard,
cherry-pick, replay, rebase, squash, or force-push interpretation.

## Ownership

The installed product has exactly one owner at each layer:

```text
editable input:    ~/.config/yazelix
generated runtime: ~/.local/share/yazelix
installed command: ~/.nix-profile/bin/yzx
```

`~/.local/share/yazelix` is generated only by the installed Nova runtime.
Operators edit `~/.config/yazelix`; they do not patch generated files.

The sole Nix profile element is `lifeos_foundation_yzx`. It provides the
profile-owned agent layout at
`configs/zellij/layouts/flexnetos_agent_workspace.kdl`, and its only desktop
entry runs `/home/flexnetos/.nix-profile/bin/yzx launch` directly. Regular
Yazelix and agent Yazelix are the same path.

`/home/flexnetos/.nix-profile` is also the selector owner: it points to its own
`.nix-profile-N-link` generation beside the frontdoor. It must not alias
`~/.local/state/nix/profile`, even when both links currently resolve to the same
store closure. The migration archives that XDG selector and its generation
links under Meta's authoritative
`/home/flexnetos/.local/state/meta/archives/yazelix-nix-profile/` root before
creating the explicit profile. Generated Yazelix runtime state is proof only.
A failed install or closure verification archives the candidate and restores
every prior link.

## Nushell

Nushell is the only supported managed shell. Product sources remain under
`nushell/config/` and `nushell/scripts/`; the Nix package substitutes their
store paths into Nova's packaged Nushell config. Nova then materializes a
generated layered config under `~/.local/share/yazelix/nu/`.

## Verification

Build source contracts before installing:

```nu
nix build .#checks.x86_64-linux.flexnetos_foundation_contracts --no-link
nix build .#checks.x86_64-linux.single_profile_contract --no-link
nix build .#lifeos_foundation_yzx --no-link --print-out-paths
~/.nix-profile/bin/yazelix_profile_check
```

The contract checks one desktop file, the direct profile `Exec`, absence of
launcher wrappers, the profile layout, both Nushell source directories,
mandatory Nushell, `yzx status`, `yzx doctor`, and generated runtime identity.
The single-profile gate additionally rejects absolute or XDG selector aliases,
broken legacy links, extra manifest elements, closure drift, and missing
frontdoor binaries. `yazelix_profile_migrate --closure <built-closure>` emits a
read-only plan by default; `--execute` is the explicit Tier-B mutation toggle.
