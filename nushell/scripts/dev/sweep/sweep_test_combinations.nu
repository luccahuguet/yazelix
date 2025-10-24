#!/usr/bin/env nu
# Sweep Testing - Test Combinations Generator
# Generates test matrices for different shell/terminal/feature combinations

use ../../utils/constants.nu *

# Test sweep definitions - using supported shells and terminals from constants
const SHELLS = ["nu", "bash", "fish", "zsh"]
const PRIMARY_SHELL = $DEFAULT_SHELL
const PRIMARY_TERMINAL = $DEFAULT_TERMINAL
const TERMINALS = $SUPPORTED_TERMINALS

# Configuration variations to test
const HELIX_MODES = ["release", "source"]
const BOOLEAN_FEATURES = [
    "enable_sidebar",
    "persistent_sessions",
    "recommended_deps",
    "yazi_extensions"
]

# Feature record builders
def make_standard_features [helix_mode: string = "release"]: nothing -> record {
    {
        helix_mode: $helix_mode,
        enable_sidebar: true,
        persistent_sessions: false,
        recommended_deps: true,
        yazi_extensions: true
    }
}

def make_minimal_features []: nothing -> record {
    {
        helix_mode: "release",
        enable_sidebar: false,
        persistent_sessions: false,
        recommended_deps: false,
        yazi_extensions: false
    }
}

def make_maximal_features []: nothing -> record {
    {
        helix_mode: "source",
        enable_sidebar: true,
        persistent_sessions: true,
        recommended_deps: true,
        yazi_extensions: true
    }
}

# Generate test combinations for non-visual mode (environment/shell testing)
export def generate_test_combinations []: nothing -> list<record> {
    mut combinations = []

    # 1. Cross-shell testing (each shell with primary terminal)
    for $shell in $SHELLS {
        $combinations = ($combinations | append {
            type: "cross_shell",
            shell: $shell,
            terminal: $PRIMARY_TERMINAL,
            features: (make_standard_features)
        })
    }

    # 2. Feature variation testing (primary shell/terminal with different features)
    for $helix_mode in $HELIX_MODES {
        $combinations = ($combinations | append {
            type: "feature_variation",
            shell: $PRIMARY_SHELL,
            terminal: $PRIMARY_TERMINAL,
            features: (make_standard_features $helix_mode)
        })
    }

    # 3. Boolean feature combinations (test key features on/off)
    $combinations = ($combinations | append {
        type: "minimal_config",
        shell: $PRIMARY_SHELL,
        terminal: $PRIMARY_TERMINAL,
        features: (make_minimal_features)
    })

    $combinations = ($combinations | append {
        type: "maximal_config",
        shell: $PRIMARY_SHELL,
        terminal: $PRIMARY_TERMINAL,
        features: (make_maximal_features)
    })

    $combinations
}

# Generate test combinations for visual mode (terminal launch testing)
export def generate_visual_test_combinations []: nothing -> list<record> {
    mut combinations = []

    # Cross-terminal testing (primary shell with each terminal)
    # Visual mode is required to actually launch and verify terminals work
    # Use standard features but with sidebar disabled for simpler visual testing
    for $terminal in $TERMINALS {
        let features = (make_standard_features) | update enable_sidebar false
        $combinations = ($combinations | append {
            type: "cross_terminal",
            shell: $PRIMARY_SHELL,
            terminal: $terminal,
            features: $features
        })
    }

    $combinations
}

# Get test constants for external use
export def get_test_constants []: nothing -> record {
    {
        shells: $SHELLS,
        terminals: $TERMINALS,
        primary_shell: $PRIMARY_SHELL,
        primary_terminal: $PRIMARY_TERMINAL,
        helix_modes: $HELIX_MODES,
        boolean_features: $BOOLEAN_FEATURES
    }
}