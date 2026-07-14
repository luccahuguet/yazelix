use std::{
    env,
    ffi::OsString,
    fs,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{self, Command},
};

fn executable_name(path: &OsString) -> Option<&str> {
    Path::new(path).file_name()?.to_str()
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn exec(mut command: Command, label: &str) -> ! {
    let error = command.exec();
    eprintln!("kache-rustc-wrapper: cannot execute {label}: {error}");
    process::exit(127);
}

fn main() {
    let invoked_as = env::args_os()
        .next()
        .unwrap_or_else(|| OsString::from("kache-rustc-wrapper"));
    let current = env::current_exe().unwrap_or_else(|_| PathBuf::from("kache-rustc-wrapper"));
    let shim_mode = executable_name(&invoked_as) == Some("rustc")
        || env::var_os("FLEXNETOS_KACHE_SHIM_MODE").is_some();
    if shim_mode {
        let cargo_auditable = env::var_os("FLEXNETOS_KACHE_CARGO_AUDITABLE")
            .unwrap_or_else(|| OsString::from("cargo-auditable"));
        let mut command = Command::new(cargo_auditable);
        command.arg("rustc").args(env::args_os().skip(1));
        exec(command, "cargo-auditable rustc");
    }

    let kache = env::var_os("KACHE_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("@kache@"));
    if !is_executable(&kache) {
        eprintln!(
            "kache-rustc-wrapper: Kache binary is not executable: {}",
            kache.display()
        );
        process::exit(127);
    }

    let shim = env::var_os("FLEXNETOS_KACHE_RUSTC_SHIM")
        .map(PathBuf::from)
        .unwrap_or_else(|| current.clone());
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let cargo_auditable = args.first().and_then(executable_name) == Some("cargo-auditable");
    let compiler = args.get(1).and_then(executable_name);
    let wrapped_compiler = compiler.is_some_and(|name| {
        name == "rustc" || name == "clippy-driver" || name.starts_with("rustc-")
    });

    let mut command = Command::new(&kache);
    if cargo_auditable && wrapped_compiler {
        command
            .env("FLEXNETOS_KACHE_CARGO_AUDITABLE", &args[0])
            .env("FLEXNETOS_KACHE_SHIM_MODE", "1");
        command.arg(shim).args(args.into_iter().skip(2));
    } else {
        command.args(args);
    }
    exec(command, &kache.display().to_string());
}
