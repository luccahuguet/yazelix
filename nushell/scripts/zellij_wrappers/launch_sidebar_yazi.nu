#!/usr/bin/env nu

use ../integrations/yazi.nu [consume_bootstrap_sidebar_cwd get_yazi_command]

def main [] {
    let bootstrap_dir = (consume_bootstrap_sidebar_cwd)
    let target_dir = if ($bootstrap_dir | is-not-empty) {
        $bootstrap_dir
    } else {
        pwd | path expand
    }
    let yazi_command = (get_yazi_command)
    run-external $yazi_command $target_dir
}
