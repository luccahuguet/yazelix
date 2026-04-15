#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ../core/yazelix.nu *
use ./yzx_test_helpers.nu [get_repo_config_dir setup_managed_config_fixture]

def run_doctor_command_for_fixture [fixture: record, command: string, extra_env?: record] {
    let base_env = {
        HOME: $fixture.tmp_home
        XDG_CONFIG_HOME: ($fixture.tmp_home | path join ".config")
        XDG_DATA_HOME: ($fixture.tmp_home | path join ".local" "share")
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
        YAZELIX_STATE_DIR: ($fixture.tmp_home | path join ".local" "share" "yazelix")
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

def write_test_legacy_yzx_wrapper [path: string] {
    [
        "#!/bin/sh"
        "# Stable Yazelix CLI entrypoint for external tools and editors."
        'exec "$(dirname "$0")/../shells/posix/yzx_cli.sh" "$@"'
    ] | str join "\n" | save --force --raw $path
    ^chmod +x $path
}

def setup_fake_profile_yzx [fixture: record] {
    let profile_yzx = ($fixture.tmp_home | path join ".nix-profile" "bin" "yzx")
    mkdir ($profile_yzx | path dirname)
    [
        "#!/bin/sh"
        "exit 0"
    ] | str join "\n" | save --force --raw $profile_yzx
    ^chmod +x $profile_yzx
    $profile_yzx
}

def setup_fake_home_manager_install_artifacts [fixture: record] {
    let fake_runtime = ($fixture.tmp_home | path join "fake_home_manager_package")
    let fake_runtime_bin = ($fake_runtime | path join "bin")
    let hm_store = ($fixture.tmp_home | path join "fake-home-manager-files")
    let hm_main = ($hm_store | path join ".config" "yazelix" "user_configs" "yazelix.toml")
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

    [
        "[Desktop Entry]"
        "Type=Application"
        "Name=Yazelix"
        $"Exec=\"($fixture.tmp_home | path join '.nix-profile' 'bin' 'yzx')\" desktop launch"
    ] | str join "\n" | save --force --raw $profile_desktop_path

    {
        runtime_root: $fake_runtime
        profile_yzx: $profile_yzx
        profile_desktop_path: $profile_desktop_path
    }
}

def doctor_output_reports_current_home_manager_install [stdout: string] {
    (
        ($stdout | str contains "Runtime/distribution capability: Home Manager-managed full runtime")
        and ($stdout | str contains "Home Manager owns the packaged Yazelix runtime path and update transition in this mode.")
        and ($stdout | str contains "Yazelix desktop entry uses the expected launcher path")
        and not ($stdout | str contains "A stale user-local yzx wrapper shadows the profile-owned Yazelix command")
        and not ($stdout | str contains "A stale host-shell yzx function or alias is shadowing the current profile command")
        and not ($stdout | str contains "Installed Yazelix runtime link is missing")
        and not ($stdout | str contains "Installed yzx command is missing")
        and not ($stdout | str contains "Installed yzx command is stale")
    )
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
            and ($stdout | str contains "Stale or unsupported yazelix.toml entries detected")
            and ($stdout | str contains "Unknown config field: core.stale_field")
            and ($stdout | str contains "yzx config reset")
        ) {
            print "  ✅ yzx doctor reports stale config fields through the narrowed v15 config surface"
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
            and ($stdout | str contains "Yazelix desktop entry does not use the expected launcher path")
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

# Defends: doctor uses the same manual stable-wrapper launcher policy as yzx desktop install.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_accepts_manual_stable_wrapper_desktop_entry [] {
    print "🧪 Testing yzx doctor accepts manual desktop entries that target the stable wrapper..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_manual_stable_wrapper_desktop_entry"
        ""
    )

    let result = (try {
        let wrapper_path = ($fixture.tmp_home | path join ".local" "bin" "yzx")
        let applications_dir = ($fixture.tmp_home | path join ".local" "share" "applications")
        let desktop_path = ($applications_dir | path join "com.yazelix.Yazelix.desktop")

        mkdir ($wrapper_path | path dirname)
        mkdir $applications_dir
        [
            "#!/bin/sh"
            "exit 0"
        ] | str join "\n" | save --force --raw $wrapper_path
        ^chmod +x $wrapper_path
        [
            "[Desktop Entry]"
            "Type=Application"
            "Name=Yazelix"
            $"Exec=\"($wrapper_path)\" desktop launch"
        ] | str join "\n" | save --force --raw $desktop_path

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose")
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix desktop entry uses the expected launcher path")
            and not ($stdout | str contains "Yazelix desktop entry does not use the expected launcher path")
        ) {
            print "  ✅ yzx doctor accepts manual desktop entries anchored to the stable wrapper"
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

# Regression: Home Manager installs use the profile yzx command and profile desktop entries without relying on user-local launcher artifacts.
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
            YAZELIX_INVOKED_YZX_PATH: $hm_install.profile_yzx
            YAZELIX_RUNTIME_DIR: $hm_install.runtime_root
        })
        let stdout = ($output.stdout | str trim)

        if (($output.exit_code == 0) and (doctor_output_reports_current_home_manager_install $stdout)) {
            print "  ✅ yzx doctor accepts the Home Manager profile yzx command and profile desktop entry without needing user-local launcher artifacts"
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
            YAZELIX_INVOKED_YZX_PATH: $hm_install.profile_yzx
            YAZELIX_RUNTIME_DIR: $hm_install.runtime_root
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "A stale user-local Yazelix desktop entry shadows the Home Manager desktop entry")
            and ($stdout | str contains "yzx desktop uninstall")
            and ($stdout | str contains "reapply your Home Manager configuration")
            and not ($stdout | str contains "refresh it with `yzx desktop install`")
            and not ($stdout | str contains "Yazelix desktop entry uses the expected launcher path")
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

# Regression: A stale legacy ~/.local/bin/yzx wrapper can shadow the profile-owned command after migration and must be reported clearly.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_reports_shadowing_manual_yzx_wrapper_for_profile_owner [] {
    print "🧪 Testing yzx doctor reports a stale manual yzx wrapper that shadows the profile-owned command..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_shadowed_manual_yzx_wrapper"
        ""
    )

    let result = (try {
        let hm_install = (setup_fake_home_manager_install_artifacts $fixture)
        let local_wrapper = ($fixture.tmp_home | path join ".local" "bin" "yzx")
        mkdir ($local_wrapper | path dirname)
        write_test_legacy_yzx_wrapper $local_wrapper

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" {
            YAZELIX_INVOKED_YZX_PATH: $local_wrapper
            YAZELIX_RUNTIME_DIR: $hm_install.runtime_root
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "A stale user-local yzx wrapper shadows the profile-owned Yazelix command")
            and ($stdout | str contains $local_wrapper)
            and ($stdout | str contains $hm_install.profile_yzx)
            and ($stdout | str contains "yzx home_manager prepare --apply")
            and ($stdout | str contains "remove the stale `~/.local/bin/yzx` wrapper")
        ) {
            print "  ✅ yzx doctor flags stale manual yzx wrapper shadowing before mixed-owner PATH state causes stale commands"
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

# Regression: a stale store-pinned host-shell yzx function can shadow the current profile command after a profile update.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_yzx_doctor_reports_stale_store_pinned_shell_shadowing [] {
    print "🧪 Testing yzx doctor reports stale store-pinned host-shell yzx shadowing..."

    let fixture = (setup_managed_config_fixture
        "yazelix_doctor_store_pinned_shell_shadowing"
        ""
    )

    let result = (try {
        let profile_yzx = (setup_fake_profile_yzx $fixture)
        let stale_store_yzx = "/nix/store/old-yazelix/bin/yzx"

        let output = (run_doctor_command_for_fixture $fixture "yzx doctor --verbose" {
            YAZELIX_INVOKED_YZX_PATH: $profile_yzx
            YAZELIX_REDIRECTED_FROM_STALE_YZX_PATH: $stale_store_yzx
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "A stale host-shell yzx function or alias is shadowing the current profile command")
            and ($stdout | str contains $stale_store_yzx)
            and ($stdout | str contains $profile_yzx)
            and ($stdout | str contains "command yzx")
        ) {
            print "  ✅ yzx doctor flags stale store-pinned host-shell shadowing instead of only checking user-local wrapper files"
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
            and ($stdout | str contains "Run `yzx doctor` to inspect generated-state issues")
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
        (test_yzx_doctor_reports_stale_desktop_entry_exec)
        (test_yzx_doctor_accepts_manual_stable_wrapper_desktop_entry)
        (test_yzx_doctor_accepts_home_manager_install_artifacts)
        (test_yzx_doctor_reports_shadowing_manual_desktop_entry_for_home_manager)
        (test_yzx_doctor_reports_shadowing_manual_yzx_wrapper_for_profile_owner)
        (test_yzx_doctor_reports_stale_store_pinned_shell_shadowing)
        (test_yzx_doctor_reports_missing_runtime_launch_assets)
        (test_yzx_doctor_respects_layout_override_for_shared_preflight)
        (test_yzx_doctor_omits_installer_artifact_checks_in_runtime_root_only_mode)
    ]
}
