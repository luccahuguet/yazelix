#!/usr/bin/env nu
# Canonical config-contract loader helpers.

use ./common.nu [get_yazelix_runtime_dir]

export const MAIN_CONFIG_CONTRACT_RELATIVE_PATH = "config_metadata/main_config_contract.toml"
export const PACK_CATALOG_CONTRACT_RELATIVE_PATH = "config_metadata/pack_catalog_contract.toml"

def get_main_config_contract_path [] {
    (get_yazelix_runtime_dir | path join $MAIN_CONFIG_CONTRACT_RELATIVE_PATH)
}

export def load_main_config_contract [] {
    open (get_main_config_contract_path)
}

def get_pack_catalog_contract_path [] {
    (get_yazelix_runtime_dir | path join $PACK_CATALOG_CONTRACT_RELATIVE_PATH)
}

export def load_pack_catalog_contract [] {
    open (get_pack_catalog_contract_path)
}

export def get_main_config_rebuild_required_paths [] {
    let contract = (load_main_config_contract)
    mut rebuild_paths = []

    for field_path in ($contract.fields | columns) {
        let field = (
            $contract.fields
            | transpose key value
            | where key == $field_path
            | get -o value.0
            | default {}
        )
        let rebuild_required = ($field.rebuild_required? | default false)
        if $rebuild_required {
            $rebuild_paths = ($rebuild_paths | append $field_path)
        }
    }

    let extra_paths = ($contract.rebuild?.extra_paths? | default [])
    [$rebuild_paths, $extra_paths] | flatten
}
