# devenv Implementation Plan

**Goal:** Migrate Yazelix from `nix develop` to devenv for instant cold starts (<10ms) while preserving full automation

**Timeline:** 2-4 hours for PoC, 2-4 hours for full migration if PoC succeeds

**Status:** Planning phase

---

## Phase 1: Proof of Concept (PoC)

### Setup

**1. Create PoC branch**
```bash
git checkout -b poc/devenv-caching
```

**2. Add devenv to flake inputs**
```nix
# flake.nix
inputs = {
  nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  flake-utils.url = "github:numtide/flake-utils";
  helix.url = "github:helix-editor/helix";
  nixgl.url = "github:guibou/nixGL";
  devenv.url = "github:cachix/devenv";  # ADD THIS
};
```

**3. Create minimal devenv.nix**
```nix
# devenv.nix - minimal starting point
{ pkgs, lib, config, inputs, ... }:

{
  # Basic packages to test
  packages = with pkgs; [
    nushell
    zellij
    helix
  ];

  # Test environment variables
  env.YAZELIX_DIR = "$HOME/.config/yazelix";
  env.IN_YAZELIX_SHELL = "true";

  # Test shell hook execution
  enterShell = ''
    echo "devenv shell activated"
  '';
}
```

**4. Update flake.nix outputs**
```nix
outputs = { self, nixpkgs, flake-utils, helix, nixgl, devenv, ... }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      # ... existing code ...
    in
    {
      # Keep existing devShells.default for now (fallback)
      devShells.default = pkgs.mkShell { ... };

      # Add devenv shell for testing
      devShells.devenv = devenv.lib.mkShell {
        inherit inputs pkgs;
        modules = [
          ./devenv.nix
        ];
      };
    }
  );
```

**5. Test basic functionality**
```bash
# Enter devenv shell
nix develop .#devenv

# Measure performance
time nix develop .#devenv --command echo "test"

# Test caching (run again)
time nix develop .#devenv --command echo "test"
```

**Expected results:**
- First run: ~4s (similar to current)
- Second run: <100ms (should be instant with caching)

---

### Test Cases

Run each test incrementally, verifying functionality at each step:

#### Test 1: Basic Package Loading
**Goal:** Verify devenv can load packages

**Implementation:**
```nix
# devenv.nix
packages = with pkgs; [
  nushell
  zellij
  helix
  yazi
  bat
  fd
  ripgrep
];
```

**Test:**
```bash
nix develop .#devenv --command nu --version
nix develop .#devenv --command zellij --version
nix develop .#devenv --command hx --version
```

**Success criteria:** All commands work, versions match expectations

---

#### Test 2: Conditional Package Selection
**Goal:** Verify devenv can handle our packs system

**Implementation:**
```nix
# devenv.nix
{ pkgs, lib, config, ... }:

let
  # Read user config (same as current approach)
  homeDir = builtins.getEnv "HOME";
  configFile = "${homeDir}/.config/yazelix/yazelix.nix";
  userConfig = if builtins.pathExists configFile
    then import configFile { inherit pkgs; }
    else {};

  # Test pack definitions
  pythonPack = with pkgs; [
    ruff
    uv
    python3Packages.ipython
  ];

  rustPack = with pkgs; [
    rustc
    cargo
    rust-analyzer
  ];

  # Conditional package selection
  packPackages =
    (if builtins.elem "python" (userConfig.packs or []) then pythonPack else []) ++
    (if builtins.elem "rust" (userConfig.packs or []) then rustPack else []);
in
{
  packages = [
    pkgs.nushell
    pkgs.zellij
    pkgs.helix
  ] ++ packPackages;
}
```

**Test:**
```bash
# Edit yazelix.nix to add packs = ["python"];
echo '{ pkgs }: { packs = ["python"]; }' > ~/.config/yazelix/yazelix.nix

# Enter shell and verify python tools available
nix develop .#devenv --command ruff --version

# Change to rust pack
echo '{ pkgs }: { packs = ["rust"]; }' > ~/.config/yazelix/yazelix.nix

# Verify cache invalidation and rust tools available
nix develop .#devenv --command cargo --version
```

**Success criteria:**
- Packages conditionally loaded based on config
- Cache invalidates when yazelix.nix changes
- Second launch with same config is instant

---

#### Test 3: nixGL Wrappers
**Goal:** Verify devenv can handle custom package derivations

**Implementation:**
```nix
# devenv.nix
{ pkgs, lib, config, inputs, ... }:

let
  pkgsWithNixGL = import inputs.nixpkgs {
    inherit (pkgs) system;
    overlays = [ inputs.nixgl.overlay ];
  };

  ghosttyWrapper = pkgs.writeShellScriptBin "yazelix-ghostty" ''
    exec ${pkgsWithNixGL.nixgl.nixGLIntel}/bin/nixGLIntel ${pkgs.ghostty}/bin/ghostty \
      --class="com.yazelix.Yazelix" \
      --title="Yazelix - Ghostty" "$@"
  '';
in
{
  packages = [
    pkgs.nushell
    pkgs.helix
    ghosttyWrapper
  ];
}
```

