#!/usr/bin/env nu

use config_surfaces.nu [load_config_surface_pair]

export const MANAGED_CONFIG_TRANSACTION_DIRNAME = ".managed_config_transactions"
export const MANAGED_CONFIG_TRANSACTION_SCHEMA_VERSION = 1

def normalize_optional_value [value: any] {
    if (($value | describe) == "nothing") {
        null
    } else {
        $value
    }
}

def make_transaction_id [] {
    let timestamp = (date now | format date "%Y%m%d_%H%M%S_%3f")
    let suffix = (random int 1000000..9999999)
    $"txn_($timestamp)_($suffix)"
}

export def get_managed_config_transaction_dir [config_path: string] {
    let resolved = ($config_path | path expand)
    ($resolved | path dirname | path join $MANAGED_CONFIG_TRANSACTION_DIRNAME)
}

def get_transaction_work_dir [config_path: string, transaction_id: string] {
    (get_managed_config_transaction_dir $config_path | path join $transaction_id)
}

def get_transaction_manifest_path [work_dir: string] {
    $work_dir | path join "manifest.json"
}

def save_manifest [manifest_path: string, manifest: record] {
    $manifest | to json | save --force --raw $manifest_path
}

def list_transaction_manifest_paths [config_path: string] {
    let tx_root = (get_managed_config_transaction_dir $config_path)
    if not ($tx_root | path exists) {
        return []
    }

    mut manifests = []
    for entry in (ls $tx_root) {
        if $entry.type != "dir" {
            continue
        }

        let manifest_path = ($entry.name | path join "manifest.json")
        if ($manifest_path | path exists) {
            $manifests = ($manifests | append $manifest_path)
        }
    }

    $manifests | sort
}

def make_target_record [
    role: string
    target_path: string
    staged_path: string
    backup_path: any
    existed_before: bool
] {
    {
        role: $role
        target_path: ($target_path | path expand)
        staged_path: ($staged_path | path expand)
        backup_path: (if $backup_path == null { null } else { $backup_path | path expand })
        existed_before: $existed_before
    }
}

def remove_path_if_exists [path: string] {
    if ($path | path exists) {
        rm -rf $path
    }
}

def restore_target_from_manifest [target: record] {
    let target_path = $target.target_path
    let backup_path = ($target.backup_path? | default null)

    if ($backup_path != null) and ($backup_path | path exists) {
        cp $backup_path $target_path
        return
    }

    if ($target.existed_before? | default false) {
        error make {msg: $"Interrupted managed config transaction is missing rollback backup for ($target.role): ($target_path)"}
    }

    if ($target_path | path exists) {
        rm $target_path
    }
}

def rollback_transaction_manifest [manifest_path: string] {
    let manifest = (open $manifest_path)
    let targets = ($manifest.targets | default [])

    for target in ($targets | reverse) {
        restore_target_from_manifest $target
    }

    for target in $targets {
        let staged_path = ($target.staged_path? | default null)
        if $staged_path != null {
            remove_path_if_exists $staged_path
        }
    }

    let work_dir = ($manifest_path | path dirname)
    remove_path_if_exists $work_dir

    {
        transaction_id: ($manifest.transaction_id? | default "unknown")
        recovered: true
    }
}

export def recover_stale_managed_config_transactions [config_path: string] {
    let manifests = (list_transaction_manifest_paths $config_path)

    if ($manifests | is-empty) {
        return {
            recovered_count: 0
            transaction_ids: []
        }
    }

    mut recovered = []
    for manifest_path in $manifests {
        let result = (rollback_transaction_manifest $manifest_path)
        $recovered = ($recovered | append $result.transaction_id)
    }

    {
        recovered_count: ($recovered | length)
        transaction_ids: $recovered
    }
}

def ensure_no_interrupted_transactions [config_path: string] {
    let manifests = (list_transaction_manifest_paths $config_path)
    if ($manifests | is-empty) {
        return
    }

    let count = ($manifests | length)
    error make {msg: $"Found ($count) unfinished managed config transaction\(s\) under ((get_managed_config_transaction_dir $config_path)). Recover them before applying a new config migration transaction."}
}

def validate_staged_pair [
    main_staged_path: string
    pack_staged_path?: string
    existing_pack_path?: string
] {
    let normalized_pack_staged_path = (normalize_optional_value $pack_staged_path)
    let normalized_existing_pack_path = (normalize_optional_value $existing_pack_path)
    let validation_pack_path = if $normalized_pack_staged_path != null {
        $normalized_pack_staged_path
    } else if ($normalized_existing_pack_path != null) and ($normalized_existing_pack_path | path exists) {
        $normalized_existing_pack_path
    } else {
        null
    }

    if $validation_pack_path == null {
        load_config_surface_pair $main_staged_path | ignore
    } else {
        load_config_surface_pair $main_staged_path $validation_pack_path | ignore
    }
}

