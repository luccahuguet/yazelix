# Python Pack Debugging Investigation

> **Legacy Note:** This investigation references the former `yazelix.nix` configuration workflow prior to the TOML migration. Current releases use `yazelix.toml` by default.

**Date:** 2025-11-02
**Issue:** Python pack is enabled in yazelix.nix but tools (uv, ruff, ty, ipython) are not available in the shell

## Problem Statement

User has `language_packs = ["python"]` in yazelix.nix, but when entering a Yazelix shell or Zellij pane:
```bash
$ uv
Error: nu::shell::external_command
Command `uv` not found
```

This worked in the old flake-based version of Yazelix (v10) but broke after switching to devenv in v10.5.

## Investigation Timeline

### 1. Initial Configuration Verification

**yazelix.nix (User Config):**
```nix
language_packs = [
  "python"  # ruff, uv, ty, ipython
];
```
✅ Python pack is enabled

**devenv.nix (Pack Definitions):**
```nix
packDefinitions = {
  python = with pkgs; [
    ruff
    uv
    ty
    python3Packages.ipython
  ];
  # ... other packs
};
```
✅ Python pack definition exists and looks correct

**Pack Selection Logic:**
```nix
selectedLanguagePacks = userConfig.language_packs or [ ];
selectedToolPacks = userConfig.tool_packs or [ ];
selectedPacks = selectedLanguagePacks ++ selectedToolPacks;
packPackages = builtins.concatLists (
  map (packName:
    if builtins.hasAttr packName packDefinitions then
      packDefinitions.${packName}
    else
      throw "Unknown pack '${packName}'"
  ) selectedPacks
);
```

Verified pack selection is working:
```bash
$ nix eval --impure --json --expr '...selectedPacks...'
{"packCount":1,"selectedPacks":["python"]}
```
✅ Pack selection logic works correctly

Verified packPackages computation:
```bash
$ nix eval --impure --json --expr '...packPackages length...'
4
```
✅ packPackages has 4 items (uv, ruff, ty, ipython)

**Final Package List:**
```nix
allDeps =
  essentialDeps
  ++ extraShellDeps
  ++ (if recommendedDepsEnabled then recommendedDeps else [ ])
  ++ (if yaziExtensionsEnabled then yaziExtensionsDeps else [ ])
  ++ (if yaziMediaEnabled then yaziMediaDeps else [ ])
  ++ packPackages  # Should include Python pack
  ++ (userConfig.user_packages or [ ]);

in {
  packages = allDeps;  # Passed to devenv
}
```
✅ packPackages is included in allDeps which is passed to devenv's `packages` attribute

### 2. Actual devenv Build Verification

**Using `devenv info`:**
```bash
$ devenv info
# packages
- zellij-0.43.1
- helix-25.07.1
- yazi-25.5.31
- nushell-0.108.0
- fzf-0.66.0
- zoxide-0.9.8
- starship-1.23.0
- bash-interactive-5.3p3
- macchina-6.4.0
- mise-2025.9.10
- libnotify-0.8.7
- yazelix-desktop-launcher
- com.yazelix.Yazelix.desktop
- nixGLIntel
- yazelix-ghostty
- ghostty-1.2.2
- lazygit-0.55.1
- atuin-18.8.0
- carapace-1.5.3
```

Count: **19 packages** (should be 23+ with Python pack)

**Using `devenv print-dev-env` PATH analysis:**

Packages found in PATH:
- ✅ Essential: zellij, helix, yazi, nushell, fzf, zoxide, starship, bash, macchina, mise
- ✅ Linux-specific: libnotify
- ✅ Desktop: yazelix-desktop-launcher, com.yazelix.Yazelix.desktop
- ✅ nixGL: nixGLIntel
- ✅ Ghostty: yazelix-ghostty, ghostty
- ✅ recommendedDeps: lazygit, atuin, carapace, markdown-oxide
- ✅ yaziExtensionsDeps: p7zip, jq, fd, ripgrep
- ❌ **yaziMediaDeps: MISSING** (ffmpeg, imagemagick)
- ❌ **Python pack: MISSING** (uv, ruff, ty, ipython)

### 3. DEVENV_PROFILE Inspection

```bash
$ ls $DEVENV_PROFILE/bin | wc -l
31
```

Only 31 binaries total, confirming Python pack tools are not built.

### 4. Key Discovery: Packages Cutoff

