use crate::{
    cmd::types::GasAndConfirmArgs,
    config::{get_config_path, read_config, write_config},
    contract::{
        ping::ping_key_on_chain,
        types::{ContractSpecs, GasSpecs, NetworkContext},
    },
};
use chrono::Utc;
use serde_json::Value;

pub async fn handle_ping_key(
    key_id: &str,
    mock: bool,
    gas_confirm: &GasAndConfirmArgs,
) -> Result<(), anyhow::Error> {
    // Load config
    let config_path = get_config_path();
    let mut config_value = match read_config(&config_path) {
        // Make mutable for update
        Ok(cfg) => cfg,
        Err(e) => {
            let e = anyhow::anyhow!("❌ Failed to read config: {e:?}");
            return Err(e);
        }
    };

    // Find key entry
    let key_array = config_value.as_array_mut().ok_or_else(|| {
        // Get mutable array
        anyhow::anyhow!("❌ Invalid config format: expected array of key entries")
    })?;

    let key_entry_index = key_array
        .iter()
        .position(|entry| entry.get("key_id").and_then(Value::as_str) == Some(key_id))
        .ok_or_else(|| anyhow::anyhow!("❌ Key ID '{}' not found in config", key_id))?;

    let key_entry = &mut key_array[key_entry_index]; // Get mutable reference to the entry

    // Build contract specs
    let (contract_addr, priv_key_path, rpc_url) = (
        key_entry
            .get("contract_address")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("❌ Missing contract_address in config"))?,
        key_entry
            .get("private_key_path")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("❌ Missing private_key_path in config"))?,
        key_entry
            .get("rpc_url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("❌ Missing rpc_url in config"))?,
    );

    let network = key_entry
        .get("network")
        .and_then(Value::as_str)
        .map(str::to_string);

    let owner_address = key_entry
        .get("owner")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("❌ Missing owner in config"))?;

    let mut specs = ContractSpecs {
        ctx: NetworkContext {
            rpc_url: rpc_url.to_string(),
            network,
        },
        contract_addr: contract_addr.to_string(),
        priv_key_path: priv_key_path.to_string(),
        priv_key: None,
    };

    // 🧪 Mock handling
    if mock {
        println!("[MOCK] Skipping on-chain ping call");
        // In mock mode, we still simulate success for config update if needed
    } else {
        // Call on-chain ping and wait for confirmation
        let _tx_hash = ping_key_on_chain(
            &mut specs,
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
    };

    // If we reach here, the transaction was successful on-chain (or mock was enabled).
    // Update last_ping_timestamp in config
    let now = Utc::now().timestamp();
    key_entry["last_ping_timestamp"] = serde_json::to_value(now)?; // Add or update last_ping_timestamp

    // Write the updated config back to disk
    write_config(&config_path, &config_value)?; // Use '?' here too

    println!(
        "✅ Key '{}' pinged successfully, and local config updated.",
        key_id
    );
    Ok(())
}
