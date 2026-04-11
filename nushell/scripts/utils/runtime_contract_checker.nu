#!/usr/bin/env nu

use failure_classes.nu [format_failure_classification]
use terminal_launcher.nu detect_terminal_candidates
use constants.nu [SUPPORTED_TERMINALS TERMINAL_METADATA]
use common.nu [get_yazelix_state_dir]

def build_runtime_check [
    id: string
    status: string
    severity: string
    owner_surface: string
    message: string
    details?
    recovery?
    failure_class?
    --blocking
    --path: string
    --candidates: any
] {
    {
        id: $id
        status: $status
        severity: $severity
        owner_surface: $owner_surface
        message: $message
        details: ($details | default null)
        recovery: ($recovery | default null)
        failure_class: ($failure_class | default null)
        blocking: $blocking
        path: ($path | default null)
        candidates: ($candidates | default null)
    }
}

def runtime_check_to_error [check: record] {
    ([ $check.message ] | append (build_runtime_check_detail_lines $check) | str join "\n")
}

export def require_runtime_check [check: record] {
    if ($check.status == "ok") {
        return $check
    }

    error make {msg: (runtime_check_to_error $check)}
}

export def runtime_check_to_doctor_result [check: record] {
    let detail_lines = (build_runtime_check_detail_lines $check)

    {
        status: (if ($check.status == "ok") { "ok" } else { $check.severity })
        message: $check.message
        details: (if ($detail_lines | is-empty) { null } else { $detail_lines | str join "\n" })
        fix_available: false
        runtime_contract_check: $check.id
        owner_surface: $check.owner_surface
    }
}

def build_runtime_check_detail_lines [check: record] {
    mut detail_lines = []

    if (($check.details? | default "") | is-not-empty) {
        $detail_lines = ($detail_lines | append $check.details)
    }

    let recovery = ($check.recovery? | default "" | into string | str trim)
    let failure_class = ($check.failure_class? | default "" | into string | str trim)
    if ($recovery | is-not-empty) and ($failure_class | is-not-empty) {
        $detail_lines = ($detail_lines | append (format_failure_classification $failure_class $recovery))
    } else if ($recovery | is-not-empty) {
        $detail_lines = ($detail_lines | append $recovery)
    }

    $detail_lines
}

export def resolve_expected_layout_path [config: record, layout_dir?: string] {
    let configured_layout = if ($config.enable_sidebar? | default true) { "yzx_side" } else { "yzx_no_side" }
    let layout = if ($env.YAZELIX_LAYOUT_OVERRIDE? | is-not-empty) {
        $env.YAZELIX_LAYOUT_OVERRIDE
    } else if ($env.YAZELIX_SWEEP_TEST_ID? | is-not-empty) and ($env.ZELLIJ_DEFAULT_LAYOUT? | is-not-empty) {
        $env.ZELLIJ_DEFAULT_LAYOUT
    } else {
        $configured_layout
    }
    let resolved_layout_dir = if ($layout_dir | is-not-empty) {
        $layout_dir
    } else {
        (get_yazelix_state_dir | path join "configs" "zellij" "layouts")
    }

    if ($layout | str contains "/") or ($layout | str ends-with ".kdl") {
        $layout
    } else {
        $resolved_layout_dir | path join $"($layout).kdl"
    }
}

def check_working_directory [
    working_dir: string
    id: string
    owner_surface: string
    missing_label: string
    missing_guidance: string
    invalid_label: string
    invalid_guidance: string
] {
    let resolved = ($working_dir | path expand)

    if not ($resolved | path exists) {
        return (build_runtime_check
            $id
            "error"
            "error"
            $owner_surface
            $"($missing_label): ($resolved)"
            $missing_guidance
            --blocking)
    }

    if (($resolved | path type) != "dir") {
        return (build_runtime_check
            $id
            "error"
            "error"
            $owner_surface
            $"($invalid_label): ($resolved)"
            $invalid_guidance
            --blocking)
    }

    build_runtime_check $id "ok" "info" $owner_surface $"Working directory is valid: ($resolved)" --path $resolved
}