**Test:**
```bash
nix develop .#devenv --command yazelix-ghostty --version
```

**Success criteria:** Custom wrapper works, wrapper script accessible

---

#### Test 4: shellHook / enterShell Scripts
**Goal:** Verify devenv can run Nushell setup scripts

**Implementation:**
```nix
# devenv.nix
{
  enterShell = ''
    # Set up directories
    mkdir -p ~/.local/share/yazelix/logs

    # Run Nushell setup script
    nu $YAZELIX_DIR/nushell/scripts/setup/environment.nu \
      "$YAZELIX_DIR" \
      "true" \
      "false" \
      "bash,nu"
  '';

  env.YAZELIX_DIR = "$HOME/.config/yazelix";
  env.IN_YAZELIX_SHELL = "true";
}
```

**Test:**
```bash
nix develop .#devenv
# Verify environment.nu ran successfully
# Check that initializers were generated
ls ~/.local/share/yazelix/shells/
```

**Success criteria:**
- Nushell scripts execute successfully
- Shell initializers generated
- No errors in enterShell

---

#### Test 5: Cross-Platform Logic
**Goal:** Verify Linux/macOS conditionals work

**Implementation:**
```nix
# devenv.nix
{ pkgs, lib, ... }:

let
  isLinux = pkgs.stdenv.isLinux;

  linuxPackages = lib.optionals isLinux [
    pkgs.ghostty
  ];

  macOSPackages = lib.optionals pkgs.stdenv.isDarwin [
    # macOS-specific packages
  ];
in
{
  packages = [
    pkgs.nushell
    pkgs.helix
  ] ++ linuxPackages ++ macOSPackages;
}
```

**Test:**
```bash
# On Linux, verify ghostty available
nix develop .#devenv --command which ghostty

# Check platform detection worked
nix develop .#devenv --command nu -c 'echo $env.GHOSTTY_AVAILABLE'
```

**Success criteria:** Platform-specific packages load correctly

---

#### Test 6: Cache Invalidation
**Goal:** Verify cache invalidates on file changes

**Implementation:**
Use the setup from Test 2 (conditional packages)

**Test sequence:**
```bash
# 1. First launch with python pack
echo '{ pkgs }: { packs = ["python"]; }' > ~/.config/yazelix/yazelix.nix
time nix develop .#devenv --command echo "test"  # ~4s (first eval)

# 2. Second launch, same config
time nix develop .#devenv --command echo "test"  # <100ms (cached!)

# 3. Change config
echo '{ pkgs }: { packs = ["rust"]; }' > ~/.config/yazelix/yazelix.nix
time nix develop .#devenv --command echo "test"  # ~4s (cache invalidated)

# 4. Launch again with same config
time nix develop .#devenv --command echo "test"  # <100ms (cached again)

# 5. Touch flake.nix (no content change)
touch flake.nix
time nix develop .#devenv --command echo "test"  # Should re-eval

# 6. Edit flake.nix (add comment)
echo "# test comment" >> flake.nix
time nix develop .#devenv --command echo "test"  # Should re-eval
```

**Success criteria:**
- Cache works when config unchanged (<100ms)
- Cache invalidates on yazelix.nix changes
- Cache invalidates on flake.nix changes
- Re-caches after invalidation

---

#### Test 7: Full Integration
**Goal:** Test complete Yazelix launch workflow

**Implementation:**
Convert full flake.nix logic to devenv.nix (see Phase 2 for details)

**Test:**
```bash
# Launch via yzx launch
nix develop .#devenv --command nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

# Measure performance
time nix develop .#devenv --command nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

# Run again
time nix develop .#devenv --command nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu
```

**Success criteria:**
- Full Yazelix environment launches successfully
- All tools available (helix, yazi, zellij)
- Configs loaded correctly
- Second launch is instant (<100ms)

---

### PoC Success Criteria Summary

**All of the following must pass:**

1. ✅ Basic packages load correctly
2. ✅ Conditional package selection works (packs system)
3. ✅ nixGL wrappers work on Linux
4. ✅ shellHook/enterShell scripts execute successfully
5. ✅ Cross-platform logic works (Linux/macOS)
6. ✅ Cache invalidation works correctly
7. ✅ Full Yazelix launch workflow works
8. ✅ Performance gains verified (<100ms cached, ~4s uncached)

**If ANY test fails:** Document the issue, attempt workaround, or abort PoC

---

## Phase 2: Full Migration (Only if PoC succeeds)

