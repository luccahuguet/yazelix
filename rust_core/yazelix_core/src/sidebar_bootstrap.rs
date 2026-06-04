use std::path::{Path, PathBuf};

pub(crate) const SIDEBAR_BOOTSTRAP_CWD_ENV: &str = "YAZELIX_BOOTSTRAP_SIDEBAR_CWD_FILE";

const SIDEBAR_BOOTSTRAP_DIR: &str = "sidebar_bootstrap";

pub(crate) fn sidebar_bootstrap_root(state_dir: &Path) -> PathBuf {
    state_dir.join(SIDEBAR_BOOTSTRAP_DIR)
}

pub(crate) fn sidebar_bootstrap_owner_dir(state_dir: &Path, owner: &str) -> PathBuf {
    sidebar_bootstrap_root(state_dir).join(owner)
}

pub(crate) fn is_sidebar_bootstrap_file(state_dir: &Path, path: &Path) -> bool {
    path.is_file() && path.starts_with(sidebar_bootstrap_root(state_dir))
}
