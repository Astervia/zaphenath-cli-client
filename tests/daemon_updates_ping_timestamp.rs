use serde_json::Value;
use std::{env, fs, process::Command};
use tempfile::tempdir;

#[test]
fn test_daemon_run_updates_ping_timestamp() {
    let _ = dotenvy::from_filename(".env.test");

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("daemon_test_config.json");
    let key_id = "daemon_key";

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    // 1. Create a key using the CLI
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
        "feedbeef",
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
        "1.5",
    ]);
    if mock {
        create.arg("--mock");
    }

    let create_status = create.status().expect("Failed to run create-key CLI");
    assert!(create_status.success(), "Key creation failed");

    // 2. Run the daemon in foreground mode for 2 seconds
    let mut daemon = Command::new("cargo");
    daemon.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "daemon",
        "run",
        "--interval",
        "1", // 1 second interval
        "--shots",
        "5",
        "--gas-buffer",
        "2",
    ]);
    if mock {
        daemon.arg("--mock");
    }

    // Use a thread to kill the daemon early if it were real — but we're mocking so we just run it once
    let status = daemon.status().expect("Failed to run daemon CLI");
    assert!(status.success(), "Daemon run failed");

    // 3. Check that last_ping_timestamp was updated in config
    let config_str = fs::read_to_string(&config_path).expect("Failed to read config file");
    let config_json: Value = serde_json::from_str(&config_str).expect("Invalid JSON in config");

    let entry = config_json
        .as_array()
        .expect("Config should be array")
        .iter()
        .find(|e| e.get("key_id").and_then(Value::as_str) == Some(key_id))
        .expect("Key entry not found");

    let last_ping = entry
        .get("last_ping_timestamp")
        .and_then(Value::as_i64)
        .expect("Missing last_ping_timestamp");

    assert!(last_ping > 0, "Ping timestamp was not updated");
}
