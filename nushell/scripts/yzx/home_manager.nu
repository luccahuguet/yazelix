#!/usr/bin/env nu

use ../utils/doctor_report_bridge.nu evaluate_install_ownership_report

def format_prepare_artifact [artifact: record] {
    let tag = if (($artifact.class? | default "") == "blocker") {
        "[BLOCKER]"
    } else {
        "[CLEANUP]"
    }

    $"  - ($tag) ($artifact.label): ($artifact.path)"
}

def render_home_manager_prepare_preview [artifacts: list<record>] {
    let blockers = ($artifacts | where class == "blocker")
    let cleanup = ($artifacts | where class == "cleanup")
    mut lines = ["Yazelix Home Manager takeover preview"]

    if not ($blockers | is-empty) {
        $lines = ($lines | append "")
        $lines = ($lines | append "Blocking manual-install artifacts:")
        $lines = ($lines | append ($blockers | each {|artifact| format_prepare_artifact $artifact }))
    }

    if not ($cleanup | is-empty) {
        $lines = ($lines | append "")
        $lines = ($lines | append "Cleanup-only manual-install artifacts:")
        $lines = ($lines | append ($cleanup | each {|artifact| format_prepare_artifact $artifact }))
    }

    $lines = ($lines | append "")
    $lines = ($lines | append "Run `yzx home_manager prepare --apply` to archive these manual-install artifacts before `home-manager switch`.")

    $lines | flatten | str join "\n"
}

def archive_artifacts [artifacts: list<record>, backup_label: string] {
    let timestamp = (date now | format date "%Y%m%d_%H%M%S_%3f")

    $artifacts | each {|artifact|
        let backup_path = $"($artifact.path).($backup_label)-backup-($timestamp)"
        mv $artifact.path $backup_path
        $artifact | upsert backup_path $backup_path
    }
}

# Show Yazelix Home Manager takeover helpers
export def "yzx home_manager" [] {
    print "Yazelix Home Manager helpers"
    print "  yzx home_manager prepare   Preview or archive manual-install artifacts before Home Manager takeover"
    print "  yzx update home_manager    Refresh the current flake input, then print `home-manager switch`"
}

# Preview or archive manual-install artifacts before Home Manager takeover
export def "yzx home_manager prepare" [
    --apply  # Archive the detected manual-install takeover blockers and cleanup-only artifacts
    --yes    # Skip confirmation prompt when using --apply
] {
    let artifacts = ((evaluate_install_ownership_report).prepare_artifacts)

    if not $apply {
        if ($artifacts | is-empty) {
            print "No manual-install Yazelix artifacts need Home Manager takeover prep."
            print "Next step:"
            print "  home-manager switch"
            return
        }

        print (render_home_manager_prepare_preview $artifacts)
        return
    }

    if ($artifacts | is-empty) {
        print "No manual-install Yazelix artifacts need Home Manager takeover prep."
        print "Next step:"
        print "  home-manager switch"
        return
    }

    if not $yes {
        print "⚠️  This archives the detected manual-install Yazelix artifacts so Home Manager can take over their paths."
        print "   Archived files stay next to the original path with a timestamped `.home-manager-prepare-backup-*` suffix."
        let confirm = try {
            (input "Continue? [y/N]: " | str downcase | str trim)
        } catch {
            "n"
        }

        if $confirm not-in ["y", "yes"] {
            print "Aborted."
            return
        }
    }

    let archived = (archive_artifacts $artifacts "home-manager-prepare")

    print "Archived manual-install artifacts for Home Manager takeover:"
    for artifact in $archived {
        print $"  - ($artifact.label): ($artifact.path) -> ($artifact.backup_path)"
    }
    print "Next step:"
    print "  home-manager switch"
}
