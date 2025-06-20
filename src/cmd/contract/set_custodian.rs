use crate::{
    cmd::types::GasAndConfirmArgs,
    config::{get_config_path, read_config, write_config},
    contract::{
        set_custodian::set_custodian_on_chain,
        types::{ContractSpecs, CustodianData, GasSpecs, NetworkContext, Role},
    },
};
use serde_json::{Value, json};
use std::str::FromStr; // Needed for FromStr trait on Role

pub async fn handle_set_custodian(
    key_id: &str,
    user_address: &str,
    role_str: &str, // Role as a string from CLI
    can_ping: bool,
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

    // 3. Parse Role from string
    let role = Role::from_str(role_str)?;

    // 4. Build ContractSpecs from config
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
        priv_key: None, // Will be loaded by set_custodian_on_chain
    };

    // 5. Call on-chain function and wait for confirmation
    let _tx_hash = set_custodian_on_chain(
        // Assign to _tx_hash as it's not directly used after this
        &mut contract_specs, // Pass mutable reference
        key_id,
        owner_address,
        &CustodianData {
            // Pass the CustodianData struct
            address: user_address.to_string(),
            can_ping,
            role,
        },
        gas_confirm.yes,
        GasSpecs {
            gas_limit: gas_confirm.gas_limit,
            gas_buffer: gas_confirm.gas_buffer,
        },
        gas_confirm.nonce,
    )
    .await?; // Use '?' to propagate errors from set_custodian_on_chain

    // If we reach here, the transaction was successful on-chain.
    // 6. Update local config
    // Ensure "custodians" field exists and is an array. If not, initialize it.
    if !key_entry["custodians"].is_array() {
        key_entry["custodians"] = json!([]);
    }

    let custodians_array = key_entry["custodians"]
        .as_array_mut()
        .expect("Expected 'custodians' to be an array after initialization");

    let new_custodian = json!({
        "address": user_address.to_lowercase(),
        "role": role_str.to_lowercase(),
        "can_ping": can_ping,
    });

    // Check if custodian already exists and update, or push new
    if let Some(existing_index) = custodians_array
        .iter()
        .position(|c| c["address"].as_str() == Some(&user_address.to_lowercase()))
    {
        custodians_array[existing_index] = new_custodian;
        println!("✅ Updated custodian '{}' in local config.", user_address);
    } else {
        custodians_array.push(new_custodian);
        println!("✅ Added custodian '{}' to local config.", user_address);
    }

    if let Err(err) = write_config(&config_path, &config_value) {
        let e =
            anyhow::anyhow!("⚠️ Custodian set on-chain but failed to update local config: {err}");
        Err(e)
    } else {
        Ok(())
    }
}
