use crate::{
    config::{get_config_path, read_config},
    contract::{
        read::read_key_on_chain,
        types::{ContractSpecs, NetworkContext},
    },
};
use serde_json::Value;

pub async fn handle_read_key(key_id: &str, decode: bool) -> Result<(), anyhow::Error> {
    // ✅ Update config only on success
    let config_value = match read_config(&get_config_path()) {
        Ok(cfg) => cfg,
        Err(e) => {
            let e = anyhow::anyhow!("❌ Failed to read config: {e:?}");
            return Err(e);
        }
    };

    let key_array = config_value
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("❌ Invalid config format"))?;

    let key_entry = key_array
        .iter()
        .find(|entry| entry.get("key_id").and_then(Value::as_str) == Some(key_id))
        .ok_or_else(|| anyhow::anyhow!("❌ Key '{}' not found in config", key_id))?;

    // Extract needed fields
    let (contract_addr, rpc_url, owner) = (
        key_entry
            .get("contract_address")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing contract_address"))?,
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
    let priv_key_path = key_entry
        .get("private_key_path")
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("Missing private_key_path"))?;

    let mut specs = ContractSpecs {
        ctx: NetworkContext {
            rpc_url: rpc_url.to_string(),
            network,
        },
        contract_addr: contract_addr.to_string(),
        priv_key_path,
        priv_key: None,
    };

    let result = read_key_on_chain(&mut specs, key_id, owner).await?;

    if decode {
        match std::str::from_utf8(&result.0) {
            Ok(s) => {
                println!("{}", s);
            }
            Err(e) => {
                let e = anyhow::anyhow!(
                    "⚠️ Data is not valid UTF-8. Use --decode only if the content is encoded as UTF-8: {e:?}"
                );
                return Err(e);
            }
        }
    } else {
        println!("0x{}", hex::encode(&result.0));
    }

    Ok(())
}
