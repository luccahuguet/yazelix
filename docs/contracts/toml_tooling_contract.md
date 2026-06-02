# TOML Tooling Contract

## Summary

Yazelix ships `tombi.toml` as the managed TOML tooling config and expects the `tombi` command from the host when TOML formatting or linting is run.

## Behavior

- The packaged runtime includes `tombi.toml`
- The packaged runtime does not bundle `tombi` by default
- `tombi` is host-managed by default and can be explicitly bundled through the Nix runtime tool source surface when needed
- The managed config surface copies `tombi.toml` into the Yazelix config root when needed
- `.taplo.toml` is not a shipped runtime support file
- The formatter corpus is limited to Yazelix-owned TOML files configured in `tombi.toml`
- Vendored Yazi flavors and Cargo manifests are not part of the Tombi formatting gate

## Non-Goals

- shipping Taplo or Tombi in the default runtime
- formatting vendored or tool-owned TOML as part of the Yazelix TOML gate
- using formatter churn as incidental cleanup

## Verification

- `tombi format --check`
- `tombi lint --offline`
- `yzx_repo_validator validate-contracts`
- runtime package smoke checks that verify `tombi.toml` and the runtime tool manifest source for `tombi`
