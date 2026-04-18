#!/usr/bin/env nu

use ../utils/config_contract.nu load_main_config_contract
use ../utils/config_state.nu [compute_config_state record_materialized_state]

const REPO_ROOT = (path self | path dirname | path dirname | path dirname | path dirname)
const MAIN_TEMPLATE_PATH = ($REPO_ROOT | path join "yazelix_default.toml")
const MODULE_PATH = ($REPO_ROOT | path join "home_manager" "module.nix")

def load_repo_main_contract [] {
    with-env { YAZELIX_RUNTIME_DIR: $REPO_ROOT } {
        load_main_config_contract
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

        $current = ($current | get -o $part)
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

def build_home_manager_desktop_entry_expr [] {
    let module_path = (escape_nix_string $MODULE_PATH)
    [
        "let"
        "  pkgs = import <nixpkgs> {};"
        "  lib = pkgs.lib;"
        "  eval = lib.evalModules {"
        "    specialArgs = { inherit pkgs; nixgl = null; };"
        "    modules = ["
        ("      (builtins.toPath \"" + $module_path + "\")")
        "      ({ lib, ... }: {"
        "        options.xdg.configHome = lib.mkOption { type = lib.types.str; default = \"/tmp/config\"; };"
        "        options.xdg.dataHome = lib.mkOption { type = lib.types.str; default = \"/tmp/data\"; };"
        "        options.xdg.dataFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };"
        "        options.xdg.configFile = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };"
        "        options.xdg.desktopEntries = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };"
        "        options.home.packages = lib.mkOption { type = lib.types.listOf lib.types.package; default = []; };"
        "        options.home.activation = lib.mkOption { type = lib.types.attrsOf lib.types.anything; default = {}; };"
        "        options.home.profileDirectory = lib.mkOption { type = lib.types.str; default = \"/tmp/profile\"; };"
        "        config.programs.yazelix.enable = true;"
        "      })"
        "    ];"
        "  };"
        "in {"
        "  exec = eval.config.xdg.desktopEntries.yazelix.exec or \"\";"
        "  terminal = eval.config.xdg.desktopEntries.yazelix.terminal or false;"
        "}"
    ] | str join "\n"
}

def load_home_manager_desktop_entry_contract [] {
    let expr = (build_home_manager_desktop_entry_expr)
    let result = (^nix eval --impure --json --expr $expr | complete)

    if $result.exit_code != 0 {
        error make {
            msg: (
                [
                    "Failed to evaluate the Home Manager desktop-entry contract."
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
        | each { |field_path| (($contract.fields | get -o $field_path | default {}) | get -o home_manager_option | default "") }
    )
    let hm_defaults = (load_home_manager_defaults $hm_option_names)

    mut errors = []

    if ($contract.contract.field_count | into int) != ($declared_fields | length) {
        $errors = ($errors | append $"main_config_contract.toml field_count mismatch: declared=($contract.contract.field_count | into int), actual=($declared_fields | length)")
    }

    for field_path in $declared_fields {
        let field = ($contract.fields | get -o $field_path | default {})
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
        let actual_hm_default = ($hm_defaults | get -o $hm_option)
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

def validate_home_manager_desktop_entry_contract [] {
    let entry = (load_home_manager_desktop_entry_contract)
    let actual_exec = ($entry.exec? | default "")
    mut errors = []

    if not ($entry.terminal? | default false) {
        $errors = ($errors | append "Home Manager desktop entry must set terminal = true so desktop-launch pre-terminal failures are visible")
    }

    if $actual_exec != "/tmp/profile/bin/yzx desktop launch" {
        $errors = ($errors | append $"Home Manager desktop entry Exec mismatch: expected /tmp/profile/bin/yzx desktop launch, got ($actual_exec | to json -r)")
    }

    $errors
}

def copy_fixture_file [source_root: string, target_root: string, relative_path: string] {
    let source = ($source_root | path join $relative_path)
    let target = ($target_root | path join $relative_path)
    mkdir ($target | path dirname)
    cp $source $target
}

def setup_config_state_fixture [] {
    let fixture_root = (^mktemp -d /tmp/yazelix_config_contract_XXXXXX | str trim)
    let runtime_root = ($fixture_root | path join "runtime")
    let runtime_root_alt = ($fixture_root | path join "runtime_alt")
    let config_root = ($fixture_root | path join "config")
    let user_config_dir = ($config_root | path join "user_configs")
    let home_root = ($fixture_root | path join "home")

    mkdir $runtime_root
    mkdir $runtime_root_alt
    mkdir $user_config_dir
    mkdir $home_root

    for relative_path in [
        ".taplo.toml"
        "yazelix_default.toml"
        "config_metadata/main_config_contract.toml"
    ] {
        copy_fixture_file $REPO_ROOT $runtime_root $relative_path
        copy_fixture_file $REPO_ROOT $runtime_root_alt $relative_path
    }

    cp $MAIN_TEMPLATE_PATH ($user_config_dir | path join "yazelix.toml")

    {
        fixture_root: $fixture_root
        runtime_root: $runtime_root
        runtime_root_alt: $runtime_root_alt
        config_root: $config_root
        home_root: $home_root
        main_config_path: ($user_config_dir | path join "yazelix.toml")
    }
}

def compute_fixture_state [fixture: record, runtime_root: string] {
    with-env {
        YAZELIX_RUNTIME_DIR: $runtime_root
        YAZELIX_CONFIG_DIR: $fixture.config_root
        HOME: $fixture.home_root
    } {
        compute_config_state
    }
}

def record_fixture_state [fixture: record, state: record, runtime_root: string] {
    with-env {
        YAZELIX_RUNTIME_DIR: $runtime_root
        YAZELIX_CONFIG_DIR: $fixture.config_root
        HOME: $fixture.home_root
    } {
        record_materialized_state $state
    }
}

def validate_generated_state_contract [] {
    let fixture = (setup_config_state_fixture)
    mut errors = []
    let caught_error = (try {
        let baseline = (compute_fixture_state $fixture $fixture.runtime_root)
        record_fixture_state $fixture $baseline $fixture.runtime_root

        let runtime_only_config = (open $fixture.main_config_path | upsert core.skip_welcome_screen true)
        $runtime_only_config | to toml | save --force $fixture.main_config_path
        let after_runtime_only = (compute_fixture_state $fixture $fixture.runtime_root)

        if $baseline.config_hash != $after_runtime_only.config_hash {
            $errors = ($errors | append "Non-rebuild runtime config change unexpectedly altered config_hash")
        }

        if $baseline.combined_hash != $after_runtime_only.combined_hash {
            $errors = ($errors | append "Non-rebuild runtime config change unexpectedly altered combined_hash")
        }

        if $after_runtime_only.needs_refresh {
            $errors = ($errors | append "Non-rebuild runtime config change unexpectedly marked generated state as stale")
        }

        let rebuild_config = (open $fixture.main_config_path | upsert editor.command "nvim")
        $rebuild_config | to toml | save --force $fixture.main_config_path
        let after_rebuild_config = (compute_fixture_state $fixture $fixture.runtime_root)

        if $after_runtime_only.config_hash == $after_rebuild_config.config_hash {
            $errors = ($errors | append "Rebuild-relevant config change did not alter config_hash")
        }

        if $after_runtime_only.combined_hash == $after_rebuild_config.combined_hash {
            $errors = ($errors | append "Rebuild-relevant config change did not alter combined_hash")
        }

        if not $after_rebuild_config.needs_refresh {
            $errors = ($errors | append "Rebuild-relevant config change did not mark generated state as stale")
        }

        record_fixture_state $fixture $after_rebuild_config $fixture.runtime_root
        let after_runtime_root_change = (compute_fixture_state $fixture $fixture.runtime_root_alt)

        if $after_rebuild_config.config_hash != $after_runtime_root_change.config_hash {
            $errors = ($errors | append "Changing only the runtime root unexpectedly altered config_hash")
        }

        if $after_rebuild_config.runtime_hash == $after_runtime_root_change.runtime_hash {
            $errors = ($errors | append "Changing the runtime root did not alter runtime_hash")
        }

        if $after_rebuild_config.combined_hash == $after_runtime_root_change.combined_hash {
            $errors = ($errors | append "Changing the runtime root did not alter combined_hash")
        }

        if not $after_runtime_root_change.needs_refresh {
            $errors = ($errors | append "Changing the runtime root did not mark generated state as stale")
        }

        null
    } catch { |err|
        $err.msg
    })

    if $caught_error != null {
        $errors = ($errors | append $"Generated-state contract validation failed unexpectedly: ($caught_error)")
    }

    rm -rf $fixture.fixture_root
    $errors
}

export def main [] {
    let errors = [
        (validate_main_contract_parity)
        (validate_home_manager_desktop_entry_contract)
        (validate_generated_state_contract)
    ] | flatten

    if ($errors | is-empty) {
        print "✅ Main config surface, Home Manager desktop entry, and generated-state contract is valid"
        return
    }

    print "❌ Main config surface, Home Manager desktop entry, and generated-state contract validation failed"
    for error_message in $errors {
        print $"  - ($error_message)"
    }

    error make {msg: "main config surface, Home Manager desktop entry, and generated-state contract validation failed"}
}
