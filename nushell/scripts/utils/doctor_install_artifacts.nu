#!/usr/bin/env nu

use common.nu get_yazelix_runtime_reference_dir
use runtime_distribution_capabilities.nu get_runtime_distribution_capability_profile
use install_ownership.nu [
    get_manual_desktop_entry_path
    get_manual_runtime_reference_path
    get_manual_yzx_cli_path
    has_home_manager_managed_install
]

def resolve_realpath_or_null [target: string] {
    let result = (^readlink -f $target | complete)
    if $result.exit_code == 0 {
        let resolved = ($result.stdout | str trim)
        if ($resolved | is-empty) { null } else { $resolved }
    } else {
        null
    }
}

def path_is_symlink [target: string] {
    let result = (^bash -lc $"test -L ($target | into string | to json -r)" | complete)
    $result.exit_code == 0
}

def detect_install_owner [] {
    if (has_home_manager_managed_install) {
        "home-manager"
    } else if (
        ((get_home_manager_profile_yzx_path) != null)
        or ((get_home_manager_profile_desktop_entry_path | path exists))
    ) {
        "home-manager"
    } else {
        "installer"
    }
}

def get_home_manager_profile_desktop_entry_path [] {
    ($env.HOME | path join ".nix-profile" "share" "applications" "yazelix.desktop")
}

def get_current_installed_runtime_target [] {
    let runtime_link = (get_manual_runtime_reference_path)
    resolve_realpath_or_null $runtime_link
}

def get_desktop_entry_exec [desktop_path: string] {
    if not ($desktop_path | path exists) {
        return null
    }

    let entry = (open $desktop_path --raw)
    let marker = (
        $entry
        | lines
        | where {|line| $line | str starts-with "Exec="}
        | get -o 0
    )

    if $marker == null {
        null
    } else {
        (
            $marker
            | str replace 'Exec=' ""
            | str trim
        )
    }
}

def desktop_entry_exec_matches_expected [desktop_exec, expected_execs: list<string>] {
    if $desktop_exec == null {
        false
    } else {
        $expected_execs | any {|expected_exec| $desktop_exec == $expected_exec }
    }
}

def get_home_manager_yzx_profile_paths [] {
    mut candidates = [
        ($env.HOME | path join ".nix-profile" "bin" "yzx")
    ]

    if ("USER" in $env) and (($env.USER | default "") != "") {
        $candidates = ($candidates | append ("/etc/profiles/per-user" | path join $env.USER "bin" "yzx"))
    }

    $candidates | uniq
}

def get_expected_desktop_entry_execs [install_owner: string] {
    let launcher_paths = if $install_owner == "home-manager" {
        get_home_manager_yzx_profile_paths
    } else {
        [ (get_manual_yzx_cli_path) ]
    }

    (
        $launcher_paths
        | each {|launcher_path|
            [
                $"\"($launcher_path)\" desktop launch"
                $"($launcher_path) desktop launch"
            ]
        }
        | flatten
        | uniq
    )
}

export def check_desktop_entry_freshness [] {
    let install_owner = (detect_install_owner)
    let local_desktop_path = (get_manual_desktop_entry_path)
    let profile_desktop_path = (get_home_manager_profile_desktop_entry_path)
    let local_desktop_exists = ($local_desktop_path | path exists)
    let profile_desktop_exists = ($profile_desktop_path | path exists)
    let desktop_path = if $local_desktop_exists {
        $local_desktop_path
    } else if $profile_desktop_exists {
        $profile_desktop_path
    } else {
        null
    }
    let expected_execs = (get_expected_desktop_entry_execs $install_owner)

    if $desktop_path == null {
        let details = if $install_owner == "home-manager" {
            $"Home Manager-managed desktop entries usually resolve through ($profile_desktop_path). Reapply your Home Manager configuration if it is missing."
        } else {
            "Run `yzx desktop install` if you want application-launcher integration."
        }
        return {
            status: "info"
            message: "Yazelix desktop entry not installed"
            details: $details
            fix_available: false
        }
    }

    let local_desktop_exec = if $local_desktop_exists {
        get_desktop_entry_exec $local_desktop_path
    } else {
        null
    }
    let profile_desktop_exec = if $profile_desktop_exists {
        get_desktop_entry_exec $profile_desktop_path
    } else {
        null
    }

    if (
        $install_owner == "home-manager"
        and $local_desktop_exists
        and $profile_desktop_exists
        and (not (desktop_entry_exec_matches_expected $local_desktop_exec $expected_execs))
        and (desktop_entry_exec_matches_expected $profile_desktop_exec $expected_execs)
    ) {
        return {
            status: "warning"
            message: "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry"
            details: $"Shadowing local entry: ($local_desktop_path)\nLocal Exec: ($local_desktop_exec | default '<missing>')\nHome Manager entry: ($profile_desktop_path)\nProfile Exec: ($profile_desktop_exec)\nRemove the shadowing local entry with `yzx desktop uninstall` or refresh it with `yzx desktop install`."
            fix_available: false
        }
    }

    let desktop_exec = if $desktop_path == $local_desktop_path {
        $local_desktop_exec
    } else {
        $profile_desktop_exec
    }
    let repair_hint = if $desktop_path == $profile_desktop_path {
        "Repair by reapplying your Home Manager configuration."
    } else {
        "Repair with `yzx desktop install`."
    }

    if $desktop_exec == null {
        return {
            status: "warning"
            message: "Yazelix desktop entry is invalid"
            details: $"The installed desktop entry has no Exec line. ($repair_hint)"
            fix_available: false
        }
    }

    if not (desktop_entry_exec_matches_expected $desktop_exec $expected_execs) {
        return {
            status: "warning"
            message: "Yazelix desktop entry does not use the stable launcher path"
            details: $"Desktop entry Exec: ($desktop_exec)\nExpected one of: ($expected_execs | str join ', ')\n($repair_hint)"
            fix_available: false
        }
    }

    {
        status: "ok"
        message: "Yazelix desktop entry uses the stable launcher path"
        details: $desktop_path
        fix_available: false
    }
}

