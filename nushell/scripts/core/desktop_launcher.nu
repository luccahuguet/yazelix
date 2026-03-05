#!/usr/bin/env nu

# Yazelix Desktop Launcher
# Reuse the yzx launch flow so desktop entries behave the same as CLI launches.

use ../yzx/launch.nu *

def main [] {
    yzx launch --home
}
