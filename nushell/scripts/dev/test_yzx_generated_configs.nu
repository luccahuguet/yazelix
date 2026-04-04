#!/usr/bin/env nu
# Test lane: default
# Defends: docs/specs/test_suite_governance.md

use ./yzx_test_helpers.nu [get_repo_config_dir repo_path setup_managed_config_fixture]
use ../setup/yazi_config_merger.nu [generate_merged_yazi_config]
use ../setup/zellij_config_merger.nu [generate_merged_zellij_config]
use ../utils/terminal_launcher.nu [build_launch_command resolve_terminal_config]
use ../utils/terminal_configs.nu [
    generate_all_terminal_configs
]

def run_parse_yazelix_config_probe [fixture: record, extra_env: record = {}] {
    with-env ({
        HOME: $fixture.tmp_home
        YAZELIX_CONFIG_DIR: $fixture.config_dir
        YAZELIX_RUNTIME_DIR: $fixture.repo_root
    } | merge $extra_env) {
        ^nu -c $"use \"($fixture.repo_root | path join "nushell" "scripts" "utils" "config_parser.nu")\" [parse_yazelix_config]; parse_yazelix_config" | complete
    }
}

# Defends: generated terminal configs do not silently take over user overrides.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_all_terminal_configs_keeps_terminal_overrides_opt_in [] {
    print "🧪 Testing bundled terminal config generation keeps user terminal overrides opt-in..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_override_scaffold_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    mkdir $fake_home

    let result = (try {
        '[terminal]
terminals = ["ghostty", "kitty", "alacritty", "wezterm", "foot"]
' | save --force --raw $config_path

        with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_DIR: ($fake_home | path join ".config" "yazelix")
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            generate_all_terminal_configs $runtime_root
        }

        let override_root = ($fake_home | path join ".config" "yazelix" "user_configs" "terminal")
        let ghostty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "config"))
        let kitty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "kitty" "kitty.conf"))
        let alacritty_entrypoint = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "alacritty" "alacritty.toml"))
        let wezterm_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "wezterm" ".wezterm.lua"))
        let foot_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "foot" "foot.ini"))

        if (
            not ($override_root | path exists)
            and ($ghostty_config | str contains $"config-file = ?\"($override_root | path join "ghostty")\"")
            and not ($kitty_config | str contains "include ~/.config/yazelix")
            and ($kitty_config | str contains $"Create ($override_root | path join "kitty.conf") if you want terminal-native Kitty tweaks.")
            and ($alacritty_entrypoint | str contains $"\"($fake_home)/.local/share/yazelix/configs/terminal_emulators/alacritty/alacritty_base.toml\"")
            and not ($alacritty_entrypoint | str contains $"\"($override_root | path join "alacritty.toml")\"")
            and ($alacritty_entrypoint | str contains $"Create ($override_root | path join "alacritty.toml") if you want terminal-native Alacritty tweaks.")
            and not ($ghostty_config | str contains "start_yazelix.sh")
            and not ($kitty_config | str contains "start_yazelix.sh")
            and not ($alacritty_entrypoint | str contains "start_yazelix.sh")
            and not ($wezterm_config | str contains "start_yazelix.sh")
            and not ($foot_config | str contains "start_yazelix.sh")
            and ($foot_config | str contains "[colors-dark]")
            and not ($foot_config | str contains "[colors]\n")
        ) {
            print "  ✅ Terminal config generation keeps user terminal overrides opt-in and keeps startup out of generated terminal configs"
            true
        } else {
            print "  ❌ Terminal config generation still scaffolded or imported unexpected user override files"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: terminal override imports must ignore Yazelix runtime roots.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
def test_terminal_override_imports_ignore_yazelix_dir_runtime_root [] {
    print "🧪 Testing terminal override imports ignore YAZELIX_DIR runtime roots..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_override_path_boundary_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let fake_runtime_root = ($tmpdir | path join "runtime_root")
    let fake_config_dir = ($fake_home | path join ".config" "yazelix")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    let terminal_configs_script = ($runtime_root | path join "nushell" "scripts" "utils" "terminal_configs.nu")
    mkdir $fake_home
    mkdir $fake_runtime_root
    mkdir ($fake_config_dir | path join "user_configs" "terminal")

    let result = (try {
        '[terminal]
terminals = ["ghostty", "kitty", "alacritty"]
' | save --force --raw $config_path

        '# user-owned ghostty override
' | save --force --raw ($fake_home | path join ".config" "yazelix" "user_configs" "terminal" "ghostty")
        '# user-owned kitty override
' | save --force --raw ($fake_home | path join ".config" "yazelix" "user_configs" "terminal" "kitty.conf")
        '# user-owned alacritty override
' | save --force --raw ($fake_home | path join ".config" "yazelix" "user_configs" "terminal" "alacritty.toml")

        let command_output = (with-env {
            HOME: $fake_home
            XDG_CONFIG_HOME: ($fake_home | path join ".config")
            YAZELIX_CONFIG_DIR: $fake_config_dir
            YAZELIX_DIR: $fake_runtime_root
            YAZELIX_RUNTIME_DIR: $runtime_root
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            ^nu -c $"use \"($terminal_configs_script)\" [generate_all_terminal_configs]; generate_all_terminal_configs \"($runtime_root)\"" | complete
        })

        let expected_override_root = ($fake_home | path join ".config" "yazelix" "user_configs" "terminal")
        let misplaced_override_root = ($fake_runtime_root | path join "user_configs" "terminal")
        let ghostty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty" "config"))
        let kitty_config = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "kitty" "kitty.conf"))
        let alacritty_entrypoint = (open --raw ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "alacritty" "alacritty.toml"))

        if (
            ($command_output.exit_code == 0)
            and ($expected_override_root | path exists)
            and (($expected_override_root | path join "ghostty") | path exists)
            and (($expected_override_root | path join "kitty.conf") | path exists)
            and (($expected_override_root | path join "alacritty.toml") | path exists)
            and ($ghostty_config | str contains $"config-file = ?\"($expected_override_root | path join "ghostty")\"")
            and ($kitty_config | str contains $"include ($expected_override_root | path join "kitty.conf")")
            and ($alacritty_entrypoint | str contains $"\"($expected_override_root | path join "alacritty.toml")\"")
            and not ($misplaced_override_root | path exists)
        ) {
            print "  ✅ Terminal override imports stay under HOME/.config/yazelix/user_configs even when YAZELIX_DIR points elsewhere"
            true
        } else {
            print $"  ❌ Unexpected override destinations: exit=($command_output.exit_code) expected_root_exists=(($expected_override_root | path exists)) misplaced_root_exists=(($misplaced_override_root | path exists)) expected_root=($expected_override_root) misplaced_root=($misplaced_override_root) stderr=(($command_output.stderr | str trim))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: managed terminal wrappers must not leak Yazelix-only config-mode args into terminal binaries.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_managed_wrapper_launch_command_does_not_forward_config_mode_flag [] {
    print "🧪 Testing managed terminal wrapper launch command keeps config-mode internal..."

    try {
        let launch_cmd = (build_launch_command {
            terminal: "ghostty"
            name: "Ghostty"
            command: "yazelix-ghostty"
            use_wrapper: true
        } null "yazelix" "/tmp" false)

        if (
            ($launch_cmd | str contains 'yazelix-ghostty')
            and not ($launch_cmd | str contains '--config-mode')
            and ($launch_cmd | str contains '--working-directory="/tmp"')
        ) {
            print "  ✅ Managed wrapper launch command now keeps config-mode internal to the wrapper"
            true
        } else {
            print $"  ❌ Unexpected managed wrapper launch command: ($launch_cmd)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

# Regression: Ghostty generated effect shader references must point at files the builder emits.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_ghostty_config_references_existing_effect_shaders [] {
    print "🧪 Testing Ghostty generated effect shader references match emitted files..."

    let tmpdir = (^mktemp -d /tmp/yazelix_ghostty_effect_shaders_XXXXXX | str trim)
    let fake_home = ($tmpdir | path join "home")
    let config_path = ($tmpdir | path join "yazelix.toml")
    let runtime_root = (pwd)
    mkdir $fake_home

    let result = (try {
        '[terminal]
terminals = ["ghostty"]
ghostty_trail_color = "reef"
ghostty_trail_effect = "tail"
ghostty_mode_effect = "ripple_rectangle"
' | save --force --raw $config_path

        with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_DIR: ($fake_home | path join ".config" "yazelix")
            YAZELIX_CONFIG_OVERRIDE: $config_path
        } {
            generate_all_terminal_configs $runtime_root
        }

        let ghostty_root = ($fake_home | path join ".local" "share" "yazelix" "configs" "terminal_emulators" "ghostty")
        let ghostty_config = (open --raw ($ghostty_root | path join "config"))
        let tail_shader = ($ghostty_root | path join "shaders" "generated_effects" "tail.glsl")
        let ripple_shader = ($ghostty_root | path join "shaders" "generated_effects" "ripple_rectangle.glsl")

        if (
            ($ghostty_config | str contains "custom-shader = ./shaders/generated_effects/tail.glsl")
            and ($ghostty_config | str contains "custom-shader = ./shaders/generated_effects/ripple_rectangle.glsl")
            and ($tail_shader | path exists)
            and ($ripple_shader | path exists)
        ) {
            print "  ✅ Ghostty generated config now points at generated effect shader filenames that exist"
            true
        } else {
            print $"  ❌ Missing Ghostty effect shader wiring: tail_exists=(($tail_shader | path exists)) ripple_exists=(($ripple_shader | path exists))"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: removed ascii mode fails with migration guidance.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_parse_yazelix_config_rejects_legacy_ascii_mode_with_migration_guidance [] {
    print "🧪 Testing parse_yazelix_config rejects legacy [ascii].mode with one clean migration path..."

    let fixture = (setup_managed_config_fixture
        "yazelix_welcome_style_legacy"
        '[ascii]
mode = "animated"
'
    )

    let result = (try {
        let parser_result = (run_parse_yazelix_config_probe $fixture)

        let stderr = ($parser_result.stderr | str trim)

        if (
            ($parser_result.exit_code != 0)
            and ($stderr | str contains "Known migration at ascii")
            and ($stderr | str contains "Replace legacy [ascii].mode with core.welcome_style")
            and ($stderr | str contains "yzx config migrate --apply")
            and not ($stderr | str contains "Unknown config field at ascii")
        ) {
            print "  ✅ Legacy [ascii].mode now points at one clean migration path during startup"
            true
        } else {
            print $"  ❌ Unexpected parser result: exit=($parser_result.exit_code) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: config parsing stays read-only and does not auto-apply migrations.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_parse_yazelix_config_does_not_auto_apply_safe_migrations [] {
    print "🧪 Testing parse_yazelix_config keeps safe migration rewrites out of read paths..."

    let fixture = (setup_managed_config_fixture
        "yazelix_parser_no_auto_apply"
        '[shell]
enable_atuin = true
'
    )

    let result = (try {
        let parser_result = (run_parse_yazelix_config_probe $fixture)
        let stderr = ($parser_result.stderr | str trim)
        let updated = (open $fixture.config_path)
        let backups = (ls $fixture.user_config_dir | where name =~ 'yazelix\.toml\.backup-')

        if (
            ($parser_result.exit_code != 0)
            and ($stderr | str contains "Known migration at shell.enable_atuin")
            and ($stderr | str contains "yzx config migrate --apply")
            and ($updated.shell.enable_atuin? | default false)
            and (($backups | length) == 0)
        ) {
            print "  ✅ parse_yazelix_config still fails cleanly without rewriting safe migration cases"
            true
        } else {
            print $"  ❌ Unexpected parser result: exit=($parser_result.exit_code) stderr=($stderr) updated=($updated | to json -r) backups=(($backups | length))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Invariant: split default config surfaces are bootstrapped when missing.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=1 total=7/10
def test_parse_yazelix_config_bootstraps_split_default_surfaces [] {
    print "🧪 Testing parse_yazelix_config bootstraps both default config surfaces on first run..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_pack_bootstrap_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")

    let result = (try {
        let parsed = (with-env {
            HOME: $tmp_home
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            parse_yazelix_config
        })

        let user_config_dir = ($temp_config_dir | path join "user_configs")
        let main_exists = (($user_config_dir | path join "yazelix.toml") | path exists)
        let pack_exists = (($user_config_dir | path join "yazelix_packs.toml") | path exists)
        let generated_main = (if $main_exists { open --raw ($user_config_dir | path join "yazelix.toml") } else { "" })
        let generated_packs = (if $pack_exists { open --raw ($user_config_dir | path join "yazelix_packs.toml") } else { "" })

        if (
            $main_exists
            and $pack_exists
            and ($generated_main | str contains "Pack configuration lives in ~/.config/yazelix/user_configs/yazelix_packs.toml")
            and ($generated_packs | str contains "[declarations]")
            and ((($parsed.pack_declarations | default {}) | columns | length) > 0)
        ) {
            print "  ✅ First-run bootstrap now materializes both user_configs TOML surfaces from runtime defaults"
            true
        } else {
            print $"  ❌ Unexpected result: main_exists=($main_exists) pack_exists=($pack_exists) parsed=($parsed | select pack_names pack_declarations | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Defends: legacy root config is rejected unless the user explicitly allows migration.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_parse_yazelix_config_rejects_legacy_root_config_without_confirmation [] {
    print "🧪 Testing parse_yazelix_config rejects legacy root-level config files when it cannot prompt..."

    let fixture = (setup_managed_config_fixture
        "yazelix_legacy_root_no_prompt"
        '[shell]
default_shell = "bash"
'
        --legacy-root
    )

    let result = (try {
        let parser_result = (run_parse_yazelix_config_probe $fixture)

        let stderr = ($parser_result.stderr | str trim)

        if (
            ($parser_result.exit_code != 0)
            and ($stderr | str contains "legacy root-level config files but could not prompt for")
            and ($stderr | str contains "confirmation")
            and ($stderr | str contains "yzx doctor --fix")
            and ($fixture.config_path | path exists)
            and not (($fixture.user_config_dir | path join "yazelix.toml") | path exists)
        ) {
            print "  ✅ Legacy root-level config now fails clearly instead of silently relocating in non-interactive startup"
            true
        } else {
            print $"  ❌ Unexpected parser result: exit=($parser_result.exit_code) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: explicit legacy-root migration uses the managed relocation path.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_parse_yazelix_config_relocates_legacy_root_config_when_explicitly_allowed [] {
    print "🧪 Testing parse_yazelix_config relocates legacy root-level config when explicitly allowed..."

    let fixture = (setup_managed_config_fixture
        "yazelix_legacy_root_allowed"
        '[shell]
default_shell = "bash"
'
        --legacy-root
    )

    let result = (try {
        let parsed = (with-env { YAZELIX_ACCEPT_USER_CONFIG_RELOCATION: "true" } {
            use ../utils/config_parser.nu [parse_yazelix_config]
            with-env {
                HOME: $fixture.tmp_home
                YAZELIX_CONFIG_DIR: $fixture.config_dir
                YAZELIX_RUNTIME_DIR: $fixture.repo_root
                YAZELIX_ACCEPT_USER_CONFIG_RELOCATION: "true"
            } {
                parse_yazelix_config
            }
        })

        let relocated_path = ($fixture.user_config_dir | path join "yazelix.toml")

        if (
            (($parsed.default_shell? | default "") == "bash")
            and ($relocated_path | path exists)
            and not ($fixture.config_path | path exists)
        ) {
            print "  ✅ Explicitly allowed relocation moves the legacy root config into user_configs"
            true
        } else {
            print $"  ❌ Unexpected result: parsed=($parsed | to json -r) relocated_exists=(($relocated_path | path exists))"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: legacy inline packs are rejected with migration guidance.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_parse_yazelix_config_rejects_legacy_main_file_packs_with_migration_guidance [] {
    print "🧪 Testing parse_yazelix_config rejects legacy [packs] in yazelix.toml and points users at migrate..."

    let fixture = (setup_managed_config_fixture
        "yazelix_pack_legacy_main"
        '[packs]
enabled = ["git"]
user_packages = ["docker"]

[packs.declarations]
git = ["gh", "prek"]
'
    )

    let result = (try {
        let parser_result = (run_parse_yazelix_config_probe $fixture)

        let stderr = ($parser_result.stderr | str trim)

        if (
            ($parser_result.exit_code != 0)
            and ($stderr | str contains "Known migration at packs")
            and ($stderr | str contains "yzx config migrate --apply")
        ) {
            print "  ✅ Legacy pack settings are now blocked with shared migration guidance"
            true
        } else {
            print $"  ❌ Unexpected parser result: exit=($parser_result.exit_code) stderr=($stderr)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fixture.tmp_home
    $result
}

# Defends: split pack ownership conflicts fail fast.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_parse_yazelix_config_rejects_split_pack_ownership [] {
    print "🧪 Testing parse_yazelix_config fails fast when yazelix.toml and yazelix_packs.toml both define packs..."

    let tmpdir = (^mktemp -d /tmp/yazelix_pack_sidecar_conflict_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let pack_path = ($tmpdir | path join "yazelix_packs.toml")
        let parser_script = (repo_path "nushell" "scripts" "utils" "config_parser.nu")

        '[packs]
enabled = ["git"]
' | save --force --raw $config_path

        'enabled = ["rust"]
' | save --force --raw $pack_path

        let output = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            ^nu -c $"source \"($parser_script)\"; try { parse_yazelix_config | ignore } catch {|err| print $err.msg }" | complete
        })
        let stdout = ($output.stdout | str trim)

        if (
            ($output.exit_code == 0)
            and ($stdout | str contains "Yazelix found pack settings in both yazelix.toml and yazelix_packs.toml.")
            and ($stdout | str contains "fully owns pack settings")
            and ($stdout | str contains "Failure class: config problem.")
        ) {
            print "  ✅ parse_yazelix_config fails fast on ambiguous split pack ownership"
            true
        } else {
            print $"  ❌ Unexpected result: exit=($output.exit_code) stdout=($stdout) stderr=($output.stderr | str trim)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: user terminal mode requires a real terminal config path.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_user_mode_requires_real_terminal_config [] {
    print "🧪 Testing terminal.config_mode = user fails fast when the user terminal config is missing..."

    let fake_home = (^mktemp -d /tmp/yazelix_user_mode_missing_XXXXXX | str trim)

    let result = (try {
        let message = (with-env { HOME: $fake_home } {
            try {
                resolve_terminal_config "ghostty" "user"
                "unexpected-success"
            } catch {|err|
                $err.msg
            }
        })

        if ($message | str contains "terminal.config_mode = user requires a real ghostty user config") {
            print "  ✅ user mode fails clearly instead of silently falling back to Yazelix-managed config"
            true
        } else {
            print $"  ❌ Unexpected message: ($message)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $fake_home
    $result
}

# Defends: removed auto terminal config mode is rejected by schema validation.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_config_schema_rejects_removed_auto_terminal_config_mode [] {
    print "🧪 Testing config schema rejects the removed terminal.config_mode = auto value..."

    let tmpdir = (^mktemp -d /tmp/yazelix_terminal_mode_schema_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '[terminal]
config_mode = "auto"
' | save --force --raw $config_path

        let findings = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_schema.nu [validate_enum_values]
            validate_enum_values (open $config_path)
        })
        let mode_findings = ($findings | where path == "terminal.config_mode")

        if (
            (($mode_findings | length) == 1)
            and (($mode_findings | get 0.kind) == "invalid_enum")
        ) {
            print "  ✅ Config schema rejects the removed auto terminal config mode"
            true
        } else {
            print $"  ❌ Unexpected findings: ($mode_findings | to json -r)"
            false
        }
    } catch {|err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: removed layout widget config is rejected by schema validation.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_config_schema_rejects_removed_layout_widget [] {
    print "🧪 Testing config schema rejects the removed zellij layout widget..."

    let tmpdir = (^mktemp -d /tmp/yazelix_widget_tray_schema_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        '[zellij]
widget_tray = ["layout", "editor"]
' | save --force --raw $config_path

        let findings = (with-env { YAZELIX_CONFIG_OVERRIDE: $config_path } {
            use ../utils/config_schema.nu [validate_enum_values]
            validate_enum_values (open $config_path)
        })
        let tray_findings = ($findings | where path == "zellij.widget_tray")

        if (
            (($tray_findings | length) == 1)
            and (($tray_findings | get 0.kind) == "invalid_enum")
            and ((($tray_findings | get 0.message) | str contains "layout"))
        ) {
            print "  ✅ Config schema rejects the removed layout widget entry"
            true
        } else {
            print $"  ❌ Unexpected findings: ($tray_findings | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

def write_minimal_user_zellij_config [fake_home: string] {
    let zellij_config_dir = ($fake_home | path join ".config" "yazelix" "user_configs" "zellij")
    let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
    mkdir $zellij_config_dir
    'keybinds { normal { bind "f1" { WriteChars "fixture"; } } }'
        | save --force --raw $zellij_config_path
}

def write_legacy_native_zellij_config [fake_home: string] {
    let zellij_config_dir = ($fake_home | path join ".config" "zellij")
    let zellij_config_path = ($zellij_config_dir | path join "config.kdl")
    mkdir $zellij_config_dir
    'scroll_buffer_size 12345'
        | save --force --raw $zellij_config_path
}

def run_merged_zellij_config_in_fake_home [tmpdir: string, extra_env: record = {}, extra_output?: closure] {
    let out_dir = ($tmpdir | path join "out")
    let fake_home = ($tmpdir | path join "home")
    let fake_config_dir = ($fake_home | path join ".config" "yazelix")

    with-env ({
        HOME: $fake_home
        XDG_CONFIG_HOME: ($fake_home | path join ".config")
        YAZELIX_CONFIG_DIR: $fake_config_dir
        YAZELIX_TEST_OUT_DIR: $out_dir
    } | merge $extra_env) {
        let root = (get_repo_config_dir)
        generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
        {
            config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
        } | merge (if $extra_output == null { {} } else { do $extra_output })
    }
}

# Regression: legacy Yazi overrides are relocated into the managed surface.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_yazi_config_relocates_legacy_user_overrides [] {
    print "🧪 Testing merged Yazi config relocates legacy user overrides into user_configs/yazi..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_yazi_user_configs_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    let legacy_user_dir = ($temp_config_dir | path join "configs" "yazi" "user")
    let canonical_user_dir = ($temp_config_dir | path join "user_configs" "yazi")
    mkdir ($tmp_home | path join ".config")
    mkdir $temp_config_dir
    mkdir $legacy_user_dir

    let result = (try {
        '-- legacy user code
return "yazi-user-marker"
' | save --force --raw ($legacy_user_dir | path join "init.lua")

        let merged_init = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: ($tmp_home | path join ".config")
            XDG_DATA_HOME: ($tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            generate_merged_yazi_config $repo_root --quiet | ignore
            open --raw ($tmp_home | path join ".local" "share" "yazelix" "configs" "yazi" "init.lua")
        })

        if (
            (($canonical_user_dir | path join "init.lua") | path exists)
            and not (($legacy_user_dir | path join "init.lua") | path exists)
            and ($merged_init | str contains "yazi-user-marker")
            and ($merged_init | str contains "~/.config/yazelix/user_configs/yazi/init.lua")
        ) {
            print "  ✅ Legacy Yazi user overrides relocate into user_configs and still merge"
            true
        } else {
            print "  ❌ Unexpected result: Yazi legacy override did not relocate or merge correctly"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Regression: generated Yazi Starship plugin config must stay writable so repeated Yazi regeneration does not crash at startup.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_yazi_config_syncs_starship_plugin_config [] {
    print "🧪 Testing merged Yazi config syncs the bundled Starship plugin config into the managed Yazi surface across repeated regenerations..."

    let repo_root = (get_repo_config_dir)
    let tmp_home = (^mktemp -d /tmp/yazelix_yazi_starship_config_XXXXXX | str trim)
    let temp_config_dir = ($tmp_home | path join ".config" "yazelix")
    mkdir ($tmp_home | path join ".config")

    let result = (try {
        let generated = (with-env {
            HOME: $tmp_home
            XDG_CONFIG_HOME: ($tmp_home | path join ".config")
            XDG_DATA_HOME: ($tmp_home | path join ".local" "share")
            YAZELIX_CONFIG_DIR: $temp_config_dir
            YAZELIX_RUNTIME_DIR: $repo_root
        } {
            let merged_dir = (generate_merged_yazi_config $repo_root --quiet)
            generate_merged_yazi_config $repo_root --quiet | ignore
            {
                merged_dir: $merged_dir
                init_lua: (open --raw ($merged_dir | path join "init.lua"))
                starship_config: (open --raw ($merged_dir | path join "yazelix_starship.toml"))
                starship_config_mode: (^stat -c '%A' ($merged_dir | path join "yazelix_starship.toml") | str trim)
            }
        })

        let expected_starship_config_path = ($generated.merged_dir | path join "yazelix_starship.toml")

        if (
            ($expected_starship_config_path | path exists)
            and ($generated.init_lua | str contains $"config_file = \"($expected_starship_config_path)\"")
            and ($generated.starship_config | str contains "# YAZELIX STARSHIP CONFIG FOR YAZI SIDEBAR")
            and ($generated.starship_config_mode != "-r--r--r--")
        ) {
            print "  ✅ Yazi Starship plugin now points at a managed sidebar-specific config that survives repeated regeneration"
            true
        } else {
            print $"  ❌ Missing stable managed Yazi Starship config wiring: path=($expected_starship_config_path) exists=(($expected_starship_config_path | path exists)) mode=($generated.starship_config_mode)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmp_home
    $result
}

# Defends: sidebar width propagates into generated Zellij layouts and plugin config.
# Strength: defect=2 behavior=2 resilience=1 cost=1 uniqueness=2 total=8/10
def test_generate_merged_zellij_config_carries_sidebar_width_to_layouts_and_plugin_config [] {
    print "🧪 Testing merged Zellij config carries editor.sidebar_width_percent into layouts and plugin config..."

    if (which zellij | is-empty) {
        print "  ℹ️  Skipping Zellij config merge test because zellij is not available"
        return true
    }

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_sidebar_width_test_XXXXXX | str trim)

    let result = (try {
        let config_path = ($tmpdir | path join "yazelix.toml")
        let out_dir = ($tmpdir | path join "out")
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home
        '[editor]
sidebar_width_percent = 25
' | save --force --raw $config_path

        let output = (with-env {
            HOME: $fake_home
            YAZELIX_CONFIG_OVERRIDE: $config_path
            YAZELIX_TEST_OUT_DIR: $out_dir
        } {
            let root = (get_repo_config_dir)
            generate_merged_zellij_config $root $env.YAZELIX_TEST_OUT_DIR | ignore
            {
                config: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "config.kdl"))
                layout: (open --raw ($env.YAZELIX_TEST_OUT_DIR | path join "layouts" "yzx_side.swap.kdl"))
            }
        })
        let generated_config = ($output.config | str trim)
        let generated_layout = ($output.layout | str trim)

        if (
            ($generated_config | str contains 'sidebar_width_percent "25"')
            and ($generated_layout | str contains 'size "25%"')
            and ($generated_layout | str contains 'size "75%"')
            and ($generated_layout | str contains 'size "45%"')
            and ($generated_layout | str contains 'size "30%"')
        ) {
            print "  ✅ Merged config and generated layouts carry the configured sidebar width end to end"
            true
        } else {
            print "  ❌ Sidebar width did not propagate through merged Zellij config generation"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Defends: native Zellij config can still be used without Yazelix taking ownership of it.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_uses_native_user_config_without_relocating_it [] {
    print "🧪 Testing merged Zellij config uses native Zellij config as a fallback without relocating it..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_user_cfg_relocate_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        write_legacy_native_zellij_config $fake_home

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {} {||
            {
                managed_exists: ((($fake_home | path join ".config" "yazelix" "user_configs" "zellij" "config.kdl") | path exists))
                legacy_exists: ((($fake_home | path join ".config" "zellij" "config.kdl") | path exists))
            }
        })
        let config_stdout = ($output.config | str trim)

        if (
            ($config_stdout | str contains 'scroll_buffer_size 12345')
            and (not $output.managed_exists)
            and $output.legacy_exists
        ) {
            print "  ✅ Merged Zellij config uses the native Zellij config as a fallback without moving it"
            true
        } else {
            print "  ❌ Unexpected result: native Zellij config was not preserved as a fallback correctly"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

# Regression: managed Zellij config wins cleanly when both native and managed files exist.
# Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
def test_generate_merged_zellij_config_prefers_managed_user_config_when_native_config_also_exists [] {
    print "🧪 Testing merged Zellij config prefers the managed user config and leaves native Zellij config alone..."

    let tmpdir = (^mktemp -d /tmp/yazelix_zellij_dual_config_XXXXXX | str trim)

    let result = (try {
        let fake_home = ($tmpdir | path join "home")
        write_minimal_user_zellij_config $fake_home
        write_legacy_native_zellij_config $fake_home

        let output = (run_merged_zellij_config_in_fake_home $tmpdir {} {||
            {
                managed_exists: ((($fake_home | path join ".config" "yazelix" "user_configs" "zellij" "config.kdl") | path exists))
                native_exists: ((($fake_home | path join ".config" "zellij" "config.kdl") | path exists))
            }
        })
        let config_stdout = ($output.config | str trim)

        if (
            ($config_stdout | str contains 'bind "f1" { WriteChars "fixture"; }')
            and not ($config_stdout | str contains 'scroll_buffer_size 12345')
            and $output.managed_exists
            and $output.native_exists
        ) {
            print "  ✅ Merged Zellij config keeps the managed user config canonical without deleting the native Zellij config"
            true
        } else {
            print "  ❌ Unexpected result: managed/native Zellij config ownership was not preserved correctly"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    })

    rm -rf $tmpdir
    $result
}

export def run_generated_config_canonical_tests [] {
    [
        (test_generate_all_terminal_configs_keeps_terminal_overrides_opt_in)
        (test_terminal_override_imports_ignore_yazelix_dir_runtime_root)
        (test_managed_wrapper_launch_command_does_not_forward_config_mode_flag)
        (test_generate_ghostty_config_references_existing_effect_shaders)
        (test_parse_yazelix_config_does_not_auto_apply_safe_migrations)
        (test_parse_yazelix_config_rejects_legacy_ascii_mode_with_migration_guidance)
        (test_parse_yazelix_config_bootstraps_split_default_surfaces)
        (test_parse_yazelix_config_rejects_legacy_root_config_without_confirmation)
        (test_parse_yazelix_config_relocates_legacy_root_config_when_explicitly_allowed)
        (test_parse_yazelix_config_rejects_legacy_main_file_packs_with_migration_guidance)
        (test_parse_yazelix_config_rejects_split_pack_ownership)
        (test_user_mode_requires_real_terminal_config)
        (test_config_schema_rejects_removed_auto_terminal_config_mode)
        (test_config_schema_rejects_removed_layout_widget)
        (test_generate_merged_yazi_config_relocates_legacy_user_overrides)
        (test_generate_merged_yazi_config_syncs_starship_plugin_config)
        (test_generate_merged_zellij_config_uses_native_user_config_without_relocating_it)
        (test_generate_merged_zellij_config_prefers_managed_user_config_when_native_config_also_exists)
        (test_generate_merged_zellij_config_carries_sidebar_width_to_layouts_and_plugin_config)
    ]
}

export def run_generated_config_tests [] {
    run_generated_config_canonical_tests
}
