def fail [message: string] {
    print --stderr $"cache/shell policy: ($message)"
    exit 1
}

def main [root: path] {
    let cache_tokens = [
        (["cach" "ix"] | str join)
        (["actions" "/cache"] | str join)
        (["Swatinem" "/rust-cache"] | str join)
        (["magic" "-nix-cache"] | str join)
        "type=gha"
        "cache-to:"
        "cache-from:"
    ]
    let workflow_files = glob $"($root)/.github/workflows/*.yml"
    for file in ($workflow_files | append $"($root)/flake.nix") {
        let content = open --raw $file
        for token in $cache_tokens {
            if ($content | str contains $token) {
                fail $"forbidden non-Kache cache token ($token | to nuon) in ($file)"
            }
        }
    }
    for file in $workflow_files {
        let content = open --raw $file
        if $content =~ '(?m)^\s*shell:\s*(bash|sh|zsh)\s*$' {
            fail $"non-Nushell workflow shell in ($file)"
        }
        if (($content | str contains "run:") and not ($content | str contains "shell: nu {0}")) {
            fail $"workflow with run steps does not select shell: nu {0}: ($file)"
        }
    }
    let shell_source_files = glob $"($root)/**/*.{sh,bash,zsh}"
    if ($shell_source_files | is-not-empty) {
        fail $"repository contains POSIX shell sources: ($shell_source_files | str join ', ')"
    }
    let scan_roots = ["checks" "crates" "runtime" "packaging" ".github"]
    let scanned_sources = (
        $scan_roots
        | each {|relative|
            let directory = $"($root)/($relative)"
            if ($directory | path exists) {
                glob $"($directory)/**/*"
                | where {|file|
                    let is_file = ($file | path type) == "file"
                    let supported = ($file | path parse | get extension) in ["rs" "nu" "nix" "yml" "yaml"]
                    $is_file and $supported
                }
            } else {
                []
            }
        }
        | flatten
        | append $"($root)/flake.nix"
    )
    let prohibited_shells = [(["s" "h"] | str join) (["ba" "sh"] | str join) (["z" "sh"] | str join)]
    for file in $scanned_sources {
        let content = open --raw $file
        for shell in $prohibited_shells {
            let markers = [
                ("#!/bin/" + $shell)
                ("#!/usr/bin/env " + $shell)
                ("#!/usr/bin/env -S " + $shell)
                ("Command::new(\"" + $shell + "\")")
            ]
            for marker in $markers {
                if ($content | str contains $marker) {
                    fail $"forbidden automatic/test shell execution marker ($marker | to nuon) in ($file)"
                }
            }
        }
    }
    let flake = open --raw $"($root)/flake.nix"
    for marker in ["writeShellApplication" "runtimeShell" "patchShebangs"] {
        if ($flake | str contains $marker) {
            fail $"flake still generates shell runtime via ($marker)"
        }
    }
    print "ok cache/shell policy: Kache only; repository automation, tests, workflows, and runtime are Nushell only"
}
