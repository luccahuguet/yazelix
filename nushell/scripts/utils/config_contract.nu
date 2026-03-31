#!/usr/bin/env nu
# Canonical config-contract loader helpers.

use ./common.nu [get_yazelix_runtime_dir]

export const MAIN_CONFIG_CONTRACT_RELATIVE_PATH = "config_metadata/main_config_contract.toml"

export def get_main_config_contract_path [] {
    (get_yazelix_runtime_dir | path join $MAIN_CONFIG_CONTRACT_RELATIVE_PATH)
}

export def load_main_config_contract [] {
    open (get_main_config_contract_path)
}

export def get_main_config_field_contract [field_path: string] {
    let contract = (load_main_config_contract)
    $contract.fields | get $field_path
}
