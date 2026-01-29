# Home Manager Integration Plan

> **Note:** The Home Manager module now outputs `yazelix.toml`; some tasks below still reference the older `yazelix.nix` flow and should be modernized.

**Status**: Implementation Phase - Option A Selected  
**Priority**: Medium  
**Risk Level**: Low (configuration-only approach minimizes risks)

## 🎯 Goals

- [ ] Provide declarative Home Manager integration for Yazelix
- [ ] Maintain 100% compatibility with existing manual installation
- [ ] Ensure zero risk to development workflow
- [ ] Create clean, maintainable module architecture

## 🎯 **DECISION: Option A Selected** 

**Configuration-Only Module** has been chosen as the implementation approach based on:
- Minimal risk and maximum safety
- Preserves existing user workflows  
- Easy implementation and maintenance
- No architectural changes needed to core Yazelix

## 📋 Implementation Strategy

### ✅ Phase 1: Research & Design (Completed)
- [x] **Research existing patterns** - Study how other terminal tools integrate with Home Manager
- [x] **Design module interface** - Define clean API that doesn't manage files directly  
- [x] **Document safety rules** - Clear guidelines for what the module should/shouldn't do
- [x] **Evaluate options** - Selected configuration-only approach

### 🔄 Phase 2: Module Development (In Progress)
- [ ] **Build Home Manager module** - Configuration-only implementation
- [ ] **Create example configurations** - Basic and advanced examples
- [ ] **Test with existing installations** - Verify compatibility

### 📋 Phase 3: Implementation Details

#### ✅ **Option A: Configuration-Only Module (SELECTED)**
```nix
programs.yazelix = {
  enable = true;
  # Only generates yazelix.toml config file
  # User still runs: nix develop ~/.config/yazelix
};
```
- [ ] Module only creates/manages `yazelix.toml` configuration
- [ ] User manually clones Yazelix repo to `~/.config/yazelix`
- [ ] Zero file conflicts, minimal risk

#### Option B: Package-Based Integration  
```nix
programs.yazelix = {
  enable = true;
  # Installs yazelix as a proper Nix package
  # Creates configs in ~/.config/yazelix-hm/
};
```
- [ ] Package Yazelix as installable Nix package
- [ ] Use separate config directory (`~/.config/yazelix-hm/`)
- [ ] Provide migration tools between manual and HM installations

#### Option C: Overlay Integration
```nix
nixpkgs.overlays = [ yazelix.overlays.default ];
# Adds yazelix package to pkgs.yazelix
```
- [ ] Create Nix overlay for Yazelix package
- [ ] Users add to their system/home configuration
- [ ] Most flexible, least prescriptive approach

### Phase 4: Documentation & Examples
- [ ] **User migration guide** - Safe transition from manual to HM
- [ ] **Multiple examples** - Different use cases and configurations  
- [ ] **Troubleshooting guide** - Common issues and solutions
- [ ] **Compatibility matrix** - Which versions work together

## 🚨 Safety Requirements

### Absolute Rules (NEVER Violate)
- [ ] **Never manage files in active git repositories**
- [ ] **Never use `path:` inputs for development repos**
- [ ] **Never overwrite user files without explicit consent**
- [ ] **Always provide rollback/uninstall capability**

### Testing Requirements  
- [ ] Test in clean NixOS VM
- [ ] Test with existing Yazelix manual installations
- [ ] Test upgrade/downgrade scenarios  
- [ ] Test with different Home Manager versions
- [ ] Test rollback scenarios

### User Safety
- [ ] Clear warnings about file management
- [ ] Backup recommendations in documentation
- [ ] Migration path documentation
- [ ] Uninstall instructions

## 🗂️ File Structure Plan

```
yazelix-home-manager/           # Separate repository
├── flake.nix                   # HM module flake
├── modules/
│   └── yazelix.toml           # Home Manager module
├── examples/
│   ├── basic.nix              # Simple configuration
│   ├── advanced.nix           # Full features
│   └── migration.nix          # Migrate from manual
├── docs/
│   ├── installation.md        # Installation guide
│   ├── configuration.md       # All options documented
│   ├── migration.md          # Manual -> HM migration
│   └── troubleshooting.md    # Common issues
└── tests/
    ├── vm-tests/              # NixOS VM tests
    └── integration-tests/     # Real-world scenarios
```

## 🔄 Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Create separate `yazelix-home-manager` repository
- [ ] Set up testing infrastructure (VM, CI)
- [ ] Research and document best practices
- [ ] Create basic module skeleton

### Phase 2: Core Module (Week 3-4)  
- [ ] Implement configuration-only approach (safest)
- [ ] Basic options: `recommended_deps`, `yazi_extensions`, etc.
- [ ] Generate `yazelix.toml` from Home Manager options
- [ ] Test with manual Yazelix installations

### Phase 3: Enhanced Features (Week 5-6)
- [ ] Add environment variable management
- [ ] Shell integration (aliases, etc.)
- [ ] Terminal emulator configurations
- [ ] Service management (persistent sessions)

### Phase 4: Documentation & Polish (Week 7-8)
- [ ] Complete documentation suite
- [ ] Migration tools and guides  
- [ ] Example configurations
- [ ] Release preparation

## 🧪 Testing Strategy

### Test Environments
- [ ] **Clean NixOS VM** - Fresh install testing
- [ ] **Existing Yazelix user** - Migration testing
- [ ] **Multiple HM versions** - Compatibility testing
- [ ] **Different shells** - bash, fish, zsh, nushell

### Test Scenarios  
- [ ] Fresh Home Manager installation
- [ ] Migration from manual Yazelix installation
- [ ] Upgrade/downgrade Yazelix versions
- [ ] Module disable/enable cycles
- [ ] Conflict resolution (existing configs)

### Automated Testing
- [ ] NixOS VM tests in CI
- [ ] Configuration validation tests
- [ ] Integration tests with real terminal emulators
- [ ] Performance impact testing

## 🎯 Success Criteria

- [ ] **Zero data loss** - Never lose user configurations
- [ ] **Easy migration** - Simple path from manual to HM
- [ ] **Full compatibility** - All Yazelix features available  
- [ ] **Clean uninstall** - Complete removal possible
- [ ] **Good documentation** - Clear guides and examples
- [ ] **Community adoption** - Positive user feedback

## 🚧 Risk Mitigation

### High-Risk Areas
- [ ] **File management** - Use separate directories or config-only approach
- [ ] **Version conflicts** - Clear compatibility documentation
- [ ] **Migration issues** - Thorough testing and rollback plans
- [ ] **Maintenance burden** - Keep module simple and focused

### Backup Plans
- [ ] **Config-only fallback** - If full integration proves problematic
- [ ] **Documentation alternative** - Comprehensive manual setup guide
- [ ] **Community package** - Let community maintain if needed

## 📝 Notes

- **Priority**: This is a nice-to-have feature, not essential
- **Risk tolerance**: Very low - must not impact existing users
- **Timeline**: No rush - better to do it right than fast
- **Community involvement**: Get feedback before major decisions

## ✅ Next Steps (Current Implementation)

1. ✅ **Research phase** - Study existing Home Manager modules
2. ✅ **Architecture decision** - Configuration-only approach selected
3. 🔄 **Build module** - Implement based on home_manager_module_design.md
4. 📋 **Create examples** - Basic and advanced configuration examples
5. 📋 **Test & document** - Verify compatibility and create user guides

---

**Last Updated**: 2025-01-26  
**Next Review**: When ready to begin implementation  
**Responsible**: TBD (likely community-driven)
