use std::{
    ffi::{OsStr, OsString},
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

use crate::{
    DESKTOP_DATABASE_UPDATER, DESKTOP_ENTRY_SOURCE,
    error::{AppError, path_error, startup},
    paths::nonempty_env,
};

const DESKTOP_ENTRY_NAME: &str = "com.flexnetos.Yazelix.Agent.desktop";
const DESKTOP_CACHE_DELAY: Duration = Duration::from_secs(6);
const HIDDEN_DESKTOP_ENTRY: &[u8] =
    b"[Desktop Entry]\nHidden=true\nName=FlexNetOS Yazelix Agent\nType=Application\nVersion=1.5\n";

pub(crate) fn run(args: Vec<OsString>) -> Result<(), AppError> {
    let Some((command, args)) = args.split_first() else {
        return Err(desktop_usage());
    };
    let print_path = match args {
        [] => false,
        [flag] if flag == "--print-path" || flag == "-p" => true,
        _ => return Err(desktop_usage()),
    };

    require_desktop_support()?;
    let applications_dir = applications_dir(
        nonempty_env("XDG_DATA_HOME").as_deref(),
        nonempty_env("HOME").as_deref(),
    )?;
    let desktop_path = applications_dir.join(DESKTOP_ENTRY_NAME);

    match command.to_string_lossy().as_ref() {
        "install" => install_at(
            Path::new(DESKTOP_ENTRY_SOURCE),
            Path::new(DESKTOP_DATABASE_UPDATER),
            &applications_dir,
            DESKTOP_CACHE_DELAY,
        )?,
        "uninstall" => uninstall_at(Path::new(DESKTOP_DATABASE_UPDATER), &applications_dir)?,
        _ => return Err(desktop_usage()),
    }

    if print_path {
        println!("{}", desktop_path.display());
    } else {
        println!(
            "{} Yazelix desktop entry: {}",
            if command == "install" {
                "Installed"
            } else {
                "Removed"
            },
            desktop_path.display()
        );
    }
    Ok(())
}

fn desktop_usage() -> AppError {
    AppError::Usage("Usage: yzx desktop <install|uninstall> [--print-path]\n".to_string())
}

fn require_desktop_support() -> Result<(), AppError> {
    if DESKTOP_ENTRY_SOURCE.is_empty() || DESKTOP_DATABASE_UPDATER.is_empty() {
        Err(AppError::Usage(
            "yzx desktop is unavailable in this package\n".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn applications_dir(
    xdg_data_home: Option<&OsStr>,
    home: Option<&OsStr>,
) -> Result<PathBuf, AppError> {
    xdg_data_home
        .map(PathBuf::from)
        .or_else(|| home.map(|path| PathBuf::from(path).join(".local/share")))
        .map(|path| path.join("applications"))
        .ok_or_else(|| {
            startup(
                "HOME is required when XDG_DATA_HOME is unset.",
                "yzx desktop install",
                1,
            )
        })
}

fn install_at(
    source: &Path,
    updater: &Path,
    applications_dir: &Path,
    cache_delay: Duration,
) -> Result<(), AppError> {
    let source_entry = fs::read(source)
        .map_err(|error| path_error("read desktop entry source", source, source, error))?;
    fs::create_dir_all(applications_dir).map_err(|error| {
        path_error(
            "create desktop applications directory",
            applications_dir,
            applications_dir,
            error,
        )
    })?;
    let desktop_path = applications_dir.join(DESKTOP_ENTRY_NAME);

    if !desktop_path.exists() {
        write_entry(&desktop_path, HIDDEN_DESKTOP_ENTRY)?;
        if let Err(error) = refresh_desktop_database(updater, applications_dir) {
            rollback_first_install(&desktop_path)?;
            return Err(error);
        }
        thread::sleep(cache_delay);
        if let Err(error) = write_entry(&desktop_path, &source_entry) {
            rollback_first_install(&desktop_path)?;
            return Err(error);
        }
    } else {
        write_entry(&desktop_path, &source_entry)?;
    }

    refresh_desktop_database(updater, applications_dir)
}

fn uninstall_at(updater: &Path, applications_dir: &Path) -> Result<(), AppError> {
    let desktop_path = applications_dir.join(DESKTOP_ENTRY_NAME);
    match fs::remove_file(&desktop_path) {
        Ok(()) => refresh_desktop_database(updater, applications_dir)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(path_error(
                "remove desktop entry",
                &desktop_path,
                &desktop_path,
                error,
            ));
        }
    }
    Ok(())
}

fn write_entry(path: &Path, contents: &[u8]) -> Result<(), AppError> {
    let temporary = temporary_path(path);
    if let Err(error) = fs::remove_file(&temporary)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        return Err(path_error(
            "remove stale desktop entry temporary file",
            &temporary,
            path,
            error,
        ));
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o644)
        .open(&temporary)
        .map_err(|error| path_error("create desktop entry", &temporary, path, error))?;
    file.write_all(contents)
        .and_then(|()| file.sync_all())
        .map_err(|error| path_error("write desktop entry", &temporary, path, error))?;
    fs::rename(&temporary, path)
        .map_err(|error| path_error("install desktop entry", path, path, error))
}

fn rollback_first_install(path: &Path) -> Result<(), AppError> {
    for candidate in [temporary_path(path), path.to_path_buf()] {
        if let Err(error) = fs::remove_file(&candidate)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            return Err(path_error(
                "roll back failed desktop entry install",
                &candidate,
                path,
                error,
            ));
        }
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    path.with_file_name(format!(".{DESKTOP_ENTRY_NAME}.tmp"))
}

fn refresh_desktop_database(updater: &Path, applications_dir: &Path) -> Result<(), AppError> {
    let status = Command::new(updater)
        .arg(applications_dir)
        .status()
        .map_err(|error| {
            startup(
                format!(
                    "failed to refresh desktop database with {}: {error}",
                    updater.display()
                ),
                applications_dir.display(),
                1,
            )
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(startup(
            format!("desktop database refresh exited with status {status}"),
            applications_dir.display(),
            status.code().unwrap_or(1),
        ))
    }
}

// Test lane: default
#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        os::unix::fs::symlink,
        process,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn fixture_root() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must follow the Unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("yzx_desktop_{}_{}", process::id(), nonce))
    }

    // Defends: desktop install and uninstall use one user-local entry sourced from the package.
    #[test]
    fn install_and_uninstall_manage_one_built_entry() {
        let root = fixture_root();
        let source = root.join("source.desktop");
        let updater = env::current_exe().expect("test executable");
        let applications = root.join("data/applications");
        let entry = b"[Desktop Entry]\nType=Application\nName=Yazelix\n";
        fs::create_dir_all(&root).expect("fixture root");
        fs::write(&source, entry).expect("desktop source");

        assert!(install_at(&source, &updater, &applications, Duration::ZERO).is_ok());
        let installed = applications.join(DESKTOP_ENTRY_NAME);
        assert_eq!(
            fs::read(&installed).expect("installed desktop entry"),
            entry
        );
        assert!(
            !applications
                .join(format!(".{DESKTOP_ENTRY_NAME}.tmp"))
                .exists()
        );

        assert!(uninstall_at(&updater, &applications).is_ok());
        assert!(!installed.exists());
        fs::remove_dir_all(root).expect("fixture cleanup");
    }

    // Regression: a failed first database refresh must not leave a same-ID hidden tombstone.
    #[test]
    fn failed_initial_refresh_rolls_back_hidden_entry() {
        let root = fixture_root();
        let source = root.join("source.desktop");
        let missing_updater = root.join("missing-update-desktop-database");
        let applications = root.join("data/applications");
        fs::create_dir_all(&root).expect("fixture root");
        fs::write(
            &source,
            b"[Desktop Entry]\nType=Application\nName=Yazelix\n",
        )
        .expect("desktop source");

        assert!(install_at(&source, &missing_updater, &applications, Duration::ZERO,).is_err());
        assert!(!applications.join(DESKTOP_ENTRY_NAME).exists());
        assert!(!temporary_path(&applications.join(DESKTOP_ENTRY_NAME)).exists());
        fs::remove_dir_all(root).expect("fixture cleanup");
    }

    // Regression: uninstall must remove a managed launcher symlink after its target disappears.
    #[test]
    fn uninstall_removes_dangling_launcher_symlink() {
        let root = fixture_root();
        let updater = env::current_exe().expect("test executable");
        let applications = root.join("data/applications");
        let installed = applications.join(DESKTOP_ENTRY_NAME);
        fs::create_dir_all(&applications).expect("applications fixture");
        symlink(root.join("garbage_collected.desktop"), &installed)
            .expect("dangling desktop symlink");

        assert!(uninstall_at(&updater, &applications).is_ok());
        assert!(matches!(
            fs::symlink_metadata(&installed),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound
        ));
        fs::remove_dir_all(root).expect("fixture cleanup");
    }
}
