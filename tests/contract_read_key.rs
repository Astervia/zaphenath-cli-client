use std::process::Command;
use std::time::Duration;
use std::{env, thread};
use tempfile::tempdir;

#[test]
fn test_create_and_read_key_succeeds() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "key_to_read";

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
        "48656c6c6f2c205a61706821", // "Hello, Zaph!" in hex
        "--timeout",
        "0",
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
    } else {
        thread::sleep(Duration::from_secs(1)); // Wait for some blocks to be mined
    }
    assert!(create.status().unwrap().success(), "key creation failed");

    // Read the key
    let mut read = Command::new("cargo");
    read.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "read-key",
        "--key-id",
        key_id,
        "--decode",
    ]);

    let output = read.output().expect("Failed to run read-key");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "read-key should succeed. stderr: {stderr}"
    );
    assert!(
        stdout.contains("Hello, Zaph!"),
        "Decoded output should contain expected string. Output: {stdout}"
    );
}

#[test]
fn test_read_nonexistent_key_fails() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "ghost_key";

    let mut read = Command::new("cargo");
    read.args([
        "run",
        "--quiet",
        "--",
        "--config",
        config_path.to_str().unwrap(),
        "contract",
        "read-key",
        "--key-id",
        key_id,
    ]);

    let output = read.output().expect("Failed to run read-key");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "read-key should fail if key does not exist"
    );
    assert!(
        stderr.contains("not found"),
        "Expected error about key not found. stderr: {stderr}"
    );
}
