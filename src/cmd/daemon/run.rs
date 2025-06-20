use crate::{
    cmd::types::GasAndConfirmArgs,
    config::{get_config_path, read_config},
    contract::{
        ping::ping_key_on_chain,
        types::{ContractSpecs, GasSpecs, NetworkContext},
    },
};
use chrono::Utc;
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Command, Stdio, exit},
    thread,
    time::Duration,
};

/// Daemon entry point
pub async fn run_daemon(
    interval_secs: u64,
    detached: bool,
    config_override: Option<String>,
    gas: GasAndConfirmArgs,
    shots: &Option<u64>,
    nonce: Option<u64>,
) {
    if detached {
        detach_process(interval_secs, config_override, gas);
        return;
    }

    let config_path = config_override
        .map(PathBuf::from)
        .unwrap_or_else(get_config_path);

    let log_path = PathBuf::from(".zaphenathd.log");
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file");

    writeln!(
        log_file,
        "[{}] 🟢 Daemon started. Ping interval: {}s",
        Utc::now(),
        interval_secs
    )
    .unwrap();

    let mut current_shots = 0;
    loop {
        let now = Utc::now();
        writeln!(log_file, "[{}] 🔁 Starting ping cycle", now).unwrap();

        match read_config(&config_path) {
            Ok(config_value) => {
                if let Some(array) = config_value.as_array() {
                    for key_entry in array {
                        let key_id = match key_entry.get("key_id").and_then(Value::as_str) {
                            Some(v) => v,
                            None => {
                                writeln!(log_file, "⚠️ Missing key_id field, skipping entry")
                                    .unwrap();
                                continue;
                            }
                        };

                        let (contract_addr, priv_key_path, rpc_url, owner) = match (
                            key_entry.get("contract_address").and_then(Value::as_str),
                            key_entry.get("private_key_path").and_then(Value::as_str),
                            key_entry.get("rpc_url").and_then(Value::as_str),
                            key_entry.get("owner").and_then(Value::as_str),
                        ) {
                            (Some(c), Some(p), Some(r), Some(o)) => (c, p, r, o),
                            _ => {
                                writeln!(
                                    log_file,
                                    "⚠️ Incomplete key entry for {}, skipping",
                                    key_id
                                )
                                .unwrap();
                                continue;
                            }
                        };

                        let network = key_entry
                            .get("network")
                            .and_then(Value::as_str)
                            .map(str::to_string);

                        let mut specs = ContractSpecs {
                            ctx: NetworkContext {
                                rpc_url: rpc_url.to_string(),
                                network,
                            },
                            contract_addr: contract_addr.to_string(),
                            priv_key_path: priv_key_path.to_string(),
                            priv_key: None,
                        };

                        let result = ping_key_on_chain(
                            &mut specs,
                            key_id,
                            owner,
                            true, // Automatically confirm
                            GasSpecs {
                                gas_limit: gas.gas_limit,
                                gas_buffer: gas.gas_buffer,
                            },
                            nonce,
                        )
                        .await;

                        match result {
                            Ok(tx_hash) => writeln!(
                                log_file,
                                "[{}] ✅ Pinged key {} (tx: {:?})",
                                Utc::now(),
                                key_id,
                                tx_hash
                            )
                            .unwrap(),
                            Err(e) => writeln!(
                                log_file,
                                "[{}] ❌ Failed to ping key {}: {:?}",
                                Utc::now(),
                                key_id,
                                e
                            )
                            .unwrap(),
                        }

                        if let Ok(mut config) = read_config(&config_path) {
                            if let Some(array) = config.as_array_mut() {
                                if let Some(entry) =
                                    array.iter_mut().find(|e| e["key_id"] == key_id)
                                {
                                    entry["last_ping_timestamp"] = json!(Utc::now().timestamp());
                                }
                            }

                            // Persist updated config
                            let _ = crate::config::write_config(&config_path, &config);
                        }
                    }
                }
            }
            Err(e) => {
                writeln!(
                    log_file,
                    "[{}] ❌ Failed to read config: {:?}",
                    Utc::now(),
                    e
                )
                .unwrap();
            }
        }

        current_shots += 1;
        if let Some(some_shots) = shots {
            if current_shots >= *some_shots {
                break;
            }
        }

        log_file.flush().unwrap();
        thread::sleep(Duration::from_secs(interval_secs));
    }
}

/// Forks the daemon to run in the background and writes PID
fn detach_process(interval: u64, config_override: Option<String>, gas: GasAndConfirmArgs) {
    let mut args = vec![
        "daemon".to_string(),
        "run".to_string(),
        "--interval".to_string(),
        interval.to_string(),
    ];

    if let Some(path) = config_override {
        args.push("--config".to_string());
        args.push(path); // We own the String already
    }

    if let Some(limit) = gas.gas_limit {
        args.push("--gas-limit".to_string());
        args.push(limit.to_string());
    }

    if let Some(buffer) = gas.gas_buffer {
        args.push("--gas-buffer".to_string());
        args.push(buffer.to_string());
    }

    let self_exe = std::env::current_exe().expect("Failed to get current binary path");
    let child = Command::new(self_exe)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to fork daemon");

    let pid = child.id();
    let mut pid_file = File::create(".zaphenathd.pid").expect("Failed to write PID file");
    writeln!(pid_file, "{pid}").unwrap();

    println!("🚀 Daemon detached and running in background (PID: {pid})");
    exit(0);
}
