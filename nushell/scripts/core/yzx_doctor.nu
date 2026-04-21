#!/usr/bin/env nu
# Doctor command-family entrypoint that still lives in Nushell.

# Run health checks and diagnostics
export def "yzx doctor" [
    --verbose(-v)  # Show detailed information
    --fix(-f)      # Attempt to auto-fix issues
    --json         # Emit machine-readable doctor data
] {
    use ../utils/doctor.nu [collect_doctor_report run_doctor_checks]

    if $json and $fix {
        error make {msg: "`yzx doctor --json` does not support `--fix` yet. Run `yzx doctor --json` for machine-readable diagnostics or `yzx doctor --fix` for the current interactive repair flow."}
    }

    if $json {
        print ((collect_doctor_report) | to json -r)
    } else if $fix {
        with-env { YAZELIX_ACCEPT_USER_CONFIG_RELOCATION: "true" } {
            run_doctor_checks $verbose $fix
        }
    } else {
        run_doctor_checks $verbose $fix
    }
}
