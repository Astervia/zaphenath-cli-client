use crate::contract::types::{ContractSpecs, CustodianData, GasSpecs}; // Added CustodianData
use dialoguer::Confirm;
use std::str::FromStr;
use web3::Web3;
use web3::contract::{Contract, Options};
use web3::signing::{Key, SecretKeyRef};
use web3::transports::Http;
use web3::types::{Address, H256, U64, U256}; // Import Web3

// Removed: use super::types::CustodianData; as it's now in crate::contract::types

pub async fn set_custodian_on_chain(
    contract_specs: &mut ContractSpecs,
    key_id: &str,
    owner_address: &str,            // The owner of the key
    custodian_data: &CustodianData, // Use the CustodianData struct
    yes: bool,
    gas_specs: GasSpecs,
    nonce: Option<u64>,
) -> Result<H256, anyhow::Error> {
    let http = Http::new(&contract_specs.ctx.rpc_url)?;
    let web3 = Web3::new(http); // Use the imported Web3

    let contract_address = Address::from_str(&contract_specs.contract_addr)
        .map_err(|_| anyhow::anyhow!("Invalid contract address"))?;

    // Ensure private key is loaded for signing
    let sk = match contract_specs.get_private_key() {
        Some(sec) => sec,
        None => contract_specs.load_private_key_if_missing()?,
    };
    let wallet = SecretKeyRef::new(sk);

    let abi_json = include_str!("../../abi/Zaphenath.json"); // Adjust path if needed
    let contract = Contract::from_json(web3.eth(), contract_address, abi_json.as_bytes())?;

    let key_hash = H256::from_slice(web3::signing::keccak256(key_id.as_bytes()).as_slice());
    let owner = Address::from_str(owner_address)?;
    let user = Address::from_str(&custodian_data.address)?;
    let role_u8: u8 = custodian_data.role.into(); // Convert Role enum to u8

    let call_params = (key_hash, owner, user, role_u8, custodian_data.can_ping);

    let gas_to_use = if let Some(limit) = gas_specs.gas_limit {
        Some(U256::from(limit))
    } else if gas_specs.gas_buffer.is_none() && yes {
        None // Auto-estimate gas without buffering if --yes and no buffer specified
    } else {
        let est: U256 = contract
            .estimate_gas(
                "setCustodian",
                call_params,
                wallet.address(),
                Options::default(),
            )
            .await?;

        let gas_with_buffer = if let Some(buffer) = gas_specs.gas_buffer {
            let est_f64 = est.as_u128() as f64;
            let buffered = est_f64 * buffer;
            U256::from(buffered as u128)
        } else {
            est
        };

        if !yes {
            let prompt = format!(
                "Estimated gas: {}\nBuffered gas limit: {}\nSend transaction to set custodian?",
                est, gas_with_buffer
            );

            if !Confirm::new().with_prompt(prompt).interact()? {
                println!("❌ Aborted.");
                return Err(anyhow::anyhow!("Transaction aborted by user"));
            }
        }

        Some(gas_with_buffer)
    };

    let options = Options {
        gas: gas_to_use,
        nonce: nonce.map(U256::from),
        ..Options::default()
    };

    let tx_hash = contract
        .signed_call("setCustodian", call_params, options, wallet)
        .await?;

    println!("⏳ Waiting for transaction {:?} to be mined...", tx_hash);

    let receipt = loop {
        match web3.eth().transaction_receipt(tx_hash).await {
            Ok(Some(receipt)) => break receipt,
            Ok(None) => {
                // Transaction not yet mined, wait and retry
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Error getting transaction receipt: {:?}",
                    e
                ));
            }
        }
    };

    if let Some(status) = receipt.status {
        if status == U64::one() {
            println!(
                "✅ Custodian '{:#x}' set for key '{}' on-chain. Tx hash: {:?}",
                user, key_id, tx_hash
            );
            Ok(tx_hash)
        } else {
            Err(anyhow::anyhow!(
                "Transaction failed on-chain. Status: {:?}",
                status
            ))
        }
    } else {
        // This case indicates the transaction was likely replaced or dropped
        Err(anyhow::anyhow!(
            "Transaction status unknown for {:?}. It might have been dropped or replaced.",
            tx_hash
        ))
    }
}
