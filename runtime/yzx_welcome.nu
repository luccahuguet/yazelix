def --wrapped main [...command: string] {
    let enabled = ($env.YZX_WELCOME_ENABLED? | default "true") != "false"
    if $enabled {
        let style = $env.YZX_WELCOME_STYLE? | default "random"
        let duration = $env.YZX_WELCOME_DURATION_SECONDS? | default "3"
        with-env { YAZELIX_SCREEN_COMMAND_NAME: "yzx screen" } {
            ^@yzs@ $style --duration-seconds $duration
        }
        if ($env.LAST_EXIT_CODE? | default 0) != 0 {
            print --stderr "yzx welcome: failed to render welcome screen"
        }
    }
    if ($command | is-not-empty) {
        exec ...$command
    }
}
