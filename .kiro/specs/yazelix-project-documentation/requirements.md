# Requirements Document

## Introduction

Yazelix is a comprehensive terminal workspace that integrates three core tools: Yazi (file manager), Zellij (terminal multiplexer), and Helix (editor). The project provides a unified, reproducible development environment built with Nix that offers IDE-like functionality in the terminal with seamless tool integration, customizable layouts, and cross-platform compatibility.

## Requirements

### Requirement 1

**User Story:** As a developer, I want a unified terminal workspace that integrates file management, terminal multiplexing, and text editing, so that I can have an IDE-like experience without leaving the terminal.

#### Acceptance Criteria

1. WHEN the user launches Yazelix THEN the system SHALL start Zellij with Yazi as a sidebar and the configured editor ready to use
2. WHEN the user opens a file from Yazi THEN the system SHALL open it in the configured editor in an intelligent way (reusing existing editor instances when possible)
3. WHEN the user presses Alt+y in Helix THEN the system SHALL reveal the current file in the Yazi sidebar
4. WHEN the user presses Alt+y in Yazi THEN the system SHALL focus the Helix editor pane

### Requirement 2

**User Story:** As a developer, I want reproducible environment setup across different machines and operating systems, so that my development environment is consistent and easy to share with team members.

#### Acceptance Criteria

1. WHEN the user installs Yazelix THEN the system SHALL require both Nix and Nushell as prerequisites
2. WHEN the user installs Yazelix using Nix THEN the system SHALL install all other required dependencies with exact versions
3. WHEN the user configures Yazelix on one machine THEN the same configuration SHALL work identically on any other supported machine
4. WHEN the user shares their Yazelix configuration THEN other users SHALL be able to reproduce the exact same environment

### Requirement 3

**User Story:** As a developer, I want flexible configuration options for shells, editors, and terminal emulators, so that I can use my preferred tools while maintaining Yazelix integration benefits.

#### Acceptance Criteria

1. WHEN the user configures a preferred shell THEN the system SHALL use that shell (bash, fish, zsh, or nushell) as the default
2. WHEN the user configures a preferred editor THEN the system SHALL use that editor instead of Helix while maintaining integration features where possible
3. WHEN the user configures a preferred terminal emulator THEN the system SHALL launch in that terminal (WezTerm, Ghostty, Kitty, or Alacritty)
4. WHEN the user modifies yazelix.nix configuration THEN the system SHALL apply changes on next startup

### Requirement 4

**User Story:** As a developer, I want intelligent pane and window management, so that I can efficiently navigate between different tools and maintain focus on my work.

#### Acceptance Criteria

1. WHEN the user opens a file from Yazi THEN the system SHALL find existing Helix instances in the top 3 panes and reuse them if found
2. WHEN no existing Helix instance is found THEN the system SHALL create a new editor pane
3. WHEN the user uses Zellij fullscreen mode THEN the system SHALL hide all other panes to maximize the current pane
4. WHEN the user uses the sidebar_closed swap layout THEN the system SHALL hide only the sidebar while keeping other panes visible
5. WHEN the user switches between panes THEN the system SHALL maintain proper focus and context

### Requirement 5

**User Story:** As a developer, I want customizable layouts for different development scenarios, so that I can optimize my workspace for specific tasks like debugging, documentation, or general development.

#### Acceptance Criteria

1. WHEN the user selects a layout THEN the system SHALL arrange panes according to the layout specification
2. WHEN the user enables sidebar mode THEN the system SHALL provide layouts with persistent Yazi file navigation
3. WHEN the user disables sidebar mode THEN the system SHALL provide clean, full-screen layouts
4. WHEN the user switches layouts THEN the system SHALL preserve existing pane content where possible

### Requirement 6

**User Story:** As a developer, I want session management capabilities, so that I can maintain my workspace state across terminal sessions and system restarts.

#### Acceptance Criteria

1. WHEN persistent sessions are enabled THEN the system SHALL attach to existing sessions instead of creating new ones
2. WHEN the user configures a session name THEN the system SHALL use that name for persistent sessions
3. WHEN the user restarts Yazelix with persistent sessions THEN the system SHALL restore the previous session state
4. WHEN persistent sessions are disabled THEN the system SHALL create fresh sessions each time

### Requirement 7

