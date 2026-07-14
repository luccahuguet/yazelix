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
}
