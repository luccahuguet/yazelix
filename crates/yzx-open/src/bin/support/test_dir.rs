use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

pub(crate) fn write_nu_executable(path: &Path, body: &str) {
    let shebang = env::var_os("YZX_TEST_NU").map_or_else(
        || "#!/usr/bin/env -S nu --no-config-file\n".to_owned(),
        |nu| {
            let nu = PathBuf::from(nu);
            assert!(nu.is_absolute(), "YZX_TEST_NU must be absolute");
            format!("#!{} --no-config-file\n", nu.display())
        },
    );
    write_executable(path, &(shebang + body));
}

pub(crate) struct TestDir {
    pub(crate) path: PathBuf,
}

impl TestDir {
    pub(crate) fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for attempt in 0..100 {
            let path = env::temp_dir().join(format!(
                "yzx-open-bin-{}-{nanos}-{attempt}",
                std::process::id()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Self { path },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!(
                    "could not create test directory {}: {error}",
                    path.display()
                ),
            }
        }
        panic!("could not create unique yzx-open test directory");
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
