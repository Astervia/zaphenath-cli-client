use crate::{
    cmd::types::GasAndConfirmArgs,
    config::{get_config_path, read_config, write_config},
    contract::{
        delete_key,
        types::{ContractSpecs, GasSpecs, NetworkContext},
    },
};
use serde_json::Value;

pub async fn handle_delete_key(
    key_id: &str,
    gas_confirm: &GasAndConfirmArgs,
) -> Result<(), anyhow::Error> {
    // Load config
    let config_path = get_config_path();
    let config_value = match read_config(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            let e = anyhow::anyhow!("❌ Failed to read config: {err}");
            return Err(e);
        }
    };

    // Find the key entry
    let key_array = match config_value.as_array() {
        Some(arr) => arr.clone(), // Clone to make it mutable
        None => {
            let e = anyhow::anyhow!("❌ Invalid config format");
            return Err(e);
        }
    };

    let Some(key_entry) = key_array
        .iter()
        .find(|entry| entry.get("key_id").and_then(Value::as_str) == Some(key_id))
    else {
        let e = anyhow::anyhow!("❌ Key with ID '{}' not found in config", key_id);
        return Err(e);
    };

    // Build contract specs from config
    let contract_specs = match (
        key_entry.get("contract_address").and_then(Value::as_str),
        key_entry.get("private_key_path").and_then(Value::as_str),
        key_entry.get("rpc_url").and_then(Value::as_str),
    ) {
        (Some(contract_addr), Some(priv_key_path), Some(rpc_url)) => ContractSpecs {
            ctx: NetworkContext {
                rpc_url: rpc_url.to_string(),
                network: Some(
                    key_entry
                        .get("network")
                        .and_then(Value::as_str)
                        .unwrap_or("custom")
                        .to_string(),
                ),
            },
            contract_addr: contract_addr.to_string(),
            priv_key_path: priv_key_path.to_string(),
            priv_key: None, // Will be loaded by delete_key_on_chain directly from file
        },
        _ => {
            let e = anyhow::anyhow!("❌ Incomplete key configuration for '{}'", key_id);
            return Err(e);
        }
    };

    let owner_address = match key_entry.get("owner").and_then(Value::as_str) {
        Some(addr) => addr,
        None => {
            let e = anyhow::anyhow!("❌ 'owner' address not found for key '{}'", key_id);
            return Err(e);
        }
    };

    // Call on-chain deletion and wait for confirmation
    let _tx_hash = delete_key::delete_key_on_chain(
        &contract_specs, // Passed as immutable reference
        key_id,
        owner_address,
        gas_confirm.yes,
        GasSpecs {
            gas_limit: gas_confirm.gas_limit,
            gas_buffer: gas_confirm.gas_buffer,
        },
        gas_confirm.nonce,
    )
    .await?;

    // If we reach here, the transaction was successful on-chain.
    // Remove key from local config
    let updated: Vec<_> = key_array
        .into_iter() // Use into_iter() for Vec<Value> (consumes key_array)
        .filter(|e| e.get("key_id").and_then(Value::as_str) != Some(key_id))
        .collect();

    if let Err(err) = write_config(&config_path, &Value::Array(updated)) {
        let e = anyhow::anyhow!("⚠️ Key removed on-chain but failed to update local config: {err}");
        Err(e)
    } else {
        println!("✅ Key removed from local config");
        Ok(())
    }
}
