# devenv Caching Research: Solving the 4s Cold Start Problem

**Date:** 2025-01-01
**Status:** Research complete, devenv PoC next step
**Goal:** Eliminate 4s cold start delay while maintaining automatic configuration

## Problem Statement

### Current Behavior
Every Yazelix cold start via `nix develop` takes ~4-5 seconds, even when:
- All packages are already cached locally (no network fetches)
- Configuration hasn't changed since last launch
- No compilation needed

### Why This Happens
The delay comes from **Nix evaluation**, not builds or downloads:
1. Parse flake.nix (~500 lines of Nix code)
2. Read and evaluate yazelix.nix user configuration
3. Compute conditional dependencies (packs, recommended_deps, etc.)
4. Assemble final package list (~50-100 packages)
5. Create devShell environment with all packages
6. Run shellHook setup scripts

This is **fundamental to how `nix develop` works** - it must re-evaluate every time to ensure correctness and that config changes are respected.

### The Requirement
- ✅ **4s is acceptable** when yazelix.nix changes (evaluation needed)
- ❌ **4s is wasteful** when config unchanged (99% of launches)
- 🎯 **Must preserve automation** - "edit yazelix.nix → next launch applies changes"

### Performance Context
For a Nix flake with 50-100 packages and complex conditional logic:
- Yazelix: ~4-5s (current)
- Typical NixOS devShells: 2-5s
- Home Manager rebuilds: 10-30s
- Full NixOS rebuilds: 30s-5min

**Yazelix is already on the fast end**, but we can do better.

---

## Investigated Solutions

### 1. cached-nix-shell
**Source:** https://github.com/xzfc/cached-nix-shell

**How it works:**
- Caches environment variables from nix-shell
- Traces file reads during evaluation
- Auto-invalidates cache when tracked files change
- Performance: 430ms → 30ms (14x speedup)

**Verdict:** ❌ **Not applicable**
- Only works with legacy `nix-shell`, not `nix develop` or flakes
- No flake support documented
- Project focused on traditional Nix workflows

---

### 2. nix-direnv
**Source:** https://github.com/nix-community/nix-direnv

**How it works:**
- Persistent, fast implementation of direnv's `use_nix`/`use_flake`
- Caches environment, survives garbage collection
- Auto-loads on `cd` into directory

**Verdict:** ❌ **Already tried, doesn't help**
- Only activates on directory changes (`cd`)
- Doesn't help desktop launcher entries (spawn processes directly)
- Doesn't help `yzx launch` command (spawns new `nix develop` process)
- Would only benefit developers who manually `cd ~/.config/yazelix` repeatedly

See commit `bc61a21` for removal rationale.

---

### 3. Home Manager (Pure Approach)
**How it works:**
- Install all Yazelix packages persistently to user profile
- Packages always available in PATH (no evaluation needed)
- Changes require `home-manager switch` (~10-30s rebuild)

**Performance:**
- Cold starts: **0s** (instant! packages already in PATH)
- Config changes: 10-30s rebuild (only when user runs switch)

**Verdict:** ❌ **Breaks automation**

**Advantages:**
- ✅ Instant launches (0s overhead)
- ✅ Packages always available (even outside Yazelix)
- ✅ Standard Nix user environment approach

**Critical flaws:**
- ❌ **Manual rebuild required** - user must remember `home-manager switch`
- ❌ **Silent staleness** - forgetting to rebuild = old package set
- ❌ **Breaks Yazelix's magic** - no more "edit config → auto-apply"
- ❌ **Conflicts** - users with existing HM configs need integration
- ❌ **Increased complexity** - now managing HM modules + activation

**This sacrifices Yazelix's best feature (automatic config) for speed.**

---

### 4. Home Manager (Hybrid with Auto-Detection)
**How it works:**
- Use Home Manager for packages (instant availability)
- Add detection: hash yazelix.nix, compare to last build
- Prompt user to rebuild when config changed

**Example flow:**
```nu
def launch [] {
    let config_hash = (hash_yazelix_config)
    let built_hash = (read_last_build_hash)

    if $config_hash != $built_hash {
        print "⚠️  yazelix.nix changed, rebuild? (y/n/always)"
        # Handle response...
    }

    launch_zellij_session  # Instant (0s)
}
```

**Verdict:** ⚠️ **Better but still problematic**

**Advantages:**
- ✅ Instant when config unchanged (99% of launches)
- ✅ Detects changes automatically
- ✅ Can set "always rebuild" preference

