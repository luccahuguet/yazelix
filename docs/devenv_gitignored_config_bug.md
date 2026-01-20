# devenv Gitignored Config Bug Investigation

> **Legacy Note:** This investigation covers issues encountered with the legacy `yazelix.nix` workflow. Yazelix now defaults to `yazelix.toml` while keeping legacy fallback support.

**Date:** 2025-11-02
**Branch:** poc/devenv-caching
**Status:** ✅ RESOLVED

## Summary

Yazelix's migration to TOML-based configuration with devenv caching revealed a critical bug: **gitignored files are invisible to pure Nix evaluation**, causing `yazelix.toml` configuration to be silently ignored in favor of defaults.

**Impact:** User configuration changes (like enabling `extra_shells = ["fish", "zsh"]` or `packs.enabled = ["python"]`) were not applied, resulting in missing packages (fish, zsh, uv, ruff, etc.) in the devenv environment.

## Problem Description

### Symptoms

1. **Configuration ignored**: Changes to `yazelix.toml` had no effect on the devenv environment
2. **Missing packages**: Fish, zsh, uv, and python pack tools were not available despite being enabled in config
3. **Silent fallback**: No warnings or errors - devenv silently used `yazelix_default.toml` instead
4. **Cache persistence**: Even clearing `.devenv/nix-eval-cache.db*` didn't help
5. **Profile unchanged**: Same Nix store path even after "rebuilding"

### User Experience

```toml
# User's yazelix.toml
[shell]
extra_shells = ["fish", "zsh"]

[packs]
enabled = ["python"]

[packs.declarations]
python = ["ruff", "uv", "ty", "python3Packages.ipython"]
```

```bash
$ which fish
# (empty - not found)

$ which uv
# (empty - not found)

$ devenv shell bash -c 'echo $extraShells'
# Output: extras=NONE, includeFish=false
```

## Root Cause Analysis

### The Gitignore Problem

**Key insight:** In pure Nix evaluation (used by flakes and devenv), **gitignored files are invisible** to path-based checks.

```nix
# This was in devenv.nix
tomlConfigFile = ./yazelix.toml;  # Relative path

rawConfig =
  if builtins.pathExists tomlConfigFile then  # ❌ Always FALSE for gitignored files!
    builtins.fromTOML (builtins.readFile tomlConfigFile)
  else
    builtins.fromTOML (builtins.readFile defaultTomlConfigFile);  # ✅ Always taken
```

**Why this happens:**
- Nix flakes and devenv use **pure evaluation** for reproducibility
- Pure evaluation restricts access to the filesystem
- Gitignored files are explicitly excluded from the evaluation context
- `builtins.pathExists ./yazelix.toml` returns `false` even though the file exists on disk

### The Misleading Comment

The original code contained an incorrect assumption:

```nix
# Import user configuration from TOML
# TOML files can be read with builtins.readFile even if untracked by Git!
# This solves the Nix flakes limitation with untracked .nix files
```

**This is FALSE.** While TOML files don't have the flake auto-import problem that `.nix` files have, **gitignored files are still invisible** in pure mode.

### Investigation Process

#### 1. Initial Hypothesis: Cache Invalidation Issue

First assumption was that devenv's SQLite cache wasn't detecting config changes.

**Evidence:**
```bash
$ sha256sum ~/.config/yazelix/yazelix.toml
dd80c4583dee061a4f071869f2c3ae16...  # Current hash

$ cat ~/.local/share/yazelix/state/config_hash
4eea1066e5c6aa236d107fd42a116416...  # Cached hash (different!)
```

Config change was detected, but packages still missing. ❌ Not the root cause.

#### 2. Testing Cache Clearing

```bash
$ rm -rf .devenv/nix-eval-cache.db*
$ devenv shell bash -c 'echo test'
# Building shell in 7.49s (fresh build, not cached)
# Output: extras=NONE, includeFish=false
```

Fresh build still showed wrong config! ❌ Not a cache issue.

#### 3. Direct TOML Parsing Test

```bash
$ nix eval --impure --expr 'let toml = builtins.fromTOML (builtins.readFile ./yazelix.toml); in toml.shell.extra_shells'
[ "fish" "zsh" ]  # ✅ Nix CAN read it with --impure

$ nix eval --impure --expr 'let toml = builtins.fromTOML (builtins.readFile ./yazelix.toml); in toml.packs.enabled'
[ "python" ]  # ✅ Config is valid
```

Nix can parse the TOML when using `--impure`, but devenv wasn't using it.

#### 4. Adding Debug Traces

Modified `devenv.nix` to add trace output:

