#!/usr/bin/env nu

const CONSTANTS_DATA_PATH = ((path self | path dirname) | path join "constants_data.json")

def load_constants_data [] {
    open $CONSTANTS_DATA_PATH
}

export def get_terminal_metadata [] {
    (load_constants_data).terminal_metadata
}

export def get_cursor_trail_shaders [] {
    (load_constants_data).cursor_trail_shaders
}

export def get_ghostty_trail_effects [] {
    (load_constants_data).ghostty_trail_effects
}

export def get_ghostty_mode_effects [] {
    (load_constants_data).ghostty_mode_effects
}