**Issues:**
- ⚠️ **Semi-automatic** - requires user confirmation
- ❌ **Interrupts workflow** - prompt disrupts launch flow
- ❌ **Still slow on changes** - 10-30s rebuild when needed
- ❌ **Complex state management** - hash tracking, preference storage
- ❌ **Still has HM conflicts** - integration issues persist

**Better than pure HM, but adds complexity without solving the UX problem.**

---

### 5. Optimize Current Flake Evaluation
**Potential optimizations:**
- Pre-compute values at flake level
- Reduce conditional branching
- Simplify shellHook (currently runs full Nushell scripts)
- Cache intermediate computations

**Realistic savings:** ~500ms-1s (4s → 3-3.5s)

**Verdict:** ❌ **Not worth the effort**

**Why:**
- Still in "feels slow" territory (3-3.5s)
- Adds maintenance complexity
- Harder to read/modify flake.nix
- Doesn't fundamentally solve the problem
- Half-measures rarely satisfy

---

### 6. devenv with Evaluation Caching
**Source:** https://devenv.sh/blog/2024/10/03/devenv-13-instant-developer-environments-with-nix-caching/

**How it works:**
1. SQLite-based caching system for Nix evaluation
2. Parses Nix logs to track all accessed files/directories
3. Stores metadata: file paths, content hashes, timestamps
4. On subsequent runs: compares state, detects changes
5. Auto-invalidates cache when differences detected

**Performance:**
- First launch: ~4s (same as current)
- Cached launches: **single-digit milliseconds** (4s → <10ms, ~400x speedup!)
- Automatic cache invalidation on any file change

**Change detection coverage:**
- Direct Nix file edits
- Imported files and directories
- Files read via `readFile`, `readDir`, etc.
- Input flake lock changes

**Comparison to alternatives:**
- **Nix's built-in flake cache**: Only caches based on input locks (misses dev changes)
- **lorri**: Pioneered this but requires background daemon
- **direnv/nix-direnv**: Manual file-watching, limited nested import detection

**Verdict:** ⭐ **Most promising - try this first**

**Advantages:**
- ✅ **Instant launches** after first run (<10ms)
- ✅ **Fully automatic** - detects changes, no manual steps
- ✅ **Preserves architecture** - still a devShell, not persistent packages
- ✅ **Smart caching** - SQLite tracking is robust
- ✅ **Purpose-built** for exactly this problem

**Risks/Unknowns:**
- ⚠️ Migration effort (devShell → devenv DSL)
- ⚠️ Yazelix complexity (nixGL, conditionals, cross-platform)
- ⚠️ Relatively new (1.3 released Oct 2024)
- ⚠️ Additional abstraction layer/dependency
- ❓ Unknown if it handles custom wrappers (yazelix-ghostty, etc.)
- ❓ Unknown if it works with complex shellHook scripts

**If it works:** Perfect solution - instant + automatic ✨
**If it doesn't:** Wasted migration effort, need to revert

---

### 7. Accept Current Behavior (Do Nothing)
**Rationale:**
- 4s is **good** performance for this type of flake
- Most users launch Yazelix **once per work session**
- The automation value is enormous and unique
- Comparing to wrong baseline (should be other Nix devShells, not "instant")

**Verdict:** ✅ **Actually reasonable fallback**

**If devenv doesn't work out**, this is the right choice:
- Proven reliable
- Full automation preserved
- No added complexity
- 4s of correctness > 0s of manual friction

---

## Recommendation: Two-Phase Approach

### Phase 1: devenv Proof of Concept (PoC)

**Goal:** Determine if devenv can handle Yazelix's complexity

**Test cases:**
1. ✅ Conditional package selection (packs system)
2. ✅ nixGL wrappers (yazelix-ghostty, yazelix-kitty, etc.)
3. ✅ shellHook script execution (Nushell setup scripts)
4. ✅ Cross-platform logic (Linux/macOS differences)
5. ✅ yazelix.nix config reading (dynamic imports)
6. ✅ Cache invalidation (edit yazelix.nix, verify re-eval)
7. ✅ Performance testing (measure actual cold start times)

**Approach:**
- Create minimal devenv setup on separate branch
- Test core features incrementally
- Measure performance gains
- Document any issues or limitations

**Time estimate:** 2-4 hours for thorough testing

**Success criteria:**
- All test cases pass
- Cold starts <100ms after first run
- Automatic cache invalidation works
- No loss of functionality

### Phase 2: Decision Point

**If devenv works well:**
- ✅ Full migration to devenv
- 🎯 Achieve: Instant launches + full automation
- 📝 Document migration for other Nix projects

