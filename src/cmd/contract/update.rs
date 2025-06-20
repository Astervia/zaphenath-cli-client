use crate::{
    cmd::types::GasAndConfirmArgs,
    config::{get_config_path, read_config, write_config},
    contract::{
        types::{ContractSpecs, GasSpecs, KeyData, NetworkContext},
        update::update_key_on_chain,
    },
};
use serde_json::{Value, json};

pub async fn handle_update_key(
    key_id: &str,
    new_data_hex: &str,
    new_timeout: u64,
    mock: bool,
    gas_confirm: &GasAndConfirmArgs,
) -> Result<(), anyhow::Error> {
    // Load config
    let config_path = get_config_path();
    let mut config = match read_config(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            let e = anyhow::anyhow!("❌ Failed to read config: {err}");
            return Err(e);
        }
    };

    let key_array = config
        .as_array_mut()
        .ok_or_else(|| anyhow::anyhow!("❌ Invalid config format"))?;

    let key_entry = key_array
        .iter_mut() // <-- mutable iterator
        .find(|e| e.get("key_id").and_then(Value::as_str) == Some(key_id))
        .ok_or_else(|| anyhow::anyhow!("❌ Key '{}' not found in config", key_id))?;

    let (contract_addr, priv_key_path, rpc_url, owner) = (
        key_entry
            .get("contract_address")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing contract_address"))?,
        key_entry
            .get("private_key_path")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing private_key_path"))?,
        key_entry
            .get("rpc_url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing rpc_url"))?,
        key_entry
            .get("owner")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing owner address"))?,
    );

    let network = key_entry
        .get("network")
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut specs = ContractSpecs {
        ctx: NetworkContext {
            rpc_url: rpc_url.to_string(),
            network,
        },
        contract_addr: contract_addr.to_string(),
        priv_key_path: priv_key_path.to_string(),
        priv_key: None,
    };

    let key_data_for_call = KeyData {
        // Renamed to avoid conflict with `key_data` for config update
        id: key_id.to_string(),
        owner: Some(owner.to_string()),
        timeout: new_timeout,
    };

    if mock {
        println!("[MOCK] Skipping on-chain update call");
        // In mock mode, we still simulate success for config update
        // No need to await anything here.
    } else {
        // Call on-chain function and wait for confirmation
        let _tx_hash = update_key_on_chain(
            &mut specs,
            owner,
            key_data_for_call,
            new_data_hex,
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
    // ✅ Update config values
    key_entry["timeout"] = json!(new_timeout);
    key_entry["data"] = json!(new_data_hex); // Assuming 'data' field exists in your config JSON structure

    write_config(&config_path, &config)?; // Use '?' here too for write_config
    println!("📝 Config updated locally.");
    Ok(())
}
