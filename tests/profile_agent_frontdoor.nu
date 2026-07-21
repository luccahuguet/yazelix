def fail [message: string] {
    print --stderr $"profile agent frontdoor test: ($message)"
    exit 1
}

def expect [condition: bool, message: string] {
    if not $condition { fail $message }
}

def render [source: path, destination: path, values: record] {
    mut rendered = (open --raw $source)
    for key in ($values | columns) {
        $rendered = ($rendered | str replace --all $"@($key)@" ($values | get $key))
    }
    $rendered | save --raw --force $destination
}

def executable-script [path: path, nu_bin: path, body: string, chmod_bin: path] {
    $"#!($nu_bin)(char newline)($body)" | save --raw --force $path
    ^$chmod_bin 0755 $path
}

def mode [path: path] {
    ls -la ($path | path dirname)
    | where {|entry| $entry.name == ($path | into string) }
    | get 0.mode
}

def main [
    root: path
    source: path
    nu_bin: path
    chmod_bin: path
    readlink_bin: path
    ln_bin: path
] {
    let profile = ($root | path join "profile")
    let runtime = ($root | path join "run")
    let marker = ($root | path join "materialized")
    mkdir $profile
    ^$ln_bin --symbolic --no-target-directory $runtime ($profile | path join "runtime")

    let payload = ($root | path join "payload")
    let payload_body = ('def --wrapped main [...args] {
  {
    codex_home: ($env.CODEX_HOME? | default "")
    claude_config_dir: ($env.CLAUDE_CONFIG_DIR? | default "")
    materialized: (("__MATERIALIZER_MARKER__" | path exists))
    args: $args
  } | to json
}
' | str replace "__MATERIALIZER_MARKER__" ($marker | into string))
    executable-script $payload $nu_bin $payload_body $chmod_bin

    let materializer = ($root | path join "materializer")
    executable-script $materializer $nu_bin $'def main [] { "ok" | save --force "($marker)" }
' $chmod_bin

    let common = {
        profileRoot: ($profile | into string)
        runtimeTarget: ($runtime | into string)
        payload: ($payload | into string)
        chmod: ($chmod_bin | into string)
        readlink: ($readlink_bin | into string)
    }
    let codex = ($root | path join "codex")
    render $source $codex ($common | merge {agent: "codex", materializer: ($materializer | into string)})
    let codex_result = (do { ^$nu_bin $codex "resume" "fixture" } | complete)
    expect ($codex_result.exit_code == 0) $"Codex wrapper failed: ($codex_result.stderr)"
    let codex_report = ($codex_result.stdout | from json)
    expect ($codex_report.codex_home == ($profile | path join "runtime/codex" | into string)) "Codex state escaped the profile runtime link"
    expect $codex_report.materialized "Codex payload ran before configuration materialization"
    expect ($codex_report.args == ["resume" "fixture"]) "Codex wrapper did not preserve arguments"
    expect ((mode ($runtime | path join "codex")) == "rwx------") "Codex runtime mode is not 0700"

    let competing_codex = (with-env {CODEX_HOME: ($root | path join "competing" | into string)} {
        do { ^$nu_bin $codex "--version" } | complete
    })
    expect ($competing_codex.exit_code != 0) "Codex wrapper accepted a competing CODEX_HOME"

    rm --force $marker
    let claude = ($root | path join "claude")
    render $source $claude ($common | merge {agent: "claude", materializer: ($materializer | into string)})
    let claude_result = (do { ^$nu_bin $claude "--resume" } | complete)
    expect ($claude_result.exit_code == 0) $"Claude wrapper failed: ($claude_result.stderr)"
    let claude_report = ($claude_result.stdout | from json)
    expect ($claude_report.claude_config_dir == ($profile | path join "runtime/claude" | into string)) "Claude state escaped the profile runtime link"
    expect $claude_report.materialized "Claude payload ran before configuration materialization"
    expect ($claude_report.args == ["--resume"]) "Claude wrapper did not preserve arguments"
    expect ((mode ($runtime | path join "claude")) == "rwx------") "Claude runtime mode is not 0700"

    let competing_claude = (with-env {CLAUDE_CONFIG_DIR: ($root | path join "competing" | into string)} {
        do { ^$nu_bin $claude "--version" } | complete
    })
    expect ($competing_claude.exit_code != 0) "Claude wrapper accepted a competing CLAUDE_CONFIG_DIR"

    print "ok profile-owned Codex and Claude frontdoors"
}