```nix
rawConfig =
  if builtins.pathExists tomlConfigFile then
    builtins.trace "📝 Reading yazelix.toml" (...)
  else
    builtins.trace "⚠️ Reading yazelix_default.toml (not found)" (...);
```

**Result:**
```bash
$ devenv shell bash -c 'echo test'
trace: ⚠️ Reading yazelix_default.toml (yazelix.toml not found)
trace: extraShells = []
trace: includeFish = FALSE
```

🎯 **Breakthrough!** Despite the file existing on disk, `builtins.pathExists` returned false.

#### 5. Confirming Gitignore as Culprit

```bash
$ cat .gitignore | grep yazelix.toml
yazelix.toml  # ✅ Confirmed gitignored

$ git ls-files yazelix.toml
# (empty - not tracked by git)
```

Pure Nix evaluation can only see:
- Files tracked in git
- Files in the Nix store
- Absolute paths accessed via `--impure` mode

## The Solution

### Approach: Use --impure Mode with Absolute Paths

Enable impure evaluation and use `$HOME` environment variable to construct an absolute path that bypasses flake purity restrictions.

### Changes Made

#### 1. devenv.nix - Read from Absolute Path

```nix
# BEFORE (lines 15-22)
tomlConfigFile = ./yazelix.toml;
defaultTomlConfigFile = ./yazelix_default.toml;

rawConfig =
  if builtins.pathExists tomlConfigFile then
    builtins.fromTOML (builtins.readFile tomlConfigFile)
  else
    builtins.fromTOML (builtins.readFile defaultTomlConfigFile);

# AFTER
# In pure Nix evaluation (flakes/devenv), gitignored files are invisible
# We must use --impure mode and read from an absolute path via $HOME
# All devenv shell calls include --impure flag to enable this
homeDir = builtins.getEnv "HOME";
tomlConfigFile = if homeDir != "" then "${homeDir}/.config/yazelix/yazelix.toml" else "";
defaultTomlConfigFile = ./yazelix_default.toml;

rawConfig =
  if tomlConfigFile != "" && builtins.pathExists (builtins.toPath tomlConfigFile) then
    builtins.fromTOML (builtins.readFile tomlConfigFile)
  else
    builtins.fromTOML (builtins.readFile defaultTomlConfigFile);
```

**Key points:**
- `builtins.getEnv "HOME"` requires `--impure` flag
- Absolute path `"${homeDir}/.config/yazelix/yazelix.toml"` can be accessed in impure mode
- Falls back to `yazelix_default.toml` if file doesn't exist or not in impure mode

#### 2. Add --impure to All devenv shell Calls

Modified 5 files to add `--impure` flag:

**nushell/scripts/core/yazelix.nu** (4 occurrences):
```nu
# BEFORE
let devenv_cmd = $"cd ($yazelix_dir) && devenv shell($refresh_flag) -- bash -c '($full_cmd)'"

# AFTER
let devenv_cmd = $"cd ($yazelix_dir) && devenv shell --impure($refresh_flag) -- bash -c '($full_cmd)'"
```

**nushell/scripts/core/start_yazelix.nu**:
```nu
let devenv_cmd = $"cd ($yazelix_dir) && devenv shell --impure($refresh_flag) -- bash -c '($cmd)'"
```

**nushell/scripts/core/desktop_launcher.nu**:
```nu
let devenv_cmd = $"cd ($yazelix_dir) && devenv shell --impure($refresh_flag) -- nu ($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu ($env.HOME)"
```

**shells/bash/start_yazelix.sh**:
```bash
HOME="$HOME" devenv shell --impure$REFRESH_FLAG -- bash -c \
  "zellij --config-dir \"$YAZELIX_DIR/configs/zellij\" ..."
```

### Files Changed

```
devenv.nix                               | 11 ++++++-----
nushell/scripts/core/desktop_launcher.nu |  2 +-
nushell/scripts/core/start_yazelix.nu    |  2 +-
nushell/scripts/core/yazelix.nu          |  8 ++++----
shells/bash/start_yazelix.sh             |  2 +-
5 files changed, 13 insertions(+), 12 deletions(-)
```

## Verification

### Before Fix

```bash
$ devenv shell bash -c 'which fish'
# (empty)

$ nix-store --query --references ~/.config/yazelix/.devenv/profile | grep -E "fish|uv"
# (empty - packages not in profile)

$ devenv shell bash -c 'echo includeFish check'
trace: ⚠️ Reading yazelix_default.toml (yazelix.toml not found)
trace: extraShells = []
trace: includeFish = FALSE
🔁 Yazelix shell config: default=nu, extras=NONE, includeFish=false, includeZsh=false
```

### After Fix

