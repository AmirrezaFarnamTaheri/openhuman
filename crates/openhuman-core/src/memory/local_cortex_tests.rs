use super::*;

#[test]
fn managed_launch_is_loopback_only_and_persists_storage() {
    let args = launch_args("test-key");
    assert!(args.windows(2).any(|pair| pair == ["--publish", "127.0.0.1:3141:3141"]));
    assert!(args.windows(2).any(|pair| pair == ["--volume", "openhuman-cortexdb-data:/data"]));
    assert!(args.windows(2).any(|pair| pair == ["--env", "CORTEX_API_KEY=test-key"]));
    assert!(args.windows(2).any(|pair| pair == ["--restart", "unless-stopped"]));
    assert!(args.windows(2).any(|pair| pair == ["--label", "openhuman.managed=memory"]));
    assert_eq!(args.last().map(String::as_str), Some("cortexdb/cortexdb:v0.10.1"));
    assert!(!args.iter().any(|value| value == "--network=host" || value == "0.0.0.0:3141:3141"));
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
