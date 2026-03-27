#!/usr/bin/env nu

use ../utils/config_migrations.nu validate_config_migration_rules

export def main [] {
    let errors = (validate_config_migration_rules)

    if not ($errors | is-empty) {
        $errors | each {|line| print $"❌ ($line)" }
        error make {msg: "Config migration rule validation failed"}
    }

    print "✅ Config migration rule metadata is valid"
}
