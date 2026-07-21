def main [root: path] {
    let source_root = ($root | path expand)
    let retired_home_tree = (["." "local"] | str join)
    let root_agent_overlay = (["/" "." "codex"] | str join)
    let text_extensions = ["" "conf" "json" "kdl" "lua" "md" "nix" "nu" "path" "rs" "service" "socket" "src" "toml" "yaml" "yml"]
    let candidates = (
        glob --no-dir ($source_root | path join "**/*")
        | where {|path|
            let relative = ($path | path relative-to $source_root)
            (not ($relative | str starts-with ".beads/")) and (not ($relative | str starts-with "assets/")) and (not ($relative | str ends-with ".lock")) and (($relative | path parse | get extension) in $text_extensions)
        }
    )
    mut failures = []
    for path in $candidates {
        let raw = (open --raw $path)
        for pattern in [$retired_home_tree $root_agent_overlay] {
            if ($raw | str contains $pattern) {
                $failures = ($failures | append {
                    path: ($path | path relative-to $source_root)
                    pattern: $pattern
                })
            }
        }
    }
    if not ($failures | is-empty) {
        print --stderr ($failures | to json --indent 2)
        error make {msg: "strict profile source ownership gate failed"}
    }
    print $"ok strict profile source ownership: ($candidates | length) text files"
}
