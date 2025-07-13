#!/usr/bin/env nu
# Simple wrapper for reveal_in_yazi function

use ./yazi.nu reveal_in_yazi

def main [buffer_name: string] {
    reveal_in_yazi $buffer_name
}