# Profile-owned ICM launcher.
#
# ICM 0.10.57 selects its SQLite database only through the global --db flag.
# Keep the default durable corpus under Meta while preserving an explicit
# operator-supplied --db Tier-B selection.

const PAYLOAD = "@payload@"
const DEFAULT_DB = "@defaultDb@"

def has-db-override [args: list<any>] {
    $args | any {|arg|
        let value = ($arg | into string)
        $value == "--db" or ($value | str starts-with "--db=")
    }
}

def --wrapped main [...args] {
    if (has-db-override $args) {
        exec $PAYLOAD ...$args
    }

    mkdir ($DEFAULT_DB | path dirname)
    exec $PAYLOAD --db $DEFAULT_DB ...$args
}
