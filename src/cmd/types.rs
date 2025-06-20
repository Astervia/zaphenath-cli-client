/// Arguments for controlling transaction confirmation and gas parameters.
/// These arguments are commonly used for any command that sends an on-chain transaction.
#[derive(clap::Args, Clone)]
pub struct GasAndConfirmArgs {
    /// Skip confirmation prompt for sending transactions.
    /// Use this flag to proceed directly with sending the transaction without manual confirmation.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Manually set the gas limit for the transaction.
    /// If not provided, gas limit will be estimated.
    #[arg(long)]
    pub gas_limit: Option<u64>,

    /// Apply a buffer multiplier to the estimated gas limit.
    /// E.g., a value of 1.2 adds a 20% buffer to the estimated gas.
    /// This is useful to prevent transactions from failing due to slight gas estimation inaccuracies.
    #[arg(long)]
    pub gas_buffer: Option<f64>,

    /// Manually specify the account nonce. Useful for advanced scenarios like parallel transactions or resubmissions.
    /// Defaults to the next available nonce from the network.
    #[arg(long)]
    pub nonce: Option<u64>,
}

/// Arguments for specifying network connection details.
/// These arguments define which blockchain network and RPC endpoint the client should connect to.
#[derive(clap::Args, Clone)]
pub struct NetworkArgs {
    /// Specify the target blockchain network (e.g., "mainnet", "sepolia", "anvil").
    /// This can influence default RPC URLs and chain IDs.
    #[arg(long)]
    pub network: Option<String>,

    /// Directly provide the RPC URL for connecting to an Ethereum-compatible node.
    /// This overrides any default RPC URL derived from the '--network' argument.
    #[arg(long)]
    pub rpc_url: Option<String>,
}
