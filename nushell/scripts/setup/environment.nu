#!/usr/bin/env nu
# Main Yazelix environment setup script
# Called from devenv.nix shellHook to reduce complexity

use ../utils/config_parser.nu parse_yazelix_config

def main [
    yazelix_dir: string
    recommended: string      # "true" or "false" from devenv.nix
    enable_atuin: string     # "true" or "false" from devenv.nix
    build_helix_from_source: string  # "true" or "false" from devenv.nix
    default_shell: string
    debug_mode: string       # "true" or "false" from devenv.nix
    extra_shells_str: string
    skip_welcome_screen: string  # "true" or "false" from devenv.nix
    helix_mode: string
    ascii_art_mode: string
    show_macchina_on_welcome: string = "false"  # "true" or "false" from devenv.nix
] {
    # Convert string booleans to actual booleans
    let recommended = ($recommended == "true")
    let enable_atuin = ($enable_atuin == "true")
    let build_helix_from_source = ($build_helix_from_source == "true")
    let debug_mode = ($debug_mode == "true")
    let skip_welcome_screen = ($skip_welcome_screen == "true")
    let show_macchina_on_welcome = ($show_macchina_on_welcome == "true")

    # Import constants and helper functions
    use ../utils/constants_with_helpers.nu *

    # DEBUG: Print skip_welcome_screen value
    if $debug_mode {
        print $"🔍 DEBUG: skip_welcome_screen parameter = ($skip_welcome_screen)"
    }

    # Detect quiet mode from environment
    let quiet_mode = ($env.YAZELIX_ENV_ONLY? == "true")

    # Detect environment first
    let env_info = (detect_environment)
    if $debug_mode {
        print $"🔍 Environment detection: ($env_info)"
    }

    # Handle different environment types
    match $env_info.environment_type {
        "home-manager" => {
            if $debug_mode {
                print "🏠 Home-manager environment detected - using read-only config approach"
            }
        }
        "read-only" => {
            print "⚠️  WARNING: Read-only configuration directory detected!"
            print "   This may indicate a managed environment or permission issue."
            print "   If using home-manager, see docs/home_manager_integration.md"
            print "   Some features may not work correctly."
        }
        "standard" => { }
    }

    # Validate user config against schema
    use ../utils/config_schema.nu validate_config_against_default

    # Parse extra shells from comma-separated string
    let extra_shells = if ($extra_shells_str | is-empty) or ($extra_shells_str == "NONE") {
        []
    } else {
        $extra_shells_str | split row "," | where $it != ""
    }

    # Determine which shells to configure (always nu/bash, plus default_shell and extra_shells)
    let shells_to_configure = (["nu", "bash"] ++ [$default_shell] ++ $extra_shells) | uniq

    # Setup logging in state directory (XDG-compliant)
    let state_dir = ($YAZELIX_STATE_DIR | str replace "~" $env.HOME)
    let log_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)
    mkdir $state_dir
    mkdir $log_dir

    # Auto-trim old logs (keep 10 most recent)
    let old_shellhook_logs = try {
        ls $"($log_dir)/shellhook_*.log"
        | sort-by modified -r
        | skip 10
        | get name
    } catch { [] }

    let old_welcome_logs = try {
        ls $"($log_dir)/welcome_*.log"
        | sort-by modified -r
        | skip 10
        | get name
    } catch { [] }

    let all_old_logs = ($old_shellhook_logs | append $old_welcome_logs)

    if not ($all_old_logs | is-empty) {
        rm ...$all_old_logs
    }

    let log_file = $"($log_dir)/shellhook_(date now | format date '%Y%m%d_%H%M%S').log"

    if not $quiet_mode {
        print $"📝 Logging to: ($log_file)"
    }

    # Generate shell initializers for configured shells only
    with-env {YAZELIX_QUIET_MODE: (if $quiet_mode { "true" } else { "false" })} {
        nu $"($yazelix_dir)/nushell/scripts/setup/initializers.nu" $yazelix_dir $recommended $enable_atuin ($shells_to_configure | str join ",")
    }

    # Setup shell hooks for configured shells
    use ./shell_hooks.nu setup_shell_hooks

    # Bash and Nushell are REQUIRED - error if config missing
    setup_shell_hooks "bash" $yazelix_dir $quiet_mode true
    setup_shell_hooks "nushell" $yazelix_dir $quiet_mode true

    # Fish and Zsh are optional - skip silently if not configured
    if ("fish" in $shells_to_configure) {
        setup_shell_hooks "fish" $yazelix_dir $quiet_mode false
    }

    if ("zsh" in $shells_to_configure) {
        setup_shell_hooks "zsh" $yazelix_dir $quiet_mode false
    }

    # Editor setup is now handled in the shellHook

    # Set permissions
    chmod +x $"($yazelix_dir)/shells/bash/start_yazelix.sh"
    chmod +x $"($yazelix_dir)/nushell/scripts/core/launch_yazelix.nu"
    chmod +x $"($yazelix_dir)/nushell/scripts/core/start_yazelix.nu"

    if not $quiet_mode {
        print "✅ Yazelix environment setup complete!"
    }

    # Import welcome module
    use ./welcome.nu *
    use ../utils/ascii_art.nu get_yazelix_colors

    # Get color scheme for consistent styling
    let colors = get_yazelix_colors

    # Build welcome message
    let welcome_message = build_welcome_message $yazelix_dir $helix_mode $colors

    # Display welcome screen or log it
    show_welcome $skip_welcome_screen $quiet_mode $ascii_art_mode $show_macchina_on_welcome $welcome_message $log_dir $colors
}
