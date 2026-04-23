#!/usr/bin/env nu

const CONSTANTS_DATA_PATH = ((path self | path dirname) | path join "constants_data.json")

def load_constants_data [] {
    open $CONSTANTS_DATA_PATH
}

export def get_shell_initializer_dirs [] {
    (load_constants_data).shell_initializer_dirs
}
