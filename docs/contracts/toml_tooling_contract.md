# TOML Tooling Contract

## Summary

Yazelix ships Tombi as the runtime TOML tool and ships `tombi.toml` as the managed TOML tooling config.

## Behavior

- The packaged runtime includes `tombi` and exposes it through `toolbin`
- The packaged runtime includes `tombi.toml`
- The managed config surface copies `tombi.toml` into the Yazelix config root when needed
- `.taplo.toml` is not a shipped runtime support file
- The formatter corpus is limited to Yazelix-owned TOML files configured in `tombi.toml`
- Vendored Yazi flavors and Cargo manifests are not part of the Tombi formatting gate

## Non-Goals

- shipping both Taplo and Tombi in the default runtime
- formatting vendored or tool-owned TOML as part of the Yazelix TOML gate
- using formatter churn as incidental cleanup

## Verification

- `tombi format --check`
- `tombi lint --offline`
- `yzx_repo_validator validate-contracts`
- runtime package smoke checks that verify `toolbin/tombi` and `tombi.toml`
