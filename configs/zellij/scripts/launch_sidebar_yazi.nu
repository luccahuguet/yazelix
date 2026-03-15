#!/usr/bin/env nu

use ~/.config/yazelix/nushell/scripts/integrations/yazi.nu consume_bootstrap_sidebar_cwd

let bootstrap_dir = (consume_bootstrap_sidebar_cwd)
if ($bootstrap_dir | is-not-empty) {
    ^yazi $bootstrap_dir
} else {
    ^yazi
}
