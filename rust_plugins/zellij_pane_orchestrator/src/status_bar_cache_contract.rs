// Test lane: default
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBarCacheRuntime {
    pub yzx_control_path: String,
    pub cache_path: String,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
}

pub fn resolve_status_bar_cache_runtime(
    session_env: &BTreeMap<String, String>,
) -> Option<StatusBarCacheRuntime> {
    let cache_path = session_env
        .get("YAZELIX_STATUS_BAR_CACHE_PATH")
        .filter(|path| !path.trim().is_empty())
        .cloned()
        .or_else(|| {
            session_env
                .get("YAZELIX_SESSION_CONFIG_PATH")
                .filter(|path| !path.trim().is_empty())
                .and_then(|path| {
                    PathBuf::from(path)
                        .parent()
                        .map(|parent| parent.join("status_bar_cache.json").display().to_string())
                })
        })?;

    let yzx_control_path = session_env
        .get("YAZELIX_YZX_CONTROL_BIN")
        .filter(|path| !path.trim().is_empty())
        .cloned()
        .or_else(|| {
            session_env
                .get("YAZELIX_RUNTIME_DIR")
                .filter(|path| !path.trim().is_empty())
                .map(|runtime_dir| {
                    PathBuf::from(runtime_dir)
                        .join("libexec")
                        .join("yzx_control")
                        .display()
                        .to_string()
                })
        })?;

    let cwd = session_env
        .get("YAZELIX_RUNTIME_DIR")
        .or_else(|| session_env.get("PWD"))
        .filter(|path| !path.trim().is_empty())
        .map(|path| PathBuf::from(path.as_str()))
        .unwrap_or_else(|| PathBuf::from("/"));

    let mut env = BTreeMap::new();
    for key in [
        "HOME",
        "PATH",
        "XDG_CACHE_HOME",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "YAZELIX_RUNTIME_DIR",
        "YAZELIX_SESSION_CONFIG_PATH",
        "YAZELIX_STATUS_BAR_CACHE_PATH",
    ] {
        if let Some(value) = session_env.get(key).filter(|value| !value.is_empty()) {
            env.insert(key.to_string(), value.clone());
        }
    }
    env.insert(
        "YAZELIX_STATUS_BAR_CACHE_PATH".to_string(),
        cache_path.clone(),
    );

    Some(StatusBarCacheRuntime {
        yzx_control_path,
        cache_path,
        cwd,
        env,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Defends: status-bar cache paths are launch-scoped so concurrent Yazelix windows do not share mutable bar facts.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn derives_cache_path_from_session_snapshot_when_explicit_cache_path_is_absent() {
        let mut env = BTreeMap::new();
        env.insert(
            "YAZELIX_SESSION_CONFIG_PATH".to_string(),
            "/tmp/yazelix/sessions/window_a/config_snapshot.json".to_string(),
        );
        env.insert(
            "YAZELIX_RUNTIME_DIR".to_string(),
            "/nix/store/yazelix".to_string(),
        );

        let runtime = resolve_status_bar_cache_runtime(&env).unwrap();

        assert_eq!(
            runtime.cache_path,
            "/tmp/yazelix/sessions/window_a/status_bar_cache.json"
        );
        assert_eq!(
            runtime.yzx_control_path,
            "/nix/store/yazelix/libexec/yzx_control"
        );
    }

    // Defends: an explicit cache path wins, which lets startup pin each window to its own cache file.
    // Strength: defect=2 behavior=2 resilience=2 cost=1 uniqueness=2 total=9/10
    #[test]
    fn explicit_cache_path_wins_over_derived_snapshot_sibling() {
        let mut env = BTreeMap::new();
        env.insert(
            "YAZELIX_STATUS_BAR_CACHE_PATH".to_string(),
            "/tmp/yazelix/sessions/window_b/status.json".to_string(),
        );
        env.insert(
            "YAZELIX_SESSION_CONFIG_PATH".to_string(),
            "/tmp/yazelix/sessions/window_a/config_snapshot.json".to_string(),
        );
        env.insert(
            "YAZELIX_YZX_CONTROL_BIN".to_string(),
            "/opt/bin/yzx_control".to_string(),
        );

        let runtime = resolve_status_bar_cache_runtime(&env).unwrap();

        assert_eq!(
            runtime.cache_path,
            "/tmp/yazelix/sessions/window_b/status.json"
        );
        assert_eq!(runtime.yzx_control_path, "/opt/bin/yzx_control");
    }
}