```bash
$ devenv shell --impure bash -c 'which fish && which uv'
trace: 📝 Reading yazelix.toml from /home/lucca/.config/yazelix/yazelix.toml
trace: extraShells = ["fish","zsh"]
trace: includeFish = TRUE
🔁 Yazelix shell config: default=nu, extras=fish,zsh, includeFish=true, includeZsh=true
/nix/store/.../bin/fish
/nix/store/.../bin/uv

$ nix-store --query --references ~/.config/yazelix/.devenv/profile | grep -E "fish|uv|ruff"
/nix/store/20bpx2vxfaaq62r5v43ii9006ayz0y77-ruff-0.14.1
/nix/store/n6chrdybb91npp8gvf8mjk55smx4sn8s-uv-0.8.23
/nix/store/r9rwlsgp3ky08jjb4cangyh408hvwjz7-fish-4.1.2-doc
/nix/store/vl5ml2v2ffgyr90k13x8k4dl98d12bi2-fish-4.1.2
```

✅ **Profile changed from:**
- `/nix/store/2awd6lbb20kxjcsj6zl2j50kr2lk78v3-devenv-profile` (47 packages, no fish/uv)

**To:**
- `/nix/store/ksy5f0z0gqdq8sg5wbqcgj7lj2d4hnp8-devenv-profile` (54 packages, includes fish/zsh/uv/ruff/ipython)

### Shell Configuration Verification

```bash
$ devenv shell --impure bash -c 'echo test'
✅ Generated 16 shell initializers successfully  # Was 8 before (only bash/nu)
✅ Bash config already sourced
✅ Nushell config already sourced
✅ Fish config already sourced       # ← NEW!
✅ Zsh config already sourced        # ← NEW!
✅ Yazelix environment setup complete!
```

## Technical Deep Dive

### Why Gitignored Files Are Invisible to Flakes

**Nix Flakes Purity Model:**
1. Flakes use Git to track inputs for reproducibility
2. Only files tracked by Git are visible to pure evaluation
3. This ensures the same flake always evaluates to the same result
4. Environment access (`builtins.getEnv`) is forbidden in pure mode

**Directory scanning in pure mode:**
```nix
# Pure mode (default)
builtins.readDir ./.           # Only sees tracked files
builtins.pathExists ./foo.txt  # False for gitignored files
./yazelix.toml                 # Path doesn't resolve

# Impure mode (--impure flag)
builtins.getEnv "HOME"         # Allowed
"/absolute/path/to/file"       # Can access any file
builtins.pathExists "/..."     # Works for all files
```

### Why This Wasn't Caught Earlier

1. **Testing bias**: Developers likely had `yazelix.toml` committed during development
2. **Silent fallback**: No error message when falling back to default config
3. **Partial functionality**: Most of yazelix still worked with default config
4. **Cache confusion**: The Nushell-level config hash detection worked fine, masking the deeper issue

### devenv's --impure Flag

The `--impure` flag tells devenv (and underlying Nix) to allow:
- Reading environment variables via `builtins.getEnv`
- Accessing files outside the Git repository
- Non-deterministic operations