def check_runtime_file [
    file_path: string
    id: string
    owner_surface: string
    missing_label: string
    invalid_label: string
    recovery: string
] {
    let resolved = ($file_path | path expand)

    if not ($resolved | path exists) {
        return (build_runtime_check
            $id
            "error"
            "error"
            $owner_surface
            $"Missing Yazelix ($missing_label): ($resolved)"
            null
            $recovery
            "generated-state"
            --blocking
            --path $resolved)
    }

    if (($resolved | path type) != "file") {
        return (build_runtime_check
            $id
            "error"
            "error"
            $owner_surface
            $"Yazelix ($invalid_label) is not a file: ($resolved)"
            null
            null
            null
            --blocking
            --path $resolved)
    }

    build_runtime_check $id "ok" "info" $owner_surface $"Yazelix ($missing_label) is present" --path $resolved
}

export def check_startup_working_dir [working_dir: string] {
    (check_working_directory
        $working_dir
        "startup_working_dir"
        "startup"
        "Startup directory does not exist"
        "Use an existing directory, or run yzx launch --home."
        "Startup path is not a directory"
        "Pass a directory to yzx launch --path.")
}

export def check_launch_working_dir [working_dir: string] {
    (check_working_directory
        $working_dir
        "launch_working_dir"
        "launch"
        "Launch directory does not exist"
        "Use an existing directory, or use --home to start from HOME."
        "Launch path is not a directory"
        "Pass a directory to yzx launch --path.")
}

export def check_runtime_script [script_path: string, id: string, label: string, owner_surface: string] {
    (check_runtime_file
        $script_path
        $id
        $owner_surface
        $label
        $label
        "Your runtime looks incomplete. Reinstall/regenerate Yazelix and try again.")
}

export def check_generated_layout [layout_path: string, owner_surface: string] {
    (check_runtime_file
        $layout_path
        "generated_layout"
        $owner_surface
        "generated Zellij layout"
        "generated Zellij layout"
        "Run `yzx refresh` to regenerate layouts, or check the configured layout name.")
}

export def check_launch_terminal_support [requested_terminal: string, terminals: list<string>] {
    if ($requested_terminal | is-not-empty) {
        let specified_terminal = $requested_terminal
        let term_meta = ($TERMINAL_METADATA | get -o $specified_terminal)
        if $term_meta == null {
            return (build_runtime_check
                "launch_terminal_support"
                "error"
                "error"
                "launch"
                $"Unsupported terminal '($specified_terminal)'"
                $"Supported terminals: ($SUPPORTED_TERMINALS | str join ', ')"
                null
                null
                --blocking)
        }

        let candidates = (detect_terminal_candidates [$specified_terminal])

        if ($candidates | is-empty) {
            let reason = $"Specified terminal '($specified_terminal)' is not installed on the host."
            let recovery = "Install it on the host, or choose a different terminal for testing."
            return (build_runtime_check
                "launch_terminal_support"
                "error"
                "error"
                "launch"
                $reason
                null
                $recovery
                "host-dependency"
                --blocking)
        }

        return (build_runtime_check
            "launch_terminal_support"
            "ok"
            "info"
            "launch"
            $"Terminal launch support is available for ($specified_terminal)"
            null
            null
            null
            --candidates $candidates)
    }

    let candidates = (detect_terminal_candidates $terminals)
    if ($candidates | is-empty) {
        let reason = "None of the configured terminal binaries are installed on the host."
        let recovery = "Install one of the configured terminals on the host, or adjust [terminal].terminals to match what is available."
        return (build_runtime_check
            "launch_terminal_support"
            "error"
            "error"
            "launch"
            $reason
            null
            $recovery
            "host-dependency"
            --blocking)
    }

    (build_runtime_check
        "launch_terminal_support"
        "ok"
        "info"
        "launch"
        "Configured terminal launch support is available"
        null
        null
        null
        --candidates $candidates)
}
