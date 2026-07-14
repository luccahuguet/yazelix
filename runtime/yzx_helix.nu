def required_owned_dir [explicit: any, xdg: any, home: any, leaf: string, label: string, xdg_label: string] {
    if (($explicit | default "") | is-not-empty) {
        $explicit
    } else if (($xdg | default "") | is-not-empty) {
        $"($xdg)/yazelix"
    } else if (($home | default "") | is-not-empty) {
        $"($home)/($leaf)/yazelix"
    } else {
        error make {msg: $"yzx-hx: HOME is required when ($label) and ($xdg_label) are unset"}
    }
}

def --wrapped main [...args: string] {
    let state_dir = (required_owned_dir $env.YAZELIX_STATE_DIR? $env.XDG_DATA_HOME? $env.HOME? ".local/share" "YAZELIX_STATE_DIR" "XDG_DATA_HOME")
    let config_home = (required_owned_dir $env.YAZELIX_CONFIG_HOME? $env.XDG_CONFIG_HOME? $env.HOME? ".config" "YAZELIX_CONFIG_HOME" "XDG_CONFIG_HOME")
    $env.YAZELIX_STATE_DIR = $state_dir
    $env.YAZELIX_CONFIG_HOME = $config_home

    let user_helix_dir = $"($config_home)/helix"
    let user_helix_config = $"($user_helix_dir)/config.toml"
    let packaged_helix_dir = "@yzxHelixConfig@"
    let packaged_helix_config = $"($packaged_helix_dir)/config.toml"
    let packaged_steel_dir = "@yzxHelixSteelConfig@"
    let effective_helix_config = $"($state_dir)/helix/config.toml"
    let user_steel = (($"($user_helix_dir)/helix.scm" | path exists) and ($"($user_helix_dir)/init.scm" | path exists))
    let user_config = (($user_helix_config | path exists) or ($"($user_helix_dir)/languages.toml" | path exists) or $user_steel)
    let helix_config_dir = if $user_config { $user_helix_dir } else { $packaged_helix_dir }
    let steel_config_dir = if $user_steel { $user_helix_dir } else if $user_config { $"($state_dir)/helix-steel" } else { $packaged_steel_dir }
    $env.HELIX_STEEL_CONFIG = $steel_config_dir

    if (($env.YAZELIX_HELIX_BRIDGE? | default "1") != "0") {
        let stamp = date now | format date "%s"
        if (($env.YAZELIX_HELIX_BRIDGE_SESSION_ID? | default "") | is-empty) {
            $env.YAZELIX_HELIX_BRIDGE_SESSION_ID = $"yzx-helper-($stamp)-($nu.pid)"
        }
        $env.YAZELIX_HELIX_BRIDGE = "1"
        $env.YAZELIX_HELIX_BRIDGE_INSTANCE_ID = $"hx-($stamp)-($nu.pid)"
        $env.YAZELIX_HELIX_BRIDGE_AUTH_TOKEN = (^@od@ -An -N32 -tx1 /dev/urandom | ^@tr@ -d " \n" | str trim)
        $env.YAZELIX_HELIX_MANAGED_CONFIG_PATH = $effective_helix_config
    }

    mkdir $state_dir
    let materialize = (^@yzxConfig@ --write-effective-helix-config $packaged_helix_config $user_helix_config $effective_helix_config | complete)
    if $materialize.exit_code != 0 {
        print --stderr $materialize.stderr
        exit $materialize.exit_code
    }
    if $user_config and not $user_steel {
        mkdir $steel_config_dir
    }
    exec @hx@ --config-dir $helix_config_dir -c $effective_helix_config ...$args
}
