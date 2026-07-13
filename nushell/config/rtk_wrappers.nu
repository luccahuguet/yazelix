# Yazelix-managed RTK routing for interactive Nushell sessions.
#
# The Yazelix Nix profile owns the external `rtk` binary. These native Nu defs
# route routine interactive commands through that binary; they never provide or
# wrap an RTK executable. Gate evidence and root-cause diagnostics retain RTK
# accounting while bypassing summarization, for example
# `^rtk proxy -- cargo test ...`, `^rtk proxy -- git status`, or
# `^rtk proxy -- codex --version`.

export def --wrapped codex         [...rest] { ^rtk codex ...$rest }
export def --wrapped claude        [...rest] { ^rtk claude ...$rest }
export def --wrapped git           [...rest] { ^rtk git ...$rest }
export def --wrapped gh            [...rest] { ^rtk gh ...$rest }
export def --wrapped glab          [...rest] { ^rtk glab ...$rest }
export def --wrapped gt            [...rest] { ^rtk gt ...$rest }
export def --wrapped cargo         [...rest] { ^rtk cargo ...$rest }
export def --wrapped go            [...rest] { ^rtk go ...$rest }
export def --wrapped pnpm          [...rest] { ^rtk pnpm ...$rest }
export def --wrapped npm           [...rest] { ^rtk npm ...$rest }
export def --wrapped npx           [...rest] { ^rtk npx ...$rest }
export def --wrapped tsc           [...rest] { ^rtk tsc ...$rest }
export def --wrapped prettier      [...rest] { ^rtk prettier ...$rest }
export def --wrapped jest          [...rest] { ^rtk jest ...$rest }
export def --wrapped vitest        [...rest] { ^rtk vitest ...$rest }
export def --wrapped playwright    [...rest] { ^rtk playwright ...$rest }
export def --wrapped prisma        [...rest] { ^rtk prisma ...$rest }
export def --wrapped pip           [...rest] { ^rtk pip ...$rest }
export def --wrapped pytest        [...rest] { ^rtk pytest ...$rest }
export def --wrapped ruff          [...rest] { ^rtk ruff ...$rest }
export def --wrapped mypy          [...rest] { ^rtk mypy ...$rest }
export def --wrapped rake          [...rest] { ^rtk rake ...$rest }
export def --wrapped rubocop       [...rest] { ^rtk rubocop ...$rest }
export def --wrapped rspec         [...rest] { ^rtk rspec ...$rest }
export def --wrapped dotnet        [...rest] { ^rtk dotnet ...$rest }
export def --wrapped gradlew       [...rest] { ^rtk gradlew ...$rest }
export def --wrapped golangci-lint [...rest] { ^rtk golangci-lint ...$rest }
export def --wrapped docker        [...rest] { ^rtk docker ...$rest }
export def --wrapped kubectl       [...rest] { ^rtk kubectl ...$rest }
export def --wrapped aws           [...rest] { ^rtk aws ...$rest }
export def --wrapped psql          [...rest] { ^rtk psql ...$rest }
export def --wrapped curl          [...rest] { ^rtk curl ...$rest }
export def --wrapped wget          [...rest] { ^rtk wget ...$rest }
export def --wrapped meta          [...rest] { ^rtk meta ...$rest }
export def --wrapped kimi          [...rest] { ^rtk kimi ...$rest }
export def --wrapped ollama        [...rest] { ^rtk ollama ...$rest }
