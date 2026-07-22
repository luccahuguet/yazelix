use std::{
    env, fs,
    io::{self, ErrorKind},
    os::unix::fs::symlink,
    path::{Path, PathBuf, absolute},
    process::{self, ExitCode},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use toml::Value;

fn main() -> ExitCode {
    match run() {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("yzx-yazi-config: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<PathBuf> {
    let args = env::args_os().collect::<Vec<_>>();
    let [_, packaged, user, state] = args.as_slice() else {
        return Err(invalid_input(
            "usage: yzx-yazi-config <packaged-yazi> <user-yazi> <state-dir>",
        ));
    };
    materialize(Path::new(packaged), Path::new(user), Path::new(state))
}

fn materialize(packaged: &Path, user: &Path, state: &Path) -> io::Result<PathBuf> {
    let packaged = absolute(packaged)?;
    let user = absolute(user)?;
    let state = absolute(state)?;
    if !managed_input_exists(&user)? {
        return Ok(packaged);
    }

    fs::create_dir_all(&state)?;
    let state = fs::canonicalize(state)?;
    let runtime = state.join("yazi");
    reject_runtime_source(&user, &runtime)?;
    let stage = state.join(format!(".yazi-{}-{}", process::id(), nonce()));
    fs::create_dir(&stage)?;

    let result = write_runtime(&packaged, &user, &stage, &runtime)
        .and_then(|()| remove_any(&runtime))
        .and_then(|()| fs::rename(&stage, &runtime));
    if let Err(error) = result {
        let _ = remove_any(&stage);
        return Err(error);
    }
    Ok(runtime)
}

fn managed_input_exists(user: &Path) -> io::Result<bool> {
    if path_entry_exists(user)? && !user.is_dir() {
        return Err(invalid_input(format!(
            "cannot read Yazi config directory {}",
            user.display()
        )));
    }
    for name in [
        "init.lua",
        "keymap.toml",
        "yazi.toml",
        "starship.toml",
        "package.toml",
        "theme.toml",
        "plugins",
        "flavors",
    ] {
        if path_entry_exists(&user.join(name))? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn write_runtime(packaged: &Path, user: &Path, stage: &Path, runtime: &Path) -> io::Result<()> {
    let user_init = user.join("init.lua");
    let user_yazi_toml = user.join("yazi.toml");
    if path_entry_exists(&user_yazi_toml)? {
        write_merged_yazi_toml(
            &packaged.join("yazi.toml"),
            &user_yazi_toml,
            &stage.join("yazi.toml"),
        )?;
    } else {
        symlink(packaged.join("yazi.toml"), stage.join("yazi.toml"))?;
    }
    let user_starship = user.join("starship.toml");
    let runtime_starship = stage.join("yazelix_starship.toml");
    if path_entry_exists(&user_starship)? {
        if !user_starship.is_file() {
            return Err(invalid_input(format!(
                "cannot read {}",
                user_starship.display()
            )));
        }
        parse_toml(&user_starship, "managed Yazi Starship")?;
        symlink_user_source(&user_starship, &runtime_starship, runtime)?;
    } else {
        symlink(packaged.join("yazelix_starship.toml"), runtime_starship)?;
    }
    write_layered_config(
        &packaged.join("init.lua"),
        &user_init,
        &stage.join("init.lua"),
        "-- Yazelix Nova user init.lua",
    )?;
    write_layered_config(
        &packaged.join("keymap.toml"),
        &user.join("keymap.toml"),
        &stage.join("keymap.toml"),
        "# Yazelix Nova user keymap.toml",
    )?;
    for name in ["package.toml", "theme.toml"] {
        let source = user.join(name);
        if !path_entry_exists(&source)? {
            continue;
        }
        if !source.is_file() {
            return Err(invalid_input(format!("cannot read {}", source.display())));
        }
        symlink_user_source(&source, &stage.join(name), runtime)?;
    }
    for (directory, kind) in [("plugins", "plugin"), ("flavors", "flavor")] {
        overlay_user_assets(
            &packaged.join(directory),
            &user.join(directory),
            &stage.join(directory),
            runtime,
            kind,
        )?;
    }
    Ok(())
}

fn write_merged_yazi_toml(packaged: &Path, user: &Path, target: &Path) -> io::Result<()> {
    let mut merged = parse_toml(packaged, "packaged Yazi")?;
    let packaged_edit = merged
        .get("opener")
        .and_then(|value| value.get("edit"))
        .cloned()
        .ok_or_else(|| invalid_data("packaged Yazi TOML is missing opener.edit"))?;
    let required_fetchers = merged
        .get("plugin")
        .and_then(|value| value.get("prepend_fetchers"))
        .and_then(Value::as_array)
        .map(|fetchers| {
            fetchers
                .iter()
                .filter(|fetcher| is_managed_git_fetcher(fetcher))
                .cloned()
                .collect::<Vec<_>>()
        })
        .filter(|fetchers| fetchers.len() == 2)
        .ok_or_else(|| invalid_data("packaged Yazi TOML must contain two managed Git fetchers"))?;

    merge_value(&mut merged, parse_toml(user, "user Yazi")?);
    let root = merged
        .as_table_mut()
        .ok_or_else(|| invalid_data("Yazi TOML root must be a table"))?;
    let opener = table_entry(root, "opener")?;
    opener.insert("edit".into(), packaged_edit);
    let plugin = table_entry(root, "plugin")?;
    let fetchers = plugin
        .entry("prepend_fetchers")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| invalid_data("plugin.prepend_fetchers must be an array"))?;
    fetchers.retain(|fetcher| !required_fetchers.contains(fetcher));
    fetchers.extend(required_fetchers);

    let text = toml::to_string_pretty(&merged)
        .map_err(|error| invalid_data(format!("could not render merged Yazi TOML: {error}")))?;
    fs::write(target, text)
}

fn parse_toml(path: &Path, owner: &str) -> io::Result<Value> {
    toml::from_str(&fs::read_to_string(path)?)
        .map_err(|error| invalid_data(format!("invalid {owner} TOML {}: {error}", path.display())))
}

fn merge_value(base: &mut Value, overlay: Value) {
    match (base, overlay) {
        (Value::Table(base), Value::Table(overlay)) => {
            for (key, value) in overlay {
                match base.get_mut(&key) {
                    Some(current) => merge_value(current, value),
                    None => {
                        base.insert(key, value);
                    }
                }
            }
        }
        (base, overlay) => *base = overlay,
    }
}

fn table_entry<'a>(table: &'a mut toml::Table, key: &str) -> io::Result<&'a mut toml::Table> {
    table
        .entry(key)
        .or_insert_with(|| Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or_else(|| invalid_data(format!("{key} must be a table")))
}

fn is_managed_git_fetcher(value: &Value) -> bool {
    value.get("run").and_then(Value::as_str) == Some("git")
        && value.get("group").and_then(Value::as_str) == Some("git")
}

fn write_layered_config(
    packaged: &Path,
    user: &Path,
    target: &Path,
    marker: &str,
) -> io::Result<()> {
    let mut contents = fs::read_to_string(packaged)?;
    if path_entry_exists(user)? {
        if !user.is_file() {
            return Err(invalid_input(format!("cannot read {}", user.display())));
        }
        contents.push('\n');
        contents.push_str(marker);
        contents.push('\n');
        contents.push_str(&fs::read_to_string(user)?);
    }
    fs::write(target, contents)
}

fn overlay_user_assets(
    packaged: &Path,
    user: &Path,
    target: &Path,
    runtime: &Path,
    kind: &str,
) -> io::Result<()> {
    fs::create_dir(target)?;
    if packaged.is_dir() {
        for entry in fs::read_dir(packaged)? {
            let entry = entry?;
            symlink(entry.path(), target.join(entry.file_name()))?;
        }
    }
    if !path_entry_exists(user)? {
        return Ok(());
    }
    if !user.is_dir() {
        return Err(invalid_input(format!(
            "cannot read {kind} directory {}",
            user.display()
        )));
    }
    for entry in fs::read_dir(user)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let display_name = name.to_string_lossy();
        if display_name == ".yazi" || !display_name.ends_with(".yazi") {
            continue;
        }
        if !path.is_dir() {
            return Err(invalid_input(format!(
                "user {kind} must be a directory: {}",
                path.display()
            )));
        }
        if kind == "flavor" && !path.join("flavor.toml").is_file() {
            return Err(invalid_input(format!(
                "user flavor must contain flavor.toml: {}",
                path.display()
            )));
        }
        let destination = target.join(&name);
        if path_entry_exists(&destination)? {
            if kind == "plugin" {
                if display_name != "starship.yazi" {
                    return Err(io::Error::new(
                        ErrorKind::AlreadyExists,
                        format!(
                            "user plugin `{display_name}` cannot replace a protected Yazelix plugin"
                        ),
                    ));
                }
                if !path.join("main.lua").is_file() {
                    return Err(invalid_input(format!(
                        "user plugin `{display_name}` must contain main.lua"
                    )));
                }
            }
            remove_any(&destination)?;
        }
        symlink_user_source(&path, &destination, runtime)?;
    }
    Ok(())
}

fn reject_runtime_source(source: &Path, runtime: &Path) -> io::Result<()> {
    if source.starts_with(runtime) || fs::canonicalize(source)?.starts_with(runtime) {
        return Err(invalid_input(format!(
            "managed Yazi source {} must be outside generated runtime {}",
            source.display(),
            runtime.display()
        )));
    }
    Ok(())
}

fn symlink_user_source(source: &Path, target: &Path, runtime: &Path) -> io::Result<()> {
    reject_runtime_source(source, runtime)?;
    symlink(source, target)
}

fn remove_any(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            fs::remove_dir_all(path)
        }
        Ok(_) => fs::remove_file(path),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn path_entry_exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message.into())
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, message.into())
}