export def apply_managed_config_transaction [
    caller: string
    config_path: string
    rewritten_main_toml: string
    pack_config_path?: string
    rewritten_pack_toml?: string
] {
    ensure_no_interrupted_transactions $config_path

    let transaction_id = (make_transaction_id)
    let work_dir = (get_transaction_work_dir $config_path $transaction_id)
    let manifest_path = (get_transaction_manifest_path $work_dir)
    let backup_stamp = (date now | format date "%Y%m%d_%H%M%S_%3f")
    let normalized_pack_config_path = (normalize_optional_value $pack_config_path)
    let normalized_pack_toml = (normalize_optional_value $rewritten_pack_toml)
    let resolved_config_path = ($config_path | path expand)
    let resolved_pack_config_path = if $normalized_pack_config_path == null {
        null
    } else {
        $normalized_pack_config_path | into string | path expand
    }

    let main_existed_before = ($resolved_config_path | path exists)
    let main_backup_path = if $main_existed_before {
        $"($resolved_config_path).backup-($backup_stamp)"
    } else {
        null
    }
    let main_staged_path = ($work_dir | path join "yazelix.toml")

    let has_pack_target = ($normalized_pack_toml != null)
    let pack_existed_before = if $has_pack_target and ($resolved_pack_config_path != null) {
        $resolved_pack_config_path | path exists
    } else {
        false
    }
    let pack_backup_path = if $has_pack_target and $pack_existed_before {
        $"($resolved_pack_config_path).backup-($backup_stamp)"
    } else {
        null
    }
    let pack_staged_path = if $has_pack_target {
        $work_dir | path join "yazelix_packs.toml"
    } else {
        null
    }

    let targets = (
        [
            (make_target_record "main" $resolved_config_path $main_staged_path $main_backup_path $main_existed_before)
        ]
        | append (
            if $has_pack_target {
                [
                    (make_target_record "packs" $resolved_pack_config_path $pack_staged_path $pack_backup_path $pack_existed_before)
                ]
            } else {
                []
            }
        )
    )

    let prepared_manifest = {
        schema_version: $MANAGED_CONFIG_TRANSACTION_SCHEMA_VERSION
        transaction_id: $transaction_id
        caller: $caller
        phase: "prepared"
        targets: $targets
    }

    mkdir $work_dir
    save_manifest $manifest_path $prepared_manifest

    try {
        if $main_backup_path != null {
            try {
                cp $resolved_config_path $main_backup_path
            } catch {|err|
                error make {msg: $"Failed to create main config backup: ($err | to nuon)"}
            }
        }

        if $pack_backup_path != null {
            try {
                cp $resolved_pack_config_path $pack_backup_path
            } catch {|err|
                error make {msg: $"Failed to create pack config backup: ($err | to nuon)"}
            }
        }

        try {
            $rewritten_main_toml | save --force --raw $main_staged_path
        } catch {|err|
            error make {msg: $"Failed to write staged main config: ($err | to nuon)"}
        }

        if $has_pack_target {
            try {
                $normalized_pack_toml | save --force --raw $pack_staged_path
            } catch {|err|
                error make {msg: $"Failed to write staged pack config: ($err | to nuon)"}
            }
        }

        try {
            validate_staged_pair $main_staged_path $pack_staged_path $resolved_pack_config_path
        } catch {|err|
            error make {msg: $"Failed to validate the staged managed config pair: ($err | to nuon)"}
        }

        try {
            save_manifest $manifest_path ($prepared_manifest | upsert phase "validated")
        } catch {|err|
            error make {msg: $"Failed to persist the validated transaction manifest: ($err | to nuon)"}
        }

        for target in $targets {
            try {
                mv -f $target.staged_path $target.target_path
            } catch {|err|
                error make {msg: $"Failed to commit the staged ($target.role) target: ($err | to nuon)"}
            }
        }

        remove_path_if_exists $manifest_path
        remove_path_if_exists $work_dir

        {
            status: "applied"
            transaction_id: $transaction_id
            config_path: $resolved_config_path
            backup_path: $main_backup_path
            pack_config_path: $resolved_pack_config_path
            pack_backup_path: $pack_backup_path
        }
    } catch {|err|
        try {
            rollback_transaction_manifest $manifest_path | ignore
        }
        let details = (try { $err | to json -r } catch { $err | to nuon })
        error make {msg: $"Failed to apply managed config transaction: ($details)"}
    }
}
