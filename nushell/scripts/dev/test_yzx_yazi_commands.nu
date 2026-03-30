#!/usr/bin/env nu

use ../integrations/yazi.nu [resolve_managed_editor_open_strategy]

def test_missing_managed_editor_opens_new_managed_pane [] {
    print "🧪 Testing shell-opened editors do not get adopted as the managed editor pane..."

    try {
        let result = (resolve_managed_editor_open_strategy "missing")

        if $result.action == "open_new_managed" {
            print "  ✅ missing managed-editor state routes Yazi opens to a new managed editor pane"
            true
        } else {
            print $"  ❌ Unexpected routing result: ($result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

def test_existing_managed_editor_is_reused [] {
    print "🧪 Testing existing managed editor panes are reused..."

    try {
        let result = (resolve_managed_editor_open_strategy "ok")

        if $result.action == "reuse_managed" {
            print "  ✅ existing managed-editor state reuses the managed editor pane"
            true
        } else {
            print $"  ❌ Unexpected routing result: ($result | to json -r)"
            false
        }
    } catch { |err|
        print $"  ❌ Exception: ($err.msg)"
        false
    }
}

export def run_yazi_canonical_tests [] {
    [
        (test_missing_managed_editor_opens_new_managed_pane)
        (test_existing_managed_editor_is_reused)
    ]
}

export def run_yazi_tests [] {
    run_yazi_canonical_tests
}
