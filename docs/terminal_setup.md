# Terminal Setup

This guide shows you how to set up Yazelix so you can launch it from any terminal using the `yazelix` or `yzx` commands. It will open your configured terminal (Ghostty or WezTerm) with the Yazelix configuration and start Zellij with the integrated Yazi sidebar and Helix editor

## Setup Process

### 1. Complete Prerequisites
Make sure you've completed steps 1-2 from the main installation guide:
- Install Nix package manager
- Download Yazelix repository

### 2. Run the Setup Script
Execute the launch script to install and configure everything:
```bash
chmod +x ~/.config/yazelix/bash/launch-yazelix.sh
~/.config/yazelix/bash/launch-yazelix.sh
```

### What This Does
- Adds convenient `yazelix` and `yzx` aliases to your shell configuration
- Detects and launches your preferred terminal (Ghostty or WezTerm) with Yazelix
- **Note**: This may take quite a few minutes on first run as Nix downloads and builds everything, but following runs will be near instant

## Using Yazelix

After setup is complete, you can launch Yazelix from any terminal:

```bash
yazelix  # or yzx for short
```

## Troubleshooting

**"yazelix: command not found"**
- Restart your terminal completely
