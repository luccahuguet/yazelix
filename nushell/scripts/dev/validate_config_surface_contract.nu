#!/usr/bin/env nu

use ../utils/config_contract.nu [
    load_main_config_contract
    load_pack_catalog_contract
]
use ../utils/config_state.nu [compute_config_state]

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const MAIN_TEMPLATE_PATH = ($REPO_ROOT | path join "yazelix_default.toml")
const PACK_TEMPLATE_PATH = ($REPO_ROOT | path join "yazelix_packs_default.toml")
const MODULE_PATH = ($REPO_ROOT | path join "home_manager" "module.nix")
const TRACKED_LAUNCH_INPUT_FILES = ["devenv.lock" "devenv.nix" "devenv.yaml"]

def load_repo_main_contract [] {
    with-env {YAZELIX_RUNTIME_DIR: $REPO_ROOT} {
        load_main_config_contract
    }
}

def load_repo_pack_contract [] {
    with-env {YAZELIX_RUNTIME_DIR: $REPO_ROOT} {
        load_pack_catalog_contract
    }
}

def get_nested_value_info [record: record, field_path: string] {
    let parts = ($field_path | split row ".")
    mut current = $record

    for part in $parts {
        if not (($current | describe) | str starts-with "record") {
            return {exists: false, value: null}
        }

        if not ($part in ($current | columns)) {
            return {exists: false, value: null}
        }

        $current = ($current | get $part)
    }

    {exists: true, value: $current}
}

def format_value [value: any] {
    $value | to json -r
}

def escape_nix_string [value: string] {
    $value
    | str replace -a "\\" "\\\\"
    | str replace -a "\"" "\\\""
}

def build_home_manager_defaults_expr [option_names: list<string>] {
    let module_path = (escape_nix_string $MODULE_PATH)
    let bindings = (
        $option_names
        | uniq
        | sort
        | each { |name| $"  ($name) = module.options.programs.yazelix.($name).default;" }
        | str join "\n"
    )

    [
        "let"
        "  pkgs = import <nixpkgs> {};"
        "  lib = pkgs.lib;"
        ("  module = import (builtins.toPath \"" + $module_path + "\") { inherit lib pkgs; config = { programs.yazelix = {}; xdg.configHome = \"/tmp\"; }; };")
        "in {"
        $bindings
        "}"
    ] | str join "\n"
}

def load_home_manager_defaults [option_names: list<string>] {
    let expr = (build_home_manager_defaults_expr $option_names)
    let result = (^nix eval --impure --json --expr $expr | complete)

    if $result.exit_code != 0 {
        error make {
            msg: (
                [
                    "Failed to evaluate Home Manager defaults for the config-surface contract validator."
                    ($result.stderr | str trim)
                ] | str join "\n"
            )
        }
    }

    $result.stdout | from json
}

def validate_main_contract_parity [] {
    let contract = (load_repo_main_contract)
    let template = (open $MAIN_TEMPLATE_PATH)
    let declared_fields = ($contract.fields | columns | sort)
    let hm_option_names = (
        $declared_fields
        | each { |field_path| ($contract.fields | get $field_path).home_manager_option }
    )
    let hm_defaults = (load_home_manager_defaults $hm_option_names)

    mut errors = []

    if ($contract.contract.field_count | into int) != ($declared_fields | length) {
        $errors = ($errors | append $"main_config_contract.toml field_count mismatch: declared=($contract.contract.field_count | into int), actual=($declared_fields | length)")
    }

    for field_path in $declared_fields {
        let field = ($contract.fields | get $field_path)
        let hm_option = ($field.home_manager_option | into string)

        if not ($hm_option in ($hm_defaults | columns)) {
            $errors = ($errors | append $"Home Manager option `($hm_option)` is missing for main-contract field `($field_path)`")
            continue
        }

        let expected_hm_default = if ($field.home_manager_default_is_null? | default false) {
            null
        } else {
            $field.default
        }
        let actual_hm_default = ($hm_defaults | get $hm_option)
        if $actual_hm_default != $expected_hm_default {
            $errors = ($errors | append $"Home Manager default mismatch for `($field_path)` via `($hm_option)`: expected (format_value $expected_hm_default), got (format_value $actual_hm_default)")
        }

        let emit_in_template = ($field.emit_in_default_template? | default true)
        let template_value = (get_nested_value_info $template $field_path)

        if not $emit_in_template {
            if $template_value.exists {
                $errors = ($errors | append $"Default template should omit `($field_path)`, but it is present with value (format_value $template_value.value)")
            }
            continue
        }

        if not $template_value.exists {
            $errors = ($errors | append $"Default template is missing required field `($field_path)`")
            continue
        }

        if $template_value.value != $field.default {
            $errors = ($errors | append $"Default template mismatch for `($field_path)`: expected (format_value $field.default), got (format_value $template_value.value)")
        }
    }

    $errors
}

