#!/bin/sh
set -eu

test_id="${YAZELIX_SWEEP_TEST_ID:-unknown}"
session_name="${ZELLIJ_SESSION_NAME:-unknown}"
result_file="/tmp/yazelix_sweep_result_${test_id}.json"
terminal="${YAZELIX_TERMINAL:-unknown}"

json_escape() {
    printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

check_tool() {
    tool_name="$1"
    version_flag="$2"
    if command -v "$tool_name" >/dev/null 2>&1; then
        version_output="$("$tool_name" "$version_flag" 2>/dev/null | sed -n '1p')"
        printf '{"available":true,"version":"%s"}' "$(json_escape "$version_output")"
    else
        printf '{"available":false,"version":null}'
    fi
}

{
    printf '{\n'
    printf '  "test_id":"%s",\n' "$(json_escape "$test_id")"
    printf '  "session":"%s",\n' "$(json_escape "$session_name")"
    printf '  "timestamp":"%s",\n' "$(date '+%Y-%m-%dT%H:%M:%S')"
    printf '  "terminal":"%s",\n' "$(json_escape "$terminal")"
    printf '  "tools":{\n'
    printf '    "zellij":%s,\n' "$(check_tool zellij --version)"
    printf '    "yazi":%s,\n' "$(check_tool yazi --version)"
    printf '    "helix":%s,\n' "$(check_tool hx --version)"
    printf '    "shell":{"available":true,"version":"%s"}\n' "$(json_escape "${SHELL:-unknown}")"
    printf '  },\n'
    printf '  "status":"pass"\n'
    printf '}\n'
} > "$result_file"

sleep 2
