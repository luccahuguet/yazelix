# Yazi Flavors (Themes)

This directory contains 24 bundled Yazi color themes (flavors) for use with Yazelix.

## Usage

Configure your theme in `~/.config/yazelix/yazelix.toml`:

```toml
[yazi]
theme = "dracula"  # or any flavor name below
# theme = "random"  # picks a different theme on each restart
```

## Bundled Flavors (24 Total)

### Official (Maintained by yazi-rs)
- catppuccin-frappe
- catppuccin-latte
- catppuccin-macchiato
- catppuccin-mocha
- dracula

### Community-Maintained
- ashen
- ayu-dark
- bluloco-dark
- bluloco-light
- everforest-medium
- flexoki-dark
- flexoki-light
- gruvbox-dark
- kanagawa
- kanagawa-dragon
- kanagawa-lotus
- monokai
- neon
- nord
- rose-pine
- rose-pine-dawn
- rose-pine-moon
- synthwave84
- tokyo-night

Plus the built-in **default** theme (25 total options).

## Attribution

All flavors are created and maintained by their respective authors:

- **Official flavors**: https://github.com/yazi-rs/flavors
- **tokyo-night**: https://github.com/BennyOe/tokyo-night.yazi
- **kanagawa**: https://github.com/dangooddd/kanagawa.yazi
- **kanagawa-dragon**: https://github.com/marcosvnmelo/kanagawa-dragon.yazi
- **kanagawa-lotus**: https://github.com/muratoffalex/kanagawa-lotus.yazi
- **gruvbox-dark**: https://github.com/bennyyip/gruvbox-dark.yazi
- **ayu-dark**: https://github.com/kmlupreti/ayu-dark.yazi
- **everforest-medium**: https://github.com/Chromium-3-Oxide/everforest-medium.yazi
- **ashen**: https://github.com/ficcdaf/ashen
- **flexoki-dark/light**: https://github.com/gosxrgxx
- **rose-pine variants**: https://github.com/Mintass
- **neon**: https://github.com/tomer-ben-david/neon.yazi
- **nord**: https://github.com/AdithyanA2005/nord.yazi
- **synthwave84**: https://github.com/Miuzarte/synthwave84.yazi
- **bluloco-dark/light**: https://github.com/hankertrix/bluloco-yazi
- **monokai**: https://github.com/Malick-Tammal/monokai.yazi

These flavors are vendored with Yazelix for out-of-the-box functionality. All credit goes to the original authors.

**Note:** Yazelix bundles only the essential `flavor.toml` files from each theme to keep the repository lean. Preview images, licenses, and other supplementary files are omitted but available in the original repositories linked above.

## Installing Additional Flavors

To install additional community flavors not bundled with Yazelix:

```bash
ya pkg add <author>/<flavor-name>
```

Browse available flavors at: https://github.com/yazi-rs/flavors
