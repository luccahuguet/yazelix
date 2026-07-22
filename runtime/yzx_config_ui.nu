def --wrapped main [...args: string] {
    # Upstream contract: the config UI always clears the user editor
    # override so its edits use the managed editor.
    hide-env --ignore-errors YAZELIX_EDITOR
    $env.EDITOR = "@yzxEditor@"
    $env.VISUAL = "@yzxEditor@"
    $env.GIT_EDITOR = "@yzxEditor@"
    exec @yzxConfig@ ...$args
}
