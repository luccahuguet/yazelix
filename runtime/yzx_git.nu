def --wrapped main [...args: string] {
    $env.EDITOR = "@yzxEditor@"
    $env.VISUAL = "@yzxEditor@"
    $env.GIT_EDITOR = "@yzxEditor@"
    let ambient = $env.LG_CONFIG_FILE? | default ""
    let native = if ($ambient | is-empty) {
        let result = (^@lazygit@ --print-config-dir | complete)
        if $result.exit_code == 0 {
            let candidate = $"(($result.stdout | str trim))/config.yml"
            if ($candidate | path exists) { $candidate } else { "" }
        } else {
            ""
        }
    } else {
        $ambient
    }
    $env.LG_CONFIG_FILE = if ($native | is-empty) { "@yzxLazyGitConfig@" } else { $"($native),@yzxLazyGitConfig@" }
    exec @lazygit@ ...$args
}
