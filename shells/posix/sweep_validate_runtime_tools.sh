#!/bin/sh

set -eu

echo "TOOLS_START"
command -v zellij
command -v yazi
command -v hx
echo "TOOLS_END"

echo "VERSION_START"
zellij --version
yazi --version
hx --version
echo "VERSION_END"
