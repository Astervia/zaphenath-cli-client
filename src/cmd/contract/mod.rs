mod create_key;
mod delete_key;
mod ping;
mod read;
mod remove_custodian;
mod set_custodian;
mod update;
use super::types::{GasAndConfirmArgs, NetworkArgs};
use crate::contract::{network, types::ContractSpecs};
use clap::Subcommand;

/// Available actions for interacting with the Zaphenath smart contract.
/// These commands enable key management (create, update, delete), liveness signaling (ping),
/// data retrieval (read), and access control management (set/remove custodian).
#[derive(Subcommand)]
pub enum ContractAction {
    /// Create a new key on the Zaphenath contract.
    /// This command registers a new encrypted data entry with an owner and a timeout.
    CreateKey {
        /// Unique identifier for the key to be created.
        /// This ID will be hashed on-chain to identify the key.
        #[arg(long)]
        key_id: String,

        /// The encrypted data (hex-encoded bytes) to associate with the key.
        /// This data becomes readable by custodians after the timeout.
        #[arg(long)]
        data: String, // Encrypted string data (hex-encoded)

        /// The timeout duration in seconds.
        /// If no ping is received within this period, the key becomes accessible to custodians.
        #[arg(long)]
        timeout: u64,

        /// The address of the deployed Zaphenath smart contract.
        #[arg(long)]
        contract_address: String,

        /// Path to the file containing the private key (hex-encoded) of the key's owner.
        /// This key is used to sign the transaction.
        #[arg(long)]
        private_key_path: String,

        /// (Internal) Skips actual on-chain interaction, useful for testing.
        #[arg(long, hide = true)]
        mock: bool,

        /// Network configuration arguments (RPC URL, network name).
        #[command(flatten)]
        network_specs: NetworkArgs,

        /// Gas and confirmation control arguments.
        #[command(flatten)]
        gas_confirm: GasAndConfirmArgs,
    },

    /// Delete an existing key from the Zaphenath contract.
    /// This operation removes the key and its associated data from the contract.
    DeleteKey {
        /// The ID of the key to delete.
        #[arg(long)]
        key_id: String,

        /// Gas and confirmation control arguments.
        #[command(flatten)]
        gas_confirm: GasAndConfirmArgs,
    },

    /// Ping an existing key to reset its inactivity timeout.
    /// Regular pings keep the key's data private until the owner becomes inactive.
    PingKey {
        /// The ID of the key to ping.
        #[arg(long)]
        key_id: String,

        /// (Internal) Skips actual on-chain interaction, useful for testing.
        #[arg(long, hide = true)]
        mock: bool,

        /// Gas and confirmation control arguments.
        #[command(flatten)]
        gas_confirm: GasAndConfirmArgs,
    },

    /// Read the data associated with a key.
    /// Data is only readable by the owner or authorized custodians after timeout.
    ReadKey {
        /// The ID of the key to read.
        #[arg(long)]
        key_id: String,

        /// Attempt to decode the output bytes as a UTF-8 string for human readability.
        /// If the data is not valid UTF-8, it will be printed as raw bytes.
        #[arg(long)]
        decode: bool,
    },

    /// Update the data and/or timeout for an existing key.
    /// Only the key's owner or an authorized writer can perform this action.
    UpdateKey {
        /// The ID of the key to update.
        #[arg(long)]
        key_id: String,

        /// The new encrypted data (hex-encoded bytes) to set for the key.
        #[arg(long)]
        data: String,

        /// The new timeout duration in seconds for the key.
        #[arg(long)]
        timeout: u64,

        /// (Internal) Skips actual on-chain interaction, useful for testing.
        #[arg(long, hide = true)]
        mock: bool,

        /// Gas and confirmation control arguments.
        #[command(flatten)]
        gas_confirm: GasAndConfirmArgs,
    },

    /// Set or update access permissions for a custodian on a specific key.
    /// Custodians are external users who gain access to the key's data under defined conditions.
    SetCustodian {
        /// The ID of the key for which to set the custodian.
        #[arg(long)]
        key_id: String,

        /// The Ethereum address of the user to set as a custodian.
        #[arg(long)]
        user_address: String,

        /// The role to assign to the custodian (Owner, Writer, Reader, None).
        /// - Owner: Full control (same as key creator).
        /// - Writer: Can update key data/timeout.
        /// - Reader: Can read key data after timeout.
        /// - None: No specific role, but can be configured for ping-only.
        #[arg(long, help = "Role for the custodian (Owner, Writer, Reader, None)")]
        role: String, // Will be parsed into Role enum

        /// Flag indicating if the custodian is allowed to ping the key.
        /// If true, the custodian can reset the key's inactivity timeout.
        #[arg(long)]
        can_ping: bool,

        /// Gas and confirmation control arguments.
        #[command(flatten)]
        gas_confirm: GasAndConfirmArgs,
    },

    /// Remove a custodian's access permissions from a key.
    /// This revokes any previously assigned roles and ping rights for the specified user.
    RemoveCustodian {
        /// The ID of the key from which to remove the custodian.
        #[arg(long)]
        key_id: String,

        /// The Ethereum address of the custodian to remove.
        #[arg(long)]
        user_address: String,

        /// Gas and confirmation control arguments.
        #[command(flatten)]
        gas_confirm: GasAndConfirmArgs,
    },
}

pub async fn handle_contract_command(action: &ContractAction) {
    match action {
        ContractAction::CreateKey {
            key_id,
            data,
            timeout,
            network_specs,
            contract_address,
            private_key_path,
            mock,
            gas_confirm,
        } => {
            let ctx = match network::build_network_context(
                network_specs.rpc_url.as_deref(),
                network_specs.network.as_deref(),
            ) {
                Ok(c) => c,
                Err(_) => {
                    let e = anyhow::anyhow!("❌ Missing network or rpc-url");
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            };

            if let Err(e) = create_key::handle_create_key(
                key_id,
                data,
                *timeout,
                &mut ContractSpecs {
                    ctx,
                    contract_addr: contract_address.to_string(),
                    priv_key_path: private_key_path.to_string(),
                    priv_key: None,
                },
                *mock,
                gas_confirm,
            )
            .await
            {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }

        ContractAction::DeleteKey {
            key_id,
            gas_confirm,
        } => {
            if let Err(e) = delete_key::handle_delete_key(key_id, gas_confirm).await {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }

        ContractAction::PingKey {
            key_id,
            mock,
            gas_confirm,
        } => {
            if let Err(e) = ping::handle_ping_key(key_id, *mock, gas_confirm).await {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }

        ContractAction::ReadKey { key_id, decode } => {
            if let Err(e) = read::handle_read_key(key_id, *decode).await {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }

        ContractAction::UpdateKey {
            key_id,
            data,
            timeout,
            mock,
            gas_confirm,
        } => {
            if let Err(e) =
                update::handle_update_key(key_id, data, *timeout, *mock, gas_confirm).await
            {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }

        ContractAction::SetCustodian {
            key_id,
            user_address,
            role,
            can_ping,
            gas_confirm,
        } => {
            if let Err(e) = set_custodian::handle_set_custodian(
                key_id,
                user_address,
                role, // Pass the role as a string, it will be parsed in handle_set_custodian
                *can_ping,
                gas_confirm,
            )
            .await
            {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }

        ContractAction::RemoveCustodian {
            key_id,
            user_address,
            gas_confirm,
        } => {
            if let Err(e) =
                remove_custodian::handle_remove_custodian(key_id, user_address, gas_confirm).await
            {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
