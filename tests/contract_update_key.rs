use std::process::Command;
use std::{env, fs};
use tempfile::tempdir;

#[test]
fn test_create_and_update_key_succeeds() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "key_to_update";

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    // 1. Create key
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
        "abc123",
        "--timeout",
        "300",
        "--rpc-url",
        &rpc_url,
        "--contract-address",
        &contract_address,
        "--private-key-path",
        &private_key_path,
        "--yes",
        "--gas-buffer",
        "1.0",
    ]);
    if mock {
        create.arg("--mock");
    }
    assert!(create.status().unwrap().success(), "key creation failed");

    // 2. Update key
    let mut update = Command::new("cargo");
    update.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "update-key",
        "--key-id",
        key_id,
        "--data",
        "deadbeef",
        "--timeout",
        "600",
        "--yes",
        "--gas-buffer",
        "1.2",
    ]);
    if mock {
        update.arg("--mock");
    }

    let status = update.status().expect("Failed to run update-key CLI");
    assert!(status.success(), "update-key should succeed");

    // 3. Verify changes in config
    let config_contents = fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(
        config_contents.contains("deadbeef"),
        "Updated data not found in config"
    );
    assert!(
        config_contents.contains("\"timeout\": 600"),
        "Updated timeout not found in config"
    );
}

#[test]
fn test_update_nonexistent_key_fails() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");

    let key_id = "nonexistent_key";

    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "update-key",
        "--key-id",
        key_id,
        "--data",
        "deadbeef",
        "--timeout",
        "1000",
        "--yes",
        "--gas-buffer",
        "1.0",
    ]);

    let output = cmd.output().expect("Failed to run CLI");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "update-key should fail for non-existent key"
    );
    assert!(
        stderr.contains("not found"),
        "Expected error about key not found, got: {stderr}"
    );
}
