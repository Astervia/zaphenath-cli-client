use crate::{
    config::{add_key, get_config_path, view_config},
    contract::{
        network,
        types::{ContractSpecs, KeyData},
    },
};
use clap::Subcommand;

/// Actions for managing the local Zaphenath configuration file.
/// This file stores details about your keys, contract addresses, and network settings.
#[derive(Subcommand)]
pub enum ConfigAction {
    /// View the current content of the local Zaphenath configuration file.
    /// This will print the JSON representation of your configured keys and their details.
    View,
    /// Add a new key entry to the local configuration file.
    /// This command *only* updates your local config; it does not interact with the blockchain.
    /// To create a key on-chain, use `zaph contract create-key`.
    Add {
        /// A unique, human-readable identifier for this key in your local config.
        #[arg(long)]
        key_id: String,

        /// The Ethereum address of the deployed Zaphenath smart contract.
        #[arg(long)]
        contract_address: String,

        /// The file path to the private key of the owner of this key.
        /// The private key should be hex-encoded.
        #[arg(long)]
        private_key_path: String,

        /// The blockchain network associated with this key (e.g., "mainnet", "sepolia", "anvil").
        #[arg(long)]
        network: String,

        /// Optional: The RPC URL for connecting to the blockchain node.
        /// If not provided, a default URL for the specified network will be used.
        #[arg(long)]
        rpc_url: Option<String>,

        /// The default inactivity timeout in seconds for this key.
        /// This is the duration after which the key might become accessible if not pinged.
        #[arg(long, value_parser = clap::value_parser!(u64))]
        timeout: u64,

        /// Optional: The Ethereum address of the key's owner.
        /// If not provided, it will be derived from the private key at the given path.
        #[arg(long)]
        owner: Option<String>,
    },
    /// Initialize a new local Zaphenath configuration file.
    /// If the file already exists, this will fail unless `--force` is passed.
    Init {
        /// Overwrite the config file if it already exists
        #[arg(long)]
        force: bool,
    },
    /// Show the absolute path to the local Zaphenath configuration file.
    /// This is the path used by all CLI operations unless overridden by the
    /// ZAPHENATH_CONFIG_PATH environment variable.
    Path,
}

pub async fn handle_config_command(action: ConfigAction) {
    let path = get_config_path();
    match action {
        ConfigAction::View => view_config(&path),
        ConfigAction::Add {
            key_id,
            contract_address,
            private_key_path,
            timeout,
            network,
            rpc_url,
            owner,
        } => {
            let ctx =
                match network::build_network_context(rpc_url.as_deref(), Some(network.as_ref())) {
                    Ok(c) => c,
                    Err(_) => {
                        let e = anyhow::anyhow!("❌ Missing network or rpc-url");
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                };

            if let Err(e) = add_key(
                &path,
                &mut ContractSpecs {
                    ctx,
                    contract_addr: contract_address,
                    priv_key_path: private_key_path,
                    priv_key: None,
                },
                KeyData {
                    id: key_id,
                    timeout,
                    owner,
                },
            )
            .await
            {
                eprintln!("{e}");
                std::process::exit(1);
            };
        }
        ConfigAction::Init { force } => {
            use std::fs::{create_dir_all, write};
            use std::path::Path;

            let path = get_config_path();
            let parent = path.parent().unwrap_or_else(|| Path::new("."));

            if path.exists() && !force {
                eprintln!(
                    "⚠️ Config already exists at {:?}. Use --force to overwrite.",
                    path
                );
                std::process::exit(1);
            }

            if let Err(e) = create_dir_all(parent) {
                eprintln!("❌ Failed to create config directory: {e}");
                std::process::exit(1);
            }

            if let Err(e) = write(&path, "[]") {
                eprintln!("❌ Failed to write config file: {e}");
                std::process::exit(1);
            }

            println!("✅ Created new config at {:?}", path);
        }
        ConfigAction::Path => {
            println!("{}", path.display());
        }
    }
}