**User Story:** As a developer, I want comprehensive shell integration, so that I can use Yazelix tools and configurations in my existing shell workflows.

#### Acceptance Criteria

1. WHEN Yazelix is installed THEN the system SHALL provide shell integration scripts for bash, fish, zsh, and nushell
2. WHEN the user runs yzx commands THEN the system SHALL provide access to Yazelix functionality from any shell
3. WHEN the user configures shell integration THEN the system SHALL automatically source the appropriate configuration files
4. WHEN the user checks configuration status THEN the system SHALL report the current state of shell integrations

### Requirement 8

**User Story:** As a developer, I want modular package management, so that I can install only the tools I need while avoiding bloat.

#### Acceptance Criteria

1. WHEN the user enables package packs THEN the system SHALL install related tools as a group (e.g., python pack includes ruff, uv, ty)
2. WHEN the user specifies individual packages THEN the system SHALL install only those specific tools
3. WHEN the user disables optional dependencies THEN the system SHALL exclude them from installation
4. WHEN the user checks package sizes THEN the system SHALL provide clear information about disk space requirements

### Requirement 9

**User Story:** As a developer, I want Home Manager integration, so that I can manage my yazelix.nix configuration declaratively as part of my system configuration.

#### Acceptance Criteria

1. WHEN the user enables the Home Manager module THEN the system SHALL allow Home Manager to control the yazelix.nix configuration file
2. WHEN the user configures Yazelix through Home Manager THEN the system SHALL use those settings instead of the local yazelix.nix file
3. WHEN the user updates their Home Manager configuration THEN Yazelix SHALL reflect the changes on next rebuild
4. WHEN the user uses Home Manager examples THEN they SHALL provide working configurations for common scenarios

### Requirement 10

**User Story:** As a developer, I want automatic shell initialization for productivity tools, so that I have a consistent, enhanced shell experience across all supported shells.

#### Acceptance Criteria

1. WHEN Yazelix starts THEN the system SHALL automatically generate initialization scripts for Starship, Zoxide, Mise, and Carapace
2. WHEN the user configures multiple shells THEN the system SHALL generate appropriate initialization scripts for each shell (bash, fish, zsh, nushell)
3. WHEN productivity tools are available THEN the system SHALL integrate them automatically
4. WHEN productivity tools are missing THEN the system SHALL skip their initialization gracefully

### Requirement 11

**User Story:** As a developer, I want dynamic Zellij configuration management, so that I can have layered configuration with Yazelix defaults, personal overrides, and automatic updates.

#### Acceptance Criteria

1. WHEN Yazelix starts THEN the system SHALL merge three configuration layers: Zellij defaults, Yazelix overrides, and user personal config
2. WHEN configuration files change THEN the system SHALL regenerate the merged configuration automatically
3. WHEN the user creates personal configuration THEN it SHALL take highest priority over Yazelix defaults
4. WHEN Zellij updates THEN the system SHALL fetch new defaults dynamically without manual intervention

### Requirement 12

**User Story:** As a developer, I want comprehensive logging and debugging capabilities, so that I can troubleshoot issues and understand system behavior.

#### Acceptance Criteria

1. WHEN debug mode is enabled THEN the system SHALL log detailed information about all operations
2. WHEN debug mode is disabled THEN the system SHALL still log basic operational information automatically
3. WHEN operations fail THEN the system SHALL log error information to appropriate log files regardless of debug mode
4. WHEN the user requests version information THEN the system SHALL display versions of all integrated tools
5. WHEN the user encounters issues THEN log files SHALL provide sufficient information for troubleshooting
6. WHEN old log files accumulate THEN the system SHALL automatically clean up old logs keeping only the 10 most recent

### Requirement 13

**User Story:** As a developer, I want intelligent configuration parsing and validation, so that my yazelix.nix settings are properly applied and validated against expected schemas.

#### Acceptance Criteria

1. WHEN the user modifies yazelix.nix THEN the system SHALL parse the configuration using simple line-based parsing
2. WHEN configuration values are missing THEN the system SHALL use sensible defaults
3. WHEN the user has both yazelix.nix and yazelix_default.nix THEN the system SHALL prefer the user's yazelix.nix file
4. WHEN configuration is invalid THEN the system SHALL provide clear error messages and fallback to defaults