Looking at `devenv info` output, devenv is only showing 19 packages, but we should have:
- 10 essential packages
- 1 libnotify
- 2 desktop entries
- 1 nixGL
- 2 ghostty items (wrapper + package)
- 4 from recommendedDeps (lazygit, atuin, carapace, markdown-oxide)
- 5 from yaziExtensionsDeps (p7zip, jq, fd, ripgrep, poppler)
- 2 from yaziMediaDeps (ffmpeg, imagemagick)
- 4 from Python pack (uv, ruff, ty, ipython)

**Expected:** 31 packages
**Actual:** 19 packages
**Missing:** 12 packages (yaziMediaDeps + Python pack)

## Findings Summary

1. ✅ Pack selection logic is correct
2. ✅ packPackages is computed correctly (4 items)
3. ✅ packPackages is included in allDeps
4. ✅ allDeps is passed to devenv's `packages` attribute
5. ✅ Some conditional deps work (recommendedDeps, yaziExtensionsDeps)
6. ❌ **devenv is NOT building yaziMediaDeps or packPackages**

## Hypothesis

devenv appears to be building packages correctly up through yaziExtensionsDeps, but then stops before processing yaziMediaDeps and packPackages. This suggests:

1. **Possible evaluation error** that's being silently ignored
2. **Nix list concatenation issue** with later items in allDeps
3. **devenv internal limit** on packages or list processing

## Next Steps

1. Check if there's a Nix evaluation error with yaziMediaDeps or packPackages
2. Test minimal reproduction case with just Python pack
3. Check devenv logs for errors during package evaluation
4. Try explicitly listing Python packages in allDeps to see if order matters

## Files Modified During Investigation

- `devenv.nix` line 372: Removed manual `PATH` setting (violates devenv best practices)
- `nushell/config/config.nu` lines 41-50: Added DEVENV_PROFILE/bin PATH restoration
- Created `docs/devenv_reference.md`: Comprehensive devenv DSL documentation

## ROOT CAUSE DISCOVERED

**Nix Flakes Git Tracking Issue**

```
error: Path 'yazelix.nix' in the repository "/home/lucca/.config/yazelix" is not tracked by Git.
```

devenv uses Nix flakes internally, and **flakes only see Git-tracked files**. Since `yazelix.nix` is not tracked by Git (it's the user's personal config file), devenv cannot read its contents during evaluation.

This explains why:
1. `userConfig` has all attribute keys but with empty values
2. `language_packs` is an empty list instead of `["python"]`
3. `user_packages` is an empty list instead of the 4 Python packages
4. packPackages is computed correctly in standalone Nix evaluation (which doesn't use flakes) but is empty in devenv context

### Verification

```bash
# Standalone Nix eval works (no flakes):
$ nix eval --impure --json --expr '...'
{"userPackagesCount":4,"firstPkg":"uv"}  # ✅ Works

# devenv eval fails (uses flakes):
$ devenv shell
language_packs count: 0  # ❌ Empty!
user_packages count: 0   # ❌ Empty!
```

### The Problem

devenv.nix imports yazelix.nix like this:

```nix
userConfig =
  if builtins.pathExists configFile then
    import configFile { inherit pkgs; }
  else if builtins.pathExists defaultConfigFile then
    import defaultConfigFile { inherit pkgs; }
  else
    { };
```

- `builtins.pathExists` returns `true` (file exists in filesystem)
- But `import` **fails silently** because flakes can't see untracked files
- Result: empty attrset with default values

## Solution Implemented

**Convert config from Nix to TOML format**

Nix flakes can READ file contents with `builtins.readFile` even for untracked files - they just can't IMPORT untracked .nix files. Solution:

1. Created `yazelix.toml` with all configuration options
2. Modified `devenv.nix` to use:
   ```nix
   rawConfig = builtins.fromTOML (builtins.readFile ./yazelix.toml)
   ```
3. Parse TOML into the structure devenv.nix expects
4. Add `yazelix.toml` to `.gitignore` so users can customize without Git conflicts

### Benefits

- ✅ User configs work without Git tracking
- ✅ Users can pull repo updates without merge conflicts
- ✅ Simpler config syntax (TOML vs Nix)
- ✅ Backward compatible (still supports yazelix.nix as fallback)

### Verification

```bash
# With yazelix.toml having language = ["python"]:
$ devenv shell -- which uv
/nix/store/.../uv
✅ Python pack works!
```

## Status

**FULLY RESOLVED** - TOML config solution implemented and tested successfully.
