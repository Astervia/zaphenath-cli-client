pub mod logs;
pub mod run;
pub mod stop;
use crate::cmd::types::GasAndConfirmArgs;
use clap::Subcommand;

/// Background daemon for automated Zaph key management.
/// The daemon periodically pings configured keys to keep them active.
/// It supports running in the foreground or as a background process.
#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the Zaph daemon to periodically ping keys and maintain their activity.
    /// Can run in foreground (default) or background (detached) mode.
    Run {
        /// Interval (in seconds) between automatic ping attempts.
        /// Each cycle attempts to ping all keys listed in the local config file.
        #[arg(long, help_heading = "Timing")]
        interval: u64,

        /// Run the daemon in detached (background) mode.
        /// Writes a PID file and continues running independently.
        #[arg(short = 'd', long, help_heading = "Mode")]
        detached: bool,

        /// Optional override path to a specific config file.
        /// If not provided, the default config path is used.
        #[arg(long, help_heading = "Config")]
        config: Option<String>,

        /// Number of ping cycles to run before exiting.
        /// Useful for testing or one-off runs. If not provided, runs indefinitely.
        #[arg(long, help_heading = "Timing")]
        shots: Option<u64>,

        /// Gas price, nonce, confirmation flags, etc.
        /// These options control how transactions are submitted to the blockchain.
        #[command(flatten)]
        gas_confirm: GasAndConfirmArgs,
    },

    /// Gracefully stop the running daemon (only valid if started in detached mode).
    /// Reads the PID from `.zaphenathd.pid` and attempts to terminate the process.
    Stop,

    /// Show recent log output from the daemon.
    /// Useful for debugging ping cycles and transaction results.
    Logs,
}

pub async fn handle_daemon_command(action: &DaemonAction) {
    match action {
        DaemonAction::Run {
            interval,
            detached,
            config,
            gas_confirm,
            shots,
        } => {
            run::run_daemon(
                *interval,
                *detached,
                config.clone(),
                gas_confirm.clone(),
                shots,
                gas_confirm.nonce,
            )
            .await
        }

        DaemonAction::Stop => {
            stop::stop_daemon();
        }

        DaemonAction::Logs => {
            logs::show_logs();
        }
    }
}