def get_home_manager_profile_yzx_path [] {
    (
        get_home_manager_yzx_profile_paths
        | where {|path| ($path | path exists) or (path_is_symlink $path) }
        | get -o 0
    )
}

def get_runtime_variants [current_runtime_target?: string] {
    let runtime_ref = (get_yazelix_runtime_reference_dir)
    if $current_runtime_target == null {
        [$runtime_ref] | uniq
    } else {
        [$runtime_ref, $current_runtime_target] | uniq
    }
}

def get_required_shell_hook_checks [current_runtime_target?: string] {
    let runtime_variants = (get_runtime_variants $current_runtime_target)
    let yzx_cli_path = (get_manual_yzx_cli_path)
    [
        {
            shell: "bash"
            file: ($env.HOME | path join ".bashrc")
            acceptable_groups: (
                $runtime_variants
                | each {|runtime|
                    [
                        $"source \"($runtime | path join 'shells' 'bash' 'yazelix_bash_config.sh')\""
                        $"    \"($yzx_cli_path)\" \"$@\""
                    ]
                }
            )
        }
        {
            shell: "nushell"
            file: ($env.HOME | path join ".config" "nushell" "config.nu")
            acceptable_groups: (
                $runtime_variants
                | each {|runtime|
                    [
                        $"source \"($runtime | path join 'nushell' 'config' 'config.nu')\""
                        $"use ($runtime | path join 'nushell' 'scripts' 'core' 'yazelix.nu') *"
                    ]
                }
            )
        }
    ]
}

def evaluate_required_shell_hook [hook: record] {
    if not ($hook.file | path exists) {
        return {
            shell: $hook.shell
            status: "missing"
            file: $hook.file
        }
    }

    let content = (open $hook.file --raw)
    let matches_any_group = (
        $hook.acceptable_groups
        | any {|group|
            $group | all {|line| $content | str contains $line }
        }
    )
    if $matches_any_group {
        {
            shell: $hook.shell
            status: "current"
            file: $hook.file
        }
    } else {
        {
            shell: $hook.shell
            status: "outdated"
            file: $hook.file
        }
    }
}

