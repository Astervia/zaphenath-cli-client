use std::{fs, str::FromStr};
use web3::signing::SecretKey;

#[derive(Debug)]
pub struct NetworkContext {
    pub rpc_url: String,
    pub network: Option<String>,
}

#[derive(Debug)]
pub struct ContractSpecs {
    pub ctx: NetworkContext,
    pub contract_addr: String,
    pub priv_key_path: String,
    pub priv_key: Option<SecretKey>, // Store the loaded private key here
}

#[derive(Debug)]
pub struct KeyData {
    pub id: String,
    pub owner: Option<String>,
    pub timeout: u64,
    // Add other fields as needed for config management, e.g., ping_interval, custodians
}

#[derive(Debug)]
pub struct CustodianData {
    pub role: Role,
    pub can_ping: bool,
    pub address: String,
}

#[derive(Debug)]
pub struct GasSpecs {
    pub gas_limit: Option<u64>,
    pub gas_buffer: Option<f64>,
}

impl ContractSpecs {
    pub fn load_private_key_if_missing(&mut self) -> Result<&SecretKey, anyhow::Error> {
        if self.priv_key.is_none() {
            let sk_bytes = fs::read_to_string(&self.priv_key_path)?
                .trim()
                .trim_start_matches("0x")
                .to_string();

            let sk = SecretKey::from_str(&sk_bytes).map_err(|_| {
                anyhow::anyhow!(
                    "Invalid private key format from path: {}",
                    self.priv_key_path
                )
            })?;
            self.priv_key = Some(sk);
        }
        Ok(self.priv_key.as_ref().unwrap())
    }

    pub fn get_private_key(&self) -> Option<&SecretKey> {
        self.priv_key.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // This ensures it's represented as a u8, matching Solidity
pub enum Role {
    Owner = 0,
    Writer = 1,
    Reader = 2,
    None = 3,
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "owner" => Ok(Role::Owner),
            "writer" => Ok(Role::Writer),
            "reader" => Ok(Role::Reader),
            "none" => Ok(Role::None),
            _ => Err(anyhow::anyhow!(
                "Invalid role: {}. Expected one of 'Owner', 'Writer', 'Reader', 'None'.",
                s
            )),
        }
    }
}

impl From<Role> for u8 {
    fn from(role: Role) -> u8 {
        role as u8
    }
}
