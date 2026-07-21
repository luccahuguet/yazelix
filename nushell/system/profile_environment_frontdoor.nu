# Profile-owned environment frontdoor for stateful foundation tools.
#
# The installed selector remains the only executable owner. On the real
# FlexNetOS home, mutable XDG state is redirected into the Meta payload and
# disposable state into the Yazelix runtime root before the immutable payload
# starts. Hermetic package checks with a different HOME retain their fixture
# environment.

const TOOL = "@tool@"
const PAYLOAD = "@payload@"
const REAL_HOME = "@realHome@"
const DATA_HOME = "@dataHome@"
const STATE_HOME = "@stateHome@"
const CACHE_HOME = "@cacheHome@"
const RUNTIME_DIR = "@runtimeDir@"
const YAZELIX_STATE_DIR = "@yazelixStateDir@"
const PROFILE_NU = "@profileNu@"
const CHMOD = "@chmod@"

def fail [message: string] {
    print --stderr $"profile-owned ($TOOL) environment: ($message)"
    exit 1
}

def --env activate-profile-environment [] {
    if (($env.HOME? | default "") != $REAL_HOME) {
        return
    }

    for directory in [$DATA_HOME $STATE_HOME $CACHE_HOME $YAZELIX_STATE_DIR] {
        mkdir $directory
    }
    for directory in [$CACHE_HOME $YAZELIX_STATE_DIR] {
        let secured = (do { ^$CHMOD 0700 $directory } | complete)
        if $secured.exit_code != 0 {
            fail $"unable to secure runtime directory: ($directory)"
        }
    }

    $env.XDG_DATA_HOME = $DATA_HOME
    $env.XDG_STATE_HOME = $STATE_HOME
    $env.XDG_CACHE_HOME = $CACHE_HOME
    $env.XDG_RUNTIME_DIR = $RUNTIME_DIR
    $env.YAZELIX_STATE_DIR = $YAZELIX_STATE_DIR
    $env.SHELL = $PROFILE_NU
}

def --wrapped main [...args] {
    activate-profile-environment
    exec $PAYLOAD ...$args
}
