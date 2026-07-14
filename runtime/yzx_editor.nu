def --wrapped main [...args: string] {
    let fallback = $env.YAZELIX_EDITOR? | default ""
    let configured = (^@yzxConfig@ --get editor.command | complete)
    let selected = if $configured.exit_code == 0 {
        $configured.stdout | str trim
    } else if ($fallback | is-not-empty) {
        $fallback
    } else {
        print --stderr $configured.stderr
        exit $configured.exit_code
    }
    let editor = if $selected in ["yzx-hx" "hx"] { "@yzxHelix@" } else { $selected }
    if (which $editor | is-empty) {
        print --stderr $"Yazelix editor command not found: ($editor). Set editor.command to one executable name or path without arguments."
        exit 127
    }
    $env.YAZELIX_HELIX_BRIDGE = "0"
    run-external $editor ...$args
    let exit_code = $env.LAST_EXIT_CODE? | default 0
    if (($env.ZELLIJ? | default "") | is-not-empty) {
        print --no-newline "\u{1b}]111\u{7}"
    }
    exit $exit_code
}
