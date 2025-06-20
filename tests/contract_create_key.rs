use std::env;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_contract_create_key_adds_to_config() {
    let _ = dotenvy::from_filename(".env.test").ok();

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");
    let key_id = "testkey123";

    let mock = env::var("ZAPHENATH_TEST_MOCK").unwrap_or_else(|_| "true".into()) == "true";
    let contract_address = env::var("ZAPHENATH_TEST_CONTRACT")
        .unwrap_or_else(|_| "0x0000000000000000000000000000000000000001".into());
    let private_key_path = env::var("ZAPHENATH_TEST_PRIVKEY").unwrap_or("/dev/null".into());
    let rpc_url = env::var("ZAPHENATH_TEST_RPC").unwrap_or("http://localhost:8545".into());

    let mut cmd = Command::new("cargo");
    cmd.args([
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
        "120",
        "--rpc-url",
        &rpc_url,
        "--contract-address",
        &contract_address,
        "--private-key-path",
        &private_key_path,
        "--yes",
        "--gas-buffer",
        "1.2",
    ]);

    if mock {
        cmd.arg("--mock");
    }

    let status = cmd.status().expect("Failed to run CLI");
    assert!(status.success(), "create-key CLI did not succeed");

    let contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    assert!(contents.contains(key_id));
    assert!(contents.contains(&contract_address));
}
