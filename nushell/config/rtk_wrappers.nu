# Yazelix-managed RTK routing for interactive Nushell sessions.
#
# The Yazelix Nix profile owns the external `rtk` binary. These native Nu defs
# route routine interactive commands through that binary; they never provide or
# wrap an RTK executable. Gate evidence and root-cause diagnostics retain RTK
# accounting while bypassing summarization, for example
# `^rtk proxy -- cargo test ...`, `^rtk proxy -- git status`, or
# `^rtk proxy -- codex --version`.
#
# Commands with a dedicated RTK filter use it directly. Native tools without a
# dedicated filter still cross the same RTK binary through `proxy`; this keeps
# one executable owner and lets Nushell run Bash/native commands without a
# second wrapper binary or plugin bridge.

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
export def --wrapped rg            [...rest] { ^rtk rg ...$rest }
export def --wrapped grep          [...rest] { ^rtk grep ...$rest }
export def --wrapped tree          [...rest] { ^rtk tree ...$rest }
export def --wrapped diff          [...rest] { ^rtk diff ...$rest }
export def --wrapped wc            [...rest] { ^rtk wc ...$rest }
export def --wrapped meta          [...rest] { ^rtk meta ...$rest }
export def --wrapped kimi          [...rest] { ^rtk kimi ...$rest }
export def --wrapped ollama        [...rest] { ^rtk ollama ...$rest }

# Native passthroughs. These intentionally retain raw output while preserving
# RTK accounting and the profile-owned process ancestry.
export def --wrapped bash          [...rest] { ^rtk proxy -- bash ...$rest }
export def --wrapped sh            [...rest] { ^rtk proxy -- sh ...$rest }
export def --wrapped zsh           [...rest] { ^rtk proxy -- zsh ...$rest }
export def --wrapped fish          [...rest] { ^rtk proxy -- fish ...$rest }
export def --wrapped xonsh         [...rest] { ^rtk proxy -- xonsh ...$rest }
export def --wrapped nu            [...rest] { ^rtk proxy -- nu ...$rest }
export def --wrapped jq            [...rest] { ^rtk proxy -- jq ...$rest }
export def --wrapped sed           [...rest] { ^rtk proxy -- sed ...$rest }
export def --wrapped awk           [...rest] { ^rtk proxy -- awk ...$rest }
export def --wrapped head          [...rest] { ^rtk proxy -- head ...$rest }
export def --wrapped tail          [...rest] { ^rtk proxy -- tail ...$rest }
export def --wrapped readlink      [...rest] { ^rtk proxy -- readlink ...$rest }
export def --wrapped realpath      [...rest] { ^rtk proxy -- realpath ...$rest }
export def --wrapped sha256sum     [...rest] { ^rtk proxy -- sha256sum ...$rest }
export def --wrapped strace        [...rest] { ^rtk proxy -- strace ...$rest }
export def --wrapped sqlite3       [...rest] { ^rtk proxy -- sqlite3 ...$rest }
export def --wrapped pgrep         [...rest] { ^rtk proxy -- pgrep ...$rest }
export def --wrapped ps            [...rest] { ^rtk proxy -- ps ...$rest }
export def --wrapped bats          [...rest] { ^rtk proxy -- bats ...$rest }
export def --wrapped actionlint    [...rest] { ^rtk proxy -- actionlint ...$rest }
export def --wrapped shellcheck    [...rest] { ^rtk proxy -- shellcheck ...$rest }
export def --wrapped nix           [...rest] { ^rtk proxy -- nix ...$rest }
export def --wrapped nix-env       [...rest] { ^rtk proxy -- nix-env ...$rest }
export def --wrapped nix-store     [...rest] { ^rtk proxy -- nix-store ...$rest }
export def --wrapped home-manager  [...rest] { ^rtk proxy -- home-manager ...$rest }
export def --wrapped yzx           [...rest] { ^rtk proxy -- yzx ...$rest }
export def --wrapped envctl        [...rest] { ^rtk proxy -- envctl ...$rest }
export def --wrapped git-kb        [...rest] { ^rtk proxy -- git-kb ...$rest }
export def --wrapped zellij        [...rest] { ^rtk proxy -- zellij ...$rest }
export def --wrapped rtk-monitor   [...rest] { ^rtk proxy -- rtk-monitor ...$rest }
export def --wrapped n8n-up        [...rest] { ^rtk proxy -- n8n-up ...$rest }
export def --wrapped n8n-down      [...rest] { ^rtk proxy -- n8n-down ...$rest }
