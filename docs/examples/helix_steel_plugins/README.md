# Helix Steel Plugin Example

This directory contains example user-owned Helix Steel plugins for Yazelix

The files here are documentation examples, not bundled default plugins. To use
one, place it under `~/.config/yazelix/helix/steel_plugins/` and declare a
matching entry in `helix.steel_plugins.extra`

## `hello_yazelix.scm`

Minimal command plugin:

```scheme
(require (only-in "helix/misc.scm" set-status!))

(provide hello-yazelix)

(define (hello-yazelix)
  (set-status! "Hello from a custom Yazelix Steel plugin"))
```

Config entry:

```jsonc
{
  "helix": {
    "steel_plugins": {
      "enabled": ["recentf", "splash", "spacemacs_theme"],
      "extra": [
        {
          "id": "hello_yazelix",
          "source": "hello_yazelix.scm",
          "public_commands": ["hello-yazelix"],
          "command_descriptions": {
            "hello-yazelix": "Show a message from a custom Steel plugin"
          }
        }
      ]
    }
  }
}
```

After regeneration, run `:hello-yazelix` in Helix