def validate_pack_contract_parity [] {
    let contract = (load_repo_pack_contract)
    let template = (open $PACK_TEMPLATE_PATH)
    let hm_defaults = (load_home_manager_defaults ["pack_names", "pack_declarations", "user_packages"])
    let declaration_names = ($contract.declarations | columns | sort)
    let template_declarations = ($template.declarations? | default {})
    let hm_pack_declarations = ($hm_defaults.pack_declarations? | default {})
    mut errors = []

    if ($contract.contract.declaration_count | into int) != ($declaration_names | length) {
        $errors = ($errors | append $"pack_catalog_contract.toml declaration_count mismatch: declared=($contract.contract.declaration_count | into int), actual=($declaration_names | length)")
    }

    if (($template_declarations | columns | sort) != $declaration_names) {
        $errors = ($errors | append "yazelix_packs_default.toml declarations do not match the canonical pack catalog names")
    }

    if (($hm_pack_declarations | columns | sort) != $declaration_names) {
        $errors = ($errors | append "Home Manager pack_declarations do not match the canonical pack catalog names")
    }

    let expected_enabled_default = ($contract.surface.enabled.default? | default [])
    if (($template.enabled? | default []) != $expected_enabled_default) {
        $errors = ($errors | append $"yazelix_packs_default.toml enabled default mismatch: expected (format_value $expected_enabled_default), got (format_value ($template.enabled? | default []))")
    }
    if (($hm_defaults.pack_names? | default []) != $expected_enabled_default) {
        $errors = ($errors | append $"Home Manager pack_names default mismatch: expected (format_value $expected_enabled_default), got (format_value ($hm_defaults.pack_names? | default []))")
    }

    let expected_user_packages_default = ($contract.surface.user_packages.default? | default [])
    if (($template.user_packages? | default []) != $expected_user_packages_default) {
        $errors = ($errors | append $"yazelix_packs_default.toml user_packages default mismatch: expected (format_value $expected_user_packages_default), got (format_value ($template.user_packages? | default []))")
    }
    if (($hm_defaults.user_packages? | default []) != $expected_user_packages_default) {
        $errors = ($errors | append $"Home Manager user_packages default mismatch: expected (format_value $expected_user_packages_default), got (format_value ($hm_defaults.user_packages? | default []))")
    }

    for pack_name in $declaration_names {
        let expected_packages = (($contract.declarations | get $pack_name).packages? | default [])
        let template_packages = ($template_declarations | get -o $pack_name | default [])
        let hm_packages = ($hm_pack_declarations | get -o $pack_name | default [])

        if $template_packages != $expected_packages {
            $errors = ($errors | append $"Pack template declaration mismatch for `($pack_name)`: expected (format_value $expected_packages), got (format_value $template_packages)")
        }

        if $hm_packages != $expected_packages {
            $errors = ($errors | append $"Home Manager pack declaration mismatch for `($pack_name)`: expected (format_value $expected_packages), got (format_value $hm_packages)")
        }
    }

    $errors
}

def copy_contract_fixture_file [source_root: string, target_root: string, relative_path: string] {
    let source = ($source_root | path join $relative_path)
    let target = ($target_root | path join $relative_path)
    mkdir ($target | path dirname)
    cp $source $target
}

