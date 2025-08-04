# Yazelix: Yazi Configuration

Yazelix provides a two-layer Yazi configuration system that prevents git conflicts when customizing Yazi settings.

## Quick Start

```bash
# Copy templates to create your personal configs (one time setup)
cp -r configs/yazi/user configs/yazi/personal

# Edit files in configs/yazi/personal/ to customize Yazi
# Your settings automatically override Yazelix defaults
```

## Documentation

For complete configuration guide, see: [Yazi Configuration Documentation](../../docs/yazi-configuration.md)

## Current Defaults

- Layout ratio optimized for sidebar mode (20% terminal width)  
- Git integration showing file status
- Editor integration with Zellij
- Custom status bar (courtesy of Yazi's creator!)
