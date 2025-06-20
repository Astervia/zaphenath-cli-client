use crate::{
    cmd::types::GasAndConfirmArgs,
    config::{get_config_path, read_config, write_config},
    contract::{
        remove_custodian::remove_custodian_on_chain, // Import the on-chain function
        types::{ContractSpecs, GasSpecs, NetworkContext},
    },
};
use serde_json::Value;

pub async fn handle_remove_custodian(
    key_id: &str,
    user_address: &str,
    gas_confirm: &GasAndConfirmArgs,
) -> Result<(), anyhow::Error> {
    // 1. Load config
    let config_path = get_config_path();
    let mut config_value = match read_config(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            let e = anyhow::anyhow!("❌ Failed to read config: {err}");
            return Err(e);
        }
    };

    // 2. Find the key entry
    let key_array = config_value
        .as_array_mut() // Need mutable access to update it later
        .ok_or_else(|| anyhow::anyhow!("❌ Invalid config format. Expected array of keys."))?;

    let key_entry_index = key_array
        .iter()
        .position(|entry| entry.get("key_id").and_then(Value::as_str) == Some(key_id))
        .ok_or_else(|| anyhow::anyhow!("❌ Key with ID '{}' not found in config", key_id))?;

    let key_entry = &mut key_array[key_entry_index];

    // 3. Build ContractSpecs from config
    let (contract_addr, priv_key_path, rpc_url, owner_address) = (
        key_entry
            .get("contract_address")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing 'contract_address' for key '{}'", key_id))?,
        key_entry
            .get("private_key_path")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing 'private_key_path' for key '{}'", key_id))?,
        key_entry
            .get("rpc_url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing 'rpc_url' for key '{}'", key_id))?,
        key_entry
            .get("owner")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing 'owner' address for key '{}'", key_id))?,
    );

    let network = key_entry
        .get("network")
        .and_then(Value::as_str)
        .map(str::to_string);

    let mut contract_specs = ContractSpecs {
        ctx: NetworkContext {
            rpc_url: rpc_url.to_string(),
            network,
        },
        contract_addr: contract_addr.to_string(),
        priv_key_path: priv_key_path.to_string(),
        priv_key: None, // Will be loaded by remove_custodian_on_chain
    };

    // 4. Call on-chain function to remove custodian
    let _tx_hash = remove_custodian_on_chain(
        &mut contract_specs, // Pass mutable reference
        key_id,
        owner_address,
        user_address,
        gas_confirm.yes,
        GasSpecs {
            gas_limit: gas_confirm.gas_limit,
            gas_buffer: gas_confirm.gas_buffer,
        },
        gas_confirm.nonce,
    )
    .await?; // Use '?' to propagate errors from remove_custodian_on_chain

    // If we reach here, the transaction was successful on-chain.
    // 5. Update local config: Remove custodian from the array
    if !key_entry["custodians"].is_array() {
        key_entry["custodians"] = Value::Array(vec![]); // Initialize if not an array
    }

    let custodians_array = key_entry["custodians"]
        .as_array_mut()
        .expect("Expected 'custodians' to be an array after initialization");

    let original_len = custodians_array.len();
    custodians_array
        .retain(|c| c.get("address").and_then(Value::as_str) != Some(&user_address.to_lowercase()));

    if custodians_array.len() == original_len {
        // If length didn't change, the custodian wasn't found in local config
        println!(
            "⚠️ Custodian '{}' not found in local config for key '{}'. On-chain action successful, but local config was already desynced or custodian was not present.",
            user_address, key_id
        );
    } else {
        println!(
            "✅ Custodian '{}' removed from local config for key '{}'.",
            user_address, key_id
        );
    }

    // 6. Write the updated config back to disk
    if let Err(err) = write_config(&config_path, &config_value) {
        let e = anyhow::anyhow!(
            "⚠️ Custodian removed on-chain but failed to update local config: {err}"
        );
        Err(e)
    } else {
        Ok(())
    }
}
