#!/usr/bin/env nu
# Lightweight wrapper to refresh Yazi sidebar when popup closes
# This runs as a transient floating pane and auto-exits immediately

use ../integrations/yazi.nu [refresh_active_sidebar_yazi]

# Refresh sidebar and exit (pane auto-closes)
refresh_active_sidebar_yazi | ignore
