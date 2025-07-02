#!/usr/bin/env bash
# ~/.config/yazelix/bash/launch-yazelix.sh
# Bash wrapper for the Nushell launcher

# Resolve HOME using shell expansion
HOME=$(eval echo ~)
if [ -z "$HOME" ] || [ ! -d "$HOME" ]; then
  echo "Error: Cannot resolve HOME directory"
  exit 1
fi

# Call the Nushell launcher
nu "$HOME/.config/yazelix/nushell/scripts/launch-yazelix.nu"
