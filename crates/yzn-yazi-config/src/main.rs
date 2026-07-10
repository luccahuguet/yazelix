use std::{
    env, fs,
    io::{self, ErrorKind},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{self, ExitCode},
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
            eprintln!("yzn-yazi-config: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<PathBuf> {
    let args = env::args_os().collect::<Vec<_>>();
    let [_, packaged, user, state] = args.as_slice() else {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "usage: yzn-yazi-config <packaged-yazi> <user-yazi> <state-dir>",
        ));
    };
    materialize(Path::new(packaged), Path::new(user), Path::new(state))
}

fn materialize(packaged: &Path, user: &Path, state: &Path) -> io::Result<PathBuf> {
    if !managed_input_exists(user)? {
        return Ok(packaged.to_path_buf());
    }

    fs::create_dir_all(state)?;
    let runtime = state.join("yazi");
    let stage = state.join(format!(".yazi-{}-{}", process::id(), nonce()));
    fs::create_dir(&stage)?;

    if let Err(error) = write_runtime(packaged, user, &stage) {
        let _ = remove_any(&stage);
        return Err(error);
    }
    remove_any(&runtime)?;
    fs::rename(stage, &runtime)?;
    Ok(runtime)
}

fn managed_input_exists(user: &Path) -> io::Result<bool> {
    for name in ["init.lua", "keymap.toml", "yazi.toml"] {
        if user.join(name).try_exists()? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn write_runtime(packaged: &Path, user: &Path, runtime: &Path) -> io::Result<()> {
    let user_init = user.join("init.lua");
    let user_yazi_toml = user.join("yazi.toml");
    if user_yazi_toml.exists() {
        write_merged_yazi_toml(
            &packaged.join("yazi.toml"),
            &user_yazi_toml,
            &runtime.join("yazi.toml"),
        )?;
    } else {
        symlink(packaged.join("yazi.toml"), runtime.join("yazi.toml"))?;
    }
    symlink(
        packaged.join("yazelix_starship.toml"),
        runtime.join("yazelix_starship.toml"),
    )?;
    write_layered_config(
        &packaged.join("init.lua"),
        &user_init,
        &runtime.join("init.lua"),
        "-- Yazelix Next user init.lua",
    )?;
    write_layered_config(
        &packaged.join("keymap.toml"),
        &user.join("keymap.toml"),
        &runtime.join("keymap.toml"),
        "# Yazelix Next user keymap.toml",
    )?;

    let runtime_plugins = runtime.join("plugins");
    fs::create_dir(&runtime_plugins)?;
    for entry in fs::read_dir(packaged.join("plugins"))? {
        let entry = entry?;
        symlink(entry.path(), runtime_plugins.join(entry.file_name()))?;
    }
    if user_init.exists() {
        overlay_user_plugins(&user.join("plugins"), &runtime_plugins)?;
    }
    Ok(())
}

fn write_merged_yazi_toml(packaged: &Path, user: &Path, target: &Path) -> io::Result<()> {
    let mut merged = parse_toml(packaged, "packaged")?;
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

    merge_value(&mut merged, parse_toml(user, "user")?);
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
    toml::from_str(&fs::read_to_string(path)?).map_err(|error| {
        invalid_data(format!(
            "invalid {owner} Yazi TOML {}: {error}",
            path.display()
        ))
    })
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
    if user.exists() {
        if !user.is_file() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("cannot read {}", user.display()),
            ));
        }
        contents.push('\n');
        contents.push_str(marker);
        contents.push('\n');
        contents.push_str(&fs::read_to_string(user)?);
    }
    fs::write(target, contents)
}

fn overlay_user_plugins(user_plugins: &Path, runtime_plugins: &Path) -> io::Result<()> {
    if !user_plugins.exists() {
        return Ok(());
    }
    if !user_plugins.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("cannot read plugin directory {}", user_plugins.display()),
        ));
    }
    for entry in fs::read_dir(user_plugins)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if !name.to_string_lossy().ends_with(".yazi") {
            continue;
        }
        if !path.is_dir() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("user plugin must be a directory: {}", path.display()),
            ));
        }
        let target = runtime_plugins.join(&name);
        if target.exists() {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "user plugin `{}` collides with a packaged plugin",
                    name.to_string_lossy()
                ),
            ));
        }
        symlink(path, target)?;
    }
    Ok(())
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

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message.into())
}

fn nonce() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
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
            let path =
                env::temp_dir().join(format!("yzn-yazi-config-{}-{}", process::id(), nonce()));
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
        fs::write(user.join("init.lua"), "user init\n").unwrap();
        fs::write(user.join("keymap.toml"), "prepend_keymap = []\n").unwrap();
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
            "packaged init\n\n-- Yazelix Next user init.lua\nuser init\n"
        );
        assert_eq!(
            fs::read_to_string(runtime.join("keymap.toml")).unwrap(),
            "[manager]\nkeymap = []\n\n# Yazelix Next user keymap.toml\nprepend_keymap = []\n"
        );
        assert!(runtime.join("plugins/example.yazi").is_dir());
        assert!(
            fs::symlink_metadata(runtime.join("yazelix_starship.toml"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn invalid_toml_or_plugin_collision_preserves_the_previous_runtime() {
        let temp = TempDir::new();
        let packaged = temp.0.join("packaged");
        let user = temp.0.join("user");
        let state = temp.0.join("state");
        packaged_yazi(&packaged);
        fs::create_dir_all(state.join("yazi")).unwrap();
        fs::create_dir_all(&user).unwrap();
        fs::write(state.join("yazi/sentinel"), "old runtime").unwrap();
        fs::write(user.join("yazi.toml"), "[mgr\n").unwrap();

        let error = materialize(&packaged, &user, &state)
            .unwrap_err()
            .to_string();
        assert!(error.contains(&user.join("yazi.toml").display().to_string()));

        fs::write(user.join("yazi.toml"), "[mgr]\nshow_hidden = true\n").unwrap();
        fs::write(user.join("init.lua"), "user init\n").unwrap();
        fs::create_dir_all(user.join("plugins/git.yazi")).unwrap();
        let error = materialize(&packaged, &user, &state)
            .unwrap_err()
            .to_string();
        assert!(error.contains("collides with a packaged plugin"));
        assert_eq!(
            fs::read_to_string(state.join("yazi/sentinel")).unwrap(),
            "old runtime"
        );
    }

    #[test]
    fn no_managed_input_keeps_the_packaged_fast_path() {
        let temp = TempDir::new();
        let packaged = temp.0.join("packaged");
        let state = temp.0.join("state");

        assert_eq!(
            materialize(&packaged, &temp.0.join("user"), &state).unwrap(),
            packaged
        );
        assert!(!state.exists());
    }
}