def setup_launch_profile_fixture [] {
    let fixture_root = (^mktemp -d /tmp/yazelix_config_contract_XXXXXX | str trim)
    let runtime_root = ($fixture_root | path join "runtime")
    let config_root = ($fixture_root | path join "config")
    let user_config_dir = ($config_root | path join "user_configs")
    let home_root = ($fixture_root | path join "home")

    mkdir $runtime_root
    mkdir $user_config_dir
    mkdir $home_root

    for relative_path in [
        "yazelix_default.toml"
        "yazelix_packs_default.toml"
        "devenv.lock"
        "devenv.nix"
        "devenv.yaml"
        "config_metadata/main_config_contract.toml"
    ] {
        copy_contract_fixture_file $REPO_ROOT $runtime_root $relative_path
    }

    cp $MAIN_TEMPLATE_PATH ($user_config_dir | path join "yazelix.toml")
    cp $PACK_TEMPLATE_PATH ($user_config_dir | path join "yazelix_packs.toml")

    {
        fixture_root: $fixture_root
        runtime_root: $runtime_root
        config_root: $config_root
        home_root: $home_root
        main_config_path: ($user_config_dir | path join "yazelix.toml")
    }
}

def compute_fixture_state [fixture: record] {
    with-env {
        YAZELIX_RUNTIME_DIR: $fixture.runtime_root
        YAZELIX_CONFIG_DIR: $fixture.config_root
        HOME: $fixture.home_root
    } {
        compute_config_state
    }
}

def validate_launch_profile_contract [] {
    let fixture = (setup_launch_profile_fixture)
    mut errors = []
    let caught_error = (try {
        let baseline = (compute_fixture_state $fixture)

        let runtime_only_config = (open $fixture.main_config_path | upsert core.skip_welcome_screen true)
        $runtime_only_config | to toml | save --force $fixture.main_config_path
        let after_runtime_only = (compute_fixture_state $fixture)

        if $baseline.config_hash != $after_runtime_only.config_hash {
            $errors = ($errors | append "Non-rebuild runtime config change unexpectedly altered config_hash")
        }

        if $baseline.combined_hash != $after_runtime_only.combined_hash {
            $errors = ($errors | append "Non-rebuild runtime config change unexpectedly altered combined_hash")
        }

        let rebuild_config = (open $fixture.main_config_path | upsert core.max_jobs "max")
        $rebuild_config | to toml | save --force $fixture.main_config_path
        let after_rebuild_config = (compute_fixture_state $fixture)

        if $after_runtime_only.config_hash == $after_rebuild_config.config_hash {
            $errors = ($errors | append "Rebuild-relevant config change did not alter config_hash")
        }

        if $after_runtime_only.combined_hash == $after_rebuild_config.combined_hash {
            $errors = ($errors | append "Rebuild-relevant config change did not alter combined_hash")
        }

        let lock_path = ($fixture.runtime_root | path join "devenv.lock")
        ((open --raw $lock_path) + "\nvalidator-touch\n") | save --force --raw $lock_path
        let after_input_change = (compute_fixture_state $fixture)

        if $after_rebuild_config.config_hash != $after_input_change.config_hash {
            $errors = ($errors | append "Tracked devenv input change unexpectedly altered config_hash instead of only combined_hash")
        }

        if $after_rebuild_config.combined_hash == $after_input_change.combined_hash {
            $errors = ($errors | append "Tracked devenv input change did not alter combined_hash")
        }

        for required_file in $TRACKED_LAUNCH_INPUT_FILES {
            let path = ($fixture.runtime_root | path join $required_file)
            if not ($path | path exists) {
                $errors = ($errors | append $"Launch-profile fixture is missing tracked input file `($required_file)`")
            }
        }
        null
    } catch { |err|
        $err.msg
    })

    if $caught_error != null {
        $errors = ($errors | append $"Launch-profile contract validation failed unexpectedly: ($caught_error)")
    }

    rm -rf $fixture.fixture_root
    $errors
}

export def main [] {
    let errors = [
        (validate_main_contract_parity)
        (validate_pack_contract_parity)
        (validate_launch_profile_contract)
    ] | flatten

    if ($errors | is-empty) {
        print "✅ Config surface and launch-profile contract is valid"
        return
    }

    print "❌ Config surface and launch-profile contract validation failed"
    for error_message in $errors {
        print $"  - ($error_message)"
    }

    error make {msg: "config surface and launch-profile contract validation failed"}
}
