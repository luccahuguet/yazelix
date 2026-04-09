#!/usr/bin/env nu

def expand_managed_root [root_path: string] {
    $root_path | path expand --no-symlink | into string
}

def expand_managed_target [target_path: string] {
    $target_path | path expand --no-symlink | into string
}

def require_target_within_root [target_path: string, root_path: string, label: string] {
    let expanded_root = (expand_managed_root $root_path)
    let expanded_target = (expand_managed_target $target_path)
    let root_prefix = $"($expanded_root)/"

    if ($expanded_target == $expanded_root) or (not ($expanded_target | str starts-with $root_prefix)) {
        error make {
            msg: $"Refusing to remove ($label) outside its managed root.\nTarget: ($expanded_target)\nRoot: ($expanded_root)"
        }
    }

    $expanded_target
}

export def remove_path_within_root [
    target_path: string
    root_path: string
    label: string
    --recursive(-r)
] {
    let expanded_target = (expand_managed_target $target_path)
    if not ($expanded_target | path exists) {
        return
    }

    let guarded_target = (require_target_within_root $expanded_target $root_path $label)
    if $recursive {
        rm -rf $guarded_target
    } else {
        rm --force $guarded_target
    }
}