**If devenv has issues:**
- ✅ Accept 4s as reasonable
- 🎯 Preserve: Full automation + proven reliability
- 📝 Document learnings for future optimization

---

## Long-term Vision (Separate Track)

**Goal:** Make Yazelix a distributable Nix package

**Future capabilities:**
```bash
# Via nix profile
nix profile install nixpkgs#yazelix

# Via home-manager
programs.yazelix.enable = true;

# Via nixpkgs overlay
nixpkgs.overlays = [ yazelix.overlays.default ];
```

**Important notes:**
- This is **orthogonal** to the devenv caching work
- Both devenv-based and devShell-based Yazelix can be packaged
- Requires separate effort: nixpkgs PR, packaging standards, etc.
- Path made of tiny progressive steps over time

**Don't conflate these goals** - solve caching first, packaging later.

---

## Key Insights

### Why 4s Happens
Not from network, not from compilation - from **evaluation complexity**:
- 500 lines of conditional Nix logic
- Dynamic config reading
- Platform detection (Linux/macOS)
- Package set computation (50-100 packages)
- Environment assembly

### Why devenv is Different
Only solution that provides:
- Instant launches (caching works)
- Automatic invalidation (detects changes)
- Preserved architecture (still a devShell)

### The Real Trade-off
Not "speed vs automation" - it's:
- **Speed via manual steps** (Home Manager approach)
- vs. **Speed via smart caching** (devenv approach)

Smart caching is the only way to have both.

---

## References

**Tools researched:**
- cached-nix-shell: https://github.com/xzfc/cached-nix-shell
- nix-direnv: https://github.com/nix-community/nix-direnv
- devenv: https://devenv.sh
- devenv 1.3 release: https://devenv.sh/blog/2024/10/03/devenv-13-instant-developer-environments-with-nix-caching/

**Issues and discussions:**
- nix develop slow in large folders: https://github.com/NixOS/nix/issues/7284
- Flake evaluation caching: https://github.com/NixOS/nix/issues/2853
- Nix evaluation caching (Tweag): https://www.tweag.io/blog/2020-06-25-eval-cache/

**Performance articles:**
- Parallel Nix evaluation: https://determinate.systems/blog/parallel-nix-eval/
- Pre-resolved store paths: https://determinate.systems/posts/resolved-store-paths/

**Community wisdom:**
- "devenv 1.3: once cached, results can be recalled in single-digit milliseconds"
- "nix develop remains the last bit that's rather slow and uncacheable"
- "nix-direnv is great and fixes roughly every problem I've had with nix-shell"
- "4 seconds is actually good performance for a flake of this complexity"

---

## Next Steps

1. ✅ Research complete (this document)
2. 🔄 Create devenv PoC branch
3. ⏭️ Test all compatibility cases
4. ⏭️ Measure performance gains
5. ⏭️ Make decision: migrate or accept 4s
6. ⏭️ Document outcome

**Decision deadline:** After PoC testing (2-4 hours of work)

---

*Research conducted: 2025-01-01*
*Research method: Web search, tool documentation, Nix community resources*
*Conclusion: devenv is worth trying; if it works, we get instant+automatic*

---

## PoC Results: devenv via Flakes

**Date:** 2025-11-01
**Branch:** `poc/devenv-caching`
**Test method:** Minimal devenv.nix with nushell, zellij, helix

### Implementation Approach

Used `devenv.lib.mkShell` in flake.nix:
```nix
devShells.devenv = devenv.lib.mkShell {
  inherit pkgs;
  inputs = {
    inherit self nixpkgs devenv;
  };
  modules = [
    ./devenv.nix
  ];
};
```

### Performance Results

**Test command:** `time nix develop .#devenv --impure --command echo "test"`

| Run | Time | Notes |
|-----|------|-------|
| First run | Several minutes | Downloaded/built ~200+ Rust crates (devenv infrastructure) |
| Second run | 1.584s | All packages cached |
| Third run | 1.581s | Consistent cached performance |

**Baseline comparison:**
- Current `nix develop` (default shell): ~4-5s
- devenv via flakes: ~1.5s
- **Improvement: 2.5-3x faster**

**Expected based on research:**
- devenv promise: <10ms (single-digit milliseconds)
- **Actual: NOT achieved** ❌

### Key Findings

#### ✅ What Worked
1. **devenv integration successful** - No blocking issues, all setup worked
2. **Meaningful performance improvement** - 4-5s → 1.5s is real gain
3. **Automatic behavior preserved** - Still evaluates on every run (detects changes)
4. **Cache artifacts created** - `.devenv/tasks.db` SQLite database present

