use clap::{Parser, Subcommand};
mod cmd;
mod config;
mod contract;

#[derive(Parser)]
#[command(
    name = "zaph",
    version,
    about = "A CLI tool for interacting with the Zaphenath smart contract on Ethereum-compatible networks.",
    long_about = "Zaphenath enables secure, time-based data release powered by smart contracts. This CLI allows users to manage keys, set custodians, and interact with the contract's core functionalities.",
    author = "Ruy Vieira"
)]
struct Cli {
    /// Override the default configuration file path.
    /// By default, it's located in your OS's standard config directory (e.g., ~/.config/zaphenath/config.json on Linux).
    #[arg(long)]
    config: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage local configuration file.
    /// This includes viewing, initializing, or adding key entries to your local Zaphenath config.
    Config {
        #[command(subcommand)]
        action: cmd::config::ConfigAction,
    },
    /// Interact directly with the Zaphenath smart contract on-chain.
    /// Commands under this subcommand perform blockchain transactions like creating, updating, or deleting keys,
    /// as well as managing custodians.
    Contract {
        #[command(subcommand)]
        action: cmd::contract::ContractAction,
    },
    /// Run background daemon tasks for automated operations.
    /// The daemon can periodically perform actions like pinging keys to keep them active.
    Daemon {
        #[command(subcommand)]
        action: cmd::daemon::DaemonAction, // Use the DaemonAction from the new module
    },
}

#[tokio::main]
async fn main() {
    // This allows you to control logging verbosity via the RUST_LOG environment variable.
    // E.g., RUST_LOG=info cargo run
    // RUST_LOG=debug cargo run
    // RUST_LOG=zaph=info,web3=warn cargo run (for specific module logging)
    env_logger::init();

    let cli = Cli::parse();

    if let Some(custom_path) = cli.config {
        // SAFETY: This is generally safe as we are setting an environment variable
        // which affects the process's own environment.
        unsafe { std::env::set_var("ZAPHENATH_CONFIG_PATH", custom_path) };
    }

    match cli.command {
        Commands::Config { action } => {
            cmd::config::handle_config_command(action).await;
        }
        Commands::Contract { action } => {
            cmd::contract::handle_contract_command(&action).await;
        }
        Commands::Daemon { action } => {
            cmd::daemon::handle_daemon_command(&action).await;
        }
    }
}