**Trade-off:**
- ✅ Allows reading gitignored user config
- ✅ Enables dynamic configuration per-machine
- ⚠️ Evaluations are no longer purely reproducible across machines (but that's okay for user config)

## Alternative Solutions Considered

### 1. Remove yazelix.toml from .gitignore ❌

**Pros:**
- Pure evaluation would work
- No need for `--impure` flag

**Cons:**
- Merge conflicts when pulling updates
- User configs would pollute git status
- Goes against the design goal of gitignored user config

**Verdict:** Rejected - defeats the purpose of gitignored config

### 2. Copy yazelix.toml to .devenv/ Before Evaluation ❌

**Pros:**
- .devenv/ is not gitignored
- Would be visible to pure evaluation

**Cons:**
- Race conditions and sync complexity
- Breaks devenv's atomic evaluation model
- Added complexity in build pipeline

**Verdict:** Rejected - too complex and fragile

### 3. Use Environment Variables Instead of TOML ❌

**Pros:**
- Simple to implement
- Already requires `--impure`

**Cons:**
- Poor user experience (100+ options as env vars)
- Loss of TOML validation and structure
- Harder to document and manage

**Verdict:** Rejected - terrible UX

### 4. Use --impure + Absolute Path ✅ CHOSEN

**Pros:**
- Minimal code changes
- Preserves TOML-based config
- Clear semantics (impure = reads user config)
- One-time setup cost

**Cons:**
- Requires `--impure` flag on all calls (but this is acceptable)
- Slightly less reproducible (but user config is inherently machine-specific)

**Verdict:** ✅ Best balance of simplicity and functionality

## Impact on Caching Performance

### Initial Concern

Would `--impure` mode disable devenv's caching benefits?

### Testing Results

**Cache still works!** devenv's SQLite evaluation cache operates independently:

```bash
# First run (cache miss)
$ time devenv shell --impure bash -c 'true'
Building shell in 7.2s

# Second run (cache hit)
$ time devenv shell --impure bash -c 'true'
Building shell in 0.08s  # ✅ 90x faster!
```

**Why caching still works:**
- devenv caches the **result** of Nix evaluation
- The `--impure` flag affects **what** is evaluated, not **whether** it's cached
- As long as inputs don't change, cached results are reused
- Changes to `yazelix.toml` are detected by the Nushell-level hash check and trigger `--refresh-eval-cache`

## Lessons Learned

### 1. Test with Gitignored Files

Always test configuration systems with files in their intended state (tracked vs. ignored).

**Test checklist:**
- ✅ Config file committed (development)
- ✅ Config file gitignored (production)
- ✅ Config file missing (fresh clone)
- ✅ Config file modified (user changes)

### 2. Fail Loudly, Not Silently

The original code silently fell back to defaults. Better approach:

```nix
rawConfig =
  if tomlConfigFile != "" && builtins.pathExists (builtins.toPath tomlConfigFile) then
    builtins.fromTOML (builtins.readFile tomlConfigFile)
  else if !impureMode then
    builtins.throw "yazelix.toml requires --impure mode"  # Fail fast
  else
    builtins.fromTOML (builtins.readFile defaultTomlConfigFile);
```

(Not implemented to avoid breaking existing workflows, but worth considering)

### 3. Document Purity Assumptions

Any code using `builtins.pathExists`, `builtins.readDir`, or relative paths should document:
- Pure vs. impure mode requirements
- Gitignore interactions
- Fallback behavior

### 4. Trace Debugging in Nix

`builtins.trace` is invaluable for debugging Nix evaluation:

```nix
value = builtins.trace "DEBUG: value = ${builtins.toJSON value}" value;
```

Output appears in build logs and helps track evaluation flow.

## Future Considerations

### 1. Explicit Impure Mode Check

Add a check to ensure `--impure` is used:

```nix
homeDir = builtins.getEnv "HOME";
_ = if homeDir == "" then
      builtins.throw "yazelix devenv.nix requires --impure mode. Use: devenv shell --impure"
    else
      null;
```

### 2. Config Validation

Validate TOML structure and provide helpful errors:

```nix
validateConfig = config:
  if !(config ? shell) then
    throw "Invalid yazelix.toml: missing [shell] section"
  else if !(config.shell ? default_shell) then
    throw "Invalid yazelix.toml: missing shell.default_shell"
  else
    config;

rawConfig = validateConfig (
  if ... then builtins.fromTOML (...) else ...
);
```

### 3. Migration Guide for Users

Document the change for users who might have their own yazelix forks:

```markdown
## Upgrading to v10.5+ (devenv caching)

If you use custom scripts that call `devenv shell`, add `--impure` flag:

# Before
devenv shell -- bash -c 'yzx info'

# After
devenv shell --impure -- bash -c 'yzx info'

This is required to read gitignored yazelix.toml configuration.
```

### 4. Alternative: Home Manager Integration

For users who want pure evaluation, recommend Home Manager integration which generates `yazelix.nix` from Nix options (not gitignored):

```nix
programs.yazelix = {
  enable = true;
  extraShells = [ "fish" "zsh" ];
  packs.enabled = [ "python" ];
};
```

This is already supported and doesn't require `--impure` mode.

## Related Issues

- **Issue:** Nix flakes and gitignored files (upstream Nix limitation)
- **PR:** Switch to TOML/devenv workflow (commit 4c91eb3)
- **PR:** Refresh devenv cache on config changes (commit fac84a8) - partially addressed this
- **Docs:** See `boot_sequence.md` for how configuration flows through the system

## References

- [Nix Manual: Flakes](https://nixos.org/manual/nix/stable/command-ref/new-cli/nix3-flake.html)
- [devenv Documentation](https://devenv.sh/)
- [Nix builtins reference](https://nixos.org/manual/nix/stable/language/builtins.html)
- Git issue: NixOS/nix#7107 - "Flakes can't access gitignored files"

## Conclusion

The bug was caused by a fundamental mismatch between:
1. Yazelix's design (gitignored user config)
2. Nix flakes' purity model (only tracked files visible)

The solution leverages `--impure` mode to access gitignored files via absolute paths, preserving both the gitignored config UX and devenv's caching benefits.

**Status:** ✅ Resolved in commit `[pending]`
**Branch:** `poc/devenv-caching`
**Next step:** Merge to main after testing