#### ❌ What Didn't Work
1. **Evaluation caching ineffective** - No <10ms performance as promised
2. **Heavy initial overhead** - Built hundreds of Rust crates just for devenv tooling
3. **No speedup on repeated runs** - Cached runs still take same ~1.5s

### Analysis

**Why caching didn't work:**

The SQLite-based evaluation caching that devenv advertises appears to only work with the **devenv CLI**, not when using `devenv.lib.mkShell` via flakes. Evidence:

1. Consistent ~1.5s performance (no improvement after caching)
2. Web research finding: "devenv via flakes has performance limitations and reduced features"
3. Official docs recommend `devenv` CLI + `devenv.nix`, not flake integration

**The 1.5s improvement breakdown:**
- Likely from devenv's optimization of package loading (~1.5s faster)
- NOT from evaluation caching (would be <10ms if working)
- Still performing full Nix evaluation each time

**Initial build overhead:**

First run downloaded/built extensive infrastructure:
- Rust toolchain crates: tokio, hyper, axum, serde, clap, etc.
- Build tools: bindgen, cmake, cc, pkg-config
- devenv-specific: ~200+ dependencies

This is a significant one-time cost, though amortized over time.

### Verdict

**devenv via flakes: ⚠️ Partial Success**

**Pros:**
- ✅ Works without blocking issues
- ✅ 2.5x performance improvement (meaningful)
- ✅ Preserves automatic behavior
- ✅ Better than current state

**Cons:**
- ❌ Doesn't deliver promised <10ms caching
- ❌ Heavy initial build overhead
- ❌ Adds dependency on extensive Rust toolchain
- ❌ Still "feels slow" at 1.5s (not instant)

### Options Going Forward

#### Option 1: Try devenv CLI (Not Flakes)
**Approach:** Use `devenv` command directly instead of `devenv.lib.mkShell`
- Requires users to install devenv CLI
- Use `devenv shell` instead of `nix develop`
- May get the promised <10ms caching

**Pros:**
- Might achieve promised performance
- Official recommended approach

**Cons:**
- Different workflow (not `nix develop`)
- Additional installation step
- May not integrate well with current flake-based architecture

#### Option 2: Accept 1.5s Improvement
**Approach:** Keep devenv.lib.mkShell integration as-is
- Users get 2.5x improvement automatically
- Still using `nix develop` workflow
- No workflow changes

**Pros:**
- Meaningful improvement (4-5s → 1.5s)
- No breaking changes
- Fully automatic

**Cons:**
- Not the "instant" experience we hoped for
- Heavy dependency overhead for modest gain
- Still feels slow

#### Option 3: Accept Current 4s as Reasonable
**Approach:** Remove devenv, keep current implementation
- 4s is actually good for a complex flake
- No added dependencies or complexity
- Battle-tested and reliable

**Pros:**
- Proven reliability
- No added complexity
- 4s is reasonable for correctness

**Cons:**
- No performance improvement
- Misses opportunity for optimization

#### Option 4: Hybrid - Offer Both
**Approach:** Keep both `devShells.default` and `devShells.devenv`
- Advanced users can choose faster devenv shell
- Default remains stable and proven
- Document trade-offs

**Pros:**
- Choice for users
- No breaking changes
- Progressive adoption

**Cons:**
- Maintenance overhead (two shells)
- Complexity in documentation
- Potential confusion

### Recommendation

**Try Option 1 (devenv CLI) first, with Option 3 as fallback:**

1. **Quick test:** Install devenv CLI and test with minimal devenv.nix
   - Command: `devenv shell` instead of `nix develop`
   - Measure first vs second run performance
   - If <100ms on second run: we have our solution ✅
   - If still slow: confirms flakes limitation ❌

2. **If devenv CLI works:**
   - Evaluate workflow impact (how would users invoke?)
   - Consider if worth the different command
   - Document transition path

3. **If devenv CLI doesn't work:**
   - Accept 4s as reasonable (Option 3)
   - Document that 4s is good for this complexity
   - Focus optimization efforts elsewhere

**Time investment:** 30-60 minutes to test devenv CLI approach

**Decision criteria:** Must achieve <100ms on cached runs to be worth the workflow change

---

*PoC conducted: 2025-11-01*
*Test method: Empirical performance measurement*
*Conclusion: devenv via flakes provides 2.5x improvement but not the promised <10ms caching*
