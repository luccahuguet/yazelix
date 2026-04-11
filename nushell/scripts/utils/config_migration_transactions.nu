#!/usr/bin/env nu

use atomic_writes.nu write_text_atomic
use config_surfaces.nu [load_config_surface_from_main]

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
    write_text_atomic $manifest_path ($manifest | to json) --raw | ignore
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

def make_cleanup_source_record [source_path: string, archived_path: string] {
    {
        source_path: ($source_path | path expand)
        archived_path: ($archived_path | path expand)
    }
}

def remove_path_if_exists [path: string] {
    if ($path | path exists) {
        rm -rf $path
    }
}

def cleanup_transaction_work_dir [work_dir: string] {
    remove_path_if_exists $work_dir

    let tx_root = ($work_dir | path dirname)
    if ($tx_root | path exists) and ((ls $tx_root | is-empty)) {
        remove_path_if_exists $tx_root
    }
}

def restore_cleanup_source_from_manifest [cleanup_source: record] {
    let source_path = $cleanup_source.source_path
    let archived_path = $cleanup_source.archived_path

    if ($archived_path | path exists) and not ($source_path | path exists) {
        mv -f $archived_path $source_path
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
    let cleanup_sources = ($manifest.cleanup_sources? | default [])

    for target in ($targets | reverse) {
        restore_target_from_manifest $target
    }

    for cleanup_source in ($cleanup_sources | reverse) {
        restore_cleanup_source_from_manifest $cleanup_source
    }

    for target in $targets {
        let staged_path = ($target.staged_path? | default null)
        if $staged_path != null {
            remove_path_if_exists $staged_path
        }
    }

    let work_dir = ($manifest_path | path dirname)
    cleanup_transaction_work_dir $work_dir

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

def validate_staged_targets [
    main_staged_path?: string
] {
    let normalized_main_staged_path = (normalize_optional_value $main_staged_path)
    if $normalized_main_staged_path != null {
        load_config_surface_from_main $normalized_main_staged_path | ignore
        return
    }

    error make {msg: "Managed config transaction has no staged targets to validate."}
}

def apply_managed_config_transaction_with_cleanup [
    caller: string
    transaction_root_config_path: string
    main_target_path?: string
    rewritten_main_toml?: string
    cleanup_source_paths: list<string> = []
] {
    ensure_no_interrupted_transactions $transaction_root_config_path

    let transaction_id = (make_transaction_id)
    let work_dir = (get_transaction_work_dir $transaction_root_config_path $transaction_id)
    let manifest_path = (get_transaction_manifest_path $work_dir)
    let backup_stamp = (date now | format date "%Y%m%d_%H%M%S_%3f")
    let normalized_main_target_path = (normalize_optional_value $main_target_path)
    let normalized_main_toml = (normalize_optional_value $rewritten_main_toml)
    let resolved_main_target_path = if $normalized_main_target_path == null {
        null
    } else {
        $normalized_main_target_path | into string | path expand
    }

    let has_main_target = ($normalized_main_toml != null) and ($resolved_main_target_path != null)
    if not $has_main_target {
        error make {msg: "Managed config transaction needs at least one staged target."}
    }

    let main_existed_before = if $has_main_target {
        $resolved_main_target_path | path exists
    } else {
        false
    }
    let main_backup_path = if $has_main_target and $main_existed_before {
        $"($resolved_main_target_path).backup-($backup_stamp)"
    } else {
        null
    }
    let main_staged_path = if $has_main_target {
        $work_dir | path join "yazelix.toml"
    } else {
        null
    }
    let targets = [
        (make_target_record "main" $resolved_main_target_path $main_staged_path $main_backup_path $main_existed_before)
    ]

    let cleanup_sources = (
        $cleanup_source_paths
        | each {|source_path|
            let normalized_source_path = ($source_path | path expand)
            let archived_name = ($normalized_source_path | path basename)
            make_cleanup_source_record $normalized_source_path ($work_dir | path join "cleanup_sources" $archived_name)
        }
        | where {|cleanup_source| $cleanup_source.source_path | path exists }
    )

    let prepared_manifest = {
        schema_version: $MANAGED_CONFIG_TRANSACTION_SCHEMA_VERSION
        transaction_id: $transaction_id
        caller: $caller
        phase: "prepared"
        targets: $targets
        cleanup_sources: $cleanup_sources
    }

    mkdir $work_dir
    save_manifest $manifest_path $prepared_manifest

    try {
        if $main_backup_path != null {
            try {
                cp $resolved_main_target_path $main_backup_path
            } catch {|err|
                error make {msg: $"Failed to create main config backup: ($err | to nuon)"}
            }
        }

        if $has_main_target {
            try {
                write_text_atomic $main_staged_path $normalized_main_toml --raw | ignore
            } catch {|err|
                error make {msg: $"Failed to write staged main config: ($err | to nuon)"}
            }
        }

        try {
            validate_staged_targets $main_staged_path
        } catch {|err|
            error make {msg: $"Failed to validate the staged managed config: ($err | to nuon)"}
        }

        try {
            save_manifest $manifest_path ($prepared_manifest | upsert phase "validated")
        } catch {|err|
            error make {msg: $"Failed to persist the validated transaction manifest: ($err | to nuon)"}
        }

        if not ($cleanup_sources | is-empty) {
            mkdir ($work_dir | path join "cleanup_sources")
            for cleanup_source in $cleanup_sources {
                try {
                    mv -f $cleanup_source.source_path $cleanup_source.archived_path
                } catch {|err|
                    error make {msg: $"Failed to stage managed config cleanup source: ($err | to nuon)"}
                }
            }
        }

        for target in $targets {
            try {
                mv -f $target.staged_path $target.target_path
            } catch {|err|
                error make {msg: $"Failed to commit the staged ($target.role) target: ($err | to nuon)"}
            }
        }

        try {
            remove_path_if_exists $manifest_path
        } catch {|err|
            error make {msg: $"Failed to clear the committed transaction manifest: ($err | to nuon)"}
        }

        try {
            cleanup_transaction_work_dir $work_dir
        } catch { }

        {
            status: "applied"
            transaction_id: $transaction_id
            config_path: ($resolved_main_target_path | default $transaction_root_config_path)
            backup_path: $main_backup_path
        }
    } catch {|err|
        try {
            rollback_transaction_manifest $manifest_path | ignore
        } catch { }
        let details = (try { $err | to json -r } catch { $err | to nuon })
        error make {msg: $"Failed to apply managed config transaction: ($details)"}
    }
}

export def apply_managed_config_transaction [
    caller: string
    config_path: string
    rewritten_main_toml: string
] {
    apply_managed_config_transaction_with_cleanup $caller $config_path $config_path $rewritten_main_toml []
}

export def apply_managed_config_relocation_transaction [caller: string, paths: record] {
    let has_legacy_main = ($paths.legacy_user_config | path exists)

    if not $has_legacy_main {
        return {
            status: "noop"
            config_path: $paths.user_config
            backup_path: null
        }
    }

    let rewritten_main_toml = if ($paths.rewritten_main_toml? | default null) != null {
        $paths.rewritten_main_toml
    } else if $has_legacy_main {
        open --raw $paths.legacy_user_config
    } else {
        null
    }
    let cleanup_source_paths = ([$paths.legacy_user_config] | where {|path| $path | path exists })

    apply_managed_config_transaction_with_cleanup $caller $paths.user_config $paths.user_config $rewritten_main_toml $cleanup_source_paths
}
