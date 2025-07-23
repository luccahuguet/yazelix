# Package Sizes & Dependencies

Control Yazelix's disk usage by enabling/disabling dependency groups in your `yazelix.nix` configuration.

## 📊 Dependency Groups

| Group | Total Size | Status | Description |
|-------|------------|--------|-------------|
| **Essential Tools** | ~200-250MB | Always included | Core Yazelix functionality |
| **Recommended Tools** | ~300-400MB | Default: enabled | Productivity enhancers |
| **Yazi Extensions** | ~100-150MB | Default: enabled | File preview & archive support |
| **Yazi Media** | ~800MB-1.5GB | Default: enabled | Heavy media processing |

**Installation sizes:**
- **Minimal**: ~200-250MB (essential only)
- **Standard**: ~1.3-2.3GB (all groups enabled)

## 📦 Key Package Breakdown

**Essential Tools** (~200-250MB): `zellij` (97MB), `lazygit` (60MB), `helix` (80MB), `yazi` (30MB), plus shell tools

**Recommended Tools** (~300-400MB): `mise` (62MB), `atuin` (38MB), `ripgrep` (52MB), `biome` (45MB), plus dev utilities  

**Yazi Extensions** (~100-150MB): `poppler` (45MB), `p7zip` (10MB), `fd` (8MB), plus preview tools

**Yazi Media** (~800MB-1.5GB): `ffmpeg` (500MB+), `imagemagick` (276MB), plus media processing tools

## ⚙️ Configuration

Edit `yazelix.nix` to control dependency groups:

```nix
{
  recommended_deps = true;   # ~300-400MB
  yazi_extensions = true;    # ~100-150MB  
  yazi_media = true;         # ~800MB-1.5GB
}
```

**Common configurations:**
- **Minimal** (~250MB): Set all to `false`
- **Lightweight** (~650MB): `yazi_media = false`
- **Full** (~2GB): Keep all `true` (default)

## 📝 Notes

Sizes verified with `nix path-info -S` commands. Actual sizes may vary based on system architecture and existing packages. Nix's store deduplication reduces overlap between packages. 