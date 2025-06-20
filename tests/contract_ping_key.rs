use std::env;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_create_and_ping_key_succeeds() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "key_to_ping";

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    // Create key first
    let mut create = Command::new("cargo");
    create.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "create-key",
        "--key-id",
        key_id,
        "--data",
        "deadbeef",
        "--timeout",
        "120",
        "--rpc-url",
        &rpc_url,
        "--contract-address",
        &contract_address,
        "--private-key-path",
        &private_key_path,
        "--yes",
        "--gas-buffer",
        "1.1",
    ]);
    if mock {
        create.arg("--mock");
    }

    assert!(create.status().unwrap().success(), "key creation failed");

    // Now ping the key
    let mut ping = Command::new("cargo");
    ping.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "ping-key",
        "--key-id",
        key_id,
        "--yes",
        "--gas-buffer",
        "1.1",
    ]);
    if mock {
        ping.arg("--mock");
    }

    let status = ping.status().expect("Failed to run ping-key CLI");
    assert!(status.success(), "ping-key should succeed");
}

#[test]
fn test_ping_nonexistent_key_should_fail() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "ghost_key";

    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "ping-key",
        "--key-id",
        key_id,
        "--yes",
        "--gas-buffer",
        "1.0",
    ]);

    let output = cmd.output().expect("Failed to run CLI");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "ping-key should fail if key not in config"
    );
    assert!(
        stderr.contains("not found"),
        "Expected error about key not found, got: {stderr}"
    );
}
