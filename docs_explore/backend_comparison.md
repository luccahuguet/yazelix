# Backend Comparison: devenv vs mise vs pixi vs cargo-binstall (tentative)

| Feature | devenv | mise | pixi | cargo-binstall |
|---------|--------|------|------|----------------|
| **Setup** |
| Written in | Nix | Rust | Rust | Rust |
| Disk usage | ~9.7GB¹ | ~1-2GB² | ~1-2GB² | ~500MB² |
| First install | Minutes | ~1-2 min | ~1-2 min | ~1-2 min |
| Tool call overhead | ~0ms | ~5-10ms³ | ~5-10ms³ | ~0ms |
| Admin rights | Yes | No | No | No |
| Reproducibility | Excellent | Good (lock file) | Excellent (lock file) | Manual |
| Rollback | Built-in | Manual | Manual | Manual |
| **Isolation Model** |
| Isolated shell | ✓ (`devenv shell`) | ✗ (global installs) | ✓ (`pixi shell`) | ✗ (global installs) |
| No global pollution | ✓ | ✗ | ✓ | ✗ |
| Fast cached launch | ✓ (~50ms) | N/A | ? | N/A |
| Rebuild on change | ~7s | Instant | ? | Instant |
| **Core Tools** |
| helix, yazi, zellij, nushell | ✓ | ✓ | ✓ | ✓ |
| fzf, zoxide, starship | ✓ | ✓ | ✓ | ✗ fzf (Go) |
| lazygit, atuin, carapace | ✓ | ✓ | ✓ | ✗ lazygit, carapace (Go) |
| **Yazi Features** |
| fd, ripgrep, jq | ✓ | ✓ | ✓ | ✗ jq (C) |
| p7zip, poppler | ✓ | System | ✓ | System |
| ffmpeg, imagemagick | ✓ | System | ✓ | System |
| **Advanced** |
| Terminal management | ✓ | ✗ | ✗ | ✗ |
| nixGL support | ✓ | ✗ | ✗ | ✗ |
| Helix source builds | ✓ | ✗ | ✗ | ✓⁴ |
| Desktop entry | ✓ | Manual | Manual | Manual |
| Language packs | ✓ | Limited | ✓ (conda) | ✗ |
| zjstatus | ✓ | Download | Download | Download |
| **yzx CLI Support** |
| yzx launch | ✓ | Partial⁵ | Partial⁵ | Partial⁵ |
| yzx env / run | ✓ | ✓ | ✓ | ✓ |
| yzx doctor | ✓ | Adapt | Adapt | Adapt |
| yzx update | ✓ | Adapt⁶ | Adapt⁶ | Adapt⁶ |
| yzx bench / profile | ✓ | ✓ | ✓ | ✓ |
| yzx versions | ✓ | ✓ | ✓ | ✓ |

¹ Nix 2.5GB + devenv CLI 5GB + yazelix tools 2.2GB
² Depends on tools installed
³ mise/pixi use PATH modification, not shims - overhead only on directory change
⁴ Use `cargo install helix-term` instead of binstall (slow, compiles from source)
⁵ No terminal wrappers - `yzx launch` works but without managed terminals
⁶ Update commands adapt: `mise upgrade` / `pixi update` / `cargo install-update -a`

## When to Use Each

**devenv (default)** — Full experience, maximum reproducibility
- All features work out of the box
- Managed terminals with nixGL
- Build helix from source
- Easy rollback to previous states

**mise** — Lightweight, familiar tooling
- No admin rights needed
- Minimal disk footprint
- Fast setup, uses GitHub releases
- Good for users already using mise/asdf

**pixi** — Best of both worlds
- No admin rights needed
- conda-forge has everything (ffmpeg, p7zip, etc.)
- Excellent lock file reproducibility
- Great for data science / Python users

**cargo-binstall** — Rust minimalist
- Smallest disk usage
- Only need core Rust tools
- Already have Rust toolchain
- OK with system-installing Go/C tools

## Global Score

| Criteria | devenv | mise | pixi | cargo-binstall |
|----------|--------|------|------|----------------|
| Feature completeness | 10 | 6 | 7 | 4 |
| yzx CLI integration | 10 | 7 | 7 | 6 |
| Isolated shell model | 10 | 3 | 9 | 3 |
| Ease of setup | 5 | 9 | 8 | 7 |
| Disk efficiency | 4 | 8 | 8 | 10 |
| Reproducibility | 10 | 6 | 9 | 3 |
| Package ecosystem | 10 | 7 | 9 | 5 |
| Caching / incremental | 10 | 6 | 7 | 5 |
| Portability (no admin) | 0 | 10 | 10 | 10 |
| **Total** | **69/90** | **62/90** | **74/90** | **53/90** |

- **pixi** scores highest: isolated shell (`pixi shell`), broad ecosystem, no admin needed
- **devenv** second: only option with 100% yazelix features (terminals, nixGL, fast caching)
- **mise** third: installs globally, pollutes user environment
- **cargo-binstall** last: global installs + limited to Rust tools

## Recommendation: Next Experiment

**Pick: pixi** (using `pixi shell`, not `pixi global`)

Reasons:
1. **Highest score** (74/90) - best overall balance
2. **Isolated shell model** - `pixi shell` spawns subshell like devenv, doesn't pollute global env
3. **No system installs** - conda-forge has p7zip, ffmpeg, poppler, imagemagick
4. **Lock file reproducibility** - `pixi.lock` like devenv.lock
5. **Growing fast** - active development, good community

Why not mise/cargo-binstall: They install tools globally to `~/.local/share/mise` or `~/.cargo/bin`, polluting the user's environment. yazelix should be self-contained.

Suggested experiment:
```bash
# Install pixi
curl -fsSL https://pixi.sh/install.sh | bash

# Create yazelix pixi environment (isolated, not global)
cd ~/.config/yazelix
pixi init --format pixi
pixi add helix yazi zellij nushell fzf zoxide starship lazygit

# Enter isolated shell (like devenv shell)
pixi shell

# Tools only available inside this shell
hx --version
```

Key question to test: How fast is `pixi shell` on subsequent launches? devenv achieves ~50ms.

## Sources

- [mise vs asdf comparison](https://mise.jdx.dev/dev-tools/comparison-to-asdf.html)
- [pixi global tools](https://prefix.dev/blog/pixi_global)
- [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)
- [Nix storage optimization](https://wiki.nixos.org/wiki/Storage_optimization)
