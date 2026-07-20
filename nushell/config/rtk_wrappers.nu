# One packaged dispatch point for RTK-routed commands.
#
# This module lives beside the profile-owned Nushell config so editable host
# config can import it from ~/.nix-profile without consulting legacy wrappers.
export def --wrapped codex [...rest] {
    ^rtk codex ...$rest
}

export def --wrapped cargo [...rest] {
    ^rtk cargo ...$rest
}
