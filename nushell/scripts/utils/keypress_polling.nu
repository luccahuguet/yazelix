#!/usr/bin/env nu

export def classify_keypress_poll_error_message [message: string] {
    let normalized = ($message | into string | str trim)

    if ($normalized | str contains "Timed out while waiting for user input") {
        "timeout"
    } else {
        "error"
    }
}

export def poll_for_keypress_status [timeout: duration] {
    try {
        input listen --types [key] --timeout $timeout | ignore
        {
            status: "key"
            message: ""
        }
    } catch {|err|
        let message = ($err.msg? | default "" | into string | str trim)
        let normalized_message = if ($message | is-empty) {
            "Unknown interactive input error"
        } else {
            $message
        }

        {
            status: (classify_keypress_poll_error_message $normalized_message)
            message: $normalized_message
        }
    }
}
