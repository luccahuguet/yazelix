export def make_temp_home [] {
    (^mktemp -d /tmp/yazelix_nixpkgs_package_XXXXXX | str trim)
}

export def require_success [result: record, failure_message: string] {
    if $result.exit_code != 0 {
        if ($result.stdout | is-not-empty) {
            print $result.stdout
        }
        if ($result.stderr | is-not-empty) {
            print $result.stderr
        }
        error make { msg: $failure_message }
    }
}

export def get_package_env [temp_home: string] {
    {
        HOME: $temp_home
        XDG_CONFIG_HOME: ($temp_home | path join ".config")
        XDG_DATA_HOME: ($temp_home | path join ".local" "share")
        SHELL: ($env.SHELL? | default "/bin/sh")
        YAZELIX_DIR: null
    }
}

export def run_yzx [package_root: string, temp_home: string, ...args: string] {
    let yzx_path = ($package_root | path join "bin" "yzx")

    with-env (get_package_env $temp_home) {
        ^$yzx_path ...$args | complete
    }
}

export def verify_yazelix_package [package_root: string] {
    let temp_home = (make_temp_home)

    for forbidden in ["yazelix_packs_default.toml"] {
        if (($package_root | path join $forbidden) | path exists) {
            error make { msg: $"Packaged Yazelix should not ship `($forbidden)`: ($package_root | path join $forbidden)" }
        }
    }

    let version_result = (run_yzx $package_root $temp_home "--version-short")
    require_success $version_result "Packaged yzx --version-short failed"
    let version_text = ($version_result.stdout | str trim)
    if not ($version_text | str starts-with "Yazelix v") {
        error make { msg: $"Unexpected packaged yzx version output: ($version_text)" }
    }

    let doctor_result = (run_yzx $package_root $temp_home "doctor" "--verbose")
    require_success $doctor_result "Packaged yzx doctor --verbose failed"

    let runtime_probe = (
        run_yzx
            $package_root
            $temp_home
            "run"
            "nu"
            "-c"
            'print ({shell: ($env.IN_YAZELIX_SHELL | default ""), runtime: ($env.YAZELIX_RUNTIME_DIR | default ""), path0: (($env.PATH | default []) | get -o 0 | default ""), path1: (($env.PATH | default []) | get -o 1 | default ""), yzx: ((which yzx | get -o 0.path | default ""))} | to json -r)'
    )
    require_success $runtime_probe "Packaged yzx run probe failed"

    let probe = ($runtime_probe.stdout | str trim | from json)
    let expected_bin1 = ($package_root | path join "bin")
    let expected_path0 = ($package_root | path join "libexec")
    if (
        ($probe.shell != "true")
        or ($probe.runtime != $package_root)
        or ($probe.path0 != $expected_path0)
        or ($probe.path1 != $expected_bin1)
        or ($probe.yzx != ($expected_bin1 | path join "yzx"))
    ) {
        error make { msg: $"Packaged Yazelix runtime probe saw the wrong env: ($probe | to json -r)" }
    }
}
