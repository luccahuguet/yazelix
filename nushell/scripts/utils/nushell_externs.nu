#!/usr/bin/env nu

use atomic_writes.nu write_text_atomic
use common.nu [get_yazelix_runtime_dir get_yazelix_state_dir]
use ./yzx_core_bridge.nu resolve_yzx_core_helper_path

const YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION = 2
const YZX_EXTERN_BRIDGE_RENDERER_VERSION = "v2-rust-metadata"

export def get_generated_yzx_extern_path [state_root?: string] {
    let state_dir = if $state_root == null {
        get_yazelix_state_dir
    } else {
        $state_root | path expand
    }
    ($state_dir | path join "initializers" "nushell" "yazelix_extern.nu")
}

export def get_generated_yzx_extern_fingerprint_path [state_root?: string] {
    let extern_path = (get_generated_yzx_extern_path $state_root)
    ($extern_path | path dirname | path join "yazelix_extern.fingerprint.json")
}

def fingerprint_file [target_path: string] {
    let expanded_path = ($target_path | path expand)
    if not ($expanded_path | path exists) {
        return {
            path: $expanded_path
            exists: false
        }
    }

    let entry = (ls -D $expanded_path | get -o 0)
    if $entry == null {
        return {
            path: $expanded_path
            exists: false
        }
    }

    {
        path: $expanded_path
        exists: true
        size: ($entry.size | into string)
        modified: ($entry.modified | into string)
    }
}

def fingerprint_yzx_core_helper [runtime_dir: string] {
    let helper_path = (try {
        resolve_yzx_core_helper_path $runtime_dir
    } catch {
        $runtime_dir | path join "libexec" "yzx_core"
    })
    fingerprint_file $helper_path
}

def compute_yzx_extern_source_fingerprint [runtime_dir: string] {
    let expanded_runtime = ($runtime_dir | path expand)
    let bridge_renderer_path = ($expanded_runtime | path join "nushell" "scripts" "utils" "nushell_externs.nu")
    {
        schema_version: $YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION
        renderer_version: $YZX_EXTERN_BRIDGE_RENDERER_VERSION
        runtime_dir: $expanded_runtime
        bridge_renderer: (fingerprint_file $bridge_renderer_path)
        yzx_core: (fingerprint_yzx_core_helper $expanded_runtime)
    } | to json -r | hash sha256
}

def hash_file_contents [target_path: string] {
    if not ($target_path | path exists) {
        return null
    }

    open --raw $target_path | hash sha256
}

def read_yzx_extern_bridge_state [fingerprint_path: string] {
    if not ($fingerprint_path | path exists) {
        return null
    }

    let state = (try {
        open --raw $fingerprint_path | from json
    } catch {
        null
    })

    if ($state == null) or (not (($state | describe) | str starts-with "record")) {
        return null
    }

    $state
}

def yzx_extern_bridge_is_current [
    extern_path: string
    fingerprint_path: string
    source_fingerprint: string
] {
    let state = (read_yzx_extern_bridge_state $fingerprint_path)
    if $state == null {
        return false
    }

    let extern_hash = (hash_file_contents $extern_path)
    if $extern_hash == null {
        return false
    }

    (
        (($state.schema_version? | default 0) == $YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION)
        and (($state.source_fingerprint? | default "") == $source_fingerprint)
        and (($state.extern_hash? | default "") == $extern_hash)
    )
}

def write_yzx_extern_bridge_state [
    fingerprint_path: string
    source_fingerprint: string
    extern_content: string
] {
    let state = {
        schema_version: $YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION
        source_fingerprint: $source_fingerprint
        extern_hash: ($extern_content | hash sha256)
    }
    write_text_atomic $fingerprint_path (($state | to json -r) + "\n") --raw | ignore
}

def fetch_yzx_extern_content [runtime_root: string] {
    let runtime_dir = ($runtime_root | path expand)
    let helper_path = (resolve_yzx_core_helper_path $runtime_dir)
    let result = (^$helper_path yzx-command-metadata.externs | complete)

    if $result.exit_code != 0 {
        let stderr = ($result.stderr | default "" | str trim)
        error make {msg: $"Failed to render yzx extern bridge from Rust command metadata: ($stderr)"}
    }

    let envelope = (try {
        $result.stdout | from json
    } catch {|err|
        error make {msg: $"Rust yzx command metadata returned invalid JSON: ($err.msg)"}
    })

    if (($envelope.status? | default "") != "ok") {
        error make {msg: $"Rust yzx command metadata returned a non-ok envelope: (($result.stdout | str trim))"}
    }

    let data = ($envelope.data? | default {})
    let extern_content = ($data.extern_content? | default "")
    if ($extern_content | is-empty) {
        error make {msg: "Rust yzx command metadata returned an empty extern bridge"}
    }

    $extern_content
}

export def sync_generated_yzx_extern_bridge [runtime_root?: string, state_root?: string] {
    let extern_path = (get_generated_yzx_extern_path $state_root)
    let fingerprint_path = (get_generated_yzx_extern_fingerprint_path $state_root)
    let runtime_dir = if $runtime_root == null {
        get_yazelix_runtime_dir
    } else {
        $runtime_root | path expand
    }

    let source_fingerprint = (compute_yzx_extern_source_fingerprint $runtime_dir)
    if (yzx_extern_bridge_is_current $extern_path $fingerprint_path $source_fingerprint) {
        return $extern_path
    }

    let placeholder = "# Yazelix generated Nushell extern bridge (empty)\n"
    let has_existing_bridge = ($extern_path | path exists)

    if not $has_existing_bridge {
        write_text_atomic $extern_path $placeholder --raw | ignore
    }

    try {
        let extern_content = (fetch_yzx_extern_content $runtime_dir)
        write_text_atomic $extern_path $extern_content --raw | ignore
        write_yzx_extern_bridge_state $fingerprint_path $source_fingerprint $extern_content
    } catch {|err|
        print $"⚠️  Failed to generate Nushell yzx extern bridge: ($err.msg)"
    }

    $extern_path
}
