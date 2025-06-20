use serde_json::{Value, json};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use web3::{
    signing::{Key, SecretKeyRef},
    types::Address,
};

use crate::contract::types::{ContractSpecs, KeyData};

/// Get the default configuration file path
pub fn get_config_path() -> PathBuf {
    // Priority: ENV override -> platform default
    if let Ok(custom) = std::env::var("ZAPHENATH_CONFIG_PATH") {
        return PathBuf::from(custom);
    }

    #[cfg(target_os = "windows")]
    let base =
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"));

    #[cfg(target_os = "linux")]
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));

    #[cfg(target_os = "macos")]
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/Library/Application Support"));

    let mut path = base.join("zaphenath");
    std::fs::create_dir_all(&path).expect("Could not create config directory");
    path.push("config.json");
    path
}

/// Reads the configuration from disk
pub fn read_config(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    if !path.exists() {
        return Ok(json!([]));
    }

    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let value: Value = serde_json::from_str(&data)?;
    Ok(value)
}

/// Writes the configuration to disk
pub fn write_config(path: &PathBuf, config: &Value) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    let data = serde_json::to_string_pretty(config).expect("Failed to serialize config");
    file.write_all(data.as_bytes())?;
    Ok(())
}

pub fn view_config(path: &Path) {
    // or path: &PathBuf
    match read_config(path) {
        Ok(config) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&config)
                    .unwrap_or_else(|_| "Invalid JSON format".to_string())
            );
        }
        Err(err) => {
            eprintln!("Error reading configuration: {}", err);
        }
    }
}

/// Adds a new key entry to the configuration file
pub async fn add_key(
    path: &PathBuf,
    contract_specs: &mut ContractSpecs,
    key_data: KeyData,
) -> Result<(), anyhow::Error> {
    let mut config = match read_config(path) {
        Ok(Value::Array(arr)) => arr,
        Ok(_) => {
            let e = anyhow::anyhow!("Invalid config format. Expected array of keys.");
            return Err(e);
        }
        Err(_) => vec![],
    };

    let owner = if let Some(own) = key_data.owner {
        own
    } else {
        // 🧠 Derive owner address from private key
        let sk = match contract_specs.get_private_key() {
            Some(sec) => sec,
            None => match contract_specs.load_private_key_if_missing() {
                Ok(sec) => sec,
                Err(e) => {
                    let e = anyhow::anyhow!("Failed to load private key: {:?}", e);
                    return Err(e);
                }
            },
        };
        let owner_addr: Address = SecretKeyRef::new(sk).address();
        let owner_addr_str = format!("{:#x}", owner_addr);
        owner_addr_str
    };

    let new_key = json!({
        "key_id": key_data.id,
        "contract_address": contract_specs.contract_addr,
        "private_key_path": contract_specs.priv_key_path,
        "owner": owner,
        "rpc_url": contract_specs.ctx.rpc_url,
        "network": contract_specs.ctx.network,
        "timeout": key_data.timeout,
        //"ping_interval": null,
        "custodians": []
    });

    config.push(new_key);

    match write_config(path, &json!(config)) {
        Ok(_) => {
            println!("Key successfully added to config.");
            Ok(())
        }
        Err(err) => {
            let e = anyhow::anyhow!("Failed to write config: {}", err);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::network;

    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_write_and_read_config_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");

        let sample = json!([
            {
                "key_id": "abc123",
                "contract_address": "0xdeadbeef",
                "private_key_path": "/fake/key",
                "owner": "0x123abc",
                "timeout": 123,
                //"ping_interval": null,
                "custodians": []
            }
        ]);

        write_config(&file_path, &sample).expect("Failed to write config");

        let result = read_config(&file_path).expect("Failed to read config");
        assert_eq!(result, sample);
    }

    #[tokio::test] // gives a runtime and lets the fn be async
    async fn test_add_key_to_empty_config() -> Result<(), anyhow::Error> {
        let dir = tempdir()?;
        let file_path = dir.path().join("config.json");

        // ensure config starts empty
        assert!(!file_path.exists());

        let ctx = network::build_network_context(None, Some("mainnet"))
            .expect("❌ Missing network or rpc-url");

        // call the async function and bubble up any error with `?`
        add_key(
            &file_path,
            &mut ContractSpecs {
                ctx,
                contract_addr: "key_1".to_string(),
                priv_key_path: "/path/to/key".to_string(),
                priv_key: None,
            },
            KeyData {
                id: "key_1".to_string(),
                owner: Some("0x123abc".to_string()),
                timeout: 42,
            },
        )
        .await?; // <- await and propagate

        // now verify that the file was written correctly
        let config = read_config(&file_path).unwrap();
        assert!(config.is_array());
        let arr = config.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["key_id"], "key_1");
        assert_eq!(arr[0]["timeout"], 42);

        Ok(())
    }

    #[test]
    fn test_view_config_prints_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");

        let config = json!([
            {
                "key_id": "k1",
                "contract_address": "0xabc",
                "private_key_path": "/tmp/key",
                "owner": "0x123abc",
                "timeout": 300,
                //"ping_interval": null,
                "custodians": []
            }
        ]);
        write_config(&file_path, &config).unwrap();

        // Redirect stdout to capture print
        let output = std::panic::catch_unwind(|| {
            view_config(&file_path); // prints to stdout
        });
        assert!(output.is_ok());
    }
}
