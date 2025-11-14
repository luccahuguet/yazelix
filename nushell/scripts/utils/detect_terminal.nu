#!/usr/bin/env nu
# Detect the current terminal emulator

# Priority 1: Yazelix-set terminal (when launched via yzx launch)
if ($env.YAZELIX_TERMINAL? | is-not-empty) {
    print $env.YAZELIX_TERMINAL
    exit
}

# Priority 2: TERM_PROGRAM (works for many modern terminals)
if ($env.TERM_PROGRAM? | is-not-empty) {
    print ($env.TERM_PROGRAM | str downcase)
    exit
}

# Priority 3: Terminal-specific variables
if ($env.KITTY_WINDOW_ID? | is-not-empty) {
    print "kitty"
    exit
}

if ($env.WEZTERM_EXECUTABLE? | is-not-empty) {
    print "wezterm"
    exit
}

if ($env.ALACRITTY_SOCKET? | is-not-empty) {
    print "alacritty"
    exit
}

if ($env.GHOSTTY_BIN_DIR? | is-not-empty) {
    print "ghostty"
    exit
}

# Check TERM for foot (foot sets TERM=foot or TERM=foot-extra)
if ($env.TERM? | is-not-empty) {
    if ($env.TERM | str starts-with "foot") {
        print "foot"
        exit
    }
}

# Priority 4: Desktop environment terminal (Cosmic, GNOME, etc.)
if ($env.XDG_CURRENT_DESKTOP? | is-not-empty) {
    if ($env.XDG_CURRENT_DESKTOP | str contains -i "cosmic") {
        print "cosmic-term"
        exit
    }
}

# Priority 5: Fallback to preferred
if ($env.YAZELIX_PREFERRED_TERMINAL? | is-not-empty) {
    print $env.YAZELIX_PREFERRED_TERMINAL
    exit
}

# Last resort
print "unknown"
