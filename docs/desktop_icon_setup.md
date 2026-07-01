# Desktop Icon Setup

## Standard Setup

The normal Linux desktop setup path is:

```bash
yzx desktop install
```

That command writes the user-local desktop entry, installs the Yazelix logo in
the standard hicolor icon sizes, and refreshes the desktop and icon caches when
the host provides the relevant tools.

For Home Manager installs, do not run `yzx desktop install`. The Home Manager
module owns the profile desktop entry and installs the icon assets declaratively.

## Manual Fallback

Use this only when diagnosing desktop integration manually. The icon source is
the shipped runtime tree at `$YAZELIX_RUNTIME_DIR/assets/icons`, not
`~/.config/yazelix/assets/icons`. Run the commands from `yzx env` or another
shell where `YAZELIX_RUNTIME_DIR` points at the active Yazelix runtime.

For crisp icon display, install the Yazelix logo in multiple sizes.

### Nushell
```nu
let runtime_dir = $env.YAZELIX_RUNTIME_DIR
let data_home = ($env.XDG_DATA_HOME? | default $"($env.HOME)/.local/share")
let icon_root = $"($data_home)/icons/hicolor"
let applications_dir = $"($data_home)/applications"

# Create icon directories
for size in [48 64 128 256] {
  mkdir $"($icon_root)/($size)x($size)/apps"
}

# Copy pre-generated icon sizes
for size in [48 64 128 256] {
  cp $"($runtime_dir)/assets/icons/($size)x($size)/yazelix.png" $"($icon_root)/($size)x($size)/apps/yazelix.png"
}

# Create icon theme index
"[Icon Theme]
Name=Hicolor
Directories=48x48/apps,64x64/apps,128x128/apps,256x256/apps" | save -f $"($icon_root)/index.theme"

# Update caches
try { ^gtk-update-icon-cache --force --ignore-theme-index $icon_root }
try { ^update-desktop-database $applications_dir }
```

### Bash/Zsh
```bash
set -eu

: "${YAZELIX_RUNTIME_DIR:?Run this from yzx env, or export YAZELIX_RUNTIME_DIR to the active Yazelix runtime.}"

data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
icon_root="$data_home/icons/hicolor"
applications_dir="$data_home/applications"

# Create icon directories
mkdir -p "$icon_root"/{48x48,64x64,128x128,256x256}/apps

# Copy pre-generated icon sizes
for size in 48 64 128 256; do
  cp "$YAZELIX_RUNTIME_DIR/assets/icons/${size}x${size}/yazelix.png" "$icon_root/${size}x${size}/apps/yazelix.png"
done

# Create icon theme index
cat > "$icon_root/index.theme" << 'EOF'
[Icon Theme]
Name=Hicolor
Directories=48x48/apps,64x64/apps,128x128/apps,256x256/apps
EOF

# Update caches
gtk-update-icon-cache --force --ignore-theme-index "$icon_root" 2>/dev/null || true
update-desktop-database "$applications_dir" 2>/dev/null || true
```
