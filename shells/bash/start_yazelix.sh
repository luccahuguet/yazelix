#!/bin/bash
# Bash convenience wrapper around the minimal POSIX Yazelix launcher.

runtime_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
exec "$runtime_dir/shells/posix/start_yazelix.sh" "$@"
