use assert_cmd::Command;
use std::path::PathBuf;

use super::fixtures::ManagedConfigFixture;

pub fn yzx_core_command() -> Command {
    Command::cargo_bin("yzx_core").unwrap()
}

pub fn yzx_control_command() -> Command {
    Command::cargo_bin("yzx_control").unwrap()
}

pub fn yzx_control_bin_path() -> PathBuf {
    assert_cmd::cargo::cargo_bin("yzx_control")
}

pub fn yzx_root_command() -> Command {
    Command::cargo_bin("yzx").unwrap()
}

pub fn apply_managed_config_env<'a>(
    command: &'a mut Command,
    fixture: &ManagedConfigFixture,
) -> &'a mut Command {
    command
        .env_clear()
        .env("HOME", &fixture.home_dir)
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("XDG_CONFIG_HOME", fixture.xdg_config_home())
        .env("XDG_DATA_HOME", fixture.xdg_data_home())
        .env("YAZELIX_RUNTIME_DIR", &fixture.runtime_dir)
        .env("YAZELIX_CONFIG_DIR", &fixture.config_dir)
        .env("YAZELIX_STATE_DIR", &fixture.state_dir);
    command
}

pub fn yzx_core_command_in_fixture(
    fixture: &ManagedConfigFixture,
    helper_command: &str,
) -> Command {
    let mut command = yzx_core_command();
    command.arg(helper_command);
    apply_managed_config_env(&mut command, fixture);
    command
}