### Migration Steps

**1. Convert flake.nix to devenv.nix**

Create complete `devenv.nix` with all current logic:

```nix
# devenv.nix
{ pkgs, lib, config, inputs, ... }:

let
  # Import all current logic from flake.nix
  # - Platform detection
  # - Config reading (yazelix.nix)
  # - Pack definitions
  # - Terminal wrappers
  # - All conditional package selection

  # Read configuration
  homeDir = builtins.getEnv "HOME";
  configFile = "${homeDir}/.config/yazelix/yazelix.nix";
  defaultConfigFile = "${homeDir}/.config/yazelix/yazelix_default.nix";

  userConfig =
    if builtins.pathExists configFile then
      import configFile { inherit pkgs; }
    else if builtins.pathExists defaultConfigFile then
      import defaultConfigFile { inherit pkgs; }
    else
      {
        recommended_deps = true;
        yazi_extensions = true;
        yazi_media = true;
        helix_mode = "release";
        default_shell = "nu";
        extra_shells = [];
        debug_mode = false;
        skip_welcome_screen = false;
        enable_sidebar = false;
        packs = [];
        user_packages = [];
        editor_command = "hx";
        helix_runtime_path = null;
      };

  # ... (copy all package definitions, wrappers, conditionals from current flake.nix)

  allPackages =
    essentialDeps ++
    extraShellDeps ++
    (if userConfig.recommended_deps or true then recommendedDeps else []) ++
    (if userConfig.yazi_extensions or true then yaziExtensionsDeps else []) ++
    (if userConfig.yazi_media or true then yaziMediaDeps else []) ++
    packPackages ++
    (userConfig.user_packages or []);
in
{
  # All packages
  packages = allPackages;

  # Environment variables (from current shellHook)
  env = {
    YAZELIX_DIR = "$HOME/.config/yazelix";
    IN_YAZELIX_SHELL = "true";
    YAZELIX_DEBUG_MODE = if userConfig.debug_mode or false then "true" else "false";
    ZELLIJ_DEFAULT_LAYOUT =
      if userConfig.enable_sidebar or false then "yzx_side" else "yzx_no_side";
    YAZELIX_DEFAULT_SHELL = userConfig.default_shell or "nu";
    YAZELIX_ENABLE_SIDEBAR =
      if userConfig.enable_sidebar or false then "true" else "false";
    YAZI_CONFIG_HOME = "$HOME/.local/share/yazelix/configs/yazi";
    YAZELIX_HELIX_MODE = userConfig.helix_mode or "release";
    # ... all other env vars
  };

  # Shell initialization (from current shellHook)
  enterShell = ''
    # Set HELIX_RUNTIME
    export HELIX_RUNTIME="${
      if userConfig.helix_runtime_path or null != null
      then userConfig.helix_runtime_path
      else "${helixPackage}/lib/runtime"
    }"

    # Set EDITOR
    export EDITOR="${editorCommand}"
    echo "📝 Set EDITOR to: ${editorCommand}"

    # Disable Nix warning
    export NIX_CONFIG="warn-dirty = false"

    # Auto-copy config if needed
    if [ ! -f "$YAZELIX_DIR/yazelix.nix" ] && [ -f "$YAZELIX_DIR/yazelix_default.nix" ]; then
      cp "$YAZELIX_DIR/yazelix_default.nix" "$YAZELIX_DIR/yazelix.nix"
      echo "Created yazelix.nix from template"
    fi

    # Run main environment setup
    nu "$YAZELIX_DIR/nushell/scripts/setup/environment.nu" \
      "$YAZELIX_DIR" \
      "${if userConfig.recommended_deps or true then "true" else "false"}" \
      "${if userConfig.enable_atuin or false then "true" else "false"}" \
      "bash,zsh,fish,nu"
  '';
}
```

**2. Update flake.nix to use devenv**

```nix
# flake.nix
outputs = { self, nixpkgs, flake-utils, helix, nixgl, devenv, ... }:
  flake-utils.lib.eachDefaultSystem (system:
    {
      # Use devenv for devShells.default
      devShells.default = devenv.lib.mkShell {
        inherit inputs pkgs;
        modules = [
          ./devenv.nix
        ];
      };
    }
  );
```

**3. Update launch scripts**

Scripts should already work since they use `nix develop`, but verify:

```bash
# Test all launch methods
yzx launch
yzx launch-desktop
nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu
```

**4. Update documentation**

Files to update:
- `README.md` - mention devenv caching
- `docs/installation.md` - note instant launches after first run
- `docs/performance.md` - new file documenting caching behavior
- `docs/devenv_caching_research.md` - mark as "IMPLEMENTED"

**5. Test on both platforms**

- Linux: Full testing
- macOS: If available, or document macOS testing needed

