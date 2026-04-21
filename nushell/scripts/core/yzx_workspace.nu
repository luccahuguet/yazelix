#!/usr/bin/env nu
# Workspace-oriented internal commands that still live in Nushell.

use ../integrations/managed_editor.nu get_managed_editor_kind
use ../integrations/yazi.nu [reveal_in_yazi sync_sidebar_yazi_state_to_directory]
use ../integrations/zellij.nu [retarget_tab_cwd resolve_tab_cwd_target]

# Retarget the current Yazelix tab workspace directory
export def "yzx cwd" [
    target?: string  # Directory path or zoxide query for the current tab workspace root (defaults to the current directory)
] {
    if ($env.ZELLIJ? | is-empty) {
        print "❌ yzx cwd only works inside Zellij."
        print "   Start Yazelix first, then run this command from the tab you want to update."
        exit 1
    }

    let resolved_dir = try {
        resolve_tab_cwd_target $target
    } catch {|err|
        print $"❌ ($err.msg)"
        exit 1
    }

    let editor_kind = ((get_managed_editor_kind) | default "")
    let result = try {
        retarget_tab_cwd $resolved_dir $editor_kind "yzx_cwd.log"
    } catch {|err|
        {
            status: "error"
            reason: $err.msg
        }
    }

    match $result.status {
        "ok" => {
            let sidebar_sync_result = if ($result.sidebar_state? | is-not-empty) {
                sync_sidebar_yazi_state_to_directory $result.sidebar_state $result.workspace_root "yzx_cwd.log"
            } else {
                {status: "skipped", reason: "sidebar_yazi_missing"}
            }
            print $"✅ Updated current tab workspace directory to: ($result.workspace_root)"
            print $"   Tab renamed to: ($result.tab_name)"
            print "   The current pane will switch after this command returns."
            print "   Other existing panes keep their current working directories."
            print "   New managed actions will use the updated tab directory."
            if (($result.editor_status? | default "") == "ok") {
                print "   Managed editor cwd synced to the updated directory."
            }
            if $sidebar_sync_result.status == "ok" {
                print "   Sidebar Yazi synced to the updated directory."
            }
        }
        "not_ready" => {
            print "❌ Yazelix tab state is not ready yet."
            print "   Wait a moment for the pane orchestrator plugin to finish loading, then try again."
            exit 1
        }
        "permissions_denied" => {
            print "❌ The Yazelix pane orchestrator plugin is missing required Zellij permissions."
            print "   Run `yzx doctor --fix`, then restart Yazelix."
            exit 1
        }
        _ => {
            let reason = ($result.reason? | default "unknown error")
            print $"❌ Failed to update the current tab workspace directory: ($reason)"
            exit 1
        }
    }
}

# Reveal a file or directory in the managed Yazi sidebar
export def "yzx reveal" [
    target: string  # File or directory to reveal in the managed Yazi sidebar
] {
    reveal_in_yazi $target
}
