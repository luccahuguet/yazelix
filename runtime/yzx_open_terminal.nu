def main [target: path] {
    let cwd = if (($target | path exists) and (($target | path type) == "dir")) {
        $target
    } else {
        $target | path dirname
    }
    exec @zellij@ action new-pane --cwd $cwd
}
