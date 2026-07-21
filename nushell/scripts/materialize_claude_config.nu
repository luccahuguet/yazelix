# Materialize reviewed, secret-free Claude configuration into profile runtime.
# Credentials, sessions, histories, databases, and other mutable Claude state
# are deliberately untouched.

const RECEIPT_SCHEMA = "yazelix.claude-config-generation.v1"
const CANONICAL_SETTINGS_INPUT = "/home/flexnetos/.nix-profile/share/yazelix/agent_configs/claude/settings.json.src"
const CANONICAL_CLAUDE_INPUT = "/home/flexnetos/.nix-profile/share/yazelix/agent_configs/claude/CLAUDE.md.src"
const CANONICAL_RTK_INPUT = "/home/flexnetos/.nix-profile/share/yazelix/agent_configs/claude/RTK.md.src"

def fail [message: string] {
    print --stderr $"claude config materializer: ($message)"
    exit 1
}

def source-hash [path: path] {
    open --raw $path | hash sha256
}

def validate-source [path: path] {
    if not ($path | path exists) { fail $"reviewed input is absent: ($path)" }
    let resolved = ($path | path expand --strict)
    if (($resolved | path type) != "file") { fail $"reviewed input is not a file: ($path)" }
    let content = (open --raw $path)
    if ($content | is-empty) { fail $"reviewed input is empty: ($path)" }
    let retired_home_tree = (["." "local"] | str join)
    let retired_agent_tree = (["." "codex"] | str join)
    let retired_claude_tree = (["." "claude"] | str join)
    for forbidden in [
        $"/home/flexnetos/($retired_home_tree)"
        $"/home/flexnetos/($retired_agent_tree)"
        $"/home/flexnetos/($retired_claude_tree)"
        "/home/flexnetos/FlexNetOS"
        "/nix/store/"
        "/nix/var/nix/profiles/"
    ] {
        if ($content | str contains $forbidden) {
            fail $"reviewed input contains a competing ownership path: ($path)"
        }
    }
}

def publish [source: path, target: path, mode: int] {
    if ($target | path exists) and (($target | path type) == "dir") {
        fail $"generated output is a directory: ($target)"
    }
    let directory = ($target | path dirname | path expand)
    mkdir $directory
    let stage = ($directory | path join $".($target | path basename).yazelix-stage-(random uuid)")
    try {
        open --raw $source | save --raw --force $stage
        ^chmod $mode $stage
        mv --force $stage $target
        do { ^sync -f $directory } | complete | ignore
    } catch {|err|
        if ($stage | path exists) { rm --force $stage }
        fail ($err.msg? | default ($err | to json --raw))
    }
}

def main [
    settings_src: path
    settings_out: path
    claude_src: path
    claude_out: path
    rtk_src: path
    rtk_out: path
] {
    for source in [$settings_src $claude_src $rtk_src] { validate-source $source }
    try { open --raw $settings_src | from json | ignore } catch {
        fail $"reviewed Claude settings are invalid JSON: ($settings_src)"
    }
    let settings = (open --raw $settings_src)
    for required in [
        "/home/flexnetos/.nix-profile/toolbin/rtk hook claude"
        "/home/flexnetos/.nix-profile/toolbin/icm hook pre"
        "/home/flexnetos/.nix-profile/toolbin/icm hook end"
    ] {
        if not ($settings | str contains $required) {
            fail $"reviewed Claude settings omit required profile hook: ($required)"
        }
    }

    let output_directories = (
        [$settings_out $claude_out $rtk_out]
        | each {|path| $path | path dirname | path expand }
        | uniq
    )
    if ($output_directories | length) != 1 {
        fail "generated Claude outputs must share one profile runtime directory"
    }

    publish $settings_src $settings_out 600
    publish $claude_src $claude_out 644
    publish $rtk_src $rtk_out 644

    let receipt = ($output_directories.0 | path join ".yazelix-claude-generation.json")
    let receipt_stage = ($output_directories.0 | path join $".yazelix-claude-generation-stage-(random uuid)")
    {
        schema: $RECEIPT_SCHEMA
        sources: [
            {installed: $CANONICAL_SETTINGS_INPUT, sha256: (source-hash $settings_src), output: ($settings_out | path basename)}
            {installed: $CANONICAL_CLAUDE_INPUT, sha256: (source-hash $claude_src), output: ($claude_out | path basename)}
            {installed: $CANONICAL_RTK_INPUT, sha256: (source-hash $rtk_src), output: ($rtk_out | path basename)}
        ]
    } | to json --indent 2 | save --raw --force $receipt_stage
    ^chmod 600 $receipt_stage
    mv --force $receipt_stage $receipt
    do { ^sync -f $output_directories.0 } | complete | ignore
    print $"ok materialized Claude settings/instructions under ($output_directories.0)"
}
