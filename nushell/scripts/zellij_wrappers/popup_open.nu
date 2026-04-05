#!/usr/bin/env nu

use ../yzx/popup.nu *

def main [...popup_args: string] {
    yzx popup ...$popup_args
}
