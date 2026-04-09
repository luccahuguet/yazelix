#!/usr/bin/env nu

def ensure_parent_dir [target_path: string] {
    let target_dir = ($target_path | path dirname)
    if ($target_dir | is-not-empty) and (not ($target_dir | path exists)) {
        mkdir $target_dir
    }
    $target_dir
}

def create_atomic_temp_path [target_path: string] {
    let target_dir = (ensure_parent_dir $target_path)
    let target_name = ($target_path | path basename)
    ^mktemp $"($target_dir)/.($target_name).yazelix-tmp-XXXXXX" | str trim
}

export def write_text_atomic [target_path: string, content: string, --raw] {
    let temp_path = (create_atomic_temp_path $target_path)

    try {
        if $raw {
            $content | save --force --raw $temp_path
        } else {
            $content | save --force $temp_path
        }
        mv --force $temp_path $target_path
    } catch {|err|
        if ($temp_path | path exists) {
            rm --force $temp_path
        }
        error make {msg: $"Failed to write Yazelix-managed file atomically at ($target_path): ($err.msg)"}
    }

    $target_path
}

export def copy_file_atomic [source_path: string, target_path: string] {
    let temp_path = (create_atomic_temp_path $target_path)

    try {
        cp --force $source_path $temp_path
        mv --force $temp_path $target_path
    } catch {|err|
        if ($temp_path | path exists) {
            rm --force $temp_path
        }
        error make {msg: $"Failed to replace Yazelix-managed file atomically at ($target_path): ($err.msg)"}
    }

    $target_path
}
