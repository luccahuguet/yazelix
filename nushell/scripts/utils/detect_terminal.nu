#!/usr/bin/env nu
# Detect the current terminal emulator

export def detect_terminal_name [] {
    # Priority 1: Yazelix-set terminal (when launched via yzx launch)
    if ($env.YAZELIX_TERMINAL? | is-not-empty) {
        return $env.YAZELIX_TERMINAL
    }

    # Priority 2: TERM_PROGRAM (works for many modern terminals)
    if ($env.TERM_PROGRAM? | is-not-empty) {
        return ($env.TERM_PROGRAM | str downcase)
    }

    # Priority 3: Terminal-specific variables
    if ($env.KITTY_WINDOW_ID? | is-not-empty) {
        return "kitty"
    }

    if ($env.WEZTERM_EXECUTABLE? | is-not-empty) {
        return "wezterm"
    }

    if ($env.ALACRITTY_SOCKET? | is-not-empty) {
        return "alacritty"
    }

    if ($env.GHOSTTY_BIN_DIR? | is-not-empty) {
        return "ghostty"
    }

    # Check TERM for foot (foot sets TERM=foot or TERM=foot-extra)
    if ($env.TERM? | is-not-empty) {
        if ($env.TERM | str starts-with "foot") {
            return "foot"
        }
    }

    # Priority 4: Desktop environment terminal (Cosmic, GNOME, etc.)
    if ($env.XDG_CURRENT_DESKTOP? | is-not-empty) {
        if ($env.XDG_CURRENT_DESKTOP | str contains -i "cosmic") {
            return "cosmic-term"
        }
    }

    # Priority 5: Fallback to preferred
    if ($env.YAZELIX_PREFERRED_TERMINAL? | is-not-empty) {
        return $env.YAZELIX_PREFERRED_TERMINAL
    }

    # Last resort
    "unknown"
}

export def main [] {
    detect_terminal_name
}
