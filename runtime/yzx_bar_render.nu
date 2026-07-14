def main [request: string] {
    let result = (^@bar@ render-yazelix-runtime --json $request | complete)
    if $result.exit_code != 0 {
        print --stderr $result.stderr
        exit $result.exit_code
    }
    let block = ($result.stdout | from json | get plugin_block | str replace --all "YZX {command_version}" "@novaBarLabel@")
    print --no-newline $block
}
