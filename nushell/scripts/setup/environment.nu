#!/usr/bin/env nu
# Main Yazelix environment setup script
# Called from devenv.nix shellHook to reduce complexity

use ../utils/config_parser.nu parse_yazelix_config
use ../utils/common.nu [get_installed_yazelix_runtime_reference_dir get_yazelix_runtime_dir resolve_yazelix_nu_bin]
use ../utils/nushell_externs.nu [sync_generated_yzx_extern_bridge]
use ../utils/shell_user_hooks.nu [sync_generated_nushell_user_hook_bridge]
use ../utils/startup_profile.nu [profile_startup_step]

def ensure_user_cli_wrapper [yazelix_dir: string] {
    let local_bin_dir = ($env.HOME | path join ".local" "bin")
    let installed_runtime_reference = (get_installed_yazelix_runtime_reference_dir)
    let cli_target = if ($installed_runtime_reference | path exists) {
        ($installed_runtime_reference | path join "bin" "yzx")
    } else {
        ($yazelix_dir | path join "bin" "yzx")
    }
    let cli_link = ($local_bin_dir | path join "yzx")

    if not ($cli_target | path exists) {
        error make {msg: $"Missing Yazelix CLI wrapper: ($cli_target)"}
    }

    mkdir $local_bin_dir
    rm -f $cli_link
    ^ln -s $cli_target $cli_link
}

def ensure_runtime_scripts_executable [yazelix_dir: string] {
    let runtime_root = ($yazelix_dir | path expand)
    if ($runtime_root | str starts-with "/nix/store/") {
        return
    }

    chmod +x $"($runtime_root)/shells/bash/start_yazelix.sh"
    chmod +x $"($runtime_root)/shells/posix/start_yazelix.sh"
    chmod +x $"($runtime_root)/shells/posix/yazelix_hx.sh"
    chmod +x $"($runtime_root)/shells/posix/yzx_cli.sh"
    chmod +x $"($runtime_root)/nushell/scripts/core/launch_yazelix.nu"
    chmod +x $"($runtime_root)/nushell/scripts/core/start_yazelix.nu"
}

def main [--welcome-source: string, --skip-welcome] {
    # Read configuration directly from TOML - single source of truth!
    let config = parse_yazelix_config

    # Extract values from config (all properly typed from TOML)
    let yazelix_dir = (get_yazelix_runtime_dir)
    let default_shell = ($config.default_shell? | default "nu")
    let debug_mode = ($config.debug_mode? | default false)
    let runtime_nu = (resolve_yazelix_nu_bin)
    let skip_welcome_screen = (
        ($config.skip_welcome_screen? | default false)
        or ($env.YAZELIX_STARTUP_PROFILE_SKIP_WELCOME? == "true")
    )
    let helix_mode = ($config.helix_mode? | default "release")
    let welcome_style = ($config.welcome_style? | default "random")
    let welcome_duration_seconds = ($config.welcome_duration_seconds? | default 2.0)
    let show_macchina_on_welcome = ($config.show_macchina_on_welcome? | default false)

    # Parse extra shells from config
    let extra_shells = ($config.extra_shells? | default [])

    # Import constants and helper functions
    use ../utils/constants_with_helpers.nu *

    # DEBUG: Print skip_welcome_screen value
    if $debug_mode {
        print $"🔍 DEBUG: skip_welcome_screen from config = ($skip_welcome_screen)"
    }

    # Noninteractive shellHook entry should stay quiet even when only the
    # welcome UI is skipped, so launch/refresh rebuilds don't replay routine
    # setup chatter in the caller terminal.
    let quiet_mode = (
        ($env.YAZELIX_ENV_ONLY? == "true")
        or $skip_welcome
        or ($env.YAZELIX_SHELLHOOK_SKIP_WELCOME? == "true")
    )
    let shellhook_phase = (
        $env.YAZELIX_STARTUP_PROFILE_PHASE?
        | default "shell_entry"
        | into string
        | str trim
    )
    let shellhook_pid = ($nu.pid | into string)

    def profile_shellhook_step [step: string, code: closure, metadata?: record] {
        profile_startup_step "shellhook" $step $code (
            ($metadata | default {})
            | upsert phase $shellhook_phase
            | upsert pid $shellhook_pid
        )
    }

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
    profile_shellhook_step "generate_initializers" {
        with-env {YAZELIX_QUIET_MODE: (if $quiet_mode { "true" } else { "false" })} {
            ^$runtime_nu $"($yazelix_dir)/nushell/scripts/setup/initializers.nu" $yazelix_dir ($shells_to_configure | str join ",")
        }
    } {
        shells: $shells_to_configure
    }
    profile_shellhook_step "sync_yzx_extern_bridge" {
        sync_generated_yzx_extern_bridge $yazelix_dir
    }
    profile_shellhook_step "sync_nushell_user_hook_bridge" {
        sync_generated_nushell_user_hook_bridge
    }

    # Setup shell hooks for configured shells
    use ./shell_hooks.nu setup_shell_hooks

    # Bash and Nushell are REQUIRED - error if config missing
    profile_shellhook_step "setup_bash_hooks" {
        setup_shell_hooks "bash" $yazelix_dir $quiet_mode true
    }
    profile_shellhook_step "setup_nushell_hooks" {
        setup_shell_hooks "nushell" $yazelix_dir $quiet_mode true
    }

    # Fish and Zsh are optional - skip silently if not configured
    if ("fish" in $shells_to_configure) {
        profile_shellhook_step "setup_fish_hooks" {
            setup_shell_hooks "fish" $yazelix_dir $quiet_mode false
        }
    }

    if ("zsh" in $shells_to_configure) {
        profile_shellhook_step "setup_zsh_hooks" {
            setup_shell_hooks "zsh" $yazelix_dir $quiet_mode false
        }
    }

    # Editor setup is now handled in the shellHook

    profile_shellhook_step "ensure_runtime_scripts_executable" {
        ensure_runtime_scripts_executable $yazelix_dir
    }
    profile_shellhook_step "ensure_user_cli_wrapper" {
        ensure_user_cli_wrapper $yazelix_dir
    }

    let zjstatus_target = $"($yazelix_dir)/configs/zellij/plugins/zjstatus.wasm"
    if not ($zjstatus_target | path exists) {
        print $"❌ Error: Vendored zjstatus wasm not found at: ($zjstatus_target)"
        exit 1
    }

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

    # Display welcome screen or log it (skip when start_yazelix handles it)
    if $welcome_source != "start" {
        show_welcome $skip_welcome_screen $quiet_mode $welcome_style $welcome_duration_seconds $show_macchina_on_welcome $welcome_message $log_dir $colors $skip_welcome
    }
}
