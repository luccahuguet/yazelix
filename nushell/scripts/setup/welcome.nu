#!/usr/bin/env nu
# Welcome Message Module
# Handles ASCII art display and welcome message generation

use ../utils/ascii_art.nu *
use ../utils/constants.nu [DEFAULT_TERMINAL YAZELIX_VERSION]
use ../utils/keypress_polling.nu poll_for_keypress_status
use ../utils/upgrade_summary.nu get_upgrade_note_entry

def poll_for_welcome_keypress [timeout: duration] {
    ((poll_for_keypress_status $timeout).status == "key")
}

def show_welcome_art [
    welcome_style: string
    welcome_duration_seconds: float
    show_macchina_on_welcome: bool
]: nothing -> bool {
    let skipped = (render_welcome_style_interruptibly $welcome_style $welcome_duration_seconds null {|timeout| poll_for_welcome_keypress $timeout })

    # Show macchina if enabled and available
    if $show_macchina_on_welcome {
        macchina -o machine -o distribution -o desktop-environment -o processor -o gpu -o terminal
    }

    $skipped
}

# Get flake last updated info
def get_flake_info [yazelix_dir: string, colors: record]: nothing -> string {
    let flake_path = $"($yazelix_dir)/flake.nix"
    let flake_days_ago = if ($flake_path | path exists) {
        try {
            let file_info = (ls $flake_path | first)
            let now_seconds = (date now | format date %s | into int)
            let mod_seconds = ($file_info.modified | format date %s | into int)
            let diff_seconds = ($now_seconds - $mod_seconds)
            ($diff_seconds / 86400 | math floor)
        } catch {
            0
        }
    } else {
        0
    }

    if ($flake_days_ago | describe) == 'int' {
        $"($colors.cyan)🕒 Flake last updated: ($flake_days_ago) day\(s\) ago($colors.reset)"
    } else {
        $"($colors.cyan)🕒 Flake last updated: unknown($colors.reset)"
    }
}

def welcome_value_is_true [value] {
    ($value | default false | into string | str downcase) == "true"
}

def format_session_info [facts: record, colors: record]: nothing -> string {
    if (welcome_value_is_true ($facts.persistent_sessions? | default false)) {
        let session_name = ($facts.session_name? | default "yazelix")
        $"($colors.green)🔗 Using persistent session: ($session_name)($colors.reset)"
    } else {
        $"($colors.yellow)🆕 Creating new Zellij session($colors.reset)"
    }
}

def format_terminal_info [facts: record, colors: record]: nothing -> string {
    let terminals = ($facts.terminals? | default [$DEFAULT_TERMINAL])
    let preferred = if ($terminals | is-empty) { "unknown" } else { $terminals | first }
    $"($colors.cyan)🖥️  Preferred host terminal: ($preferred)($colors.reset)"
}

# Build complete welcome message
def get_startup_release_headline [] {
    let raw_headline = (try {
        let entry = (get_upgrade_note_entry)
        if $entry == null {
            ""
        } else {
            ($entry.headline? | default "" | into string | str trim)
        }
    } catch {
        ""
    })

    $raw_headline | str replace -r '\.+$' ""
}

export def build_welcome_message [
    yazelix_dir: string
    colors: record
    welcome_facts: record
]: nothing -> list<string> {
    let flake_info = get_flake_info $yazelix_dir $colors
    let session_info = format_session_info $welcome_facts $colors
    let terminal_info = format_terminal_info $welcome_facts $colors
    let release_headline = (get_startup_release_headline)

    [
        "",
        $"($colors.purple)🎉 Welcome to Yazelix ($YAZELIX_VERSION)!($colors.reset)",
        (if ($release_headline | is-not-empty) { $"($colors.blue)($release_headline)($colors.reset)" } else { "" }),
        $flake_info,
        $"($colors.cyan)✨ Now with Nix auto-setup, lazygit, Starship, and markdown-oxide($colors.reset)",
        $session_info,
        $terminal_info,
        $"($colors.yellow)⚠️  First run: grant the required Yazelix plugin permissions. Focus the top zjstatus bar and press 'y' if it prompts, and also say yes to the Yazelix orchestrator permission popup.($colors.reset)",
        $"($colors.cyan)💡 Quick tips: Use 'alt hjkl' to navigate, 'Ctrl y' to jump between the editor and sidebar, 'Alt y' to toggle the sidebar, and 'Alt [' or 'Alt ]' to change layout family($colors.reset)"
    ] | where $it != ""
}

# Display welcome screen or log it based on mode
export def show_welcome [
    skip_welcome_screen: bool
    quiet_mode: bool
    welcome_style: string
    welcome_duration_seconds: float
    show_macchina_on_welcome: bool
    welcome_message: list<string>
    log_dir: string
    colors: record
    bootstrap_skip_welcome: bool = false
]: nothing -> nothing {
    # Check modes
    let env_only_mode = ($env.YAZELIX_ENV_ONLY? == "true")
    let should_skip_welcome = $skip_welcome_screen or $env_only_mode or $bootstrap_skip_welcome

    # Show ASCII art first (if not skipping)
    if (not $should_skip_welcome) and (not $quiet_mode) {
        show_welcome_art $welcome_style $welcome_duration_seconds $show_macchina_on_welcome | ignore
    }

    # Show welcome or log it
    if $should_skip_welcome {
        if $env_only_mode {
            print $"($colors.cyan)🔧 Yazelix environment loaded! Launch the full interface in a separate terminal with 'yzx launch' or here with 'yzx enter'.($colors.reset)"
        } else if $bootstrap_skip_welcome {
            return
        } else {
            # Log welcome info
            let welcome_log_file = $"($log_dir)/welcome_(date now | format date '%Y%m%d_%H%M%S').log"
            $welcome_message | str join "\n" | save -f $welcome_log_file
            print $"($colors.cyan)💡 Welcome screen skipped. Welcome info logged to: ($welcome_log_file)($colors.reset)"
        }
    } else {
        # Display welcome message
        for $line in $welcome_message {
            print $line
        }

        # Prompt for a single key (if interactive)
        try {
            print -n $"($colors.purple)Press any key to launch Zellij and start your session... ($colors.reset)"
            input listen --types [key] | ignore
            print ""
        } catch {
            # Non-interactive shell, just continue
        }

        print $"($colors.purple)Launching Zellij...($colors.reset)"
    }
}