**6. Update Home Manager module**

Ensure `home_manager/module.nix` works with devenv-based Yazelix

---

### Performance Benchmarks

Before and after measurements:

**Current (nix develop with mkShell):**
```bash
# Cold start
time nix develop --impure --command echo "test"
# Expected: ~4-5s

# Second run (no caching)
time nix develop --impure --command echo "test"
# Expected: ~4-5s (same)
```

**After (devenv with caching):**
```bash
# First run (builds cache)
time nix develop --impure --command echo "test"
# Expected: ~4-5s (same as before)

# Second run (uses cache)
time nix develop --impure --command echo "test"
# Expected: <100ms (40-50x faster!)

# After config change
echo '{ pkgs }: { packs = ["python"]; }' > ~/.config/yazelix/yazelix.nix
time nix develop --impure --command echo "test"
# Expected: ~4-5s (cache invalidated, rebuilds)

# Second run with new config
time nix develop --impure --command echo "test"
# Expected: <100ms (cached again)
```

**Success metric:** 40x speedup for cached launches

---

### Migration Checklist

Before merging to main:

- [ ] All PoC tests pass
- [ ] Full Yazelix functionality works
- [ ] Performance benchmarks meet expectations
- [ ] No regressions in features
- [ ] Documentation updated
- [ ] Tested on Linux (required)
- [ ] Tested on macOS (nice to have)
- [ ] Home Manager integration works
- [ ] `yzx doctor` updated if needed
- [ ] Changelog/README updated with devenv info

---

## Phase 3: Rollback Plan (If PoC fails)

### When to Abort

Abort the PoC if:
- devenv can't handle nixGL wrappers
- Cache invalidation doesn't work correctly
- Significant features break
- Performance gains don't materialize (<10x speedup)
- Too complex to maintain

### Rollback Steps

1. Document what didn't work (add to research doc)
2. Delete PoC branch: `git branch -D poc/devenv-caching`
3. Accept 4s as reasonable (Option 5 from research)
4. Update `docs/devenv_caching_research.md` with findings
5. Consider future optimization (Option 4) as time permits

### Fallback Position

**Current approach is good enough:**
- 4s is reasonable for a complex Nix flake
- Full automation is preserved
- Proven reliable
- Users launch once per work session

**The automation value > speed difference**

---

## Technical Notes

### devenv vs mkShell Differences

**Key differences:**
1. **DSL change:** `mkShell` → devenv module system
2. **Packages:** `buildInputs = [...]` → `packages = [...]`
3. **shellHook:** `shellHook = ''...''` → `enterShell = ''...''`
4. **Env vars:** In `shellHook` → `env.VAR = "value"`
5. **Caching:** None → Automatic SQLite tracking

**Compatibility:**
- Most Nix expressions should work as-is
- Custom derivations (wrappers) should work
- Platform conditionals (`stdenv.isLinux`) work
- Config imports (`builtins.pathExists`) work

### Potential Issues

**Issue 1: nixGL complexity**
- devenv might not handle nixGL overlay well
- **Solution:** Test early in PoC (Test 3)

**Issue 2: Complex conditionals**
- Many if/then/else in package selection
- **Solution:** Should work, just Nix evaluation

**Issue 3: Nushell script execution**
- enterShell runs bash by default
- **Solution:** Explicitly call `nu` for scripts

**Issue 4: Cache false positives**
- Cache might not invalidate when it should
- **Solution:** Test thoroughly (Test 6), document edge cases

---

## Timeline

**PoC Phase (2-4 hours):**
- Setup: 30 min
- Test 1-3: 1 hour
- Test 4-6: 1 hour
- Test 7: 30 min
- Decision: 30 min

**Migration Phase (2-4 hours, if PoC succeeds):**
- Convert devenv.nix: 1-2 hours
- Update docs: 30 min
- Testing: 1 hour
- Benchmarking: 30 min

**Total time investment:** 4-8 hours

---

## Success Definition

**PoC Success:**
- All 7 tests pass
- <100ms cached launches verified
- No showstopper issues

**Migration Success:**
- Full Yazelix works with devenv
- 40x speedup on cached launches
- Automatic cache invalidation works
- Zero feature regressions
- Documentation updated

**If successful:** Yazelix gets instant launches + full automation ✨

**If unsuccessful:** Clear documentation of what didn't work, accept 4s as reasonable

---

## Next Steps

1. Review this plan
2. Create PoC branch
3. Start with Test 1 (basic packages)
4. Work through tests incrementally
5. Make go/no-go decision after Test 6
6. If go: Complete Test 7 + full migration
7. If no-go: Document findings, close PoC branch

---

*Plan created: 2025-01-01*
*Estimated effort: 4-8 hours total*
*Risk: Low (PoC approach minimizes wasted effort)*
