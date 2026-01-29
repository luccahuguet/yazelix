# devenv DSL Reference

Comprehensive reference for devenv.nix configuration syntax.

**Source:** https://devenv.sh/
**Last Updated:** 2025-11-02

## Table of Contents

- [Basic Structure](#basic-structure)
- [Core Options](#core-options)
- [Environment Variables](#environment-variables)
- [Packages](#packages)
- [Shell Hooks](#shell-hooks)
- [Scripts](#scripts)
- [Processes](#processes)
- [Services](#services)
- [Languages](#languages)
- [Advanced Features](#advanced-features)
- [Built-in Variables](#built-in-variables)

## Basic Structure

```nix
{ pkgs, lib, config, inputs, ... }:

{
  # Configuration attributes
  packages = [ ];
  env = { };
  enterShell = "";
  # ... other options
}
```

### Available Inputs

- `pkgs` - nixpkgs package set
- `lib` - nixpkgs library functions
- `config` - current devenv configuration
- `inputs` - flake inputs (defined in devenv.yaml)

## Core Options

### packages

**Type:** `listOf package`
**Description:** Add Nix packages to your development environment

```nix
packages = [ pkgs.git pkgs.jq pkgs.curl ];
```

**Important:** Packages are **automatically added to PATH** when you enter the shell. No manual PATH manipulation needed.

**Example:**
```nix
packages = [
  pkgs.git
  pkgs.jq
  pkgs.libffi  # libraries/headers also supported
  pkgs.zlib
];
```

### env

**Type:** `attrsOf str`
**Description:** Define environment variables accessible in the shell

```nix
env.GREET = "hello";
env.DATABASE_URL = "postgres://localhost";
env.DEBUG = "true";
```

**Note:** For PATH, devenv automatically handles packages. You typically don't need to set PATH manually.

### enterShell

**Type:** `str` (bash script)
**Description:** Execute commands automatically when entering the environment

```nix
enterShell = ''
  echo "Welcome to the environment!"
  git --version
  echo "Ready to work"
'';
```

**Best Practice:** Use `tasks` instead of `enterShell` for complex setup operations, as tasks provide better control over execution order, dependencies, and parallel execution.

### enterTest

**Type:** `str` (bash script)
**Description:** Run commands during testing phases

```nix
enterTest = ''
  pytest tests/
  mypy src/
'';
```

## Environment Variables

### env Attribute

Simple key-value pairs:

```nix
env = {
  MY_VAR = "value";
  ANOTHER_VAR = "123";
};
```

Dot notation:

```nix
env.MY_VAR = "value";
env.ANOTHER_VAR = "123";
```

### .env File Integration

Load environment variables from `.env` files:

```nix
dotenv.enable = true;
# Optional: specify filename
dotenv.filename = ".env.production";
```

**Priority:** Variables set in `devenv.nix` take priority over `.env` file variables.

## Packages

### Adding Packages

```nix
packages = [
  pkgs.package-name
  pkgs.another-package
];
```

### Package Lists

You can build lists dynamically:

```nix
let
  essentialDeps = [ pkgs.git pkgs.curl ];
  optionalDeps = [ pkgs.jq pkgs.yq ];
in {
  packages = essentialDeps ++ optionalDeps;
}
```

### Conditional Packages

```nix
let
  includeDev = true;
  devPackages = if includeDev then [ pkgs.gdb ] else [ ];
in {
  packages = [ pkgs.git ] ++ devPackages;
}
```

### PATH Behavior

- **Automatic:** Packages are added to PATH when shell activates
- **No manual PATH manipulation needed**
- **Access:** Just use the command directly: `git`, `jq`, etc.

## Shell Hooks

### enterShell

Runs every time you enter the shell:

```nix
enterShell = ''
  export MY_CUSTOM_VAR="value"
  echo "Environment ready"

  # Can run any bash code
  if [ ! -f ".initialized" ]; then
    echo "First time setup"
    touch .initialized
  fi
'';
```

### Best Practices

1. Keep `enterShell` lightweight
2. Use `tasks` for complex operations
3. Avoid long-running commands
4. Use conditional checks to skip unnecessary work

## Scripts

Define custom shell scripts and commands (see https://devenv.sh/scripts/):

```nix
scripts.hello.exec = ''
  echo "Hello, $1!"
'';
```

**Note:** Scripts get their own package scope and don't pollute the global environment.

## Processes

Configure background processes (see https://devenv.sh/processes/):

```nix
processes = {
  server.exec = "python -m http.server 8000";
  worker.exec = "celery worker";
};
```

Supports process managers:
- Hivemind
- Honcho
- Overmind

## Services

Run infrastructure services locally (see https://devenv.sh/reference/options/#services):

### Database Services

```nix
services.postgres = {
  enable = true;
  initialDatabases = [{ name = "mydb"; }];
};

services.mysql.enable = true;
services.mongodb.enable = true;
```

### Cache Services

```nix
services.redis.enable = true;
services.memcached.enable = true;
```

### Message Brokers

```nix
services.kafka.enable = true;
services.rabbitmq.enable = true;
services.nats.enable = true;
```

### Other Services

- Elasticsearch
- Keycloak
- Mailhog
- Minio
- And 30+ more

## Languages

Enable language-specific tooling (50+ languages supported):

### Python

```nix
languages.python = {
  enable = true;
  version = "3.11";
};
```

### Rust

```nix
languages.rust.enable = true;
```

### JavaScript/TypeScript

```nix
languages.javascript = {
  enable = true;
  package = pkgs.nodejs_20;
};
```

### Go

```nix
languages.go.enable = true;
```

And many more: Ruby, PHP, Elixir, Haskell, Java, Kotlin, etc.

## Advanced Features

### git-hooks

Integrate pre-commit hooks:

```nix
git-hooks.hooks = {
  nixfmt.enable = true;
  shellcheck.enable = true;
};
```

### containers

Build and manage container images:

```nix
containers.myapp = {
  name = "myapp";
  copyToRoot = [ pkgs.bash ];
};
```

### profiles

Create environment variations:

```nix
profiles = {
  production.env.DEBUG = "false";
  development.env.DEBUG = "true";
};
```

### overlays

Override or extend Nix packages:

```nix
overlays = [
  (self: super: {
    mypackage = super.mypackage.override {
      enableFeature = true;
    };
  })
];
```

## Built-in Variables

devenv provides several environment variables automatically:

### $DEVENV_ROOT

Points to the root of the project where `devenv.nix` is located.

```bash
echo $DEVENV_ROOT
# /home/user/myproject
```

### $DEVENV_DOTFILE

Points to `$DEVENV_ROOT/.devenv`.

```bash
echo $DEVENV_DOTFILE
# /home/user/myproject/.devenv
```

### $DEVENV_STATE

Points to `$DEVENV_DOTFILE/state`.

Used for storing stateful data.

### $DEVENV_RUNTIME

Points to a temporary directory with a path unique to each `$DEVENV_ROOT`.

Used for storing sockets and other runtime files.

### $DEVENV_PROFILE

**Most Important:** Points to the Nix store path that has the final profile of packages/scripts provided by devenv.

Useful for teaching other programs about `/bin`, `/etc`, `/var` folders.

```bash
echo $DEVENV_PROFILE
# /nix/store/xxxxx-devenv-profile

ls $DEVENV_PROFILE/bin
# Shows all binaries from your packages
```

**Usage:** This is how PATH gets automatically populated. All packages added to `packages = [ ];` get their binaries symlinked into `$DEVENV_PROFILE/bin`, which is added to PATH.

## Common Patterns

### Dynamic Package Lists

```nix
let
  userConfig = import ./config.nix { inherit pkgs; };

  essentialDeps = [ pkgs.git pkgs.curl ];

  optionalDeps = if userConfig.includeDev
    then [ pkgs.gdb pkgs.valgrind ]
    else [ ];

  allPackages = essentialDeps ++ optionalDeps;
in {
  packages = allPackages;
}
```

### Conditional Configuration

```nix
let
  isLinux = pkgs.stdenv.isLinux;
  isDarwin = pkgs.stdenv.isDarwin;
in {
  packages = [ pkgs.git ]
    ++ lib.optionals isLinux [ pkgs.linux-specific-tool ]
    ++ lib.optionals isDarwin [ pkgs.darwin-specific-tool ];
}
```

### Environment Variable Cascading

```nix
{
  env = {
    # Base configuration
    LOG_LEVEL = "info";

    # Can be overridden by .env file when dotenv.enable = true
  };

  dotenv.enable = true;

  enterShell = ''
    # Can modify further in shell hook
    export LOG_LEVEL="''${LOG_LEVEL:-debug}"
  '';
}
```

## Important Notes for Yazelix

### PATH Management

**DO NOT manually manipulate PATH in `env` or `enterShell`.**

✅ Correct:
```nix
{
  packages = [ pkgs.uv pkgs.ruff pkgs.python3 ];
  # PATH is automatically set up
}
```

❌ Incorrect:
```nix
{
  packages = [ pkgs.uv ];
  env.PATH = "${pkgs.lib.makeBinPath [ pkgs.uv ]}:$PATH";  # Redundant!
}
```

### Shell Environment Inheritance

When using devenv with Zellij or other terminal multiplexers:

1. Environment variables set in `env` are available
2. `$DEVENV_PROFILE/bin` is in PATH
3. All packages are accessible
4. `enterShell` runs when the shell starts

**Key Issue:** If child shells (Zellij panes) don't inherit the environment properly, they won't have access to packages.

**Solution:** Ensure Zellij spawns shells within the devenv context, or capture and preserve `$PATH` for child shells.

### Configuration Changes

devenv uses evaluation caching. When you change `devenv.nix`:

1. File hashes are tracked
2. Cache automatically invalidates
3. Next shell entry re-evaluates
4. New packages become available

**No manual cache clearing needed.**

## References

- Official Documentation: https://devenv.sh/
- Options Reference: https://devenv.sh/reference/options/
- Getting Started: https://devenv.sh/getting-started/
- Basics: https://devenv.sh/basics/
- Files & Variables: https://devenv.sh/files-and-variables/
- Packages: https://devenv.sh/packages/
