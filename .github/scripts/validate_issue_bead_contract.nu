#!/usr/bin/env nu

const REPO_ROOT = (
    path self
    | path dirname
    | path join ".." ".."
    | path expand
)

export def main [] {
    let result = (
        do {
            cd $REPO_ROOT
            ^cargo run --quiet --manifest-path rust_core/Cargo.toml -p yazelix_core --bin yzx_repo_maintainer -- sync-issues --dry-run
        } | complete
    )

    if $result.exit_code != 0 {
        let stderr = ($result.stderr | default "" | str trim)
        error make {msg: $"GitHub/Beads contract validation failed: ($stderr)"}
    }

    let stdout = ($result.stdout | default "" | str trim)
    if ($stdout | is-empty) {
        error make {msg: "GitHub/Beads contract validation returned no output."}
    }

    let lines = ($stdout | lines)
    let summary = try {
        $lines | last | from json
    } catch {
        error make {msg: $"GitHub/Beads contract validation returned invalid summary JSON.\n($stdout)"}
    }

    let mutations = (
        [
            ($summary.created? | default 0)
            ($summary.reopened? | default 0)
            ($summary.closed? | default 0)
            ($summary.comments_created? | default 0)
            ($summary.comments_updated? | default 0)
        ] | math sum
    )

    if $mutations != 0 {
        print "Bead/GitHub issue contract violations detected:"
        $lines
        | drop nth (($lines | length) - 1)
        | each { |line| print $line }
        error make {msg: "Bead/GitHub issue contract is invalid" }
    }

    print "Bead/GitHub issue contract is valid."
}
