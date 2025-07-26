# Yazelix Enhancement Ideas

This document contains potential feature additions and improvements for Yazelix based on analysis of the current codebase and development workflow needs.

## Project/Workspace Management

**Concept**: A project switcher that saves context per project - recently opened files, custom layouts, and environment variables.

**Implementation Ideas**:
- `yzx switch myproject` - restores exact state including:
  - File manager location and selected files
  - Open editor buffers and cursor positions
  - Custom environment variables per project
  - Last used shell and working directory
- Project configuration stored in `~/.config/yazelix/projects/`
- Integration with git repositories for automatic project detection
- Quick switch via fuzzy finder: `yzx projects` → interactive selection

**Benefits**: Eliminates context switching friction, similar to VS Code workspaces but for terminal environments.

## Session Templates

**Concept**: Pre-configured Zellij layouts for different development scenarios.

**Template Examples**:
- `yzx new web-dev` - creates panes for:
  - Editor (main focus)
  - File manager sidebar
  - Development server terminal
  - Logs/output pane
- `yzx new debugging` - layout optimized for debugging:
  - Editor with debugger integration
  - Variable inspection pane
  - Debug console
  - Log viewer
- `yzx new documentation` - writing-focused layout:
  - Markdown editor
  - Live preview pane (if using mdbook/similar)
  - Reference documentation browser

**Implementation**: Extend existing KDL layout system with parameterized templates.

## Integrated Task Runner

**Concept**: Built-in task runner with visual status indicators in the sidebar.

**Features**:
- Define tasks in `.yazelix.toml` project file:
  ```toml
  [tasks]
  dev = "npm run dev"
  test = "npm test"
  build = "npm run build"
  ```
- Visual status in Yazi sidebar:
  - 🟢 Running
  - 🔴 Failed  
  - ✅ Success
  - ⏸️ Stopped
- One-click access to task logs
- Background task execution with notifications
- Integration with file watching for auto-restart

**Benefits**: Eliminates need for separate terminal panes just for running common tasks.

## Enhanced Search Interface

**Concept**: Unified search across multiple contexts simultaneously.

**Search Targets**:
- File contents (current ripgrep functionality)
- Git commit history and messages
- Shell command history (atuin integration)
- Currently open editor buffers
- Project documentation and README files
- Configuration files

**Interface**:
- `Ctrl+Shift+F` for global search
- Categorized results with context indicators
- Jump-to-definition for code symbols
- Search result preview with syntax highlighting

**Implementation**: Could leverage existing tools (ripgrep, git, atuin) with unified interface.

## Minor Enhancements

### Smart File Opening
- Remember preferred editor per file type
- Open images in image viewer, PDFs in document viewer
- Context-aware opening (README files in preview mode)

### Enhanced Git Integration
- Branch switcher in sidebar
- Visual diff indicators for modified files
- Commit message templates based on file changes

### Notification System
- Desktop notifications for long-running tasks
- Build status notifications
- Git push/pull status updates

### Performance Monitoring
- Resource usage display in status bar
- Slow operation warnings
- Memory usage per pane/process

## Implementation Priority

1. **Project/Workspace Management** - Highest impact for development workflow
2. **Session Templates** - Natural extension of existing layout system
3. **Enhanced Search Interface** - Builds on existing search capabilities
4. **Integrated Task Runner** - More complex but high value
5. **Minor Enhancements** - Lower complexity, good for iterative improvement