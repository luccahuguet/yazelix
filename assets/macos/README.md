# Yazelix macOS Application Bundle

This directory contains the Yazelix.app bundle for macOS integration with Spotlight, Launchpad, and the Dock.

## Quick Installation

```bash
# Copy the app bundle to your Applications folder
cp -r ~/.config/yazelix/assets/macos/Yazelix.app /Applications/

# Optional: Create the .icns icon (requires macOS)
nu ~/.config/yazelix/assets/macos/create_icns.nu
```

After installation, you can:
- Search for "Yazelix" in Spotlight (Cmd+Space)
- Find it in Launchpad
- Add it to your Dock
- Set up keyboard shortcuts in System Settings

## What's Included

### Yazelix.app Bundle Structure
```
Yazelix.app/
├── Contents/
│   ├── Info.plist          # App metadata
│   ├── MacOS/
│   │   └── yazelix         # Launcher script
│   └── Resources/
│       └── yazelix.icns    # App icon (created by create_icns.sh)
```

### Info.plist
Contains app metadata:
- Bundle identifier: `com.yazelix.Yazelix`
- Category: Developer Tools
- Minimum macOS: 10.15 (Catalina)
- High resolution support enabled

### Launcher Script
The `MacOS/yazelix` script:
1. Sources shell profiles (.bash_profile, .profile, .zprofile)
2. Ensures Nix and devenv are in PATH
3. Executes the desktop launcher script

## Icon Creation (Optional)

The `create_icns.nu` script converts the PNG icons into macOS .icns format:

```bash
nu ~/.config/yazelix/assets/macos/create_icns.nu
```

**Requirements:**
- macOS system with `iconutil` command (built-in)
- PNG icon files in `assets/icons/` directory
- Nushell (already installed as part of yazelix)

The app works without the icon, but having it provides a better visual experience.

## Keyboard Shortcuts

To set up a global keyboard shortcut:

1. Open **System Settings** → **Keyboard** → **Keyboard Shortcuts**
2. Select **App Shortcuts** in the sidebar
3. Click the **+** button
4. Select "Yazelix" from the application list
5. Set your preferred keyboard shortcut

## Troubleshooting

### App doesn't appear in Spotlight
- Rebuild Spotlight index: `sudo mdutil -E /Applications`
- Wait a few minutes for reindexing to complete

### "App is damaged" or Gatekeeper warning
```bash
# Remove extended attributes that trigger Gatekeeper
xattr -cr /Applications/Yazelix.app
```

### Nix/devenv not found
Ensure your shell profile files (.bash_profile, .zprofile, etc.) properly source the Nix environment:
```bash
# Usually added by Nix installer
if [ -e /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh ]; then
  . /nix/var/nix/profiles/default/etc/profile.d/nix-daemon.sh
fi
```

### Want to use different terminal/shell
Edit your `~/.config/yazelix/yazelix.toml` configuration:
- `preferred_terminal` - Choose terminal emulator
- `default_shell` - Choose shell environment

## Notes

- The .app bundle is just a launcher - the actual yazelix code lives in `~/.config/yazelix`
- Updates to yazelix (via git pull) don't require reinstalling the .app
- The bundle identifier matches the Linux desktop entry for consistency
