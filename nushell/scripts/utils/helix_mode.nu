#!/usr/bin/env nu
# Helix binary helpers for Yazelix

# Get the appropriate Helix binary path from environment
# Note: This assumes EDITOR is set to a Helix binary
export def get_helix_binary [] {
    let managed_binary = ($env.YAZELIX_MANAGED_HELIX_BINARY? | default "" | into string | str trim)
    if ($managed_binary | is-not-empty) {
        return $managed_binary
    }

    # Only return EDITOR if it's actually Helix, fallback to 'hx' for safety
    let editor = ($env.EDITOR? | default "" | into string | str trim)
    if ($editor | is-empty) {
        return "hx"
    }
    let is_helix = ($editor | str ends-with "/hx") or ($editor == "hx") or ($editor | str ends-with "/helix") or ($editor == "helix")
    if $is_helix {
        $editor
    } else {
        "hx"  # Fallback for non-Helix editors
    }
}
