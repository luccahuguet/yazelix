#!/usr/bin/env nu
# Test lane: maintainer
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_root resolve_test_yzx_core_bin]
use ../utils/common.nu [get_yazelix_config_dir get_yazelix_runtime_dir get_yazelix_state_dir]
use ../utils/yzx_core_bridge.nu [build_default_yzx_core_error_surface compute_runtime_env_via_yzx_core run_yzx_core_json_command]

const HELIX_MATERIALIZATION_GENERATE_COMMAND = "helix-materialization.generate"

def materialize_managed_helix_config_for_test [] {
    let runtime_dir = (get_yazelix_runtime_dir)
    let result = (run_yzx_core_json_command
        $runtime_dir
        (build_default_yzx_core_error_surface)
        [
            $HELIX_MATERIALIZATION_GENERATE_COMMAND
            "--runtime-dir" $runtime_dir
            "--config-dir" (get_yazelix_config_dir)
            "--state-dir" (get_yazelix_state_dir)
        ]
        "Yazelix Rust helix-materialization helper returned invalid JSON.")

    $result.generated_path
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: managed Helix config preserves user overrides while enforcing the Yazelix reveal binding.
def test_generate_managed_helix_config_merges_user_config_and_enforces_reveal [] {
    print "🧪 Testing managed Helix config generation keeps user settings while enforcing Yazelix reveal..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_helix_config_XXXXXX | str trim)
    let config_dir = ($tmp_home | path join ".config" "yazelix")
    let user_config_dir = ($config_dir | path join "user_configs")
    mkdir ($tmp_home | path join ".config")
    mkdir $config_dir
    mkdir $user_config_dir

    let result = (try {
        let helix_user_dir = ($user_config_dir | path join "helix")
        mkdir $helix_user_dir
        '[editor]
line-number = "relative"

[keys.normal]
g = "goto_file_start"
A-r = ":noop"
' | save --force --raw ($helix_user_dir | path join "config.toml")

        let merged = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_YZX_CORE_BIN: (resolve_test_yzx_core_bin)
        } {
            let output_path = (materialize_managed_helix_config_for_test)
            {
                output_path: $output_path
                config: (open $output_path)
            }
        })

        let normal_keys = ($merged.config.keys | get normal)
        let expected_output_path = ($tmp_home | path join ".local" "share" "yazelix" "configs" "helix" "config.toml")

        if (
            ($merged.output_path == $expected_output_path)
            and (($merged.config.editor | get "line-number") == "relative")
            and (($normal_keys | get g) == "goto_file_start")
            and (($normal_keys | get "A-r") == ':sh yzx reveal "%{buffer_name}"')
        ) {
            print "  ✅ Managed Helix config preserves user overrides while forcing the Yazelix reveal binding"
            true
        } else {
            print $"  ❌ Unexpected managed Helix config: (($merged | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Defends: runtime env wraps Helix with the managed Yazelix wrapper and preserves the wrapped binary hint.
def test_get_runtime_env_wraps_helix_with_managed_wrapper [] {
    print "🧪 Testing runtime env wraps Helix with the Yazelix-managed wrapper and records the wrapped binary hint..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_managed_helix_launch_env_XXXXXX | str trim)

    let result = (try {
        let runtime_env = (with-env {
            HOME: $tmp_home
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_YZX_CORE_BIN: (resolve_test_yzx_core_bin)
        } {
            compute_runtime_env_via_yzx_core {
                editor_command: "hx"
                enable_sidebar: true
                helix_runtime_path: null
            }
        })

        let expected_wrapper = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")
        let expected_binary = "hx"

        let retired_env_keys = [
            "YAZELIX_DEBUG_MODE"
            "YAZELIX_DIR"
            "YAZELIX_ENABLE_SIDEBAR"
            "YAZELIX_HELIX_MODE"
            "YAZELIX_MANAGED_EDITOR_KIND"
            "YAZELIX_NU_BIN"
            "YAZELIX_TERMINAL_CONFIG_MODE"
            "YAZELIX_WELCOME_STYLE"
        ]
        let retired_keys_absent = (
            $retired_env_keys
            | all {|key| not ($runtime_env | columns | any {|column| $column == $key }) }
        )

        if (
            ($runtime_env.EDITOR == $expected_wrapper)
            and (($runtime_env | get YAZELIX_MANAGED_HELIX_BINARY) == $expected_binary)
            and $retired_keys_absent
        ) {
            print "  ✅ Runtime env routes managed Helix sessions through the Yazelix wrapper, preserves the wrapped Helix hint, and omits dead export-only vars"
            true
        } else {
            print $"  ❌ Unexpected managed Helix runtime env: (($runtime_env | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
# Regression: the canonical Nu runtime env must export the curated toolbin instead of leaking the full runtime helper closure.
def test_get_runtime_env_exports_curated_toolbin_and_keeps_runtime_local_yzx [] {
    print "🧪 Testing canonical Nu runtime env exports toolbin, keeps runtime-local yzx reachable, and hides runtime-only helpers..."

    let tmp_home = (^mktemp -d /tmp/yazelix_runtime_env_self_contained_XXXXXX | str trim)

    let result = (try {
        let fake_runtime = ($tmp_home | path join "runtime")
        let fake_runtime_toolbin = ($fake_runtime | path join "toolbin")
        let fake_runtime_bin = ($fake_runtime | path join "bin")
        let fake_runtime_libexec = ($fake_runtime | path join "libexec")
        mkdir $fake_runtime
        mkdir $fake_runtime_toolbin
        mkdir $fake_runtime_bin
        mkdir $fake_runtime_libexec
        "" | save --force --raw ($fake_runtime | path join "yazelix_default.toml")

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_runtime_bin | path join "yzx")
        ^chmod +x ($fake_runtime_bin | path join "yzx")

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_runtime_toolbin | path join "rg")
        ^chmod +x ($fake_runtime_toolbin | path join "rg")

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_runtime_libexec | path join "dirname")
        ^chmod +x ($fake_runtime_libexec | path join "dirname")

        let runtime_env = (with-env {
            HOME: $tmp_home
            PATH: []
            YAZELIX_RUNTIME_DIR: $fake_runtime
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_YZX_CORE_BIN: (resolve_test_yzx_core_bin)
        } {
            compute_runtime_env_via_yzx_core {
                editor_command: "hx"
                enable_sidebar: true
                helix_runtime_path: null
            }
        })

        let resolved_yzx = (with-env $runtime_env {
            which yzx | get -o 0.path | default ""
        })
        let resolved_rg = (with-env $runtime_env {
            which rg | get -o 0.path | default ""
        })
        let resolved_dirname = (with-env $runtime_env {
            which dirname | get -o 0.path | default ""
        })

        if (
            (($runtime_env.PATH | get -o 0 | default "") == $fake_runtime_toolbin)
            and (($runtime_env.PATH | get -o 1 | default "") == $fake_runtime_bin)
            and ($resolved_yzx == ($fake_runtime_bin | path join "yzx"))
            and ($resolved_rg == ($fake_runtime_toolbin | path join "rg"))
            and ($resolved_dirname == "")
        ) {
            print "  ✅ Canonical Nu runtime env exports toolbin and bin while keeping runtime-only helpers off PATH"
            true
        } else {
            print $"  ❌ Unexpected canonical runtime PATH contract: (($runtime_env | to json -r)) resolved_yzx=($resolved_yzx) resolved_rg=($resolved_rg) resolved_dirname=($resolved_dirname)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
# Regression: the managed Helix wrapper must ignore legacy YAZELIX_DIR and infer its runtime from the wrapper path.
def test_yazelix_hx_ignores_legacy_runtime_alias_and_uses_wrapper_runtime_root [] {
    print "🧪 Testing yazelix_hx ignores legacy YAZELIX_DIR and uses the wrapper runtime root..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_hx_runtime_root_XXXXXX | str trim)
    let fake_bin = ($tmp_home | path join "fake_bin")
    let yzx_core_log = ($tmp_home | path join "yzx_core.log")
    let hx_log = ($tmp_home | path join "hx.log")
    let managed_config_path = ($tmp_home | path join "managed_config.toml")
    let helper_envelope = ({
        schema_version: 1
        command: "helix-materialization.generate"
        status: "ok"
        data: {
            generated_path: $managed_config_path
            template_path: ""
            user_config_merged: false
            reveal_binding_enforced: true
            import_notice: null
        }
        warnings: []
    } | to json -r)
    mkdir $fake_bin
    mkdir ($tmp_home | path join ".config" "yazelix" "user_configs")
    mkdir ($tmp_home | path join ".local" "share" "yazelix")
    "" | save --force --raw $managed_config_path

    let fake_yzx_core = ($fake_bin | path join "yzx_core")
    [
        "#!/bin/sh"
        $"printf '%s\\n' \"$@\" > '($yzx_core_log)'"
        $"printf '%s\\n' '($helper_envelope)'"
    ] | str join "\n" | save --force --raw $fake_yzx_core
    chmod +x $fake_yzx_core

    let fake_hx = ($fake_bin | path join "hx")
    [
        "#!/bin/sh"
        $"printf '%s\\n' \"$1\" > '($hx_log)'"
        $"printf '%s\\n' \"$2\" >> '($hx_log)'"
        $"printf '%s\\n' \"$3\" >> '($hx_log)'"
        "exit 0"
    ] | str join "\n" | save --force --raw $fake_hx
    chmod +x $fake_hx

    let result = (try {
        let wrapper_path = ($repo_root | path join "shells" "posix" "yazelix_hx.sh")
        let output = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: ($tmp_home | path join ".config")
            XDG_DATA_HOME: ($tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: ($tmp_home | path join ".config" "yazelix")
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
            YAZELIX_RUNTIME_DIR: null
            YAZELIX_DIR: "/hostile/legacy_runtime"
            YAZELIX_MANAGED_HELIX_BINARY: $fake_hx
            YAZELIX_YZX_CORE_BIN: $fake_yzx_core
        } {
            ^$wrapper_path "file.txt" | complete
        })

        let expected_helper_args = [
            "helix-materialization.generate"
            "--runtime-dir"
            $repo_root
            "--config-dir"
            ($tmp_home | path join ".config" "yazelix")
            "--state-dir"
            ($tmp_home | path join ".local" "share" "yazelix")
        ]
        let yzx_core_lines = if ($yzx_core_log | path exists) { open $yzx_core_log | lines } else { [] }
        let hx_lines = if ($hx_log | path exists) { open $hx_log | lines } else { [] }

        if (
            ($output.exit_code == 0)
            and ($yzx_core_lines == $expected_helper_args)
            and ($hx_lines == ["-c", $managed_config_path, "file.txt"])
        ) {
            print "  ✅ yazelix_hx now ignores legacy YAZELIX_DIR and resolves Helix config generation directly through the Rust helper"
            true
        } else {
            print $"  ❌ Unexpected wrapper result: exit=($output.exit_code) stdout=(($output.stdout | str trim)) stderr=(($output.stderr | str trim)) yzx_core=(($yzx_core_lines | to json -r)) hx=(($hx_lines | to json -r))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
# Defends: yzx import helix copies personal config into managed overrides and makes a backup on --force.
def test_yzx_import_helix_copies_personal_config_with_force_backups [] {
    print "🧪 Testing yzx import helix copies personal Helix config and backs up managed overrides on --force..."

    let repo_root = (get_repo_root)
    let tmp_home = (^mktemp -d /tmp/yazelix_import_helix_XXXXXX | str trim)
    let xdg_config_home = ($tmp_home | path join ".config")
    let native_helix_dir = ($xdg_config_home | path join "helix")
    let yazelix_config_dir = ($xdg_config_home | path join "yazelix")
    let managed_helix_dir = ($yazelix_config_dir | path join "user_configs" "helix")
    mkdir $native_helix_dir
    mkdir ($yazelix_config_dir | path join "user_configs")
    mkdir $managed_helix_dir

    let result = (try {
        '[editor]
cursorline = true
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let import_script = ($repo_root | path join "nushell" "scripts" "core" "yazelix.nu")
        let first_import = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $yazelix_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu -c $"use \"($import_script)\" *; yzx import helix" | complete
        })

        '[editor]
cursorline = false
' | save --force --raw ($managed_helix_dir | path join "config.toml")
        '[editor]
line-number = "relative"
' | save --force --raw ($native_helix_dir | path join "config.toml")

        let forced_import = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: $xdg_config_home
            YAZELIX_CONFIG_DIR: $yazelix_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
            YAZELIX_STATE_DIR: ($tmp_home | path join ".local" "share" "yazelix")
        } {
            ^nu -c $"use \"($import_script)\" *; yzx import helix --force" | complete
        })

        let managed_config_path = ($managed_helix_dir | path join "config.toml")
        let managed_config = (open $managed_config_path)
        let backups = (ls $managed_helix_dir | where name =~ 'config\.toml\.backup-')

        if (
            ($first_import.exit_code == 0)
            and ($forced_import.exit_code == 0)
            and (($managed_config.editor | get "line-number") == "relative")
            and (($backups | length) == 1)
        ) {
            print "  ✅ yzx import helix copies personal Helix config into user_configs/helix and backs up the previous managed file on --force"
            true
        } else {
            print $"  ❌ Unexpected helix import result: first_exit=($first_import.exit_code) force_exit=($forced_import.exit_code) managed=(($managed_config | to json -r)) backups=(($backups | length)) stderr=(($forced_import.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

export def run_helix_managed_config_contract_tests [] {
    [
        (test_generate_managed_helix_config_merges_user_config_and_enforces_reveal)
        (test_get_runtime_env_wraps_helix_with_managed_wrapper)
        (test_get_runtime_env_exports_curated_toolbin_and_keeps_runtime_local_yzx)
        (test_yazelix_hx_ignores_legacy_runtime_alias_and_uses_wrapper_runtime_root)
        (test_yzx_import_helix_copies_personal_config_with_force_backups)
    ]
}

export def main [] {
    let results = (run_helix_managed_config_contract_tests)
    let passed = ($results | where {|result| $result } | length)
    let total = ($results | length)

    if $passed == $total {
        print $"✅ All Helix managed config contract tests passed \(($passed)/($total)\)"
    } else {
        error make { msg: $"Helix managed config contract tests failed \(($passed)/($total)\)" }
    }
}
