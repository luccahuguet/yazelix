# Nushell Scripts Organization

This directory contains all Nushell scripts for the Yazelix project, organized by functionality.

## Directory Structure

### `core/` - Core Yazelix Functionality
Essential scripts that provide the main Yazelix functionality:
- `start_yazelix.nu` - Main launcher that starts Zellij with Yazelix layout
- `launch_yazelix.nu` - Terminal launcher that opens your preferred terminal emulator
- `yzx_session.nu` - Restart/session entrypoints that still own shell-heavy session behavior

### `integrations/` - Tool Integration Scripts
Scripts that handle integration between Yazi, Zellij, and Helix:
- `yazi.nu` - Yazi integration utilities (file opening, reveal in sidebar)
- `zellij.nu` - Zellij integration utilities (pane management, Helix detection)
- `helix.nu` - Helix integration utilities (binary detection, testing)
- `open_file.nu` - Wrapper script called by Yazi to open files in Helix
- `reveal_in_yazi.nu` - Wrapper script for revealing files in Yazi sidebar

### `setup/` - Setup and Configuration Scripts
Scripts for initial setup and configuration:
- `environment.nu` - Main environment setup script
- `initializers.nu` - Initializer script generation

### `utils/` - Utility Functions
Reusable utility functions and helpers:
- `constants.nu` - Project constants and configuration
- `version_info.nu` - Version information utilities
- `helix_mode.nu` - Helix binary resolution helpers
- `common.nu` - Common utility functions
- `logging.nu` - Logging utilities

### `dev/` - Development Tools
Scripts for development, testing, and maintenance:
- `validate_syntax.nu` - Validate syntax of all Nushell scripts using nu-check

### Manual Maintainer Helpers
These are manual or exploratory helpers, not normal runtime entrypoints:
- `record_demo.nu` - VHS demo recording helper
- `record_demo_fonts.nu` - Font-testing helper for demo recording

Canonical maintainer entrypoints:
- `yzx dev build_pane_orchestrator --sync` - Build and sync the pane orchestrator wasm
- `yzx dev update` - Refresh runtime pins, vendored zjstatus, vendored Yazi plugins, and update canaries

## Usage

### Core Commands
```bash
# Start Yazelix (source-checkout / maintainer path)
nu ~/.config/yazelix/nushell/scripts/core/start_yazelix.nu

# Launch terminal (source-checkout / maintainer path)
nu ~/.config/yazelix/nushell/scripts/core/launch_yazelix.nu

# Use the stable CLI wrapper (source-checkout / maintainer path)
~/.config/yazelix/shells/posix/yzx_cli.sh help
```

For normal installed usage, prefer `yzx launch`, `yzx run`, and the shipped runtime entrypoints rather than calling repo paths directly.

### Development Tools
```bash
# Validate script syntax
yzx dev test --lint-only # Run syntax validation only
yzx dev test             # Run the default non-sweep regression suite (includes syntax validation)
yzx dev test --sweep     # Run only the config/shell sweep
yzx dev test --visual    # Run only the visual terminal sweep

# Record demos (maintainer path)
nu ~/.config/yazelix/nushell/scripts/dev/record_demo.nu quick

# Test fonts (maintainer path)
nu ~/.config/yazelix/nushell/scripts/dev/record_demo_fonts.nu
```

## File Naming Convention
All files use underscores (e.g., `start_yazelix.nu`, `open_file.nu`) - never hyphens
