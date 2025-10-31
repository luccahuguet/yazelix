# Yazelix Launch Performance

## Understanding Launch Speed

### Fast Launches (Inside Yazelix)

When you're already inside a Yazelix environment:

```bash
# Inside Yazelix shell
❯ yzx launch
# ⚡ Instant (~100ms)
```

**Why it's fast:**
- Nix environment already loaded
- All packages available
- No evaluation needed
- Just spawns terminal with configs

### Smart Launches (Cold Start with Session Detection)

When launching from outside Yazelix, the behavior depends on whether a session exists:

```bash
# From regular shell - NO existing session
$ yzx launch
# ⏱️  Takes ~4 seconds (first launch)

# From regular shell - WITH existing session
$ yzx launch
# ⚡ Instant! (~100ms - attaches to existing session)
```

**Why it's fast with existing sessions:**
- **Session detection**: Checks for existing Zellij session matching your `session_name`
- **Instant attach**: If found, attaches directly (no Nix environment load needed)
- **Automatic optimization**: Works regardless of `persistent_sessions` setting

**Why first launch takes time:**
- **~3.5s**: Nix evaluates flake.nix (even with cache)
- **~0.25s**: shellHook runs (config merging, setup)
- **~0.25s**: Terminal spawns

**Session persistence:**
- Zellij sessions survive terminal closures by default
- Even with `persistent_sessions = false`, sessions may persist if Zellij server is running
- Yazelix now automatically detects and reuses existing sessions

---

## The Simple Truth

### The Elegant Solution

Instead of complex daemons or boot services, Yazelix takes a simpler approach:

**Automatic Session Reattachment:**
1. **First launch of the day**: 4 seconds to create session
2. **All subsequent launches**: Instant! (auto-detects and attaches to existing session)
3. **Launches from inside**: Instant!
4. **Most of your workflow**: Fast!

### Why We Don't Need a Daemon

The session reattachment feature solves the cold start problem elegantly:

1. **Most launches are now instant** - Session detection makes cold starts fast
2. **Only first launch is slow** - 4s once per day is acceptable
3. **No complexity** - Simple session detection, no daemon management
4. **Works automatically** - No configuration or opt-in required
5. **Robust** - Leverages Zellij's built-in session management

No daemon, no complexity, no process management - just smart session reuse.

---

## Launch Speed Comparison

| Scenario | Speed | Why |
|----------|-------|-----|
| `yzx launch` from inside Yazelix | ~100ms | ✅ Environment loaded |
| `yzx launch --here` from inside | ~100ms | ✅ Environment loaded |
| `yzx launch` from outside (session exists) | ~100ms | ✅ Attaches to existing session |
| `yzx launch` from outside (first launch) | ~4000ms | ⚠️ Must create new session |
| `yzx launch --here` from outside (session exists) | ~100ms | ✅ Attaches to existing session |
| Desktop launcher (session exists) | ~100ms | ✅ Attaches to existing session |
| Desktop launcher (first launch) | ~4000ms | ⚠️ Must create new session |

---

## Optimization Tips

### 1. Session Reattachment (Automatic!)

**New in Yazelix:** Cold starts are now instant when a session exists!

```bash
# First launch of the day (4s)
$ yzx launch

# Later launches - instant! (100ms)
$ yzx launch
# Automatically detects and attaches to existing session
```

**How it works:**
- Yazelix checks for existing sessions before loading Nix environment
- If a session matching your `session_name` exists, attaches instantly
- Works regardless of `persistent_sessions` setting
- No configuration needed - it just works!

### 2. Launch From Inside

Once you're in Yazelix, all subsequent `yzx launch` calls are instant:

```bash
# Inside Yazelix, launch more terminals (instant!)
❯ yzx launch
❯ yzx launch --path ~/projects
❯ yzx launch --home
# All instant!
```

### 3. Use Persistent Sessions (Optional)

For guaranteed session persistence, enable in `yazelix.nix`:

```nix
{
  persistent_sessions = true;
  session_name = "main";
}
```

**Note:** Even without this setting, sessions often survive terminal closures, and Yazelix will reuse them automatically.

### 4. Multiple Terminals Without Restart

Inside Yazelix, use `yzx launch` to spawn multiple terminal windows - all instant!

---

## Future: Boot Startup Option

### Planned Feature

For users who want Yazelix always available:

**Option 1: Systemd User Service** (Linux)

```bash
# ~/.config/systemd/user/yazelix.service
[Unit]
Description=Yazelix Terminal Environment
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/env nix develop --impure ~/.config/yazelix --command nu -c "sleep infinity"
Restart=on-failure

[Install]
WantedBy=default.target
```

Enable:
```bash
systemctl --user enable yazelix
systemctl --user start yazelix
```

**Option 2: Auto-Start Script** (All platforms)

Add to shell rc file:

```bash
# ~/.bashrc or ~/.zshrc
if ! pgrep -f "yazelix-startup" > /dev/null; then
    (cd ~/.config/yazelix && nix develop --impure --command nu -c "sleep infinity") &
    disown
fi
```

**Option 3: Login Launch** (macOS)

Use launchd:

```xml
<!-- ~/Library/LaunchAgents/com.yazelix.startup.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.yazelix.startup</string>
    <key>ProgramArguments</key>
    <array>
        <string>nix</string>
        <string>develop</string>
        <string>--impure</string>
        <string>/Users/YOU/.config/yazelix</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
```

### Why Not Included Yet?

1. **Most users don't need it** - Launching once per session is fine
2. **Platform-specific** - Requires different solutions per OS
3. **Resource usage** - Keeps Nix environment in memory
4. **Complexity** - Need proper lifecycle management

We'll add this as an **optional feature** when there's demand.

---

## Performance Benchmarking

Want to measure launch speed on your system?

```bash
# Benchmark terminal launches
yzx bench -n 5

# Test specific terminal
yzx bench -t ghostty -n 10

# Compare all terminals
yzx bench
```

Typical results:
- **From inside Yazelix**: 80-120ms
- **Cold start**: 3500-4500ms

---

## Summary

✅ **Inside Yazelix**: Launches are instant (~100ms)
✅ **Cold starts with existing session**: Instant (~100ms) - automatically detected!
✅ **First launch only**: 4s for initial session creation
✅ **No configuration needed**: Session reattachment works out of the box
✅ **Simple, maintainable**: Clean codebase without daemon complexity

**The elegant solution: Let Zellij sessions persist, detect and reuse them automatically.**
