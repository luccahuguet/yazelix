#!/usr/bin/env nu

use ../utils/common.nu get_yazelix_runtime_dir
use ../utils/atomic_writes.nu write_text_atomic
use ../utils/safe_remove.nu remove_path_within_root

const runtime_dir_placeholder = "__YAZELIX_RUNTIME_DIR__"

export def render_runtime_root_placeholders [content: string] {
    let runtime_dir = (get_yazelix_runtime_dir)
    $content | str replace -a $runtime_dir_placeholder $runtime_dir
}

def render_runtime_root_placeholders_in_directory [root_dir: string] {
    if not ($root_dir | path exists) {
        return
    }

    let candidate_files = (
        ls -la $root_dir
        | where type == file
        | get name
    )

    for file_path in $candidate_files {
        let content = (open --raw $file_path)
        if ($content | str contains $runtime_dir_placeholder) {
            let chmod_result = (^chmod u+w $file_path | complete)
            if ($chmod_result.exit_code != 0) {
                error make {msg: $"Failed to make generated Yazi plugin file writable at ($file_path): ($chmod_result.stderr | str trim)"}
            }
            let rendered = (render_runtime_root_placeholders $content)
            write_text_atomic $file_path $rendered --raw | ignore
        }
    }

    let child_dirs = (
        ls -la $root_dir
        | where type == dir
        | get name
    )

    for child_dir in $child_dirs {
        render_runtime_root_placeholders_in_directory $child_dir
    }
}

def copy_plugins_directory [source_dir: string, merged_dir: string, --quiet] {
    let source_plugins = $"($source_dir)/plugins"
    let merged_plugins = $"($merged_dir)/plugins"

    if not $quiet {
        print "   📁 Copying plugins directory..."
    }

    if not ($merged_plugins | path exists) {
        mkdir $merged_plugins
    }

    if ($source_plugins | path exists) {
        let bundled_plugins = (ls $source_plugins | where type == dir | get name)

        for plugin_path in $bundled_plugins {
            let plugin_name = ($plugin_path | path basename)
            let target = $"($merged_plugins)/($plugin_name)"

            if ($target | path exists) {
                let chmod_result = (^chmod -R u+w $target | complete)
                if ($chmod_result.exit_code != 0) and (($chmod_result.stderr | str trim) | is-not-empty) {
                    print $"⚠ Failed to relax Yazi plugin permissions before cleanup: ($chmod_result.stderr | str trim)"
                }
                try {
                    remove_path_within_root $target $merged_plugins $"bundled Yazi plugin ($plugin_name)" --recursive
                } catch {|err|
                    error make {msg: $"Failed to remove existing Yazelix Yazi plugin at ($target): ($err.msg)"}
                }
            }

            let copy_result = (^cp -R $plugin_path $target | complete)
            if $copy_result.exit_code != 0 {
                error make {msg: $"Failed to copy Yazi plugin from ($plugin_path) to ($target): ($copy_result.stderr | str trim)"}
            }

            let chmod_result = (^chmod -R u+w $target | complete)
            if $chmod_result.exit_code != 0 {
                error make {msg: $"Failed to make generated Yazi plugin writable at ($target): ($chmod_result.stderr | str trim)"}
            }

            render_runtime_root_placeholders_in_directory $target
        }

        if not $quiet {
            print "     ✅ Yazelix plugins copied (user plugins preserved)"
        }
    }
}

def copy_flavors_directory [source_dir: string, merged_dir: string, --quiet] {
    let source_flavors = $"($source_dir)/flavors"
    let merged_flavors = $"($merged_dir)/flavors"

    if not $quiet {
        print "   🎨 Copying flavor themes..."
    }

    if not ($merged_flavors | path exists) {
        mkdir $merged_flavors
    }

    if ($source_flavors | path exists) {
        let bundled_flavors = (ls $source_flavors | where type == dir | get name)

        for flavor_path in $bundled_flavors {
            let flavor_name = ($flavor_path | path basename)
            let target = $"($merged_flavors)/($flavor_name)"

            if ($target | path exists) {
                let chmod_result = (^chmod -R u+w $target | complete)
                if ($chmod_result.exit_code != 0) and (($chmod_result.stderr | str trim) | is-not-empty) {
                    print $"⚠ Failed to relax Yazi flavor permissions before cleanup: ($chmod_result.stderr | str trim)"
                }
                try {
                    remove_path_within_root $target $merged_flavors $"bundled Yazi flavor ($flavor_name)" --recursive
                } catch {|err|
                    error make {msg: $"Failed to remove existing Yazelix Yazi flavor at ($target): ($err.msg)"}
                }
            }

            let copy_result = (^cp -R $flavor_path $target | complete)
            if $copy_result.exit_code != 0 {
                error make {msg: $"Failed to copy Yazi flavor from ($flavor_path) to ($target): ($copy_result.stderr | str trim)"}
            }

            let chmod_result = (^chmod -R u+w $target | complete)
            if $chmod_result.exit_code != 0 {
                error make {msg: $"Failed to make generated Yazi flavor writable at ($target): ($chmod_result.stderr | str trim)"}
            }
        }

        if not $quiet {
            print $"     ✅ ($bundled_flavors | length) flavor themes copied \(user flavors preserved\)"
        }
    }
}

def sync_starship_yazi_config [source_dir: string, merged_dir: string, --quiet] {
    let source_config = ($source_dir | path join "yazelix_starship.toml")
    let target_config = ($merged_dir | path join "yazelix_starship.toml")

    if not ($source_config | path exists) {
        error make {msg: $"Missing bundled Yazi Starship config at: ($source_config)"}
    }

    if ($target_config | path exists) {
        let chmod_result = (^chmod u+w $target_config | complete)
        if ($chmod_result.exit_code != 0) and (($chmod_result.stderr | str trim) | is-not-empty) {
            print $"⚠ Failed to relax Yazi Starship config permissions before refresh: ($chmod_result.stderr | str trim)"
        }

        try {
            remove_path_within_root $target_config $merged_dir "bundled Yazi Starship config"
        } catch {|err|
            error make {msg: $"Failed to remove existing bundled Yazi Starship config at ($target_config): ($err.msg)"}
        }
    }

    let copy_result = (^cp $source_config $target_config | complete)
    if $copy_result.exit_code != 0 {
        error make {msg: $"Failed to copy bundled Yazi Starship config from ($source_config) to ($target_config): ($copy_result.stderr | str trim)"}
    }

    let chmod_result = (^chmod u+w $target_config | complete)
    if $chmod_result.exit_code != 0 {
        error make {msg: $"Failed to make generated Yazi Starship config writable at ($target_config): ($chmod_result.stderr | str trim)"}
    }

    if not $quiet {
        print "     ✅ Bundled Yazi Starship config synced"
    }
}

export def sync_bundled_yazi_assets [source_dir: string, merged_dir: string, --quiet] {
    copy_plugins_directory $source_dir $merged_dir --quiet=$quiet
    copy_flavors_directory $source_dir $merged_dir --quiet=$quiet
    sync_starship_yazi_config $source_dir $merged_dir --quiet=$quiet
}
