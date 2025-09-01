# Desktop Icon Setup

## Standard Setup (All Desktop Environments)

For crisp icon display, install the Yazelix logo in multiple sizes:

### Nushell
```nu
# Create icon directories
mkdir ~/.local/share/icons/hicolor/48x48/apps ~/.local/share/icons/hicolor/64x64/apps ~/.local/share/icons/hicolor/128x128/apps ~/.local/share/icons/hicolor/256x256/apps

# Copy pre-generated icon sizes
for size in [48 64 128 256] {
  cp $"~/.config/yazelix/assets/icons/($size)x($size)/yazelix.png" $"~/.local/share/icons/hicolor/($size)x($size)/apps/"
}

# Create icon theme index
"[Icon Theme]
Name=Hicolor
Directories=48x48/apps,64x64/apps,128x128/apps,256x256/apps" | save ~/.local/share/icons/hicolor/index.theme

# Update caches
try { ^gtk-update-icon-cache ~/.local/share/icons/hicolor/ }
try { ^update-desktop-database ~/.local/share/applications }
```

### Bash/Zsh
```bash
# Create icon directories
mkdir -p ~/.local/share/icons/hicolor/{48x48,64x64,128x128,256x256}/apps

# Copy pre-generated icon sizes
for size in 48 64 128 256; do
  cp ~/.config/yazelix/assets/icons/${size}x${size}/yazelix.png ~/.local/share/icons/hicolor/${size}x${size}/apps/
done

# Create icon theme index
cat > ~/.local/share/icons/hicolor/index.theme << 'EOF'
[Icon Theme]
Name=Hicolor
Directories=48x48/apps,64x64/apps,128x128/apps,256x256/apps
EOF

# Update caches
gtk-update-icon-cache ~/.local/share/icons/hicolor/ 2>/dev/null || true
update-desktop-database ~/.local/share/applications
```

