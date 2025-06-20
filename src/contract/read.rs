use crate::contract::types::ContractSpecs;
use std::str::FromStr;
use web3::signing::{Key, SecretKeyRef};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::{Address, Bytes, H256},
};

pub async fn read_key_on_chain(
    contract_specs: &mut ContractSpecs,
    key_id: &str,
    owner_address: &str,
) -> Result<Bytes, anyhow::Error> {
    let http = Http::new(&contract_specs.ctx.rpc_url)?;
    let web3 = web3::Web3::new(http);

    let contract_address = Address::from_str(&contract_specs.contract_addr)
        .map_err(|_| anyhow::anyhow!("Invalid contract address"))?;

    let abi_json = include_str!("../../abi/Zaphenath.json");
    let contract = Contract::from_json(web3.eth(), contract_address, abi_json.as_bytes())?;

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

    let wallet = SecretKeyRef::new(sk);
    let caller_addr: Address = wallet.address();

    let key_hash = H256::from_slice(web3::signing::keccak256(key_id.as_bytes()).as_slice());
    let owner = Address::from_str(owner_address)?;

    let options = Options {
        ..Options::default()
    };

    let result: Bytes = contract
        .query(
            "readKey",
            (key_hash, owner),
            Some(caller_addr),
            options,
            None,
        )
        .await?;

    Ok(result)
}
