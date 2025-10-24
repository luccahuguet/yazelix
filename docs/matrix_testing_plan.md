# Configuration Matrix Testing Plan

## Overview

Comprehensive testing framework that sweeps through shell/terminal combinations to catch real integration issues that unit tests miss. This "configuration sweep" testing validates that Yazelix works reliably across different environments and configurations.

## Goals

- **Catch integration bugs** that only appear with specific shell/terminal combinations
- **Validate configuration parsing** across different scenarios
- **Test environment setup** for each combination
- **Ensure consistent behavior** across all supported platforms
- **Prevent regressions** when adding new features

## Test Matrix Structure

### Core Matrix (Essential Combinations)

#### 1. Cross-Shell Testing
Test each shell with Ghostty (primary terminal):
- `nu` + `ghostty`
- `bash` + `ghostty`
- `fish` + `ghostty`
- `zsh` + `ghostty`

#### 2. Cross-Terminal Testing
Test Nushell (primary shell) with each terminal:
- `nu` + `ghostty`
- `nu` + `wezterm`
- `nu` + `kitty`
- `nu` + `alacritty`
- `nu` + `foot` (Linux only)

### Configuration Dimensions

#### 3. Helix Modes
- `helix_mode = "release"`
- `helix_mode = "source"`

#### 4. Key Features
- `enable_sidebar = true/false`
- `persistent_sessions = true/false`
- `terminal_config_mode = "auto"/"user"/"yazelix"`

#### 5. Edge Cases
- Minimal config (just defaults)
- Heavy config (all extras enabled)
- Mixed transparency/cursor_trail settings

## Validation Criteria

Each test should validate:

### 1. Configuration Parsing
- Does the configuration load without errors?
- Are all expected fields present and valid?
- Do computed values match expectations?

### 2. Environment Setup
- Does `yzx env --no-shell` work without hanging?
- Are expected tools available in PATH?
- Are key environment variables set correctly?

### 3. Launch Capability
- Can we start without crashing?
- Does the launch process complete within timeout?
- Are processes cleaned up properly?

### 4. Integration Points
- Do shell-specific features work?
- Are terminal-specific configurations applied?
- Do environment variables propagate correctly?

## Implementation Strategy

### 1. Temporary Config Generation

```nushell
def generate_test_config [shell: string, terminal: string, features: record] {
    # Create temporary yazelix.nix with specific combination
    # Ensure clean isolation between tests
    # Return path to temporary config
}
```

### 2. Test Execution Framework

```nushell
def run_matrix_test [config_path: string, timeout: duration] {
    # Test with yzx env --no-shell
    # Validate expected tools/env vars are available
    # Timeout protection (30s per test)
    # Clean up temp configs
}
```

### 3. Validation Functions

```nushell
def validate_environment [expected_tools: list, expected_vars: list] {
    # Check that required tools are in PATH
    # Verify environment variables are set
    # Test basic functionality of key tools
}
```

### 4. Reporting System

```nushell
def generate_matrix_report [results: list] {
    # Clear pass/fail reporting for each combination
    # Detailed logs for debugging failures
    # Summary statistics (X/Y combinations passed)
}
```

## Test Scenarios

### Basic Scenarios
1. **Default configuration** - Standard yazelix_default.nix
2. **Minimal configuration** - Only required settings
3. **Complete configuration** - All features enabled

### Shell-Specific Scenarios
1. **Nushell-specific features** - Custom functions, modules
2. **Bash compatibility** - POSIX compliance, aliases
3. **Fish shell quirks** - Unique syntax, behaviors
4. **Zsh extensions** - Oh-my-zsh compatibility

### Terminal-Specific Scenarios
1. **Ghostty features** - Cursor trails, transparency
2. **WezTerm configuration** - Image preview, multiplexing
3. **Kitty protocols** - Graphics, keyboard handling
4. **Alacritty simplicity** - Minimal feature set
5. **Foot Wayland** - Linux-specific behaviors

## Integration with Existing Test Runner

### Add to `yzx test`
- `yzx test --matrix` - Run matrix tests only
- `yzx test --matrix --verbose` - Detailed matrix output
- `yzx test --matrix --filter "nu"` - Test only combinations with nu
- `yzx test --matrix --timeout 60` - Custom timeout per test

### Existing Test Integration
- Matrix tests run after unit tests
- Share logging infrastructure
- Consistent reporting format
- Same timeout and cleanup mechanisms

## Success Metrics

### Coverage Targets
- **All shell combinations**: 4 shells × 1 primary terminal = 4 tests
- **All terminal combinations**: 1 primary shell × 5 terminals = 5 tests
- **Configuration variations**: Key features × 2-3 variants = 6-9 tests
- **Total target**: ~15-20 test combinations

### Performance Targets
- **Individual test**: < 30 seconds
- **Full matrix**: < 10 minutes
- **Parallel execution**: Where safe (different temp dirs)

### Quality Targets
- **Zero false positives**: Tests should not fail due to test infrastructure
- **Clear failure reasons**: When tests fail, reason should be obvious
- **Reproducible**: Same input should always produce same result

## Implementation Timeline

### Phase 1: Core Framework
1. Design test matrix structure
2. Implement temporary config generation
3. Create basic validation functions
4. Add timeout and cleanup mechanisms

### Phase 2: Test Scenarios
1. Implement shell/terminal combinations
2. Add configuration variation tests
3. Create edge case scenarios
4. Validate against known working combinations

### Phase 3: Integration
1. Integrate with existing test runner
2. Add CLI flags and options
3. Implement reporting and logging
4. Test on different platforms

### Phase 4: Validation
1. Run against current system
2. Test with known broken configurations
3. Validate that real issues are caught
4. Performance optimization and parallel execution

## Benefits

### For Developers
- **Catch regressions** before they reach users
- **Validate changes** across multiple environments
- **Test coverage** for complex integration scenarios
- **Debugging aid** for environment-specific issues

### For Users
- **Increased reliability** across different setups
- **Better support** for edge case configurations
- **Confidence** in system stability
- **Faster issue resolution** when problems occur

### For Project Health
- **Quality assurance** for releases
- **Documentation** of supported combinations
- **Baseline** for new feature compatibility
- **Prevention** of environment-specific bugs

## Future Enhancements

### Advanced Testing
- **Performance benchmarks** for each combination
- **Memory usage** validation
- **Startup time** measurements
- **Resource cleanup** verification

### CI/CD Integration
- **Automated testing** on multiple platforms
- **Regression detection** in pull requests
- **Release validation** before publishing
- **Performance trend** tracking

### User Feedback
- **Self-diagnostic mode** for user environments
- **Configuration recommendations** based on detected setup
- **Troubleshooting guides** for failed combinations
- **Community contribution** of test scenarios

This matrix testing framework will significantly improve Yazelix's reliability and catch the kind of real-world integration bugs that matter most to users across different shell/terminal combinations.