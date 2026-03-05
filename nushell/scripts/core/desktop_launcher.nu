#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Reuse the yzx launch flow so desktop entries behave the same as CLI launches.

use ../yzx/launch.nu *

def main [] {
    if ($env.YAZELIX_VERBOSE? | default "") == "true" {
        yzx launch --home --verbose
    } else {
        yzx launch --home
    }
}
