#!/usr/bin/env nu
# yzx enter command - Start Yazelix in the current terminal

use ../utils/doctor.nu print_runtime_version_drift_warning
use ../utils/entrypoint_config_migrations.nu [run_entrypoint_config_migration_preflight]
use ../core/start_yazelix.nu [start_yazelix_session]

# Start Yazelix in the current terminal
export def "yzx enter" [
    --path(-p): string # Start in specific directory
    --home             # Start in home directory
    --verbose          # Enable verbose logging
    --reuse            # Reuse the last built profile without rebuilding
    --skip-refresh(-s) # Skip explicit refresh trigger and allow potentially stale environment
    --force-reenter    # Force re-entering devenv before startup
] {
    use ../utils/nix_detector.nu ensure_nix_available
    ensure_nix_available
    print_runtime_version_drift_warning
    run_entrypoint_config_migration_preflight "yzx enter" | ignore

    let verbose_mode = $verbose
    if $verbose_mode {
        print "🔍 yzx enter: verbose mode enabled"
    }

    $env.YAZELIX_ENV_ONLY = "false"

    let requested_path = ($path | default "")
    let cwd_override = if $home {
        $env.HOME
    } else if ($requested_path | is-not-empty) {
        $requested_path
    } else {
        null
    }

    if ($cwd_override != null) {
        start_yazelix_session $cwd_override --verbose=$verbose_mode --reuse=$reuse --skip-refresh=$skip_refresh --force-reenter=$force_reenter
    } else {
        start_yazelix_session --verbose=$verbose_mode --reuse=$reuse --skip-refresh=$skip_refresh --force-reenter=$force_reenter
    }
}
