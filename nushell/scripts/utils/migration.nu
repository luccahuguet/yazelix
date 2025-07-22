#!/usr/bin/env nu
# ~/.config/yazelix/nushell/scripts/utils/migration.nu
# Migration utilities for XDG compliance

# Migrate files from old to new locations for XDG compliance
export def migrate_to_xdg_directories [] {
    use ./constants.nu *

    print "🔄 Checking for files to migrate to XDG-compliant directories..."

    # Migrate old logs directory
    migrate_logs
    
    # Migrate old initializer directories
    migrate_initializers

    print "✅ XDG migration check completed"
}

# Migrate logs from old config directory to new state directory
def migrate_logs [] {
    use ./constants.nu *
    
    let old_logs_dir = ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME | path join "logs")
    let new_logs_dir = ($YAZELIX_LOGS_DIR | str replace "~" $env.HOME)

    if ($old_logs_dir | path exists) {
        print $"📂 Migrating logs from ($old_logs_dir) to ($new_logs_dir)"
        
        # Ensure new directory exists
        mkdir $new_logs_dir
        
        # Move all log files
        try {
            let log_files = (ls $old_logs_dir | where type == file)
            if not ($log_files | is-empty) {
                for file in $log_files {
                    let dest_file = ($new_logs_dir | path join ($file.name | path basename))
                    mv $file.name $dest_file
                    print $"  ✓ Moved ($file.name | path basename)"
                }
            }
            
            # Remove old directory if empty
            let remaining_files = (ls $old_logs_dir)
            if ($remaining_files | is-empty) {
                rmdir $old_logs_dir
                print $"  ✓ Removed empty old logs directory"
            } else {
                print $"  ⚠️  Old logs directory not empty, keeping: ($old_logs_dir)"
            }
        } catch {|err|
            print $"  ⚠️  Error migrating logs: ($err.msg)"
        }
    }
}

# Migrate initializers from old locations to new state directory
def migrate_initializers [] {
    use ./constants.nu *
    
    let config_dir = ($YAZELIX_CONFIG_DIR | str replace "~" $env.HOME)
    
    # Old initializer locations that might exist
    let old_locations = [
        ($config_dir | path join "nushell/initializers")
        ($config_dir | path join "shells/bash/initializers") 
        ($config_dir | path join "shells/fish/initializers")
        ($config_dir | path join "shells/zsh/initializers")
    ]

    mut any_migrated = false
    
    for old_dir in $old_locations {
        if ($old_dir | path exists) {
            print $"📂 Found old initializers directory: ($old_dir)"
            
            # Determine shell type and new location
            let shell_type = if ($old_dir | str contains "nushell") { 
                "nushell" 
            } else if ($old_dir | str contains "bash") { 
                "bash" 
            } else if ($old_dir | str contains "fish") { 
                "fish" 
            } else if ($old_dir | str contains "zsh") { 
                "zsh" 
            } else { 
                "unknown" 
            }
            
            if $shell_type != "unknown" {
                let new_dir = ($SHELL_INITIALIZER_DIRS | get $shell_type | str replace "~" $env.HOME)
                
                try {
                    # Ensure new directory exists
                    mkdir $new_dir
                    
                    let init_files = (ls $old_dir | where type == file)
                    if not ($init_files | is-empty) {
                        print $"  → Migrating ($shell_type) initializers to ($new_dir)"
                        
                        for file in $init_files {
                            let dest_file = ($new_dir | path join ($file.name | path basename))
                            mv $file.name $dest_file
                            print $"    ✓ Moved ($file.name | path basename)"
                        }
                        $any_migrated = true
                    }
                    
                    # Remove old directory if empty
                    let remaining_files = (ls $old_dir)
                    if ($remaining_files | is-empty) {
                        rmdir $old_dir
                        print $"    ✓ Removed empty old initializers directory"
                    }
                } catch {|err|
                    print $"  ⚠️  Error migrating ($shell_type) initializers: ($err.msg)"
                }
            }
        }
    }
    
    if $any_migrated {
        print "  💡 Initializers will be regenerated with current tool versions on next startup"
    }
} 