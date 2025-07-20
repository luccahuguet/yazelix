#!/usr/bin/env nu
# Yazelix Configuration Schema
# Defines the expected structure for yazelix.nix configuration

# Define the expected config schema
export def get_config_schema [] {
    {
        include_optional_deps: {
            type: "boolean"
            default: true
            description: "Include optional tools like lazygit, mise, etc."
        }
        include_yazi_extensions: {
            type: "boolean"
            default: true
            description: "Include Yazi extensions for previews, archives, etc."
        }
        include_yazi_media: {
            type: "boolean"
            default: true
            description: "Include heavy media packages for Yazi (WARNING: ~800MB-1.2GB)"
        }
        helix_mode: {
            type: "string"
            default: "release"
            valid_values: ["release", "source", "default"]
            description: "Helix build mode: 'release' (nixpkgs), 'source' (flake), or 'default' (stable)"
        }
        default_shell: {
            type: "string"
            default: "nu"
            valid_values: ["nu", "bash", "fish", "zsh"]
            description: "Default shell for Zellij"
        }
        extra_shells: {
            type: "array"
            default: []
            description: "Extra shells to install beyond nu/bash"
        }
        preferred_terminal: {
            type: "string"
            default: "wezterm"
            valid_values: ["wezterm", "ghostty"]
            description: "Preferred terminal emulator"
        }
        editor_config: {
            type: "object"
            default: {
                set_editor: true
                override_existing: true
                editor_command: "hx"
            }
            description: "Editor configuration"
            fields: {
                set_editor: {
                    type: "boolean"
                    default: true
                    description: "Whether to set EDITOR environment variable"
                }
                override_existing: {
                    type: "boolean"
                    default: true
                    description: "Whether to override existing EDITOR if already set"
                }
                editor_command: {
                    type: "string"
                    default: "hx"
                    description: "Custom editor command"
                }
            }
        }
        debug_mode: {
            type: "boolean"
            default: false
            description: "Enable verbose debug logging"
        }
        skip_welcome_screen: {
            type: "boolean"
            default: false
            description: "Skip the welcome screen on startup"
        }
        ascii_art_mode: {
            type: "string"
            default: "animated"
            valid_values: ["static", "animated"]
            description: "ASCII art display mode: 'static' or 'animated'"
        }
        show_macchina_on_welcome: {
            type: "boolean"
            default: false
            description: "Show macchina system info on the welcome screen if enabled (uses macchina, always available in Yazelix)"
        }
        user_packages: {
            type: "array"
            default: []
            description: "User packages - add your custom Nix packages here"
        }
    }
}