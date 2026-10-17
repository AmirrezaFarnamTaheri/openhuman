use super::*;

#[test]
fn managed_launch_is_loopback_only_and_persists_storage() {
    let args = launch_args("test-key");
    assert!(args
        .windows(2)
        .any(|pair| pair == ["--publish", "127.0.0.1:3141:3141"]));
    assert!(args
        .windows(2)
        .any(|pair| pair == ["--volume", "openhuman-cortexdb-data:/data"]));
    assert!(args
        .windows(2)
        .any(|pair| pair == ["--env", "CORTEX_API_KEY=test-key"]));
    assert!(args
        .windows(2)
        .any(|pair| pair == ["--restart", "unless-stopped"]));
    assert!(args
        .windows(2)
        .any(|pair| pair == ["--label", "openhuman.managed=memory"]));
    assert_eq!(
        args.last().map(String::as_str),
        Some("cortexdb/cortexdb:v0.10.1")
    );
    assert!(!args
        .iter()
        .any(|value| value == "--network=host" || value == "0.0.0.0:3141:3141"));
}

#[test]
fn existing_managed_container_recovers_its_key_without_a_new_secret() {
    let env = r#"["PATH=/usr/bin", "CORTEX_API_KEY=existing-key", "RUST_LOG=info"]"#;
    assert_eq!(recover_key(env), Some("existing-key".to_string()));
    assert_eq!(recover_key(r#"["PATH=/usr/bin"]"#), None);
    assert_eq!(recover_key("not valid JSON"), None);
}

#[test]
fn managed_endpoint_never_uses_a_network_interface() {
    assert_eq!(MANAGED_ENDPOINT, "http://127.0.0.1:3141");
}

#[cfg(unix)]
#[tokio::test]
async fn docker_failure_names_the_step_without_disclosing_process_output() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let executable = tmp.path().join("docker");
    std::fs::write(
        &executable,
        "#!/bin/sh\nprintf '%s\\n' 'CORTEX_API_KEY=private-test-key'\nprintf '%s\\n' 'private-test-key' >&2\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();

    for (step, expected) in [
        (DockerStep::Readiness, "Docker is not ready"),
        (DockerStep::Inspect, "inspect"),
        (DockerStep::Credential, "credential"),
        (DockerStep::Start, "start the existing"),
        (DockerStep::Create, "download or create"),
    ] {
        let error = checked(&executable, &["{{json .Config.Env}}"], 5, step)
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{error}");
        assert!(!error.contains("private-test-key"));
        assert!(!error.contains("CORTEX_API_KEY"));
        assert!(!error.contains("Config.Env"));
    }
}
