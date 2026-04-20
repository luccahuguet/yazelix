#!/usr/bin/env nu

export def fix_helix_runtime_conflicts [conflicts: list] {
    mut success = true

    for $conflict in $conflicts {
        if $conflict.severity == "error" {
            let backup_path = $"($conflict.path).backup"

            let move_result = try {
                mv $conflict.path $backup_path
                print $"✅ Moved ($conflict.name) from ($conflict.path) to ($backup_path)"
                true
            } catch {
                print $"❌ Failed to move ($conflict.name) from ($conflict.path)"
                false
            }

            if not $move_result {
                $success = false
            }
        }
    }

    $success
}
