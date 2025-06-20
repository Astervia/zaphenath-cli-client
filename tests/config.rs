use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_config_add_and_view_with_network_and_rpc_url() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test_config.json");

    // Step 1: Run `config add`
    let status = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "config",
            "add",
            "--key-id",
            "testkey1",
            "--contract-address",
            "0xdeadbeef",
            "--private-key-path",
            "/fake/key",
            "--network",
            "sepolia",
            "--rpc-url",
            "http://localhost:8545",
            "--timeout",
            "600",
            "--owner",
            "0x123abc",
        ])
        .status()
        .expect("Failed to run config add");
    assert!(status.success(), "Command did not exit successfully");

    // Step 2: Read the file directly and validate contents
    let contents = fs::read_to_string(&config_path).expect("Failed to read config file");
    assert!(contents.contains("testkey1"));
    assert!(contents.contains("sepolia"));
    assert!(contents.contains("localhost:8545"));

    // Step 3: Run `config view`
    let output = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "config",
            "view",
        ])
        .output()
        .expect("Failed to run config view");
    assert!(output.status.success(), "View command failed");

    let out_str = String::from_utf8_lossy(&output.stdout);
    assert!(out_str.contains("testkey1"));
    assert!(out_str.contains("sepolia"));
    assert!(out_str.contains("localhost:8545"));
}
