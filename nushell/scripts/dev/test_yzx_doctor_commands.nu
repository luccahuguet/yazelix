#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ../core/yazelix.nu *
use ./yzx_test_helpers.nu [get_repo_config_dir repo_path setup_managed_config_fixture]

def run_doctor_command_for_fixture [fixture: record, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    }
    let merged_env = if ($extra_env | is-empty) {
        $base_env
    } else {
        $base_env | merge $extra_env
    }

    with-env $merged_env {
        ^nu -c $"use \"($fixture.yzx_script)\" *; ($command)" | complete
    }
}

def setup_fake_home_manager_install_artifacts [fixture: record] {
    let fake_runtime = ($fixture.tmp_home | path join "fake_home_manager_package")
    let fake_runtime_bin = ($fake_runtime | path join "bin")
    let hm_store = ($fixture.tmp_home | path join "fake-home-manager-files")
    let hm_main = ($hm_store | path join ".config" "yazelix" "user_configs" "yazelix.toml")
    let hm_pack = ($hm_store | path join ".config" "yazelix" "user_configs" "yazelix_packs.toml")
    let pack_config_path = ($fixture.user_config_dir | path join "yazelix_packs.toml")
    let profile_yzx = ($fixture.tmp_home | path join ".nix-profile" "bin" "yzx")
    let profile_desktop_path = ($fixture.tmp_home | path join ".nix-profile" "share" "applications" "yazelix.desktop")

    mkdir $fake_runtime
    mkdir $fake_runtime_bin
    mkdir ($hm_main | path dirname)
    mkdir ($profile_yzx | path dirname)
    mkdir ($profile_desktop_path | path dirname)

    cp ($fixture.repo_root | path join "yazelix_default.toml") ($fake_runtime | path join "yazelix_default.toml")
    cp ($fixture.repo_root | path join ".taplo.toml") ($fake_runtime | path join ".taplo.toml")
    ^ln -s ($fixture.repo_root | path join "config_metadata") ($fake_runtime | path join "config_metadata")
    cp ($fixture.repo_root | path join "yazelix_default.toml") $hm_main
    'enabled = ["git"]

[declarations]
git = ["gh"]
' | save --force --raw $hm_pack

    [
        "#!/bin/sh"
        "exit 0"
    ] | str join "\n" | save --force --raw ($fake_runtime_bin | path join "yzx")
    ^chmod +x ($fake_runtime_bin | path join "yzx")
    [
        "#!/bin/sh"
        "exit 0"
    ] | str join "\n" | save --force --raw ($fake_runtime_bin | path join "nu")
    ^chmod +x ($fake_runtime_bin | path join "nu")

    ^ln -s ($fake_runtime | path join "bin" "yzx") $profile_yzx
    rm -f $fixture.config_path
    ^ln -s $hm_main $fixture.config_path
    ^ln -s $hm_pack $pack_config_path

    [
        "[Desktop Entry]"
        "Type=Application"
        "Name=Yazelix"
        $"Exec=\"($fixture.tmp_home | path join '.nix-profile' 'bin' 'yzx')\" desktop launch"
    ] | str join "\n" | save --force --raw $profile_desktop_path

    { runtime_root: $fake_runtime }
}

def doctor_output_reports_current_home_manager_install [stdout: string] {
    (
        ($stdout | str contains "Runtime/distribution capability: Home Manager-managed full runtime")
        and ($stdout | str contains "Home Manager owns the packaged Yazelix runtime path and update transition in this mode.")
        and ($stdout | str contains "Yazelix desktop entry uses the stable launcher path")
        and not ($stdout | str contains "Installed Yazelix runtime link is missing")
        and not ($stdout | str contains "Installed yzx command is missing")
        and not ($stdout | str contains "Installed yzx command is stale")
    )
}

