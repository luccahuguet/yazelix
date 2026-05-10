# Startup Performance

These percentages come from saved startup-profile baselines on the same maintainer machine

- Warm current-terminal compares the v15.3 release-era branch against an April 18, 2026 pre-v15.2 v15 baseline
- Cold clear-cache current-terminal compares the v15.3 release-era branch against an April 4, 2026 late-v13.11, pre-v13.12 baseline
- Desktop launch and managed new-window launch compare the v15.3 release-era branch against April 18, 2026 pre-v15.2 v15 baselines

A large part of the gain comes from Rust ownership cuts across config, materialization, and generated-file work, combined with delete-first removal of redundant Nushell owner seams

- Warm current-terminal startup is 75.6% faster
- Cold clear-cache current-terminal startup is 95.6% faster
- Desktop launch startup is 55.6% faster
- Managed new-window launch startup is 59.0% faster

These numbers come from the built-in structured startup profiler under `~/.local/share/yazelix/profiles/startup/`

You can collect your own reports with `yzx dev profile`, `yzx dev profile --cold --clear-cache`, `yzx dev profile --desktop`, and `yzx dev profile --launch`, then compare saved runs with `yzx dev profile compare`
