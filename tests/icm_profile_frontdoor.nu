def expect [condition: bool, message: string] {
    if not $condition {
        print --stderr $"FAIL: ($message)"
        exit 1
    }
    print $"ok: ($message)"
}

def run [wrapper: path, args: list<string>] {
    let result = (do { ^$wrapper ...$args } | complete)
    if $result.exit_code != 0 {
        print --stderr $result.stderr
        exit $result.exit_code
    }
    $result.stdout | str trim | from json
}

def main [workdir: path, source: path, nu_bin: path, chmod_bin: path] {
    let root = ($workdir | path join "icm-profile-frontdoor")
    mkdir $root
    let payload = ($root | path join "payload")
    let wrapper = ($root | path join "icm")
    let default_db = ($root | path join "meta" | path join "var" | path join "lib" | path join "icm" | path join "memories.db")

    [
        $"#!($nu_bin)"
        "def --wrapped main [...args] {"
        "    print ($args | each {|arg| $arg | into string } | to json --raw)"
        "}"
        ""
    ] | str join "\n" | save --force $payload
    ^$chmod_bin 755 $payload

    let rendered = (
        open --raw $source
        | str replace --all "@payload@" ($payload | into string)
        | str replace --all "@defaultDb@" ($default_db | into string)
    )
    $"#!($nu_bin)\n($rendered)" | save --force $wrapper
    ^$chmod_bin 755 $wrapper

    let default = (run $wrapper ["recall" "profile ownership"])
    expect ($default == ["--db" ($default_db | into string) "recall" "profile ownership"]) "default invocation injects Meta-owned database"
    expect (($default_db | path dirname | path exists)) "default database parent is materialized"

    let explicit = (run $wrapper ["--db" "/tmp/operator.db" "health"])
    expect ($explicit == ["--db" "/tmp/operator.db" "health"]) "explicit split --db remains authoritative"

    let explicit_equals = (run $wrapper ["--db=/tmp/operator-equals.db" "topics"])
    expect ($explicit_equals == ["--db=/tmp/operator-equals.db" "topics"]) "explicit equals --db remains authoritative"

    print "ok: ICM profile frontdoor contract passed"
}