export def check_install_artifact_staleness [capability_profile?: record] {
    let profile = if $capability_profile == null {
        get_runtime_distribution_capability_profile
    } else {
        $capability_profile
    }

    if not ($profile.supports_install_artifact_checks? | default false) {
        return [{
            status: "info"
            message: $"Installer-owned runtime artifact checks skipped in ($profile.title)"
            details: $"Current runtime root: (($profile.runtime_dir? | default '<unknown>'))\n($profile.runtime_update_guidance)"
            fix_available: false
        }]
    }

    mut results = []
    let install_owner = (detect_install_owner)
    let repair_hint = if $install_owner == "home-manager" {
        "Repair by reapplying your Home Manager configuration (for example `home-manager switch`)."
    } else {
        "Repair with `nix run github:luccahuguet/yazelix#install`."
    }

    let runtime_link = (get_manual_runtime_reference_path)
    let current_runtime_target = if $install_owner == "home-manager" {
        resolve_realpath_or_null (get_yazelix_runtime_reference_dir)
    } else {
        get_current_installed_runtime_target
    }

    if ($install_owner != "home-manager") and (not ($runtime_link | path exists)) and (not (path_is_symlink $runtime_link)) {
        $results = ($results | append {
            status: "warning"
            message: "Installed Yazelix runtime link is missing"
            details: $"Expected runtime link: ($runtime_link)\n($repair_hint)"
            fix_available: false
        })
    } else if ($install_owner != "home-manager") and ($current_runtime_target == null) {
        $results = ($results | append {
            status: "warning"
            message: "Installed Yazelix runtime link is broken"
            details: $"Runtime link exists but does not resolve to a valid runtime: ($runtime_link)\n($repair_hint)"
            fix_available: false
        })
    } else if $install_owner != "home-manager" {
        $results = ($results | append {
            status: "ok"
            message: "Installed Yazelix runtime link is healthy"
            details: $"($runtime_link) -> ($current_runtime_target)"
            fix_available: false
        })
    }

    let yzx_cli_path = if $install_owner == "home-manager" {
        get_home_manager_profile_yzx_path
    } else {
        get_manual_yzx_cli_path
    }
    let yzx_cli_target = if $yzx_cli_path == null {
        null
    } else {
        resolve_realpath_or_null $yzx_cli_path
    }

    if $yzx_cli_path == null {
        let missing_yzx_details = if $install_owner == "home-manager" {
            let hm_paths = (get_home_manager_yzx_profile_paths)
            $"Expected Home Manager profile command at one of: ($hm_paths | str join ', ')\n($repair_hint)"
        } else {
            $"Expected CLI path: ((get_manual_yzx_cli_path))\n($repair_hint)"
        }
        $results = ($results | append {
            status: "warning"
            message: "Installed yzx command is missing"
            details: $missing_yzx_details
            fix_available: false
        })
    } else if $yzx_cli_target == null {
        $results = ($results | append {
            status: "warning"
            message: "Installed yzx command is broken"
            details: $"The yzx command exists but does not resolve cleanly: ($yzx_cli_path)\n($repair_hint)"
            fix_available: false
        })
    } else {
        if $install_owner == "home-manager" {
            let expected_runtime_bin_targets = (
                get_runtime_variants $current_runtime_target
                | each {|runtime| $runtime | path join "bin" "yzx" }
            )
            let expected_runtime_bin_targets_resolved = (
                $expected_runtime_bin_targets
                | each {|target| resolve_realpath_or_null $target }
                | compact
            )
            let all_expected_runtime_bin_targets = ($expected_runtime_bin_targets | append $expected_runtime_bin_targets_resolved | uniq)
            if ($all_expected_runtime_bin_targets | any {|target| $yzx_cli_target == $target }) {
                $results = ($results | append {
                    status: "ok"
                    message: "Installed yzx command matches the current runtime"
                    details: $"($yzx_cli_path) -> ($yzx_cli_target)"
                    fix_available: false
                })
            } else {
                $results = ($results | append {
                    status: "warning"
                    message: "Installed yzx command is stale"
                    details: $"yzx target: ($yzx_cli_target)\nExpected runtime targets: ($all_expected_runtime_bin_targets | str join ', ')\n($repair_hint)"
                    fix_available: false
                })
            }
        } else {
            let expected_yzx_targets = (
                get_runtime_variants $current_runtime_target
                | each {|runtime|
                    [
                        ($runtime | path join "shells" "posix" "yzx_cli.sh")
                        ($runtime | path join "bin" "yzx")
                    ]
                }
                | flatten
            )
            let expected_yzx_targets_resolved = (
                $expected_yzx_targets
                | each {|target| resolve_realpath_or_null $target }
                | compact
            )
            let all_expected_yzx_targets = ($expected_yzx_targets | append $expected_yzx_targets_resolved | uniq)
            if not ($all_expected_yzx_targets | any {|target| $yzx_cli_target == $target }) {
                $results = ($results | append {
                    status: "warning"
                    message: "Installed yzx command is stale"
                    details: $"yzx target: ($yzx_cli_target)\nExpected one of: ($all_expected_yzx_targets | str join ', ')\n($repair_hint)"
                    fix_available: false
                })
            } else {
                $results = ($results | append {
                    status: "ok"
                    message: "Installed yzx command matches the current runtime"
                    details: $"($yzx_cli_path) -> ($yzx_cli_target)"
                    fix_available: false
                })
            }
        }
    }

    if $install_owner == "home-manager" {
        $results = ($results | append {
            status: "info"
            message: "Shell-hook freshness checks skipped for Home Manager-managed Yazelix install"
            details: "Home Manager owns the profile-provided `yzx` command directly. Host shell hooks are optional for this install path and may be managed separately."
            fix_available: false
        })
    } else {
        let shell_hook_results = (
            get_required_shell_hook_checks $current_runtime_target
            | each {|hook| evaluate_required_shell_hook $hook }
        )
        for hook in $shell_hook_results {
            if $hook.status == "current" {
                $results = ($results | append {
                    status: "ok"
                    message: $"Required ($hook.shell) Yazelix hook is current"
                    details: $hook.file
                    fix_available: false
                })
            } else if $hook.status == "outdated" {
                $results = ($results | append {
                    status: "warning"
                    message: $"Required ($hook.shell) Yazelix hook is stale"
                    details: $"Config file: ($hook.file)\n($repair_hint)"
                    fix_available: false
                })
            } else if $hook.status == "missing" {
                $results = ($results | append {
                    status: "warning"
                    message: $"Required ($hook.shell) Yazelix hook is missing"
                    details: $"Config file: ($hook.file)\n($repair_hint)"
                    fix_available: false
                })
            }
        }
    }

    $results
}
