def --wrapped main [...args: string] {
    if (($env.YAZELIX_EDITOR? | default "") | is-empty) {
        $env.YAZELIX_EDITOR = "@yzxHelix@"
    }
    $env.EDITOR = "@yzxEditor@"
    $env.VISUAL = "@yzxEditor@"
    $env.GIT_EDITOR = "@yzxEditor@"
    exec @yzxConfig@ ...$args
}
