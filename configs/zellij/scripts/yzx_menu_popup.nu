#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

use ~/.config/yazelix/nushell/scripts/core/yazelix.nu *

$env.YAZELIX_MENU_POPUP = "true"
yzx menu
