use crate::{
    cmd::types::GasAndConfirmArgs,
    config::{get_config_path, read_config, write_config},
    contract::{
        create_key,
        types::{ContractSpecs, GasSpecs},
    },
};
use serde_json::json;
use web3::{
    signing::{Key, SecretKeyRef},
    types::Address,
};

pub async fn handle_create_key(
    key_id: &str,
    data: &str,
    timeout: u64,
    contract_specs: &mut ContractSpecs,
    mock: bool,
    gas_confirm: &GasAndConfirmArgs,
) -> Result<(), anyhow::Error> {
    match contract_specs.load_private_key_if_missing() {
        Ok(r) => r,
        Err(err) => {
            let e = anyhow::anyhow!("❌load private key failed. Reason: {:?}", err);
            return Err(e);
        }
    };

    // 🧪 Skip on-chain interaction if mock is enabled
    // tx_result will now directly contain the H256 on success or an Error
    if mock {
        println!("[MOCK] Skipping actual on-chain call");
        Ok(Default::default()) // Return a dummy H256 for mock
    } else {
        create_key::create_key_on_chain(
            contract_specs,
            key_id,
            data,
            timeout,
            gas_confirm.yes,
            GasSpecs {
                gas_limit: gas_confirm.gas_limit,
                gas_buffer: gas_confirm.gas_buffer,
            },
            gas_confirm.nonce,
        )
        .await // This await now waits for the full transaction confirmation
    }?; // Use '?' to propagate errors from create_key_on_chain

    // If we reach here, the transaction was successful (or mock was enabled)
    // 🧠 Derive owner address from private key
    let sk = match contract_specs.get_private_key() {
        Some(sec) => sec,
        None => {
            // This case should ideally not happen if load_private_key_if_missing was successful
            // and no external factors cleared the priv_key.
            // However, added as a fallback/safety.
            match contract_specs.load_private_key_if_missing() {
                Ok(sec) => sec,
                Err(e) => {
                    let e =
                        anyhow::anyhow!("Failed to load private key for owner derivation: {:?}", e);
                    return Err(e);
                }
            }
        }
    };

    let owner_addr: Address = SecretKeyRef::new(sk).address();
    let owner_addr_str = format!("{:#x}", owner_addr);

    // ✅ Update config only on success
    let mut config = match read_config(&get_config_path()) {
        Ok(cfg) => cfg,
        Err(e) => {
            let e = anyhow::anyhow!("❌ Failed to read config: {e:?}");
            return Err(e);
        }
    };

    let new_key = json!({
        "key_id": key_id,
        "contract_address": contract_specs.contract_addr,
        "private_key_path": contract_specs.priv_key_path,
        "owner": owner_addr_str,
        "network": contract_specs.ctx.network,
        "rpc_url": contract_specs.ctx.rpc_url,
        "timeout": timeout,
        "custodians": []
    });

    if let Some(arr) = config.as_array_mut() {
        if arr
            .iter()
            .any(|e| e["key_id"].as_str().is_some_and(|k| k == key_id))
        {
            let e = anyhow::anyhow!("⚠️ Key already exists in config. Skipping save.");
            Err(e)
        } else {
            arr.push(new_key);
            if let Err(e) = write_config(&get_config_path(), &config) {
                let e = anyhow::anyhow!("❌ Failed to write config: {e:?}");
                Err(e)
            } else {
                println!("✅ Key config saved locally");
                Ok(())
            }
        }
    } else {
        let e = anyhow::anyhow!("❌ Invalid config format");
        Err(e)
    }
}
