use std::{env, fs, process::Command};
use tempfile::tempdir;

#[test]
fn test_daemon_logs_file_is_created() {
    let _ = dotenvy::from_filename(".env.test");

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("daemon_log_test_config.json");
    let key_id = "log_key";

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    // Create key
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
        "feedface",
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
        "2",
    ]);
    if mock {
        create.arg("--mock");
    }
    assert!(create.status().unwrap().success(), "Key creation failed");

    // Run daemon
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
        "1",
        "--shots",
        "5",
        "--gas-buffer",
        "2",
    ]);
    if mock {
        daemon.arg("--mock");
    }

    let _ = daemon.status().expect("Failed to run daemon CLI");

    // Check that log file was created and is not empty
    let log_contents = fs::read_to_string(".zaphenathd.log").expect("Missing log file");
    assert!(
        log_contents.contains("Pinged key") || log_contents.contains("🔁 Starting ping cycle"),
        "Expected log output not found"
    );
}
