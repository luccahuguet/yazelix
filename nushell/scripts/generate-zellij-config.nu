#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/generate-zellij-config.nu

# Generate Zellij configuration using built-in commands
# This creates a minimal config with Yazelix-specific settings

# Get the default config from Zellij
# Note: You can replace this with your own existing config file:
#   let default_config = (open ~/.config/zellij/config.kdl)
let default_config = (zellij setup --dump-config)

# Read Yazelix-specific overrides
# Note: You can edit the yazelix-overrides.kdl file to add your own settings as well
let yazelix_overrides = (open ~/.config/yazelix/zellij/yazelix-overrides.kdl)

# Combine default config with Yazelix overrides
let yazelix_config = $default_config + "\n\n" + $yazelix_overrides

# Generate config file
$yazelix_config | save -f ~/.config/yazelix/zellij/config.kdl

echo "Zellij configuration generated successfully!"
echo "- config.kdl: Default config with Yazelix overrides from yazelix-overrides.kdl"