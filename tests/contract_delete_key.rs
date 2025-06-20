use std::env;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_create_and_delete_key() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");

    let key_id = "key_to_delete";

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    // Create the key
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
        "00ffcc",
        "--timeout",
        "60",
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

    let status = create.status().expect("Failed to run create-key CLI");
    assert!(status.success(), "create-key should succeed");

    let contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    assert!(contents.contains(key_id), "Key should be present in config");

    // Delete the key
    let mut delete = Command::new("cargo");
    delete.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "delete-key",
        "--key-id",
        key_id,
        "--yes",
    ]);

    let status = delete.status().expect("Failed to run delete-key CLI");
    assert!(status.success(), "delete-key should succeed");

    let contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    assert!(
        !contents.contains(key_id),
        "Key should be removed from config after deletion"
    );
}

#[test]
fn test_delete_nonexistent_key_fails_gracefully() {
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
        "delete-key",
        "--key-id",
        key_id,
        "--yes",
    ]);

    let output = cmd.output().expect("Failed to run CLI");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "delete-key should fail for non-existent key"
    );
    assert!(
        stderr.contains("not found"),
        "Expected error about key not found, got: {stderr}"
    );
}
