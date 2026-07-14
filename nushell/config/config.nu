# FlexNetOS Nushell layer consumed by the packaged Yazelix Nova runtime.
# This source remains editable in the Yazelix repository; Nix substitutes the
# owned store paths before the generated runtime config sources it.

use @rtkWrappers@ *
source @stackPromptGuard@
source @flexnetosInit@

# The installed FlexNetOS product has one Nushell owner. Refuse to publish a
# different shell path when running under the real product home.
if (($env.HOME? | default "") == "/home/flexnetos") {
    let profile_nu = "@profileNu@"
    if not ($profile_nu | path exists) {
        error make { msg: $"profile-owned Nushell is missing: ($profile_nu)" }
    }
    $env.SHELL = $profile_nu

    let volatile_root = (($env.XDG_RUNTIME_DIR? | default "/run/user/1001") | path join "yazelix" "volatile")
    let volatile_cache = ($volatile_root | path join "cache")
    let volatile_tmp = ($volatile_root | path join "tmp")
    let cargo_home = ($volatile_root | path join "cargo-home")
    let cargo_target = ($volatile_root | path join "cargo-target")
    let rustup_home = ($volatile_root | path join "rustup-home")
    for path in [$volatile_cache $volatile_tmp $cargo_home $cargo_target $rustup_home] {
        mkdir $path
    }

    $env.XDG_CACHE_HOME = $volatile_cache
    $env.NIX_CACHE_HOME = ($volatile_cache | path join "nix")
    $env.TMPDIR = $volatile_tmp
    $env.TMP = $volatile_tmp
    $env.TEMP = $volatile_tmp
    $env.CARGO_HOME = $cargo_home
    $env.CARGO_TARGET_DIR = $cargo_target
    $env.RUSTUP_HOME = $rustup_home
    $env.npm_config_cache = ($volatile_cache | path join "npm")
    $env.BUN_INSTALL_CACHE_DIR = ($volatile_cache | path join "bun")
    $env.YARN_CACHE_FOLDER = ($volatile_cache | path join "yarn")
    $env.COREPACK_HOME = ($volatile_cache | path join "corepack")
    $env.UV_CACHE_DIR = ($volatile_cache | path join "uv")
    $env.PIP_CACHE_DIR = ($volatile_cache | path join "pip")
    $env.PIP_NO_CACHE_DIR = "1"
    $env.GOCACHE = ($volatile_cache | path join "go-build")
    $env.GOMODCACHE = ($volatile_cache | path join "go-mod")
    $env.GRADLE_USER_HOME = ($volatile_cache | path join "gradle")
    $env.DENO_DIR = ($volatile_cache | path join "deno")
    $env.HF_HOME = ($volatile_cache | path join "huggingface")
    $env.TORCH_HOME = ($volatile_cache | path join "torch")
    $env.CUDA_CACHE_PATH = ($volatile_cache | path join "cuda")
    $env.PLAYWRIGHT_BROWSERS_PATH = ($volatile_cache | path join "playwright")
    $env.KACHE_CACHE_DIR = "/home/flexnetos/.cache/kache"
    $env.RUSTC_WRAPPER = "/home/flexnetos/.nix-profile/bin/kache-rustc-wrapper"
    $env.CARGO_BUILD_RUSTC_WRAPPER = "/home/flexnetos/.nix-profile/bin/kache-rustc-wrapper"
    $env.NIX_SENTRY_ENDPOINT = ""
    $env.DETSYS_IDS_TELEMETRY = "disabled"
}
