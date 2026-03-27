#!/usr/bin/env nu

use ../../../nushell/scripts/integrations/yazi.nu consume_bootstrap_sidebar_cwd

let bootstrap_dir = (consume_bootstrap_sidebar_cwd)
let target_dir = if ($bootstrap_dir | is-not-empty) {
    $bootstrap_dir
} else {
    pwd | path expand
}

^yazi $target_dir
