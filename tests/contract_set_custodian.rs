use serde_json::Value;
use std::env;
use std::fs;
use std::process::Command;
use tempfile::tempdir; // Import Value and json for config parsing

#[test]
fn test_create_key_and_set_custodian_succeeds() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "key_for_custodian";
    let custodian_address = "0x1234567890123456789012345678901234567890"; // Example address
    let role = "Reader"; // Corresponds to Role::Reader in Solidity

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    // 1. Create the key first
    let mut create_cmd = Command::new("cargo");
    create_cmd.args([
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
        "deadbeef", // Initial data for the key
        "--timeout",
        "3600",
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
        create_cmd.arg("--mock");
    }

    let create_status = create_cmd.status().expect("Failed to run create-key CLI");
    assert!(create_status.success(), "Key creation failed");

    // Assert that the key exists in config after creation
    let contents_after_create =
        fs::read_to_string(&config_path).expect("Failed to read config file");
    assert!(
        contents_after_create.contains(key_id),
        "Key was not created in config"
    );

    // 2. Set the custodian for the created key
    let mut set_custodian_cmd = Command::new("cargo");
    set_custodian_cmd.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "set-custodian",
        "--key-id",
        key_id,
        "--user-address",
        custodian_address,
        "--role",
        role,
        "--can-ping",
        "--yes",
        "--gas-buffer",
        "1.1",
    ]);
    if mock {
        set_custodian_cmd.arg("--mock");
    }

    let set_custodian_status = set_custodian_cmd
        .status()
        .expect("Failed to run set-custodian CLI");
    assert!(
        set_custodian_status.success(),
        "Set custodian should succeed"
    );

    // 3. Verify the config file contains the new custodian
    let final_contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config_json: Value =
        serde_json::from_str(&final_contents).expect("Failed to parse config JSON");

    let key_entry = config_json
        .as_array()
        .expect("Config should be an array")
        .iter()
        .find(|entry| entry.get("key_id").and_then(Value::as_str) == Some(key_id))
        .expect("Key entry not found after setting custodian");

    let custodians = key_entry
        .get("custodians")
        .expect("Custodians array not found")
        .as_array()
        .expect("Custodians should be an array");

    assert_eq!(custodians.len(), 1, "Should have exactly one custodian");
    let added_custodian = &custodians[0];

    assert_eq!(
        added_custodian.get("address").and_then(Value::as_str),
        Some(custodian_address.to_lowercase().as_str()),
        "Custodian address mismatch"
    );
    assert_eq!(
        added_custodian.get("role").and_then(Value::as_str),
        Some(role.to_lowercase().as_str()),
        "Custodian role mismatch"
    );
    assert_eq!(
        added_custodian.get("can_ping").and_then(Value::as_bool),
        Some(true),
        "Custodian can_ping mismatch"
    );
}

#[test]
fn test_set_custodian_nonexistent_key_fails() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "nonexistent_key_for_custodian"; // This key will not be created
    let custodian_address = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let role = "Reader";

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";

    let mut set_custodian_cmd = Command::new("cargo");
    set_custodian_cmd.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "set-custodian",
        "--key-id",
        key_id,
        "--user-address",
        custodian_address,
        "--role",
        role,
        "--yes",
    ]);
    if mock {
        set_custodian_cmd.arg("--mock");
    }

    let output = set_custodian_cmd
        .output()
        .expect("Failed to run set-custodian CLI");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "set-custodian should fail for non-existent key"
    );
    assert!(
        stderr.contains("not found in config"), // Matches the error message from handle_set_custodian
        "Expected error about key not found, got: {}",
        stderr
    );
    // Ensure config file is not created or modified
    assert!(
        !config_path.exists()
            || fs::read_to_string(&config_path)
                .unwrap_or_default()
                .is_empty(),
        "Config file should not contain any data for a nonexistent key operation"
    );
}