def seed_launch_profile_fixture_state [fixture: record, profile_path: string] {
    let state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix")
    let config_state_module = (repo_path "nushell" "scripts" "utils" "config_state.nu")
    let launch_state_module = (repo_path "nushell" "scripts" "utils" "launch_state.nu")

    let output = (with-env {
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
        YAZELIX_STATE_DIR: $state_dir
    } {
        let snippet = ([
            $"use \"($config_state_module)\" [compute_config_state record_materialized_state]"
            $"use \"($launch_state_module)\" [record_launch_profile_state]"
            "let state = (compute_config_state)"
            "record_materialized_state $state"
            $"record_launch_profile_state $state \"($profile_path)\""
            "{"
            "    combined_hash: $state.combined_hash"
            "    config_hash: $state.config_hash"
            "    lock_hash: $state.lock_hash"
            "    devenv_nix_hash: ($state.devenv_nix_hash? | default '')"
            "    devenv_yaml_hash: ($state.devenv_yaml_hash? | default '')"
            "    runtime_hash: ($state.runtime_hash? | default '')"
            "} | to json -r"
        ] | str join "\n")
        ^nu -c $snippet | complete
    })

    if $output.exit_code != 0 {
        error make {msg: $"Failed to seed launch-profile fixture state: (($output.stderr | default $output.stdout | str trim))"}
    }

    {
        state_dir: $state_dir
        state: ($output.stdout | lines | last | str trim | from json)
    }
}

