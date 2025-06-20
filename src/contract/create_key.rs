use crate::contract::types::{ContractSpecs, GasSpecs};
use dialoguer::Confirm;
use std::str::FromStr;
use web3::contract::{Contract, Options};
use web3::signing::{Key, SecretKeyRef};
use web3::transports::Http;
use web3::types::{Address, Bytes, H256, U64, U256};

pub async fn create_key_on_chain(
    contract_specs: &mut ContractSpecs,
    key_id: &str,
    data: &str,
    timeout: u64,
    yes: bool,
    gas_specs: GasSpecs,
    nonce: Option<u64>,
) -> Result<H256, anyhow::Error> {
    let http = Http::new(&contract_specs.ctx.rpc_url)?;
    let web3 = web3::Web3::new(http);

    let contract_address = Address::from_str(&contract_specs.contract_addr)
        .map_err(|_| anyhow::anyhow!("Invalid contract address"))?;

    let sk = if let Some(sec) = contract_specs.get_private_key() {
        sec
    } else {
        // here you know you need exclusive access
        contract_specs.load_private_key_if_missing()? // requires &mut self
    };
    let wallet = SecretKeyRef::new(sk);

    let abi_json = include_str!("../../abi/Zaphenath.json");
    let contract = Contract::from_json(web3.eth(), contract_address, abi_json.as_bytes())?;

    let key_hash = H256::from_slice(web3::signing::keccak256(key_id.as_bytes()).as_slice());

    let data_bytes = Bytes::from(
        hex::decode(data).map_err(|_| anyhow::anyhow!("Invalid hex data in --data: {}", data))?,
    );

    let call_params = (key_hash, data_bytes, U256::from(timeout));

    let gas_to_use = if let Some(limit) = gas_specs.gas_limit {
        Some(U256::from(limit))
    } else if gas_specs.gas_buffer.is_none() && yes {
        None
    } else {
        let est: U256 = contract
            .estimate_gas(
                "createKey",
                call_params.clone(),
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
                "Estimated gas: {}\nBuffered gas limit: {}\nSend transaction?",
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
        .signed_call("createKey", call_params, options, wallet)
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
            println!("✅ Key created on-chain. Tx hash: {:?}", tx_hash);
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
