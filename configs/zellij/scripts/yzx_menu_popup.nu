#!/usr/bin/env nu
# Wrapper script for yzx menu popup (called from Zellij keybind)

use ../../../nushell/scripts/core/yazelix.nu *

$env.YAZELIX_MENU_POPUP = "true"
yzx menu
