// Test lane: default

mod support;

#[cfg(unix)]
mod unix {
    use serde_json::{Value, json};
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixListener;
    use std::path::Path;
    use std::thread;

    use super::support::commands::{apply_managed_config_env, yzx_control_command};
    use super::support::fixtures::{managed_config_fixture, write_session_config_snapshot_with_id};

    fn assert_success(output: &std::process::Output) {
        assert_eq!(output.status.code(), Some(0));
        assert!(output.stderr.is_empty());
    }

    fn write_bridge_registry(
        state_dir: &Path,
        session_id: &str,
        instance_id: &str,
        socket_path: &Path,
        token_path: &Path,
    ) {
        let registry_dir = state_dir.join("helix_bridge").join(session_id);
        fs::create_dir_all(&registry_dir).unwrap();
        fs::write(token_path, "secret").unwrap();
        fs::write(
            registry_dir.join(format!("{instance_id}.json")),
            serde_json::to_string_pretty(&json!({
                "schema_version": 1,
                "session_id": session_id,
                "instance_id": instance_id,
                "socket_path": socket_path.to_string_lossy(),
                "auth_token_path": token_path.to_string_lossy(),
                "pid": std::process::id(),
                "zellij_session_name": null,
                "zellij_tab_position": null,
                "zellij_pane_id": "terminal:7",
                "started_at_unix_ms": 1,
                "managed_config_path": "/tmp/yazelix/helix/config.toml"
            }))
            .unwrap(),
        )
        .unwrap();
    }

    fn spawn_mock_bridge(socket_path: &Path) -> thread::JoinHandle<()> {
        let listener = UnixListener::bind(socket_path).unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut line = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            reader.read_line(&mut line).unwrap();
            let request: Value = serde_json::from_str(&line).unwrap();
            assert_eq!(request["schema_version"], 1);
            assert_eq!(request["auth_token"], "secret");
            assert_eq!(request["action"], "helix.get_context");
            let response = json!({
                "schema_version": 1,
                "request_id": request["request_id"],
                "status": "ok",
                "data": {
                    "cwd": "/tmp/project",
                    "current_file": null,
                    "selection_count": 1,
                    "mode": "normal"
                }
            });
            writeln!(stream, "{response}").unwrap();
        })
    }

    // Defends: the internal control-plane route can discover a managed Helix registry and exchange one typed bridge request without a live TUI.
    #[test]
    fn yzx_control_helix_action_round_trips_to_bridge_socket() {
        let fixture = managed_config_fixture("");
        let session_id = "launch-bridge-test";
        let instance_id = "inst-1";
        let snapshot = write_session_config_snapshot_with_id(&fixture, session_id, &[]);
        let registry_dir = fixture.state_dir.join("helix_bridge").join(session_id);
        fs::create_dir_all(&registry_dir).unwrap();
        let socket_path = registry_dir.join("inst-1.sock");
        let token_path = registry_dir.join("inst-1.token");
        let bridge = spawn_mock_bridge(&socket_path);
        write_bridge_registry(
            &fixture.state_dir,
            session_id,
            instance_id,
            &socket_path,
            &token_path,
        );

        let mut command = yzx_control_command();
        apply_managed_config_env(&mut command, &fixture);
        let output = command
            .env("YAZELIX_SESSION_CONFIG_PATH", snapshot)
            .arg("helix")
            .arg("action")
            .arg("helix.get_context")
            .arg("--zellij-pane-id")
            .arg("terminal:7")
            .arg("--json")
            .output()
            .unwrap();

        assert_success(&output);
        bridge.join().unwrap();
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(response["status"], "ok");
        assert_eq!(response["data"]["cwd"], "/tmp/project");
    }
}