static NONCE: AtomicU64 = AtomicU64::new(0);

fn nonce() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{timestamp}-{}", NONCE.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    const PACKAGED_TOML: &str = r#"
[mgr]
ratio = [1, 2, 3]
show_hidden = false

[preview]
max_width = 600

[[plugin.prepend_fetchers]]
url = "*"
run = "git"
group = "git"

[[plugin.prepend_fetchers]]
url = "*/"
run = "git"
group = "git"

[opener]
edit = [{ run = "managed-open %s", block = false, for = "unix" }]
"#;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = PathBuf::from("target").join(format!("yzx-{}-{}", process::id(), nonce()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn packaged_yazi(path: &Path) {
        fs::create_dir_all(path.join("plugins/git.yazi")).unwrap();
        fs::create_dir_all(path.join("plugins/starship.yazi")).unwrap();
        fs::create_dir_all(path.join("flavors/packaged.yazi")).unwrap();
        fs::write(
            path.join("plugins/starship.yazi/main.lua"),
            "packaged starship\n",
        )
        .unwrap();
        fs::write(
            path.join("plugins/starship.yazi/packaged-only.lua"),
            "packaged helper\n",
        )
        .unwrap();
        fs::write(path.join("init.lua"), "packaged init\n").unwrap();
        fs::write(path.join("keymap.toml"), "[manager]\nkeymap = []\n").unwrap();
        fs::write(path.join("yazi.toml"), PACKAGED_TOML).unwrap();
        fs::write(
            path.join("yazelix_starship.toml"),
            "format = '$directory'\n",
        )
        .unwrap();
    }

    #[test]
    fn merges_native_toml_and_preserves_managed_integrations_and_sidecars() {
        let temp = TempDir::new();
        let packaged = temp.0.join("packaged");
        let user = temp.0.join("user");
        packaged_yazi(&packaged);
        fs::create_dir_all(user.join("plugins/example.yazi")).unwrap();
        fs::create_dir_all(user.join("plugins/starship.yazi")).unwrap();
        fs::write(
            user.join("plugins/starship.yazi/main.lua"),
            "user starship\n",
        )
        .unwrap();
        fs::create_dir_all(user.join("flavors/packaged.yazi")).unwrap();
        fs::write(
            user.join("flavors/packaged.yazi/flavor.toml"),
            "[mgr]\ncwd = { fg = \"blue\" }\n",
        )
        .unwrap();
        fs::write(user.join("init.lua"), "user init\n").unwrap();
        fs::write(user.join("keymap.toml"), "prepend_keymap = []\n").unwrap();
        fs::write(user.join("package.toml"), "[plugin]\ndeps = []\n").unwrap();
        fs::write(
            user.join("theme.toml"),
            "[flavor]\ndark = \"packaged\"\nlight = \"packaged\"\n",
        )
        .unwrap();
        fs::write(
            user.join("yazi.toml"),
            r#"
[mgr]
ratio = [1, 4, 0]
show_hidden = true

[preview]
max_width = 1200

[[plugin.prepend_fetchers]]
url = "*.zip"
run = "archive"

[[plugin.prepend_fetchers]]
url = "*"
run = "git"
group = "git"

[[plugin.prepend_fetchers]]
url = "*"
run = "git"
group = "git"

[[plugin.prepend_previewers]]
url = "*.md"
run = "markdown"

[opener]
edit = [{ run = "nvim %s" }]
"#,
        )
        .unwrap();

        let runtime = materialize(&packaged, &user, &temp.0.join("state")).unwrap();
        let merged =
            toml::from_str::<Value>(&fs::read_to_string(runtime.join("yazi.toml")).unwrap())
                .unwrap();
        assert_eq!(
            merged["mgr"]["ratio"],
            Value::Array(vec![1.into(), 4.into(), 0.into()])
        );
        assert_eq!(merged["mgr"]["show_hidden"].as_bool(), Some(true));
        assert_eq!(merged["preview"]["max_width"].as_integer(), Some(1200));
        assert_eq!(
            merged["opener"]["edit"][0]["run"].as_str(),
            Some("managed-open %s")
        );
        let fetchers = merged["plugin"]["prepend_fetchers"].as_array().unwrap();
        assert_eq!(fetchers.len(), 3);
        assert_eq!(
            fetchers
                .iter()
                .filter(|fetcher| is_managed_git_fetcher(fetcher))
                .count(),
            2
        );
        assert!(
            fetchers
                .iter()
                .any(|fetcher| fetcher["run"].as_str() == Some("archive"))
        );
        assert_eq!(
            merged["plugin"]["prepend_previewers"][0]["run"].as_str(),
            Some("markdown")
        );
        assert_eq!(
            fs::read_to_string(runtime.join("init.lua")).unwrap(),
            "packaged init\n\n-- Yazelix Nova user init.lua\nuser init\n"
        );
        assert_eq!(
            fs::read_to_string(runtime.join("keymap.toml")).unwrap(),
            "[manager]\nkeymap = []\n\n# Yazelix Nova user keymap.toml\nprepend_keymap = []\n"
        );
        assert!(runtime.join("plugins/example.yazi").is_dir());
        assert_eq!(
            fs::read_to_string(runtime.join("plugins/starship.yazi/main.lua")).unwrap(),
            "user starship\n"
        );
        assert!(
            !runtime
                .join("plugins/starship.yazi/packaged-only.lua")
                .exists()
        );
        assert!(
            fs::read_to_string(runtime.join("flavors/packaged.yazi/flavor.toml"))
                .unwrap()
                .contains("blue")
        );
        for name in ["package.toml", "theme.toml"] {
            assert_eq!(
                fs::read_to_string(runtime.join(name)).unwrap(),
                fs::read_to_string(user.join(name)).unwrap()
            );
        }
        assert!(
            fs::symlink_metadata(runtime.join("yazelix_starship.toml"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn invalid_inputs_preserve_runtime_and_clean_staging() {
        let temp = TempDir::new();
        let packaged = temp.0.join("packaged");
        let user = temp.0.join("user");
        let state = temp.0.join("state");
        packaged_yazi(&packaged);
        fs::create_dir_all(state.join("yazi")).unwrap();
        fs::write(state.join("yazi/sentinel"), "old runtime").unwrap();
        let fail = |user: &Path, state: &Path| {
            materialize(&packaged, user, state).unwrap_err().to_string()
        };

        symlink(temp.0.join("missing-user"), &user).unwrap();
        let error = fail(&user, &state);
        assert!(error.contains(&user.display().to_string()), "{error}");
        fs::remove_file(&user).unwrap();
        fs::create_dir_all(&user).unwrap();

        let theme = user.join("theme.toml");
        symlink(temp.0.join("missing-theme.toml"), &theme).unwrap();
        let error = fail(&user, &state);
        assert!(error.contains(&theme.display().to_string()), "{error}");
        fs::remove_file(theme).unwrap();

        fs::write(user.join("yazi.toml"), "[mgr\n").unwrap();

        let error = fail(&user, &state);
        assert!(error.contains(&user.join("yazi.toml").display().to_string()));

        fs::write(user.join("yazi.toml"), "[mgr]\nshow_hidden = true\n").unwrap();
        fs::create_dir_all(user.join("plugins/git.yazi")).unwrap();
        let error = fail(&user, &state);
        assert!(
            error.contains("user plugin `git.yazi` cannot replace a protected Yazelix plugin"),
            "{error}"
        );
        assert_eq!(
            fs::read_to_string(state.join("yazi/sentinel")).unwrap(),
            "old runtime"
        );

        fs::remove_dir_all(user.join("plugins")).unwrap();
        fs::create_dir_all(user.join("plugins/starship.yazi")).unwrap();
        let error = fail(&user, &state);
        assert!(error.contains("main.lua"), "{error}");

        fs::remove_dir_all(user.join("plugins")).unwrap();
        fs::create_dir_all(user.join("flavors/packaged.yazi")).unwrap();
        let error = fail(&user, &state);
        assert!(error.contains("flavor.toml"), "{error}");
        fs::remove_dir_all(user.join("flavors")).unwrap();

        let theme = user.join("theme.toml");
        symlink(
            fs::canonicalize(state.join("yazi/sentinel")).unwrap(),
            &theme,
        )
        .unwrap();
        let error = fail(&user, &state);
        assert!(error.contains("outside generated runtime"), "{error}");
        assert!(state.join("yazi/sentinel").is_file());
        fs::remove_file(theme).unwrap();

        let starship = user.join("starship.toml");
        fs::write(&starship, "[directory\n").unwrap();
        let error = fail(&user, &state);
        assert!(error.contains(&starship.display().to_string()), "{error}");
        assert_eq!(
            fs::read_to_string(state.join("yazi/sentinel")).unwrap(),
            "old runtime"
        );
        fs::remove_file(starship).unwrap();

        let overlap_state = temp.0.join("overlap");
        let overlap_user = overlap_state.join("yazi");
        fs::create_dir_all(&overlap_user).unwrap();
        fs::write(overlap_user.join("yazi.toml"), "[mgr]").unwrap();
        let error = fail(&overlap_user, &overlap_state);
        assert!(error.contains("outside generated runtime"), "{error}");
        assert!(overlap_user.join("yazi.toml").is_file());

        let runtime = state.join("yazi");
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o000)).unwrap();
        let result = materialize(&packaged, &user, &state);
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
        assert_eq!(fs::read_dir(&state).unwrap().count(), 1);
    }

    #[test]
    fn starship_config_alone_is_a_complete_replacement() {
        let temp = TempDir::new();
        let packaged = temp.0.join("packaged");
        let user = temp.0.join("user");
        let state = temp.0.join("state");
        packaged_yazi(&packaged);
        fs::create_dir_all(&user).unwrap();
        fs::write(
            user.join("starship.toml"),
            "format = '$directory$git_branch'\n",
        )
        .unwrap();

        let runtime = materialize(&packaged, &user, &state).unwrap();
        let starship = runtime.join("yazelix_starship.toml");
        assert_eq!(
            fs::read_to_string(&starship).unwrap(),
            "format = '$directory$git_branch'\n"
        );
        assert!(
            fs::symlink_metadata(starship)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn relative_asset_directories_activate_materialization_and_remove_stale_links() {
        let temp = TempDir::new();
        let packaged = temp.0.join("packaged");
        let user = temp.0.join("user");
        let state = temp.0.join("state");
        packaged_yazi(&packaged);

        assert_eq!(
            materialize(&packaged, &user, &state).unwrap(),
            std::path::absolute(&packaged).unwrap()
        );
        assert!(!state.exists());

        fs::create_dir_all(user.join("plugins/example.yazi")).unwrap();
        let runtime = materialize(&packaged, &user, &state).unwrap();
        assert!(runtime.join("plugins/example.yazi").is_dir());

        fs::remove_dir_all(user.join("plugins")).unwrap();
        fs::create_dir_all(user.join("flavors/example.yazi")).unwrap();
        fs::write(user.join("flavors/example.yazi/flavor.toml"), "").unwrap();
        fs::create_dir_all(user.join("flavors/.yazi")).unwrap();
        let runtime = materialize(&packaged, &user, &state).unwrap();
        assert!(!runtime.join("plugins/example.yazi").exists());
        assert!(runtime.join("flavors/example.yazi").is_dir());
        assert!(!runtime.join("flavors/.yazi").exists());
    }
}
