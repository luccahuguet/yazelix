#!/usr/bin/env nu

use atomic_writes.nu write_text_atomic
use common.nu [get_yazelix_runtime_dir get_yazelix_state_dir resolve_yazelix_nu_bin]

const YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION = 1
const YZX_EXTERN_BRIDGE_RENDERER_VERSION = "v1"

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

def get_yzx_command_surface_paths [runtime_dir: string] {
    let expanded_runtime = ($runtime_dir | path expand)
    let core_path = ($expanded_runtime | path join "nushell" "scripts" "core" "yazelix.nu")
    let renderer_path = ($expanded_runtime | path join "nushell" "scripts" "utils" "nushell_externs.nu")
    let yzx_dir = ($expanded_runtime | path join "nushell" "scripts" "yzx")
    let yzx_files = if ($yzx_dir | path exists) {
        glob ($yzx_dir | path join "*.nu") | sort
    } else {
        []
    }

    [$core_path, $renderer_path]
    | append $yzx_files
    | each {|path| $path | path expand }
    | uniq
    | sort
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

def compute_yzx_extern_source_fingerprint [runtime_dir: string] {
    let expanded_runtime = ($runtime_dir | path expand)
    {
        schema_version: $YZX_EXTERN_BRIDGE_STATE_SCHEMA_VERSION
        renderer_version: $YZX_EXTERN_BRIDGE_RENDERER_VERSION
        runtime_dir: $expanded_runtime
        command_surface: (
            get_yzx_command_surface_paths $expanded_runtime
            | each {|path| fingerprint_file $path }
        )
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

def render_flag [parameter: record] {
    let name = ($parameter.parameter_name? | default "")
    let short_flag = ($parameter.short_flag? | default "")
    if ($short_flag | is-empty) {
        $"    --($name)"
    } else {
        ("    --" + $name + "(-" + $short_flag + ")")
    }
}

def normalize_shape [syntax_shape?: string] {
    let shape = ($syntax_shape | default "" | into string | str trim)
    match $shape {
        "" => "string"
        "any" => "string"
        "list<string>" => "string"
        _ => $shape
    }
}

def render_named [parameter: record] {
    let name = ($parameter.parameter_name? | default "")
    let short_flag = ($parameter.short_flag? | default "")
    let shape = (normalize_shape ($parameter.syntax_shape? | default "string"))
    if ($short_flag | is-empty) {
        $"    --($name): ($shape)"
    } else {
        ("    --" + $name + "(-" + $short_flag + "): " + $shape)
    }
}

def render_positional [parameter: record] {
    let name = ($parameter.parameter_name? | default "arg")
    let shape = (normalize_shape ($parameter.syntax_shape? | default "string"))
    if ($parameter.is_optional? | default false) {
        $"    ($name)?: ($shape)"
    } else {
        $"    ($name): ($shape)"
    }
}

def render_rest [parameter: record] {
    let name = ($parameter.parameter_name? | default "rest")
    let shape = (normalize_shape ($parameter.syntax_shape? | default "string"))
    $"    ...($name): ($shape)"
}

def render_parameter [parameter: record] {
    let parameter_type = ($parameter.parameter_type? | default "")
    match $parameter_type {
        "switch" => (render_flag $parameter)
        "named" => (render_named $parameter)
        "positional" => (render_positional $parameter)
        "rest" => (render_rest $parameter)
        _ => null
    }
}

def render_extern_block [command: record] {
    let parameters = (
        $command.signatures.any
        | where {|parameter|
            let parameter_type = ($parameter.parameter_type? | default "")
            $parameter_type not-in ["input", "output"]
        }
        | each {|parameter| render_parameter $parameter }
        | where {|line| $line != null }
    )

    if ($parameters | is-empty) {
        $"export extern \"($command.name)\" []"
    } else {
        let body = ($parameters | str join "\n")
        $"export extern \"($command.name)\" [\n($body)\n]"
    }
}

def fetch_yzx_command_metadata [runtime_root: string] {
    let runtime_dir = ($runtime_root | path expand)
    let nu_bin = (resolve_yazelix_nu_bin)
    let probe = (do {
        cd $runtime_dir
        ^$nu_bin -c 'source nushell/scripts/core/yazelix.nu; scope commands | where name =~ "^yzx( |$)" | sort-by name | to json -r' | complete
    })

    if $probe.exit_code != 0 {
        let stderr = ($probe.stderr | default "" | str trim)
        error make {msg: $"Failed to inspect yzx command metadata for Nushell extern generation: ($stderr)"}
    }

    $probe.stdout | from json
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
        let commands = (fetch_yzx_command_metadata $runtime_dir)
        let header = [
            "# Generated by Yazelix from the real yzx command tree."
            "# Restores Nushell completion/signature knowledge for the external yzx CLI."
            ""
        ] | str join "\n"
        let body = ($commands | each {|command| render_extern_block $command } | str join "\n\n")
        let rust_control_externs = (
            [
                "# Rust-owned leaf commands (not in the Nushell scope tree; explicit extern parity)."
                "export extern \"yzx env\" ["
                "    --no-shell(-n)"
                "]"
                ""
                "export extern \"yzx run\" ["
                "    ...argv: string"
                "]"
            ] | str join "\n"
        )
        let extern_content = $"($header)($body)\n\n($rust_control_externs)\n"
        write_text_atomic $extern_path $extern_content --raw | ignore
        write_yzx_extern_bridge_state $fingerprint_path $source_fingerprint $extern_content
    } catch {|err|
        print $"⚠️  Failed to generate Nushell yzx extern bridge: ($err.msg)"
    }

    $extern_path
}
