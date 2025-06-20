use serde_json::Value;
use std::env;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_create_key_set_and_remove_custodian_succeeds() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "key_for_custodian_removal";
    let custodian_address = "0x1234567890123456789012345678901234567890";
    let role = "Writer"; // Using Writer for testing removal of a specific role

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    // 1. Create the key
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
        "48656c6c6f2c205a61706821", // "Hello, Zaph!" in hex
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
    assert!(
        create_status.success(),
        "Key creation failed: {:?}",
        create_cmd.output()
    );

    // 2. Set the custodian
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
        "Set custodian failed: {:?}",
        set_custodian_cmd.output()
    );

    // Verify custodian is in config
    let contents_after_set = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config_json_after_set: Value =
        serde_json::from_str(&contents_after_set).expect("Failed to parse config JSON");
    let key_entry_after_set = config_json_after_set
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry.get("key_id").and_then(Value::as_str) == Some(key_id))
        .unwrap();
    let custodians_after_set = key_entry_after_set
        .get("custodians")
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(
        custodians_after_set.len(),
        1,
        "Custodian not added to config"
    );
    assert!(
        custodians_after_set
            .iter()
            .any(|c| c["address"].as_str() == Some(&custodian_address.to_lowercase())),
        "Added custodian not found in config"
    );

    // 3. Remove the custodian
    let mut remove_custodian_cmd = Command::new("cargo");
    remove_custodian_cmd.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "remove-custodian",
        "--key-id",
        key_id,
        "--user-address",
        custodian_address,
        "--yes",
        "--gas-buffer",
        "1.1",
    ]);
    if mock {
        remove_custodian_cmd.arg("--mock");
    }
    let remove_custodian_status = remove_custodian_cmd
        .status()
        .expect("Failed to run remove-custodian CLI");
    assert!(
        remove_custodian_status.success(),
        "Remove custodian failed: {:?}",
        remove_custodian_cmd.output()
    );

    // 4. Verify custodian is removed from config
    let final_contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config_json: Value =
        serde_json::from_str(&final_contents).expect("Failed to parse config JSON");

    let key_entry = config_json
        .as_array()
        .expect("Config should be an array")
        .iter()
        .find(|entry| entry.get("key_id").and_then(Value::as_str) == Some(key_id))
        .expect("Key entry not found after removal");

    let custodians = key_entry
        .get("custodians")
        .expect("Custodians array not found")
        .as_array()
        .expect("Custodians should be an array");

    assert_eq!(custodians.len(), 0, "Custodian was not removed from config");
    assert!(
        !custodians
            .iter()
            .any(|c| c["address"].as_str() == Some(&custodian_address.to_lowercase())),
        "Removed custodian still found in config"
    );
}