# Defends: doctor warns on stale config fields with actionable guidance.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yzx_doctor_warns_on_stale_config_fields [] {
    print "🧪 Testing yzx doctor warns about stale config fields..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_stale_fields"
        ""
    )

    let result = (try {
        let stale_config = (
            open ($fixture.repo_root | path join "yazelix_default.toml")
            | upsert core.stale_field true
            | upsert packs.declarations.custom_pack ["hello"]
            | upsert packs.enabled ["custom_pack"]
        )
        $stale_config | to toml | save --force $fixture.config_path

        let output = with-env {
            HOME: $fixture.tmp_home
            XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fixture.config_dir
            YAZELIX_RUNTIME_DIR: $fixture.repo_root
        } {
            ^nu -c $"use \"($fixture.yzx_script)\" *; yzx doctor --verbose" | complete
        }
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Stale, unsupported, or migration-aware yazelix.toml entries detected")
            and ($stdout | str contains "Unknown config field: core.stale_field")
            and ($stdout | str contains "yzx config reset")
            and not ($stdout | str contains "packs.declarations.custom_pack")
        ) {
            print "  ✅ yzx doctor reports stale config fields without flagging custom pack declarations"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: doctor must still report config migrations when the Zellij plugin-health branch runs.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_reports_known_migration_inside_zellij_session [] {
    print "🧪 Testing yzx doctor still reports known migrations from inside a Zellij session..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_migration_zellij"
        '[zellij]
widget_tray = ["layout", "editor"]
')

    let result = (try {
        let fake_bin = ($fixture.tmp_home | path join "bin")
        mkdir $fake_bin
        [
            "#!/bin/sh"
            "printf '%s\\n' 'fake-zellij-no-session' >&2"
            "exit 1"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "zellij")
        ^chmod +x ($fake_bin | path join "zellij")

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" {
            ZELLIJ: "0"
            PATH: ([$fake_bin] | append $env.PATH)
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Known migration at zellij.widget_tray")
            and ($stdout | str contains "Safe preview: `yzx config migrate`")
            and ($stdout | str contains "Safe apply: `yzx config migrate --apply` or `yzx doctor --fix`")
            and (
                ($stdout | str contains "Yazelix pane-orchestrator")
                or ($stdout | str contains "Could not contact the Yazelix pane-orchestrator plugin")
            )
        ) {
            print "  ✅ yzx doctor reports config migrations with shared fix guidance even when plugin health executes inside Zellij"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: doctor fix applies safe config migrations.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_fix_applies_safe_config_migrations [] {
    print "🧪 Testing yzx doctor --fix applies safe config migrations..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_fix"
        '[zellij]
widget_tray = ["layout", "editor"]

[shell]
enable_atuin = true
')

    let result = (try {
        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --fix")
        let stdout = ($output.stdout | str trim)
        let rewritten = (open $fixture.config_path)
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Applied 2 config migration fix")
            and (($rewritten | get zellij.widget_tray) == ["editor"])
            and not (($rewritten.shell? | default {}) | columns | any {|column| $column == "enable_atuin" })
            and (($backups | length) == 1)
        ) {
            print "  ✅ yzx doctor --fix applies safe config migrations with backup"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) rewritten=($rewritten | to json -r) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: doctor fix splits legacy pack config into the supported sidecar path.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yzx_doctor_fix_splits_legacy_pack_config [] {
    print "🧪 Testing yzx doctor --fix relocates legacy pack config into user_configs/yazelix_packs.toml..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_fix_packs"
        '[packs]
enabled = ["git"]
user_packages = ["docker"]

[packs.declarations]
git = ["gh", "prek"]
'
        --legacy-root
    )

    let result = (try {
        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --fix")
        let stdout = ($output.stdout | str trim)
        let rewritten = (open ($fixture.user_config_dir | path join "yazelix.toml"))
        let pack_path = ($fixture.user_config_dir | path join "yazelix_packs.toml")
        let pack_rewritten = (if ($pack_path | path exists) { open $pack_path } else { null })
        let pack_rendered = (if $pack_rewritten == null { "<missing>" } else { $pack_rewritten | to json -r })
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Applied 1 config migration fix")
            and ($stdout | str contains "Wrote pack config to")
            and not ("packs" in ($rewritten | columns))
            and ($pack_rewritten.enabled == ["git"])
            and ($pack_rewritten.user_packages == ["docker"])
            and (($pack_rewritten.declarations | get git) == ["gh", "prek"])
            and (($backups | length) == 1)
            and not ($fixture.config_path | path exists)
        ) {
            print "  ✅ yzx doctor --fix relocates legacy pack ownership into user_configs"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) main=($rewritten | to json -r) pack=($pack_rendered) backups=(($backups | length))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: doctor reports stale desktop-entry launcher wiring as a diagnostic, not a launch blocker.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_yzx_doctor_reports_stale_desktop_entry_exec [] {
    print "🧪 Testing yzx doctor reports stale desktop entry Exec wiring..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_desktop_entry"
        ""
    )

    let result = (try {
        let applications_dir = ($fixture.tmp_home | path join ".local" "share" "applications")
        let desktop_path = ($applications_dir | path join "com.yazelix.Yazelix.desktop")
        mkdir $applications_dir
        [
            "[Desktop Entry]"
            "Type=Application"
            "Name=Yazelix"
            'Exec="/nix/store/old-yazelix-runtime/bin/yzx" desktop launch'
        ] | str join "\n" | save --force --raw $desktop_path

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix desktop entry does not use the stable launcher path")
            and ($stdout | str contains 'Repair with `yzx desktop install`.')
        ) {
            print "  ✅ yzx doctor reports stale desktop launcher wiring with focused repair guidance"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: Home Manager installs use the profile yzx command, profile desktop entries, and optional host shell hooks.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_accepts_home_manager_install_artifacts [] {
    print "🧪 Testing yzx doctor accepts the Home Manager profile yzx command and desktop entry..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_home_manager_install"
        ""
    )

    let result = (try {
        let hm_install = (setup_fake_home_manager_install_artifacts $fixture)

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" {
            YAZELIX_RUNTIME_DIR: $hm_install.runtime_root
        })
        let stdout = ($output.stdout | str trim)

        if (($output.exit_code == 0) and (doctor_output_reports_current_home_manager_install $stdout)) {
            print "  ✅ yzx doctor accepts the Home Manager profile yzx command, profile desktop entry, and optional host shell hooks"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: A stale manual desktop entry can shadow a correct Home Manager desktop entry and must not be treated as healthy.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_reports_shadowing_manual_desktop_entry_for_home_manager [] {
    print "🧪 Testing yzx doctor reports a stale manual desktop entry that shadows the Home Manager desktop entry..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_home_manager_shadowed_desktop_entry"
        ""
    )

    let result = (try {
        let hm_install = (setup_fake_home_manager_install_artifacts $fixture)

        let local_desktop_path = ($fixture.tmp_home | path join ".local" "share" "applications" "com.yazelix.Yazelix.desktop")
        mkdir ($local_desktop_path | path dirname)
        [
            "[Desktop Entry]"
            "Type=Application"
            "Name=Yazelix"
            'Exec="/nix/store/old-yazelix-runtime/bin/yzx" desktop launch'
        ] | str join "\n" | save --force --raw $local_desktop_path

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" {
            YAZELIX_RUNTIME_DIR: $hm_install.runtime_root
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry")
            and ($stdout | str contains "yzx desktop uninstall")
            and ($stdout | str contains "yzx desktop install")
            and not ($stdout | str contains "Yazelix desktop entry uses the stable launcher path")
        ) {
            print "  ✅ yzx doctor flags a stale manual desktop entry that would shadow the Home Manager launcher"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: doctor surfaces shared runtime preflight failures for missing runtime launch assets.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_reports_missing_runtime_launch_assets [] {
    print "🧪 Testing yzx doctor reports missing runtime launch assets through the shared runtime checker..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_runtime_preflight"
        '[terminal]
manage_terminals = false
terminals = ["ghostty"]
'
    )

    let result = (try {
        let fake_runtime = ($fixture.tmp_home | path join "runtime")
        let fake_state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix")
        let fake_bin = ($fixture.tmp_home | path join "bin")
        let fake_runtime_link = ($fake_state_dir | path join "runtime" "current")
        mkdir $fake_runtime
        mkdir $fake_state_dir
        mkdir ($fake_state_dir | path join "runtime")
        mkdir $fake_bin
        cp ($fixture.repo_root | path join ".taplo.toml") ($fake_runtime | path join ".taplo.toml")
        cp ($fixture.repo_root | path join "yazelix_default.toml") ($fake_runtime | path join "yazelix_default.toml")
        ^ln -s ($fixture.repo_root | path join "config_metadata") ($fake_runtime | path join "config_metadata")
        ^ln -s $fake_runtime $fake_runtime_link

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "ghostty")
        ^chmod +x ($fake_bin | path join "ghostty")

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" {
            YAZELIX_RUNTIME_DIR: $fake_runtime
            YAZELIX_STATE_DIR: $fake_state_dir
            PATH: ([$fake_bin] | append $env.PATH)
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Missing Yazelix launch script")
            and ($stdout | str contains "Missing Yazelix generated Zellij layout")
            and ($stdout | str contains "Run `yzx refresh` to regenerate layouts")
        ) {
            print "  ✅ yzx doctor reuses the shared runtime checker for missing launch assets"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Regression: doctor must resolve the same expected layout override path as startup.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_respects_layout_override_for_shared_preflight [] {
    print "🧪 Testing yzx doctor respects YAZELIX_LAYOUT_OVERRIDE for shared layout preflight..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_layout_override"
        '[terminal]
manage_terminals = false
terminals = ["ghostty"]
'
    )

    let result = (try {
        let fake_state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix")
        let layouts_dir = ($fake_state_dir | path join "configs" "zellij" "layouts")
        let fake_bin = ($fixture.tmp_home | path join "bin")
        mkdir $layouts_dir
        mkdir $fake_bin
        "" | save --force --raw ($layouts_dir | path join "yzx_no_side.kdl")

        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw ($fake_bin | path join "ghostty")
        ^chmod +x ($fake_bin | path join "ghostty")

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" {
            YAZELIX_STATE_DIR: $fake_state_dir
            YAZELIX_LAYOUT_OVERRIDE: "yzx_no_side"
            PATH: ([$fake_bin] | append $env.PATH)
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix generated Zellij layout is present")
            and not ($stdout | str contains $"Missing Yazelix generated Zellij layout: ($layouts_dir | path join 'yzx_side.kdl')")
        ) {
            print "  ✅ yzx doctor uses the same layout-override resolution as startup"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: launch-profile freshness diagnostics use the canonical config-state and launch-state contract.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_reports_launch_profile_freshness_states [] {
    print "🧪 Testing yzx doctor reports healthy, stale, and missing cached launch-profile states..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_launch_profile"
        '[terminal]
terminals = ["ghostty"]
'
    )

    let result = (try {
        let state_dir = ($fixture.tmp_home | path join ".local" "share" "yazelix")
        let profile_dir = ($fixture.tmp_home | path join "cached_profile")
        let launch_state_path = ($state_dir | path join "state" "launch_state.json")
        let rebuild_state_path = ($state_dir | path join "state" "rebuild_hash")
        mkdir $profile_dir

        let seeded = (seed_launch_profile_fixture_state $fixture $profile_dir)
        let doctor_env = {YAZELIX_STATE_DIR: $state_dir}

        let healthy_output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" $doctor_env)
        let healthy_stdout = ($healthy_output.stdout | str trim)

        '[terminal]
terminals = ["kitty"]
' | save --force --raw $fixture.config_path
        let stale_config_output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" $doctor_env)
        let stale_config_stdout = ($stale_config_output.stdout | str trim)

        '[terminal]
terminals = ["ghostty"]
' | save --force --raw $fixture.config_path
        {
            config_hash: $seeded.state.config_hash
            lock_hash: "stale-lock-hash"
            devenv_nix_hash: $seeded.state.devenv_nix_hash
            devenv_yaml_hash: $seeded.state.devenv_yaml_hash
            runtime_hash: $seeded.state.runtime_hash
        } | to json | save --force $rebuild_state_path
        let stale_inputs_output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" $doctor_env)
        let stale_inputs_stdout = ($stale_inputs_output.stdout | str trim)

        {
            config_hash: $seeded.state.config_hash
            lock_hash: $seeded.state.lock_hash
            devenv_nix_hash: $seeded.state.devenv_nix_hash
            devenv_yaml_hash: $seeded.state.devenv_yaml_hash
            runtime_hash: $seeded.state.runtime_hash
        } | to json | save --force $rebuild_state_path
        rm --force $launch_state_path
        let missing_output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" $doctor_env)
        let missing_stdout = ($missing_output.stdout | str trim)

        if (
            ($healthy_output.exit_code == 0)
            and ($healthy_stdout | str contains "Cached launch profile matches the current rebuild-relevant config and tracked inputs")
            and ($healthy_stdout | str contains $profile_dir)
            and ($stale_config_output.exit_code == 0)
            and ($stale_config_stdout | str contains "Cached launch profile is stale because rebuild-relevant config changed")
            and ($stale_config_stdout | str contains "Run `yzx refresh` before relying on `yzx enter --reuse`")
            and ($stale_inputs_output.exit_code == 0)
            and ($stale_inputs_stdout | str contains "Cached launch profile is stale because tracked runtime/devenv inputs changed")
            and ($missing_output.exit_code == 0)
            and ($missing_stdout | str contains "No verified cached launch profile exists for the current rebuild-relevant config and tracked inputs")
        ) {
            print "  ✅ yzx doctor now classifies cached launch-profile health across healthy, stale-config, stale-input, and missing-profile states"
            true
        } else {
            print $"  ❌ Unexpected launch-profile doctor output:\nHEALTHY:\n($healthy_stdout)\n\nSTALE CONFIG:\n($stale_config_stdout)\n\nSTALE INPUTS:\n($stale_inputs_stdout)\n\nMISSING:\n($missing_stdout)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: doctor must not imply runtime-root-only sessions own installer artifact checks or installer repair surfaces.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_omits_installer_artifact_checks_in_runtime_root_only_mode [] {
    print "🧪 Testing yzx doctor omits installer-owned artifact checks in runtime-root-only mode..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_runtime_root_only"
        ""
    )

    let result = (try {
        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Runtime/distribution capability: runtime-root-only mode")
            and ($stdout | str contains "This Yazelix session has a runtime root but no package-manager-owned distribution surface.")
            and not ($stdout | str contains "Installed Yazelix runtime link is missing")
            and not ($stdout | str contains "Installed yzx command is missing")
            and not ($stdout | str contains "Installer-owned runtime artifact checks skipped")
        ) {
            print "  ✅ yzx doctor now reports the narrowed runtime-root-only tier without implying installer-owned repair or installer artifact checks"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=(($output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

export def run_doctor_canonical_tests [] {
    [
        (test_yzx_doctor_warns_on_stale_config_fields)
        (test_yzx_doctor_reports_known_migration_inside_zellij_session)
        (test_yzx_doctor_fix_applies_safe_config_migrations)
        (test_yzx_doctor_fix_splits_legacy_pack_config)
        (test_yzx_doctor_reports_stale_desktop_entry_exec)
        (test_yzx_doctor_accepts_home_manager_install_artifacts)
        (test_yzx_doctor_reports_shadowing_manual_desktop_entry_for_home_manager)
        (test_yzx_doctor_reports_missing_runtime_launch_assets)
        (test_yzx_doctor_respects_layout_override_for_shared_preflight)
        (test_yzx_doctor_reports_launch_profile_freshness_states)
        (test_yzx_doctor_omits_installer_artifact_checks_in_runtime_root_only_mode)
    ]
}
