# Patchy Helix Integration

Yazelix integrates with [patchy](https://github.com/nik-rev/patchy) to build Helix with community pull requests, giving you access to cutting-edge features.

## Quick Start

1. **Enable patchy:**
   ```nix
   # Edit ~/.config/yazelix/yazelix.nix
   use_patchy_helix = true;
   ```

2. **Restart Yazelix:**
   ```bash
   yazelix
   ```

## Default PRs

Yazelix includes these community PRs by default:

- **#12309**: Syntax highlighting for nginx files
- **#8908**: Global status line
- **#13197**: Welcome Screen
- **#11700**: Per-view search location and total matches
- **#11497**: Rounded corners for borders
- **#13133**: Inline Git Blame

## Commands

- `yazelix_patchy status` - Show current status
- `yazelix_patchy list` - List configured PRs
- `yazelix_patchy sync` - Sync and rebuild PRs
- `yazelix_patchy clean` - Clean patchy directory

## Adding More PRs

Edit your `yazelix.nix`:

```nix
patchy_helix_config = {
  pull_requests = [
    "12309"   # default PRs...
    "8908"
    "13197"
    "11700"
    "11497"
    "13133"
    "15432"   # your additional PR
  ];
};
```

## Troubleshooting

- **Build failures**: Try `yazelix_patchy clean` then restart Yazelix
- **Command not found**: Ensure `use_patchy_helix = true` in your config
- **To disable**: Set `use_patchy_helix = false` and restart 