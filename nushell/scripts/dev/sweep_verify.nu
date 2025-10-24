#!/usr/bin/env nu
# Sweep Test Verification Script
# Runs inside the launched Zellij session to verify tools are available

# Get test ID from environment variable (set by sweep test launcher)
let test_id = ($env.YAZELIX_SWEEP_TEST_ID? | default "unknown")

# Get session name from Zellij environment
let session_name = ($env.ZELLIJ_SESSION_NAME? | default "unknown")

# Result file path
let result_file = $"/tmp/yazelix_sweep_result_($test_id).json"

print "🔍 Running Yazelix Sweep Test Verification"
print $"Session: ($session_name)"
print $"Test ID: ($test_id)"
print $"Result file: ($result_file)"
print ""

# Check if a command exists and get its version
def check_tool [tool_name: string, version_flag: string] {
    if (which $tool_name | is-not-empty) {
        let version_output = (do {
            ^$tool_name $version_flag
        } | complete | get stdout | lines | first)
        print $"✓ ($tool_name): ($version_output)"
        {
            available: true,
            version: $version_output
        }
    } else {
        print $"✗ ($tool_name): NOT FOUND"
        {
            available: false,
            version: null
        }
    }
}

# Build results
let results = {
    test_id: $test_id,
    session: $session_name,
    timestamp: (date now | format date "%Y-%m-%dT%H:%M:%S"),
    tools: {
        zellij: (check_tool "zellij" "--version"),
        yazi: (check_tool "yazi" "--version"),
        helix: (check_tool "hx" "--version"),
        shell: {
            available: true,
            version: ($env.SHELL? | default "unknown")
        }
    },
    status: "pass"
}

# Write results to JSON file
$results | to json | save --force $result_file

print ""
print $"✅ Verification complete - results written to ($result_file)"
print "Pane will close in 2 seconds..."
sleep 2sec
